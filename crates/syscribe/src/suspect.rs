//! `syscribe suspect` — content-baseline suspect-link management
//! (§SuspectLinks; REQ-TRS-SUS-LINKS-005/006).
//!
//! `suspect list`   — report suspect and unbaselined trace links (read-only).
//! `suspect accept` — capture/refresh a target's projection hash into the
//!                    source's `traceBaselines` map, clearing the suspect flag.
//!
//! The detection/projection logic lives in `syscribe_model::suspect`; this module
//! owns the CLI surface and the surgical file write-back.

use std::collections::BTreeMap;
use std::path::Path;

use syscribe_model::element::RawElement;
use syscribe_model::frontmatter::{parse_frontmatter, split_frontmatter};
use syscribe_model::resolver::Resolver;
use syscribe_model::suspect::{self, LinkState, SuspectLink};

/// Dispatch `suspect <sub> ...`. Returns a process exit code (0 = success).
pub fn cmd_suspect(
    elems: &[RawElement],
    resolver: &Resolver,
    model_root: &Path,
    sub: &str,
    rest: &[String],
) -> i32 {
    match sub {
        "list" => cmd_list(elems, resolver),
        "accept" => cmd_accept(elems, resolver, model_root, rest),
        "" => {
            eprintln!("Usage: syscribe -m <root> suspect <list|accept> ...");
            1
        }
        other => {
            eprintln!("Unknown `suspect` subcommand: {other}");
            eprintln!("Usage: syscribe -m <root> suspect <list|accept> ...");
            1
        }
    }
}

/// `suspect list` — deterministic, read-only report of suspect + unbaselined links.
fn cmd_list(elems: &[RawElement], resolver: &Resolver) -> i32 {
    let links = suspect::scan(elems, resolver);

    let suspect: Vec<&SuspectLink> =
        links.iter().filter(|l| l.state == LinkState::Suspect).collect();
    let unbaselined: Vec<&SuspectLink> =
        links.iter().filter(|l| l.state == LinkState::Unbaselined).collect();

    println!("# Suspect links ({})", suspect.len());
    if suspect.is_empty() {
        println!("  (none)");
    } else {
        for l in &suspect {
            println!("  SUSPECT  {} {} {}", l.source_label(), l.kind, l.target_ref);
        }
    }

    println!("\n# Unbaselined links ({})", unbaselined.len());
    if unbaselined.is_empty() {
        println!("  (none)");
    } else {
        for l in &unbaselined {
            println!("  no-baseline  {} {} {}", l.source_label(), l.kind, l.target_ref);
        }
    }

    0
}

/// A resolved single-link baseline write: which source file to edit, the authored
/// target ref used as the `traceBaselines` key, and the target's current hash.
pub struct AcceptPlan {
    pub source_file: String,
    pub authored_key: String,
    pub hash: String,
}

/// Resolve a `(source, target)` pair to a concrete baseline write. Shared by the
/// CLI `suspect accept` and the MCP `suspect_accept` tool so both surfaces agree
/// on resolution, the stored key, and the hash. `source`/`target` accept a stable
/// id or a qualified name. Errors (as human-readable strings) when the source does
/// not resolve, the target is not referenced by any trace link on the source, or
/// the target does not resolve.
pub fn plan_accept(
    elems: &[RawElement],
    resolver: &Resolver,
    source_ref: &str,
    target_ref: &str,
) -> Result<AcceptPlan, String> {
    let source = suspect::resolve_target(elems, resolver, source_ref)
        .ok_or_else(|| format!("source `{source_ref}` does not resolve to any element"))?;

    // The `traceBaselines` key must be the ref exactly as authored on the link, so
    // validation looks it up identically. Match the requested target either by the
    // exact authored string or by resolving to the same element.
    let target_elem = suspect::resolve_target(elems, resolver, target_ref);
    let authored_key = suspect::trace_links(&source.frontmatter)
        .into_iter()
        .map(|(_, r)| r)
        .find(|r| {
            r == target_ref
                || (target_elem.is_some()
                    && suspect::resolve_target(elems, resolver, r).map(|e| &e.qualified_name)
                        == target_elem.map(|e| &e.qualified_name))
        })
        .ok_or_else(|| {
            format!(
                "`{target_ref}` is not referenced by any trace link on `{source_ref}` — nothing to baseline"
            )
        })?;

    let tgt = suspect::resolve_target(elems, resolver, &authored_key)
        .ok_or_else(|| format!("target `{authored_key}` does not resolve to any element"))?;
    let hash = suspect::projection_hash(tgt);

    Ok(AcceptPlan { source_file: source.file_path.clone(), authored_key, hash })
}

/// `suspect accept <source> <target>` or `suspect accept --all[-unbaselined]`.
fn cmd_accept(
    elems: &[RawElement],
    resolver: &Resolver,
    _model_root: &Path,
    rest: &[String],
) -> i32 {
    let all = rest.iter().any(|a| a == "--all");
    let all_unbaselined = rest.iter().any(|a| a == "--all-unbaselined");
    if all && all_unbaselined {
        eprintln!("error: --all and --all-unbaselined are mutually exclusive");
        return 1;
    }
    if all {
        return accept_all(elems, resolver);
    }
    if all_unbaselined {
        return accept_all_unbaselined(elems, resolver);
    }

    let positionals: Vec<&str> =
        rest.iter().filter(|a| !a.starts_with("--")).map(|s| s.as_str()).collect();
    if positionals.len() != 2 {
        eprintln!("Usage: syscribe -m <root> suspect accept <source> <target>");
        eprintln!("       syscribe -m <root> suspect accept --all");
        eprintln!("       syscribe -m <root> suspect accept --all-unbaselined");
        return 1;
    }
    let (source_ref, target_ref) = (positionals[0], positionals[1]);

    let plan = match plan_accept(elems, resolver, source_ref, target_ref) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    match write_baseline(Path::new(&plan.source_file), &plan.authored_key, &plan.hash) {
        Ok(()) => {
            println!("baselined {} → {} ({})", plan.source_file, plan.authored_key, plan.hash);
            0
        }
        Err(e) => {
            eprintln!("error: failed to write baseline to {}: {e}", plan.source_file);
            1
        }
    }
}

/// `suspect accept --all` — re-baseline every currently-suspect link. Groups the
/// writes per source file so a source with several suspect links is written once.
fn accept_all(elems: &[RawElement], resolver: &Resolver) -> i32 {
    let links = suspect::scan(elems, resolver);
    let suspect: Vec<&SuspectLink> =
        links.iter().filter(|l| l.state == LinkState::Suspect).collect();

    if suspect.is_empty() {
        println!("no suspect links to accept");
        return 0;
    }

    // (source_file) -> [(target_ref, new_hash)]
    let mut by_file: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for l in &suspect {
        if let Some(hash) = &l.current {
            by_file
                .entry(l.source_file.clone())
                .or_default()
                .push((l.target_ref.clone(), hash.clone()));
        }
    }

    let mut count = 0usize;
    for (file, entries) in &by_file {
        for (target, hash) in entries {
            if let Err(e) = write_baseline(Path::new(file), target, hash) {
                eprintln!("error: failed to write baseline to {file}: {e}");
                return 1;
            }
            count += 1;
        }
    }
    println!("re-baselined {count} suspect link(s)");
    0
}

/// `suspect accept --all-unbaselined` — onboarding: baseline every link that has
/// no baseline yet and whose target resolves (REQ-TRS-SUS-LINKS-008). It never
/// touches an existing baseline, so it cannot silently clear an outstanding
/// suspect flag; it is idempotent (a second run finds nothing unbaselined).
fn accept_all_unbaselined(elems: &[RawElement], resolver: &Resolver) -> i32 {
    let links = suspect::scan(elems, resolver);
    // Unbaselined links with a resolvable target carry a computed `current` hash;
    // unresolvable targets have `current == None` and are skipped.
    let onboard: Vec<&SuspectLink> = links
        .iter()
        .filter(|l| l.state == LinkState::Unbaselined && l.current.is_some())
        .collect();

    if onboard.is_empty() {
        println!("no unbaselined links to onboard");
        return 0;
    }

    // (source_file) -> [(target_ref, new_hash)]
    let mut by_file: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for l in &onboard {
        if let Some(hash) = &l.current {
            by_file
                .entry(l.source_file.clone())
                .or_default()
                .push((l.target_ref.clone(), hash.clone()));
        }
    }

    let mut count = 0usize;
    for (file, entries) in &by_file {
        for (target, hash) in entries {
            if let Err(e) = write_baseline(Path::new(file), target, hash) {
                eprintln!("error: failed to write baseline to {file}: {e}");
                return 1;
            }
            count += 1;
        }
    }
    println!("baselined {count} previously-unbaselined link(s)");
    0
}

/// Write (create or overwrite) a single `traceBaselines` entry into a source
/// file's frontmatter (REQ-TRS-SUS-LINKS-005).
///
/// Prefers a **surgical** edit: only the `traceBaselines` block is added or
/// patched; every other byte — field order, quoting, list indentation, the body —
/// is preserved, so re-baselining produces a minimal diff (the ADR's diff-churn
/// goal). Falls back to a whole-frontmatter YAML round-trip only when the surgical
/// edit can't be applied safely (e.g. an inline `traceBaselines: {...}` or CRLF).
pub fn write_baseline(file: &Path, target_key: &str, hash: &str) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(file)?;
    let new_content = match surgical_insert(&content, target_key, hash) {
        Some(c) if baseline_present(&c, target_key, hash) => c,
        _ => roundtrip_insert(&content, target_key, hash)?,
    };
    std::fs::write(file, new_content)?;
    Ok(())
}

/// True if `content`'s frontmatter parses and carries `target_key → hash`.
fn baseline_present(content: &str, target_key: &str, hash: &str) -> bool {
    let (fm_opt, _) = split_frontmatter(content);
    match fm_opt.and_then(|fm| parse_frontmatter(fm).ok()) {
        Some(fm) => fm
            .trace_baselines
            .as_ref()
            .and_then(|m| m.get(target_key))
            .map(|v| v == hash)
            .unwrap_or(false),
        None => false,
    }
}

/// Surgically add/patch the `traceBaselines` entry in the frontmatter text,
/// re-splicing it into `content` by byte offset so delimiters and the body are
/// byte-identical. Returns `None` when it declines (no frontmatter, CRLF, or an
/// inline `traceBaselines`), signalling the caller to fall back.
fn surgical_insert(content: &str, key: &str, hash: &str) -> Option<String> {
    let (fm_opt, _body) = split_frontmatter(content);
    let fm = fm_opt?;
    if fm.contains('\r') {
        return None; // CRLF: let the round-trip normalize it
    }
    let new_fm = edit_frontmatter_text(fm, key, hash)?;

    // Re-splice by byte offset: fm is a sub-slice of content, so the opener,
    // closing delimiter, and body around it are preserved exactly.
    let base = content.as_ptr() as usize;
    let fm_start = fm.as_ptr() as usize - base;
    let fm_end = fm_start + fm.len();
    let mut out = String::with_capacity(content.len() + new_fm.len());
    out.push_str(&content[..fm_start]);
    out.push_str(&new_fm);
    out.push_str(&content[fm_end..]);
    Some(out)
}

/// Edit the frontmatter YAML *text* (no reserialization). Appends a
/// `traceBaselines` block if absent, patches the target's line if present, or adds
/// a new child line under an existing block. `None` if an inline `traceBaselines`
/// mapping is detected (unsafe to text-edit).
fn edit_frontmatter_text(fm: &str, key: &str, hash: &str) -> Option<String> {
    let value = format!("\"{hash}\"");
    let mut lines: Vec<String> = fm.split('\n').map(str::to_string).collect();

    // A top-level `traceBaselines:` block header is the exact line (no indent, no
    // inline content). Anything else (e.g. `traceBaselines: {}`) → decline.
    let header = lines.iter().position(|l| l == "traceBaselines:");
    if header.is_none() && lines.iter().any(|l| l.trim_start().starts_with("traceBaselines:")) {
        return None;
    }

    match header {
        None => {
            // Append a fresh block. `fm` carries no trailing newline.
            Some(format!("{fm}\ntraceBaselines:\n  {key}: {value}"))
        }
        Some(idx) => {
            let mut child_indent: Option<String> = None;
            let mut key_line: Option<usize> = None;
            for i in (idx + 1)..lines.len() {
                let l = &lines[i];
                let indent_len = l.len() - l.trim_start().len();
                if l.trim().is_empty() {
                    continue;
                }
                if indent_len == 0 {
                    break; // dedent to top level → end of the block
                }
                if child_indent.is_none() {
                    child_indent = Some(l[..indent_len].to_string());
                }
                let t = l.trim_start();
                if let Some(colon) = t.find(':') {
                    if t[..colon].trim() == key {
                        key_line = Some(i);
                        break;
                    }
                }
            }
            let indent = child_indent.unwrap_or_else(|| "  ".to_string());
            match key_line {
                Some(i) => lines[i] = format!("{indent}{key}: {value}"),
                None => lines.insert(idx + 1, format!("{indent}{key}: {value}")),
            }
            Some(lines.join("\n"))
        }
    }
}

/// Fallback writer: round-trip the whole frontmatter through an order-preserving
/// YAML value. Reformats cosmetically but is always correct.
fn roundtrip_insert(content: &str, target_key: &str, hash: &str) -> anyhow::Result<String> {
    let (fm_opt, body) = split_frontmatter(content);
    let fm = fm_opt.ok_or_else(|| anyhow::anyhow!("no frontmatter block"))?;

    let mut doc: serde_yaml::Value = serde_yaml::from_str(fm)?;
    let map = doc
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("frontmatter is not a mapping"))?;

    let tb_key = serde_yaml::Value::String("traceBaselines".to_string());
    let entry = map
        .entry(tb_key)
        .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
    let tb = entry
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("traceBaselines is not a mapping"))?;
    tb.insert(
        serde_yaml::Value::String(target_key.to_string()),
        serde_yaml::Value::String(hash.to_string()),
    );

    let new_fm = serde_yaml::to_string(&doc)?; // ends with a trailing newline
    Ok(format!("---\n{new_fm}---\n\n{body}"))
}

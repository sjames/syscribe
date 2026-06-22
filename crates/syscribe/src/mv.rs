//! `syscribe move <source> <dest>` — relocate an element or package to a new
//! qualified name and rewrite every qualified-name reference to it, atomically.
//!
//! - `source` is an element (resolved by qualified name or stable id) or a
//!   package (a namespace directory).
//! - `dest` is the new fully-qualified name. The `.md` file (element) or the
//!   directory (package, with its whole subtree) is relocated to the path
//!   derived from `dest`.
//! - References are rewritten textually within frontmatter (formatting-preserving):
//!   a YAML walk identifies which scalar values are references to `source`
//!   (value equal to it, or beginning with `source::` for descendants/sub-features),
//!   then those exact qualified-name tokens are replaced. Stable ids (`REQ-*`,
//!   `TC-*`, …) are never touched.
//! - All planned writes are computed first; on any failure the original state is
//!   restored (all-or-nothing).

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use syscribe_model::element::RawElement;
use syscribe_model::frontmatter::split_frontmatter;
use syscribe_model::resolver::Resolver;

/// Free-text frontmatter keys whose scalar values are never qualified-name
/// references (so a display name equal to a qname is not rewritten).
const NON_REF_KEYS: &[&str] = &["name", "title", "shortName", "short_name", "label"];

/// True when `q` is a syntactically valid qualified name (`Seg(::Seg)*`).
fn valid_qname(q: &str) -> bool {
    !q.is_empty()
        && q.split("::").all(|seg| {
            !seg.is_empty()
                && seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

/// If `tok` is the moved qname or one of its descendants, return its rewritten form.
fn rewrite_qname(tok: &str, old: &str, new: &str) -> Option<String> {
    if tok == old {
        Some(new.to_string())
    } else {
        tok.strip_prefix(old)
            .filter(|rest| rest.starts_with("::"))
            .map(|rest| format!("{}{}", new, rest))
    }
}

/// Walk a YAML value collecting the exact reference scalars that must change.
fn collect_refs(
    val: &serde_yaml::Value,
    key_ctx: Option<&str>,
    old: &str,
    new: &str,
    out: &mut BTreeMap<String, String>,
) {
    match val {
        serde_yaml::Value::String(s) => {
            if let Some(k) = key_ctx {
                if NON_REF_KEYS.contains(&k) {
                    return;
                }
            }
            if let Some(rewritten) = rewrite_qname(s, old, new) {
                out.insert(s.clone(), rewritten);
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                collect_refs(item, key_ctx, old, new, out);
            }
        }
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                collect_refs(v, k.as_str(), old, new, out);
            }
        }
        _ => {}
    }
}

/// Replace every whole qualified-name token `old` with `new` in `text`.
/// A token boundary is any character that is not part of a qualified name
/// (`[A-Za-z0-9_:]`), so `Pkg::Widget` does not match inside `Pkg::WidgetExtended`.
fn replace_whole_token(text: &str, old: &str, new: &str) -> String {
    if old.is_empty() {
        return text.to_string();
    }
    let is_q = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == ':';
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find(old) {
        let before = &rest[..pos];
        let after = &rest[pos + old.len()..];
        let prev_ok = before.chars().next_back().is_none_or(|c| !is_q(c));
        let next_ok = after.chars().next().is_none_or(|c| !is_q(c));
        out.push_str(before);
        out.push_str(if prev_ok && next_ok { new } else { old });
        rest = after;
    }
    out.push_str(rest);
    out
}

/// Rewrite the YAML frontmatter text, returning the new text if anything changed.
/// Uses a YAML walk (denylist-aware) to find exactly which scalar values are
/// references, then replaces those qualified-name tokens textually.
fn rewrite_frontmatter(fm: &str, old: &str, new: &str) -> Option<String> {
    let yaml: serde_yaml::Value = serde_yaml::from_str(fm).ok()?;
    let mut refs = BTreeMap::new();
    collect_refs(&yaml, None, old, new, &mut refs);
    if refs.is_empty() {
        return None;
    }
    // Longest first so substring qnames never interfere (boundary checks also guard).
    let mut keys: Vec<&String> = refs.keys().collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    let mut out = fm.to_string();
    for k in keys {
        out = replace_whole_token(&out, k, &refs[k]);
    }
    (out != fm).then_some(out)
}

/// Rewrite qualified-name references in free text (a Markdown body or an SVG
/// file). Only **multi-segment** qname tokens (containing `::`) are considered,
/// so genuine references — `` `Pkg::Sub::Widget` `` in prose, descendant
/// endpoints, and SVG attributes like `sysml:ref="Pkg::Sub::Widget"` /
/// `data-qname=` / `href=".../Pkg::Sub::Widget"` — are updated while ordinary
/// words and sanitized ids are never touched. The `rewrite_qname` boundary rules
/// ensure a prefix-sharing sibling is left alone.
fn rewrite_qname_text(text: &str, old: &str, new: &str) -> Option<String> {
    // A qname token with at least one `::` separator.
    let re = regex::Regex::new(r"[A-Za-z0-9_]+(?:::[A-Za-z0-9_]+)+").unwrap();
    let out = re.replace_all(text, |caps: &regex::Captures| {
        let tok = &caps[0];
        rewrite_qname(tok, old, new).unwrap_or_else(|| tok.to_string())
    });
    (out != text).then(|| out.into_owned())
}

/// Compute the new content for `path` if it references `old`; `None` if unchanged.
/// Rewrites both frontmatter (structural references) and Markdown body
/// (qualified-name mentions), preserving every other byte.
fn rewrite_file(path: &Path, old: &str, new: &str) -> Option<(String, String)> {
    let content = std::fs::read_to_string(path).ok()?;
    let (fm_opt, body) = split_frontmatter(&content);

    let new_fm = fm_opt.and_then(|fm| rewrite_frontmatter(fm, old, new));
    let new_body = rewrite_qname_text(body, old, new);

    if new_fm.is_none() && new_body.is_none() {
        return None;
    }

    // Reassemble using byte offsets into the original content, so untouched
    // regions (delimiters, BOM, spacing) are preserved exactly.
    let base = content.as_ptr() as usize;
    let body_start = body.as_ptr() as usize - base;
    let body_end = body_start + body.len();
    let body_final = new_body.as_deref().unwrap_or(body);

    let result = match fm_opt {
        Some(fm) => {
            let fm_start = fm.as_ptr() as usize - base;
            let fm_end = fm_start + fm.len();
            let fm_final = new_fm.as_deref().unwrap_or(fm);
            format!(
                "{}{}{}{}{}",
                &content[..fm_start],
                fm_final,
                &content[fm_end..body_start],
                body_final,
                &content[body_end..]
            )
        }
        None => format!("{}{}{}", &content[..body_start], body_final, &content[body_end..]),
    };
    Some((content, result))
}

/// `move` subcommand entry point.
pub fn cmd_move(
    model_root: &Path,
    elements: &[RawElement],
    resolver: &Resolver,
    source_key: &str,
    dest: &str,
    dry_run: bool,
) {
    // ── Resolve the source (element id/qname, or an implicit package directory) ──
    let (old, src_file): (String, Option<PathBuf>) =
        match resolver.resolve_ref(elements, source_key) {
            Some(e) => (e.qualified_name.clone(), Some(PathBuf::from(&e.file_path))),
            None => {
                let norm = source_key.replace('/', "::");
                let dir = model_root.join(norm.replace("::", "/"));
                if dir.is_dir() {
                    (norm, None)
                } else {
                    eprintln!("Source not found: {source_key}");
                    std::process::exit(1);
                }
            }
        };

    let new = dest.replace('/', "::");

    // ── Validate destination ────────────────────────────────────────────────
    if !valid_qname(&new) {
        eprintln!("Invalid destination qualified name: '{new}'");
        std::process::exit(1);
    }
    if new == old {
        eprintln!("Destination equals source ('{old}') — nothing to do.");
        std::process::exit(1);
    }
    if new.starts_with(&format!("{old}::")) {
        eprintln!("Cannot move '{old}' into its own subtree ('{new}').");
        std::process::exit(1);
    }

    // ── Determine filesystem source/target (file vs package directory) ────────
    let dir_path = model_root.join(old.replace("::", "/"));
    let is_pkg = dir_path.is_dir();
    let old_fs = if is_pkg {
        dir_path
    } else {
        match src_file {
            Some(ref p) => p.clone(),
            None => {
                eprintln!("Source '{old}' has no file to move.");
                std::process::exit(1);
            }
        }
    };
    let new_fs = if is_pkg {
        model_root.join(new.replace("::", "/"))
    } else {
        model_root.join(format!("{}.md", new.replace("::", "/")))
    };

    if new_fs.exists() {
        eprintln!("Destination already exists: {}", new_fs.display());
        std::process::exit(1);
    }

    // ── Compute every planned reference-rewrite (across all model files) ───────
    let mut edits: Vec<(PathBuf, String, String)> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    // Element (.md) files: frontmatter (structural) + body (incl. inline SVG).
    for e in elements {
        let p = PathBuf::from(&e.file_path);
        if !seen.insert(p.clone()) {
            continue;
        }
        if let Some((orig, updated)) = rewrite_file(&p, &old, &new) {
            edits.push((p, orig, updated));
        }
    }
    // Companion SVG files: rewrite qname references in `sysml:ref` / `data-qname`
    // / `href` attributes and the like.
    for entry in walkdir::WalkDir::new(model_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let p = entry.path();
        if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("svg")) {
            let pb = p.to_path_buf();
            if !seen.insert(pb.clone()) {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(p) {
                if let Some(updated) = rewrite_qname_text(&content, &old, &new) {
                    edits.push((pb, content, updated));
                }
            }
        }
    }

    let kind = if is_pkg { "package" } else { "element" };

    if dry_run {
        println!("[dry-run] move {kind} {old} -> {new}");
        println!("[dry-run]   relocate {} -> {}", old_fs.display(), new_fs.display());
        if edits.is_empty() {
            println!("[dry-run]   no reference updates needed");
        } else {
            for (p, _, _) in &edits {
                println!("[dry-run]   update references in {}", p.display());
            }
        }
        return;
    }

    // ── Apply atomically: write edits, then relocate; roll back on any error ──
    let mut backups: Vec<(PathBuf, String)> = Vec::new();
    for (p, orig, updated) in &edits {
        backups.push((p.clone(), orig.clone()));
        if let Err(e) = std::fs::write(p, updated) {
            eprintln!("Write failed for {} ({e}); rolling back.", p.display());
            rollback(&backups);
            std::process::exit(1);
        }
    }

    if let Some(parent) = new_fs.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("Cannot create destination directory {} ({e}); rolling back.", parent.display());
            rollback(&backups);
            std::process::exit(1);
        }
    }

    if let Err(e) = std::fs::rename(&old_fs, &new_fs) {
        eprintln!("Relocation failed ({e}); rolling back.");
        rollback(&backups);
        std::process::exit(1);
    }

    println!("Moved {kind} {old} -> {new}");
    println!("  {} -> {}", old_fs.display(), new_fs.display());
    println!("  updated references in {} file(s)", edits.len());
}

/// Restore original file contents recorded before any write.
fn rollback(backups: &[(PathBuf, String)]) {
    for (p, orig) in backups {
        let _ = std::fs::write(p, orig);
    }
}

//! `syscribe baseline` — release-baseline management (ADR-SYS-BASELINE-001;
//! REQ-TRS-BL-004/006/007/008).
//!
//! `create` seals a scope; `verify` proves a baseline; `diff` compares two;
//! `list`/`show` inventory. The seal/scope/manifest engine lives in
//! `syscribe_model::baseline`; this module owns the CLI surface and git anchoring.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use syscribe_model::baseline::{self, aggregate, element_key, manifest_path, Manifest};
use syscribe_model::config::{detect_git_root, git_output, load_baseline_config, ValidateConfig};
use syscribe_model::element::{BaselineSeal, FrozenScope, RawElement};
use syscribe_model::resolver::Resolver;
use syscribe_model::suspect::{self, LinkState};
use syscribe_model::validator;

/// Dispatch `baseline <sub> ...`. Returns a process exit code.
pub fn cmd_baseline(
    elems: &[RawElement],
    resolver: &Resolver,
    model_root: &Path,
    sub: &str,
    rest: &[String],
) -> i32 {
    match sub {
        "create" => cmd_create(elems, resolver, model_root, rest),
        "verify" => cmd_verify(elems, resolver, model_root, rest),
        "diff" => cmd_diff(elems, resolver, model_root, rest),
        "list" => cmd_list(elems),
        "show" => cmd_show(elems, resolver, model_root, rest),
        _ => {
            eprintln!("Usage: syscribe -m <root> baseline <create|verify|diff|list|show> ...");
            1
        }
    }
}

// ── small arg helpers ────────────────────────────────────────────────────────

fn flag_val<'a>(rest: &'a [String], name: &str) -> Option<&'a str> {
    rest.windows(2).find(|w| w[0] == name).map(|w| w[1].as_str())
}
fn has_flag(rest: &[String], name: &str) -> bool {
    rest.iter().any(|a| a == name)
}
fn positionals(rest: &[String]) -> Vec<&str> {
    // Skip flags and their values.
    let mut out = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        let a = &rest[i];
        if a.starts_with("--") {
            // value-taking flags consume the next token
            if matches!(a.as_str(), "--tag" | "--name" | "--approver" | "--frozen-scope" | "--id" | "--date") {
                i += 2;
            } else {
                i += 1;
            }
        } else {
            out.push(a.as_str());
            i += 1;
        }
    }
    out
}

/// Resolve a configured directory against `base` (relative → joined; absolute → as-is),
/// lexically normalizing `.`/`..` so an escape can be detected without touching disk.
fn resolve_dir(base: &Path, dir: &str) -> PathBuf {
    let p = Path::new(dir);
    let joined = if p.is_absolute() { p.to_path_buf() } else { base.join(p) };
    let mut out = PathBuf::new();
    for c in joined.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Parse `package=X;types=A,B;status=approved;tags=safety` into a `FrozenScope`.
fn parse_scope(sel: &str) -> Result<FrozenScope, String> {
    let mut scope = FrozenScope::default();
    for clause in sel.split(';').map(str::trim).filter(|c| !c.is_empty()) {
        let (key, val) = clause
            .split_once('=')
            .ok_or_else(|| format!("malformed scope clause `{clause}` (expected key=value)"))?;
        let list = || val.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        match key.trim() {
            "config" => scope.config = Some(val.trim().to_string()),
            "closureFrom" => scope.closure_from = Some(list()),
            "package" => scope.package = Some(val.trim().to_string()),
            "types" => scope.types = Some(list()),
            "status" => scope.status = Some(list()),
            "tags" => scope.tags = Some(list()),
            other => return Err(format!("unknown scope key `{other}` (expected config|closureFrom|package|types|status|tags)")),
        }
    }
    Ok(scope)
}

/// Derive a `BL-*` id from a tag by uppercasing and replacing runs of
/// non-`[A-Z0-9]` with `-`, then ensuring a `BL-` prefix.
fn derive_id(tag: &str) -> String {
    let mut s = String::new();
    let mut last_dash = false;
    for c in tag.to_ascii_uppercase().chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c);
            last_dash = false;
        } else if !last_dash {
            s.push('-');
            last_dash = true;
        }
    }
    let core = s.trim_matches('-').to_string();
    // Drop a conventional release-tag prefix so `REL-2026-07` → `BL-2026-07`.
    let core = core
        .strip_prefix("BL-")
        .or_else(|| core.strip_prefix("REL-"))
        .map(str::to_string)
        .unwrap_or(core);
    format!("BL-{core}")
}

// ── create (REQ-TRS-BL-004) ──────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BaselineDoc {
    #[serde(rename = "type")]
    type_: &'static str,
    id: String,
    name: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approver: Option<String>,
    git_tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    git_commit: Option<String>,
    frozen_scope: FrozenScope,
    seal: BaselineSeal,
    #[serde(skip_serializing_if = "Option::is_none")]
    supersedes: Option<String>,
}

fn cmd_create(elems: &[RawElement], resolver: &Resolver, model_root: &Path, rest: &[String]) -> i32 {
    let Some(tag) = flag_val(rest, "--tag") else {
        eprintln!("Usage: syscribe -m <root> baseline create --tag <tag> [--name <n>] [--approver <a>] [--frozen-scope <sel>] [--id <BL-id>] [--allow-dirty] [--require-reviewed]");
        return 1;
    };
    let id = flag_val(rest, "--id").map(str::to_string).unwrap_or_else(|| derive_id(tag));
    if !syscribe_model::resolver::is_baseline_id(&id) {
        eprintln!("error: derived/`--id` `{id}` is not a valid BL-* id (^BL(-[A-Z0-9]{{2,12}})+$)");
        return 1;
    }
    let name = flag_val(rest, "--name").unwrap_or(tag).to_string();
    let approver = flag_val(rest, "--approver").map(str::to_string);

    let scope = match flag_val(rest, "--frozen-scope") {
        Some(sel) => match parse_scope(sel) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        },
        None => FrozenScope::default(),
    };

    // Collision: id / manifest / element file must not already exist.
    if resolver.resolve_ref(elems, &id).is_some() {
        eprintln!("error: a baseline with id `{id}` already exists");
        return 1;
    }
    let git_root = detect_git_root(model_root);
    // Output locations (REQ-TRS-BL-010): [baselines] element_dir / manifest_dir, defaults
    // `Baselines` (under the model root) and `baselines` (under the git root).
    let bcfg = load_baseline_config(model_root);
    let cmr = std::fs::canonicalize(model_root).unwrap_or_else(|_| model_root.to_path_buf());
    let element_dir = resolve_dir(&cmr, bcfg.element_dir.as_deref().unwrap_or("Baselines"));
    if !element_dir.starts_with(&cmr) {
        eprintln!(
            "error: [baselines] element_dir `{}` escapes the model root — the Baseline element must live within the model tree",
            element_dir.display()
        );
        return 1;
    }
    let element_abs = element_dir.join(format!("{id}.md"));

    let mbase = std::fs::canonicalize(git_root.clone().unwrap_or_else(|| cmr.clone()))
        .unwrap_or_else(|_| cmr.clone());
    let manifest_abs =
        resolve_dir(&mbase, bcfg.manifest_dir.as_deref().unwrap_or("baselines")).join(format!("{id}.manifest.json"));
    // Stored seal path: git-root-relative when under the git root (portable), else absolute.
    let manifest_rel = match &git_root {
        Some(gr) => {
            let grc = std::fs::canonicalize(gr).unwrap_or_else(|_| gr.clone());
            match manifest_abs.strip_prefix(&grc) {
                Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
                Err(_) => manifest_abs.to_string_lossy().to_string(),
            }
        }
        None => manifest_abs.to_string_lossy().to_string(),
    };

    if manifest_abs.exists() || element_abs.exists() {
        eprintln!("error: baseline `{id}` already has an element or manifest on disk");
        return 1;
    }

    // Git anchor: capture HEAD, enforce a clean tree unless --allow-dirty.
    let (git_commit, date) = match &git_root {
        Some(root) => {
            let dirty = git_output(root, &["status", "--porcelain"]).is_some();
            if dirty && !has_flag(rest, "--allow-dirty") {
                eprintln!("error: working tree has uncommitted changes; commit them or pass --allow-dirty");
                return 1;
            }
            let commit = git_output(root, &["rev-parse", "HEAD"]);
            let d = flag_val(rest, "--date").map(str::to_string).or_else(|| {
                git_output(root, &["show", "-s", "--format=%cs", "HEAD"])
            });
            (commit, d)
        }
        None => {
            eprintln!("warning: not a git repository — gitCommit will be empty");
            (None, flag_val(rest, "--date").map(str::to_string))
        }
    };

    // Resolve scope (projecting to the variant when config is set, REQ-TRS-BL-011);
    // refuse an unresolvable config or an empty seal.
    let in_scope_owned = match baseline::resolve_in_scope(elems, &scope) {
        Ok(v) => v,
        Err(m) => {
            eprintln!("error: scope config did not resolve: {m}");
            return 1;
        }
    };
    if in_scope_owned.is_empty() {
        eprintln!("error: the resolved scope is empty — nothing to seal");
        return 1;
    }
    let in_scope: Vec<&RawElement> = in_scope_owned.iter().collect();

    // Review awareness (REQ-TRS-BL-004): warn on suspect/unbaselined in-scope links;
    // --require-reviewed upgrades to a refusal.
    let in_scope_files: std::collections::HashSet<&str> =
        in_scope.iter().map(|e| e.file_path.as_str()).collect();
    let mut unreviewed = 0usize;
    for link in suspect::scan(elems, resolver) {
        if !in_scope_files.contains(link.source_file.as_str()) {
            continue;
        }
        if matches!(link.state, LinkState::Suspect | LinkState::Unbaselined) {
            unreviewed += 1;
        }
    }
    if unreviewed > 0 {
        if has_flag(rest, "--require-reviewed") {
            eprintln!("error: {unreviewed} in-scope trace link(s) are suspect or unbaselined; --require-reviewed refuses");
            return 1;
        }
        eprintln!("warning: {unreviewed} in-scope trace link(s) are suspect or unbaselined");
    }

    // Validation counts for the readiness snapshot.
    let vcfg = ValidateConfig::with_model_root(model_root);
    let result = validator::validate_with_config(elems, &vcfg);
    let errors = result.errors().count();
    let warnings = result.warnings().count();

    let (aggregate_hash, element_count) = aggregate(&in_scope);
    let mut manifest = Manifest::build(
        &id,
        Some(&name),
        date.as_deref(),
        approver.as_deref(),
        Some(tag),
        git_commit.as_deref(),
        &scope,
        &in_scope,
        errors,
        warnings,
    );

    if let Some(gr) = &git_root {
        manifest.relativize(gr);
    }
    let seal = BaselineSeal {
        aggregate_hash: aggregate_hash.clone(),
        element_count,
        manifest: manifest_rel.clone(),
    };
    let doc = BaselineDoc {
        type_: "Baseline",
        id: id.clone(),
        name: name.clone(),
        status: "draft".to_string(),
        date,
        approver,
        git_tag: tag.to_string(),
        git_commit,
        frozen_scope: scope,
        seal,
        supersedes: None,
    };
    let fm = match serde_yaml::to_string(&doc) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not serialize baseline: {e}");
            return 1;
        }
    };
    let element_md = format!("---\n{fm}---\n\n{name} — release baseline.\n");

    // Write manifest then element.
    if let Some(parent) = manifest_abs.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("error: could not create {}: {e}", parent.display());
            return 1;
        }
    }
    if let Err(e) = std::fs::write(&manifest_abs, manifest.to_json_pretty()) {
        eprintln!("error: could not write manifest {}: {e}", manifest_abs.display());
        return 1;
    }
    if let Some(parent) = element_abs.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&element_abs, element_md) {
        eprintln!("error: could not write element {}: {e}", element_abs.display());
        return 1;
    }

    println!("baselined {id} ({element_count} elements) → {aggregate_hash}");
    println!("  element:  {}", element_abs.display());
    println!("  manifest: {}", manifest_abs.display());
    println!("note: create records gitTag `{tag}` but does not create the git tag — tag the release yourself.");
    0
}

// ── verify (REQ-TRS-BL-008) ──────────────────────────────────────────────────

fn baselines(elems: &[RawElement]) -> Vec<&RawElement> {
    let mut v: Vec<&RawElement> =
        elems.iter().filter(|e| baseline::is_baseline(&e.frontmatter)).collect();
    v.sort_by(|a, b| element_key(b).cmp(&element_key(a))); // most-recent-id first
    v
}

fn cmd_verify(elems: &[RawElement], resolver: &Resolver, model_root: &Path, rest: &[String]) -> i32 {
    let targets: Vec<&RawElement> = if has_flag(rest, "--all") {
        baselines(elems)
    } else {
        match positionals(rest).first().and_then(|k| resolver.resolve_ref(elems, k)) {
            Some(e) if baseline::is_baseline(&e.frontmatter) => vec![e],
            _ => {
                eprintln!("Usage: syscribe -m <root> baseline verify <BL-id> | --all");
                return 1;
            }
        }
    };
    if targets.is_empty() {
        println!("no baselines to verify");
        return 0;
    }
    let git_root = detect_git_root(model_root);
    let mut failed = false;
    for b in targets {
        let id = element_key(b);
        let fm = &b.frontmatter;
        // A superseded baseline is history, not a gate: `validate` skips its content drift
        // and so does `verify` (GH #262); seal/manifest and git-tag problems still count.
        let superseded = fm.status.as_deref() == Some("superseded");
        let Some(seal) = &fm.seal else {
            println!("{id}: FAIL (no seal)");
            failed = true;
            continue;
        };
        let scope = fm.frozen_scope.clone().unwrap_or_default();
        let (current, _n) = baseline::aggregate_for_scope(elems, &scope);
        let mut msgs: Vec<String> = Vec::new();
        if !superseded && current != seal.aggregate_hash {
            msgs.push("content drift (recomputed aggregate ≠ seal)".to_string());
        }
        if let Some(m) = Manifest::from_file(&manifest_path(model_root, seal)) {
            if m.aggregate_hash != seal.aggregate_hash {
                msgs.push("manifest aggregate ≠ seal".to_string());
            }
        }
        // Git consistency: gitTag must resolve to gitCommit when the tag exists.
        if let (Some(root), Some(tag), Some(commit)) =
            (&git_root, fm.git_tag.as_deref(), fm.git_commit.as_deref())
        {
            match git_output(root, &["rev-parse", &format!("{tag}^{{commit}}")]) {
                Some(tag_commit) if tag_commit == commit => {}
                Some(tag_commit) => msgs.push(format!("gitTag `{tag}` → {} ≠ gitCommit {}", &tag_commit[..tag_commit.len().min(12)], &commit[..commit.len().min(12)])),
                None => println!("{id}: note — gitTag `{tag}` does not resolve yet (not pushed?)"),
            }
        }
        if msgs.is_empty() && superseded {
            println!("{id}: skipped (superseded)");
        } else if msgs.is_empty() {
            println!("{id}: OK");
        } else {
            println!("{id}: FAIL — {}", msgs.join("; "));
            failed = true;
            if has_flag(rest, "--detail") && !superseded && current != seal.aggregate_hash {
                if let Some(m) = Manifest::from_file(&manifest_path(model_root, seal)) {
                    match current_items(elems, &scope) {
                        Ok(items) => print_changes(model_root, &m, &items, true),
                        Err(e) => println!("  (cannot list the drift: scope config did not resolve: {e})"),
                    }
                }
            }
        }
    }
    if failed {
        2
    } else {
        0
    }
}

// ── diff (REQ-TRS-BL-006) ────────────────────────────────────────────────────

fn load_manifest_for(b: &RawElement, model_root: &Path) -> Option<Manifest> {
    let seal = b.frontmatter.seal.as_ref()?;
    Manifest::from_file(&manifest_path(model_root, seal))
}

/// One element on either side of a comparison.
struct Item {
    key: String,
    type_name: String,
    hash: String,
    file: String,
}

fn manifest_items(m: &Manifest) -> Vec<Item> {
    m.elements
        .iter()
        .map(|e| Item {
            key: e.id.clone().unwrap_or_else(|| e.qname.clone()),
            type_name: e.type_name.clone(),
            hash: e.hash.clone(),
            file: e.file.clone(),
        })
        .collect()
}

/// The elements a baseline's scope selects in the working tree now (projected when the
/// scope names a config), hashed exactly as the seal and the manifest hash them.
fn current_items(elems: &[RawElement], scope: &FrozenScope) -> Result<Vec<Item>, String> {
    let owned = baseline::resolve_in_scope(elems, scope)?;
    Ok(owned
        .iter()
        .map(|e| Item {
            key: e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone()),
            type_name: e
                .frontmatter
                .element_type
                .as_ref()
                .map(baseline::element_type_name)
                .unwrap_or_else(|| "Unknown".to_string()),
            hash: syscribe_model::suspect::content_hash(e),
            file: e.file_path.clone(),
        })
        .collect())
}

fn cmd_diff(elems: &[RawElement], resolver: &Resolver, model_root: &Path, rest: &[String]) -> i32 {
    let pos = positionals(rest);
    let current = has_flag(rest, "--current");
    if pos.len() != if current { 1 } else { 2 } {
        eprintln!("Usage: syscribe -m <root> baseline diff <BL-A> <BL-B> [--detail] | <BL> --current [--detail]");
        return 1;
    }
    let resolve = |k: &str| resolver.resolve_ref(elems, k).filter(|e| baseline::is_baseline(&e.frontmatter));
    let Some(a) = resolve(pos[0]) else {
        eprintln!("error: `{}` must resolve to a Baseline element", pos[0]);
        return 1;
    };
    let Some(ma) = load_manifest_for(a, model_root) else {
        eprintln!("error: could not load the manifest of `{}`", pos[0]);
        return 1;
    };
    let detail = has_flag(rest, "--detail");
    if current {
        let scope = a.frontmatter.frozen_scope.clone().unwrap_or_default();
        let items = match current_items(elems, &scope) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("error: the baseline's scope config did not resolve: {e}");
                return 1;
            }
        };
        println!("# baseline diff {} → working tree", ma.baseline);
        print_changes(model_root, &ma, &items, detail);
        return 0;
    }
    let Some(b) = resolve(pos[1]) else {
        eprintln!("error: `{}` must resolve to a Baseline element", pos[1]);
        return 1;
    };
    let Some(mb) = load_manifest_for(b, model_root) else {
        eprintln!("error: could not load the manifest of `{}`", pos[1]);
        return 1;
    };
    println!("# baseline diff {} → {}", ma.baseline, mb.baseline);
    if ma.aggregate_hash == mb.aggregate_hash {
        println!("aggregate: identical");
    }
    print_changes_between(model_root, &ma, Some(&mb), &manifest_items(&ma), &manifest_items(&mb), detail);
    0
}

/// Compare manifest `a` with the working tree `items` and print the changes.
fn print_changes(model_root: &Path, a: &Manifest, items: &[Item], detail: bool) {
    print_changes_between(model_root, a, None, &manifest_items(a), items, detail);
}

/// Print added / removed / changed elements between two item sets. `mb` is the newer
/// manifest, or `None` when the newer side is the working tree.
fn print_changes_between(
    model_root: &Path,
    ma: &Manifest,
    mb: Option<&Manifest>,
    old: &[Item],
    new: &[Item],
    detail: bool,
) {
    let map_a: BTreeMap<&str, &Item> = old.iter().map(|e| (e.key.as_str(), e)).collect();
    let map_b: BTreeMap<&str, &Item> = new.iter().map(|e| (e.key.as_str(), e)).collect();
    let mut added: Vec<&Item> = Vec::new();
    let mut removed: Vec<&Item> = Vec::new();
    let mut changed: Vec<(&Item, &Item)> = Vec::new();
    for (k, eb) in &map_b {
        match map_a.get(k) {
            None => added.push(eb),
            Some(ea) if ea.hash != eb.hash => changed.push((ea, eb)),
            Some(_) => {}
        }
    }
    for (k, ea) in &map_a {
        if !map_b.contains_key(k) {
            removed.push(ea);
        }
    }
    let group = |label: &str, items: &[&Item]| {
        println!("\n## {label} ({})", items.len());
        let mut by_type: BTreeMap<&str, Vec<&Item>> = BTreeMap::new();
        for e in items {
            by_type.entry(e.type_name.as_str()).or_default().push(e);
        }
        for (ty, es) in by_type {
            for e in es {
                println!("  [{ty}] {}", e.key);
            }
        }
    };
    group("added", &added);
    group("removed", &removed);
    println!("\n## changed ({})", changed.len());
    let mut by_type: BTreeMap<&str, Vec<(&Item, &Item)>> = BTreeMap::new();
    for (ea, eb) in &changed {
        by_type.entry(eb.type_name.as_str()).or_default().push((ea, eb));
    }
    for (ty, es) in by_type {
        for (ea, eb) in es {
            println!("  [{ty}] {}", eb.key);
            if detail {
                print_detail(model_root, ma, mb, ea, eb);
            }
        }
    }
}

/// Candidate repo-root-relative paths for a manifest `file`: as stored when relative,
/// else the part under the git root, else every trailing sub-path longest-first (an
/// absolute path written on another machine, GH #261).
fn repo_paths(root: &Path, file: &str) -> Vec<String> {
    let p = Path::new(file);
    if !p.is_absolute() {
        return vec![file.replace('\\', "/")];
    }
    let rootc = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    for r in [root, rootc.as_path()] {
        if let Ok(rel) = p.strip_prefix(r) {
            return vec![rel.to_string_lossy().replace('\\', "/")];
        }
    }
    let comps: Vec<String> = p
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();
    (0..comps.len()).map(|i| comps[i..].join("/")).collect()
}

/// Reconstruct the old and new versions of a changed element and print a diff. The old
/// side comes from `git show` at the older baseline's commit; the new side from the newer
/// baseline's commit, or the file on disk when comparing against the working tree.
fn print_detail(model_root: &Path, ma: &Manifest, mb: Option<&Manifest>, ea: &Item, eb: &Item) {
    let Some(root) = detect_git_root(model_root) else { return };
    let show = |commit: &Option<String>, file: &str| -> Option<String> {
        let c = commit.as_ref()?;
        repo_paths(&root, file).iter().find_map(|path| git_output(&root, &["show", &format!("{c}:{path}")]))
    };
    let old = show(&ma.git_commit, &ea.file);
    let new = match mb {
        Some(mb) => show(&mb.git_commit, &eb.file),
        None => std::fs::read_to_string(&eb.file).ok(),
    };
    match (old, new) {
        (Some(o), Some(n)) => {
            for line in unified_lines(&o, &n) {
                println!("      {line}");
            }
        }
        _ => println!("      (content not retrievable from git — hash-level change only)"),
    }
}

/// A minimal line-level unified diff (added/removed lines) — enough for a review view.
fn unified_lines(old: &str, new: &str) -> Vec<String> {
    let ol: Vec<&str> = old.lines().collect();
    let nl: Vec<&str> = new.lines().collect();
    let oset: std::collections::HashSet<&str> = ol.iter().copied().collect();
    let nset: std::collections::HashSet<&str> = nl.iter().copied().collect();
    let mut out = Vec::new();
    for l in &ol {
        if !nset.contains(l) {
            out.push(format!("- {l}"));
        }
    }
    for l in &nl {
        if !oset.contains(l) {
            out.push(format!("+ {l}"));
        }
    }
    out
}

// ── list / show (REQ-TRS-BL-007) ─────────────────────────────────────────────

fn scope_summary(scope: &FrozenScope) -> String {
    let mut parts = Vec::new();
    if let Some(c) = &scope.config {
        parts.push(format!("config={c}"));
    }
    if let Some(s) = &scope.closure_from {
        parts.push(format!("closureFrom={}", s.join(",")));
    }
    if let Some(p) = &scope.package {
        parts.push(format!("package={p}"));
    }
    if let Some(t) = &scope.types {
        parts.push(format!("types={}", t.join(",")));
    }
    if let Some(s) = &scope.status {
        parts.push(format!("status={}", s.join(",")));
    }
    if let Some(t) = &scope.tags {
        parts.push(format!("tags={}", t.join(",")));
    }
    if parts.is_empty() {
        "whole-model".to_string()
    } else {
        parts.join(" ")
    }
}

fn cmd_list(elems: &[RawElement]) -> i32 {
    let bs = baselines(elems);
    println!("# Baselines ({})", bs.len());
    for b in bs {
        let fm = &b.frontmatter;
        let scope = fm.frozen_scope.clone().unwrap_or_default();
        println!(
            "  {}  [{}]  {}  {}  ({})",
            element_key(b),
            fm.status.as_deref().unwrap_or("draft"),
            fm.date.as_deref().unwrap_or("—"),
            fm.name.as_deref().unwrap_or(""),
            scope_summary(&scope),
        );
    }
    0
}

fn cmd_show(elems: &[RawElement], resolver: &Resolver, model_root: &Path, rest: &[String]) -> i32 {
    let Some(b) = positionals(rest).first().and_then(|k| resolver.resolve_ref(elems, k)) else {
        eprintln!("Usage: syscribe -m <root> baseline show <BL-id>");
        return 1;
    };
    if !baseline::is_baseline(&b.frontmatter) {
        eprintln!("error: `{}` is not a Baseline", element_key(b));
        return 1;
    }
    let fm = &b.frontmatter;
    let scope = fm.frozen_scope.clone().unwrap_or_default();
    println!("# Baseline {}", element_key(b));
    println!("name:      {}", fm.name.as_deref().unwrap_or(""));
    println!("status:    {}", fm.status.as_deref().unwrap_or("draft"));
    println!("date:      {}", fm.date.as_deref().unwrap_or("—"));
    println!("approver:  {}", fm.approver.as_deref().unwrap_or("—"));
    println!("gitTag:    {}", fm.git_tag.as_deref().unwrap_or("—"));
    println!("gitCommit: {}", fm.git_commit.as_deref().unwrap_or("—"));
    println!("scope:     {}", scope_summary(&scope));
    if let Some(seal) = &fm.seal {
        println!("elements:  {}", seal.element_count);
        println!("aggregate: {}", seal.aggregate_hash);
        println!("manifest:  {}", manifest_path(model_root, seal).display());
    }
    if let Some(sup) = fm.supersedes.as_ref().filter(|v| !v.is_empty()) {
        println!("supersedes: {}", sup.join(", "));
    }
    let _ = PathBuf::new();
    0
}

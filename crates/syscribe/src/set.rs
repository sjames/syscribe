//! `syscribe set <qname|id> <op>` — narrow, schema-aware mutation of a small
//! allowlist of existing element fields (issue #112).
//!
//! Deliberately narrower than the MCP `update_element` tool's arbitrary-field
//! merge: every operation here validates its own value *before* anything is
//! written — an out-of-enum `status`, a dangling `achieves`/`evidence` target
//! — so an agent-authored typo is caught at the point of the edit, not only
//! at the next full-model `validate` (which might not catch it at all, if the
//! typo doesn't happen to trip an existing rule). Byte-preserving for
//! everything else in the file via `apply_update_fields`/`patch_frontmatter`
//! — the same "surgical edit" bar `move` already holds itself to for
//! reference rewriting.
//!
//! Three operations, matching the issue's own proposed shape exactly:
//!   syscribe set <id> status=<value>
//!   syscribe set <id> evidence.add ref=<id> | path=<path> [rationale=<text>]
//!   syscribe set <id> achieves.add <req-id>

use std::path::Path;

use serde_json::{json, Value};

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::frontmatter::{splice_frontmatter, split_frontmatter, yaml_scalar};
use syscribe_model::mutate::{apply_update_fields, file_unified_diff};
use syscribe_model::resolver::Resolver;
use syscribe_model::{config::ValidateConfig, validator};

/// Per-type `status:` enum, mirroring the validator's own E007 (Requirement/
/// TestCase)/E604 (TestPlan)/E304 (ADR)/E708 (PlanningItem) checks. There is no
/// single shared source of truth for these today (each lives inline in its own
/// validator pass) so this list is kept in sync by hand. `None` means the
/// type's `status:` is free text in this format — write it unvalidated, same
/// as `validate` itself does for those types.
fn status_enum_for(et: &ElementType) -> Option<&'static [&'static str]> {
    match et {
        ElementType::Requirement => Some(&["draft", "review", "approved", "implemented", "verified"]),
        ElementType::TestCase => Some(&["draft", "review", "approved", "active", "retired"]),
        ElementType::TestPlan => Some(&["draft", "review", "approved", "active", "retired"]),
        ElementType::ADR => Some(&["proposed", "accepted", "deprecated", "superseded"]),
        ElementType::PlanningItem => Some(&["todo", "in_progress", "blocked", "done"]),
        ElementType::ReviewRecord => Some(&["open", "closed", "waived"]),
        _ => None,
    }
}

fn yaml_to_json(v: &serde_yaml::Value) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

/// Print the would-be file content as a unified diff (or, on commit, write it),
/// returning whether anything actually changed.
fn preview_or_write(model_root: &Path, elem: &RawElement, new_content: &str, dry_run: bool) -> bool {
    let rel = elem
        .file_path
        .strip_prefix(&*model_root.to_string_lossy())
        .map(|s| s.trim_start_matches(['/', '\\']))
        .unwrap_or(&elem.file_path);
    let old = std::fs::read_to_string(&elem.file_path).ok();
    let diff = file_unified_diff(rel, old.as_deref(), Some(new_content));
    if diff.is_empty() {
        println!("{}: no change.", elem.qualified_name);
        return false;
    }
    if dry_run {
        print!("{diff}");
        return false;
    }
    if let Err(e) = std::fs::write(&elem.file_path, new_content) {
        eprintln!("Write failed for {}: {e}", elem.file_path);
        std::process::exit(1);
    }
    print!("{diff}");
    true
}

/// `status=<value>` — validated per-type enum before anything is written.
fn cmd_set_status(
    model_root: &Path,
    elements: &[RawElement],
    elem: &RawElement,
    value: &str,
    dry_run: bool,
) {
    if let Some(et) = &elem.frontmatter.element_type {
        if let Some(allowed) = status_enum_for(et) {
            if !allowed.contains(&value) {
                eprintln!(
                    "Refusing to set status='{}' on {} — not a valid status for {:?}. Allowed: {}",
                    value,
                    elem.qualified_name,
                    et,
                    allowed.join(", ")
                );
                std::process::exit(1);
            }
        }
    }

    // PlanningItem.status=done: reuse the exact W310 check (issue #114) against
    // an in-memory candidate, rather than re-deriving it — non-blocking (matches
    // W310's own warning severity): the write still proceeds, but the agent
    // making the change sees the same thing `validate` would tell it next.
    if matches!(elem.frontmatter.element_type, Some(ElementType::PlanningItem)) && value == "done" {
        let mut candidate: Vec<RawElement> = elements.to_vec();
        for e in candidate.iter_mut() {
            if e.qualified_name == elem.qualified_name {
                e.frontmatter.status = Some(value.to_string());
            }
        }
        let vcfg = ValidateConfig::with_model_root(model_root);
        let result = validator::validate_with_config(&candidate, &vcfg);
        for f in result.findings.iter().filter(|f| f.code == "W310" && f.file == elem.file_path) {
            eprintln!("warning: {}", f.message);
        }
    }

    // A true single-line splice (not `apply_update_fields`'s full YAML
    // round-trip) so every other byte of the file — including another
    // field's quoting style — is left exactly as it was (acceptance
    // criterion: "writes only that field, byte-identical elsewhere").
    // Replaces in place when `status:` already exists (preserving field
    // order), appends it otherwise (an element with no `status:` yet).
    let content = std::fs::read_to_string(&elem.file_path).unwrap_or_default();
    let (yaml_opt, _) = split_frontmatter(&content);
    let Some(yaml) = yaml_opt else {
        eprintln!("{} has no YAML frontmatter to edit.", elem.file_path);
        std::process::exit(1);
    };
    let new_line = format!("status: {}", yaml_scalar(value));
    let mut found = false;
    let mut lines: Vec<String> = Vec::new();
    for line in yaml.lines() {
        if line.starts_with("status:") {
            lines.push(new_line.clone());
            found = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !found {
        lines.push(new_line);
    }
    let new_fm = lines.join("\n");
    let new_content = splice_frontmatter(&content, yaml, &new_fm);

    let committed = preview_or_write(model_root, elem, &new_content, dry_run);
    if committed {
        println!("Set status: {value} on {}", elem.qualified_name);
    }
}

/// `achieves.add <req-id>` — the target must resolve to a native `Requirement`
/// (mirrors `E714`/`E715`) before anything is written; appends without
/// disturbing existing order.
fn cmd_achieves_add(
    model_root: &Path,
    elements: &[RawElement],
    resolver: &Resolver,
    elem: &RawElement,
    req_ref: &str,
    dry_run: bool,
) {
    match resolver.resolve_ref(elements, req_ref) {
        None => {
            eprintln!("Refusing to add achieves '{}' on {} — it does not resolve to any model element.", req_ref, elem.qualified_name);
            std::process::exit(1);
        }
        Some(target) if !Resolver::is_native_requirement(target) => {
            eprintln!(
                "Refusing to add achieves '{}' on {} — it does not resolve to a native Requirement.",
                req_ref, elem.qualified_name
            );
            std::process::exit(1);
        }
        Some(_) => {}
    }

    let mut list = elem.frontmatter.achieves.clone().unwrap_or_default();
    if list.iter().any(|s| s == req_ref) {
        println!("{} already achieves '{}' — nothing to do.", elem.qualified_name, req_ref);
        return;
    }
    list.push(req_ref.to_string());

    let content = std::fs::read_to_string(&elem.file_path).unwrap_or_default();
    let fields = json!({ "achieves": list });
    let new_content = match apply_update_fields(&content, Some(&fields), None) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cannot patch {}: {e}", elem.file_path);
            std::process::exit(1);
        }
    };
    let committed = preview_or_write(model_root, elem, &new_content, dry_run);
    if committed {
        println!("Added achieves: {req_ref} on {}", elem.qualified_name);
    }
}

/// `evidence.add ref=<id> | path=<path> [rationale=<text>]` — a `ref:` must
/// resolve to some model element (mirrors `E716`'s bar; permissive by kind,
/// like `blockedBy:`); a `path:` must exist on disk under the model root or be
/// an accepted remote URI (mirrors `E717`). Appends without disturbing
/// existing evidence order.
fn cmd_evidence_add(
    model_root: &Path,
    elements: &[RawElement],
    resolver: &Resolver,
    elem: &RawElement,
    kv_args: &[&str],
    dry_run: bool,
) {
    let mut r#ref: Option<&str> = None;
    let mut path: Option<&str> = None;
    let mut rationale: Option<&str> = None;
    for a in kv_args {
        if let Some(v) = a.strip_prefix("ref=") {
            r#ref = Some(v);
        } else if let Some(v) = a.strip_prefix("path=") {
            path = Some(v);
        } else if let Some(v) = a.strip_prefix("rationale=") {
            rationale = Some(v);
        } else {
            eprintln!("evidence.add: unrecognized argument '{a}' (expected ref=<id>, path=<path>, or rationale=<text>)");
            std::process::exit(1);
        }
    }

    if r#ref.is_none() && path.is_none() {
        eprintln!("evidence.add requires ref=<id> or path=<path>");
        std::process::exit(1);
    }
    if r#ref.is_some() && path.is_some() {
        eprintln!("evidence.add: pass ref=<id> or path=<path>, not both");
        std::process::exit(1);
    }

    if let Some(r) = r#ref {
        if resolver.resolve_ref(elements, r).is_none() {
            eprintln!(
                "Refusing to add evidence ref='{}' on {} — it does not resolve to any model element.",
                r, elem.qualified_name
            );
            std::process::exit(1);
        }
    }
    if let Some(p) = path {
        let is_remote = p.starts_with("http://") || p.starts_with("https://");
        let exists = model_root.join(p).exists();
        if !is_remote && !exists {
            eprintln!(
                "Refusing to add evidence path='{}' on {} — it does not exist on disk (and is not a remote URI).",
                p, elem.qualified_name
            );
            std::process::exit(1);
        }
    }

    let mut entry = serde_json::Map::new();
    if let Some(r) = r#ref {
        entry.insert("ref".to_string(), Value::String(r.to_string()));
    }
    if let Some(p) = path {
        entry.insert("path".to_string(), Value::String(p.to_string()));
    }
    if let Some(r) = rationale {
        entry.insert("rationale".to_string(), Value::String(r.to_string()));
    }

    let mut list: Vec<Value> = elem
        .frontmatter
        .evidence
        .as_ref()
        .map(|v| v.iter().map(yaml_to_json).collect())
        .unwrap_or_default();
    list.push(Value::Object(entry));

    let content = std::fs::read_to_string(&elem.file_path).unwrap_or_default();
    let fields = json!({ "evidence": list });
    let new_content = match apply_update_fields(&content, Some(&fields), None) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cannot patch {}: {e}", elem.file_path);
            std::process::exit(1);
        }
    };
    let committed = preview_or_write(model_root, elem, &new_content, dry_run);
    if committed {
        println!("Added evidence entry on {}", elem.qualified_name);
    }
}

/// `set` subcommand entry point. `op_args` is every token after the target key,
/// `--dry-run` already stripped by the caller.
pub fn cmd_set(
    model_root: &Path,
    elements: &[RawElement],
    resolver: &Resolver,
    target_key: &str,
    op_args: &[&str],
    dry_run: bool,
) {
    let elem = match resolver.resolve_ref(elements, target_key) {
        Some(e) => e,
        None => {
            eprintln!("Element not found: {target_key}");
            std::process::exit(1);
        }
    };

    let Some(&op) = op_args.first() else {
        eprintln!("Usage: syscribe set <qname|id> status=<value> | evidence.add ref=<id>|path=<path> | achieves.add <req-id>");
        std::process::exit(1);
    };

    if let Some(value) = op.strip_prefix("status=") {
        cmd_set_status(model_root, elements, elem, value, dry_run);
        return;
    }
    if op == "evidence.add" {
        cmd_evidence_add(model_root, elements, resolver, elem, &op_args[1..], dry_run);
        return;
    }
    if op == "achieves.add" {
        let Some(&req_ref) = op_args.get(1) else {
            eprintln!("Usage: syscribe set <qname|id> achieves.add <req-id>");
            std::process::exit(1);
        };
        cmd_achieves_add(model_root, elements, resolver, elem, req_ref, dry_run);
        return;
    }
    if let Some((field, _)) = op.split_once('=') {
        eprintln!(
            "Unsupported field '{field}' for `set` — only `status` is a direct field= assignment today; use `evidence.add`/`achieves.add` for list fields."
        );
    } else {
        eprintln!("Unrecognized `set` operation '{op}' — expected status=<value>, evidence.add, or achieves.add");
    }
    std::process::exit(1);
}

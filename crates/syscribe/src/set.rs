//! `syscribe set <qname|id> <op>` — narrow, schema-aware mutation of a small
//! allowlist of existing element fields (issue #112).
//!
//! Deliberately narrower than the MCP `update_element` tool's arbitrary-field
//! merge: every operation here validates its own value *before* anything is
//! written — an out-of-enum `status`, a dangling `achieves`/`evidence` target
//! — so an agent-authored typo is caught at the point of the edit, not only
//! at the next full-model `validate` (which might not catch it at all, if the
//! typo doesn't happen to trip an existing rule). Every operation is a
//! line-level edit of the frontmatter — `status=` replaces one line, the
//! `.add` operations insert the new list item's lines (GH #152) — so every
//! other byte of the file, YAML comments included, is preserved: the same
//! "surgical edit" bar `move` already holds itself to for reference
//! rewriting.
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

/// A string rendered as a one-line YAML scalar, quoted exactly when YAML needs
/// it (so `src/a.rs` stays plain while `true`, `123` or `a: b` are quoted and
/// never re-typed on the next parse) — the same choice the full
/// re-serialisation made. A value that would need a multi-line block scalar
/// is written as a JSON (= YAML double-quoted) string instead.
fn line_scalar(value: &str) -> String {
    match serde_yaml::to_string(&serde_yaml::Value::String(value.to_string())) {
        Ok(s) if !s.trim_end_matches('\n').contains('\n') => s.trim_end_matches('\n').to_string(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| yaml_scalar(value)),
    }
}

/// Render `items` (each one list item: its first line, then any continuation
/// lines of a mapping item) as block-sequence lines at indentation `ind`.
fn render_items(items: &[Vec<String>], ind: &str) -> Vec<String> {
    let mut out = Vec::new();
    for item in items {
        for (i, line) in item.iter().enumerate() {
            if i == 0 {
                out.push(format!("{ind}- {line}"));
            } else {
                out.push(format!("{ind}  {line}"));
            }
        }
    }
    out
}

/// The lines of one already-parsed list item, as a mapping's `key: value`
/// lines or a single scalar line.
fn value_item_lines(v: &serde_yaml::Value) -> Option<Vec<String>> {
    let text = serde_yaml::to_string(v).ok()?;
    Some(text.lines().map(str::to_string).collect())
}

/// Append `new_items` to the top-level list `key:` of the frontmatter `yaml`
/// by editing only the lines of that list (GH #152) — every other line,
/// including comments, quoting and the existing items themselves, is left
/// byte-for-byte as it was. Handles an absent key (a new block list is
/// appended at the end of the frontmatter), a block list (the items are
/// inserted after its last item, at its own indentation, before any trailing
/// blank/column-0 comment lines) and a one-line inline value (`[]`, `~`, a
/// flow list or a scalar), which is rewritten as a block list. Returns `None`
/// for a layout it cannot edit line-wise (a flow list spanning several lines);
/// the caller then falls back to a full re-serialisation.
fn append_list_items(yaml: &str, key: &str, new_items: &[Vec<String>]) -> Option<String> {
    let lines: Vec<&str> = yaml.lines().collect();
    let key_prefix = format!("{key}:");
    let key_idx = lines.iter().position(|l| {
        l.strip_prefix(&key_prefix)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '\t']))
    });
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    let Some(i) = key_idx else {
        out.push(key_prefix);
        out.extend(render_items(new_items, "  "));
        return Some(out.join("\n"));
    };
    let inline = lines[i][key_prefix.len()..].trim();
    if inline.is_empty() || inline.starts_with('#') {
        // Block list (or an empty value): the block runs over every following
        // indented, `-`-led, blank or comment line.
        let mut end = i + 1;
        while end < lines.len() {
            let l = lines[end];
            if l.trim().is_empty() || l.starts_with([' ', '\t', '-', '#']) {
                end += 1;
            } else {
                break;
            }
        }
        // Trailing blank lines and column-0 comments belong to what follows.
        while end > i + 1 && (lines[end - 1].trim().is_empty() || lines[end - 1].starts_with('#')) {
            end -= 1;
        }
        let ind = lines[i + 1..end]
            .iter()
            .find_map(|l| {
                let t = l.trim_start();
                (t.starts_with("- ") || t == "-").then(|| &l[..l.len() - t.len()])
            })
            .unwrap_or("  ");
        let rendered = render_items(new_items, ind);
        out.splice(end..end, rendered);
        return Some(out.join("\n"));
    }
    // A one-line inline value: rewrite just this line as a block list.
    let existing: Vec<serde_yaml::Value> = match serde_yaml::from_str::<serde_yaml::Value>(inline).ok()? {
        serde_yaml::Value::Null => Vec::new(),
        serde_yaml::Value::Sequence(s) => s,
        other => vec![other],
    };
    let mut items: Vec<Vec<String>> = Vec::new();
    for v in &existing {
        items.push(value_item_lines(v)?);
    }
    items.extend(new_items.iter().cloned());
    let mut replacement = vec![key_prefix];
    replacement.extend(render_items(&items, "  "));
    out.splice(i..=i, replacement);
    Some(out.join("\n"))
}

/// Append `new_items` to list `key:` in `content`'s frontmatter, line-wise
/// ([`append_list_items`]) when possible, else by re-serialising the
/// frontmatter with `full_list` as the field's new value.
fn append_to_list_field(
    content: &str,
    file_path: &str,
    key: &str,
    new_items: &[Vec<String>],
    full_list: Value,
) -> String {
    let (yaml_opt, _) = split_frontmatter(content);
    if let Some(yaml) = yaml_opt {
        if let Some(new_fm) = append_list_items(yaml, key, new_items) {
            return splice_frontmatter(content, yaml, &new_fm);
        }
    }
    let mut fields = serde_json::Map::new();
    fields.insert(key.to_string(), full_list);
    match apply_update_fields(content, Some(&Value::Object(fields)), None) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cannot patch {file_path}: {e}");
            std::process::exit(1);
        }
    }
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
    let new_content = append_to_list_field(
        &content,
        &elem.file_path,
        "achieves",
        &[vec![line_scalar(req_ref)]],
        json!(list),
    );
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

    // An entry naming the same `ref:`/`path:` target already exists: a
    // reported no-op, like `achieves.add` on an existing requirement (GH
    // #152) — the existing entry (and its rationale) is kept as authored.
    let Some((target_key, target)) = r#ref.map(|r| ("ref", r)).or(path.map(|p| ("path", p))) else {
        eprintln!("evidence.add requires ref=<id> or path=<path>");
        std::process::exit(1);
    };
    let existing = elem.frontmatter.evidence.as_deref().unwrap_or(&[]);
    if existing.iter().any(|e| {
        e.as_mapping()
            .and_then(|m| m.get(serde_yaml::Value::String(target_key.to_string())))
            .and_then(|v| v.as_str())
            == Some(target)
    }) {
        println!(
            "{} already has evidence {}='{}' — nothing to do.",
            elem.qualified_name, target_key, target
        );
        return;
    }

    let mut entry = serde_json::Map::new();
    let mut entry_lines = vec![format!("{target_key}: {}", line_scalar(target))];
    entry.insert(target_key.to_string(), Value::String(target.to_string()));
    if let Some(r) = rationale {
        entry.insert("rationale".to_string(), Value::String(r.to_string()));
        entry_lines.push(format!("rationale: {}", line_scalar(r)));
    }

    let mut list: Vec<Value> = existing.iter().map(yaml_to_json).collect();
    list.push(Value::Object(entry));

    let content = std::fs::read_to_string(&elem.file_path).unwrap_or_default();
    let new_content = append_to_list_field(&content, &elem.file_path, "evidence", &[entry_lines], json!(list));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn item(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn appends_to_a_block_list_keeping_comments_and_indentation() {
        let yaml = "id: PI-001\n# evidence so far\nevidence:\n    # first proof\n    - ref: TC-001  # the test\n      rationale: \"waived\"\n# trailing note\nstatus: done";
        let out = append_list_items(yaml, "evidence", &[item(&["ref: TC-002"])]).unwrap();
        assert_eq!(
            out,
            "id: PI-001\n# evidence so far\nevidence:\n    # first proof\n    - ref: TC-001  # the test\n      rationale: \"waived\"\n    - ref: TC-002\n# trailing note\nstatus: done"
        );
    }

    #[test]
    fn appends_to_a_zero_indented_block_list() {
        let yaml = "achieves:\n- REQ-001\nstatus: todo";
        let out = append_list_items(yaml, "achieves", &[item(&["REQ-002"])]).unwrap();
        assert_eq!(out, "achieves:\n- REQ-001\n- REQ-002\nstatus: todo");
    }

    #[test]
    fn adds_an_absent_key_at_the_end() {
        let yaml = "id: PI-001 # keep me\nstatus: todo";
        let out = append_list_items(yaml, "evidence", &[item(&["path: src/a.rs", "rationale: \"why\""])]).unwrap();
        assert_eq!(out, "id: PI-001 # keep me\nstatus: todo\nevidence:\n  - path: src/a.rs\n    rationale: \"why\"");
    }

    #[test]
    fn rewrites_a_one_line_inline_value_as_a_block_list() {
        let yaml = "# c\nachieves: [REQ-001, REQ-002]\nstatus: todo";
        let out = append_list_items(yaml, "achieves", &[item(&["REQ-003"])]).unwrap();
        assert_eq!(out, "# c\nachieves:\n  - REQ-001\n  - REQ-002\n  - REQ-003\nstatus: todo");
        let yaml = "achieves: []\nstatus: todo";
        let out = append_list_items(yaml, "achieves", &[item(&["REQ-003"])]).unwrap();
        assert_eq!(out, "achieves:\n  - REQ-003\nstatus: todo");
        let yaml = "achieves: REQ-001";
        let out = append_list_items(yaml, "achieves", &[item(&["REQ-003"])]).unwrap();
        assert_eq!(out, "achieves:\n  - REQ-001\n  - REQ-003");
    }

    #[test]
    fn a_multi_line_flow_list_is_left_to_the_fallback() {
        let yaml = "achieves: [REQ-001,\n  REQ-002]\nstatus: todo";
        assert!(append_list_items(yaml, "achieves", &[item(&["REQ-003"])]).is_none());
    }

    #[test]
    fn a_key_that_merely_shares_a_prefix_is_not_the_list() {
        let yaml = "evidenceNote: x\nstatus: todo";
        let out = append_list_items(yaml, "evidence", &[item(&["ref: TC-001"])]).unwrap();
        assert_eq!(out, "evidenceNote: x\nstatus: todo\nevidence:\n  - ref: TC-001");
    }

    #[test]
    fn line_scalar_quotes_values_that_would_be_retyped() {
        for v in ["TC-001", "src/a.rs", "https://x.org/r.html", "true", "123", "a: b", "a b", "two\nlines", "#x"] {
            let line = format!("k: {}", line_scalar(v));
            assert!(!line.contains('\n'), "{line}");
            let back: serde_yaml::Value = serde_yaml::from_str(&line).unwrap();
            assert_eq!(back["k"].as_str(), Some(v), "{line}");
        }
        assert_eq!(line_scalar("src/a.rs"), "src/a.rs");
        assert_ne!(line_scalar("true"), "true");
    }
}

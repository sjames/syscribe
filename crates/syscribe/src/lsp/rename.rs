//! `textDocument/prepareRename` + `textDocument/rename` (ADR-SYS-LSP-002).
//!
//! Scoped to stable-id-identified elements only (REQ-TRS-LSP-010); a qname/name rename
//! is a file move and stays out of scope (`syscribe mcp move_element`'s job). The
//! server never writes to disk — it returns a `WorkspaceEdit` for the client to apply
//! (REQ-TRS-LSP-011), after validating the candidate entirely in memory
//! (REQ-TRS-LSP-012): the general model validator already catches a malformed new id
//! (`E006`-family) and an id collision (`E101`), so the gate needs no bespoke
//! pattern/collision checks of its own — it just diffs findings before vs. after.

use std::collections::HashMap;
use std::path::Path;

use tower_lsp::lsp_types::*;

use syscribe_model::element::RawElement;
use syscribe_model::resolver::is_stable_id_type_name;
use syscribe_model::validator::validate_with_config;

use crate::query::type_label;

use super::store::LspStore;

fn value_contains_needle(v: &serde_yaml::Value, needle: &str) -> bool {
    match v {
        serde_yaml::Value::String(s) => s == needle,
        serde_yaml::Value::Sequence(seq) => seq.iter().any(|x| value_contains_needle(x, needle)),
        serde_yaml::Value::Mapping(m) => {
            m.iter().any(|(k, v)| matches!(k, serde_yaml::Value::String(ks) if ks == needle) || value_contains_needle(v, needle))
        }
        _ => false,
    }
}

fn replace_needle(v: &mut serde_yaml::Value, old: &str, new: &str) {
    match v {
        serde_yaml::Value::String(s) => {
            if s == old {
                *s = new.to_string();
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for x in seq.iter_mut() {
                replace_needle(x, old, new);
            }
        }
        serde_yaml::Value::Mapping(m) => {
            let mut rebuilt = serde_yaml::Mapping::new();
            for (k, mut val) in std::mem::take(m) {
                replace_needle(&mut val, old, new);
                let k = match k {
                    serde_yaml::Value::String(ks) if ks == old => serde_yaml::Value::String(new.to_string()),
                    other => other,
                };
                rebuilt.insert(k, val);
            }
            *m = rebuilt;
        }
        _ => {}
    }
}

/// One full-line `TextEdit` per line where `old` occurs as a whole token (not a
/// substring of a longer token — e.g. renaming `REQ-FX-001` must not touch
/// `REQ-FX-0011` if such an id existed).
fn replace_whole_word_edits(text: &str, old: &str, new: &str) -> Option<Vec<TextEdit>> {
    let old_chars: Vec<char> = old.chars().collect();
    let mut edits = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if !line.contains(old) {
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::new();
        let mut j = 0;
        let mut changed = false;
        while j < chars.len() {
            if chars[j..].starts_with(old_chars.as_slice()) {
                let before_ok = j == 0 || !super::is_tok_char(chars[j - 1]);
                let after = j + old_chars.len();
                let after_ok = after >= chars.len() || !super::is_tok_char(chars[after]);
                if before_ok && after_ok {
                    out.push_str(new);
                    j = after;
                    changed = true;
                    continue;
                }
            }
            out.push(chars[j]);
            j += 1;
        }
        if changed {
            edits.push(TextEdit {
                range: Range::new(Position::new(i as u32, 0), Position::new(i as u32, chars.len() as u32)),
                new_text: out,
            });
        }
    }
    if edits.is_empty() {
        None
    } else {
        Some(edits)
    }
}

fn refuse_if_name_identified(elem: &RawElement) -> Result<(), String> {
    let type_str = elem.frontmatter.element_type.as_ref().map(type_label).unwrap_or("");
    if is_stable_id_type_name(type_str) {
        Ok(())
    } else {
        Err(format!(
            "'{type_str}' elements are name-identified — a rename here would be a file move, out of scope for textDocument/rename (use `syscribe mcp move_element`)"
        ))
    }
}

pub(super) fn prepare(store: &LspStore, path: &Path, position: Position) -> Result<PrepareRenameResponse, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let line = text.lines().nth(position.line as usize).ok_or_else(|| "position out of range".to_string())?;
    let (token, start, end) =
        super::token_at_with_range(line, position.character as usize).ok_or_else(|| "no identifier at this position".to_string())?;
    let elem = store
        .resolver
        .resolve_ref(&store.elements, &token)
        .ok_or_else(|| format!("'{token}' does not resolve to any element"))?;
    refuse_if_name_identified(elem)?;
    Ok(PrepareRenameResponse::RangeWithPlaceholder {
        range: Range::new(Position::new(position.line, start as u32), Position::new(position.line, end as u32)),
        placeholder: token,
    })
}

pub(super) fn compute(store: &LspStore, path: &Path, position: Position, new_name: &str) -> Result<WorkspaceEdit, String> {
    let target = super::resolve_token_element(store, path, position).ok_or_else(|| "no renameable identifier at this position".to_string())?;
    refuse_if_name_identified(target)?;
    let old_id = target.frontmatter.id.clone().ok_or_else(|| "target has no stable id".to_string())?;
    let target_qname = target.qualified_name.clone();

    let mut candidate: Vec<RawElement> = Vec::with_capacity(store.elements.len());
    for e in &store.elements {
        if e.qualified_name == target_qname || value_contains_needle(&serde_yaml::to_value(&e.frontmatter).unwrap_or_default(), &old_id) {
            let mut e2 = e.clone();
            if let Ok(mut val) = serde_yaml::to_value(&e2.frontmatter) {
                replace_needle(&mut val, &old_id, new_name);
                if let Ok(fm) = serde_yaml::from_value(val) {
                    e2.frontmatter = fm;
                }
            }
            candidate.push(e2);
        } else {
            candidate.push(e.clone());
        }
    }

    // Compared by (code, file) only, not the message text: renaming the id changes
    // the *message* of any pre-existing finding that happens to mention it (e.g. a
    // W005 "possible orphan" warning naming the old id) without that finding being
    // genuinely new. The file path itself is stable across an id rename (only the
    // `id:` value changes, not the file), so (code, file) identifies "the same
    // underlying issue, before and after" precisely enough for this gate.
    let before = validate_with_config(&store.elements, &store.config).findings;
    let after = validate_with_config(&candidate, &store.config).findings;
    if let Some(new_finding) = after.iter().find(|f| !before.iter().any(|b| b.code == f.code && b.file == f.file)) {
        return Err(format!("rename refused: it would introduce {} in {}: {}", new_finding.code, new_finding.file, new_finding.message));
    }

    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    for e in &store.elements {
        let is_target = e.qualified_name == target_qname;
        if !is_target && !value_contains_needle(&serde_yaml::to_value(&e.frontmatter).unwrap_or_default(), &old_id) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&e.file_path) else { continue };
        if let Some(edits) = replace_whole_word_edits(&text, &old_id, new_name) {
            if let Some(uri) = super::path_to_uri(&e.file_path) {
                changes.insert(uri, edits);
            }
        }
    }

    Ok(WorkspaceEdit { changes: Some(changes), ..Default::default() })
}

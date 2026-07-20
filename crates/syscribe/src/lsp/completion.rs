//! `textDocument/completion` (ADR-SYS-LSP-002): field-aware id/qname candidates for
//! cross-reference fields (REQ-TRS-LSP-008), plus enum candidates for `type`/`status`
//! (REQ-TRS-LSP-009).

use std::path::Path;

use tower_lsp::lsp_types::*;

use crate::query::type_label;
use syscribe_model::element::RawElement;

use super::store::LspStore;

/// For a cross-reference field, the element-type label(s) a valid value must resolve
/// to, given the enclosing element's own type label. `None` means the field is not a
/// recognized element cross-reference (no field-aware filtering applies) — notably
/// `implementedBy`, which is a source-artifact **path**, not a model element reference.
fn cross_ref_target_types(field: &str, enclosing_type: &str) -> Option<Vec<String>> {
    match field {
        "supertype" | "subsets" | "redefines" => Some(vec![enclosing_type.to_string()]),
        "typedBy" => Some(vec![if enclosing_type.ends_with("Def") {
            enclosing_type.to_string()
        } else {
            format!("{enclosing_type}Def")
        }]),
        // satisfies/verifies/derivedFrom all point upstream to a Requirement
        // (§12.1 OSLC link direction).
        "derivedFrom" | "verifies" | "satisfies" => Some(vec!["Requirement".to_string()]),
        "breakdownAdr" => Some(vec!["ADR".to_string()]),
        _ => None,
    }
}

/// The valid `status:` values for an element type, sourced from the same domain table
/// `mcp`'s `describe_type`/`template` tools already use — not a second hand-maintained
/// copy. Empty when the type has no defined status domain.
fn status_domain(type_name: &str) -> Vec<String> {
    crate::mcp::type_field_specs(type_name)
        .into_iter()
        .find(|f| f.get("name").and_then(|n| n.as_str()) == Some("status"))
        .and_then(|f| f.get("enum").cloned())
        .and_then(|e| e.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect()
}

/// The nearest enclosing top-level frontmatter key for `line` (0-indexed), or `None`
/// if `line` is outside the frontmatter block. A "top-level" line has no leading
/// whitespace and contains `:` — anything indented under it (list items, nested
/// values) belongs to that key.
fn enclosing_frontmatter_field(lines: &[&str], line: usize) -> Option<String> {
    if lines.first().copied() != Some("---") {
        return None;
    }
    let close = lines.iter().enumerate().skip(1).find(|(_, l)| **l == "---").map(|(i, _)| i)?;
    if line == 0 || line >= close {
        return None;
    }
    for l in lines.iter().take(line + 1).skip(1).rev() {
        if !l.starts_with(char::is_whitespace) {
            if let Some((key, _)) = l.split_once(':') {
                let key = key.trim();
                if !key.is_empty() {
                    return Some(key.to_string());
                }
            }
        }
    }
    None
}

pub(super) fn candidates(store: &LspStore, path: &Path, position: Position) -> Option<CompletionResponse> {
    let text = std::fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = text.lines().collect();
    let field = enclosing_frontmatter_field(&lines, position.line as usize)?;

    let path_str = path.to_string_lossy();
    let enclosing: Option<&RawElement> = store.elements.iter().find(|e| e.file_path == path_str);
    let enclosing_type = enclosing.and_then(|e| e.frontmatter.element_type.as_ref()).map(type_label).unwrap_or("");

    if field == "status" {
        let items: Vec<CompletionItem> = status_domain(enclosing_type)
            .into_iter()
            .map(|s| CompletionItem { label: s, kind: Some(CompletionItemKind::ENUM_MEMBER), ..Default::default() })
            .collect();
        return Some(CompletionResponse::Array(items));
    }
    if field == "type" {
        let items: Vec<CompletionItem> = crate::mcp::known_type_names()
            .into_iter()
            .map(|t| CompletionItem { label: t.to_string(), kind: Some(CompletionItemKind::ENUM_MEMBER), ..Default::default() })
            .collect();
        return Some(CompletionResponse::Array(items));
    }

    let target_types = cross_ref_target_types(&field, enclosing_type)?;
    let items: Vec<CompletionItem> = store
        .elements
        .iter()
        .filter(|e| {
            e.frontmatter
                .element_type
                .as_ref()
                .map(type_label)
                .map(|t| target_types.iter().any(|tt| tt == t))
                .unwrap_or(false)
        })
        .map(|e| CompletionItem {
            label: e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone()),
            detail: e.frontmatter.name.clone(),
            kind: Some(CompletionItemKind::REFERENCE),
            ..Default::default()
        })
        .collect();
    Some(CompletionResponse::Array(items))
}

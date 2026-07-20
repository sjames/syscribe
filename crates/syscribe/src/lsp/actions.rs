//! `textDocument/codeLens`, `textDocument/codeAction`, and the `syscribe.suspectAccept`
//! `workspace/executeCommand` backend (ADR-SYS-LSP-003).

use std::path::Path;

use tower_lsp::lsp_types::*;

use syscribe_model::suspect::LinkState;
use syscribe_model::validator::validate_with_config;

use super::store::LspStore;

fn ranges_overlap(a: &Range, b: &Range) -> bool {
    a.start.line <= b.end.line && b.start.line <= a.end.line
}

// ── codeLens (REQ-TRS-LSP-013) ────────────────────────────────────────────────

/// Display-only lenses: `Command.title` is the visible text, `Command.command` is an
/// empty no-op sentinel (the LSP type requires a command identifier, but this phase
/// attaches no click behavior — drilling in uses `textDocument/references`, v1).
pub(super) fn lenses_for(store: &LspStore, path: &Path) -> Vec<CodeLens> {
    let path_str = path.to_string_lossy().to_string();
    let Some(elem) = store.elements.iter().find(|e| e.file_path == path_str) else {
        return vec![];
    };
    let result = validate_with_config(&store.elements, &store.config);
    let mut parts: Vec<String> = Vec::new();
    if let Some(id) = &elem.frontmatter.id {
        if let Some(v) = result.verified_by.get(id) {
            if !v.is_empty() {
                parts.push(format!("{} verifiedBy", v.len()));
            }
        }
        if let Some(v) = result.derived_children.get(id) {
            if !v.is_empty() {
                parts.push(format!("{} derivedChildren", v.len()));
            }
        }
    }
    let suspect_count = syscribe_model::suspect::scan(&store.elements, &store.resolver)
        .into_iter()
        .filter(|l| l.source_file == path_str && l.state == LinkState::Suspect)
        .count();
    if suspect_count > 0 {
        parts.push(format!("{suspect_count} suspect link{}", if suspect_count == 1 { "" } else { "s" }));
    }
    if parts.is_empty() {
        return vec![];
    }
    vec![CodeLens {
        range: super::frontmatter_range(path),
        command: Some(Command { title: parts.join(" · "), command: String::new(), arguments: None }),
        data: None,
    }]
}

// ── codeAction (REQ-TRS-LSP-014/015/016) ──────────────────────────────────────

pub(super) fn actions_for(store: &LspStore, path: &Path, requested_range: &Range) -> Vec<CodeActionOrCommand> {
    let fm_range = super::frontmatter_range(path);
    if !ranges_overlap(requested_range, &fm_range) {
        return vec![];
    }
    let path_str = path.to_string_lossy().to_string();
    let Some(elem) = store.elements.iter().find(|e| e.file_path == path_str) else {
        return vec![];
    };

    let mut actions: Vec<CodeActionOrCommand> = Vec::new();

    // E310: Requirement with derivedFrom set and no breakdownAdr — one action per
    // accepted ADR (the target is inherently ambiguous otherwise; the lightbulb menu
    // resolves it).
    let is_requirement = elem.frontmatter.element_type.as_ref().map(crate::query::type_label) == Some("Requirement");
    let missing_breakdown_adr =
        elem.frontmatter.derived_from.as_ref().is_some_and(|d| !d.is_empty()) && elem.frontmatter.breakdown_adr.is_none();
    if is_requirement && missing_breakdown_adr {
        let insert_line = fm_range.end.line.saturating_sub(1);
        for adr in store
            .elements
            .iter()
            .filter(|e| e.frontmatter.element_type.as_ref().map(crate::query::type_label) == Some("ADR"))
            .filter(|e| e.frontmatter.status.as_deref() == Some("accepted"))
        {
            let name = adr.frontmatter.name.clone().unwrap_or_else(|| adr.qualified_name.clone());
            let mut changes = std::collections::HashMap::new();
            if let Some(uri) = super::path_to_uri(&elem.file_path) {
                changes.insert(
                    uri,
                    vec![TextEdit {
                        range: Range::new(Position::new(insert_line, 0), Position::new(insert_line, 0)),
                        new_text: format!("breakdownAdr: {}\n", adr.qualified_name),
                    }],
                );
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Set breakdownAdr: {} ({name})", adr.qualified_name),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: None,
                    edit: Some(WorkspaceEdit { changes: Some(changes), ..Default::default() }),
                    command: None,
                    is_preferred: None,
                    disabled: None,
                    data: None,
                }));
            }
        }
    }

    // W090: one "Accept as reviewed" command action per stale link sourced from this
    // file, dispatched through workspace/executeCommand (a real side effect — the hash
    // write — that only the server can compute).
    let stale: Vec<_> = syscribe_model::suspect::scan(&store.elements, &store.resolver)
        .into_iter()
        .filter(|l| l.source_file == path_str && l.state == LinkState::Suspect)
        .collect();
    for link in &stale {
        let title = if stale.len() == 1 {
            "Accept as reviewed".to_string()
        } else {
            format!("Accept as reviewed: {} {}", link.kind, link.target_ref)
        };
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title,
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: None,
            command: Some(Command {
                title: "Accept as reviewed".to_string(),
                command: "syscribe.suspectAccept".to_string(),
                arguments: Some(vec![serde_json::json!(link.source_label()), serde_json::json!(link.target_ref)]),
            }),
            is_preferred: None,
            disabled: None,
            data: None,
        }));
    }

    actions
}

// ── syscribe.suspectAccept (workspace/executeCommand backend) ─────────────────

/// Runs the same `plan_accept` + `write_baseline` pair `mcp`'s guarded `suspect_accept`
/// tool already calls. No dry-run: the codeAction click that got here already is the
/// human-in-the-loop confirmation (ADR-SYS-LSP-003).
pub(super) fn execute_suspect_accept(store: &LspStore, source_ref: &str, target_ref: &str) -> Result<(), String> {
    let plan = crate::suspect::plan_accept(&store.elements, &store.resolver, source_ref, target_ref)?;
    crate::suspect::write_baseline(Path::new(&plan.source_file), &plan.authored_key, &plan.hash).map_err(|e| e.to_string())
}

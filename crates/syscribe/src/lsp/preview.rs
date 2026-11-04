//! Preview before write for the front end's mutating actions (GH #426, `REQ-TRS-LSPPREVIEW-001`).
//!
//! * `syscribe/preview` (read-only) — plans a `bind` or a `scaffold`, previews it on a candidate copy through the
//!   guarded-write engine, and returns the report (diff, findings the change would add or resolve), whether it is
//!   `commitable`, a `WorkspaceEdit` the client can apply itself (the editor's own undo then applies), and an `undo`.
//! * `workspace/executeCommand` `syscribe.apply` — the same plan, committed only when it adds no error; returns the
//!   report and an undo `WorkspaceEdit`. A rejected commit writes nothing.
//!
//! The server writes only through `syscribe.apply` (ADR-SYS-LSP-003's single exception, extended to this command).

use std::path::PathBuf;

use serde_json::{json, Value};
use tower_lsp::lsp_types::*;

use super::store::LspStore;

/// A planned action: its report and the files it touches (path, content now if any, content after).
pub(super) struct Planned {
    pub report: Value,
    pub files: Vec<(PathBuf, Option<String>, String)>,
    pub commitable: bool,
}

fn str_of(p: &Value, k: &str) -> Option<String> {
    p.get(k).and_then(Value::as_str).map(str::to_string)
}

fn scaffold_request(p: &Value) -> Result<crate::newcmd::Request, String> {
    use crate::newcmd::Request;
    let kind = str_of(p, "kind").ok_or("scaffold needs `kind` (feature, configuration or parameter)")?;
    let name = str_of(p, "name").ok_or("scaffold needs `name`")?;
    Ok(match kind.as_str() {
        "feature" => Request::Feature { name, under: str_of(p, "under").ok_or("kind=feature needs `under`")?, group: str_of(p, "group"), mandatory: p.get("mandatory").and_then(Value::as_bool).unwrap_or(false), id: str_of(p, "id") },
        "configuration" => Request::Configuration {
            id: name,
            name: str_of(p, "displayName"),
            from: str_of(p, "from"),
            stage: str_of(p, "stage"),
            feature_model: str_of(p, "featureModel"),
            under: str_of(p, "under"),
            select: p.get("select").and_then(Value::as_object).map(|m| m.iter().filter_map(|(k, v)| v.as_bool().map(|b| (k.clone(), b))).collect()).unwrap_or_default(),
        },
        "parameter" => Request::Parameter {
            feature: str_of(p, "feature").ok_or("kind=parameter needs `feature`")?,
            name,
            ty: str_of(p, "type"),
            unit: str_of(p, "unit"),
            range: str_of(p, "range"),
            enum_values: p.get("enumValues").and_then(Value::as_array).map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            default: str_of(p, "default"),
            required: p.get("required").and_then(Value::as_bool).unwrap_or(false),
            stage: str_of(p, "paramStage"),
            latest: str_of(p, "latest"),
        },
        other => return Err(format!("unknown kind '{other}' (feature, configuration, parameter)")),
    })
}

/// Plan and preview the action in `params` (`{action: "bind" | "scaffold", …}`).
pub(super) fn plan(store: &LspStore, params: &Value) -> Result<Planned, String> {
    let action = str_of(params, "action").ok_or("`action` is required: bind or scaffold")?;
    match action.as_str() {
        "bind" => {
            let req = crate::bind::Request {
                configuration: str_of(params, "configuration").ok_or("bind needs `configuration`")?,
                target: str_of(params, "target").ok_or("bind needs `target`")?,
                value: params.get("value").cloned().ok_or("bind needs `value`")?,
            };
            let plan = crate::bind::plan(&store.elements, &req).map_err(|(_, m)| m)?;
            let path = PathBuf::from(&plan.file);
            let old = std::fs::read_to_string(&path).ok();
            let outcome = crate::preview::preview_files(&store.model_root, &store.elements, &store.config, &[(path.clone(), plan.new_content.clone())])?;
            let n = crate::preview::new_error_count(&outcome);
            let refused = (n > 0).then(|| format!("refused: the binding would introduce {n} error(s)"));
            Ok(Planned { report: crate::bind::report(&plan, &req.value, &outcome, false, refused.as_deref()), files: vec![(path, old, plan.new_content)], commitable: refused.is_none() })
        }
        "scaffold" => {
            let req = scaffold_request(params)?;
            let plan = crate::newcmd::plan(&store.model_root, &store.elements, &req).map_err(|(_, m)| m)?;
            let files: Vec<(PathBuf, String)> = plan.files.iter().map(|f| (f.path.clone(), f.content.clone())).collect();
            let outcome = crate::preview::preview_files(&store.model_root, &store.elements, &store.config, &files)?;
            let n = crate::preview::new_error_count(&outcome);
            let refused = (n > 0).then(|| format!("refused: it would introduce {n} error(s)"));
            let report = crate::newcmd::report(&plan, &outcome, false, refused.as_deref(), &store.model_root);
            let files = plan.files.iter().map(|f| (f.path.clone(), if f.creates { None } else { std::fs::read_to_string(&f.path).ok() }, f.content.clone())).collect();
            Ok(Planned { report, files, commitable: refused.is_none() })
        }
        other => Err(format!("unknown action '{other}' (bind, scaffold)")),
    }
}

/// One edit turning `old` into `new`: the lines between their common prefix and suffix are replaced.
fn diff_edit(old: &str, new: &str) -> Option<TextEdit> {
    if old == new {
        return None;
    }
    let (a, b): (Vec<&str>, Vec<&str>) = (old.split_inclusive('\n').collect(), new.split_inclusive('\n').collect());
    let pre = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let suf = a[pre..].iter().rev().zip(b[pre..].iter().rev()).take_while(|(x, y)| x == y).count();
    let replaced: String = b[pre..b.len() - suf].concat();
    Some(TextEdit { range: Range::new(Position::new(pre as u32, 0), Position::new((a.len() - suf) as u32, 0)), new_text: replaced })
}

fn doc_edit(uri: Url, edit: TextEdit) -> DocumentChangeOperation {
    DocumentChangeOperation::Edit(TextDocumentEdit { text_document: OptionalVersionedTextDocumentIdentifier { uri, version: None }, edits: vec![OneOf::Left(edit)] })
}

/// The edit that makes the planned change.
pub(super) fn edit_of(files: &[(PathBuf, Option<String>, String)]) -> WorkspaceEdit {
    let mut ops = Vec::new();
    for (path, old, new) in files {
        let Some(uri) = super::path_to_uri(&path.to_string_lossy()) else { continue };
        match old {
            None => {
                ops.push(DocumentChangeOperation::Op(ResourceOp::Create(CreateFile { uri: uri.clone(), options: Some(CreateFileOptions { overwrite: Some(false), ignore_if_exists: Some(false) }), annotation_id: None })));
                ops.push(doc_edit(uri, TextEdit { range: Range::new(Position::new(0, 0), Position::new(0, 0)), new_text: new.clone() }));
            }
            Some(old) => {
                if let Some(e) = diff_edit(old, new) {
                    ops.push(doc_edit(uri, e));
                }
            }
        }
    }
    WorkspaceEdit { document_changes: Some(DocumentChanges::Operations(ops)), ..Default::default() }
}

/// The edit that reverses [`edit_of`].
pub(super) fn undo_of(files: &[(PathBuf, Option<String>, String)]) -> WorkspaceEdit {
    let mut ops = Vec::new();
    for (path, old, new) in files {
        let Some(uri) = super::path_to_uri(&path.to_string_lossy()) else { continue };
        match old {
            None => ops.push(DocumentChangeOperation::Op(ResourceOp::Delete(DeleteFile { uri, options: Some(DeleteFileOptions { recursive: Some(false), ignore_if_not_exists: Some(true), annotation_id: None }) }))),
            Some(old) => {
                if let Some(e) = diff_edit(new, old) {
                    ops.push(doc_edit(uri, e));
                }
            }
        }
    }
    WorkspaceEdit { document_changes: Some(DocumentChanges::Operations(ops)), ..Default::default() }
}

/// The `syscribe/preview` payload.
pub(super) fn preview_json(p: &Planned) -> Value {
    json!({
        "schemaVersion": "1.0",
        "report": p.report,
        "commitable": p.commitable,
        "edit": serde_json::to_value(edit_of(&p.files)).unwrap_or(Value::Null),
        "undo": serde_json::to_value(undo_of(&p.files)).unwrap_or(Value::Null),
    })
}

/// Write the planned files. Nothing is written unless the plan is `commitable`.
pub(super) fn commit(p: &Planned) -> Result<(), String> {
    if !p.commitable {
        return Err(p.report["refused"].as_str().unwrap_or("refused").to_string());
    }
    for (path, _, new) in &p.files {
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d).map_err(|e| format!("cannot create {}: {e}", d.display()))?;
        }
        std::fs::write(path, new).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_edits_replace_only_the_changed_lines_and_invert() {
        let (a, b) = ("a\nb\nc\nd\n", "a\nB\nB2\nc\nd\n");
        let e = diff_edit(a, b).unwrap();
        assert_eq!((e.range.start.line, e.range.end.line, e.new_text.as_str()), (1, 2, "B\nB2\n"));
        // Applying it, then the inverse, round-trips.
        let apply = |text: &str, e: &TextEdit| {
            let ls: Vec<&str> = text.split_inclusive('\n').collect();
            format!("{}{}{}", ls[..e.range.start.line as usize].concat(), e.new_text, ls[e.range.end.line as usize..].concat())
        };
        assert_eq!(apply(a, &e), b);
        assert_eq!(apply(b, &diff_edit(b, a).unwrap()), a);
        assert!(diff_edit(a, a).is_none());
        // Appended and removed lines at the ends.
        assert_eq!(apply("x\n", &diff_edit("x\n", "x\ny\n").unwrap()), "x\ny\n");
        assert_eq!(apply("x\ny\n", &diff_edit("x\ny\n", "x\n").unwrap()), "x\n");
    }
}

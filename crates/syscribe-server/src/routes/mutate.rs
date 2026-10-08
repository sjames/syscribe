//! Guarded structural-edit and write endpoints (`ADR-SYS-DE-001`, `REQ-TRS-DE-002`/
//! `003`/`005`): create element, update element, delete element, patch
//! diagram layout, add connection, remove connection. All six funnel through
//! the shared `syscribe_model::mutate::guard::guarded_write` engine — the same
//! dry-run/candidate-copy/re-validate/commit-gate machinery
//! `crates/syscribe/src/mcp/write.rs` drives for the MCP write tools — reshaped
//! into Axum JSON handlers instead of MCP tool args/responses.
//!
//! `dry_run` defaults to **`false`** here, unlike MCP's default-`true`: these
//! endpoints back direct, immediate user actions in the diagram editor with
//! synchronous UI feedback (an accepted/rejected response right away), not an
//! LLM proposing a change for a human to review before committing. A caller
//! that wants a preview can still pass `dryRun: true` explicitly. `update_element`
//! and `patch_layout` (formerly `routes::write::put_element`/`patch_layout`,
//! unguarded raw-disk writes with no validation, no reload, no `/ws` broadcast)
//! now live here on the same engine as everything else in this module.
//!
//! ## Route shape deviation from `REQ-TRS-DE-002`
//!
//! The requirement names `POST/DELETE /api/elements/{*qname}/connections`.
//! That path is not expressible in axum/matchit: a catch-all path segment
//! (`{*qname}`) must be the last component of a route, so nothing can be
//! registered underneath it — verified empirically, registering such a route
//! panics at startup ("catch-all parameters are only allowed at the end of a
//! route" pre-registration, or "Insertion failed due to conflict with
//! previously registered route" once `/api/elements/{*qname}` already exists).
//! Connection add/remove instead extend the existing flat `/api/connections`
//! path (already used read-only by `routes::api_graph::get_connections` as
//! `?qname=`), taking the owning element's qualified name (`qname`) as a
//! request-body field instead of a URL path segment.
//!
//! ## Diagram frontmatter shapes (`REQ-TRS-DE-003`)
//!
//! `shapes:`/`edges:`/`layout:` are untyped `serde_yaml::Value` frontmatter
//! fields (`RawFrontmatter::shapes/edges/layout`); the one manifest parser,
//! `syscribe_model::vis::manifest` (`REQ-TRS-VIS-002`), reads them as:
//! - `shapes: { <shapeId>: { ref: <qname>, kind: <string> } }` (or the
//!   string shorthand `<shapeId>: <qname>`)
//! - `edges: { <edgeId>: { ref?: <qname>, source: <shapeId>, target: <shapeId>, kind: <string> } }`
//! - `layout: { <shapeId>: { x: <num>, y: <num>, w?: <num>, h?: <num> } }`
//!
//! The helpers below hand-build/patch YAML mappings matching those exact
//! shapes — they *write* YAML rather than build the IR — but the key names
//! (`shapes`/`edges`/`layout`/`ref`/`kind`) are shared via that module's
//! `KEY_*` constants so this write side and the parser agree on one spelling.
//! A section that is present but not a map is never silently replaced
//! (`sub_mapping` refuses, and the guarded write with it): that would destroy
//! the author's data, and the validator already reports it as `E405`.

use std::collections::HashMap;
use std::path::Path;

use axum::{
    extract::{Path as AxumPath, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use syscribe_model::connections::{add_connection as add_conn_entry, remove_connection as remove_conn_entry};
use syscribe_model::vis::manifest::{KEY_EDGES, KEY_KIND, KEY_LAYOUT, KEY_REF, KEY_SHAPES};
use syscribe_model::element::ElementType;
use syscribe_model::frontmatter::patch_frontmatter;
use syscribe_model::mutate::{
    apply_update_fields, plan_create, referrers, write_confined, Entry, GuardedWriteOutcome,
};
use syscribe_model::resolver::Resolver;
use syscribe_model::walker::walk_model;

use crate::state::SharedState;

// ---------------------------------------------------------------------------
// Response shape: mirrors MCP's write-tool JSON, flattened (no
// `validationDelta` wrapper — the field names are hoisted to the top level
// per this task's spec) and camelCase, matching this server's `get_element`
// JSON convention (`routes::elements::get_element` already emits
// `qualifiedName`/`filePath` camelCase).
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct FindingJson {
    pub code: String,
    pub severity: String,
    pub file: String,
    pub message: String,
}

/// One response convention for every guarded write in this module, refusal
/// included: **always `StatusCode::OK`**, `written:false` + `reason` (and, for
/// `delete_element`'s referrer-blocked case, a populated `blocked_by`) instead
/// of a 4xx/5xx. A real HTTP status code stays reserved for genuine
/// transport-level facts axum itself already owns (malformed JSON body,
/// unregistered route) — never for a business-logic refusal, so a client can
/// always `.json()` the body without first branching on status.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteResponse {
    pub written: bool,
    pub new_errors: Vec<FindingJson>,
    pub resolved_errors: Vec<FindingJson>,
    pub new_warnings: Vec<FindingJson>,
    pub resolved_warnings: Vec<FindingJson>,
    pub diff: String,
    pub reason: Option<String>,
    /// Populated only by `delete_element`'s referrer-blocked refusal; empty
    /// (and still present on the wire, not omitted) for every other outcome.
    #[serde(default)]
    pub blocked_by: Vec<BlockedByEntry>,
}

fn findings(entries: &[Entry], severity: &str) -> Vec<FindingJson> {
    entries
        .iter()
        .map(|(code, file, message)| FindingJson {
            code: code.clone(),
            severity: severity.to_string(),
            file: file.clone(),
            message: message.clone(),
        })
        .collect()
}

fn to_response(outcome: &GuardedWriteOutcome) -> WriteResponse {
    WriteResponse {
        written: outcome.written,
        new_errors: findings(&outcome.new_errors, "error"),
        resolved_errors: findings(&outcome.resolved_errors, "error"),
        new_warnings: findings(&outcome.new_warnings, "warning"),
        resolved_warnings: findings(&outcome.resolved_warnings, "warning"),
        diff: outcome.diff.clone(),
        reason: outcome.reason.clone(),
        blocked_by: vec![],
    }
}

/// A refusal computed before any candidate work was staged (invalid qname,
/// unresolved reference, …) — empty delta/diff, mirrors MCP's `write::refuse`.
fn refused(reason: impl Into<String>) -> WriteResponse {
    WriteResponse {
        written: false,
        new_errors: vec![],
        resolved_errors: vec![],
        new_warnings: vec![],
        resolved_warnings: vec![],
        diff: String::new(),
        reason: Some(reason.into()),
        blocked_by: vec![],
    }
}

/// `delete_element`'s referrer-blocked refusal: same always-200/`written:false`
/// convention as [`refused`], but with `blocked_by` populated so the client
/// can list what's still referencing the target.
fn blocked(reason: impl Into<String>, blocked_by: Vec<BlockedByEntry>) -> WriteResponse {
    WriteResponse {
        written: false,
        new_errors: vec![],
        resolved_errors: vec![],
        new_warnings: vec![],
        resolved_warnings: vec![],
        diff: String::new(),
        reason: Some(reason.into()),
        blocked_by,
    }
}

/// `update_element`/`delete_element` refuse a synthesized element (see their
/// call sites) — same posture as the unresolved-reference refusal they
/// already return, no new response shape needed. See
/// `syscribe_model::walker::is_synthesized`'s doc comment for why.
fn is_synthesized(elem: &syscribe_model::element::RawElement, model_root: &Path) -> bool {
    let rel = rel_to_root(&elem.file_path, model_root);
    syscribe_model::walker::is_synthesized(elem, Path::new(&rel))
}

/// Normalise a `RawElement::file_path` (always rooted at whatever `model_root`
/// the store was loaded from) to a path relative to that root, for joining
/// against a *different* root (the guarded-write candidate copy) later.
fn rel_to_root(file_path: &str, root: &Path) -> String {
    let root_s = root.to_string_lossy();
    file_path
        .strip_prefix(root_s.as_ref())
        .map(|s| s.trim_start_matches(['/', '\\']).to_string())
        .unwrap_or_else(|| file_path.to_string())
}

/// Resolve `qname` against a fresh walk of `root` and return its file path.
/// `walk_model(root)`'s `file_path`s are always rooted at whichever `root` was
/// passed in (the candidate copy or the real model root), so the returned
/// path is directly usable against that same `root` with no further joining.
fn resolve_file(root: &Path, qname: &str) -> Result<String, String> {
    let elems = walk_model(root).map_err(|e| e.to_string())?;
    let resolver = Resolver::new(&elems);
    resolver
        .resolve_ref(&elems, qname)
        .map(|e| e.file_path.clone())
        .ok_or_else(|| format!("unresolved reference: {qname}"))
}

// ---------------------------------------------------------------------------
// Request / diagram-context types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeDiagramContext {
    pub qname: String,
    pub shape_id: String,
    pub x: f64,
    pub y: f64,
    pub kind: String,
}

/// `sourceShapeId`/`targetShapeId` are required to add an edge; remove only
/// needs `qname` + `edgeId`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeDiagramContext {
    pub qname: String,
    pub edge_id: String,
    pub source_shape_id: Option<String>,
    pub target_shape_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateElementRequest {
    pub qname: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub fields: Option<serde_json::Value>,
    pub doc: Option<String>,
    pub diagram: Option<ShapeDiagramContext>,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeleteElementQuery {
    #[serde(default)]
    pub force: bool,
    #[serde(default)]
    pub dry_run: bool,
}

/// `PUT /api/elements/{*qname}` body (mirrors `CreateElementRequest`'s shape).
/// `fields` merges into the frontmatter mapping — an explicit JSON `null`
/// value removes a key, any other value is inserted/overwritten — via
/// `syscribe_model::mutate::apply_update_fields`; `doc`, if given, replaces
/// the Markdown body wholesale.
///
/// `extraYaml`, if given, is raw YAML text (the detail panel's `#edit-extra`
/// textarea, Phase 8) parsed **server-side** into a mapping and merged
/// underneath `fields` (a key present in both loses to `fields`, though in
/// practice the two don't overlap: the textarea is pre-filled with the
/// element's frontmatter *minus* `name`, and `fields` is where the UI puts
/// `name`). This is a deliberate choice over parsing YAML client-side: the
/// textarea can legitimately contain nested sequences/mappings (`features:`,
/// `connections:`, `shapes:`, …), and reimplementing even a YAML subset
/// parser in JS risked being wrong in exactly the cases — indentation,
/// nested lists — that matter most, where `serde_yaml` here is already
/// correct and load-bearing for every other write path in this file.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateElementRequest {
    pub fields: Option<serde_json::Value>,
    pub extra_yaml: Option<String>,
    pub doc: Option<String>,
    #[serde(default)]
    pub dry_run: bool,
}

/// Parse `extra_yaml` (if given and non-blank) into a JSON object and layer
/// `fields` on top of it. Returns `Err` (a pre-staging refusal, same as an
/// invalid qname elsewhere in this module) if `extra_yaml` doesn't parse as
/// YAML or doesn't parse to a mapping — a malformed edit must never reach
/// `apply_update_fields`.
fn merge_extra_yaml(
    fields: Option<serde_json::Value>,
    extra_yaml: Option<&str>,
) -> Result<Option<serde_json::Value>, String> {
    let extra_obj = match extra_yaml.map(str::trim) {
        Some(y) if !y.is_empty() => {
            let parsed: serde_yaml::Value =
                serde_yaml::from_str(y).map_err(|e| format!("invalid YAML in extra fields: {e}"))?;
            match parsed {
                serde_yaml::Value::Mapping(_) => Some(
                    serde_json::to_value(&parsed)
                        .map_err(|e| format!("could not convert extra fields to JSON: {e}"))?,
                ),
                serde_yaml::Value::Null => None,
                _ => return Err("extra fields YAML must be a mapping".to_string()),
            }
        }
        _ => None,
    };
    match (extra_obj, fields) {
        (Some(serde_json::Value::Object(mut extra_map)), Some(serde_json::Value::Object(f))) => {
            for (k, v) in f {
                extra_map.insert(k, v);
            }
            Ok(Some(serde_json::Value::Object(extra_map)))
        }
        (Some(extra), None) => Ok(Some(extra)),
        (None, fields) => Ok(fields),
        // `fields` wasn't a JSON object (shouldn't happen from this UI, but a
        // programmatic caller could send something else) — let it win as-is
        // rather than silently dropping it.
        (Some(_), fields @ Some(_)) => Ok(fields),
    }
}

/// One value of the `PATCH /api/diagrams/layout/{*qname}` body: shape id ->
/// `{x, y, w?, h?}` writes a pin (`w`/`h` only when given); the map value
/// `null` removes that shape's pin (`REQ-TRS-VIS-006`).
#[derive(Debug, Deserialize)]
pub struct PositionUpdate {
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub w: Option<f64>,
    #[serde(default)]
    pub h: Option<f64>,
}

/// `PUT /api/diagrams/svg/{*qname}` body (`REQ-TRS-VIS-011`, *Save companion
/// SVG*): the serialised SVG document to write as the diagram's companion.
#[derive(Debug, Deserialize)]
pub struct PutSvgRequest {
    pub svg: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockedByEntry {
    pub qname: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddConnectionRequest {
    pub qname: String,
    pub from: String,
    pub to: String,
    pub typed_by: Option<String>,
    pub diagram: Option<EdgeDiagramContext>,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveConnectionRequest {
    pub qname: String,
    pub from: String,
    pub to: String,
    pub diagram: Option<EdgeDiagramContext>,
    #[serde(default)]
    pub dry_run: bool,
}

// ---------------------------------------------------------------------------
// Diagram-sync helpers (REQ-TRS-DE-003)
// ---------------------------------------------------------------------------

/// Fetch-or-create the sub-mapping at `map[key]`. An absent or `null` value
/// becomes an empty map; a value of any other shape (a scalar, a sequence) is
/// refused with a message naming the key, so a guarded write never replaces
/// an author's malformed-but-present section with an empty one.
fn sub_mapping<'m>(map: &'m mut serde_yaml::Mapping, key: &str) -> Result<&'m mut serde_yaml::Mapping, String> {
    match map.get(key) {
        None | Some(serde_yaml::Value::Null) => {
            map.insert(key.into(), serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
        }
        Some(serde_yaml::Value::Mapping(_)) => {}
        Some(other) => {
            let shape = match other {
                serde_yaml::Value::Sequence(_) => "a sequence",
                serde_yaml::Value::String(_) => "a string",
                serde_yaml::Value::Number(_) => "a number",
                serde_yaml::Value::Bool(_) => "a boolean",
                _ => "not a map",
            };
            return Err(format!(
                "`{key}:` is {shape}, not a map of entries — fix the frontmatter (validate reports it as E405) before editing it from a diagram"
            ));
        }
    }
    match map.get_mut(key) {
        Some(serde_yaml::Value::Mapping(m)) => Ok(m),
        _ => unreachable!("just ensured a Mapping is present at this key"),
    }
}

/// Run `patch_frontmatter` with a mutation that may refuse (a `sub_mapping`
/// error): the refusal is propagated and the file is left untouched.
fn patch_frontmatter_checked(
    content: &str,
    mutate: impl FnOnce(&mut serde_yaml::Mapping) -> Result<(), String>,
) -> Result<String, String> {
    let mut refused: Option<String> = None;
    let new_content = patch_frontmatter(content, None, |map| {
        if let Err(e) = mutate(map) {
            refused = Some(e);
        }
    })
    .map_err(|e| e.to_string())?;
    match refused {
        Some(e) => Err(e),
        None => Ok(new_content),
    }
}

/// Add `shapeId`'s shape + layout entry to `diagram`'s frontmatter, pointing
/// at `new_qname` — creating an element from within a diagram.
fn sync_shape_add(root: &Path, d: &ShapeDiagramContext, new_qname: &str, pinned: bool) -> Result<(), String> {
    let file = resolve_file(root, &d.qname)?;
    let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
    let new_content = patch_frontmatter_checked(&content, |map| {
        let shapes = sub_mapping(map, KEY_SHAPES)?;
        let mut shape = serde_yaml::Mapping::new();
        shape.insert(KEY_REF.into(), new_qname.into());
        shape.insert(KEY_KIND.into(), d.kind.clone().into());
        shapes.insert(d.shape_id.clone().into(), serde_yaml::Value::Mapping(shape));

        if pinned {
            let layout = sub_mapping(map, KEY_LAYOUT)?;
            let mut pos = serde_yaml::Mapping::new();
            pos.insert("x".into(), d.x.into());
            pos.insert("y".into(), d.y.into());
            layout.insert(d.shape_id.clone().into(), serde_yaml::Value::Mapping(pos));
        }
        Ok(())
    })?;
    std::fs::write(&file, new_content).map_err(|e| e.to_string())
}

/// Remove every shape (and its layout entry) in every `Diagram` element under
/// `root` whose `ref` equals `deleted_qname`, and any edge whose `source`/
/// `target` names one of those removed shape ids — deleting an element.
fn sync_shapes_removed(root: &Path, deleted_qname: &str) -> Result<(), String> {
    let elems = walk_model(root).map_err(|e| e.to_string())?;
    for e in &elems {
        if e.frontmatter.element_type != Some(ElementType::Diagram) {
            continue;
        }
        let Some(serde_yaml::Value::Mapping(shapes)) = &e.frontmatter.shapes else {
            continue;
        };
        let removed_ids: Vec<String> = shapes
            .iter()
            .filter_map(|(id, v)| {
                let serde_yaml::Value::Mapping(sm) = v else { return None };
                let r = sm.get(KEY_REF).and_then(|v| v.as_str())?;
                if r != deleted_qname {
                    return None;
                }
                id.as_str().map(String::from)
            })
            .collect();
        if removed_ids.is_empty() {
            continue;
        }

        let content = std::fs::read_to_string(&e.file_path).map_err(|err| err.to_string())?;
        let new_content = patch_frontmatter(&content, None, |map| {
            if let Some(serde_yaml::Value::Mapping(sm)) = map.get_mut(KEY_SHAPES) {
                for id in &removed_ids {
                    sm.remove(id.as_str());
                }
            }
            if let Some(serde_yaml::Value::Mapping(lm)) = map.get_mut(KEY_LAYOUT) {
                for id in &removed_ids {
                    lm.remove(id.as_str());
                }
            }
            if let Some(serde_yaml::Value::Mapping(em)) = map.get_mut(KEY_EDGES) {
                let dead: Vec<String> = em
                    .iter()
                    .filter_map(|(k, v)| {
                        let serde_yaml::Value::Mapping(evm) = v else { return None };
                        let src = evm.get("source").and_then(|v| v.as_str())?;
                        let tgt = evm.get("target").and_then(|v| v.as_str())?;
                        let hit = removed_ids.iter().any(|id| id == src)
                            || removed_ids.iter().any(|id| id == tgt);
                        if hit {
                            k.as_str().map(String::from)
                        } else {
                            None
                        }
                    })
                    .collect();
                for k in dead {
                    em.remove(k.as_str());
                }
            }
        })
        .map_err(|err| err.to_string())?;
        std::fs::write(&e.file_path, new_content).map_err(|err| err.to_string())?;
    }
    Ok(())
}

/// Add one `{source, target, kind}` edge entry to `d.qname`'s `edges:` map —
/// connecting two ports from within a diagram. Requires `sourceShapeId`/
/// `targetShapeId` on the diagram context.
fn sync_edge_add(root: &Path, d: &EdgeDiagramContext) -> Result<(), String> {
    let src = d
        .source_shape_id
        .clone()
        .ok_or_else(|| "diagram.sourceShapeId is required to add an edge".to_string())?;
    let tgt = d
        .target_shape_id
        .clone()
        .ok_or_else(|| "diagram.targetShapeId is required to add an edge".to_string())?;
    let file = resolve_file(root, &d.qname)?;
    let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
    let new_content = patch_frontmatter_checked(&content, |map| {
        let edges = sub_mapping(map, KEY_EDGES)?;
        let mut edge = serde_yaml::Mapping::new();
        edge.insert("source".into(), src.clone().into());
        edge.insert("target".into(), tgt.clone().into());
        // No richer semantic kind is available from a bare port-to-port
        // connect gesture; `connection` is the manifest's default edge kind
        // (`EdgeKind::Connection`, spec §8.16.4).
        edge.insert(KEY_KIND.into(), "connection".into());
        edges.insert(d.edge_id.clone().into(), serde_yaml::Value::Mapping(edge));
        Ok(())
    })?;
    std::fs::write(&file, new_content).map_err(|e| e.to_string())
}

/// Remove `d.edgeId`'s entry from `d.qname`'s `edges:` map.
fn sync_edge_remove(root: &Path, d: &EdgeDiagramContext) -> Result<(), String> {
    let file = resolve_file(root, &d.qname)?;
    let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
    let new_content = patch_frontmatter(&content, None, |map| {
        if let Some(serde_yaml::Value::Mapping(em)) = map.get_mut(KEY_EDGES) {
            em.remove(d.edge_id.as_str());
        }
    })
    .map_err(|e| e.to_string())?;
    std::fs::write(&file, new_content).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// connections: sequence mutation helpers
// ---------------------------------------------------------------------------

fn apply_connection_add(
    root: &Path,
    owner_qname: &str,
    from: &str,
    to: &str,
    typed_by: Option<&str>,
) -> Result<(), String> {
    let file = resolve_file(root, owner_qname)?;
    let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
    let new_content = patch_frontmatter(&content, None, |map| {
        let mut seq = match map.get("connections") {
            Some(serde_yaml::Value::Sequence(s)) => s.clone(),
            _ => Vec::new(),
        };
        add_conn_entry(&mut seq, from, to, typed_by);
        map.insert("connections".into(), serde_yaml::Value::Sequence(seq));
    })
    .map_err(|e| e.to_string())?;
    std::fs::write(&file, new_content).map_err(|e| e.to_string())
}

/// Returns `Err` if no matching `connections:` entry was found (nothing to
/// remove) so the caller's `apply` closure refuses the whole guarded write
/// cleanly, the same way an unresolved reference does elsewhere in this module.
fn apply_connection_remove(root: &Path, owner_qname: &str, from: &str, to: &str) -> Result<(), String> {
    let file = resolve_file(root, owner_qname)?;
    let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
    let mut found = false;
    let new_content = patch_frontmatter(&content, None, |map| {
        let mut seq = match map.get("connections") {
            Some(serde_yaml::Value::Sequence(s)) => s.clone(),
            _ => Vec::new(),
        };
        found = remove_conn_entry(&mut seq, from, to);
        map.insert("connections".into(), serde_yaml::Value::Sequence(seq));
    })
    .map_err(|e| e.to_string())?;
    if !found {
        return Err(format!(
            "no connection between '{from}' and '{to}' on {owner_qname}"
        ));
    }
    std::fs::write(&file, new_content).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /api/elements` — create a new element file, optionally syncing a
/// diagram's shape/layout in the same commit (`REQ-TRS-DE-002`/`003`).
pub async fn create_element(
    State(state): State<SharedState>,
    Json(req): Json<CreateElementRequest>,
) -> Json<WriteResponse> {
    let mut store = state.write().await;

    let plan = match plan_create(
        &store.elements,
        &req.qname,
        &req.type_name,
        req.fields.as_ref(),
        req.doc.as_deref(),
    ) {
        Ok(p) => p,
        Err(e) => return Json(refused(e.to_string())),
    };

    let new_qname = req.qname.replace('/', "::");
    let rel = plan.rel.clone();
    let content = plan.content.clone();
    let diagram = req.diagram.clone();

    let apply = move |root: &Path| -> Result<(), String> {
        write_confined(root, &rel, &content).map_err(|e| e.to_string())?;
        if let Some(d) = &diagram {
            sync_shape_add(root, d, &new_qname, true)?;
        }
        Ok(())
    };

    let outcome = store.commit(
        req.dry_run,
        true, // gate: a created element's own references must still resolve
        false, // no env-var escape hatch on the server — the client controls
        // intent explicitly via dryRun instead
        apply,
    );
    Json(to_response(&outcome))
}

/// `DELETE /api/elements/{*qname}[?force=true][&dryRun=true]` — delete an
/// element, refusing when other elements still reference it unless
/// `force=true`, mirroring MCP's `delete_element` referrer check. Diagram
/// shapes/edges pointing at the deleted element are cleaned up in the same
/// commit across every `Diagram` element (`REQ-TRS-DE-003`).
///
/// Both refusal paths below (element not found, blocked by referrers) return
/// `StatusCode::OK` with `written:false` — this module's one response
/// convention (see the [`WriteResponse`] doc comment) — rather than the `404`/
/// `409` this handler used before this task's cleanup.
pub async fn delete_element(
    State(state): State<SharedState>,
    AxumPath(qname): AxumPath<String>,
    Query(q): Query<DeleteElementQuery>,
) -> Json<WriteResponse> {
    let mut store = state.write().await;
    let qname_norm = qname.replace('/', "::");

    let target = match store.elements.iter().find(|e| e.qualified_name == qname_norm) {
        Some(e) => e.clone(),
        None => return Json(refused(format!("unresolved reference: {qname_norm}"))),
    };
    if is_synthesized(&target, &store.model_root) {
        return Json(refused(format!(
            "'{qname_norm}' is synthesized from a shared sheet file ({}) — deleting it would delete every sibling entry in that file; edit the sheet directly instead",
            target.file_path
        )));
    }

    if !q.force {
        let refs = referrers(&store.elements, &target.qualified_name);
        if !refs.is_empty() {
            let blocked_by = refs
                .into_iter()
                .map(|(qn, id)| BlockedByEntry { qname: qn, id })
                .collect();
            return Json(blocked(
                "delete blocked by inbound references; pass force=true to override",
                blocked_by,
            ));
        }
    }

    let rel = rel_to_root(&target.file_path, &store.model_root);
    let target_qname = target.qualified_name.clone();

    let apply = move |root: &Path| -> Result<(), String> {
        std::fs::remove_file(root.join(&rel)).map_err(|e| e.to_string())?;
        sync_shapes_removed(root, &target_qname)
    };

    // gate=false: deletion may legitimately orphan a reference under `force`;
    // the referrer check above is delete's own safety net, not the
    // referential-integrity gate (mirrors MCP's delete_element rationale).
    let outcome = store.commit(q.dry_run, false, false, apply);
    Json(to_response(&outcome))
}

/// `POST /api/connections` — add one `connections:` entry to the owning
/// element (`qname` in the body), optionally syncing a diagram edge in the
/// same commit (`REQ-TRS-DE-002`/`003`). See the module doc comment for why
/// this is `/api/connections` rather than `/api/elements/{*qname}/connections`.
pub async fn add_connection(
    State(state): State<SharedState>,
    Json(req): Json<AddConnectionRequest>,
) -> Json<WriteResponse> {
    let mut store = state.write().await;

    let owner_qname = match store.resolver.resolve_ref(&store.elements, &req.qname) {
        Some(e) => e.qualified_name.clone(),
        None => return Json(refused(format!("unresolved reference: {}", req.qname))),
    };

    let from = req.from.clone();
    let to = req.to.clone();
    let typed_by = req.typed_by.clone();
    let diagram = req.diagram.clone();
    let apply = move |root: &Path| -> Result<(), String> {
        apply_connection_add(root, &owner_qname, &from, &to, typed_by.as_deref())?;
        if let Some(d) = &diagram {
            sync_edge_add(root, d)?;
        }
        Ok(())
    };

    let outcome = store.commit(req.dry_run, true, false, apply);
    Json(to_response(&outcome))
}

/// `DELETE /api/connections` — remove one `connections:` entry from the
/// owning element (`qname` in the body), optionally removing the
/// corresponding diagram edge in the same commit.
pub async fn remove_connection(
    State(state): State<SharedState>,
    Json(req): Json<RemoveConnectionRequest>,
) -> Json<WriteResponse> {
    let mut store = state.write().await;

    let owner_qname = match store.resolver.resolve_ref(&store.elements, &req.qname) {
        Some(e) => e.qualified_name.clone(),
        None => return Json(refused(format!("unresolved reference: {}", req.qname))),
    };

    let from = req.from.clone();
    let to = req.to.clone();
    let diagram = req.diagram.clone();
    let apply = move |root: &Path| -> Result<(), String> {
        apply_connection_remove(root, &owner_qname, &from, &to)?;
        if let Some(d) = &diagram {
            sync_edge_remove(root, d)?;
        }
        Ok(())
    };

    // gate=false: removing a connection can't introduce a dangling reference
    // (mirrors the delete-family rationale — there's nothing new to check).
    let outcome = store.commit(req.dry_run, false, false, apply);
    Json(to_response(&outcome))
}

/// `PUT /api/elements/{*qname}` — merge `fields`/`doc` into an existing
/// element's frontmatter and body (replaces `routes::write::put_element`,
/// which wrote straight to disk with no validation, reload, or `/ws`
/// broadcast). Unlike that handler, this is a full member of the guarded-write
/// family: dry-run preview, candidate-copy re-validation, and — because a
/// field update can point a reference (`supertype`, `typedBy`, …) at
/// something unresolved — `gate=true`, same reasoning as `create_element`.
pub async fn update_element(
    State(state): State<SharedState>,
    AxumPath(qname): AxumPath<String>,
    Json(req): Json<UpdateElementRequest>,
) -> Json<WriteResponse> {
    let mut store = state.write().await;
    let qname_norm = qname.replace('/', "::");

    let target = match store.elements.iter().find(|e| e.qualified_name == qname_norm) {
        Some(e) => e.clone(),
        None => return Json(refused(format!("unresolved reference: {qname_norm}"))),
    };
    if is_synthesized(&target, &store.model_root) {
        return Json(refused(format!(
            "'{qname_norm}' is synthesized from a shared sheet file ({}) — a field update here would patch the sheet's own top-level frontmatter, not this entry; edit the sheet directly instead",
            target.file_path
        )));
    }
    let rel = rel_to_root(&target.file_path, &store.model_root);

    let fields = match merge_extra_yaml(req.fields.clone(), req.extra_yaml.as_deref()) {
        Ok(f) => f,
        Err(e) => return Json(refused(e)),
    };
    let doc = req.doc.clone();
    let apply = move |root: &Path| -> Result<(), String> {
        let file = root.join(&rel);
        let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let new_content = apply_update_fields(&content, fields.as_ref(), doc.as_deref())
            .map_err(|e| e.to_string())?;
        std::fs::write(&file, new_content).map_err(|e| e.to_string())
    };

    // gate: an updated field can point a reference at something unresolved
    let outcome = store.commit(req.dry_run, true, false, apply);
    Json(to_response(&outcome))
}

/// `PATCH /api/diagrams/layout/{*qname}` — persist pins for one or more
/// shapes into the diagram element's `layout:` frontmatter map
/// (`REQ-TRS-VIS-006`): a `{x, y, w?, h?}` value writes `x`/`y` and, when
/// given, `w`/`h` (an absent `w`/`h` leaves any existing one untouched); a
/// `null` value removes that shape's pin. Built on `patch_frontmatter` + the
/// `sub_mapping` helper already used by this module's diagram-sync functions.
///
/// Unknown-qname handling: this used to be a genuine `404`. It moves onto the
/// same always-`OK`/`written:false` convention as every other handler in this
/// module instead of staying a status code — an unresolved-qname refusal is
/// exactly what `update_element` (the handler this one is structurally
/// closest to: both look up `qname` in `store.elements`, then patch a file)
/// already reports as `written:false`, and `create_element`/`add_connection`/
/// `remove_connection` do the same for *their* "the referenced thing doesn't
/// resolve" case. Treating patch_layout's identical situation as a special
/// last status-code holdout would reintroduce the very inconsistency this
/// task removes elsewhere; "the path segment doesn't parse to anything" is
/// meaningfully different from an unregistered *route* (still a real 404,
/// untouched) but not from any other handler's "no such element" refusal.
pub async fn patch_layout(
    State(state): State<SharedState>,
    AxumPath(qname): AxumPath<String>,
    Json(positions): Json<HashMap<String, Option<PositionUpdate>>>,
) -> Json<WriteResponse> {
    let mut store = state.write().await;
    let qname_norm = qname.replace('/', "::");

    let file_path = match store.elements.iter().find(|e| e.qualified_name == qname_norm) {
        Some(e) => e.file_path.clone(),
        None => return Json(refused(format!("unresolved reference: {qname_norm}"))),
    };
    let rel = rel_to_root(&file_path, &store.model_root);

    let apply = move |root: &Path| -> Result<(), String> {
        let file = root.join(&rel);
        let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let new_content = patch_frontmatter_checked(&content, |map| {
            let layout = sub_mapping(map, KEY_LAYOUT)?;
            for (shape_id, pos) in &positions {
                let Some(pos) = pos else {
                    // `null`: unpin. Removing a pin that isn't there is a no-op.
                    layout.remove(shape_id.as_str());
                    continue;
                };
                let shape = sub_mapping(layout, shape_id)?;
                let num = |v: f64| serde_yaml::Value::Number(serde_yaml::Number::from(v.round() as i64));
                shape.insert("x".into(), num(pos.x));
                shape.insert("y".into(), num(pos.y));
                if let Some(w) = pos.w {
                    shape.insert("w".into(), num(w));
                }
                if let Some(h) = pos.h {
                    shape.insert("h".into(), num(h));
                }
            }
            Ok(())
        })?;
        std::fs::write(&file, new_content).map_err(|e| e.to_string())
    };

    // gate=false: a layout patch only ever touches numeric x/y/w/h under
    // `layout:`, never a cross-reference field, so it cannot introduce a new
    // unresolved reference (same rationale as delete_element/remove_connection).
    let outcome = store.commit(
        false, // no dryRun in this endpoint's wire shape — always commits
        false,
        false,
        apply,
    );
    Json(to_response(&outcome))
}

/// Look up a `Diagram` element for the layout/svg routes: `Err` is the
/// refusal reason for an unknown qname or a non-`Diagram` element.
fn diagram_target(
    store: &crate::state::ModelStore,
    qname: &str,
) -> Result<syscribe_model::element::RawElement, String> {
    let qname_norm = qname.replace('/', "::");
    let target = store
        .elements
        .iter()
        .find(|e| e.qualified_name == qname_norm)
        .cloned()
        .ok_or_else(|| format!("unresolved reference: {qname_norm}"))?;
    if target.frontmatter.element_type != Some(ElementType::Diagram) {
        return Err(format!(
            "'{qname_norm}' is a {}, not a Diagram",
            target.frontmatter.element_type.as_ref().map(|t| t.name()).unwrap_or("untyped element")
        ));
    }
    Ok(target)
}

/// `DELETE /api/diagrams/layout/{*qname}` — remove every pin: the whole
/// `layout:` key is dropped from the diagram's frontmatter (the browser's
/// *Auto-layout* button, `REQ-TRS-VIS-006`/`-011`). A diagram with no
/// `layout:` is a successful no-op write. Same always-`OK`/`written:false`
/// refusal convention as the rest of this module for an unknown qname or a
/// non-`Diagram` element.
pub async fn delete_layout(
    State(state): State<SharedState>,
    AxumPath(qname): AxumPath<String>,
) -> Json<WriteResponse> {
    let mut store = state.write().await;
    let target = match diagram_target(&store, &qname) {
        Ok(t) => t,
        Err(reason) => return Json(refused(reason)),
    };
    let rel = rel_to_root(&target.file_path, &store.model_root);

    let apply = move |root: &Path| -> Result<(), String> {
        let file = root.join(&rel);
        let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let new_content = patch_frontmatter(&content, None, |map| {
            map.remove(KEY_LAYOUT);
        })
        .map_err(|e| e.to_string())?;
        std::fs::write(&file, new_content).map_err(|e| e.to_string())
    };

    // gate=false: dropping `layout:` touches no cross-reference field.
    let outcome = store.commit(false, false, false, apply);
    Json(to_response(&outcome))
}

/// `POST /api/diagrams/shapes/{*qname}` body (`REQ-TRS-VIS-024`).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddShapeRequest {
    /// The element to add: a qualified name or a stable id.
    #[serde(rename = "ref")]
    pub element_ref: String,
    /// Optional pin position; absent, the shape is left unpinned for ELK to place.
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
}

/// `POST /api/diagrams/shapes/{*qname}` — add an **existing** element to a
/// manifest diagram as a pinned shape (`REQ-TRS-VIS-024`, the browser's *Add
/// existing element*). Writes `shapes.<id> = {ref, kind}` and, when `x`/`y` are
/// given, a `layout.<id>` pin in one guarded write; the element's own file is never touched. Refused
/// (always-`OK`/`written:false`, like the rest of this module) for an unknown
/// diagram, a target that is not a `Diagram`, a **derived** diagram (it follows
/// its subject — a hand-added shape would silently turn it into a manifest),
/// an unresolved `ref`, a `Diagram` as the element to add, and an element the
/// diagram already shows.
pub async fn add_shape(
    State(state): State<SharedState>,
    AxumPath(qname): AxumPath<String>,
    Json(req): Json<AddShapeRequest>,
) -> Json<WriteResponse> {
    let mut store = state.write().await;
    let target = match diagram_target(&store, &qname) {
        Ok(t) => t,
        Err(reason) => return Json(refused(reason)),
    };
    if syscribe_model::vis::source_of(&target.frontmatter) != syscribe_model::vis::Source::Manifest {
        return Json(refused(format!(
            "'{}' is a derived diagram: it follows its subject, so shapes cannot be added by hand — \
             narrow or widen it with `include:`/`exclude:`, or start a blank diagram",
            target.qualified_name
        )));
    }
    let element = match store.resolver.resolve_ref(&store.elements, req.element_ref.trim()) {
        Some(e) => e.clone(),
        None => return Json(refused(format!("unresolved reference: {}", req.element_ref))),
    };
    let Some(element_type) = element.frontmatter.element_type.clone() else {
        return Json(refused(format!("'{}' has no element type", element.qualified_name)));
    };
    if element_type == ElementType::Diagram {
        return Json(refused(format!("'{}' is a Diagram; diagrams cannot be shapes of a diagram", element.qualified_name)));
    }

    // Already on the diagram, and the shape ids in use (to keep the new one unique).
    let existing: Vec<(String, String)> = match target.frontmatter.shapes.as_ref() {
        Some(serde_yaml::Value::Mapping(m)) => m
            .iter()
            .filter_map(|(k, v)| {
                let id = k.as_str()?.to_string();
                let r = match v {
                    serde_yaml::Value::String(s) => s.clone(),
                    serde_yaml::Value::Mapping(vm) => vm.get(KEY_REF).and_then(|x| x.as_str())?.to_string(),
                    _ => return None,
                };
                Some((id, r))
            })
            .collect(),
        _ => Vec::new(),
    };
    if let Some((id, _)) = existing.iter().find(|(_, r)| {
        r == &element.qualified_name
            || store.resolver.resolve_ref(&store.elements, r).is_some_and(|e| e.qualified_name == element.qualified_name)
    }) {
        return Json(refused(format!("'{}' is already on this diagram (shape '{id}')", element.qualified_name)));
    }
    let base = syscribe_model::vis::ir::derived_shape_id(&element.qualified_name);
    let mut shape_id = base.clone();
    let mut n = 2;
    while existing.iter().any(|(id, _)| id == &shape_id) {
        shape_id = format!("{base}-{n}");
        n += 1;
    }
    // A shape added without a position is left unpinned: ELK places it around
    // the pinned ones, which beats any fixed default. Given `x`/`y`, it is pinned there.
    let pinned = req.x.is_some() || req.y.is_some();
    let ctx = ShapeDiagramContext {
        qname: target.qualified_name.clone(),
        shape_id,
        x: req.x.unwrap_or(0.0).round(),
        y: req.y.unwrap_or(0.0).round(),
        kind: element_type.name().to_string(),
    };
    let new_qname = element.qualified_name.clone();
    let apply = move |root: &Path| -> Result<(), String> { sync_shape_add(root, &ctx, &new_qname, pinned) };

    // gate=false: the added `ref` was just resolved; nothing new can dangle.
    let outcome = store.commit(false, false, false, apply);
    Json(to_response(&outcome))
}

/// The companion SVG file of a diagram, relative to the `.md` file's
/// directory: `svgFile:` as written when set, else `<stem>.svg` beside it
/// (the validator's `E402` default). The second value is the `src` the body's
/// `<img>` uses (`./<stem>.svg` for the default).
fn companion_svg_name(target: &syscribe_model::element::RawElement) -> (String, String) {
    match target.frontmatter.svg_file.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(sf) => (sf.to_string(), sf.to_string()),
        None => {
            let stem = Path::new(&target.file_path)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "diagram".to_string());
            (format!("{stem}.svg"), format!("./{stem}.svg"))
        }
    }
}

/// `PUT /api/diagrams/svg/{*qname}` — *Save companion SVG* (`REQ-TRS-VIS-011`):
/// write the body's `svg` to the diagram's companion file (`svgFile:` if set,
/// relative to the `.md`; else `<stem>.svg` beside it) and, in the same
/// guarded write, set `svgMode: companion` and `svgFile:` in the frontmatter
/// when absent and append an `<img>` to the body when it has none (so `W405`
/// stays clean). Refused — nothing written — unless the body is an SVG
/// document (trimmed, starts with `<svg` and contains `</svg>`), for an
/// unknown qname, or for a non-`Diagram` element.
pub async fn put_svg(
    State(state): State<SharedState>,
    AxumPath(qname): AxumPath<String>,
    Json(req): Json<PutSvgRequest>,
) -> Json<WriteResponse> {
    let svg = req.svg.trim();
    if !svg.starts_with("<svg") || !svg.contains("</svg>") {
        return Json(refused("body `svg` is not an SVG document (must start with `<svg` and contain `</svg>`)"));
    }
    let svg = format!("{svg}\n");

    let mut store = state.write().await;
    let target = match diagram_target(&store, &qname) {
        Ok(t) => t,
        Err(reason) => return Json(refused(reason)),
    };
    let md_rel = rel_to_root(&target.file_path, &store.model_root);
    let (svg_name, img_src) = companion_svg_name(&target);
    let svg_rel = match Path::new(&md_rel).parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join(&svg_name).to_string_lossy().into_owned(),
        _ => svg_name.clone(),
    };
    let alt = target
        .frontmatter
        .name
        .clone()
        .unwrap_or_else(|| target.qualified_name.rsplit("::").next().unwrap_or(&target.qualified_name).to_string());
    let set_mode = target.frontmatter.svg_mode.is_none();
    let set_file = target.frontmatter.svg_file.as_deref().map(str::trim).filter(|s| !s.is_empty()).is_none();

    let apply = move |root: &Path| -> Result<(), String> {
        write_confined(root, &svg_rel, &svg).map_err(|e| e.to_string())?;
        let file = root.join(&md_rel);
        let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let (_, body) = syscribe_model::frontmatter::split_frontmatter(&content);
        let new_body = if body.contains("<img") {
            None
        } else {
            let mut b = body.trim_end_matches('\n').to_string();
            b.push_str(&format!("\n<img src=\"{img_src}\" alt=\"{}\" width=\"100%\"/>\n", alt.replace('"', "&quot;")));
            Some(b)
        };
        let new_content = patch_frontmatter(&content, new_body.as_deref(), |map| {
            if set_mode {
                map.insert("svgMode".into(), "companion".into());
            }
            if set_file {
                map.insert("svgFile".into(), img_src.clone().into());
            }
        })
        .map_err(|e| e.to_string())?;
        std::fs::write(&file, new_content).map_err(|e| e.to_string())
    };

    // gate=false: the write touches `svgMode`/`svgFile`/the body, never a
    // cross-reference field.
    let outcome = store.commit(false, false, false, apply);
    Json(to_response(&outcome))
}

#[cfg(test)]
mod sub_mapping_tests {
    //! `sub_mapping` must never replace an author's present-but-malformed
    //! `shapes:`/`edges:`/`layout:` section with an empty map (data loss);
    //! it refuses, and the guarded write that called it is refused with it.
    use super::{patch_frontmatter_checked, sub_mapping, KEY_LAYOUT, KEY_SHAPES};

    fn yaml(s: &str) -> serde_yaml::Mapping {
        match serde_yaml::from_str(s).unwrap() {
            serde_yaml::Value::Mapping(m) => m,
            other => panic!("not a mapping: {other:?}"),
        }
    }

    #[test]
    fn absent_or_null_section_becomes_an_empty_map() {
        let mut m = yaml("type: Diagram\nlayout: ~\n");
        assert!(sub_mapping(&mut m, KEY_SHAPES).unwrap().is_empty());
        assert!(sub_mapping(&mut m, KEY_LAYOUT).unwrap().is_empty());
        assert!(matches!(m.get(KEY_SHAPES), Some(serde_yaml::Value::Mapping(_))));
        assert!(matches!(m.get(KEY_LAYOUT), Some(serde_yaml::Value::Mapping(_))));
    }

    #[test]
    fn existing_map_is_returned_untouched() {
        let mut m = yaml("shapes:\n  a:\n    ref: X::A\n    kind: block\n");
        let shapes = sub_mapping(&mut m, KEY_SHAPES).unwrap();
        assert_eq!(shapes.len(), 1);
        assert!(shapes.contains_key("a"));
    }

    #[test]
    fn non_map_section_is_refused_and_left_as_is() {
        for (fm, what) in [
            ("shapes:\n  - X::A\n  - X::B\n", "a sequence"),
            ("shapes: 42\n", "a number"),
            ("shapes: just text\n", "a string"),
            ("shapes: true\n", "a boolean"),
        ] {
            let mut m = yaml(fm);
            let before = m.clone();
            let err = sub_mapping(&mut m, KEY_SHAPES).expect_err(fm);
            assert!(err.contains("`shapes:`"), "{err}");
            assert!(err.contains(what), "{err}");
            assert!(err.contains("E405"), "{err}");
            assert_eq!(m, before, "the malformed section is not replaced: {fm}");
        }
    }

    #[test]
    fn a_refused_sub_mapping_refuses_the_whole_patch() {
        let content = "---\ntype: Diagram\nname: D\nshapes:\n  - X::A\n---\n\nBody.\n";
        let result = patch_frontmatter_checked(content, |map| {
            let shapes = sub_mapping(map, KEY_SHAPES)?;
            shapes.insert("b".into(), "X::B".into());
            Ok(())
        });
        let err = result.expect_err("a sequence at `shapes:` refuses the write");
        assert!(err.contains("`shapes:` is a sequence"), "{err}");

        // The same patch on a well-formed file goes through and keeps the body.
        let ok = patch_frontmatter_checked("---\ntype: Diagram\nshapes:\n  a: X::A\n---\n\nBody.\n", |map| {
            let shapes = sub_mapping(map, KEY_SHAPES)?;
            shapes.insert("b".into(), "X::B".into());
            Ok(())
        })
        .unwrap();
        assert!(ok.contains("b: X::B"), "{ok}");
        assert!(ok.contains("Body."), "{ok}");
    }
}

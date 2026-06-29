//! `syscribe mcp` — a Model Context Protocol server over stdio.
//!
//! Exposes a curated set of read tools and guarded-write tools over an in-memory
//! [`McpStore`], plus the format spec as resources and the authoring prompt.

mod diff;
mod store;
mod util;
mod variability;
mod write;

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::model::{
    AnnotateAble, CallToolRequestParams, CallToolResult, CompleteRequestParams, CompleteResult,
    CompletionInfo, Content, GetPromptRequestParams, GetPromptResult, Implementation,
    ListPromptsResult, ListResourceTemplatesResult, ListResourcesResult, ListToolsResult,
    LoggingLevel, LoggingMessageNotificationParam, PaginatedRequestParams, Prompt, PromptArgument,
    PromptMessage, PromptMessageRole, RawResource, RawResourceTemplate, ReadResourceRequestParams,
    ReadResourceResult, Reference, ResourceContents, ServerCapabilities, ServerInfo,
    SetLevelRequestParams,
};
use rmcp::service::RequestContext;
use rmcp::{
    tool, tool_handler, tool_router, ErrorData, Peer, RoleServer, ServerHandler, ServiceExt,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use tokio::sync::RwLock;

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::graph::{children_of, EdgeKind};
use syscribe_model::resolver::{is_stable_id, Resolver, STABLE_ID_KINDS};
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

use crate::mv;
use crate::query::{fuzzy_score, next_id_value, template_str, type_label};
use crate::spec;
use store::McpStore;
use util::{elem_detail, elem_summary, finding_json, json_to_yaml, rel_file, severity_str};
use write::{guarded_write, refuse};

fn default_true() -> bool {
    true
}

// ── Tool parameter structs ──────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetElementArgs {
    r#ref: String,
    #[serde(default)]
    detail: bool,
    #[serde(default)]
    #[allow(dead_code)]
    fields: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SearchArgs {
    query: Option<String>,
    r#type: Option<String>,
    status: Option<String>,
    domain: Option<String>,
    /// Custom-field predicate, e.g. `custom.<key>=<value>` (see the CLI `--where`).
    r#where: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ListByTypeArgs {
    r#type: String,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TreeArgs {
    r#ref: Option<String>,
    depth: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NeighborsArgs {
    r#ref: String,
    edges: Option<Vec<String>>,
    direction: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GraphQueryArgs {
    from: String,
    #[allow(dead_code)]
    to: Option<String>,
    edges: Option<Vec<String>>,
    direction: Option<String>,
    depth: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TraceArgs {
    r#ref: String,
    /// "verification" (default) | "derivation" | "all".
    kind: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ImpactArgs {
    r#ref: String,
    /// "downstream" (default) | "upstream" | "both".
    direction: Option<String>,
    depth: Option<u32>,
    edges: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ValidateArgs {
    file: Option<String>,
    severity: Option<String>,
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ValidateElementArgs {
    r#ref: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DescribeTypeArgs {
    r#type: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TemplateArgs {
    r#type: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ExplainFindingArgs {
    code: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CheckRefArgs {
    r#ref: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NextIdArgs {
    prefix: String,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct CoverageArgs {}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct FeaturesArgs {
    feature: Option<String>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct FeatureCheckArgs {
    #[serde(default)]
    deep: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ConfigureArgs {
    config: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ProjectArgs {
    config: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DiffConfigsArgs {
    a: String,
    b: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct WhyActiveArgs {
    r#ref: String,
    config: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CreateElementArgs {
    qname: String,
    r#type: String,
    fields: Option<Value>,
    doc: Option<String>,
    #[serde(default = "default_true")]
    dry_run: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct UpdateElementArgs {
    r#ref: String,
    fields: Option<Value>,
    doc: Option<String>,
    #[serde(default = "default_true")]
    dry_run: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MoveElementArgs {
    r#ref: String,
    dest: String,
    #[serde(default = "default_true")]
    dry_run: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DeleteElementArgs {
    r#ref: String,
    #[serde(default)]
    force: bool,
    #[serde(default = "default_true")]
    dry_run: bool,
}

/// One operation in an `apply_changes` batch. Carries the union of the single
/// write tools' arguments; `op` selects which are read.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BatchOp {
    op: String,
    qname: Option<String>,
    r#type: Option<String>,
    fields: Option<Value>,
    doc: Option<String>,
    r#ref: Option<String>,
    dest: Option<String>,
    #[allow(dead_code)]
    force: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ApplyChangesArgs {
    operations: Vec<BatchOp>,
    #[serde(default = "default_true")]
    dry_run: bool,
}

// ── Small result helpers ────────────────────────────────────────────────────

fn ok(v: Value) -> Result<CallToolResult, ErrorData> {
    Ok(CallToolResult::success(vec![Content::json(v)?]))
}

fn tool_error(msg: impl Into<String>) -> Result<CallToolResult, ErrorData> {
    Ok(CallToolResult::error(vec![Content::text(msg.into())]))
}

fn extra_map(pairs: Value) -> Map<String, Value> {
    pairs.as_object().cloned().unwrap_or_default()
}

// ── Graph / element JSON helpers ────────────────────────────────────────────

fn node_json(store: &McpStore, qname: &str) -> Value {
    let e = store.resolver.get(&store.elements, qname);
    json!({
        "qname": qname,
        "id": e.and_then(|e| e.frontmatter.id.clone()),
        "type": e.and_then(|e| e.frontmatter.element_type.as_ref().map(|t| type_label(t).to_string())),
    })
}

fn tree_json(store: &McpStore, qname: &str, depth: usize) -> Value {
    let e = store.resolver.get(&store.elements, qname);
    let children: Vec<Value> = if depth > 0 {
        children_of(&store.graph, &store.node_idx, qname)
            .iter()
            .map(|c| tree_json(store, c, depth - 1))
            .collect()
    } else {
        Vec::new()
    };
    json!({
        "qname": qname,
        "id": e.and_then(|e| e.frontmatter.id.clone()),
        "type": e.and_then(|e| e.frontmatter.element_type.as_ref().map(|t| type_label(t).to_string())),
        "name": e.and_then(|e| e.frontmatter.name.clone()),
        "children": children,
    })
}

/// BFS over the model graph honouring an edge-kind filter, a direction, and an
/// optional hop limit. Returns discovered nodes as `(qname, distance)` pairs in
/// BFS order (the start node at distance 0) and the traversed `(from, to, edge)`
/// triples.
#[allow(clippy::type_complexity)]
fn graph_bfs(
    store: &McpStore,
    start: &str,
    edges: &Option<HashSet<String>>,
    direction: &str,
    depth: Option<usize>,
) -> (Vec<(String, usize)>, Vec<(String, String, &'static str)>) {
    use std::collections::VecDeque;
    let Some(&start_ni) = store.node_idx.get(start) else {
        return (Vec::new(), Vec::new());
    };
    let want = |k: &EdgeKind| edges.as_ref().is_none_or(|set| set.contains(k.name()));
    let do_out = direction == "out" || direction == "both";
    let do_in = direction == "in" || direction == "both";

    let mut visited: HashSet<NodeIndex> = HashSet::from([start_ni]);
    let mut order: Vec<(NodeIndex, usize)> = vec![(start_ni, 0usize)];
    let mut queue: VecDeque<(NodeIndex, usize)> = VecDeque::from([(start_ni, 0usize)]);
    let mut edge_acc: Vec<(NodeIndex, NodeIndex, &'static str)> = Vec::new();

    while let Some((n, d)) = queue.pop_front() {
        if depth.is_some_and(|m| d >= m) {
            continue;
        }
        if do_out {
            for e in store.graph.edges_directed(n, Direction::Outgoing) {
                if !want(e.weight()) {
                    continue;
                }
                edge_acc.push((e.source(), e.target(), e.weight().name()));
                if visited.insert(e.target()) {
                    order.push((e.target(), d + 1));
                    queue.push_back((e.target(), d + 1));
                }
            }
        }
        if do_in {
            for e in store.graph.edges_directed(n, Direction::Incoming) {
                if !want(e.weight()) {
                    continue;
                }
                edge_acc.push((e.source(), e.target(), e.weight().name()));
                if visited.insert(e.source()) {
                    order.push((e.source(), d + 1));
                    queue.push_back((e.source(), d + 1));
                }
            }
        }
    }

    let nodes = order
        .iter()
        .map(|&(n, dist)| (store.graph[n].clone(), dist))
        .collect();
    let edges_str = edge_acc
        .iter()
        .map(|(s, t, k)| (store.graph[*s].clone(), store.graph[*t].clone(), *k))
        .collect();
    (nodes, edges_str)
}

/// `{id, qname, name}` for a reference, resolved through the resolver. Falls back
/// to the raw key when it does not resolve (so a dangling id is still legible).
fn ref_entry(store: &McpStore, key: &str) -> Value {
    match store.resolver.resolve_ref(&store.elements, key) {
        Some(e) => json!({
            "id": e.frontmatter.id,
            "qname": e.qualified_name,
            "name": e.frontmatter.name,
        }),
        None => json!({ "id": key, "qname": key, "name": Value::Null }),
    }
}

/// The built-in stable-id prefix for an id-identified element type (e.g. `REQ`).
fn builtin_prefix(et: &ElementType) -> Option<&'static str> {
    let label = type_label(et);
    STABLE_ID_KINDS
        .iter()
        .find(|(ty, _, _)| *ty == label)
        .map(|(_, p, _)| *p)
}

/// One field descriptor for `describe_type`.
fn field_spec(name: &str, required: bool, ty: &str, domain: Option<&[&str]>) -> Value {
    let mut o = serde_json::Map::new();
    o.insert("name".into(), Value::String(name.into()));
    o.insert("required".into(), Value::Bool(required));
    o.insert("type".into(), Value::String(ty.into()));
    if let Some(vals) = domain {
        o.insert(
            "enum".into(),
            Value::Array(vals.iter().map(|v| Value::String((*v).into())).collect()),
        );
    }
    Value::Object(o)
}

/// The field schema for a type. Enum domains mirror the validator (E007 etc.).
/// Known id-identified types report their `id`/`name`/`status` triplet; the
/// Requirement schema is the most detailed (its `status` enum is contract-tested).
fn type_field_specs(type_name: &str) -> Vec<Value> {
    let req_status: &[&str] = &["draft", "review", "approved", "implemented", "verified"];
    let domain: &[&str] = &["system", "hardware", "software"];
    let req_class: &[&str] = &["stakeholder", "system", "derived"];
    let tc_status: &[&str] = &["draft", "review", "approved", "active", "retired"];
    let tc_level: &[&str] = &["L1", "L2", "L3", "L4", "L5"];
    match type_name {
        "Requirement" => vec![
            field_spec("id", true, "string", None),
            field_spec("name", true, "string", None),
            field_spec("status", true, "enum", Some(req_status)),
            field_spec("reqDomain", false, "enum", Some(domain)),
            field_spec("reqClass", false, "enum", Some(req_class)),
            field_spec("derivedFrom", false, "list<ref>", None),
            field_spec("breakdownAdr", false, "ref", None),
            field_spec("verificationMethod", false, "string", None),
        ],
        "TestCase" => vec![
            field_spec("id", true, "string", None),
            field_spec("name", true, "string", None),
            field_spec("status", true, "enum", Some(tc_status)),
            field_spec("testLevel", true, "enum", Some(tc_level)),
            field_spec("verifies", false, "list<ref>", None),
            field_spec("sourceFile", false, "string", None),
        ],
        other => {
            // Generic shape: id-identified types carry an id; everything carries
            // a type + name. Sufficient for type introspection of the long tail.
            let id_identified = is_stable_id_type(other);
            let mut v = vec![field_spec("type", true, "string", None)];
            if id_identified {
                v.push(field_spec("id", true, "string", None));
            }
            v.push(field_spec("name", !id_identified, "string", None));
            v
        }
    }
}

/// True when `type_name` is an id-identified element type (carries a stable id).
fn is_stable_id_type(type_name: &str) -> bool {
    STABLE_ID_KINDS.iter().any(|(ty, _, _)| *ty == type_name)
}

/// The set of recognised element-type names (for `describe_type` enumeration).
fn known_type_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = vec![
        "PartDef", "Part", "ItemDef", "Item", "PortDef", "Port", "ConnectionDef",
        "Connection", "InterfaceDef", "Interface", "ActionDef", "Action", "ConstraintDef",
        "Constraint", "CalculationDef", "Calculation", "StateDef", "State", "FlowDef",
        "EnumerationDef", "Enumeration", "AttributeDef", "RequirementDef", "UseCaseDef",
        "UseCase", "ViewDef", "View", "ViewpointDef", "MetadataDef", "Metadata",
        "VerificationCaseDef", "VerificationCase", "AnalysisCaseDef", "AnalysisCase",
        "AllocationDef", "Allocation", "Diagram", "Package", "LibraryPackage", "Namespace",
        "Dependency", "FeatureDef", "Configuration", "Requirement", "TestCase", "TestPlan",
        "ADR",
    ];
    for (ty, _, _) in STABLE_ID_KINDS {
        if !names.contains(ty) {
            names.push(ty);
        }
    }
    names
}

/// Look up a finding code's human explanation from the embedded `validation` spec
/// section. Rows look like `| `W001` | Native Requirement … |`.
fn explain_code(code: &str) -> Option<String> {
    let (_, text) = spec::SECTIONS.iter().find(|(n, _)| *n == "validation")?;
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').collect();
        if cells.len() < 2 {
            continue;
        }
        let cell_code = cells[0].trim().trim_matches('`').trim();
        if cell_code.eq_ignore_ascii_case(code) {
            let explanation = cells[1].trim();
            if !explanation.is_empty() {
                return Some(explanation.to_string());
            }
        }
    }
    None
}

/// Apply a frontmatter-only / body update to a single file, mirroring the
/// server's `put_element` round-trip (preserve unknown keys + body).
fn apply_update(path: &Path, fields: Option<&Value>, doc: Option<&str>) -> Result<(), String> {
    use syscribe_model::frontmatter::split_frontmatter;
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (fm_opt, body) = split_frontmatter(&content);
    let mut yaml_val: serde_yaml::Value = match fm_opt {
        Some(s) => serde_yaml::from_str(s)
            .unwrap_or_else(|_| serde_yaml::Value::Mapping(serde_yaml::Mapping::new())),
        None => serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
    };
    if let (Some(Value::Object(o)), serde_yaml::Value::Mapping(map)) = (fields, &mut yaml_val) {
        for (k, v) in o {
            let key = serde_yaml::Value::String(k.clone());
            if v.is_null() {
                map.remove(&key);
            } else {
                map.insert(key, json_to_yaml(v));
            }
        }
    }
    let new_yaml = serde_yaml::to_string(&yaml_val).map_err(|e| e.to_string())?;
    let final_body = doc.unwrap_or(body);
    let new_content = format!("---\n{}---\n\n{}", new_yaml, final_body);
    std::fs::write(path, new_content).map_err(|e| e.to_string())?;
    Ok(())
}

/// A planned new-element write: where to write it, the file content, and the
/// reported id (auto-allocated for id-identified types, or the explicit one).
struct CreatePlan {
    rel: String,
    content: String,
    id: Value,
}

/// Validate inputs and build the file content for a `create` (single tool or a
/// batch op). Returns `Err` on an already-existing or syntactically invalid qname.
fn plan_create(
    elements: &[RawElement],
    qname_raw: &str,
    type_name: &str,
    fields: Option<&Value>,
    doc: Option<&str>,
) -> Result<CreatePlan, String> {
    let qname = qname_raw.replace('/', "::");
    if elements.iter().any(|e| e.qualified_name == qname) {
        return Err("an element with this qualified name already exists".into());
    }
    if !mv::valid_qname(&qname) {
        return Err("not a valid basic qualified name".into());
    }
    let etype: ElementType = serde_yaml::from_value(serde_yaml::Value::String(type_name.to_string()))
        .unwrap_or(ElementType::Unknown);
    let fields_obj = fields.and_then(|v| v.as_object());
    let explicit_id = fields_obj
        .and_then(|o| o.get("id"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let mut allocated_id: Option<String> = None;
    if etype.is_id_identified() && explicit_id.is_none() {
        if let Some(prefix) = builtin_prefix(&etype) {
            let mut n = 1u32;
            let id = loop {
                let cand = format!("{prefix}-GEN-{n:03}");
                let taken = elements
                    .iter()
                    .any(|e| e.frontmatter.id.as_deref() == Some(cand.as_str()));
                if !taken && is_stable_id(&cand) {
                    break cand;
                }
                n += 1;
            };
            allocated_id = Some(id);
        }
    }

    let mut map = serde_yaml::Mapping::new();
    map.insert("type".into(), serde_yaml::Value::String(type_name.to_string()));
    if let Some(id) = &allocated_id {
        map.insert("id".into(), serde_yaml::Value::String(id.clone()));
    }
    if let Some(o) = fields_obj {
        for (k, v) in o {
            map.insert(serde_yaml::Value::String(k.clone()), json_to_yaml(v));
        }
    }
    let yaml = serde_yaml::to_string(&serde_yaml::Value::Mapping(map)).unwrap_or_default();
    let content = format!("---\n{yaml}---\n\n{}\n", doc.unwrap_or(""));
    let rel = format!("{}.md", qname.replace("::", "/"));
    let id = allocated_id
        .or(explicit_id)
        .map(Value::String)
        .unwrap_or(Value::Null);
    Ok(CreatePlan { rel, content, id })
}

/// Write `content` to `<root>/<rel>`, confirming the resolved parent stays within
/// the canonicalized model root (defeats `..`/symlink traversal).
fn write_confined(root: &Path, rel: &str, content: &str) -> Result<(), String> {
    let target = root.join(rel);
    let canon_root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let canon_parent = std::fs::canonicalize(parent).map_err(|e| e.to_string())?;
        if !canon_parent.starts_with(&canon_root) {
            return Err("resolved path escapes the model root".into());
        }
    }
    std::fs::write(&target, content).map_err(|e| e.to_string())?;
    Ok(())
}

/// Apply one batch operation against the working model root `root`, re-reading
/// the current on-disk state so a later op can depend on an earlier one.
fn apply_op(root: &Path, op: &BatchOp) -> Result<(), String> {
    match op.op.as_str() {
        "create" => {
            let qname = op.qname.as_deref().ok_or("create op missing `qname`")?;
            let ty = op.r#type.as_deref().ok_or("create op missing `type`")?;
            let elems = walk_model(root).map_err(|e| e.to_string())?;
            let plan = plan_create(&elems, qname, ty, op.fields.as_ref(), op.doc.as_deref())?;
            write_confined(root, &plan.rel, &plan.content)
        }
        "update" => {
            let r = op.r#ref.as_deref().ok_or("update op missing `ref`")?;
            let elems = walk_model(root).map_err(|e| e.to_string())?;
            let resolver = Resolver::new(&elems);
            let file = resolver
                .resolve_ref(&elems, r)
                .map(|e| e.file_path.clone())
                .ok_or_else(|| format!("unresolved reference: {r}"))?;
            apply_update(Path::new(&file), op.fields.as_ref(), op.doc.as_deref())
        }
        "move" => {
            let r = op.r#ref.as_deref().ok_or("move op missing `ref`")?;
            let dest = op.dest.as_deref().ok_or("move op missing `dest`")?;
            let dest_n = dest.replace('/', "::");
            if !mv::valid_qname(&dest_n) {
                return Err(format!("invalid destination qualified name: {dest}"));
            }
            let elems = walk_model(root).map_err(|e| e.to_string())?;
            let resolver = Resolver::new(&elems);
            mv::move_element(root, &elems, &resolver, r, &dest_n, false).map(|_| ())
        }
        "delete" => {
            let r = op.r#ref.as_deref().ok_or("delete op missing `ref`")?;
            let elems = walk_model(root).map_err(|e| e.to_string())?;
            let resolver = Resolver::new(&elems);
            let file = resolver
                .resolve_ref(&elems, r)
                .map(|e| e.file_path.clone())
                .ok_or_else(|| format!("unresolved reference: {r}"))?;
            std::fs::remove_file(&file).map_err(|e| e.to_string())
        }
        other => Err(format!("unknown op: {other}")),
    }
}

/// The write tools, hidden and rejected under `--read-only`.
const WRITE_TOOLS: &[&str] = &[
    "create_element",
    "update_element",
    "move_element",
    "delete_element",
    "apply_changes",
];

fn is_write_tool(name: &str) -> bool {
    WRITE_TOOLS.contains(&name)
}

/// After a committed write (or reload), tell the client the resource list changed
/// so it can re-fetch. Best-effort: a transport error is ignored.
async fn notify_committed(peer: &Peer<RoleServer>, res: &Value) {
    if res.get("written").and_then(|w| w.as_bool()) == Some(true) {
        let _ = peer.notify_resource_list_changed().await;
    }
}

// ── The handler ─────────────────────────────────────────────────────────────

pub struct SyscribeMcp {
    store: Arc<RwLock<McpStore>>,
    read_only: bool,
}

#[tool_router]
impl SyscribeMcp {
    fn new(store: Arc<RwLock<McpStore>>, read_only: bool) -> Self {
        Self { store, read_only }
    }

    #[tool(
        description = "Fetch one element by stable id, qualified name, or display name. \
        Summary by default; pass detail=true to include the Markdown body and frontmatter, \
        or fields=[..] to project a chosen subset of frontmatter keys.",
        annotations(read_only_hint = true)
    )]
    async fn get_element(
        &self,
        Parameters(args): Parameters<GetElementArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        match store.resolver.resolve_ref(&store.elements, &args.r#ref) {
            Some(e) => {
                // A `fields` projection wins: return only the requested frontmatter
                // keys, and no doc body (the most token-frugal view).
                if let Some(fields) = args.fields.as_ref().filter(|f| !f.is_empty()) {
                    let full = serde_json::to_value(&e.frontmatter).unwrap_or(Value::Null);
                    let mut fm = serde_json::Map::new();
                    for k in fields {
                        if let Some(v) = full.get(k) {
                            fm.insert(k.clone(), v.clone());
                        }
                    }
                    let mut out = elem_summary(e);
                    if let Some(obj) = out.as_object_mut() {
                        obj.insert("frontmatter".into(), Value::Object(fm));
                    }
                    ok(out)
                } else if args.detail {
                    ok(elem_detail(e))
                } else {
                    ok(elem_summary(e))
                }
            }
            None => tool_error(format!("unresolved reference: {}", args.r#ref)),
        }
    }

    #[tool(
        description = "Ranked element search over id, qualified name, display name, and body.",
        annotations(read_only_hint = true)
    )]
    async fn search(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let limit = args.limit.unwrap_or(20) as usize;
        let offset = args.offset.unwrap_or(0) as usize;
        let type_filter = args.r#type.as_deref();
        let status_filter = args.status.as_deref();
        let domain_filter = args.domain.as_deref();
        let query = args.query.as_deref().filter(|q| !q.is_empty());
        let query_lc = query.map(|q| q.to_lowercase());

        // Parse the optional custom-field `where` predicate up front.
        let where_pred = match args.r#where.as_deref() {
            Some(w) => match crate::query::parse_custom_where(w) {
                Ok(p) => Some(p),
                Err(e) => return tool_error(e),
            },
            None => None,
        };

        let mut scored: Vec<(u32, &RawElement)> = Vec::new();
        for e in &store.elements {
            // ── query match: fuzzy score, else case-insensitive body match ──
            let score = match (&query, &query_lc) {
                (None, _) => 0,
                (Some(q), Some(q_lc)) => {
                    let fs = fuzzy_score(e, q);
                    if fs > 0 {
                        fs
                    } else if e.doc.to_lowercase().contains(q_lc.as_str()) {
                        0
                    } else {
                        continue; // query supplied but neither name/id/qname nor body matches
                    }
                }
                _ => 0,
            };
            // ── filters (logical AND) ──
            let fm = &e.frontmatter;
            if let Some(t) = type_filter {
                if fm.element_type.as_ref().map(type_label) != Some(t) {
                    continue;
                }
            }
            if let Some(s) = status_filter {
                if fm.status.as_deref() != Some(s) {
                    continue;
                }
            }
            if let Some(d) = domain_filter {
                let hit = fm.req_domain.as_deref() == Some(d) || fm.domain.as_deref() == Some(d);
                if !hit {
                    continue;
                }
            }
            if let Some(pred) = &where_pred {
                if !crate::query::custom_or_extra_matches(e, pred) {
                    continue;
                }
            }
            scored.push((score, e));
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        let total = scored.len();
        let results: Vec<Value> = scored
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(s, e)| {
                json!({
                    "qname": e.qualified_name,
                    "id": e.frontmatter.id,
                    "type": e.frontmatter.element_type.as_ref().map(type_label),
                    "name": e.frontmatter.name,
                    "score": s,
                })
            })
            .collect();
        ok(json!({ "results": results, "total": total }))
    }

    #[tool(
        description = "List every element of a given type.",
        annotations(read_only_hint = true)
    )]
    async fn list_by_type(
        &self,
        Parameters(args): Parameters<ListByTypeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let limit = args.limit.unwrap_or(100) as usize;
        let offset = args.offset.unwrap_or(0) as usize;
        let matching: Vec<&RawElement> = store
            .elements
            .iter()
            .filter(|e| e.frontmatter.element_type.as_ref().map(type_label) == Some(args.r#type.as_str()))
            .collect();
        let total = matching.len();
        let items: Vec<Value> = matching
            .iter()
            .skip(offset)
            .take(limit)
            .map(|e| {
                json!({
                    "qname": e.qualified_name,
                    "id": e.frontmatter.id,
                    "name": e.frontmatter.name,
                })
            })
            .collect();
        ok(json!({ "items": items, "total": total }))
    }

    #[tool(
        description = "Containment subtree rooted at an element (or the model root).",
        annotations(read_only_hint = true)
    )]
    async fn tree(
        &self,
        Parameters(args): Parameters<TreeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let depth = args.depth.unwrap_or(2) as usize;
        let root = match &args.r#ref {
            Some(r) => match store.resolver.resolve_ref(&store.elements, r) {
                Some(e) => e.qualified_name.clone(),
                None => return tool_error(format!("unresolved reference: {r}")),
            },
            None => String::new(),
        };
        ok(tree_json(&store, &root, depth))
    }

    #[tool(
        description = "Adjacent graph nodes of an element, by edge kind and direction.",
        annotations(read_only_hint = true)
    )]
    async fn neighbors(
        &self,
        Parameters(args): Parameters<NeighborsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let qname = match store.resolver.resolve_ref(&store.elements, &args.r#ref) {
            Some(e) => e.qualified_name.clone(),
            None => return tool_error(format!("unresolved reference: {}", args.r#ref)),
        };
        let direction = args.direction.as_deref().unwrap_or("both");
        let filter: Option<HashSet<String>> = args.edges.map(|v| v.into_iter().collect());
        let want = |k: &EdgeKind| filter.as_ref().is_none_or(|set| set.contains(k.name()));
        let Some(&ni) = store.node_idx.get(&qname) else {
            return ok(json!({ "out": [], "in": [] }));
        };
        let mut out = Vec::new();
        let mut inb = Vec::new();
        if direction == "out" || direction == "both" {
            for e in store.graph.edges_directed(ni, Direction::Outgoing) {
                if !want(e.weight()) {
                    continue;
                }
                let tq = store.graph[e.target()].clone();
                let id = store.resolver.get(&store.elements, &tq).and_then(|x| x.frontmatter.id.clone());
                out.push(json!({ "edge": e.weight().name(), "qname": tq, "id": id }));
            }
        }
        if direction == "in" || direction == "both" {
            for e in store.graph.edges_directed(ni, Direction::Incoming) {
                if !want(e.weight()) {
                    continue;
                }
                let sq = store.graph[e.source()].clone();
                let id = store.resolver.get(&store.elements, &sq).and_then(|x| x.frontmatter.id.clone());
                inb.push(json!({ "edge": e.weight().name(), "qname": sq, "id": id }));
            }
        }
        ok(json!({ "out": out, "in": inb }))
    }

    #[tool(
        description = "Walk the typed-edge model graph from a node, returning nodes and edges.",
        annotations(read_only_hint = true)
    )]
    async fn graph_query(
        &self,
        Parameters(args): Parameters<GraphQueryArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let from = match store.resolver.resolve_ref(&store.elements, &args.from) {
            Some(e) => e.qualified_name.clone(),
            None => return tool_error(format!("unresolved reference: {}", args.from)),
        };
        let direction = args.direction.as_deref().unwrap_or("both");
        let filter: Option<HashSet<String>> = args.edges.map(|v| v.into_iter().collect());
        let depth = args.depth.map(|d| d as usize);
        let (nodes, edges) = graph_bfs(&store, &from, &filter, direction, depth);
        let nodes_json: Vec<Value> = nodes.iter().map(|(q, _)| node_json(&store, q)).collect();
        let edges_json: Vec<Value> = edges
            .iter()
            .map(|(f, t, k)| json!({ "from": f, "to": t, "edge": k }))
            .collect();
        ok(json!({ "nodes": nodes_json, "edges": edges_json }))
    }

    #[tool(
        description = "Verification/derivation trace for a requirement: who verifies it, \
        its derived children, and what it derives from.",
        annotations(read_only_hint = true)
    )]
    async fn trace(
        &self,
        Parameters(args): Parameters<TraceArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let elem = match store.resolver.resolve_ref(&store.elements, &args.r#ref) {
            Some(e) => e,
            None => return tool_error(format!("unresolved reference: {}", args.r#ref)),
        };
        let key = elem
            .frontmatter
            .id
            .clone()
            .unwrap_or_else(|| elem.qualified_name.clone());
        let ref_node = json!({
            "qname": elem.qualified_name,
            "id": elem.frontmatter.id,
            "type": elem.frontmatter.element_type.as_ref().map(type_label),
            "name": elem.frontmatter.name,
        });
        let derived_from_keys = elem.frontmatter.derived_from.clone().unwrap_or_default();

        let kind = args.kind.as_deref().unwrap_or("verification");
        let include_ver = kind == "verification" || kind == "all";
        let include_der = kind == "derivation" || kind == "all";

        let result = validate_with_config(&store.elements, &store.config);
        let verified_by: Vec<Value> = if include_ver {
            result
                .verified_by
                .get(&key)
                .map(|ids| ids.iter().map(|id| ref_entry(&store, id)).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let derived_children: Vec<Value> = if include_der {
            result
                .derived_children
                .get(&key)
                .map(|ids| ids.iter().map(|id| ref_entry(&store, id)).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let derived_from: Vec<Value> = if include_der {
            derived_from_keys
                .iter()
                .map(|k| ref_entry(&store, k))
                .collect()
        } else {
            Vec::new()
        };

        ok(json!({
            "ref": ref_node,
            "verifiedBy": verified_by,
            "derivedChildren": derived_children,
            "derivedFrom": derived_from,
        }))
    }

    #[tool(
        description = "Reachability/impact analysis over the model graph from an element. \
        downstream = dependents; upstream = dependencies.",
        annotations(read_only_hint = true)
    )]
    async fn impact(
        &self,
        Parameters(args): Parameters<ImpactArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let from = match store.resolver.resolve_ref(&store.elements, &args.r#ref) {
            Some(e) => e.qualified_name.clone(),
            None => return tool_error(format!("unresolved reference: {}", args.r#ref)),
        };
        // Edges point from the referencing element to its referent (e.g. Derived
        // --supertype--> Base), so a node's *dependents* are reached by walking
        // incoming edges. "downstream" therefore traverses inbound edges.
        let direction = match args.direction.as_deref().unwrap_or("downstream") {
            "upstream" => "out",
            "both" => "both",
            _ => "in",
        };
        let filter: Option<HashSet<String>> = args.edges.map(|v| v.into_iter().collect());
        let depth = args.depth.map(|d| d as usize);
        let (nodes, _edges) = graph_bfs(&store, &from, &filter, direction, depth);
        let affected: Vec<Value> = nodes
            .iter()
            .filter(|(_, dist)| *dist > 0)
            .map(|(q, dist)| {
                let e = store.resolver.get(&store.elements, q);
                json!({
                    "qname": q,
                    "id": e.and_then(|e| e.frontmatter.id.clone()),
                    "type": e.and_then(|e| e.frontmatter.element_type.as_ref().map(|t| type_label(t).to_string())),
                    "distance": dist,
                })
            })
            .collect();
        let total = affected.len();
        ok(json!({ "affected": affected, "total": total }))
    }

    #[tool(
        description = "Validation findings for the whole model, filtered by severity.",
        annotations(read_only_hint = true)
    )]
    async fn validate(
        &self,
        Parameters(args): Parameters<ValidateArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let result = validate_with_config(&store.elements, &store.config);
        let rank = |s: &str| match s {
            "error" => 2,
            "warning" => 1,
            _ => 0,
        };
        let threshold = rank(args.severity.as_deref().unwrap_or("warning"));
        let root = &store.model_root;
        let file_filter = args.file.as_deref();
        let mut included: Vec<Value> = Vec::new();
        let (mut ce, mut cw, mut ci) = (0u64, 0u64, 0u64);
        for f in &result.findings {
            let sev = severity_str(&f.severity);
            if rank(sev) < threshold {
                continue;
            }
            if let Some(target) = file_filter {
                if rel_file(&f.file, root) != *target && !f.file.ends_with(target) {
                    continue;
                }
            }
            match sev {
                "error" => ce += 1,
                "warning" => cw += 1,
                _ => ci += 1,
            }
            included.push(finding_json(f, root));
        }
        if let Some(limit) = args.limit {
            included.truncate(limit as usize);
        }
        ok(json!({
            "findings": included,
            "counts": { "error": ce, "warning": cw, "info": ci },
        }))
    }

    #[tool(
        description = "Validation findings scoped to a single element's file.",
        annotations(read_only_hint = true)
    )]
    async fn validate_element(
        &self,
        Parameters(args): Parameters<ValidateElementArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let file = match store.resolver.resolve_ref(&store.elements, &args.r#ref) {
            Some(e) => e.file_path.clone(),
            None => return tool_error(format!("unresolved reference: {}", args.r#ref)),
        };
        let result = validate_with_config(&store.elements, &store.config);
        let root = &store.model_root;
        let findings: Vec<Value> = result
            .findings
            .iter()
            .filter(|f| f.file == file)
            .map(|f| finding_json(f, root))
            .collect();
        ok(json!({ "findings": findings }))
    }

    #[tool(
        description = "Re-read the model from disk. Returns the element count.",
        annotations(read_only_hint = true)
    )]
    async fn reload(&self, peer: Peer<RoleServer>) -> Result<CallToolResult, ErrorData> {
        let count = {
            let mut store = self.store.write().await;
            if let Err(e) = store.reload() {
                return tool_error(format!("reload failed: {e}"));
            }
            store.elements.len()
        };
        // Significant event → a log message and a resource-list-changed signal.
        #[allow(deprecated)]
        let _ = peer
            .notify_logging_message(LoggingMessageNotificationParam {
                level: LoggingLevel::Info,
                logger: Some("syscribe".to_string()),
                data: json!({ "event": "reload", "count": count }),
            })
            .await;
        let _ = peer.notify_resource_list_changed().await;
        ok(json!({ "count": count }))
    }

    #[tool(
        description = "Describe a type's frontmatter schema (fields, required flags, enum \
        domains); with no `type`, enumerate the recognised element types.",
        annotations(read_only_hint = true)
    )]
    async fn describe_type(
        &self,
        Parameters(args): Parameters<DescribeTypeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        match args.r#type.as_deref().filter(|t| !t.is_empty()) {
            None => ok(json!({ "types": known_type_names() })),
            Some(t) => ok(json!({ "type": t, "fields": type_field_specs(t) })),
        }
    }

    #[tool(
        description = "Return a starter frontmatter+body skeleton for an element type.",
        annotations(read_only_hint = true)
    )]
    async fn template(
        &self,
        Parameters(args): Parameters<TemplateArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        match template_str(&args.r#type) {
            Some(content) => ok(json!({ "content": content })),
            None => tool_error(format!("unknown element type: {}", args.r#type)),
        }
    }

    #[tool(
        description = "Explain a validation finding code (sourced from the format spec).",
        annotations(read_only_hint = true)
    )]
    async fn explain_finding(
        &self,
        Parameters(args): Parameters<ExplainFindingArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        match explain_code(&args.code) {
            Some(explanation) => ok(json!({ "code": args.code, "explanation": explanation })),
            None => tool_error(format!("unknown finding code: {}", args.code)),
        }
    }

    #[tool(
        description = "Check whether a reference resolves, reporting the target's identity.",
        annotations(read_only_hint = true)
    )]
    async fn check_ref(
        &self,
        Parameters(args): Parameters<CheckRefArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        match store.resolver.resolve_ref(&store.elements, &args.r#ref) {
            Some(e) => ok(json!({
                "resolved": true,
                "qname": e.qualified_name,
                "id": e.frontmatter.id,
                "type": e.frontmatter.element_type.as_ref().map(type_label),
            })),
            None => ok(json!({ "resolved": false })),
        }
    }

    #[tool(
        description = "Next free stable id for a prefix (e.g. REQ-FX -> REQ-FX-004).",
        annotations(read_only_hint = true)
    )]
    async fn next_id(
        &self,
        Parameters(args): Parameters<NextIdArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        ok(json!({ "id": next_id_value(&store.elements, &args.prefix) }))
    }

    #[tool(
        description = "Requirement verification coverage: counts plus the unverified requirements.",
        annotations(read_only_hint = true)
    )]
    async fn coverage(
        &self,
        Parameters(_args): Parameters<CoverageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        use syscribe_model::resolver::Resolver;
        let store = self.store.read().await;
        let result = validate_with_config(&store.elements, &store.config);
        let mut verified = 0u64;
        let mut unverified: Vec<Value> = Vec::new();
        for e in &store.elements {
            if !Resolver::is_native_requirement(e) {
                continue;
            }
            let id = e.frontmatter.id.as_deref().unwrap_or("");
            let is_verified = result
                .verified_by
                .get(id)
                .is_some_and(|tcs| !tcs.is_empty());
            if is_verified {
                verified += 1;
            } else {
                unverified.push(json!({
                    "qname": e.qualified_name,
                    "id": e.frontmatter.id,
                    "name": e.frontmatter.name,
                }));
            }
        }
        ok(json!({
            "verifiedCount": verified,
            "unverifiedCount": unverified.len(),
            "unverified": unverified,
        }))
    }

    #[tool(
        description = "List the feature model (or one feature's card).",
        annotations(read_only_hint = true)
    )]
    async fn features(
        &self,
        Parameters(args): Parameters<FeaturesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        ok(variability::features(&store.elements, args.feature.as_deref()))
    }

    #[tool(
        description = "Validate the feature model; deep=true runs the SAT-backed analysis \
        (void/dead/core/false-optional/invalid-configs/diagnoses).",
        annotations(read_only_hint = true)
    )]
    async fn feature_check(
        &self,
        Parameters(args): Parameters<FeatureCheckArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        ok(variability::feature_check(&store.elements, args.deep, &store.model_root))
    }

    #[tool(
        description = "Assisted configuration: forced/free features for a Configuration.",
        annotations(read_only_hint = true)
    )]
    async fn configure(
        &self,
        Parameters(args): Parameters<ConfigureArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        ok(variability::configure_tool(&store.elements, &args.config))
    }

    #[tool(
        description = "Project the model onto a Configuration (or ad-hoc feature set): \
        active elements + projected validation findings.",
        annotations(read_only_hint = true)
    )]
    async fn project(
        &self,
        Parameters(args): Parameters<ProjectArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        ok(variability::project_tool(
            &store.elements,
            &store.config,
            &args.config,
            &store.model_root,
        ))
    }

    #[tool(
        description = "Diff two projections' active element sets (a/b may be a Configuration \
        or an ad-hoc feature set).",
        annotations(read_only_hint = true)
    )]
    async fn diff_configs(
        &self,
        Parameters(args): Parameters<DiffConfigsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        ok(variability::diff_configs(&store.elements, &args.a, &args.b))
    }

    #[tool(
        description = "Explain whether an element is active under a configuration and why \
        (effective appliesWhen, referenced features).",
        annotations(read_only_hint = true)
    )]
    async fn why_active(
        &self,
        Parameters(args): Parameters<WhyActiveArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        // Lenient resolution: the resolver only indexes *valid* stable ids, but an
        // element may carry a non-canonical `id:` (e.g. `REQ-FX-SAT`) — fall back to
        // a raw id / qualified-name match.
        let elem = store.resolver.resolve_ref(&store.elements, &args.r#ref).or_else(|| {
            store.elements.iter().find(|e| {
                e.frontmatter.id.as_deref() == Some(args.r#ref.as_str())
                    || e.qualified_name == args.r#ref
            })
        });
        match elem {
            Some(elem) => ok(variability::why_active(&store.elements, elem, &args.config)),
            None => tool_error(format!("unresolved reference: {}", args.r#ref)),
        }
    }

    #[tool(
        description = "Create a new element file. dry_run defaults to true (preview only).",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn create_element(
        &self,
        Parameters(args): Parameters<CreateElementArgs>,
        peer: Peer<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut store = self.store.write().await;
        let qname = args.qname.replace('/', "::");
        let mut extra = extra_map(json!({ "qname": qname }));

        let plan = match plan_create(
            &store.elements,
            &args.qname,
            &args.r#type,
            args.fields.as_ref(),
            args.doc.as_deref(),
        ) {
            Ok(p) => p,
            Err(e) => return ok(refuse(extra, &e)),
        };
        extra.insert("id".into(), plan.id.clone());

        let rel = plan.rel.clone();
        let content = plan.content.clone();
        let apply = move |root: &Path| -> Result<(), String> { write_confined(root, &rel, &content) };

        let res = guarded_write(&mut store, args.dry_run, true, extra, apply);
        drop(store);
        notify_committed(&peer, &res).await;
        ok(res)
    }

    #[tool(
        description = "Update an element's frontmatter and/or body. dry_run defaults to true.",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn update_element(
        &self,
        Parameters(args): Parameters<UpdateElementArgs>,
        peer: Peer<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut store = self.store.write().await;
        let file = match store.resolver.resolve_ref(&store.elements, &args.r#ref) {
            Some(e) => e.file_path.clone(),
            None => return tool_error(format!("unresolved reference: {}", args.r#ref)),
        };
        let rel = rel_file(&file, &store.model_root);
        let extra = extra_map(json!({ "ref": args.r#ref }));
        let fields = args.fields.clone();
        let doc = args.doc.clone();
        let apply = move |root: &Path| -> Result<(), String> {
            apply_update(&root.join(&rel), fields.as_ref(), doc.as_deref())
        };
        let res = guarded_write(&mut store, args.dry_run, true, extra, apply);
        drop(store);
        notify_committed(&peer, &res).await;
        ok(res)
    }

    #[tool(
        description = "Move/rename an element, rewriting inbound references. dry_run defaults to true.",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn move_element(
        &self,
        Parameters(args): Parameters<MoveElementArgs>,
        peer: Peer<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut store = self.store.write().await;
        let dest = args.dest.replace('/', "::");
        let extra = extra_map(json!({ "ref": args.r#ref, "dest": dest }));
        if !mv::valid_qname(&dest) {
            return ok(refuse(extra, "not a valid basic destination qualified name"));
        }
        let source = args.r#ref.clone();
        let dest_c = dest.clone();
        let apply = move |root: &Path| -> Result<(), String> {
            let elems = walk_model(root).map_err(|e| e.to_string())?;
            let resolver = Resolver::new(&elems);
            mv::move_element(root, &elems, &resolver, &source, &dest_c, false).map(|_| ())
        };
        let res = guarded_write(&mut store, args.dry_run, true, extra, apply);
        drop(store);
        notify_committed(&peer, &res).await;
        ok(res)
    }

    #[tool(
        description = "Delete an element file. Refuses (blockedBy) if other elements reference \
        it unless force=true. dry_run defaults to true.",
        annotations(read_only_hint = false, destructive_hint = true)
    )]
    async fn delete_element(
        &self,
        Parameters(args): Parameters<DeleteElementArgs>,
        peer: Peer<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut store = self.store.write().await;
        let (target_qname, rel) = match store.resolver.resolve_ref(&store.elements, &args.r#ref) {
            Some(e) => (
                e.qualified_name.clone(),
                rel_file(&e.file_path, &store.model_root),
            ),
            None => return tool_error(format!("unresolved reference: {}", args.r#ref)),
        };

        if !args.force {
            let refs = write::referrers(&store.elements, &target_qname);
            if !refs.is_empty() {
                let blocked: Vec<Value> = refs
                    .iter()
                    .map(|(q, id)| json!({ "qname": q, "id": id }))
                    .collect();
                let extra = extra_map(json!({ "ref": args.r#ref, "blockedBy": blocked }));
                return ok(refuse(
                    extra,
                    "delete blocked by inbound references; pass force:true to override",
                ));
            }
        }

        let extra = extra_map(json!({ "ref": args.r#ref }));
        let rel_c = rel.clone();
        let apply = move |root: &Path| -> Result<(), String> {
            std::fs::remove_file(root.join(&rel_c)).map_err(|e| e.to_string())
        };
        // gate=false: deletion may legitimately orphan a reference (esp. under force);
        // the reference-impact guard above is delete's safety mechanism.
        let res = guarded_write(&mut store, args.dry_run, false, extra, apply);
        drop(store);
        notify_committed(&peer, &res).await;
        ok(res)
    }

    #[tool(
        description = "Apply an ordered batch of create/update/move/delete operations \
        atomically (all-or-nothing). dry_run defaults to true.",
        annotations(read_only_hint = false, destructive_hint = true)
    )]
    async fn apply_changes(
        &self,
        Parameters(args): Parameters<ApplyChangesArgs>,
        peer: Peer<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut store = self.store.write().await;
        let extra = extra_map(json!({ "operations": args.operations.len() }));
        let ops = args.operations;
        let apply = move |root: &Path| -> Result<(), String> {
            for op in &ops {
                apply_op(root, op)?;
            }
            Ok(())
        };
        let res = guarded_write(&mut store, args.dry_run, true, extra, apply);
        drop(store);
        notify_committed(&peer, &res).await;
        ok(res)
    }
}

#[tool_handler]
impl ServerHandler for SyscribeMcp {
    #[allow(deprecated)] // `enable_logging` is soft-deprecated upstream but still the
                         // way to advertise the logging capability the tests require.
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_resources_list_changed()
                .enable_resources_subscribe()
                .enable_prompts()
                .enable_completions()
                .enable_logging()
                .build(),
        )
        .with_server_info(Implementation::new("syscribe-mcp", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "Query and guard-write a Syscribe systems model over MCP. Read tools are \
             token-efficient; write tools default to dry_run and refuse commits that \
             introduce new validation errors."
                .to_string(),
        )
    }

    // Accept logging/setLevel (the level is advisory; we always emit at Info).
    async fn set_level(
        &self,
        _request: SetLevelRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), ErrorData> {
        Ok(())
    }

    // Filter the generated tool list under --read-only (hides the write tools).
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let mut tools = Self::tool_router().list_all();
        if self.read_only {
            tools.retain(|t| !is_write_tool(t.name.as_ref()));
        }
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }

    // Reject write tools under --read-only; otherwise dispatch via the router.
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        if self.read_only && is_write_tool(&request.name) {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "tool '{}' is disabled in --read-only mode",
                request.name
            ))]));
        }
        let tcc = ToolCallContext::new(self, request, context);
        Self::tool_router().call(tcc).await
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let mut resources: Vec<_> = spec::SECTIONS
            .iter()
            .map(|(name, _)| {
                RawResource::new(format!("syscribe://spec/{name}"), format!("spec: {name}"))
                    .no_annotation()
            })
            .collect();
        resources.push(
            RawResource::new("syscribe://config", "project configuration (.syscribe.toml)")
                .no_annotation(),
        );
        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        let templates = vec![RawResourceTemplate::new(
            "syscribe://element/{qname}",
            "model element",
        )
        .no_annotation()];
        Ok(ListResourceTemplatesResult::with_all_items(templates))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let uri = request.uri;
        if let Some(section) = uri.strip_prefix("syscribe://spec/") {
            if let Some((_, text)) = spec::SECTIONS.iter().find(|(n, _)| *n == section) {
                return Ok(ReadResourceResult::new(vec![ResourceContents::text(
                    *text,
                    uri.clone(),
                )]));
            }
        }
        if let Some(qname) = uri.strip_prefix("syscribe://element/") {
            let store = self.store.read().await;
            if let Some(e) = store.resolver.resolve_ref(&store.elements, qname) {
                let text = serde_json::to_string(&elem_detail(e))
                    .unwrap_or_else(|_| "{}".to_string());
                return Ok(ReadResourceResult::new(vec![ResourceContents::text(
                    text,
                    uri.clone(),
                )]));
            }
        }
        if uri == "syscribe://config" {
            // The project config; absent file → empty string (not an error).
            let store = self.store.read().await;
            let text = std::fs::read_to_string(store.model_root.join(".syscribe.toml"))
                .unwrap_or_default();
            return Ok(ReadResourceResult::new(vec![ResourceContents::text(
                text,
                uri.clone(),
            )]));
        }
        Err(ErrorData::resource_not_found(
            format!("unknown resource: {uri}"),
            None,
        ))
    }

    async fn complete(
        &self,
        request: CompleteRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, ErrorData> {
        // Only element-resource references are completable; the completed argument
        // is a qname prefix, matched against element qnames and stable ids.
        let is_element_ref = match &request.r#ref {
            Reference::Resource(r) => r.uri.starts_with("syscribe://element/"),
            _ => false,
        };
        if !is_element_ref {
            return Ok(CompleteResult::default());
        }
        let prefix = request.argument.value.to_lowercase();
        let store = self.store.read().await;
        let mut values: Vec<String> = Vec::new();
        for e in &store.elements {
            let qn = &e.qualified_name;
            if qn.to_lowercase().contains(&prefix) {
                values.push(qn.clone());
            }
            if let Some(id) = e.frontmatter.id.as_deref() {
                if id.to_lowercase().contains(&prefix) && !values.iter().any(|v| v == id) {
                    values.push(id.to_string());
                }
            }
            if values.len() >= CompletionInfo::MAX_VALUES {
                break;
            }
        }
        let total = values.len() as u32;
        Ok(CompleteResult::new(CompletionInfo {
            values,
            total: Some(total),
            has_more: Some(false),
        }))
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        let target_arg = || {
            vec![PromptArgument::new("target")
                .with_description("Stable id or qualified name of the target element")
                .with_required(true)]
        };
        let prompts = vec![
            Prompt::new(
                "create-model",
                Some("Full instructions for authoring a valid Syscribe model"),
                None,
            ),
            Prompt::new(
                "create-magicgrid-model",
                Some("Instructions for authoring a MagicGrid-profile Syscribe model"),
                None,
            ),
            Prompt::new(
                "add-requirement",
                Some("Author a new native Requirement following the project conventions"),
                None,
            ),
            Prompt::new(
                "break-down-requirement",
                Some("Decompose a requirement into derived child requirements"),
                Some(target_arg()),
            ),
            Prompt::new(
                "add-testcase-for",
                Some("Author a verifying TestCase for a requirement"),
                Some(target_arg()),
            ),
            Prompt::new(
                "traceability-review",
                Some("Review requirement→test→architecture traceability for gaps"),
                None,
            ),
        ];
        Ok(ListPromptsResult::with_all_items(prompts))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        let target = request
            .arguments
            .as_ref()
            .and_then(|m| m.get("target"))
            .and_then(|v| v.as_str())
            .unwrap_or("<target>");
        let text: String = match request.name.as_str() {
            "create-model" => crate::AGENT_INSTRUCTIONS.to_string(),
            "create-magicgrid-model" => crate::MAGICGRID_INSTRUCTIONS.to_string(),
            "add-requirement" => "Author a new native Requirement (type: Requirement, stable \
                 REQ-* id via the `next_id` tool). Set `status` \
                 (draft|review|approved|implemented|verified), `reqDomain` \
                 (system|hardware|software), and `reqClass`. Write a single normative `shall` \
                 statement as the body. If this requirement is derived from a parent, set \
                 `derivedFrom:` to the parent and `breakdownAdr:` to the accepted ADR that records \
                 the breakdown (every requirement with `derivedFrom` must carry a `breakdownAdr`). \
                 Use the `template` tool with type=Requirement for a skeleton and `validate_element` \
                 to check the result."
                .to_string(),
            "break-down-requirement" => format!(
                "Decompose requirement '{target}' into derived child requirements. Each child sets \
                 `derivedFrom: [{target}]` and a `breakdownAdr:` pointing at the accepted ADR for \
                 this breakdown. Break down until each leaf can be satisfied by a single \
                 architecture element. Use `trace` on '{target}' to see existing children and \
                 `next_id` to allocate ids."
            ),
            "add-testcase-for" => format!(
                "Author a TestCase (type: TestCase, stable TC-* id) that verifies requirement \
                 '{target}'. Set `verifies: [{target}]`, a `testLevel` (L1–L5), and a Gherkin \
                 scenario body. Use the `template` tool with type=TestCase and `next_id` for the id."
            ),
            "traceability-review" => "Review the model's traceability. Use the `coverage` tool to \
                 find unverified requirements, `trace` to inspect each requirement's verifiedBy / \
                 derivedChildren / derivedFrom, and `validate` to surface W300/E310-class gaps. \
                 Report requirements lacking a verifying TestCase and any `derivedFrom` without a \
                 `breakdownAdr`."
                .to_string(),
            other => {
                return Err(ErrorData::invalid_params(
                    format!("unknown prompt: {other}"),
                    None,
                ))
            }
        };
        Ok(GetPromptResult::new(vec![PromptMessage::new_text(
            PromptMessageRole::User,
            text,
        )]))
    }
}

/// `syscribe mcp` entry point: build a runtime, load the store, serve over stdio.
/// `read_only` hides and rejects the write tools (`--read-only`).
pub fn cmd_mcp(model_root: &Path, read_only: bool) -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let store = McpStore::load(model_root)?;
        let handler = SyscribeMcp::new(Arc::new(RwLock::new(store)), read_only);
        let service = handler.serve(rmcp::transport::stdio()).await?;
        service.waiting().await?;
        Ok::<(), anyhow::Error>(())
    })
}

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

use crate::lint_docs::lint_docs_findings;
use crate::matrix::matrix_json;
use crate::mv;
use crate::query::{fuzzy_score, next_id_value, tc_verdict, template_str, type_label, TcVerdict};
use crate::spec;
use syscribe_model::plantuml::render_plantuml;
use syscribe_model::results::{FnVerdict, ResultsData};
use store::McpStore;
use util::{elem_detail, elem_summary, finding_json, json_to_yaml, rel_file, severity_str};
use write::{guarded_write, refuse};

/// Schema for a free-form frontmatter map. `serde_json::Value` makes schemars
/// emit the boolean schema `true` ("any"), which strict MCP clients (zod) reject
/// as an invalid property schema; emit a plain `{"type":"object"}` instead.
fn fields_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "type": "object",
        "description": "Frontmatter fields as a JSON object (key -> value)."
    })
}

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
struct RunReportArgs {
    command: String,
    #[serde(default)]
    args: Option<Vec<String>>,
    /// "text" (default) | "json".
    format: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CreateElementArgs {
    qname: String,
    r#type: String,
    #[schemars(schema_with = "fields_schema")]
    fields: Option<Value>,
    doc: Option<String>,
    #[serde(default = "default_true")]
    dry_run: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct UpdateElementArgs {
    r#ref: String,
    #[schemars(schema_with = "fields_schema")]
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SuspectListArgs {
    /// Also report links that have no baseline yet (candidates for suspect_accept).
    #[serde(default = "default_true")]
    include_unbaselined: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SuspectAcceptArgs {
    /// The source element holding the link (stable id or qualified name).
    source: String,
    /// The link target to baseline (stable id or qualified name).
    target: String,
    #[serde(default = "default_true")]
    dry_run: bool,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct BaselineListArgs {}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BaselineDiffArgs {
    /// The earlier baseline id.
    from: String,
    /// The later baseline id.
    to: String,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct BaselineVerifyArgs {
    /// A baseline id to verify; omit (or "all") to verify every baseline.
    #[serde(default)]
    r#ref: Option<String>,
}

/// One operation in an `apply_changes` batch. Carries the union of the single
/// write tools' arguments; `op` selects which are read.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BatchOp {
    op: String,
    qname: Option<String>,
    r#type: Option<String>,
    #[schemars(schema_with = "fields_schema")]
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct IngestResultsArgs {
    format: Option<String>,
    path: Option<String>,
    content: Option<String>,
    #[serde(default = "default_true")]
    dry_run: bool,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct CoverageMatrixArgs {
    config: Option<String>,
    status: Option<String>,
    tag: Option<String>,
    gaps_only: Option<bool>,
    linked_only: Option<bool>,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct CoverageGapsArgs {
    config: Option<String>,
    status: Option<String>,
    class: Option<String>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct StatsArgs {
    /// Re-key the primary histogram by this facet crossed with top-level package:
    /// status | reqDomain | silLevel | asilLevel | tags.
    group_by: Option<String>,
    /// Custom-field predicate restricting the requirement set (e.g. `custom.k=v`).
    r#where: Option<String>,
    /// Restrict to requirements with this status.
    status: Option<String>,
    /// Restrict to requirements carrying this tag.
    tag: Option<String>,
    /// Aggregate only the elements active in this Configuration (id/qname or
    /// comma-separated FeatureDef qualified names).
    config: Option<String>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct DigestArgs {
    /// Custom-field predicate restricting the requirement set (e.g. `custom.k=v`).
    r#where: Option<String>,
    status: Option<String>,
    tag: Option<String>,
    /// Aggregate only requirements active in this Configuration.
    config: Option<String>,
    /// Emit at most this many rows (cursor paging).
    limit: Option<u32>,
    /// Skip the first N rows.
    offset: Option<u32>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct SearchTextArgs {
    /// The full-text query (BM25-ranked over element body + name).
    query: String,
    /// Restrict to one element type (e.g. `Requirement`).
    r#type: Option<String>,
    status: Option<String>,
    /// Search only the elements active in this Configuration.
    config: Option<String>,
    /// Return at most this many results, ordered by score (default 10).
    limit: Option<u32>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct SummarizeArgs {
    /// Restrict to the subtree rooted at this package qualified name.
    scope: Option<String>,
    /// Bound the nesting depth reported.
    depth: Option<u32>,
    /// Project onto this Configuration before summarising.
    config: Option<String>,
    /// Bypass and rewrite the content-hash cache.
    no_cache: Option<bool>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct TopicsArgs {
    /// Element type to analyse (default `Requirement`).
    r#type: Option<String>,
    /// Terms per package (default 10).
    top: Option<u32>,
    config: Option<String>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct ClustersArgs {
    /// Number of clusters (default min(8, element count); clamped to the count).
    k: Option<u32>,
    /// Element type to cluster (default `Requirement`).
    r#type: Option<String>,
    config: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EvidenceArgs {
    r#ref: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LintDocsArgs {
    paths: Vec<String>,
    codes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RenderDiagramArgs {
    r#ref: String,
    format: Option<String>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct DiagramCoverageArgs {
    root: Option<String>,
    types: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GenerateViewArgs {
    kind: String,
    root: Option<String>,
    format: Option<String>,
}

// ── Small result helpers ────────────────────────────────────────────────────

/// Extract the inner source of the first ```mermaid fenced block in `doc`.
fn extract_mermaid(doc: &str) -> Option<String> {
    let mut out: Vec<&str> = Vec::new();
    let mut in_block = false;
    for line in doc.lines() {
        let t = line.trim_start();
        if t.starts_with("```") {
            if in_block {
                return Some(out.join("\n"));
            }
            if t.contains("mermaid") {
                in_block = true;
            }
            continue;
        }
        if in_block {
            out.push(line);
        }
    }
    (!out.is_empty()).then(|| out.join("\n"))
}

fn tc_verdict_str(v: TcVerdict) -> &'static str {
    match v {
        TcVerdict::Pass => "pass",
        TcVerdict::Fail => "fail",
        TcVerdict::Unknown => "unknown",
    }
}

fn fn_verdict_str(v: FnVerdict) -> &'static str {
    match v {
        FnVerdict::Pass => "pass",
        FnVerdict::Fail => "fail",
        FnVerdict::Ignored => "ignored",
        FnVerdict::Missing => "missing",
    }
}

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
    "ingest_results",
    "suspect_accept",
];

fn is_write_tool(name: &str) -> bool {
    WRITE_TOOLS.contains(&name)
}

/// Read-only CLI report/analysis commands `run_report` may invoke. Anything else
/// (notably write commands like `move`) is refused.
const REPORT_ALLOWLIST: &[&str] = &[
    "audit",
    "stats",
    "digest",
    "summarize",
    "matrix",
    "magicgrid",
    "trade-study",
    "verification-depth",
    "testplan",
    "metrics",
    "cyber-risk",
    "co-analysis",
    "safety-case",
    "behavioral-coverage",
    "sbom",
    "zones",
    "conduits",
    "n2",
    "fmea",
    "fault-tree",
    "impact",
    "lint-docs",
];

/// Reject a caller-supplied `run_report` argument that would redirect the model
/// root, redirect output, use an absolute path, or escape via `..`. The model
/// root is fixed to the server's own `store.model_root`.
fn check_report_arg(a: &str) -> Result<(), String> {
    let bad = a == "-m"
        || a == "--model"
        || a.starts_with("-m")
        || a.starts_with("--model=")
        || a == "-o"
        || a == "--output"
        || a == "--out"
        || a.starts_with("--output=")
        || a.starts_with("--out=")
        || a.starts_with('/')
        || a.contains("..");
    if bad {
        return Err(format!(
            "argument '{a}' is not permitted (model/output redirection or path escape)"
        ));
    }
    Ok(())
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
        let store = self.store.read().await;
        let result = validate_with_config(&store.elements, &store.config);
        let summary =
            crate::coverage::coverage_summary(&store.elements, &result, store.config.results.as_ref());

        let entries = |v: &[crate::coverage::CoverageEntry]| -> Vec<Value> {
            v.iter().map(|e| json!({ "qname": e.qname, "id": e.id, "name": e.name })).collect()
        };
        let unverified_leaves = entries(&summary.unverified_leaves);
        let planned = entries(&summary.planned);
        let parents_missing_integration: Vec<Value> = summary
            .parents_missing_integration
            .iter()
            .map(|e| {
                json!({
                    "qname": e.qname,
                    "id": e.id,
                    "name": e.name,
                    "childCount": e.child_count.unwrap_or(0),
                })
            })
            .collect();

        ok(json!({
            "verifiedCount": summary.verified_count,
            "unverifiedLeaves": unverified_leaves,
            "planned": planned,
            "parentsMissingIntegrationTest": parents_missing_integration,
        }))
    }

    #[tool(
        description = "Corpus-shape digest for fast scanning of large requirement sets \
        (REQ-TRS-OUT-021): per-facet histograms (status, reqDomain, silLevel, asilLevel, \
        package, tags) plus coverage and orphan rollups, in one call. Mirrors `stats --json`. \
        Optional `group_by` (facet x package), `where`/`status`/`tag` scoping, and `config` \
        projection lens. Use this as the cheap first hop before per-element `get_element`.",
        annotations(read_only_hint = true)
    )]
    async fn stats(
        &self,
        Parameters(args): Parameters<StatsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        // A single `where` predicate mirrors the CLI (which also accepts repeats).
        let wheres = match args.r#where.as_deref() {
            Some(w) => match crate::query::parse_custom_where(w) {
                Ok(p) => vec![p],
                Err(msg) => return Err(ErrorData::invalid_params(msg, None)),
            },
            None => vec![],
        };
        let opts = crate::stats::StatsOptions {
            group_by: args.group_by.as_deref(),
            wheres: &wheres,
            status: args.status.as_deref(),
            tag: args.tag.as_deref(),
            package_top_n: None,
        };
        match crate::stats::stats_document(
            &store.elements,
            &store.config,
            args.config.as_deref(),
            &opts,
        ) {
            Ok(doc) => ok(doc),
            Err(msg) => Err(ErrorData::invalid_params(msg, None)),
        }
    }

    #[tool(
        description = "Token-budgeted bulk view (REQ-TRS-OUT-022): one compact row per \
        Requirement — {id, name, status, reqDomain, sil?, asil?, text, verified} — paginated. \
        Mirrors `digest --json`. The 'dump the slice' companion to `stats`: narrow with \
        `stats`, then page rows here. Reuses the same where/status/tag/config scoping.",
        annotations(read_only_hint = true)
    )]
    async fn digest(
        &self,
        Parameters(args): Parameters<DigestArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let wheres = match args.r#where.as_deref() {
            Some(w) => match crate::query::parse_custom_where(w) {
                Ok(p) => vec![p],
                Err(msg) => return Err(ErrorData::invalid_params(msg, None)),
            },
            None => vec![],
        };
        let opts = crate::digest::DigestOptions {
            wheres: &wheres,
            status: args.status.as_deref(),
            tag: args.tag.as_deref(),
            limit: args.limit.map(|l| l as usize),
            offset: args.offset.map(|o| o as usize),
        };
        match crate::digest::digest_document(&store.elements, args.config.as_deref(), &opts) {
            Ok(doc) => ok(doc),
            Err(msg) => Err(ErrorData::invalid_params(msg, None)),
        }
    }

    #[tool(
        description = "Ranked full-text search (REQ-TRS-SEARCH-001): Okapi BM25 over element \
        body + name, best-first with a snippet marking the hit. Mirrors `search-text --json`. \
        Use to find elements by what they SAY, ranked by relevance (the `search` tool matches \
        identifiers). Optional `type`/`status`/`config` scoping.",
        annotations(read_only_hint = true)
    )]
    async fn search_text(
        &self,
        Parameters(args): Parameters<SearchTextArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let limit = args.limit.unwrap_or(10) as usize;
        // Apply the optional --config projection lens before searching.
        let projected = match args.config.as_deref() {
            None => None,
            Some(c) => match syscribe_model::projection::resolve_selection(&store.elements, c) {
                syscribe_model::projection::SelectionOutcome::Dormant => None,
                syscribe_model::projection::SelectionOutcome::Resolved(sel) => {
                    Some(syscribe_model::projection::project(&store.elements, &sel))
                }
                syscribe_model::projection::SelectionOutcome::Error(m) => {
                    return Err(ErrorData::invalid_params(m, None))
                }
            },
        };
        let view = projected.as_deref().unwrap_or(&store.elements);
        match crate::ftsearch::search_document(
            view,
            &args.query,
            args.r#type.as_deref(),
            args.status.as_deref(),
            limit,
        ) {
            Ok(doc) => ok(doc),
            Err(msg) => Err(ErrorData::invalid_params(msg, None)),
        }
    }

    #[tool(
        description = "Hierarchical content digest (REQ-TRS-OUT-023): a bottom-up per-package \
        rollup — count, status split, TF-IDF 'about' terms, and one-line extracts of \
        representative requirements — so you read a few package summaries, not 15k files. \
        Deterministic/extractive (not an LLM summary), content-hash cached. Mirrors \
        `summarize --json`. Optional scope/depth/config.",
        annotations(read_only_hint = true)
    )]
    async fn summarize(
        &self,
        Parameters(args): Parameters<SummarizeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        match crate::summarize::summarize_document(
            &store.elements,
            &store.model_root,
            args.scope.as_deref(),
            args.depth.map(|d| d as usize),
            args.no_cache.unwrap_or(false),
            args.config.as_deref(),
        ) {
            Ok(doc) => ok(doc),
            Err(msg) => Err(ErrorData::invalid_params(msg, None)),
        }
    }

    #[tool(
        description = "Distinctive per-package keywords via TF-IDF (REQ-TRS-SEARCH-002): names \
        what each package is about, demoting vocabulary common to every package. Deterministic/\
        offline. Mirrors `topics --json`. Optional type/top/config.",
        annotations(read_only_hint = true)
    )]
    async fn topics(
        &self,
        Parameters(args): Parameters<TopicsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        match crate::topics::topics_document(
            &store.elements,
            args.r#type.as_deref(),
            args.top.unwrap_or(10) as usize,
            args.config.as_deref(),
        ) {
            Ok(doc) => ok(doc),
            Err(msg) => Err(ErrorData::invalid_params(msg, None)),
        }
    }

    #[tool(
        description = "Topical clustering via TF-IDF cosine k-means (REQ-TRS-SEARCH-003): groups \
        elements by shared distinctive vocabulary, surfacing cross-package themes. Deterministic \
        (fixed init, no random seed) and offline (no neural embeddings). Mirrors `clusters --json`. \
        Optional k/type/config.",
        annotations(read_only_hint = true)
    )]
    async fn clusters(
        &self,
        Parameters(args): Parameters<ClustersArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        match crate::clusters::clusters_document(
            &store.elements,
            args.r#type.as_deref(),
            args.k.unwrap_or(8) as usize,
            args.config.as_deref(),
        ) {
            Ok(doc) => ok(doc),
            Err(msg) => Err(ErrorData::invalid_params(msg, None)),
        }
    }

    #[tool(
        description = "Requirement x Configuration coverage grid (matches `matrix --json`): \
        per-cell passing/covered/gap/na plus the coverage rollup. Reflects ingested results.",
        annotations(read_only_hint = true)
    )]
    async fn coverage_matrix(
        &self,
        Parameters(args): Parameters<CoverageMatrixArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let mut v = matrix_json(
            &store.elements,
            args.tag.as_deref(),
            args.status.as_deref(),
            args.gaps_only.unwrap_or(false),
            store.config.results.as_ref(),
            args.linked_only.unwrap_or(false),
        );
        // Optional single-Configuration projection: keep only that column/cell.
        if let Some(cfg) = args.config.as_deref() {
            if let Some(cols) = v.get_mut("columns").and_then(|c| c.as_array_mut()) {
                cols.retain(|c| c.as_str() == Some(cfg));
            }
            if let Some(rows) = v.get_mut("rows").and_then(|r| r.as_array_mut()) {
                for row in rows.iter_mut() {
                    if let Some(cells) = row.get_mut("cells").and_then(|c| c.as_object_mut()) {
                        cells.retain(|k, _| k == cfg);
                    }
                }
            }
        }
        // Optional row paging.
        if let Some(rows) = v.get_mut("rows").and_then(|r| r.as_array_mut()) {
            let off = args.offset.unwrap_or(0) as usize;
            if off > 0 {
                rows.drain(..off.min(rows.len()));
            }
            if let Some(lim) = args.limit {
                rows.truncate(lim as usize);
            }
        }
        ok(v)
    }

    #[tool(
        description = "The actionable coverage subset: rows classed uncovered / failing / \
        unverified-claim, each with the governing finding code. Complements `coverage`.",
        annotations(read_only_hint = true)
    )]
    async fn coverage_gaps(
        &self,
        Parameters(args): Parameters<CoverageGapsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let results = store.config.results.as_ref();
        let class_filter = args.class.as_deref();
        let want = |c: &str| class_filter.is_none_or(|f| f == c);
        let status_ok = |e: &RawElement| {
            args.status.as_deref().is_none_or(|s| e.frontmatter.status.as_deref() == Some(s))
        };
        let mut gaps: Vec<Value> = Vec::new();

        // uncovered: requirement active in a configuration with no verifying TestCase (gap cell).
        if want("uncovered") {
            let mj = matrix_json(&store.elements, None, args.status.as_deref(), false, results, false);
            if let Some(rows) = mj.get("rows").and_then(|r| r.as_array()) {
                for row in rows {
                    let id = row.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
                    let gap_cfgs: Vec<String> = match row.get("cells").and_then(|c| c.as_object()) {
                        Some(cells) => cells
                            .iter()
                            .filter(|(_, v)| v.as_str() == Some("gap"))
                            .map(|(k, _)| k.clone())
                            .collect(),
                        // flat fallback: `verified: false` is the gap.
                        None if row.get("verified").and_then(|b| b.as_bool()) == Some(false) => {
                            vec!["(flat)".to_string()]
                        }
                        None => Vec::new(),
                    };
                    if gap_cfgs.is_empty() {
                        continue;
                    }
                    if let Some(cfg) = args.config.as_deref() {
                        if !gap_cfgs.iter().any(|c| c == cfg) {
                            continue;
                        }
                    }
                    gaps.push(json!({ "ref": id, "configs": gap_cfgs, "class": "uncovered", "code": "W015" }));
                }
            }
        }

        // failing / unverified-claim: from the validator + verdicts.
        if want("failing") || want("unverified-claim") {
            let result = validate_with_config(&store.elements, &store.config);
            if want("failing") {
                for e in store.elements.iter().filter(|e| {
                    e.frontmatter.element_type == Some(ElementType::Requirement) && status_ok(e)
                }) {
                    let key = e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone());
                    let failing = result.verified_by.get(&key).is_some_and(|tcs| {
                        tcs.iter().any(|tid| {
                            store
                                .resolver
                                .resolve_ref(&store.elements, tid)
                                .is_some_and(|tc| tc_verdict(tc, results) == TcVerdict::Fail)
                        })
                    });
                    if failing {
                        gaps.push(json!({ "ref": key, "configs": [], "class": "failing", "code": "W010" }));
                    }
                }
            }
            if want("unverified-claim") {
                for f in result.findings.iter().filter(|f| f.code == "W029") {
                    if let Some(e) = store.elements.iter().find(|e| e.file_path == f.file) {
                        if !status_ok(e) {
                            continue;
                        }
                        let key = e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone());
                        gaps.push(json!({ "ref": key, "configs": [], "class": "unverified-claim", "code": "W029" }));
                    }
                }
            }
        }
        ok(json!({ "gaps": gaps }))
    }

    #[tool(
        description = "A requirement's verification chain: each verifying TestCase, its test \
        functions, and the latest ingested verdict (unknown when no results are loaded).",
        annotations(read_only_hint = true)
    )]
    async fn evidence(
        &self,
        Parameters(args): Parameters<EvidenceArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let req = match store.resolver.resolve_ref(&store.elements, &args.r#ref) {
            Some(e) => e,
            None => return tool_error(format!("unresolved reference: {}", args.r#ref)),
        };
        let result = validate_with_config(&store.elements, &store.config);
        let key = req.frontmatter.id.clone().unwrap_or_else(|| req.qualified_name.clone());
        let tc_ids = result
            .verified_by
            .get(&key)
            .cloned()
            .or_else(|| result.verified_by.get(&req.qualified_name).cloned())
            .unwrap_or_default();
        let results = store.config.results.as_ref();
        let fkey = serde_yaml::Value::String("function".into());
        let skey = serde_yaml::Value::String("sourceFile".into());
        let mut chain: Vec<Value> = Vec::new();
        for tid in &tc_ids {
            let tc = match store.resolver.resolve_ref(&store.elements, tid) {
                Some(t) => t,
                None => continue,
            };
            let funcs: Vec<Value> = tc
                .frontmatter
                .test_functions
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .filter_map(|tf| {
                    let m = tf.as_mapping()?;
                    let function = m.get(&fkey).and_then(|v| v.as_str())?.to_string();
                    let source = m.get(&skey).and_then(|v| v.as_str()).map(String::from);
                    let verdict = match results {
                        Some(r) => fn_verdict_str(r.verdict_for(&function)),
                        None => "unknown",
                    };
                    Some(json!({ "function": function, "sourceFile": source, "verdict": verdict }))
                })
                .collect();
            let tc_v = match results {
                Some(r) => tc_verdict_str(tc_verdict(tc, Some(r))),
                None => "unknown",
            };
            chain.push(json!({
                "testCase": tc.frontmatter.id,
                "qname": tc.qualified_name,
                "functions": funcs,
                "verdict": tc_v,
            }));
        }
        ok(json!({ "ref": req.qualified_name, "chain": chain }))
    }

    #[tool(
        description = "Ingest an external test report (cargo-json | junit) into the results \
        sidecar. dry_run defaults to true (returns the verdict delta without writing).",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn ingest_results(
        &self,
        Parameters(args): Parameters<IngestResultsArgs>,
        peer: Peer<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let (text, source) = match (&args.path, &args.content) {
            (Some(p), None) => match std::fs::read_to_string(p) {
                Ok(t) => (t, p.clone()),
                Err(e) => return tool_error(format!("cannot read '{p}': {e}")),
            },
            (None, Some(c)) => (c.clone(), "<inline>".to_string()),
            _ => return tool_error("supply exactly one of `path` or `content`"),
        };
        let fmt = match args.format.as_deref() {
            Some("cargo-json") => "cargo-json",
            Some("junit") => "junit",
            Some(o) => return tool_error(format!("unknown format '{o}': expected cargo-json | junit")),
            None if source.ends_with(".xml") => "junit",
            None => "cargo-json",
        };
        let parsed = match fmt {
            "junit" => ResultsData::parse_junit(&text, &source),
            _ => ResultsData::parse_cargo_json(&text, &source),
        };
        if parsed.count == 0 {
            return tool_error("no test records parsed from report (malformed or unrecognised for the given format)");
        }
        let mut store = self.store.write().await;
        let mut delta: Vec<Value> = Vec::new();
        for e in &store.elements {
            if e.frontmatter.element_type != Some(ElementType::TestCase) {
                continue;
            }
            let from = tc_verdict(e, store.config.results.as_ref());
            let to = tc_verdict(e, Some(&parsed));
            if from != to {
                delta.push(json!({
                    "testCase": e.frontmatter.id,
                    "qname": e.qualified_name,
                    "from": tc_verdict_str(from),
                    "to": tc_verdict_str(to),
                }));
            }
        }
        let extra = extra_map(json!({ "format": fmt, "count": parsed.count, "delta": delta }));
        let apply = move |root: &Path| -> Result<(), String> {
            parsed
                .write_sidecar(root)
                .map(|_| ())
                .map_err(|e| format!("cannot write results sidecar: {e}"))
        };
        let res = guarded_write(&mut store, args.dry_run, false, extra, apply);
        drop(store);
        notify_committed(&peer, &res).await;
        ok(res)
    }

    #[tool(
        description = "Scan .md/.svg files or directories for unresolvable model references \
        (W099 prose ids, W100 mermaid qnames, W101 SVG sysml:ref, W102 missing local embeds).",
        annotations(read_only_hint = true)
    )]
    async fn lint_docs(
        &self,
        Parameters(args): Parameters<LintDocsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let paths: Vec<&str> = args.paths.iter().map(|s| s.as_str()).collect();
        let findings = lint_docs_findings(&store.elements, &paths, args.codes.as_deref());
        ok(json!({ "findings": findings }))
    }

    #[tool(
        description = "Return a Diagram element's SOURCE (PlantUML by default, or the Mermaid \
        source) plus its W400-W415 structural findings. Does not render an image.",
        annotations(read_only_hint = true)
    )]
    async fn render_diagram(
        &self,
        Parameters(args): Parameters<RenderDiagramArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let elem = match store.resolver.resolve_ref(&store.elements, &args.r#ref) {
            Some(e) => e,
            None => return tool_error(format!("unresolved reference: {}", args.r#ref)),
        };
        if elem.frontmatter.element_type != Some(ElementType::Diagram) {
            return tool_error(format!("{} is not a Diagram element", args.r#ref));
        }
        let result = validate_with_config(&store.elements, &store.config);
        let root = &store.model_root;
        let findings: Vec<Value> = result
            .findings
            .iter()
            .filter(|f| f.file == elem.file_path && f.code.starts_with("W40"))
            .map(|f| finding_json(f, root))
            .collect();
        let (format, source) = if elem.frontmatter.diagram_kind.as_deref() == Some("Mermaid") {
            ("mermaid".to_string(), extract_mermaid(&elem.doc).unwrap_or_default())
        } else {
            let src = render_plantuml(elem, &store.elements, None).unwrap_or_default();
            (args.format.clone().unwrap_or_else(|| "plantuml".to_string()), src)
        };
        ok(json!({ "format": format, "source": source, "findings": findings }))
    }

    #[tool(
        description = "View-vs-model drift: in-scope elements referenced by no Diagram shape, \
        and diagram shape refs that resolve to no element (the W402 set).",
        annotations(read_only_hint = true)
    )]
    async fn diagram_coverage(
        &self,
        Parameters(args): Parameters<DiagramCoverageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        // Qualified names referenced by any Diagram shape's `ref`.
        let ref_key = serde_yaml::Value::String("ref".into());
        let mut referenced: HashSet<String> = HashSet::new();
        for e in &store.elements {
            if e.frontmatter.element_type != Some(ElementType::Diagram) {
                continue;
            }
            if let Some(shapes) = e.frontmatter.shapes.as_ref().and_then(|v| v.as_mapping()) {
                for (_id, sv) in shapes {
                    if let Some(r) = sv
                        .as_mapping()
                        .and_then(|m| m.get(&ref_key))
                        .and_then(|v| v.as_str())
                    {
                        referenced.insert(r.to_string());
                    }
                }
            }
        }
        let root = args.root.as_deref();
        let types = args.types.as_ref();
        // Structural/container types are not expected to appear in diagrams.
        let skip = |e: &RawElement| {
            matches!(
                e.frontmatter.element_type.as_ref().map(type_label),
                Some("Package") | Some("LibraryPackage") | Some("Namespace") | Some("Diagram") | None
            )
        };
        let uncovered: Vec<Value> = store
            .elements
            .iter()
            .filter(|e| !skip(e))
            .filter(|e| root.is_none_or(|r| e.qualified_name.starts_with(r)))
            .filter(|e| {
                types.is_none_or(|ts| {
                    e.frontmatter
                        .element_type
                        .as_ref()
                        .map(type_label)
                        .is_some_and(|tl| ts.iter().any(|t| t == tl))
                })
            })
            .filter(|e| !referenced.contains(&e.qualified_name))
            .map(|e| {
                json!({
                    "qname": e.qualified_name,
                    "id": e.frontmatter.id,
                    "type": e.frontmatter.element_type.as_ref().map(type_label),
                })
            })
            .collect();
        let result = validate_with_config(&store.elements, &store.config);
        let unresolved: Vec<Value> = result
            .findings
            .iter()
            .filter(|f| f.code == "W402")
            .map(|f| finding_json(f, &store.model_root))
            .collect();
        ok(json!({ "uncoveredElements": uncovered, "unresolvedRefs": unresolved }))
    }

    #[tool(
        description = "Synthesise diagram source (Mermaid) from the model graph for a view kind: \
        traceability | containment | feature | allocation. Embeds %% ref: annotations.",
        annotations(read_only_hint = true)
    )]
    async fn generate_view(
        &self,
        Parameters(args): Parameters<GenerateViewArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let result = validate_with_config(&store.elements, &store.config);
        let root = args.root.as_deref();
        let in_scope = |q: &str| root.is_none_or(|r| q.starts_with(r));
        let san = |q: &str| q.replace("::", "_").replace(['-', ' ', '.'], "_");
        let label = |e: &RawElement| {
            e.frontmatter
                .id
                .clone()
                .or_else(|| e.frontmatter.name.clone())
                .unwrap_or_else(|| e.qualified_name.clone())
        };
        let mut s = String::from("graph TD\n");
        match args.kind.as_str() {
            "traceability" => {
                for e in store.elements.iter().filter(|e| {
                    e.frontmatter.element_type == Some(ElementType::Requirement)
                        && in_scope(&e.qualified_name)
                }) {
                    let rid = san(&e.qualified_name);
                    s.push_str(&format!("  {rid}[\"{}\"]\n", label(e)));
                    s.push_str(&format!("  %% ref: {}\n", e.qualified_name));
                    let key = e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone());
                    if let Some(tcs) = result.verified_by.get(&key) {
                        for tid in tcs {
                            if let Some(tc) = store.resolver.resolve_ref(&store.elements, tid) {
                                let tcid = san(&tc.qualified_name);
                                s.push_str(&format!("  {tcid}[\"{}\"]\n", label(tc)));
                                s.push_str(&format!("  %% ref: {}\n", tc.qualified_name));
                                s.push_str(&format!("  {rid} --> {tcid}\n"));
                            }
                        }
                    }
                }
            }
            "containment" => {
                for e in store.elements.iter().filter(|e| in_scope(&e.qualified_name)) {
                    let nid = san(&e.qualified_name);
                    s.push_str(&format!("  {nid}[\"{}\"]\n", label(e)));
                    s.push_str(&format!("  %% ref: {}\n", e.qualified_name));
                    for child in children_of(&store.graph, &store.node_idx, &e.qualified_name) {
                        if in_scope(child) {
                            s.push_str(&format!("  {nid} --> {}\n", san(child)));
                        }
                    }
                }
            }
            "feature" => {
                for e in store.elements.iter().filter(|e| {
                    e.frontmatter.element_type == Some(ElementType::FeatureDef)
                        && in_scope(&e.qualified_name)
                }) {
                    let nid = san(&e.qualified_name);
                    s.push_str(&format!("  {nid}[\"{}\"]\n", label(e)));
                    s.push_str(&format!("  %% ref: {}\n", e.qualified_name));
                    for child in children_of(&store.graph, &store.node_idx, &e.qualified_name) {
                        s.push_str(&format!("  {nid} --> {}\n", san(child)));
                    }
                }
            }
            "allocation" => {
                for e in store.elements.iter().filter(|e| {
                    e.frontmatter.element_type == Some(ElementType::Allocation)
                        && in_scope(&e.qualified_name)
                }) {
                    let nid = san(&e.qualified_name);
                    s.push_str(&format!("  {nid}[\"{}\"]\n", label(e)));
                    s.push_str(&format!("  %% ref: {}\n", e.qualified_name));
                }
            }
            other => return tool_error(format!("unknown view kind: {other}")),
        }
        ok(json!({
            "kind": args.kind,
            "format": args.format.clone().unwrap_or_else(|| "mermaid".to_string()),
            "source": s,
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
        match store.resolver.resolve_ref(&store.elements, &args.r#ref) {
            Some(elem) => ok(variability::why_active(&store.elements, elem, &args.config)),
            None => tool_error(format!("unresolved reference: {}", args.r#ref)),
        }
    }

    #[tool(
        description = "Run an allow-listed read-only report command (audit, matrix, metrics, …) \
        over THIS model and return its output. The model root is fixed; arguments that \
        redirect the model/output or escape the root are refused.",
        annotations(read_only_hint = true)
    )]
    async fn run_report(
        &self,
        Parameters(args): Parameters<RunReportArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        if !REPORT_ALLOWLIST.contains(&args.command.as_str()) {
            return tool_error(format!(
                "command '{}' is not an allowed report command",
                args.command
            ));
        }
        let caller_args = args.args.clone().unwrap_or_default();
        for a in &caller_args {
            if let Err(e) = check_report_arg(a) {
                return tool_error(e);
            }
        }

        let model_root = { self.store.read().await.model_root.clone() };
        let exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => return tool_error(format!("cannot locate the syscribe executable: {e}")),
        };

        // argv: <command> -m <fixed model root> <sanitized caller args…> [--json]
        let mut argv: Vec<String> = vec![
            args.command.clone(),
            "-m".to_string(),
            model_root.to_string_lossy().into_owned(),
        ];
        argv.extend(caller_args.iter().cloned());
        if args.format.as_deref() == Some("json") {
            argv.push("--json".to_string());
        }

        let out = match std::process::Command::new(&exe).args(&argv).output() {
            Ok(o) => o,
            Err(e) => return tool_error(format!("failed to run report '{}': {e}", args.command)),
        };
        let output = String::from_utf8_lossy(&out.stdout).into_owned();
        let exit_code = out.status.code().unwrap_or(-1);
        ok(json!({
            "command": args.command,
            "args": caller_args,
            "output": output,
            "exitCode": exit_code,
        }))
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

    #[tool(
        description = "List suspect trace links (a baselined link whose target's content \
        changed since review, i.e. the W090 set) and, by default, links that have no \
        baseline yet. Each entry gives the source (id + qname), target ref, and link kind. \
        Read-only; deterministic ordering. See suspect_accept to clear a suspect link.",
        annotations(read_only_hint = true)
    )]
    async fn suspect_list(
        &self,
        Parameters(args): Parameters<SuspectListArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let links = syscribe_model::suspect::scan(&store.elements, &store.resolver);
        let to_json = |l: &syscribe_model::suspect::SuspectLink| {
            json!({
                "source": l.source_label(),
                "sourceQname": l.source_qname,
                "target": l.target_ref,
                "kind": l.kind,
            })
        };
        let suspect: Vec<Value> = links
            .iter()
            .filter(|l| l.state == syscribe_model::suspect::LinkState::Suspect)
            .map(to_json)
            .collect();
        let mut out = json!({ "suspect": suspect });
        if args.include_unbaselined {
            let unbaselined: Vec<Value> = links
                .iter()
                .filter(|l| l.state == syscribe_model::suspect::LinkState::Unbaselined)
                .map(to_json)
                .collect();
            out["unbaselined"] = Value::Array(unbaselined);
        }
        ok(out)
    }

    #[tool(
        description = "Baseline a reviewed trace link: capture the target's current content \
        hash into the source's traceBaselines, clearing its suspect flag (W090). Obeys the \
        write guard — dry_run defaults to true (returns a diff + validation delta, clearing \
        the link shows the W090 under resolvedWarnings; nothing is written). Pass \
        dry_run:false to commit. source/target accept a stable id or qualified name.",
        annotations(read_only_hint = false)
    )]
    async fn suspect_accept(
        &self,
        Parameters(args): Parameters<SuspectAcceptArgs>,
        peer: Peer<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut store = self.store.write().await;

        // Resolve against the live model first so an unresolved source / non-referenced
        // target is a clean tool error (no candidate staging).
        if let Err(e) =
            crate::suspect::plan_accept(&store.elements, &store.resolver, &args.source, &args.target)
        {
            return tool_error(e);
        }

        let extra = extra_map(json!({ "source": args.source, "target": args.target }));
        let (source_ref, target_ref) = (args.source.clone(), args.target.clone());
        // The baseline is recomputed against whichever tree the guard applies to
        // (temp candidate, then the real model on commit), so hashes match the
        // model actually being written.
        let apply = move |root: &Path| -> Result<(), String> {
            let elems = syscribe_model::walker::walk_model(root).map_err(|e| e.to_string())?;
            let resolver = Resolver::new(&elems);
            let plan =
                crate::suspect::plan_accept(&elems, &resolver, &source_ref, &target_ref)?;
            // plan.source_file is the absolute path under `root` produced by walk_model.
            crate::suspect::write_baseline(Path::new(&plan.source_file), &plan.authored_key, &plan.hash)
                .map_err(|e| e.to_string())
        };
        // gate=false: baselining never changes cross-references, so the referential-
        // integrity gate does not apply (like delete_element).
        let res = guarded_write(&mut store, args.dry_run, false, extra, apply);
        drop(store);
        notify_committed(&peer, &res).await;
        ok(res)
    }

    #[tool(
        description = "List every release Baseline (BL-*) with its id, name, status, date, \
        scope, gitTag/gitCommit, elementCount, and aggregateHash. Read-only. Sealing a new \
        baseline is a CLI/CI action (`baseline create`), not an MCP tool.",
        annotations(read_only_hint = true)
    )]
    async fn baseline_list(
        &self,
        Parameters(_args): Parameters<BaselineListArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let mut bs: Vec<&RawElement> = store
            .elements
            .iter()
            .filter(|e| syscribe_model::baseline::is_baseline(&e.frontmatter))
            .collect();
        bs.sort_by(|a, b| syscribe_model::baseline::element_key(b).cmp(&syscribe_model::baseline::element_key(a)));
        let list: Vec<Value> = bs
            .iter()
            .map(|b| {
                let fm = &b.frontmatter;
                json!({
                    "id": syscribe_model::baseline::element_key(b),
                    "name": fm.name,
                    "status": fm.status,
                    "date": fm.date,
                    "gitTag": fm.git_tag,
                    "gitCommit": fm.git_commit,
                    "scope": fm.frozen_scope,
                    "elementCount": fm.seal.as_ref().map(|s| s.element_count),
                    "aggregateHash": fm.seal.as_ref().map(|s| s.aggregate_hash.clone()),
                    "supersedes": fm.supersedes,
                })
            })
            .collect();
        ok(json!({ "baselines": list }))
    }

    #[tool(
        description = "Diff two release baselines by id: element-level added / removed / \
        changed (keyed by stable id, grouped by type) from their manifests, plus whether the \
        aggregates are identical. Read-only.",
        annotations(read_only_hint = true)
    )]
    async fn baseline_diff(
        &self,
        Parameters(args): Parameters<BaselineDiffArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let load = |id: &str| -> Option<syscribe_model::baseline::Manifest> {
            let b = store.resolver.resolve_ref(&store.elements, id)?;
            if !syscribe_model::baseline::is_baseline(&b.frontmatter) {
                return None;
            }
            let seal = b.frontmatter.seal.as_ref()?;
            syscribe_model::baseline::Manifest::from_file(&syscribe_model::baseline::manifest_path(
                &store.model_root,
                seal,
            ))
        };
        let (Some(a), Some(b)) = (load(&args.from), load(&args.to)) else {
            return tool_error(format!(
                "could not load manifests for `{}` and `{}` (both must be baselines with a manifest on disk)",
                args.from, args.to
            ));
        };
        ok(serde_json::to_value(syscribe_model::baseline::diff_manifests(&a, &b)).unwrap_or(Value::Null))
    }

    #[tool(
        description = "Verify release baselines: recompute each seal (content proof vs seal vs \
        manifest) and check git tag↔commit consistency, returning `passed` per baseline. Pass \
        `ref` for one, or omit to verify all. Read-only.",
        annotations(read_only_hint = true)
    )]
    async fn baseline_verify(
        &self,
        Parameters(args): Parameters<BaselineVerifyArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let store = self.store.read().await;
        let all = args.r#ref.as_deref().is_none_or(|r| r == "all");
        let targets: Vec<&RawElement> = if all {
            store.elements.iter().filter(|e| syscribe_model::baseline::is_baseline(&e.frontmatter)).collect()
        } else {
            match store.resolver.resolve_ref(&store.elements, args.r#ref.as_deref().unwrap()) {
                Some(e) if syscribe_model::baseline::is_baseline(&e.frontmatter) => vec![e],
                _ => return tool_error(format!("`{}` is not a baseline", args.r#ref.unwrap_or_default())),
            }
        };
        let results: Vec<Value> = targets
            .iter()
            .map(|b| {
                serde_json::to_value(syscribe_model::baseline::verify_baseline(
                    &store.elements,
                    b,
                    &store.model_root,
                ))
                .unwrap_or(Value::Null)
            })
            .collect();
        let passed = results.iter().all(|r| r.get("passed").and_then(|p| p.as_bool()).unwrap_or(false));
        ok(json!({ "passed": passed, "results": results }))
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

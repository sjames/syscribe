//! `GET /api/req-graph` and `/api/req-graph/overview` — the data layer of the Requirements
//! Explorer (GH #269, REQ-TRS-REQGRAPH-001): a bounded, typed traceability neighbourhood.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::projection::{resolve_config_flag, SelectionOutcome};
use syscribe_model::resolver::Resolver;

use crate::state::SharedState;

const MAX_DEPTH: usize = 6;
const DEFAULT_LIMIT: usize = 200;
const MAX_LIMIT: usize = 2000;

type Err = (StatusCode, Json<Value>);

fn bad(code: StatusCode, msg: impl Into<String>) -> Err {
    (code, Json(json!({"error": msg.into()})))
}

/// The node id of an element: its stable id, else its qualified name.
fn node_id(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Every relation of the model as `(from element, to element, kind)`, both ends resolved.
fn all_edges<'a>(elems: &'a [RawElement], resolver: &'a Resolver) -> Vec<(&'a RawElement, &'a RawElement, &'static str)> {
    let mut out = Vec::new();
    let pkg = syscribe_model::variability::package_conditions(elems);
    let features: Vec<&RawElement> = elems.iter().filter(|f| f.frontmatter.element_type == Some(ElementType::FeatureDef)).collect();
    for e in elems {
        let fm = &e.frontmatter;
        let mut push = |kind: &'static str, refs: &[String]| {
            for r in refs {
                if let Some(t) = resolver.resolve_ref(elems, r) {
                    if !std::ptr::eq(t, e) {
                        out.push((e, t, kind));
                    }
                }
            }
        };
        let lists: [(&'static str, Option<&Vec<String>>); 11] = [
            ("derivedFrom", fm.derived_from.as_ref()),
            ("satisfies", fm.satisfies.as_ref()),
            ("verifies", fm.verifies.as_ref()),
            ("allocatedTo", fm.allocated_to.as_ref()),
            ("refines", fm.refines.as_ref()),
            ("supersedes", fm.supersedes.as_ref()),
            ("blockedBy", fm.blocked_by.as_ref()),
            ("covers", fm.covers.as_ref()),
            ("analyses", fm.analyses.as_ref()),
            ("runsOn", fm.runs_on.as_ref()),
            ("achieves", fm.achieves.as_ref()),
        ];
        for (kind, list) in lists {
            if let Some(l) = list {
                push(kind, l);
            }
        }
        if let Some(g) = fm.derived_from_safety_goal.as_ref() {
            push("derivedFromSafetyGoal", std::slice::from_ref(g));
        }
        if let Some(a) = fm.breakdown_adr.as_ref() {
            push("breakdownAdr", std::slice::from_ref(a));
        }
        let mut ev: Vec<String> = Vec::new();
        for entry in fm.evidence.iter().flatten() {
            match entry {
                serde_yaml::Value::String(s) => ev.push(s.clone()),
                serde_yaml::Value::Mapping(m) => {
                    if let Some(r) = m.get("ref").and_then(|v| v.as_str()) {
                        ev.push(r.to_string());
                    }
                }
                _ => {}
            }
        }
        push("evidence", &ev);
        // `appliesWhen` operands (own, else the nearest ancestor package's) that name features.
        if let Some((aw, _)) = syscribe_model::variability::effective_applies_when(e, &pkg) {
            let text = match &aw {
                serde_yaml::Value::String(s) => s.clone(),
                serde_yaml::Value::Sequence(seq) => seq.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" "),
                _ => String::new(),
            };
            let toks = text
                .split(|c: char| c.is_whitespace() || "()!&|,[]".contains(c))
                .filter(|t| !t.is_empty() && !matches!(*t, "and" | "or" | "not" | "AND" | "OR" | "NOT"));
            for t in toks {
                if let Some(f) = features.iter().find(|f| f.qualified_name == t || f.frontmatter.id.as_deref() == Some(t)) {
                    out.push((e, f, "appliesWhen"));
                }
            }
        }
    }
    out
}

/// `qualified name -> (has a non-draft verifying TestCase, has only draft ones)`.
fn verification_map(elems: &[RawElement], resolver: &Resolver) -> HashMap<String, (bool, bool)> {
    let mut m: HashMap<String, (bool, bool)> = HashMap::new();
    for tc in elems.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::TestCase)) {
        // As the matrix/coverage classifier: a draft TestCase is planned intent, a retired one is
        // out of use, every other status (review, approved, active) counts as evidence.
        let status = tc.frontmatter.status.as_deref();
        if status == Some("retired") {
            continue;
        }
        let active = status != Some("draft");
        for r in tc.frontmatter.verifies.iter().flatten() {
            if let Some(t) = resolver.resolve_ref(elems, r) {
                let slot = m.entry(t.qualified_name.clone()).or_default();
                if active {
                    slot.0 = true;
                } else {
                    slot.1 = true;
                }
            }
        }
    }
    m
}

fn verification_of(e: &RawElement, vmap: &HashMap<String, (bool, bool)>) -> &'static str {
    if !Resolver::is_native_requirement(e) {
        return "na";
    }
    match vmap.get(&e.qualified_name) {
        Some((true, _)) => "verified",
        Some((false, true)) => "planned",
        _ => "unverified",
    }
}

fn node_json(e: &RawElement, root: bool, vmap: &HashMap<String, (bool, bool)>) -> Value {
    let fm = &e.frontmatter;
    json!({
        "id": node_id(e),
        "qname": e.qualified_name,
        "type": fm.element_type.as_ref().map(|t| t.name()),
        "name": fm.name,
        "status": fm.status,
        "reqClass": fm.req_class,
        "asil": fm.asil_level,
        "verification": verification_of(e, vmap),
        "root": root,
    })
}

/// The element set to answer from: the live model borrowed as is, or — for a `config` — its
/// projection onto that Configuration (owned). No placeholder substitution: bodies are not used.
fn view<'a>(live: &'a [RawElement], config: Option<&str>) -> Result<std::borrow::Cow<'a, [RawElement]>, Err> {
    use std::borrow::Cow;
    match config {
        None => Ok(Cow::Borrowed(live)),
        Some(c) => match resolve_config_flag(live, c) {
            SelectionOutcome::Dormant => Ok(Cow::Borrowed(live)),
            SelectionOutcome::Resolved(sel) => Ok(Cow::Owned(syscribe_model::projection::project_raw(live, &sel))),
            SelectionOutcome::Error(m) => Err(bad(StatusCode::BAD_REQUEST, m)),
        },
    }
}

const EDGE_KINDS: &[&str] = &[
    "derivedFrom", "satisfies", "verifies", "allocatedTo", "refines", "supersedes", "derivedFromSafetyGoal", "breakdownAdr",
    "blockedBy", "covers", "analyses", "runsOn", "achieves", "evidence", "appliesWhen",
];

fn parse_usize(params: &HashMap<String, String>, key: &str) -> Result<Option<usize>, Err> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v.parse::<usize>().map(Some).map_err(|_| bad(StatusCode::BAD_REQUEST, format!("`{key}` must be a non-negative integer"))),
    }
}

/// GET /api/req-graph?root=&depth=&edges=&limit=&config=
pub async fn get_req_graph(State(state): State<SharedState>, Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, Err> {
    let root_q = params.get("root").filter(|r| !r.trim().is_empty()).ok_or_else(|| bad(StatusCode::BAD_REQUEST, "`root` is required"))?;
    let depth = parse_usize(&params, "depth")?.unwrap_or(1).min(MAX_DEPTH);
    let limit = parse_usize(&params, "limit")?.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let kinds: Option<BTreeSet<&str>> = params.get("edges").map(|e| e.split(',').map(str::trim).filter(|k| !k.is_empty()).collect());

    if let Some(ks) = &kinds {
        if let Some(bad_kind) = ks.iter().find(|k| !EDGE_KINDS.contains(k)) {
            return Err(bad(StatusCode::BAD_REQUEST, format!("unknown edge kind '{bad_kind}' — valid: {}", EDGE_KINDS.join(", "))));
        }
    }
    // Computed while the read guard is held: the live model is borrowed, not cloned.
    let store = state.read().await;
    let config = params.get("config").map(String::as_str);
    let elems = view(&store.elements, config)?;
    let resolver = Resolver::new(&elems);
    // The root is an exact stable id or qualified name — never a fuzzy display-name match.
    let root = elems
        .iter()
        .find(|e| e.qualified_name == *root_q || e.frontmatter.id.as_deref() == Some(root_q.as_str()))
        .ok_or_else(|| {
            bad(
                StatusCode::NOT_FOUND,
                match config {
                    Some(c) => format!("'{root_q}' is not an element id or qualified name, or is not active in configuration '{c}'"),
                    None => format!("'{root_q}' is not an element id or qualified name"),
                },
            )
        })?;

    let edges: Vec<_> = all_edges(&elems, &resolver).into_iter().filter(|(_, _, k)| kinds.as_ref().is_none_or(|ks| ks.contains(k))).collect();
    // Undirected adjacency, deterministic.
    let mut adj: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (a, b, _) in &edges {
        adj.entry(a.qualified_name.as_str()).or_default().insert(b.qualified_name.as_str());
        adj.entry(b.qualified_name.as_str()).or_default().insert(a.qualified_name.as_str());
    }
    let mut seen: Vec<&str> = vec![root.qualified_name.as_str()];
    let mut seen_set: BTreeSet<&str> = seen.iter().copied().collect();
    let mut queue: VecDeque<(&str, usize)> = VecDeque::from([(root.qualified_name.as_str(), 0)]);
    let mut truncated = false;
    'bfs: while let Some((q, d)) = queue.pop_front() {
        if d >= depth {
            continue;
        }
        for n in adj.get(q).into_iter().flatten().copied() {
            if seen_set.contains(n) {
                continue;
            }
            if seen.len() >= limit {
                truncated = true;
                break 'bfs;
            }
            seen_set.insert(n);
            seen.push(n);
            queue.push_back((n, d + 1));
        }
    }
    let by_q: HashMap<&str, &RawElement> = elems.iter().map(|e| (e.qualified_name.as_str(), e)).collect();
    let vmap = verification_map(&elems, &resolver);
    let nodes: Vec<Value> = seen.iter().filter_map(|q| by_q.get(q)).map(|e| node_json(e, e.qualified_name == root.qualified_name, &vmap)).collect();
    let mut out_edges: Vec<Value> = edges
        .iter()
        .filter(|(a, b, _)| seen_set.contains(a.qualified_name.as_str()) && seen_set.contains(b.qualified_name.as_str()))
        .map(|(a, b, k)| json!({"from": node_id(a), "to": node_id(b), "fromQname": a.qualified_name, "toQname": b.qualified_name, "kind": k}))
        .collect();
    out_edges.sort_by_key(|e| e.to_string());
    out_edges.dedup();
    Ok(Json(json!({"root": node_id(root), "nodes": nodes, "edges": out_edges, "truncated": truncated})))
}

/// GET /api/req-graph/overview[?config=]
pub async fn get_overview(State(state): State<SharedState>, Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, Err> {
    let store = state.read().await;
    let elems = view(&store.elements, params.get("config").map(String::as_str))?;
    let resolver = Resolver::new(&elems);
    let vmap = verification_map(&elems, &resolver);
    let linked: BTreeSet<&str> = all_edges(&elems, &resolver)
        .iter()
        .filter(|(_, _, k)| *k != "appliesWhen")
        .flat_map(|(a, b, _)| [a.qualified_name.as_str(), b.qualified_name.as_str()])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let (mut by_class, mut by_status, mut verification) = (BTreeMap::<String, u64>::new(), BTreeMap::<String, u64>::new(), BTreeMap::<&str, u64>::new());
    let (mut total, mut unlinked) = (0u64, 0u64);
    for r in elems.iter().filter(|e| Resolver::is_native_requirement(e)) {
        total += 1;
        *by_class.entry(r.frontmatter.req_class.clone().unwrap_or_else(|| "(none)".into())).or_default() += 1;
        *by_status.entry(r.frontmatter.status.clone().unwrap_or_else(|| "(none)".into())).or_default() += 1;
        *verification.entry(verification_of(r, &vmap)).or_default() += 1;
        if !linked.contains(r.qualified_name.as_str()) {
            unlinked += 1;
        }
    }
    Ok(Json(json!({"requirements": total, "byClass": by_class, "byStatus": by_status, "unlinked": unlinked, "verification": verification})))
}

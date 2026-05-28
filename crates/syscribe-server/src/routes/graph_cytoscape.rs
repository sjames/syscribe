use std::collections::HashSet;
use axum::{extract::State, Json};
use serde::Serialize;
use crate::state::SharedState;

// ── Output structs ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct GraphResponse {
    pub nodes: Vec<CyNode>,
    pub edges: Vec<CyEdge>,
}

#[derive(Serialize)]
pub struct CyNode {
    pub data: NodeData,
}

#[derive(Serialize)]
pub struct NodeData {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub domain: Option<String>,
    pub status: Option<String>,
    pub qname: String,
    pub parent: Option<String>,
    #[serde(rename = "hasDoc")]
    pub has_doc: bool,
}

#[derive(Serialize)]
pub struct CyEdge {
    pub data: EdgeData,
}

#[derive(Serialize)]
pub struct EdgeData {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: String,
}

// ── Helper: sanitise a qname for use inside an edge ID ───────────────────────

fn sanitise(s: &str) -> String {
    s.replace("::", "-")
}

/// Extract the parent package qname from a qualified name (everything before
/// the last `::`), or `None` for root-level names.
fn parent_of(qname: &str) -> Option<String> {
    qname.rfind("::").map(|pos| qname[..pos].to_string())
}

/// Extract `from` and `to` string values from a YAML map used in
/// `connections:` / `flow_connections:` / etc.
fn connection_endpoints(v: &serde_yaml::Value) -> Option<(String, String)> {
    let map = v.as_mapping()?;
    let from = map
        .get(serde_yaml::Value::String("from".into()))
        .and_then(|v| v.as_str())
        .map(String::from)?;
    let to = map
        .get(serde_yaml::Value::String("to".into()))
        .and_then(|v| v.as_str())
        .map(String::from)?;
    Some((from, to))
}

/// Resolve a string that may be a stable ID (REQ-*, TC-*, …) to a qname using
/// the resolver, or return it unchanged if it already looks like a qname.
fn resolve_to_qname<'a>(
    resolver: &syscribe_model::resolver::Resolver,
    elements: &'a [syscribe_model::element::RawElement],
    r: &str,
) -> Option<String> {
    resolver
        .resolve_ref(elements, r)
        .map(|e| e.qualified_name.clone())
}

// ── Main handler ──────────────────────────────────────────────────────────────

/// GET /api/graph
/// Returns a Cytoscape-compatible node/edge JSON representation of the full
/// model.  Always returns 200 — on an empty model the arrays are empty.
pub async fn get_graph(State(state): State<SharedState>) -> Json<GraphResponse> {
    let store = state.read().await;

    let elements = &store.elements;
    let resolver = &store.resolver;

    // ── 1. Collect Package nodes ──────────────────────────────────────────────
    // For every element qname, every "::" prefix is a package namespace.
    let mut package_qnames: HashSet<String> = HashSet::new();
    for elem in elements.iter().filter(|e| !e.qualified_name.is_empty()) {
        let qn = &elem.qualified_name;
        let parts: Vec<&str> = qn.split("::").collect();
        // Emit packages for every prefix up to (but not including) the last segment.
        for len in 1..parts.len() {
            let pkg = parts[..len].join("::");
            package_qnames.insert(pkg);
        }
    }
    // Remove qnames that are already covered by actual Package-type elements.
    let explicit_pkg_qnames: HashSet<String> = elements
        .iter()
        .filter(|e| {
            matches!(
                e.frontmatter.element_type,
                Some(
                    syscribe_model::element::ElementType::Package
                        | syscribe_model::element::ElementType::LibraryPackage
                        | syscribe_model::element::ElementType::Namespace
                )
            )
        })
        .map(|e| e.qualified_name.clone())
        .collect();

    // ── 2. Build nodes ────────────────────────────────────────────────────────
    let mut nodes: Vec<CyNode> = Vec::new();

    // 2a. Package nodes (implicit namespaces only — explicit ones come from elements)
    for pkg_qn in &package_qnames {
        if explicit_pkg_qnames.contains(pkg_qn) {
            continue; // handled as a real element below
        }
        let label = pkg_qn
            .rsplit("::")
            .next()
            .unwrap_or(pkg_qn.as_str())
            .to_string();
        nodes.push(CyNode {
            data: NodeData {
                id: pkg_qn.clone(),
                label,
                node_type: "Package".to_string(),
                domain: None,
                status: None,
                qname: pkg_qn.clone(),
                parent: parent_of(pkg_qn),
                has_doc: false,
            },
        });
    }

    // 2b. Element nodes (skip root _index.md which has an empty qname)
    for elem in elements.iter().filter(|e| !e.qualified_name.is_empty()) {
        let type_str = elem
            .frontmatter
            .element_type
            .as_ref()
            .map(|t| format!("{:?}", t))
            .unwrap_or_else(|| "Unknown".to_string());

        let label = elem
            .frontmatter
            .name
            .clone()
            .unwrap_or_else(|| {
                elem.qualified_name
                    .rsplit("::")
                    .next()
                    .unwrap_or(&elem.qualified_name)
                    .to_string()
            });

        nodes.push(CyNode {
            data: NodeData {
                id: elem.qualified_name.clone(),
                label,
                node_type: type_str,
                domain: elem.frontmatter.domain.clone(),
                status: elem.frontmatter.status.clone(),
                qname: elem.qualified_name.clone(),
                parent: parent_of(&elem.qualified_name),
                has_doc: !elem.doc.trim().is_empty(),
            },
        });
    }

    // ── 3. Build edges ────────────────────────────────────────────────────────
    let mut seen_edges: HashSet<String> = HashSet::new();
    let mut edges: Vec<CyEdge> = Vec::new();

    let add_edge = |edges: &mut Vec<CyEdge>,
                        seen: &mut HashSet<String>,
                        source: &str,
                        target: &str,
                        kind: &str| {
        let edge_id = format!(
            "e-{}-{}-{}",
            sanitise(source),
            kind,
            sanitise(target)
        );
        if seen.insert(edge_id.clone()) {
            edges.push(CyEdge {
                data: EdgeData {
                    id: edge_id,
                    source: source.to_string(),
                    target: target.to_string(),
                    kind: kind.to_string(),
                },
            });
        }
    };

    for elem in elements.iter().filter(|e| !e.qualified_name.is_empty()) {
        let src = &elem.qualified_name;
        let fm = &elem.frontmatter;

        // supertype
        if let Some(ref st) = fm.supertype {
            for s in yaml_strings(st) {
                if let Some(tgt) = resolve_to_qname(resolver, elements, s) {
                    add_edge(&mut edges, &mut seen_edges, src, &tgt, "supertype");
                }
            }
        }

        // subsets
        if let Some(ref ss) = fm.subsets {
            for s in ss {
                if let Some(tgt) = resolve_to_qname(resolver, elements, s) {
                    add_edge(&mut edges, &mut seen_edges, src, &tgt, "subsets");
                }
            }
        }

        // redefines
        if let Some(ref rd) = fm.redefines {
            for s in yaml_strings(rd) {
                if let Some(tgt) = resolve_to_qname(resolver, elements, s) {
                    add_edge(&mut edges, &mut seen_edges, src, &tgt, "redefines");
                }
            }
        }

        // typedBy
        if let Some(ref tb) = fm.typed_by {
            for s in yaml_strings(tb) {
                if let Some(tgt) = resolve_to_qname(resolver, elements, s) {
                    add_edge(&mut edges, &mut seen_edges, src, &tgt, "typedBy");
                }
            }
        }

        // satisfies
        if let Some(ref sat) = fm.satisfies {
            for s in sat {
                if let Some(tgt) = resolve_to_qname(resolver, elements, s) {
                    add_edge(&mut edges, &mut seen_edges, src, &tgt, "satisfies");
                }
            }
        }

        // verifies
        if let Some(ref vs) = fm.verifies {
            for v in vs {
                if let Some(tgt) = resolve_to_qname(resolver, elements, v) {
                    add_edge(&mut edges, &mut seen_edges, src, &tgt, "verifies");
                }
            }
        }

        // derivedFrom
        if let Some(ref dfs) = fm.derived_from {
            for df in dfs {
                if let Some(tgt) = resolve_to_qname(resolver, elements, df) {
                    add_edge(&mut edges, &mut seen_edges, src, &tgt, "derivedFrom");
                }
            }
        }

        // allocatedTo
        if let Some(ref at_) = fm.allocated_to {
            if let Some(tgt) = resolve_to_qname(resolver, elements, at_) {
                add_edge(&mut edges, &mut seen_edges, src, &tgt, "allocatedTo");
            }
        }

        // breakdownAdr — requirement → the ADR that justifies its derivation
        if let Some(ref adr_ref) = fm.breakdown_adr {
            if let Some(tgt) = resolve_to_qname(resolver, elements, adr_ref) {
                add_edge(&mut edges, &mut seen_edges, src, &tgt, "breakdownAdr");
            }
        }

        // connections: extract from/to pairs from YAML maps
        let conn_lists: [Option<&Vec<serde_yaml::Value>>; 4] = [
            fm.connections.as_ref(),
            fm.flow_connections.as_ref(),
            fm.binding_connections.as_ref(),
            fm.succession_connections.as_ref(),
        ];
        for list_opt in conn_lists.iter() {
            if let Some(list) = list_opt {
                for v in list.iter() {
                    if let Some((from, to)) = connection_endpoints(v) {
                        // Qualify relative names against the containing element's parent namespace
                        let resolve_conn_ref = |r: &str| -> Option<String> {
                            // Try as-is first (absolute qname)
                            if resolver.resolve_ref(elements, r).is_some() {
                                return resolve_to_qname(resolver, elements, r);
                            }
                            // Try relative to containing namespace
                            if let Some(parent_ns) = parent_of(src) {
                                let qualified = format!("{}::{}", parent_ns, r);
                                if resolver.resolve_ref(elements, &qualified).is_some() {
                                    return Some(qualified);
                                }
                            }
                            None
                        };
                        if let (Some(from_qn), Some(to_qn)) =
                            (resolve_conn_ref(&from), resolve_conn_ref(&to))
                        {
                            add_edge(
                                &mut edges,
                                &mut seen_edges,
                                &from_qn,
                                &to_qn,
                                "connection",
                            );
                        }
                    }
                }
            }
        }
    }

    Json(GraphResponse { nodes, edges })
}

/// Extract string values from a YAML value that may be a String or Sequence.
fn yaml_strings(v: &serde_yaml::Value) -> Vec<&str> {
    match v {
        serde_yaml::Value::String(s) => vec![s.as_str()],
        serde_yaml::Value::Sequence(seq) => {
            seq.iter().filter_map(|x| x.as_str()).collect()
        }
        _ => vec![],
    }
}

use std::collections::HashMap;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use crate::element::RawElement;
use crate::resolver::Resolver;

/// Extract qualified name strings from a field that may be a YAML String or Sequence.
fn yaml_strings(v: &serde_yaml::Value) -> Vec<&str> {
    match v {
        serde_yaml::Value::String(s) => vec![s.as_str()],
        serde_yaml::Value::Sequence(seq) => seq.iter().filter_map(|x| x.as_str()).collect(),
        _ => vec![],
    }
}

/// Edge kinds in the model graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    Contains,      // parent package → child element
    Supertype,     // element → its supertype definition
    TypedBy,       // usage → its type definition
    Subsets,       // feature → subsetted feature
    Redefines,     // feature → redefined feature
    Verifies,      // test case → requirement (native or SysML)
    DerivedFrom,   // child requirement → parent requirement
    AllocatedFrom, // allocation source
    AllocatedTo,   // allocation target
    ConditionalOn, // element → FeatureDef (appliesWhen:)
    Satisfies,     // Part/Config → Requirement/FeatureDef (satisfies:)
}

pub type ModelGraph = DiGraph<String, EdgeKind>;

/// Build a graph from a flat list of elements.
/// Returns (graph, node_index_by_qname).
pub fn build_graph(elements: &[RawElement]) -> (ModelGraph, HashMap<String, NodeIndex>) {
    let resolver = Resolver::new(elements);
    let mut graph = DiGraph::new();
    let mut idx: HashMap<String, NodeIndex> = HashMap::new();

    // First pass: add all nodes
    for elem in elements {
        let ni = graph.add_node(elem.qualified_name.clone());
        idx.insert(elem.qualified_name.clone(), ni);
    }

    // Helper: resolve a ref string to a NodeIndex using ID-first lookup.
    let resolve_to_idx = |r: &str| -> Option<NodeIndex> {
        resolver
            .resolve_ref(elements, r)
            .and_then(|e| idx.get(&e.qualified_name))
            .copied()
    };

    // Second pass: add edges
    for elem in elements {
        let Some(&src) = idx.get(&elem.qualified_name) else { continue };

        // Contains: if qname has a "::" prefix, parent is everything before last "::"
        if let Some(sep) = elem.qualified_name.rfind("::") {
            let parent_qn = &elem.qualified_name[..sep];
            if let Some(&parent_ni) = idx.get(parent_qn) {
                graph.add_edge(parent_ni, src, EdgeKind::Contains);
            }
        }

        let fm = &elem.frontmatter;

        // Supertype — may be a string or a sequence
        if let Some(ref st) = fm.supertype {
            for s in yaml_strings(st) {
                if let Some(dst) = idx.get(s).copied() {
                    graph.add_edge(src, dst, EdgeKind::Supertype);
                }
            }
        }

        // TypedBy — may be a string or a sequence
        if let Some(ref tb) = fm.typed_by {
            for s in yaml_strings(tb) {
                if let Some(dst) = idx.get(s).copied() {
                    graph.add_edge(src, dst, EdgeKind::TypedBy);
                }
            }
        }

        // Subsets (Vec<String>)
        if let Some(ref ss) = fm.subsets {
            for s in ss {
                if let Some(dst) = idx.get(s.as_str()).copied() {
                    graph.add_edge(src, dst, EdgeKind::Subsets);
                }
            }
        }

        // Redefines — may be a string or a sequence
        if let Some(ref rd) = fm.redefines {
            for s in yaml_strings(rd) {
                if let Some(dst) = idx.get(s).copied() {
                    graph.add_edge(src, dst, EdgeKind::Redefines);
                }
            }
        }

        // Verifies — use ID-based resolution so REQ-* refs work
        if let Some(ref vs) = fm.verifies {
            for v in vs {
                if let Some(dst) = resolve_to_idx(v) {
                    graph.add_edge(src, dst, EdgeKind::Verifies);
                }
            }
        }

        // DerivedFrom — use ID-based resolution
        if let Some(ref dfs) = fm.derived_from {
            for df in dfs {
                if let Some(dst) = resolve_to_idx(df) {
                    graph.add_edge(src, dst, EdgeKind::DerivedFrom);
                }
            }
        }

        // allocatedFrom / allocatedTo
        if let Some(ref afs) = fm.allocated_from {
            for af in afs {
                if let Some(dst) = idx.get(af.as_str()).copied() {
                    graph.add_edge(src, dst, EdgeKind::AllocatedFrom);
                }
            }
        }
        if let Some(ref ats) = fm.allocated_to {
            for at_ in ats {
                if let Some(dst) = idx.get(at_.as_str()).copied() {
                    graph.add_edge(src, dst, EdgeKind::AllocatedTo);
                }
            }
        }

        // Satisfies — each entry resolves to a Requirement or FeatureDef
        if let Some(ref sat) = fm.satisfies {
            for s in sat {
                if let Some(dst) = resolve_to_idx(s) {
                    graph.add_edge(src, dst, EdgeKind::Satisfies);
                }
            }
        }

        // AppliesWhen — each resolved target must be a FeatureDef
        if let Some(ref aw) = fm.applies_when {
            for s in yaml_strings(aw) {
                if let Some(dst) = idx.get(s).copied() {
                    graph.add_edge(src, dst, EdgeKind::ConditionalOn);
                }
            }
        }
    }

    (graph, idx)
}

/// Return qualified names of direct children (Contains edges from node).
pub fn children_of<'a>(
    graph: &'a ModelGraph,
    idx: &'a HashMap<String, NodeIndex>,
    qname: &str,
) -> Vec<&'a str> {
    let Some(&ni) = idx.get(qname) else { return vec![] };
    graph
        .edges(ni)
        .filter(|e| *e.weight() == EdgeKind::Contains)
        .map(|e| graph[e.target()].as_str())
        .collect()
}

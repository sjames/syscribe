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
    Contains,              // parent package → child element
    Supertype,             // element → its supertype definition
    TypedBy,               // usage → its type definition
    Subsets,               // feature → subsetted feature
    Redefines,             // feature → redefined feature
    Verifies,              // test case → requirement (native or SysML)
    DerivedFrom,           // child requirement → parent requirement
    AllocatedFrom,         // allocation → source element
    AllocatedTo,           // allocation → target element
    ConditionalOn,         // element → FeatureDef (appliesWhen:)
    Satisfies,             // Part/Config → Requirement/FeatureDef (satisfies:)
    // Safety analysis (ISO 26262 / IEC 61508)
    TopEvent,              // FaultTree → SafetyGoal
    FaultTreeInput,        // FaultTreeGate → input gate/event
    HazardousEventRef,     // SafetyGoal → HazardousEvent
    DerivedFromSafetyGoal, // Requirement → SafetyGoal
    // Security analysis (ISO/SAE 21434)
    DamageScenarioRef,       // ThreatScenario → DamageScenario
    ThreatScenarioRef,       // CybersecurityGoal → ThreatScenario
    ImplementsGoal,          // SecurityControl → CybersecurityGoal
    MitigatedBy,             // VulnerabilityReport → SecurityControl
    DerivedFromSecurityGoal, // Requirement → CybersecurityGoal
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

        // ── Safety analysis ──────────────────────────────────────────────────

        // topEvent: FaultTree → SafetyGoal
        if let Some(ref te) = fm.top_event {
            if let Some(dst) = resolve_to_idx(te) {
                graph.add_edge(src, dst, EdgeKind::TopEvent);
            }
        }

        // inputs: FaultTreeGate → input gates/events
        if let Some(ref ins) = fm.inputs {
            for i in ins {
                if let Some(dst) = resolve_to_idx(i) {
                    graph.add_edge(src, dst, EdgeKind::FaultTreeInput);
                }
            }
        }

        // hazardousEvents: SafetyGoal → HazardousEvent refs
        if let Some(ref hes) = fm.hazardous_events {
            for he in hes {
                if let Some(dst) = resolve_to_idx(he) {
                    graph.add_edge(src, dst, EdgeKind::HazardousEventRef);
                }
            }
        }

        // derivedFromSafetyGoal: Requirement → SafetyGoal
        if let Some(ref sg) = fm.derived_from_safety_goal {
            if let Some(dst) = resolve_to_idx(sg) {
                graph.add_edge(src, dst, EdgeKind::DerivedFromSafetyGoal);
            }
        }

        // ── Security analysis ────────────────────────────────────────────────

        // damageScenarios: ThreatScenario → DamageScenario refs
        if let Some(ref ds) = fm.damage_scenarios {
            for d in ds {
                if let Some(dst) = resolve_to_idx(d) {
                    graph.add_edge(src, dst, EdgeKind::DamageScenarioRef);
                }
            }
        }

        // threatScenarios: CybersecurityGoal → ThreatScenario refs
        if let Some(ref ts) = fm.threat_scenarios {
            for t in ts {
                if let Some(dst) = resolve_to_idx(t) {
                    graph.add_edge(src, dst, EdgeKind::ThreatScenarioRef);
                }
            }
        }

        // implementsGoals: SecurityControl → CybersecurityGoal refs
        if let Some(ref ig) = fm.implements_goals {
            for g in ig {
                if let Some(dst) = resolve_to_idx(g) {
                    graph.add_edge(src, dst, EdgeKind::ImplementsGoal);
                }
            }
        }

        // mitigatedBy: VulnerabilityReport → SecurityControl refs
        if let Some(ref mbs) = fm.mitigated_by {
            for mb in mbs {
                if let Some(dst) = resolve_to_idx(mb) {
                    graph.add_edge(src, dst, EdgeKind::MitigatedBy);
                }
            }
        }

        // derivedFromSecurityGoal: Requirement → CybersecurityGoal
        if let Some(ref csg) = fm.derived_from_security_goal {
            if let Some(dst) = resolve_to_idx(csg) {
                graph.add_edge(src, dst, EdgeKind::DerivedFromSecurityGoal);
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

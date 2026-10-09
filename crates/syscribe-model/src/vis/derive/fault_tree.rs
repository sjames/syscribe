//! The FaultTree generator (GH #223).
//!
//! Subject: a `FaultTree`, or a `SafetyGoal` (every `FaultTree` whose
//! `topEvent:` names it). One node per `FaultTreeGate` / `FaultTreeEvent` of
//! the tree — a gate drawn with the symbol of its `gateType` (AND, OR, XOR,
//! NOT, inhibit), an event with the shape of its `eventKind` (basic circle,
//! undeveloped diamond, house pentagon) — and a [`EdgeKind::GateInput`] edge
//! from each gate to each of its `inputs:`.
//!
//! The overlay comes from [`crate::fta::analyze_fault_tree`], the one place
//! the numbers are computed (the `fault-tree analyze` report and the `metrics`
//! roll-up read the same): every event carries its probability (and λ), its
//! role (single point of failure, member of a dual / multi-point cut set,
//! irrelevant, unreachable, house constant) as status and tone, the number of
//! minimal cut sets it appears in as a badge, and the root carries the exact
//! top-event probability. When the analysis fails (a gate cycle, no top
//! node) the structure is still drawn, without the numbers, and the root says
//! why.

use std::collections::HashMap;

use crate::element::{ElementType, RawElement};
use crate::fta::{
    analyze_fault_tree, fault_trees, AnalysisOptions, EventOrigin, EventRole, FaultTreeAnalysis, FtKind, FtStructure, GateKind,
};
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, Node, NodeKind, NodeMark, Tone};
use super::super::manifest::Issue;
use super::requirement::{keeps_keys, unmatched_key_issues};
use super::{short_name, w418, Filters};

/// Scientific notation with two significant digits (`2.0e-9`).
pub(crate) fn sci(v: f64) -> String {
    if v == 0.0 {
        "0".to_string()
    } else {
        format!("{v:.1e}")
    }
}

fn gate_kind(k: Option<GateKind>) -> NodeKind {
    match k {
        Some(GateKind::And) => NodeKind::GateAnd,
        Some(GateKind::Xor) => NodeKind::GateXor,
        Some(GateKind::Not) => NodeKind::GateNot,
        Some(GateKind::Inhibit) => NodeKind::GateInhibit,
        // An OR gate, and a gate with no valid `gateType` (which the analysis
        // also treats as OR).
        Some(GateKind::Or) | None => NodeKind::GateOr,
    }
}

fn event_kind(kind: &str) -> NodeKind {
    match kind {
        "undeveloped" => NodeKind::EventUndeveloped,
        "house" => NodeKind::EventHouse,
        _ => NodeKind::EventBasic,
    }
}

/// The trees a subject stands for: itself, or the trees naming it as `topEvent`.
fn trees_of<'a>(subject: &'a RawElement, elements: &'a [RawElement], resolver: &Resolver) -> Option<Vec<&'a RawElement>> {
    match subject.frontmatter.element_type {
        Some(ElementType::FaultTree) => Some(vec![subject]),
        Some(ElementType::SafetyGoal) => Some(
            fault_trees(elements)
                .filter(|t| {
                    t.frontmatter
                        .top_event
                        .as_deref()
                        .and_then(|r| resolver.resolve_ref(elements, r))
                        .is_some_and(|g| g.qualified_name == subject.qualified_name)
                })
                .collect(),
        ),
        _ => None,
    }
}

fn keys(qname: &str, id: &str) -> Vec<String> {
    vec![qname.to_string(), id.to_string(), short_name(qname).to_string()]
}

/// The mark of an event from its analysis result.
fn event_mark(a: &FaultTreeAnalysis, id: &str, src: &RawElement) -> Option<NodeMark> {
    let e = a.events.iter().find(|e| e.id == id && e.origin == EventOrigin::Model)?;
    let mut value = Vec::new();
    if let Some(l) = e.failure_rate {
        value.push(format!("\u{03bb} {}/h", sci(l)));
    }
    if let Some(p) = e.probability {
        value.push(format!("P {}", sci(p)));
    }
    let (status, tone, emphasis) = match e.role {
        EventRole::SinglePoint => ("single point of failure", Tone::Bad, true),
        EventRole::DualPoint => ("dual-point", Tone::Warn, true),
        EventRole::MultiPoint => ("multi-point", Tone::Ok, false),
        EventRole::Irrelevant => ("in no cut set", Tone::Neutral, false),
        EventRole::Unreachable => ("unreachable", Tone::Neutral, false),
        EventRole::House => ("house event", Tone::Neutral, false),
    };
    let mut badges = Vec::new();
    if e.cut_set_count > 0 {
        badges.push(format!("{} MCS", e.cut_set_count));
    }
    if let Some(g) = src.frontmatter.ccf_group.as_deref() {
        badges.push(format!("CCF {g}"));
    }
    Some(NodeMark {
        status: Some(status.to_string()),
        value: (!value.is_empty()).then(|| value.join(" \u{00b7} ")),
        tone,
        badges,
        emphasis,
    })
}

/// The mark of the root node: the top-event probability, or why the analysis
/// is unavailable.
fn root_mark(result: &Result<FaultTreeAnalysis, crate::fta::FtaError>) -> NodeMark {
    match result {
        Ok(a) => {
            let singles = a.cut_sets.iter().filter(|c| c.order == 1).count();
            let mut badges = vec![format!("{} MCS", a.cut_sets.len())];
            if singles > 0 {
                badges.push(format!("{singles} single point"));
            }
            if !a.coherent {
                badges.push("non-coherent".to_string());
            }
            if a.truncated {
                badges.push("truncated".to_string());
            }
            NodeMark {
                status: Some("top event".to_string()),
                value: a.top_probability.map(|p| format!("P {}", sci(p))),
                tone: if singles > 0 { Tone::Warn } else { Tone::Neutral },
                badges,
                emphasis: true,
            }
        }
        Err(e) => NodeMark {
            status: Some("analysis unavailable".to_string()),
            value: Some(e.to_string().chars().take(48).collect()),
            tone: Tone::Warn,
            badges: Vec::new(),
            emphasis: true,
        },
    }
}

/// The diagram of one `FaultTree` without a `Diagram` element behind it (the
/// `fault-tree render --format` command): the same derivation, no filters.
pub fn tree_diagram(elements: &[RawElement], resolver: &Resolver, tree: &RawElement) -> DiagramGraph {
    let name = tree.frontmatter.name.clone().unwrap_or_else(|| short_name(&tree.qualified_name).to_string());
    let mut graph = DiagramGraph::empty(super::super::ir::DiagramKind::FaultTree, "", &name, Some(&tree.qualified_name));
    graph.derived = true;
    generate(&mut graph, tree, elements, resolver, &Filters::default(), &mut Vec::new());
    graph
}

pub fn generate(
    graph: &mut DiagramGraph,
    subject: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    filters: &Filters,
    issues: &mut Vec<Issue>,
) {
    let Some(trees) = trees_of(subject, elements, resolver) else {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — a FaultTree diagram subject must be a FaultTree or a SafetyGoal",
            subject.qualified_name,
            subject.frontmatter.element_type.as_ref().map(|t| t.name()).unwrap_or("element")
        )));
        return;
    };
    if trees.is_empty() {
        issues.push(w418(format!("`subject` '{}' is named as `topEvent` by no FaultTree — nothing to draw", subject.qualified_name)));
        return;
    }

    let by_qname: HashMap<&str, &RawElement> = elements.iter().map(|e| (e.qualified_name.as_str(), e)).collect();
    let mut candidates: Vec<Vec<String>> = Vec::new();
    let mut staged: Vec<(Vec<Node>, Vec<Edge>)> = Vec::new();

    for tree in &trees {
        let st = FtStructure::build(elements, resolver, tree);
        let result = analyze_fault_tree(elements, resolver, tree, &AnalysisOptions::default());
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let ids: Vec<String> = st.nodes.iter().map(|n| derived_shape_id(&n.qname)).collect();
        for (i, n) in st.nodes.iter().enumerate() {
            candidates.push(keys(&n.qname, &n.id));
            let src = by_qname.get(n.qname.as_str()).copied();
            let (kind, element_type) = match &n.kind {
                FtKind::Gate(k) => (gate_kind(*k), "FaultTreeGate"),
                FtKind::Event(k) => (event_kind(k), "FaultTreeEvent"),
            };
            let is_root = st.root == Some(i);
            let mut mark = match (&n.kind, &result) {
                (FtKind::Event(_), Ok(a)) => src.and_then(|s| event_mark(a, &n.id, s)),
                _ => None,
            };
            if is_root {
                mark = Some(root_mark(&result));
            } else if !st.reachable.contains(&i) && mark.is_none() {
                mark = Some(NodeMark { status: Some("unreachable".to_string()), tone: Tone::Neutral, ..Default::default() });
            }
            if let (FtKind::Gate(None), Some(m)) = (&n.kind, mark.as_mut()) {
                m.badges.push("no gateType".to_string());
            } else if let FtKind::Gate(None) = &n.kind {
                mark = Some(NodeMark { tone: Tone::Warn, badges: vec!["no gateType".to_string()], ..Default::default() });
            }
            nodes.push(Node {
                id: ids[i].clone(),
                element_ref: n.qname.clone(),
                resolved: true,
                element_type: Some(element_type.to_string()),
                kind,
                label: n.name.clone(),
                stereotype: Some(n.id.clone()),
                parent: None,
                direction: None,
                side: None,
                lines: Vec::new(),
                is_abstract: false,
                pin: None,
                banners: Vec::new(),
                feature: None,
                mark,
            });
            for &t in &n.inputs {
                edges.push(Edge {
                    id: format!("e-input-{}-{}", ids[i], ids[t]),
                    element_ref: Some(n.qname.clone()),
                    source: ids[i].clone(),
                    target: ids[t].clone(),
                    kind: EdgeKind::GateInput,
                    label: None,
                    waypoints: None,
                });
            }
        }
        staged.push((nodes, edges));
    }

    unmatched_key_issues(filters, &candidates, issues);
    for (nodes, edges) in staged {
        let kept: Vec<Node> = nodes
            .into_iter()
            .filter(|n| {
                let id = n.stereotype.clone().unwrap_or_default();
                keeps_keys(filters, &keys(&n.element_ref, &id))
            })
            .collect();
        let ids: std::collections::HashSet<&str> = kept.iter().map(|n| n.id.as_str()).collect();
        let edges: Vec<Edge> = edges.into_iter().filter(|e| ids.contains(e.source.as_str()) && ids.contains(e.target.as_str())).collect();
        graph.nodes.extend(kept);
        graph.edges.extend(edges);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vis::derive::testkit::*;
    use crate::vis::ir::DiagramKind;

    pub(crate) fn tree_model() -> Vec<RawElement> {
        vec![
            raw("Safety", ElementType::Package, |_| {}),
            raw("Safety::SG1", ElementType::SafetyGoal, |fm| {
                fm.id = Some("SG-TX-001".into());
            }),
            raw("Safety::FT", ElementType::FaultTree, |fm| {
                fm.id = Some("FT-TX-001".into());
                fm.top_event = Some("SG-TX-001".into());
                fm.mission_time = Some("1000 h".into());
            }),
            raw("Safety::FT::Top", ElementType::FaultTreeGate, |fm| {
                fm.id = Some("FTG-TX-001".into());
                fm.gate_type = Some("OR".into());
                fm.inputs = Some(vec!["FTG-TX-002".into(), "FTE-TX-003".into()]);
            }),
            raw("Safety::FT::Both", ElementType::FaultTreeGate, |fm| {
                fm.id = Some("FTG-TX-002".into());
                fm.gate_type = Some("AND".into());
                fm.inputs = Some(vec!["FTE-TX-001".into(), "FTE-TX-002".into()]);
            }),
            raw("Safety::FT::A", ElementType::FaultTreeEvent, |fm| {
                fm.id = Some("FTE-TX-001".into());
                fm.event_kind = Some("basic".into());
                fm.failure_rate = Some(1e-4);
            }),
            raw("Safety::FT::B", ElementType::FaultTreeEvent, |fm| {
                fm.id = Some("FTE-TX-002".into());
                fm.event_kind = Some("undeveloped".into());
                fm.probability = Some(0.01);
            }),
            raw("Safety::FT::C", ElementType::FaultTreeEvent, |fm| {
                fm.id = Some("FTE-TX-003".into());
                fm.event_kind = Some("basic".into());
                fm.probability = Some(1e-6);
            }),
        ]
    }

    fn derive_with(model: Vec<RawElement>, subject: &str) -> (DiagramGraph, Vec<Issue>) {
        let d = diagram("FaultTree", subject, |_| {});
        let mut elements = model;
        elements.push(d.clone());
        let resolver = Resolver::new(&elements);
        super::super::derive(&d, DiagramKind::FaultTree, &elements, &resolver)
    }

    #[test]
    fn gates_events_and_inputs_are_drawn_with_their_symbols() {
        let (g, issues) = derive_with(tree_model(), "Safety::FT");
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.nodes.len(), 5);
        assert_eq!(g.node("s-safety-ft-top").unwrap().kind, NodeKind::GateOr);
        assert_eq!(g.node("s-safety-ft-both").unwrap().kind, NodeKind::GateAnd);
        assert_eq!(g.node("s-safety-ft-a").unwrap().kind, NodeKind::EventBasic);
        assert_eq!(g.node("s-safety-ft-b").unwrap().kind, NodeKind::EventUndeveloped);
        assert_eq!(g.node("s-safety-ft-top").unwrap().stereotype.as_deref(), Some("FTG-TX-001"));
        assert_eq!(g.edges.len(), 4);
        assert!(g.edges.iter().all(|e| e.kind == EdgeKind::GateInput));
        assert!(g.edges.iter().any(|e| e.source == "s-safety-ft-top" && e.target == "s-safety-ft-c"));
    }

    #[test]
    fn analysis_overlay_marks_single_points_and_the_root_probability() {
        let (g, _) = derive_with(tree_model(), "Safety::FT");
        let root = g.node("s-safety-ft-top").unwrap().mark.clone().unwrap();
        assert_eq!(root.status.as_deref(), Some("top event"));
        assert!(root.value.as_deref().unwrap().starts_with("P "), "{root:?}");
        assert!(root.emphasis);
        assert_eq!(root.tone, Tone::Warn, "C alone causes the top event");
        // C is an order-1 cut set.
        let c = g.node("s-safety-ft-c").unwrap().mark.clone().unwrap();
        assert_eq!((c.status.as_deref(), c.tone, c.emphasis), (Some("single point of failure"), Tone::Bad, true));
        // A and B form an order-2 cut set; A's probability is 1 - e^(-0.1).
        let a = g.node("s-safety-ft-a").unwrap().mark.clone().unwrap();
        assert_eq!((a.status.as_deref(), a.tone), (Some("dual-point"), Tone::Warn));
        assert!(a.value.as_deref().unwrap().contains("\u{03bb} 1.0e-4/h"), "{a:?}");
        assert_eq!(a.badges, vec!["1 MCS"]);
    }

    #[test]
    fn a_safety_goal_subject_finds_its_trees() {
        let (g, issues) = derive_with(tree_model(), "Safety::SG1");
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.nodes.len(), 5);
    }

    #[test]
    fn a_gate_cycle_degrades_to_the_bare_structure() {
        let mut m = tree_model();
        // C feeds back into the AND gate it sits under: a cycle.
        m.iter_mut().find(|e| e.qualified_name == "Safety::FT::A").unwrap().frontmatter.inputs = Some(vec!["FTG-TX-002".into()]);
        let (g, _) = derive_with(m, "Safety::FT");
        assert_eq!(g.nodes.len(), 5, "the structure is still drawn");
        let root = g.node("s-safety-ft-top").unwrap().mark.clone().unwrap();
        assert_eq!(root.status.as_deref(), Some("analysis unavailable"));
        assert!(root.value.as_deref().unwrap().contains("cycle"));
        assert!(g.node("s-safety-ft-c").unwrap().mark.is_none(), "no per-event numbers without an analysis");
    }

    #[test]
    fn a_subject_view_needs_no_diagram_element() {
        use crate::vis::{build_subject_graph, safety_kinds_of};
        let elements = tree_model();
        let resolver = Resolver::new(&elements);
        let goal = elements.iter().find(|e| e.qualified_name == "Safety::SG1").unwrap();
        let tree = elements.iter().find(|e| e.qualified_name == "Safety::FT").unwrap();
        assert_eq!(safety_kinds_of(tree, &elements, &resolver), vec![DiagramKind::FaultTree]);
        assert_eq!(safety_kinds_of(goal, &elements, &resolver), vec![DiagramKind::SafetyCase, DiagramKind::FaultTree]);
        let pkg = elements.iter().find(|e| e.qualified_name == "Safety").unwrap();
        assert!(safety_kinds_of(pkg, &elements, &resolver).is_empty());
        let (g, issues) = build_subject_graph(tree, DiagramKind::FaultTree, &elements, &resolver);
        assert!(issues.is_empty() && g.derived);
        assert_eq!(g.nodes.len(), 5);
        assert_eq!(g.subject.as_deref(), Some("Safety::FT"));
    }

    #[test]
    fn a_wrong_subject_is_w418_and_a_goal_without_a_tree_too() {
        let (g, issues) = derive_with(tree_model(), "Safety");
        assert!(g.nodes.is_empty());
        assert_eq!(issues.iter().filter(|i| i.code == "W418").count(), 1, "{issues:?}");
        let mut m = tree_model();
        m.retain(|e| !e.qualified_name.starts_with("Safety::FT"));
        let (g, issues) = derive_with(m, "Safety::SG1");
        assert!(g.nodes.is_empty());
        assert_eq!(issues.iter().filter(|i| i.code == "W418").count(), 1, "{issues:?}");
    }

    #[test]
    fn include_and_exclude_filter_by_id_and_report_w417() {
        let d = diagram("FaultTree", "Safety::FT", |fm| {
            fm.exclude = Some(vec!["FTE-TX-003".into(), "nope".into()]);
        });
        let mut elements = tree_model();
        elements.push(d.clone());
        let resolver = Resolver::new(&elements);
        let (g, issues) = super::super::derive(&d, DiagramKind::FaultTree, &elements, &resolver);
        assert_eq!(g.nodes.len(), 4);
        assert!(g.edges.iter().all(|e| e.target != "s-safety-ft-c"));
        assert_eq!(issues.iter().filter(|i| i.code == "W417").count(), 1);
    }
}

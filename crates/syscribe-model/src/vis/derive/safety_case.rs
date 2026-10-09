//! The SafetyCase (GSN) generator (GH #223).
//!
//! Subject: a `SafetyGoal` (its argument), an `Argument` (the argument of every
//! goal it belongs to) or a package (the arguments of every goal under it). The structure is `syscribe_model::safety_case::build` — the
//! one traversal the `safety-case` report also renders — mapped onto GSN
//! symbols: the goal and every claim / requirement a rectangle, a strategy a
//! parallelogram, a solution (and a test case, the evidence) a circle, context
//! a rounded box, justification and assumption (and an `AssumptionOfUse`) an
//! ellipse with `J` / `A`, and a node with nothing under it a goal wearing
//! GSN's undeveloped diamond. `SupportedBy` edges run from a node to what
//! supports it, `InContextOf` edges to its context.
//!
//! The overlay is the node's rolled-up `NodeStatus` (tone), `UNDEVELOPED` on
//! the node that originates the gap, the test verdict on a test case and the
//! goal verdict on the root. Test verdicts come from the `.syscribe/results.json`
//! sidecar of the registered model root ([`super::context`]) and read `unknown`
//! only for a test case the sidecar does not cover; [`graph_of_case`] renders a
//! case built by the caller (the `safety-case --format` CLI).

use std::collections::HashSet;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::safety_case::{build, BuildOptions, GoalTree, GoalVerdict, NodeKind as SkKind, NodeStatus, SafetyCase, SafetyCaseNode};

use super::super::ir::{derived_shape_id, DiagramGraph, DiagramKind, Edge, EdgeKind, Node, NodeKind, NodeMark, Tone};
use super::super::manifest::Issue;
use super::requirement::{is_under, keeps_keys, unmatched_key_issues};
use super::{short_name, w418, Filters};

fn is_package(t: &ElementType) -> bool {
    matches!(t, ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace)
}

fn node_kind(n: &SafetyCaseNode) -> NodeKind {
    let undeveloped = n.undeveloped || n.kind == SkKind::UndevelopedClaim;
    match &n.kind {
        SkKind::Strategy => NodeKind::Strategy,
        SkKind::Solution | SkKind::TestCase | SkKind::Other(_) => NodeKind::Solution,
        SkKind::Context => NodeKind::Context,
        SkKind::Justification => NodeKind::Justification,
        SkKind::Assumption | SkKind::AssumptionOfUse => NodeKind::Assumption,
        SkKind::Goal | SkKind::Claim | SkKind::UndevelopedClaim | SkKind::Requirement | SkKind::Unresolved => {
            if undeveloped {
                NodeKind::UndevelopedGoal
            } else {
                NodeKind::Goal
            }
        }
    }
}

fn element_type(n: &SafetyCaseNode) -> Option<String> {
    Some(
        match &n.kind {
            SkKind::Goal => "SafetyGoal".to_string(),
            SkKind::Requirement => "Requirement".to_string(),
            SkKind::TestCase => "TestCase".to_string(),
            SkKind::AssumptionOfUse => "AssumptionOfUse".to_string(),
            SkKind::Other(t) => t.clone(),
            SkKind::Unresolved => return None,
            _ => "Argument".to_string(),
        },
    )
}

fn tone_of(status: NodeStatus) -> Tone {
    match status {
        NodeStatus::Supported => Tone::Ok,
        NodeStatus::Unverified | NodeStatus::Undeveloped => Tone::Warn,
        NodeStatus::Unresolved | NodeStatus::Failing => Tone::Bad,
        NodeStatus::Context => Tone::Neutral,
    }
}

fn mark_of(n: &SafetyCaseNode, goal_verdict: Option<GoalVerdict>) -> Option<NodeMark> {
    if n.status == NodeStatus::Context && n.kind != SkKind::TestCase {
        return None;
    }
    let status = if n.undeveloped || n.kind == SkKind::UndevelopedClaim {
        "UNDEVELOPED".to_string()
    } else if n.kind == SkKind::Unresolved {
        "unresolved".to_string()
    } else {
        n.status.as_str().to_string()
    };
    let mut badges = Vec::new();
    if n.implicit {
        badges.push("implicit".to_string());
    }
    if n.cycle {
        badges.push("cycle".to_string());
    }
    if let Some(k) = &n.decomposition_kind {
        badges.push(k.clone());
    }
    let value = match (n.verdict, goal_verdict) {
        (Some(v), _) => Some(v.as_str().to_string()),
        (None, Some(g)) => Some(format!("verdict {}", g.as_str())),
        _ => None,
    };
    Some(NodeMark { detail: None, status: Some(status), value, tone: tone_of(n.status), badges, emphasis: goal_verdict.is_some() })
}

fn node_ref(n: &SafetyCaseNode) -> String {
    n.qualified_name.clone().unwrap_or_else(|| n.id.clone())
}

fn keys(n: &SafetyCaseNode) -> Vec<String> {
    let mut k = vec![n.id.clone()];
    if let Some(q) = &n.qualified_name {
        k.push(q.clone());
        k.push(short_name(q).to_string());
    }
    k
}

struct Emitter<'a> {
    graph: &'a mut DiagramGraph,
    seen: HashSet<String>,
    filters: &'a Filters,
}

impl Emitter<'_> {
    /// Emit `n` (once per diagram) and its subtree; returns the node's id, or
    /// `None` when a filter dropped it.
    fn walk(&mut self, n: &SafetyCaseNode, goal_verdict: Option<GoalVerdict>) -> Option<String> {
        if goal_verdict.is_none() && !keeps_keys(self.filters, &keys(n)) {
            return None;
        }
        let id = derived_shape_id(&node_ref(n));
        if !self.seen.insert(id.clone()) {
            return Some(id);
        }
        // The label is the concise id; the statement hangs under it, wrapped.
        let mut mark = mark_of(n, goal_verdict);
        if !n.title.is_empty() && n.title != n.id {
            mark.get_or_insert_with(NodeMark::default).detail = Some(n.title.clone());
        }
        self.graph.nodes.push(Node {
            id: id.clone(),
            element_ref: node_ref(n),
            resolved: n.kind != SkKind::Unresolved,
            element_type: element_type(n),
            kind: node_kind(n),
            label: n.id.clone(),
            stereotype: None,
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
        // A node that is its own ancestor was not expanded (the cycle is
        // flagged on it): its edge back to the ancestor is drawn by the parent.
        for c in &n.children {
            let Some(cid) = self.walk(c, None) else { continue };
            let kind = if c.kind.is_context() { EdgeKind::InContextOf } else { EdgeKind::SupportedBy };
            let eid = format!("e-{}-{id}-{cid}", kind.as_str());
            if !self.graph.edges.iter().any(|e| e.id == eid) {
                self.graph.edges.push(Edge { id: eid, element_ref: Some(node_ref(n)), source: id.clone(), target: cid, kind, label: None, waypoints: None });
            }
        }
        Some(id)
    }
}

/// Draw `goals` (already built, with whatever verdicts the caller had) into
/// `graph`, de-duplicating nodes shared between goals.
pub fn add_goals(graph: &mut DiagramGraph, goals: &[GoalTree], filters: &Filters) {
    let mut e = Emitter { graph, seen: HashSet::new(), filters };
    for g in goals {
        e.walk(&g.root, Some(g.verdict));
    }
}

/// A standalone GSN diagram of a built [`SafetyCase`] (every goal in it).
pub fn graph_of_case(case: &SafetyCase, name: &str) -> DiagramGraph {
    let mut g = DiagramGraph::empty(DiagramKind::SafetyCase, "", name, None);
    g.derived = true;
    add_goals(&mut g, &case.goals, &Filters::default());
    g
}

pub fn generate(
    graph: &mut DiagramGraph,
    subject: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    filters: &Filters,
    issues: &mut Vec<Issue>,
) {
    let Some(st) = subject.frontmatter.element_type.as_ref() else { return };
    let is_goal = matches!(st, ElementType::SafetyGoal);
    let is_argument = matches!(st, ElementType::Argument);
    if !(is_goal || is_argument || is_package(st)) {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — a SafetyCase diagram subject must be a SafetyGoal, an Argument or a Package",
            subject.qualified_name,
            st.name()
        )));
        return;
    }
    // Test verdicts come from the results sidecar of the registered model
    // root (`unknown` for a test case it does not cover, or with no sidecar).
    let results = super::context::results();
    let verdict_of = |tc: &RawElement| crate::results::testcase_verdict(tc, results.as_ref());
    let case = build(elements, resolver, "", BuildOptions::default(), &verdict_of);
    let goals: Vec<GoalTree> = case
        .goals
        .into_iter()
        .filter(|g| {
            let q = g.root.qualified_name.as_deref().unwrap_or("");
            if is_goal {
                q == subject.qualified_name
            } else if is_argument {
                // The goals whose argument contains this Argument.
                let mut found = false;
                g.root.walk(&mut |n| found |= n.qualified_name.as_deref() == Some(subject.qualified_name.as_str()));
                found
            } else {
                is_under(q, &subject.qualified_name)
            }
        })
        .collect();
    if goals.is_empty() {
        issues.push(w418(format!("`subject` '{}' covers no SafetyGoal — nothing to draw", subject.qualified_name)));
        return;
    }
    let mut candidates: Vec<Vec<String>> = Vec::new();
    for g in &goals {
        g.root.walk(&mut |n| {
            if n.id != g.root.id {
                candidates.push(keys(n));
            }
        });
    }
    unmatched_key_issues(filters, &candidates, issues);
    add_goals(graph, &goals, filters);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vis::derive::testkit::*;

    fn model() -> Vec<RawElement> {
        vec![
            raw("Safe", ElementType::Package, |_| {}),
            raw("Safe::SG", ElementType::SafetyGoal, |fm| {
                fm.id = Some("SG-TX-001".into());
                fm.name = Some("Avoid harm".into());
            }),
            raw("Safe::ARG1", ElementType::Argument, |fm| {
                fm.id = Some("ARG-TX-001".into());
                fm.name = Some("Argue over monitors".into());
                fm.argument_type = Some("strategy".into());
                fm.supports = Some(vec!["SG-TX-001".into()]);
                fm.evidence = Some(vec!["REQ-TX-001".into(), "ARG-TX-002".into(), "ARG-TX-003".into()]);
            }),
            raw("Safe::ARG2", ElementType::Argument, |fm| {
                fm.id = Some("ARG-TX-002".into());
                fm.name = Some("Monitor is independent".into());
                fm.supports = Some(vec!["ARG-TX-001".into()]);
            }),
            raw("Safe::ARG3", ElementType::Argument, |fm| {
                fm.id = Some("ARG-TX-003".into());
                fm.name = Some("Operating context".into());
                fm.argument_type = Some("context".into());
                fm.supports = Some(vec!["ARG-TX-001".into()]);
            }),
            raw("Safe::REQ", ElementType::Requirement, |fm| {
                fm.id = Some("REQ-TX-001".into());
                fm.name = Some("Monitor shall trip".into());
                fm.derived_from_safety_goal = Some("SG-TX-001".into());
            }),
            raw("Safe::TC", ElementType::TestCase, |fm| {
                fm.id = Some("TC-TX-001".into());
                fm.name = Some("Trip test".into());
                fm.verifies = Some(vec!["REQ-TX-001".into()]);
            }),
        ]
    }

    fn derive_with(m: Vec<RawElement>, subject: &str) -> (DiagramGraph, Vec<Issue>) {
        let d = diagram("SafetyCase", subject, |_| {});
        let mut elements = m;
        elements.push(d.clone());
        let resolver = Resolver::new(&elements);
        super::super::derive(&d, DiagramKind::SafetyCase, &elements, &resolver)
    }

    #[test]
    fn the_argument_maps_onto_gsn_shapes_and_edges() {
        let (g, issues) = derive_with(model(), "Safe::SG");
        assert!(issues.is_empty(), "{issues:?}");
        let kind = |id: &str| g.node(id).unwrap_or_else(|| panic!("{id} in {:?}", g.nodes.iter().map(|n| &n.id).collect::<Vec<_>>())).kind;
        assert_eq!(kind("s-safe-sg"), NodeKind::Goal);
        assert_eq!(kind("s-safe-arg1"), NodeKind::Strategy);
        assert_eq!(kind("s-safe-tc"), NodeKind::Solution);
        assert_eq!(kind("s-safe-arg3"), NodeKind::Context);
        // ARG2 supports the strategy but has nothing under it.
        assert_eq!(kind("s-safe-arg2"), NodeKind::UndevelopedGoal);
        let edge = |t: &str| g.edges.iter().find(|e| e.target == t).map(|e| e.kind);
        assert_eq!(edge("s-safe-arg1"), Some(EdgeKind::SupportedBy));
        assert_eq!(edge("s-safe-arg3"), Some(EdgeKind::InContextOf));
    }

    #[test]
    fn overlays_carry_status_verdicts_and_the_goal_verdict() {
        let (g, _) = derive_with(model(), "Safe::SG");
        let root = g.node("s-safe-sg").unwrap().mark.clone().unwrap();
        assert_eq!(root.value.as_deref(), Some("verdict incomplete"));
        assert!(root.emphasis);
        let und = g.node("s-safe-arg2").unwrap().mark.clone().unwrap();
        assert_eq!((und.status.as_deref(), und.tone), (Some("UNDEVELOPED"), Tone::Warn));
        let tc = g.node("s-safe-tc").unwrap().mark.clone().unwrap();
        assert_eq!((tc.value.as_deref(), tc.tone), (Some("unknown"), Tone::Warn));
        assert!(g.node("s-safe-arg3").unwrap().mark.as_ref().is_none_or(|m| m.is_detail_only()), "context nodes carry no status");
    }

    #[test]
    fn a_package_subject_draws_every_goal_and_a_wrong_subject_is_w418() {
        let (g, issues) = derive_with(model(), "Safe");
        assert!(issues.is_empty(), "{issues:?}");
        assert!(g.node("s-safe-sg").is_some());
        let (g, issues) = derive_with(model(), "Safe::REQ");
        assert!(g.nodes.is_empty());
        assert_eq!(issues.iter().filter(|i| i.code == "W418").count(), 1);
    }

    #[test]
    fn graph_of_case_renders_a_built_case_with_real_verdicts() {
        let elements = model();
        let resolver = Resolver::new(&elements);
        let pass = |_: &RawElement| crate::safety_case::Verdict::Pass;
        let case = build(&elements, &resolver, "SG-TX-001", BuildOptions::default(), &pass);
        let g = graph_of_case(&case, "SG");
        let tc = g.node("s-safe-tc").unwrap().mark.clone().unwrap();
        assert_eq!((tc.value.as_deref(), tc.tone), (Some("pass"), Tone::Ok));
    }
}

//! The AttackTree generator (GH #223).
//!
//! Subject: an `AttackTree`, or a `ThreatScenario` (every `AttackTree` whose
//! `threatRef:` names it). One node per `AttackTreeGate` (AND / OR symbol) and
//! `AttackStep` under the tree, with a [`EdgeKind::GateInput`] edge from each
//! gate to each of its `inputs:`.
//!
//! The colouring is the weakest-link roll-up of [`crate::attack_tree`] — the
//! single definition the validator's `W035` also uses — evaluated per node:
//! high feasibility is red, medium amber, low and very low green, a node the
//! roll-up cannot score grey. The root carries the tree's rolled-up
//! feasibility; the easiest path (an OR gate follows its most feasible input,
//! an AND gate every input) is emphasised, its edges drawn heavier; and a
//! tree whose computed feasibility disagrees with its `ThreatScenario`'s
//! declared one wears the `W035` badge, exactly the reconciliation the
//! validator reports.

use std::collections::{BTreeSet, HashMap};

use crate::attack_tree::{feasibility_label, node_feasibility_rank, reconcile, tree_root};
use crate::cyber_config::CyberConfig;
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, Node, NodeKind, NodeMark, Tone};
use super::super::manifest::Issue;
use super::requirement::{element_keys, keeps_keys, unmatched_key_issues};
use super::{w418, Filters};

fn is_node(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::AttackTreeGate) | Some(ElementType::AttackStep))
}

fn tone_of(rank: Option<u8>) -> Tone {
    match rank {
        Some(3) => Tone::Bad,
        Some(2) => Tone::Warn,
        Some(_) => Tone::Ok,
        None => Tone::Neutral,
    }
}

/// The trees a subject stands for: itself, or the trees naming it as `threatRef`.
fn trees_of<'a>(subject: &'a RawElement, elements: &'a [RawElement], resolver: &Resolver) -> Option<Vec<&'a RawElement>> {
    match subject.frontmatter.element_type {
        Some(ElementType::AttackTree) => Some(vec![subject]),
        Some(ElementType::ThreatScenario) => Some(
            elements
                .iter()
                .filter(|t| matches!(t.frontmatter.element_type, Some(ElementType::AttackTree)))
                .filter(|t| {
                    t.frontmatter
                        .threat_ref
                        .as_deref()
                        .and_then(|r| resolver.resolve_ref(elements, r))
                        .is_some_and(|g| g.qualified_name == subject.qualified_name)
                })
                .collect(),
        ),
        _ => None,
    }
}

/// The qualified names of the nodes on the easiest path from `root`, and the
/// `(from, to)` pairs of the edges along it.
fn easiest_path(
    root: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    cfg: &CyberConfig,
) -> (BTreeSet<String>, BTreeSet<(String, String)>) {
    let mut nodes = BTreeSet::new();
    let mut edges = BTreeSet::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if !nodes.insert(n.qualified_name.clone()) {
            continue;
        }
        if !matches!(n.frontmatter.element_type, Some(ElementType::AttackTreeGate)) {
            continue;
        }
        let kids: Vec<&RawElement> = n
            .frontmatter
            .inputs
            .iter()
            .flatten()
            .filter_map(|r| resolver.resolve_ref(elements, r))
            .filter(|c| is_node(c))
            .collect();
        let chosen: Vec<&RawElement> = if n.frontmatter.gate_type.as_deref() == Some("AND") {
            kids
        } else {
            // OR (alternatives): the attacker's easiest input; the first on a tie.
            let mut best: Option<(u8, &RawElement)> = None;
            for k in kids {
                if let Some(r) = node_feasibility_rank(k, elements, resolver, cfg) {
                    if best.map_or(true, |(b, _)| r > b) {
                        best = Some((r, k));
                    }
                }
            }
            best.map(|(_, k)| vec![k]).unwrap_or_default()
        };
        for c in chosen {
            edges.insert((n.qualified_name.clone(), c.qualified_name.clone()));
            stack.push(c);
        }
    }
    (nodes, edges)
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
            "`subject` '{}' is a {} — an AttackTree diagram subject must be an AttackTree or a ThreatScenario",
            subject.qualified_name,
            subject.frontmatter.element_type.as_ref().map(|t| t.name()).unwrap_or("element")
        )));
        return;
    };
    if trees.is_empty() {
        issues.push(w418(format!("`subject` '{}' is named as `threatRef` by no AttackTree — nothing to draw", subject.qualified_name)));
        return;
    }
    let cfg = CyberConfig::default();

    let mut candidates: Vec<Vec<String>> = Vec::new();
    let mut staged: Vec<(Vec<Node>, Vec<Edge>)> = Vec::new();
    for tree in &trees {
        // The tree's own nodes, then any gate input that lives elsewhere.
        let prefix = format!("{}::", tree.qualified_name);
        let mut members: Vec<&RawElement> = elements.iter().filter(|e| is_node(e) && e.qualified_name.starts_with(&prefix)).collect();
        members.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
        let mut known: HashMap<String, usize> = members.iter().enumerate().map(|(i, e)| (e.qualified_name.clone(), i)).collect();
        let mut i = 0;
        while i < members.len() {
            for r in members[i].frontmatter.inputs.iter().flatten() {
                if let Some(t) = resolver.resolve_ref(elements, r).filter(|t| is_node(t)) {
                    if !known.contains_key(&t.qualified_name) {
                        known.insert(t.qualified_name.clone(), members.len());
                        members.push(t);
                    }
                }
            }
            i += 1;
        }

        let root = tree_root(tree, elements, resolver);
        let (path_nodes, path_edges) = root.map(|r| easiest_path(r, elements, resolver, &cfg)).unwrap_or_default();
        let rec = reconcile(tree, elements, resolver, &cfg);

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        for e in &members {
            candidates.push(element_keys(e));
            let rank = node_feasibility_rank(e, elements, resolver, &cfg);
            let is_root = root.is_some_and(|r| r.qualified_name == e.qualified_name);
            let kind = match e.frontmatter.element_type {
                Some(ElementType::AttackStep) => NodeKind::Step,
                _ if e.frontmatter.gate_type.as_deref() == Some("AND") => NodeKind::GateAnd,
                _ => NodeKind::GateOr,
            };
            let mut badges = Vec::new();
            if is_root {
                if let Some(r) = rec.as_ref().filter(|r| r.mismatch()) {
                    badges.push("W035".to_string());
                    let _ = r;
                }
            }
            let mut mark = NodeMark {
                status: Some(match rank {
                    Some(r) => format!("feasibility {}", feasibility_label(r)),
                    None => "unscored".to_string(),
                }),
                value: None,
                tone: tone_of(rank),
                badges,
                emphasis: path_nodes.contains(&e.qualified_name),
            };
            if is_root {
                mark.emphasis = true;
                if let Some(r) = &rec {
                    mark.value = Some(if r.mismatch() {
                        format!("threat {} declares {}", r.threat_ref, r.declared)
                    } else {
                        format!("matches threat {}", r.threat_ref)
                    });
                }
            }
            nodes.push(Node {
                id: derived_shape_id(&e.qualified_name),
                element_ref: e.qualified_name.clone(),
                resolved: true,
                element_type: e.frontmatter.element_type.as_ref().map(|t| t.name().to_string()),
                kind,
                label: e.frontmatter.name.clone().or_else(|| e.frontmatter.id.clone()).unwrap_or_else(|| super::short_name(&e.qualified_name).to_string()),
                stereotype: e.frontmatter.id.clone(),
                parent: None,
                direction: None,
                side: None,
                lines: Vec::new(),
                is_abstract: false,
                pin: None,
                banners: Vec::new(),
                feature: None,
                mark: Some(mark),
            });
            for r in e.frontmatter.inputs.iter().flatten() {
                let Some(t) = resolver.resolve_ref(elements, r).filter(|t| known.contains_key(&t.qualified_name)) else { continue };
                let (s, d) = (derived_shape_id(&e.qualified_name), derived_shape_id(&t.qualified_name));
                let id = format!("e-input-{s}-{d}");
                if edges.iter().any(|x: &Edge| x.id == id) {
                    continue;
                }
                let hot = path_edges.contains(&(e.qualified_name.clone(), t.qualified_name.clone()));
                edges.push(Edge {
                    id,
                    element_ref: Some(e.qualified_name.clone()),
                    source: s,
                    target: d,
                    kind: if hot { EdgeKind::CriticalPath } else { EdgeKind::GateInput },
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
                let mut keys = vec![n.element_ref.clone(), super::short_name(&n.element_ref).to_string()];
                keys.extend(n.stereotype.clone());
                keeps_keys(filters, &keys)
            })
            .collect();
        let ids: BTreeSet<&str> = kept.iter().map(|n| n.id.as_str()).collect();
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

    fn step(q: &str, id: &str, feas: &str) -> RawElement {
        raw(q, ElementType::AttackStep, |fm| {
            fm.id = Some(id.into());
            fm.attack_feasibility = Some(feas.into());
        })
    }

    /// The worked example of `attack_tree.rs`: OR[ AND[high, low], medium ] is medium.
    fn model(declared: &str) -> Vec<RawElement> {
        vec![
            raw("Sec", ElementType::Package, |_| {}),
            raw("Sec::TS1", ElementType::ThreatScenario, |fm| {
                fm.id = Some("TS-TX-001".into());
                fm.attack_feasibility = Some(declared.into());
            }),
            raw("Sec::AT", ElementType::AttackTree, |fm| {
                fm.id = Some("AT-TX-001".into());
                fm.threat_ref = Some("TS-TX-001".into());
            }),
            raw("Sec::AT::Root", ElementType::AttackTreeGate, |fm| {
                fm.id = Some("ATG-TX-001".into());
                fm.gate_type = Some("OR".into());
                fm.inputs = Some(vec!["ATG-TX-002".into(), "ATS-TX-003".into()]);
            }),
            raw("Sec::AT::Chain", ElementType::AttackTreeGate, |fm| {
                fm.id = Some("ATG-TX-002".into());
                fm.gate_type = Some("AND".into());
                fm.inputs = Some(vec!["ATS-TX-001".into(), "ATS-TX-002".into()]);
            }),
            step("Sec::AT::S1", "ATS-TX-001", "high"),
            step("Sec::AT::S2", "ATS-TX-002", "low"),
            step("Sec::AT::S3", "ATS-TX-003", "medium"),
        ]
    }

    fn derive_with(m: Vec<RawElement>, subject: &str) -> (DiagramGraph, Vec<Issue>) {
        let d = diagram("AttackTree", subject, |_| {});
        let mut elements = m;
        elements.push(d.clone());
        let resolver = Resolver::new(&elements);
        super::super::derive(&d, DiagramKind::AttackTree, &elements, &resolver)
    }

    #[test]
    fn nodes_are_coloured_by_rolled_up_feasibility() {
        let (g, issues) = derive_with(model("medium"), "Sec::AT");
        assert!(issues.is_empty(), "{issues:?}");
        let tone = |id: &str| g.node(id).unwrap().mark.as_ref().unwrap().tone;
        assert_eq!(g.node("s-sec-at-root").unwrap().kind, NodeKind::GateOr);
        assert_eq!(g.node("s-sec-at-chain").unwrap().kind, NodeKind::GateAnd);
        assert_eq!(g.node("s-sec-at-s1").unwrap().kind, NodeKind::Step);
        assert_eq!(tone("s-sec-at-s1"), Tone::Bad, "high");
        assert_eq!(tone("s-sec-at-s2"), Tone::Ok, "low");
        assert_eq!(tone("s-sec-at-chain"), Tone::Ok, "AND = min(high, low) = low");
        assert_eq!(tone("s-sec-at-root"), Tone::Warn, "OR = max(low, medium) = medium");
        let root = g.node("s-sec-at-root").unwrap().mark.clone().unwrap();
        assert_eq!(root.status.as_deref(), Some("feasibility medium"));
        assert_eq!(root.value.as_deref(), Some("matches threat TS-TX-001"));
        assert!(root.badges.is_empty());
    }

    #[test]
    fn the_easiest_path_is_emphasised() {
        let (g, _) = derive_with(model("medium"), "Sec::AT");
        let emph = |id: &str| g.node(id).unwrap().mark.as_ref().unwrap().emphasis;
        assert!(emph("s-sec-at-root") && emph("s-sec-at-s3"));
        assert!(!emph("s-sec-at-chain") && !emph("s-sec-at-s1"), "the OR follows medium, not the low chain");
        let kind = |t: &str| g.edges.iter().find(|e| e.target == t).unwrap().kind;
        assert_eq!(kind("s-sec-at-s3"), EdgeKind::CriticalPath);
        assert_eq!(kind("s-sec-at-chain"), EdgeKind::GateInput);
    }

    #[test]
    fn a_threat_mismatch_wears_the_w035_badge() {
        let (g, _) = derive_with(model("high"), "Sec::AT");
        let root = g.node("s-sec-at-root").unwrap().mark.clone().unwrap();
        assert_eq!(root.badges, vec!["W035"]);
        assert_eq!(root.value.as_deref(), Some("threat TS-TX-001 declares high"));
    }

    #[test]
    fn an_unscored_step_is_neutral_and_a_threat_subject_finds_its_tree() {
        let mut m = model("medium");
        m.iter_mut().find(|e| e.qualified_name == "Sec::AT::S3").unwrap().frontmatter.attack_feasibility = None;
        let (g, issues) = derive_with(m, "Sec::TS1");
        assert!(issues.is_empty(), "{issues:?}");
        let m3 = g.node("s-sec-at-s3").unwrap().mark.clone().unwrap();
        assert_eq!((m3.status.as_deref(), m3.tone), (Some("unscored"), Tone::Neutral));
    }

    #[test]
    fn a_wrong_subject_is_w418() {
        let (g, issues) = derive_with(model("medium"), "Sec");
        assert!(g.nodes.is_empty());
        assert_eq!(issues.iter().filter(|i| i.code == "W418").count(), 1);
    }
}

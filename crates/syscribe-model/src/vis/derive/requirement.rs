//! The Requirement generator (`REQ-TRS-VIS-020`).
//!
//! Subject: a `Package`/`LibraryPackage`/`Namespace`, a `RequirementDef` or a
//! `Requirement`. Produces one `Requirement` node per native `Requirement`,
//! SysML `RequirementDef` or `Requirement` that is the subject or lies under
//! it at any depth, with a `Compartment` child carrying the stable `id` and
//! `status` when present; `Derive` edges from each requirement to its
//! `derivedFrom:` targets and `Refine` edges to its `refines:` targets;
//! `Satisfy` edges from a `Block` context node (the satisfying element, drawn
//! with its real type's stereotype) for every element whose `satisfies:`
//! names a requirement on the diagram; `Verify` edges from a `TestCase`
//! context node for every `TestCase` whose `verifies:` names one; and
//! `Containment` edges from a `RequirementDef` to each requirement it owns.
//! An edge is emitted only when both ends are nodes of the diagram, after
//! `include:`/`exclude:`, which apply to requirements and context nodes alike
//! by qualified name, stable id or short name.

use std::collections::BTreeSet;

use crate::element::{ElementType, RawElement};
use crate::members::direct_members;
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, Node, NodeKind};
use super::super::manifest::Issue;
use super::{block_node, short_name, w417, w418, Filters};

fn is_package(t: &ElementType) -> bool {
    matches!(t, ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace)
}

fn is_requirement(t: &ElementType) -> bool {
    matches!(t, ElementType::Requirement | ElementType::RequirementDef)
}

/// Whether `qname` is `root` or lies under it at any depth (the model root,
/// whose qualified name is empty, contains everything).
pub(crate) fn is_under(qname: &str, root: &str) -> bool {
    root.is_empty() || qname == root || qname.strip_prefix(root).map(|rest| rest.starts_with("::")).unwrap_or(false)
}

/// The names an `include:`/`exclude:` entry may use for an element: its
/// qualified name, its stable `id` when it has one, and its short name.
pub(crate) fn element_keys(e: &RawElement) -> Vec<String> {
    let mut keys = vec![e.qualified_name.clone()];
    if let Some(id) = e.frontmatter.id.as_deref() {
        keys.push(id.to_string());
    }
    keys.push(short_name(&e.qualified_name).to_string());
    keys
}

/// [`Filters::keeps`] over several names of one member.
pub(crate) fn keeps_keys(filters: &Filters, keys: &[String]) -> bool {
    let hit = |list: &[String]| list.iter().any(|e| keys.iter().any(|k| k == e));
    (filters.include.is_empty() || hit(&filters.include)) && !hit(&filters.exclude)
}

/// `W417` for every `include:`/`exclude:` entry matching none of the
/// candidates' names (`candidates` are the [`element_keys`] of each member).
pub(crate) fn unmatched_key_issues(filters: &Filters, candidates: &[Vec<String>], issues: &mut Vec<Issue>) {
    for (field, list) in [("include", &filters.include), ("exclude", &filters.exclude)] {
        for entry in list {
            if !candidates.iter().any(|keys| keys.iter().any(|k| k == entry)) {
                issues.push(w417(format!("`{field}` entry '{entry}' names no member of the subject")));
            }
        }
    }
}

/// A requirement's display label: its `name`, else its stable `id`, else
/// its short name.
fn requirement_label(e: &RawElement) -> String {
    e.frontmatter
        .name
        .clone()
        .or_else(|| e.frontmatter.id.clone())
        .unwrap_or_else(|| short_name(&e.qualified_name).to_string())
}

/// The compartment lines of a requirement: `id = …` and `status = …` when present.
fn requirement_lines(e: &RawElement) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(id) = e.frontmatter.id.as_deref() {
        lines.push(format!("id = {id}"));
    }
    if let Some(status) = e.frontmatter.status.as_deref() {
        lines.push(format!("status = {status}"));
    }
    lines
}

/// The requirement node of `e` and, when it has any lines, its compartment.
fn requirement_nodes(e: &RawElement, elements: &[RawElement], resolver: &Resolver) -> Vec<Node> {
    let id = derived_shape_id(&e.qualified_name);
    let mut out = vec![block_node(id.clone(), e, NodeKind::Requirement, None, requirement_label(e), elements, resolver)];
    let lines = requirement_lines(e);
    if !lines.is_empty() {
        out.push(Node {
            id: format!("{id}-compartment"),
            element_ref: e.qualified_name.clone(),
            resolved: true,
            element_type: None,
            kind: NodeKind::Compartment,
            label: String::new(),
            stereotype: None,
            parent: Some(id),
            direction: None,
            side: None,
            lines,
            is_abstract: false,
            pin: None,
            banners: Vec::new(),
            feature: None,
        });
    }
    out
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
    if !(is_package(st) || is_requirement(st)) {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — a Requirement diagram subject must be a Package, RequirementDef or Requirement",
            subject.qualified_name,
            st.name()
        )));
        return;
    }
    let sq = subject.qualified_name.as_str();

    // ── candidates: the requirements under the subject, then the elements
    //    that satisfy or verify any of them ────────────────────────────────
    let mut reqs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| e.frontmatter.element_type.as_ref().map(is_requirement).unwrap_or(false))
        .filter(|e| is_under(&e.qualified_name, sq))
        .collect();
    reqs.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
    let req_qnames: BTreeSet<&str> = reqs.iter().map(|e| e.qualified_name.as_str()).collect();
    let names_a_candidate = |refs: Option<&Vec<String>>| -> bool {
        refs.map(|v| v.iter().any(|r| resolver.resolve_ref(elements, r).map(|t| req_qnames.contains(t.qualified_name.as_str())).unwrap_or(false)))
            .unwrap_or(false)
    };
    let mut context: Vec<&RawElement> = elements
        .iter()
        .filter(|e| !req_qnames.contains(e.qualified_name.as_str()))
        .filter(|e| {
            names_a_candidate(e.frontmatter.satisfies.as_ref())
                || (matches!(e.frontmatter.element_type, Some(ElementType::TestCase)) && names_a_candidate(e.frontmatter.verifies.as_ref()))
        })
        .collect();
    context.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));

    // ── filters ─────────────────────────────────────────────────────────
    let candidates: Vec<Vec<String>> = reqs.iter().chain(context.iter()).map(|e| element_keys(e)).collect();
    unmatched_key_issues(filters, &candidates, issues);
    reqs.retain(|e| keeps_keys(filters, &element_keys(e)));
    context.retain(|e| keeps_keys(filters, &element_keys(e)));

    // ── requirement nodes ───────────────────────────────────────────────
    let req_ids: BTreeSet<String> = reqs.iter().map(|e| derived_shape_id(&e.qualified_name)).collect();
    for e in &reqs {
        graph.nodes.extend(requirement_nodes(e, elements, resolver));
    }
    // The requirement node a reference (qualified name or stable id) lands on.
    let req_node_of = |r: &str| -> Option<String> {
        let t = resolver.resolve_ref(elements, r)?;
        let id = derived_shape_id(&t.qualified_name);
        req_ids.contains(&id).then_some(id)
    };

    // ── edges between requirements ──────────────────────────────────────
    let mut edges: Vec<Edge> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut push = |edges: &mut Vec<Edge>, kind: EdgeKind, src: String, tgt: String, element_ref: &str| {
        let id = format!("e-{}-{src}-{tgt}", kind.as_str());
        if seen.insert(id.clone()) {
            edges.push(Edge { id, element_ref: Some(element_ref.to_string()), source: src, target: tgt, kind, label: None, waypoints: None });
        }
    };
    for e in &reqs {
        let src = derived_shape_id(&e.qualified_name);
        // Containment: a RequirementDef → each requirement it owns.
        if matches!(e.frontmatter.element_type, Some(ElementType::RequirementDef)) {
            for child in direct_members(elements, &e.qualified_name) {
                if let Some(tgt) = req_node_of(&child.qualified_name) {
                    push(&mut edges, EdgeKind::Containment, src.clone(), tgt, &e.qualified_name);
                }
            }
        }
        // Derive: child → parent; Refine: refining → refined.
        for (list, kind) in [(e.frontmatter.derived_from.as_ref(), EdgeKind::Derive), (e.frontmatter.refines.as_ref(), EdgeKind::Refine)] {
            for r in list.into_iter().flatten() {
                if let Some(tgt) = req_node_of(r) {
                    push(&mut edges, kind, src.clone(), tgt, &e.qualified_name);
                }
            }
        }
        // A requirement on the diagram that itself satisfies another.
        for r in e.frontmatter.satisfies.iter().flatten() {
            if let Some(tgt) = req_node_of(r) {
                push(&mut edges, EdgeKind::Satisfy, src.clone(), tgt, &e.qualified_name);
            }
        }
    }

    // ── context nodes: satisfiers and verifiers, with their edges ───────
    for e in &context {
        let src = derived_shape_id(&e.qualified_name);
        let is_test = matches!(e.frontmatter.element_type, Some(ElementType::TestCase));
        let mut own: Vec<Edge> = Vec::new();
        for r in e.frontmatter.satisfies.iter().flatten() {
            if let Some(tgt) = req_node_of(r) {
                push(&mut own, EdgeKind::Satisfy, src.clone(), tgt, &e.qualified_name);
            }
        }
        if is_test {
            for r in e.frontmatter.verifies.iter().flatten() {
                if let Some(tgt) = req_node_of(r) {
                    push(&mut own, EdgeKind::Verify, src.clone(), tgt, &e.qualified_name);
                }
            }
        }
        // A context node only earns its place through an edge to a kept requirement.
        if own.is_empty() {
            continue;
        }
        let kind = if is_test { NodeKind::TestCase } else { NodeKind::Block };
        let label = e.frontmatter.name.clone().unwrap_or_else(|| short_name(&e.qualified_name).to_string());
        graph.nodes.push(block_node(src, e, kind, None, label, elements, resolver));
        edges.extend(own);
    }
    graph.edges.extend(edges);
}

#[cfg(test)]
mod tests {
    use super::super::testkit::*;
    use super::*;

    fn edge_tuples(g: &DiagramGraph) -> Vec<(EdgeKind, String, String)> {
        g.edges.iter().map(|e| (e.kind, e.source.clone(), e.target.clone())).collect()
    }

    #[test]
    fn package_subject_lists_requirements_with_compartments_context_nodes_and_edges() {
        let d = diagram("Requirement", "Reqs", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        let reqs: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::Requirement).map(|n| n.element_ref.as_str()).collect();
        assert_eq!(reqs, vec!["Reqs::Parent", "Reqs::Safety", "Reqs::Safety::Child"], "the subject's requirements at any depth, sorted");
        let parent = g.node("s-reqs-parent").unwrap();
        assert_eq!(parent.stereotype.as_deref(), Some("requirement"));
        assert_eq!(parent.element_type.as_deref(), Some("Requirement"));
        assert_eq!(parent.label, "Parent requirement", "labelled by name");
        let comp = g.node("s-reqs-parent-compartment").unwrap();
        assert_eq!(comp.kind, NodeKind::Compartment);
        assert_eq!(comp.parent.as_deref(), Some("s-reqs-parent"));
        assert_eq!(comp.lines, vec!["id = REQ-TK-001", "status = approved"]);
        let def = g.node("s-reqs-safety").unwrap();
        assert_eq!(def.stereotype.as_deref(), Some("requirement def"));
        assert!(def.is_abstract);
        assert!(g.node("s-reqs-safety-compartment").is_none(), "no id, no status: no compartment");
        assert_eq!(g.node("s-reqs-safety-child-compartment").unwrap().lines, vec!["id = REQ-TK-002"]);
        // Context nodes with their real type's stereotype.
        let ctrl = g.node("s-reqs-controller").unwrap();
        assert_eq!((ctrl.kind, ctrl.stereotype.as_deref(), ctrl.element_type.as_deref()), (NodeKind::Block, Some("part def"), Some("PartDef")));
        let tc = g.node("s-reqs-controllertest").unwrap();
        assert_eq!((tc.kind, tc.stereotype.as_deref()), (NodeKind::TestCase, Some("test case")));
        // Edges: child → parent derive, def → owned requirement containment,
        // block → requirement satisfy, test case → requirement verify.
        let edges = edge_tuples(&g);
        assert!(edges.contains(&(EdgeKind::Containment, "s-reqs-safety".into(), "s-reqs-safety-child".into())));
        assert!(edges.contains(&(EdgeKind::Derive, "s-reqs-safety-child".into(), "s-reqs-parent".into())));
        assert!(edges.contains(&(EdgeKind::Satisfy, "s-reqs-controller".into(), "s-reqs-safety-child".into())));
        assert!(edges.contains(&(EdgeKind::Verify, "s-reqs-controllertest".into(), "s-reqs-safety-child".into())));
        assert_eq!(edges.len(), 4, "{edges:?}");
        assert!(g.edges.iter().all(|e| e.label.is_none()), "the «keyword» comes from the style, not a label");
        assert!(g.edges.iter().any(|e| e.id == "e-derive-s-reqs-safety-child-s-reqs-parent"));
        assert_eq!(g.layout_hints.reversed_kinds, vec![EdgeKind::Derive, EdgeKind::Satisfy, EdgeKind::Verify, EdgeKind::Refine]);
    }

    #[test]
    fn requirement_def_and_requirement_subjects_root_the_tree() {
        let d = diagram("Requirement", "Reqs::Safety", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        let reqs: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::Requirement).map(|n| n.id.as_str()).collect();
        assert_eq!(reqs, vec!["s-reqs-safety", "s-reqs-safety-child"]);
        assert!(g.node("s-reqs-parent").is_none(), "the parent is outside the subject, so no derive edge either");
        assert!(g.edges.iter().all(|e| e.kind != EdgeKind::Derive));
        assert!(g.node("s-reqs-controller").is_some() && g.node("s-reqs-controllertest").is_some());

        let d = diagram("Requirement", "REQ-TK-002", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Requirement).count(), 1, "a Requirement subject by stable id: itself");
        assert_eq!(g.edges.iter().map(|e| e.kind).collect::<Vec<_>>(), vec![EdgeKind::Satisfy, EdgeKind::Verify]);
    }

    #[test]
    fn filters_apply_to_requirements_and_context_nodes_by_qname_id_or_short_name() {
        let d = diagram("Requirement", "Reqs", |fm| {
            fm.include = Some(vec!["REQ-TK-002".into(), "Reqs::Parent".into(), "ControllerTest".into(), "Ghost".into()]);
        });
        let (g, issues) = derive_it(&d);
        let ids: Vec<&str> = g.nodes.iter().filter(|n| n.kind != NodeKind::Compartment).map(|n| n.id.as_str()).collect();
        assert_eq!(ids, vec!["s-reqs-parent", "s-reqs-safety-child", "s-reqs-controllertest"]);
        assert_eq!(g.edges.iter().map(|e| e.kind).collect::<Vec<_>>(), vec![EdgeKind::Derive, EdgeKind::Verify]);
        assert_eq!(issues, vec![super::super::w417("`include` entry 'Ghost' names no member of the subject".into())]);

        let d = diagram("Requirement", "Reqs", |fm| fm.exclude = Some(vec!["Controller".into(), "Reqs::Safety".into()]));
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert!(g.node("s-reqs-controller").is_none() && g.node("s-reqs-safety").is_none());
        assert!(g.edges.iter().all(|e| !matches!(e.kind, EdgeKind::Satisfy | EdgeKind::Containment)));
    }

    #[test]
    fn wrong_subject_type_is_w418_and_empty() {
        let d = diagram("Requirement", "Sys::Engine", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(g.nodes.is_empty());
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "W418");
        assert!(issues[0].message.contains("PartDef"));
    }

    #[test]
    fn is_under_matches_the_subject_and_any_depth_below_it() {
        assert!(is_under("Reqs", "Reqs"));
        assert!(is_under("Reqs::A::B", "Reqs"));
        assert!(!is_under("Requirements", "Reqs"));
        assert!(is_under("Anything", ""));
    }
}

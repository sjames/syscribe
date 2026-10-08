//! The FeatureModel generator (`REQ-TRS-FMED-001`).
//!
//! Subject: a `FeatureDef` (its subtree), a `FeatureModel` sheet, or a
//! `Package`/`LibraryPackage`/`Namespace` holding feature definitions. One
//! `Feature` node per `FeatureDef` under the subject, carrying its
//! notation state (`mandatory`, the group kind of its children, the number of
//! children, its stable id) and its parameters and cardinality as compartment
//! lines; a `FeatureChild` edge from each feature to each of its children;
//! `Requires` and `Excludes` edges for the cross-tree constraints whose two
//! ends are both on the diagram (a mutual `excludes` is drawn once).
//! `include:`/`exclude:` apply by qualified name, `FEAT-*` id or short name;
//! a feature kept whose parent was excluded becomes a root.

use std::collections::{BTreeSet, HashSet};

use crate::element::{ElementType, RawElement};
use crate::feature_tree::{feature_tree, is_under, FeatureNode};
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, FeatureMark, Node, NodeKind};
use super::super::manifest::Issue;
use super::{short_name, w418, Filters};

fn is_package(t: &ElementType) -> bool {
    matches!(t, ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace)
}

fn keys(f: &FeatureNode) -> Vec<String> {
    let mut k = vec![f.qname.clone(), short_name(&f.qname).to_string()];
    if let Some(id) = &f.id {
        k.push(id.clone());
    }
    k
}

pub fn generate(
    graph: &mut DiagramGraph,
    subject: &RawElement,
    elements: &[RawElement],
    _resolver: &Resolver,
    filters: &Filters,
    issues: &mut Vec<Issue>,
) {
    let Some(st) = subject.frontmatter.element_type.as_ref() else { return };
    if !(is_package(st) || matches!(st, ElementType::FeatureDef | ElementType::FeatureModel)) {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — a FeatureModel diagram subject must be a FeatureDef, a FeatureModel sheet or a Package",
            subject.qualified_name,
            st.name()
        )));
        return;
    }
    let all = feature_tree(elements);
    let root = subject.qualified_name.as_str();
    let selected = select_under(&all, root, matches!(st, ElementType::FeatureDef));
    emit(graph, selected, filters, issues);
}

/// The features a subject covers: a `FeatureDef` subject is itself and its
/// descendants through the feature tree; anything else is every feature under
/// its qualified name (an empty name covers every feature).
fn select_under<'a>(all: &'a [FeatureNode], root: &str, is_feature: bool) -> Vec<&'a FeatureNode> {
    if is_feature {
        let mut inc: HashSet<&str> = HashSet::new();
        for f in all {
            if f.qname == root || f.parent.as_deref().is_some_and(|p| inc.contains(p)) {
                inc.insert(f.qname.as_str());
            }
        }
        all.iter().filter(|f| inc.contains(f.qname.as_str())).collect()
    } else {
        all.iter().filter(|f| is_under(&f.qname, root)).collect()
    }
}

/// The feature diagram of the model without a `Diagram` element behind it
/// (`/features`, `diagram export` of the whole feature model): the subtree of
/// the `FeatureDef` named `root`, else every feature under `root` read as a
/// qualified-name prefix (a package), else, with no `root`, every feature.
pub fn feature_diagram(elements: &[RawElement], root: Option<&str>) -> DiagramGraph {
    let all = feature_tree(elements);
    let is_feature = root.is_some_and(|r| all.iter().any(|f| f.qname == r));
    let selected = select_under(&all, root.unwrap_or(""), is_feature);
    let mut graph = DiagramGraph::empty(super::super::ir::DiagramKind::FeatureModel, "", "Feature model", root);
    graph.derived = true;
    emit(&mut graph, selected, &Filters::default(), &mut Vec::new());
    graph
}

fn emit(graph: &mut DiagramGraph, mut selected: Vec<&FeatureNode>, filters: &Filters, issues: &mut Vec<Issue>) {
    // include:/exclude: over feature names, with W417 for entries naming none.
    let candidates: Vec<Vec<String>> = selected.iter().map(|f| keys(f)).collect();
    super::requirement::unmatched_key_issues(filters, &candidates, issues);
    selected.retain(|f| super::requirement::keeps_keys(filters, &keys(f)));

    let on: HashSet<&str> = selected.iter().map(|f| f.qname.as_str()).collect();
    for f in &selected {
        let child_count = f.children.len();
        let id = derived_shape_id(&f.qname);
        let mut lines: Vec<String> = f.parameters.clone();
        if let Some(c) = &f.cardinality {
            lines.push(format!("cardinality = {c}"));
        }
        graph.nodes.push(Node {
            id: id.clone(),
            element_ref: f.qname.clone(),
            resolved: true,
            element_type: Some("FeatureDef".to_string()),
            kind: NodeKind::Feature,
            label: f.name.clone(),
            stereotype: None,
            parent: None,
            direction: None,
            side: None,
            lines,
            is_abstract: f.is_abstract,
            pin: None,
            banners: Vec::new(),
            feature: Some(FeatureMark {
                mandatory: f.mandatory,
                group: f.group_kind.as_str().to_string(),
                child_count,
                id: f.id.clone(),
                requires: f.requires.clone(),
                excludes: f.excludes.clone(),
            }),
        });
    }

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut push = |graph: &mut DiagramGraph, kind: EdgeKind, src: &str, tgt: &str| {
        let (s, t) = (derived_shape_id(src), derived_shape_id(tgt));
        let id = format!("e-{}-{s}-{t}", kind.as_str());
        if seen.insert(id.clone()) {
            graph.edges.push(Edge { id, element_ref: Some(src.to_string()), source: s, target: t, kind, label: None, waypoints: None });
        }
    };
    for f in &selected {
        if let Some(p) = f.parent.as_deref().filter(|p| on.contains(p)) {
            push(graph, EdgeKind::FeatureChild, p, &f.qname);
        }
        for r in f.requires.iter().filter(|r| on.contains(r.as_str())) {
            push(graph, EdgeKind::Requires, &f.qname, r);
        }
        for x in f.excludes.iter().filter(|x| on.contains(x.as_str())) {
            // `A excludes B` and `B excludes A` are one constraint: draw it once.
            let (a, b) = if f.qname.as_str() <= x.as_str() { (f.qname.as_str(), x.as_str()) } else { (x.as_str(), f.qname.as_str()) };
            push(graph, EdgeKind::Excludes, a, b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vis::ir::DiagramKind;

    fn fd(qname: &str, id: &str, fm: &str) -> RawElement {
        let text = format!("---\ntype: FeatureDef\nid: {id}\nname: {}\n{fm}---\n", qname.rsplit("::").next().unwrap());
        let (frontmatter, _) = crate::frontmatter::split_frontmatter(&text);
        let mut e = RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("{qname}.md"),
            frontmatter: serde_yaml::from_str(frontmatter.unwrap()).unwrap(),
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: Vec::new(),
            locale_docs: Default::default(),
            about_notes: Default::default(),
        };
        e.frontmatter.shrink();
        e
    }

    fn pkg(qname: &str) -> RawElement {
        let mut e = fd(qname, "X", "");
        e.frontmatter.element_type = Some(ElementType::Package);
        e.frontmatter.id = None;
        e
    }

    fn model() -> Vec<RawElement> {
        vec![
            pkg("Features"),
            fd("Features::Car", "FEAT-CAR", "mandatory: true\ngroupKind: optional\n"),
            fd("Features::Car::Engine", "FEAT-ENGINE", "mandatory: true\ngroupKind: alternative\n"),
            fd("Features::Car::Engine::Petrol", "FEAT-PETROL", "excludes: [FEAT-ELECTRIC]\n"),
            fd("Features::Car::Engine::Electric", "FEAT-ELECTRIC", "requires: [Features::Car::Charger]\n"),
            fd("Features::Car::Charger", "FEAT-CHARGER", "parameters:\n  - name: power\n    type: Real\n    unit: kW\n"),
        ]
    }

    fn graph_for(subject: &str, elements: &[RawElement], filters: &Filters) -> (DiagramGraph, Vec<Issue>) {
        let resolver = Resolver::new(elements);
        let subj = elements.iter().find(|e| e.qualified_name == subject).unwrap();
        let mut g = DiagramGraph::empty(DiagramKind::FeatureModel, "D", "D", Some(subject));
        let mut issues = Vec::new();
        generate(&mut g, subj, elements, &resolver, filters, &mut issues);
        (g, issues)
    }

    #[test]
    fn a_package_subject_draws_every_feature_with_its_notation() {
        let els = model();
        let (g, issues) = graph_for("Features", &els, &Filters::default());
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.nodes.len(), 5);
        let engine = g.nodes.iter().find(|n| n.element_ref == "Features::Car::Engine").unwrap();
        let mark = engine.feature.as_ref().unwrap();
        assert!(mark.mandatory);
        assert_eq!(mark.group, "alternative");
        assert_eq!(mark.child_count, 2);
        assert_eq!(mark.id.as_deref(), Some("FEAT-ENGINE"));
        let petrol = g.nodes.iter().find(|n| n.label == "Petrol").unwrap();
        assert!(!petrol.feature.as_ref().unwrap().mandatory);
        let charger = g.nodes.iter().find(|n| n.label == "Charger").unwrap();
        assert_eq!(charger.lines, vec!["power: Real [kW]".to_string()]);
    }

    #[test]
    fn tree_and_constraint_edges_are_drawn_once_with_ids_resolved() {
        let els = model();
        let (g, _) = graph_for("Features", &els, &Filters::default());
        let count = |k: EdgeKind| g.edges.iter().filter(|e| e.kind == k).count();
        assert_eq!(count(EdgeKind::FeatureChild), 4, "Car>Engine, Car>Charger, Engine>Petrol, Engine>Electric");
        assert_eq!(count(EdgeKind::Requires), 1, "Electric requires Charger");
        assert_eq!(count(EdgeKind::Excludes), 1, "Petrol excludes Electric, by its FEAT id");
        let ex = g.edges.iter().find(|e| e.kind == EdgeKind::Excludes).unwrap();
        assert!(g.nodes.iter().any(|n| n.id == ex.source) && g.nodes.iter().any(|n| n.id == ex.target));
    }

    #[test]
    fn a_feature_subject_draws_its_subtree_and_only_constraints_between_drawn_features() {
        let els = model();
        let (g, _) = graph_for("Features::Car::Engine", &els, &Filters::default());
        let labels: Vec<&str> = g.nodes.iter().map(|n| n.label.as_str()).collect();
        assert_eq!(labels, vec!["Engine", "Electric", "Petrol"], "children sort by qualified name");
        assert!(g.edges.iter().all(|e| e.kind != EdgeKind::Requires), "Charger is not on the diagram");
        assert_eq!(g.edges.iter().filter(|e| e.kind == EdgeKind::Excludes).count(), 1);
    }

    #[test]
    fn exclude_removes_a_feature_and_its_edges_and_unknown_entries_warn() {
        let els = model();
        let f = Filters { include: vec![], exclude: vec!["FEAT-ELECTRIC".into(), "Nope".into()] };
        let (g, issues) = graph_for("Features", &els, &f);
        assert!(g.nodes.iter().all(|n| n.label != "Electric"));
        assert!(g.edges.iter().all(|e| e.kind != EdgeKind::Excludes && e.kind != EdgeKind::Requires));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "W417");
    }

    #[test]
    fn a_subject_of_the_wrong_type_is_w418() {
        let mut els = model();
        let mut other = fd("Parts::Engine", "X", "");
        other.frontmatter.element_type = Some(ElementType::PartDef);
        els.push(other);
        let (g, issues) = graph_for("Parts::Engine", &els, &Filters::default());
        assert!(g.nodes.is_empty());
        assert_eq!(issues[0].code, "W418");
    }
}

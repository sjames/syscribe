//! The BDD generator (`REQ-TRS-VIS-004`).
//!
//! Subject: a `Package`/`LibraryPackage`/`Namespace`, `PartDef` or `ItemDef`.
//! Produces one `Block` per definition that is a direct member of the subject
//! package (or the subject definition itself plus its direct
//! sub-definitions), a `Compartment` child listing attributes and ports, and
//! `Inheritance` (from `supertype:`), `Composition` (from part usages typed by
//! a block on the diagram) and `Association` (from a `ConnectionDef` whose
//! ends are blocks on the diagram) edges. An edge is emitted only when both
//! ends are nodes of the diagram, after `include:`/`exclude:`.

use crate::element::{ElementType, RawElement};
use crate::members::direct_members;
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, Node, NodeKind};
use super::super::manifest::Issue;
use super::{block_node, display_name, feature_line, features_of, map_str, short_name, w418, yaml_strings, Feature, FeatureRole, Filters};

/// The definition kinds a BDD shows.
fn is_bdd_block(t: &ElementType) -> bool {
    matches!(
        t,
        ElementType::PartDef | ElementType::ItemDef | ElementType::PortDef | ElementType::InterfaceDef | ElementType::ConnectionDef
    )
}

fn is_package(t: &ElementType) -> bool {
    matches!(t, ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace)
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
    let candidates: Vec<&RawElement> = if is_package(st) {
        direct_members(elements, &subject.qualified_name)
            .into_iter()
            .filter(|e| e.frontmatter.element_type.as_ref().map(is_bdd_block).unwrap_or(false))
            .collect()
    } else if matches!(st, ElementType::PartDef | ElementType::ItemDef) {
        let mut v = vec![subject];
        v.extend(
            direct_members(elements, &subject.qualified_name)
                .into_iter()
                .filter(|e| e.frontmatter.element_type.as_ref().map(is_bdd_block).unwrap_or(false)),
        );
        v
    } else {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — a BDD subject must be a Package, PartDef or ItemDef",
            subject.qualified_name,
            st.name()
        )));
        return;
    };

    let names: Vec<(String, String)> = candidates
        .iter()
        .map(|e| (e.qualified_name.clone(), short_name(&e.qualified_name).to_string()))
        .collect();
    filters.unmatched_issues(&names, issues);
    let blocks: Vec<&RawElement> = candidates
        .into_iter()
        .filter(|e| filters.keeps(&e.qualified_name, short_name(&e.qualified_name)))
        .collect();

    // ── nodes ───────────────────────────────────────────────────────────
    for e in &blocks {
        let id = derived_shape_id(&e.qualified_name);
        let mut node = block_node(id.clone(), e, NodeKind::Block, None, display_name(e), elements, resolver);
        node.lines = Vec::new();
        graph.nodes.push(node);
        let lines: Vec<String> = features_of(e, elements, resolver)
            .iter()
            .filter(|f| matches!(f.role, FeatureRole::Attribute | FeatureRole::Port))
            .map(feature_line)
            .collect();
        if !lines.is_empty() {
            graph.nodes.push(Node {
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
                mark: None,
            });
        }
    }

    let node_id_of = |qname: &str| -> Option<String> {
        let id = derived_shape_id(qname);
        graph.node(&id).map(|_| id)
    };
    let resolve_block = |r: &str| -> Option<String> { resolver.resolve_ref(elements, r).and_then(|t| node_id_of(&t.qualified_name)) };

    // ── edges ───────────────────────────────────────────────────────────
    let mut edges: Vec<Edge> = Vec::new();
    for e in &blocks {
        let src = derived_shape_id(&e.qualified_name);
        // Inheritance: sub → super.
        for sup in yaml_strings(e.frontmatter.supertype.as_ref()) {
            if let Some(tgt) = resolve_block(&sup) {
                edges.push(Edge {
                    id: format!("e-inheritance-{src}-{tgt}"),
                    element_ref: Some(e.qualified_name.clone()),
                    source: src.clone(),
                    target: tgt,
                    kind: EdgeKind::Inheritance,
                    label: None,
                    waypoints: None,
                });
            }
        }
        // Composition: owner → the block typing each part usage (inline
        // features, and child `Part` elements).
        let mut parts: Vec<(String, Option<String>, Option<String>)> = features_of(e, elements, resolver)
            .into_iter()
            .filter(|f| f.role == FeatureRole::Part)
            .map(|f: Feature| (f.name, f.typed_by, f.multiplicity))
            .collect();
        for child in direct_members(elements, &e.qualified_name) {
            if matches!(child.frontmatter.element_type, Some(ElementType::Part) | Some(ElementType::Item)) {
                parts.push((
                    short_name(&child.qualified_name).to_string(),
                    yaml_strings(child.frontmatter.typed_by.as_ref()).into_iter().next(),
                    child.frontmatter.multiplicity.clone(),
                ));
            }
        }
        for (name, typed_by, mult) in parts {
            let Some(tb) = typed_by else { continue };
            if let Some(tgt) = resolve_block(&tb) {
                let label = match mult.as_deref() {
                    Some(m) if m != "1" => format!("{name} [{m}]"),
                    _ => name.clone(),
                };
                edges.push(Edge {
                    id: format!("e-composition-{src}-{tgt}-{}", name.to_ascii_lowercase()),
                    element_ref: Some(format!("{}::{name}", e.qualified_name)),
                    source: src.clone(),
                    target: tgt,
                    kind: EdgeKind::Composition,
                    label: Some(label),
                    waypoints: None,
                });
            }
        }
        // Association: a ConnectionDef on the diagram whose two ends type blocks on it.
        if matches!(e.frontmatter.element_type, Some(ElementType::ConnectionDef)) {
            let ends: Vec<Option<String>> = e
                .frontmatter
                .ends
                .as_ref()
                .map(|v| {
                    v.iter()
                        .filter_map(|x| x.as_mapping())
                        .map(|m| map_str(m, "typedBy").and_then(resolve_block))
                        .collect()
                })
                .unwrap_or_default();
            if let [Some(a), Some(b), ..] = ends.as_slice() {
                edges.push(Edge {
                    id: format!("e-association-{a}-{b}-{}", short_name(&e.qualified_name).to_ascii_lowercase()),
                    element_ref: Some(e.qualified_name.clone()),
                    source: a.clone(),
                    target: b.clone(),
                    kind: EdgeKind::Association,
                    label: Some(display_name(e)),
                    waypoints: None,
                });
            }
        }
    }
    graph.edges.extend(edges);
}

#[cfg(test)]
mod tests {
    use super::super::testkit::*;
    use super::*;

    #[test]
    fn package_subject_lists_definitions_with_compartments_and_edges() {
        let d = diagram("BDD", "Sys", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        let blocks: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::Block).map(|n| n.element_ref.as_str()).collect();
        assert_eq!(blocks, vec!["Sys::Base", "Sys::Engine", "Sys::Motor", "Sys::PowerLink", "Sys::PowerPort", "Sys::PowerSystem", "Sys::Sensor"]);
        assert!(!blocks.contains(&"Sys::Startup"), "an ActionDef is never on a BDD");
        let base = g.node("s-sys-base").unwrap();
        assert!(base.is_abstract);
        assert_eq!(base.stereotype.as_deref(), Some("part def"));
        assert_eq!(g.node("s-sys-powerlink").unwrap().stereotype.as_deref(), Some("connection def"));
        // Compartment lines for Engine: the attribute and the port.
        let comp = g.node("s-sys-engine-compartment").unwrap();
        assert_eq!(comp.kind, NodeKind::Compartment);
        assert_eq!(comp.parent.as_deref(), Some("s-sys-engine"));
        assert_eq!(comp.lines, vec!["mass : Real [kg]", "port powerOut : PowerPort (out)"]);
        // PowerSystem's compartment lists only its port; its parts are composition edges.
        assert_eq!(g.node("s-sys-powersystem-compartment").unwrap().lines, vec!["port mainOut : PowerPort (out)"]);
        // Edges.
        let kinds: Vec<(EdgeKind, &str, &str, Option<&str>)> =
            g.edges.iter().map(|e| (e.kind, e.source.as_str(), e.target.as_str(), e.label.as_deref())).collect();
        assert!(kinds.contains(&(EdgeKind::Inheritance, "s-sys-engine", "s-sys-base", None)));
        assert!(kinds.contains(&(EdgeKind::Inheritance, "s-sys-motor", "s-sys-base", None)));
        assert!(kinds.contains(&(EdgeKind::Composition, "s-sys-powersystem", "s-sys-engine", Some("engine"))));
        assert!(kinds.contains(&(EdgeKind::Composition, "s-sys-powersystem", "s-sys-motor", Some("motor [2]"))));
        assert!(kinds.contains(&(EdgeKind::Composition, "s-sys-powersystem", "s-sys-motor", Some("aux"))), "child Part element");
        assert!(kinds.contains(&(EdgeKind::Association, "s-sys-engine", "s-sys-motor", Some("PowerLink"))));
        assert_eq!(g.edges.len(), 6, "{kinds:?}");
        // Deterministic ids.
        assert!(g.edges.iter().any(|e| e.id == "e-composition-s-sys-powersystem-s-sys-motor-motor"));
    }

    #[test]
    fn definition_subject_includes_itself_and_edges_only_between_diagram_nodes() {
        let d = diagram("BDD", "Sys::PowerSystem", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Block).count(), 1);
        assert!(g.edges.is_empty(), "Engine/Motor are not on the diagram, so no composition edges: {:?}", g.edges);
    }

    #[test]
    fn include_and_exclude_filter_members_and_flag_unknown_entries() {
        let d = diagram("BDD", "Sys", |fm| {
            fm.include = Some(vec!["Engine".into(), "Sys::Base".into(), "Ghost".into()]);
        });
        let (g, issues) = derive_it(&d);
        let blocks: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::Block).map(|n| n.element_ref.as_str()).collect();
        assert_eq!(blocks, vec!["Sys::Base", "Sys::Engine"]);
        assert_eq!(g.edges.len(), 1, "only Engine → Base inheritance survives");
        assert_eq!(issues, vec![super::super::w417("`include` entry 'Ghost' names no member of the subject".into())]);

        let d = diagram("BDD", "Sys", |fm| fm.exclude = Some(vec!["Sys::Base".into()]));
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty());
        assert!(g.node("s-sys-base").is_none());
        assert!(g.edges.iter().all(|e| e.kind != EdgeKind::Inheritance));
    }

    #[test]
    fn wrong_subject_type_is_w418_and_empty() {
        let d = diagram("BDD", "Sys::Startup", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(g.nodes.is_empty());
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "W418");
        assert!(issues[0].message.contains("ActionDef"));
    }
}

//! The IBD generator (`REQ-TRS-VIS-005`).
//!
//! Subject: a `PartDef` or `Part`. Produces a `Boundary` for the subject with
//! its own ports on the boundary, a `Block` child per owned part usage
//! (inline `features:` typed by a definition, or child `Part`/`Item`
//! elements) carrying that usage's ports (its own port features plus the
//! ports its definition declares), and `Connection`/`Flow`/`Binding`/
//! `Succession` edges from the subject's `connections:`,
//! `flowConnections:`, `bindingConnections:` and `successionConnections:`,
//! resolving dotted feature chains (`engine.powerOut`) exactly as the
//! graph builder does. The generator never invents a port: a chain whose
//! endpoint is not a node of the diagram produces no edge.

use std::collections::HashMap;

use crate::connections::parse_entry;
use crate::element::{ElementType, RawElement};
use crate::members::direct_members;
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, NodeKind};
use super::super::manifest::Issue;
use super::{block_node, display_name, features_of, map_str, port_node, short_name, w418, yaml_strings, Feature, FeatureRole, Filters};

/// One owned part usage of the subject.
struct Usage<'a> {
    name: String,
    /// The usage element when it is a child `Part` file; `None` for an inline feature.
    element: Option<&'a RawElement>,
    /// The definition the usage is typed by, when it resolves.
    definition: Option<&'a RawElement>,
    /// Port features declared on the usage itself (inline features of a child
    /// `Part` element).
    own_ports: Vec<Feature>,
    multiplicity: Option<String>,
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
    if !matches!(st, ElementType::PartDef | ElementType::Part | ElementType::ItemDef | ElementType::Item) {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — an IBD subject must be a PartDef or Part",
            subject.qualified_name,
            st.name()
        )));
        return;
    }
    let sq = subject.qualified_name.as_str();

    // ── the boundary and its ports ──────────────────────────────────────
    let boundary_id = derived_shape_id(sq);
    graph.nodes.push(block_node(boundary_id.clone(), subject, NodeKind::Boundary, None, display_name(subject), elements, resolver));
    let subject_features = features_of(subject, elements, resolver);
    // A `Part` subject also carries the ports of its definition.
    let subject_def_ports: Vec<Feature> = if matches!(st, ElementType::Part | ElementType::Item) {
        yaml_strings(subject.frontmatter.typed_by.as_ref())
            .first()
            .and_then(|t| resolver.resolve_ref(elements, t))
            .map(|d| features_of(d, elements, resolver).into_iter().filter(|f| f.role == FeatureRole::Port).collect())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    for f in subject_features.iter().chain(subject_def_ports.iter()).filter(|f| f.role == FeatureRole::Port) {
        let qn = format!("{sq}::{}", f.name);
        let id = derived_shape_id(&qn);
        if graph.node(&id).is_none() {
            graph.nodes.push(port_node(id, sq, f, &boundary_id));
        }
    }

    // ── owned part usages ───────────────────────────────────────────────
    let mut usages: Vec<Usage> = subject_features
        .iter()
        .filter(|f| f.role == FeatureRole::Part)
        .map(|f| Usage {
            name: f.name.clone(),
            element: None,
            definition: f.typed_by.as_deref().and_then(|t| resolver.resolve_ref(elements, t)),
            own_ports: Vec::new(),
            multiplicity: f.multiplicity.clone(),
        })
        .collect();
    for child in direct_members(elements, sq) {
        if !matches!(child.frontmatter.element_type, Some(ElementType::Part) | Some(ElementType::Item)) {
            continue;
        }
        let name = short_name(&child.qualified_name).to_string();
        if usages.iter().any(|u| u.name == name) {
            continue;
        }
        usages.push(Usage {
            name,
            element: Some(child),
            definition: yaml_strings(child.frontmatter.typed_by.as_ref()).first().and_then(|t| resolver.resolve_ref(elements, t)),
            own_ports: features_of(child, elements, resolver).into_iter().filter(|f| f.role == FeatureRole::Port).collect(),
            multiplicity: child.frontmatter.multiplicity.clone(),
        });
    }
    let candidates: Vec<(String, String)> = usages.iter().map(|u| (format!("{sq}::{}", u.name), u.name.clone())).collect();
    filters.unmatched_issues(&candidates, issues);
    usages.retain(|u| filters.keeps(&format!("{sq}::{}", u.name), &u.name));

    for u in &usages {
        let qn = format!("{sq}::{}", u.name);
        let id = derived_shape_id(&qn);
        let type_label = u.definition.map(display_name);
        let mut label = match &type_label {
            Some(t) => format!("{} : {t}", u.name),
            None => u.name.clone(),
        };
        if let Some(m) = u.multiplicity.as_deref() {
            if m != "1" {
                label.push_str(&format!(" [{m}]"));
            }
        }
        let mut node = match u.element {
            Some(e) => block_node(id.clone(), e, NodeKind::Block, Some(boundary_id.clone()), label, elements, resolver),
            None => {
                let mut n = block_node(id.clone(), subject, NodeKind::Block, Some(boundary_id.clone()), label, elements, resolver);
                n.element_ref = qn.clone();
                n.is_abstract = false;
                // An inline `features:` usage has no `metadata:` of its own.
                n.banners = Vec::new();
                n
            }
        };
        node.element_type = Some("Part".to_string());
        node.stereotype = Some("part".to_string());
        graph.nodes.push(node);

        // Ports: the usage's own, then its definition's (own wins on a name clash).
        let def_ports: Vec<Feature> = u
            .definition
            .map(|d| features_of(d, elements, resolver).into_iter().filter(|f| f.role == FeatureRole::Port).collect())
            .unwrap_or_default();
        for f in u.own_ports.iter().chain(def_ports.iter()) {
            let pq = format!("{qn}::{}", f.name);
            let pid = derived_shape_id(&pq);
            if graph.node(&pid).is_none() {
                graph.nodes.push(port_node(pid, &qn, f, &id));
            }
        }
    }

    // ── edges from the subject's connection lists ───────────────────────
    // A chain `a.b` names port `b` of usage `a`; a single segment names a
    // usage or a boundary port. Resolution is by the deterministic ids.
    let chain_to_id = |chain: &str| -> Option<String> {
        let qn = format!("{sq}::{}", chain.trim().replace('.', "::"));
        let id = derived_shape_id(&qn);
        graph.node(&id).map(|_| id)
    };
    let mut counters: HashMap<String, usize> = HashMap::new();
    let mut edges = Vec::new();
    let lists: [(Option<&Vec<serde_yaml::Value>>, EdgeKind); 4] = [
        (subject.frontmatter.connections.as_ref(), EdgeKind::Connection),
        (subject.frontmatter.flow_connections.as_ref(), EdgeKind::Flow),
        (subject.frontmatter.binding_connections.as_ref(), EdgeKind::Binding),
        (subject.frontmatter.succession_connections.as_ref(), EdgeKind::Succession),
    ];
    for (list, kind) in lists {
        let Some(list) = list else { continue };
        for entry in list {
            let (ends, name): (Vec<String>, Option<String>) = match kind {
                EdgeKind::Succession => {
                    let m = entry.as_mapping();
                    let after = m.and_then(|m| map_str(m, "after"));
                    let before = m.and_then(|m| map_str(m, "before"));
                    match (after, before) {
                        (Some(a), Some(b)) => (vec![a.to_string(), b.to_string()], m.and_then(|m| map_str(m, "name")).map(str::to_string)),
                        _ => continue,
                    }
                }
                _ => match parse_entry(entry) {
                    Some(p) => {
                        let name = entry
                            .as_mapping()
                            .and_then(|m| map_str(m, "name").map(str::to_string))
                            .or_else(|| p.typed_by.as_deref().map(|t| short_name(t).to_string()));
                        (p.endpoints.into_iter().map(|e| e.chain).collect(), name)
                    }
                    None => continue,
                },
            };
            if ends.len() < 2 {
                continue;
            }
            let (Some(src), Some(tgt)) = (chain_to_id(&ends[0]), chain_to_id(&ends[1])) else { continue };
            let base = format!("e-{}-{src}-{tgt}", kind.as_str());
            let n = counters.entry(base.clone()).or_insert(0);
            *n += 1;
            let id = if *n == 1 { base } else { format!("{base}-{n}") };
            edges.push(Edge {
                id,
                element_ref: Some(sq.to_string()),
                source: src,
                target: tgt,
                kind,
                label: name,
                waypoints: None,
            });
        }
    }
    graph.edges.extend(edges);
}

#[cfg(test)]
mod tests {
    use super::super::testkit::*;
    use super::*;
    use crate::vis::ir::PortDirection;

    #[test]
    fn partdef_subject_yields_boundary_parts_ports_and_edges() {
        let d = diagram("IBD", "Sys::PowerSystem", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        let b = g.node("s-sys-powersystem").unwrap();
        assert_eq!(b.kind, NodeKind::Boundary);
        assert_eq!(b.stereotype.as_deref(), Some("part def"));
        // Boundary port.
        let main_out = g.node("s-sys-powersystem-mainout").unwrap();
        assert_eq!(main_out.kind, NodeKind::Port);
        assert_eq!(main_out.parent.as_deref(), Some("s-sys-powersystem"));
        assert_eq!(main_out.direction, Some(PortDirection::Out));
        // Owned parts: two inline features plus the child Part element.
        let parts: Vec<(&str, &str)> = g
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Block)
            .map(|n| (n.id.as_str(), n.label.as_str()))
            .collect();
        assert_eq!(
            parts,
            vec![
                ("s-sys-powersystem-engine", "engine : Engine"),
                ("s-sys-powersystem-motor", "motor : Motor [2]"),
                ("s-sys-powersystem-aux", "aux : Motor"),
            ]
        );
        assert!(g.nodes.iter().filter(|n| n.kind == NodeKind::Block).all(|n| n.parent.as_deref() == Some("s-sys-powersystem")));
        // Ports come from the typing definition, with direction.
        let p = g.node("s-sys-powersystem-engine-powerout").unwrap();
        assert_eq!(p.parent.as_deref(), Some("s-sys-powersystem-engine"));
        assert_eq!(p.direction, Some(PortDirection::Out));
        assert_eq!(p.element_ref, "Sys::PowerSystem::engine::powerOut");
        assert_eq!(g.node("s-sys-powersystem-aux-powerin").unwrap().direction, Some(PortDirection::In));
        // Edges: the connection port→port and the binding to the boundary port.
        let edges: Vec<(EdgeKind, &str, &str, Option<&str>)> =
            g.edges.iter().map(|e| (e.kind, e.source.as_str(), e.target.as_str(), e.label.as_deref())).collect();
        assert_eq!(
            edges,
            vec![
                (EdgeKind::Connection, "s-sys-powersystem-engine-powerout", "s-sys-powersystem-motor-powerin", Some("PowerLink")),
                (EdgeKind::Binding, "s-sys-powersystem-motor-powerin", "s-sys-powersystem-mainout", None),
            ]
        );
        assert_eq!(g.edges[0].id, "e-connection-s-sys-powersystem-engine-powerout-s-sys-powersystem-motor-powerin");
        assert_eq!(g.layout_hints.hierarchical, true);
    }

    #[test]
    fn part_usage_subject_carries_its_definitions_ports() {
        let d = diagram("IBD", "Sys::PowerSystem::aux", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.node("s-sys-powersystem-aux").unwrap().kind, NodeKind::Boundary);
        let p = g.node("s-sys-powersystem-aux-powerin").unwrap();
        assert_eq!(p.kind, NodeKind::Port);
        assert_eq!(p.direction, Some(PortDirection::In));
    }

    #[test]
    fn exclude_drops_a_usage_and_its_edges_and_the_generator_never_invents_a_port() {
        let d = diagram("IBD", "Sys::PowerSystem", |fm| fm.exclude = Some(vec!["motor".into()]));
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert!(g.node("s-sys-powersystem-motor").is_none());
        assert!(g.edges.is_empty(), "both edges touched motor.powerIn: {:?}", g.edges);
    }

    #[test]
    fn include_naming_no_usage_is_w417() {
        let d = diagram("IBD", "Sys::PowerSystem", |fm| fm.include = Some(vec!["engine".into(), "turbine".into()]));
        let (g, issues) = derive_it(&d);
        assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Block).count(), 1);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "W417");
        assert!(issues[0].message.contains("'turbine'"));
    }

    #[test]
    fn wrong_subject_type_is_w418() {
        let d = diagram("IBD", "Sys", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(g.nodes.is_empty());
        assert_eq!(issues[0].code, "W418");
        assert!(issues[0].message.contains("Package"));
    }

    #[test]
    fn a_chain_to_an_unknown_port_produces_no_edge() {
        let mut d = diagram("IBD", "Sys::PowerSystem", |_| {});
        // Subject override with a dangling chain is awkward through the shared
        // model, so derive against a subject copy whose connections point nowhere.
        let mut elements = model();
        let ps = elements.iter_mut().find(|e| e.qualified_name == "Sys::PowerSystem").unwrap();
        ps.frontmatter.connections = Some(yaml_list("- {from: engine.powerOut, to: ghost.powerIn}\n"));
        ps.frontmatter.binding_connections = None;
        elements.push(d.clone());
        let resolver = Resolver::new(&elements);
        d.frontmatter.subject = Some("Sys::PowerSystem".into());
        let (g, issues) = super::super::derive(&d, crate::vis::ir::DiagramKind::Ibd, &elements, &resolver);
        assert!(issues.is_empty());
        assert!(g.edges.is_empty());
    }
}

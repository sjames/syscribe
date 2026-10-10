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
//!
//! Composition is followed beyond the subject's own members (GH #243, `REQ-TRS-BDDX-001`): the
//! blocks typing the parts of a diagram block are pulled in as **external** blocks (mark status
//! `external`), `depth:` levels deep (default 1; 0 keeps only the subject's members). A package
//! subject's `include:` also accepts a name qualified relative to the subject.

use crate::element::{ElementType, RawElement};
use crate::members::direct_members;
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, Node, NodeKind, NodeMark};
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

/// The part usages of a block: inline `Part` features and child `Part`/`Item` elements, as
/// `(name, typedBy, multiplicity)`.
fn parts_of(e: &RawElement, elements: &[RawElement], resolver: &Resolver) -> Vec<(String, Option<String>, Option<String>)> {
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
    parts
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
    // Include entries naming a definition by a relative/qualified path are rewritten to its
    // qualified name, so the member filter and `W417` see them as matched.
    let mut normalised = filters.clone();
    let filters = &mut normalised;
    let candidates: Vec<&RawElement> = if is_package(st) {
        let mut v: Vec<&RawElement> = direct_members(elements, &subject.qualified_name)
            .into_iter()
            .filter(|e| e.frontmatter.element_type.as_ref().map(is_bdd_block).unwrap_or(false))
            .collect();
        // An `include:` entry naming a nested or distant definition (relative to the subject, or a
        // full qualified name / id) adds that definition.
        for entry in filters.include.iter_mut() {
            if v.iter().any(|e| &e.qualified_name == entry || short_name(&e.qualified_name) == entry) {
                continue;
            }
            // A qualified name relative to the subject, a full qualified name or a stable id — never
            // a bare display name (that would match an unrelated definition elsewhere).
            let rel = format!("{}::{entry}", subject.qualified_name);
            let by_exact = |q: &str| elements.iter().find(|e| e.qualified_name == q || e.frontmatter.id.as_deref() == Some(q));
            let hit = by_exact(&rel).or_else(|| by_exact(entry));
            if let Some(t) = hit.filter(|t| t.frontmatter.element_type.as_ref().map(is_bdd_block).unwrap_or(false)) {
                if !v.iter().any(|e| e.qualified_name == t.qualified_name) {
                    v.push(t);
                }
                *entry = t.qualified_name.clone();
            }
        }
        v
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
    let filters = &*filters;
    let cand_names: std::collections::HashSet<String> = candidates.iter().map(|e| e.qualified_name.clone()).collect();
    let blocks: Vec<&RawElement> = candidates
        .into_iter()
        .filter(|e| filters.keeps(&e.qualified_name, short_name(&e.qualified_name)))
        .collect();

    // ── external blocks: composition followed beyond the subject's members ──
    let depth = filters.depth.unwrap_or(1);
    let mut have: std::collections::HashSet<String> = blocks.iter().map(|e| e.qualified_name.clone()).collect();
    let mut externals: Vec<&RawElement> = Vec::new();
    let mut frontier: Vec<&RawElement> = blocks.clone();
    // Names an `exclude:` entry may legitimately name although they are not members: the
    // external blocks it keeps out (so `W417` does not fire for them).
    let mut excluded_externals: Vec<(String, String)> = Vec::new();
    for _ in 0..depth {
        let mut next: Vec<&RawElement> = Vec::new();
        for e in &frontier {
            for (_, typed_by, _) in parts_of(e, elements, resolver) {
                let Some(tb) = typed_by else { continue };
                let Some(t) = resolver.resolve_ref(elements, &tb) else { continue };
                let is_block = t.frontmatter.element_type.as_ref().map(is_bdd_block).unwrap_or(false);
                // A member the `include:`/`exclude:` lists left out stays out; it is not "external".
                if !is_block || cand_names.contains(&t.qualified_name) {
                    continue;
                }
                let excluded = filters.exclude.iter().any(|x| x == &t.qualified_name || x == short_name(&t.qualified_name));
                if excluded {
                    excluded_externals.push((t.qualified_name.clone(), short_name(&t.qualified_name).to_string()));
                    continue;
                }
                if have.insert(t.qualified_name.clone()) {
                    externals.push(t);
                    next.push(t);
                }
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    let mut known = names.clone();
    known.extend(excluded_externals);
    filters.unmatched_issues(&known, issues);
    let all_blocks: Vec<&RawElement> = blocks.iter().chain(externals.iter()).copied().collect();

    // ── nodes ───────────────────────────────────────────────────────────
    for e in &all_blocks {
        let id = derived_shape_id(&e.qualified_name);
        let mut node = block_node(id.clone(), e, NodeKind::Block, None, display_name(e), elements, resolver);
        node.lines = Vec::new();
        if externals.iter().any(|x| x.qualified_name == e.qualified_name) {
            node.mark = Some(NodeMark { status: Some("external".to_string()), ..Default::default() });
        }
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
    for e in &all_blocks {
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
        for (name, typed_by, mult) in parts_of(e, elements, resolver) {
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
        // Engine/Motor are composed by PowerSystem: pulled in as external blocks (GH #243).
        assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Block).count(), 3);
        assert_eq!(g.edges.iter().filter(|e| e.kind == EdgeKind::Composition).count(), 3);
        // `depth: 0` keeps only the subject: no external blocks, so no composition edges.
        let d = diagram("BDD", "Sys::PowerSystem", |fm| fm.depth = Some(0));
        let (g, _) = derive_it(&d);
        assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Block).count(), 1);
        assert!(g.edges.is_empty(), "{:?}", g.edges);
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

    fn block_refs(g: &DiagramGraph) -> Vec<&str> {
        g.nodes.iter().filter(|n| n.kind == NodeKind::Block).map(|n| n.element_ref.as_str()).collect()
    }

    #[test]
    fn a_package_subject_pulls_in_cross_package_composition_as_external_blocks() {
        let d = diagram("BDD", "Xp", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(block_refs(&g), vec!["Xp::Top", "Xp::Sub::Box"], "depth 1 by default");
        let ext = g.node("s-xp-sub-box").unwrap();
        assert_eq!(ext.mark.as_ref().and_then(|m| m.status.as_deref()), Some("external"));
        assert!(g.node("s-xp-top").unwrap().mark.is_none());
        assert!(g.edges.iter().any(|e| e.kind == EdgeKind::Composition && e.source == "s-xp-top" && e.target == "s-xp-sub-box"));
    }

    #[test]
    fn depth_limits_how_far_composition_is_followed() {
        let d = diagram("BDD", "Xp", |fm| fm.depth = Some(0));
        let (g, _) = derive_it(&d);
        assert_eq!(block_refs(&g), vec!["Xp::Top"]);
        assert!(g.edges.is_empty());
        let d = diagram("BDD", "Xp", |fm| fm.depth = Some(2));
        let (g, _) = derive_it(&d);
        assert_eq!(block_refs(&g), vec!["Xp::Top", "Xp::Sub::Box", "Xp::Sub::Leaf"]);
        // the self-composing Box terminates and keeps its own edge
        assert!(g.edges.iter().any(|e| e.source == "s-xp-sub-box" && e.target == "s-xp-sub-box"));
    }

    #[test]
    fn a_definition_subject_shows_its_composed_blocks() {
        let d = diagram("BDD", "Xp::Top", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(block_refs(&g), vec!["Xp::Top", "Xp::Sub::Box"]);
        assert!(g.edges.iter().any(|e| e.kind == EdgeKind::Composition));
    }

    #[test]
    fn include_accepts_a_name_qualified_relative_to_the_subject() {
        let d = diagram("BDD", "Xp", |fm| fm.include = Some(vec!["Top".into(), "Sub::Box".into(), "Ghost".into()]));
        let (g, issues) = derive_it(&d);
        // Box is a block of the diagram now, so what it composes (Leaf) is one level of external.
        assert_eq!(block_refs(&g), vec!["Xp::Top", "Xp::Sub::Box", "Xp::Sub::Leaf"]);
        assert_eq!(issues.len(), 1, "{issues:?}");
        assert!(issues[0].message.contains("Ghost"));
        // an explicitly included block is not an external one
        assert!(g.node("s-xp-sub-box").unwrap().mark.is_none());
        // exclude keeps an external block out
        let d = diagram("BDD", "Xp", |fm| fm.exclude = Some(vec!["Xp::Sub::Box".into()]));
        let (g, issues) = derive_it(&d);
        assert_eq!(block_refs(&g), vec!["Xp::Top"]);
        assert!(issues.is_empty(), "excluding an external block is not a W417: {issues:?}");
    }

    #[test]
    fn include_still_narrows_and_never_matches_a_bare_display_name_elsewhere() {
        // Sys has Base, Engine, Motor, …; Xp::Top is outside it. `include: [Engine]` keeps Engine only:
        // Base (Engine's supertype) is a member the list left out, so it is neither a block nor external.
        let d = diagram("BDD", "Sys", |fm| fm.include = Some(vec!["Engine".into()]));
        let (g, _) = derive_it(&d);
        assert_eq!(block_refs(&g), vec!["Sys::Engine"]);
        assert!(g.nodes.iter().all(|n| n.mark.is_none()));
        // A bare name of a definition that is not a member of the subject does not resolve.
        let d = diagram("BDD", "Xp", |fm| fm.include = Some(vec!["Top".into(), "Leaf".into()]));
        let (g, issues) = derive_it(&d);
        assert_eq!(block_refs(&g), vec!["Xp::Top", "Xp::Sub::Box"], "Leaf is not a direct member; only depth reaches Box");
        assert!(issues.iter().any(|i| i.message.contains("'Leaf'")), "{issues:?}");
    }
}

//! The one manifest parser (`REQ-TRS-VIS-002`): `shapes:`/`edges:`/`layout:`
//! frontmatter → [`DiagramGraph`].
//!
//! Honours the whole of spec §8.16.3–8.16.4 — the string shorthand
//! (`id: Qualified::Name`), the map form, an omitted `kind:` (defaults to
//! `block`), `parent:` nesting, `label:` on shapes and edges — and reports
//! what it cannot honour instead of silently rendering nothing:
//!
//! * `E405` — an entry is malformed (not a map or string, missing `ref`/
//!   `source`/`target`, an unknown `kind:`, a `parent:` naming no shape, a
//!   `layout:` entry without numeric `x`/`y`). The entry is skipped; the rest
//!   of the diagram still builds.
//! * `W416` — a `layout:` key names no shape or edge of the diagram (a stale
//!   pin).
//!
//! The validator turns [`Issue`]s into findings; renderers ignore them and
//! draw the graph that survived. `W402`/`W403` (unresolved ref, dangling edge
//! endpoint) keep their meaning in the validator and are not duplicated here;
//! an edge whose endpoint names no shape is simply not part of the graph.

use serde_yaml::{Mapping, Value};

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

use super::ir::{
    stereotype_for_type, DiagramGraph, DiagramKind, Edge, EdgeKind, Node, NodeKind, Point, PortDirection, Rect,
};

/// Frontmatter key for a diagram's `shapes:` map.
pub const KEY_SHAPES: &str = "shapes";
/// Frontmatter key for a diagram's `edges:` map.
pub const KEY_EDGES: &str = "edges";
/// Frontmatter key for a diagram's `layout:` map.
pub const KEY_LAYOUT: &str = "layout";
/// Key, within one `shapes:`/`edges:` entry, naming the qualified name it refers to.
pub const KEY_REF: &str = "ref";
/// Key, within one `shapes:`/`edges:` entry, naming its role.
pub const KEY_KIND: &str = "kind";
/// Key, within one `shapes:` entry, naming the enclosing shape.
pub const KEY_PARENT: &str = "parent";
/// Key, within one `shapes:`/`edges:` entry, overriding the display label.
pub const KEY_LABEL: &str = "label";
pub const KEY_SOURCE: &str = "source";
pub const KEY_TARGET: &str = "target";
/// Key, within one `layout:` entry keyed by an edge id, listing pinned waypoints.
pub const KEY_POINTS: &str = "points";

/// A problem found while parsing a manifest. `code` is `E405` or `W416`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub code: &'static str,
    pub message: String,
}

fn e405(message: String) -> Issue {
    Issue { code: "E405", message }
}

fn w416(message: String) -> Issue {
    Issue { code: "W416", message }
}

fn str_of<'a>(m: &'a Mapping, key: &str) -> Option<&'a str> {
    m.get(Value::String(key.into())).and_then(Value::as_str)
}

fn f64_of(m: &Mapping, key: &str) -> Option<f64> {
    m.get(Value::String(key.into())).and_then(Value::as_f64)
}

/// The entries of a `shapes:`/`edges:` value as `(id, entry)` pairs. Accepts
/// the spec's map form and the legacy sequence-of-maps-with-`id` form.
/// Returns `Err` when the value is neither.
fn entries<'a>(section: &str, v: &'a Value) -> Result<Vec<(String, &'a Value)>, Issue> {
    match v {
        Value::Mapping(m) => Ok(m
            .iter()
            .map(|(k, v)| {
                let id = match k {
                    Value::String(s) => s.clone(),
                    other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
                };
                (id, v)
            })
            .collect()),
        Value::Sequence(seq) => {
            let mut out = Vec::with_capacity(seq.len());
            for (i, item) in seq.iter().enumerate() {
                match item {
                    Value::Mapping(m) => match str_of(m, "id") {
                        Some(id) => out.push((id.to_string(), item)),
                        None => return Err(e405(format!("`{section}` entry #{i} (sequence form) has no `id`"))),
                    },
                    _ => return Err(e405(format!("`{section}` entry #{i} (sequence form) is not a map"))),
                }
            }
            Ok(out)
        }
        Value::Null => Ok(Vec::new()),
        _ => Err(e405(format!("`{section}` must be a map of entries"))),
    }
}

/// What a manifest `kind:` value meant: a §8.16.8 role, or an element type
/// name (`PartDef`, `Requirement`, …) standing in for one.
fn parse_shape_kind(raw: &str) -> Option<(NodeKind, Option<ElementType>)> {
    if let Some(t) = ElementType::ALL.iter().find(|t| t.name() == raw) {
        let role = match t {
            ElementType::Port | ElementType::PortDef => NodeKind::Port,
            ElementType::Requirement | ElementType::RequirementDef => NodeKind::Requirement,
            ElementType::TestCase => NodeKind::TestCase,
            ElementType::State | ElementType::StateDef | ElementType::ExhibitState => NodeKind::State,
            ElementType::UseCase | ElementType::UseCaseDef => NodeKind::UseCase,
            _ => NodeKind::Block,
        };
        return Some((role, Some(t.clone())));
    }
    NodeKind::parse_role(raw).map(|k| (k, None))
}

/// Stereotype text for a node that did not resolve to a typed element.
fn role_stereotype(kind: NodeKind) -> Option<&'static str> {
    match kind {
        NodeKind::Port => Some("port"),
        NodeKind::State => Some("state"),
        NodeKind::Action => Some("action"),
        NodeKind::Actor => Some("actor"),
        NodeKind::Requirement => Some("requirement"),
        NodeKind::TestCase => Some("test case"),
        NodeKind::UseCase => Some("use case"),
        NodeKind::Note => Some("comment"),
        _ => None,
    }
}

struct RawShape {
    id: String,
    element_ref: String,
    kind: NodeKind,
    typed_as: Option<ElementType>,
    parent: Option<String>,
    label: Option<String>,
}

fn parse_shape(id: &str, v: &Value, issues: &mut Vec<Issue>) -> Option<RawShape> {
    match v {
        Value::String(r) => Some(RawShape {
            id: id.to_string(),
            element_ref: r.clone(),
            kind: NodeKind::Block,
            typed_as: None,
            parent: None,
            label: None,
        }),
        Value::Mapping(m) => {
            let Some(element_ref) = str_of(m, KEY_REF) else {
                issues.push(e405(format!("shape `{id}` has no `ref`")));
                return None;
            };
            let (kind, typed_as) = match str_of(m, KEY_KIND) {
                None => (NodeKind::Block, None),
                Some(raw) => match parse_shape_kind(raw) {
                    Some(k) => k,
                    None => {
                        issues.push(e405(format!("shape `{id}` has unknown `kind` '{raw}'")));
                        return None;
                    }
                },
            };
            Some(RawShape {
                id: id.to_string(),
                element_ref: element_ref.to_string(),
                kind,
                typed_as,
                parent: str_of(m, KEY_PARENT).map(str::to_string),
                label: str_of(m, KEY_LABEL).map(str::to_string),
            })
        }
        _ => {
            issues.push(e405(format!("shape `{id}` is neither a map nor a qualified-name string")));
            None
        }
    }
}

fn parse_edge(id: &str, v: &Value, issues: &mut Vec<Issue>) -> Option<Edge> {
    let Value::Mapping(m) = v else {
        issues.push(e405(format!("edge `{id}` is not a map")));
        return None;
    };
    let (Some(source), Some(target)) = (str_of(m, KEY_SOURCE), str_of(m, KEY_TARGET)) else {
        issues.push(e405(format!("edge `{id}` needs both `source` and `target`")));
        return None;
    };
    let kind = match str_of(m, KEY_KIND) {
        None => EdgeKind::Connection,
        Some(raw) => match EdgeKind::parse(raw) {
            Some(k) => k,
            None => {
                issues.push(e405(format!("edge `{id}` has unknown `kind` '{raw}'")));
                return None;
            }
        },
    };
    Some(Edge {
        id: id.to_string(),
        element_ref: str_of(m, KEY_REF).map(str::to_string),
        source: source.to_string(),
        target: target.to_string(),
        kind,
        label: str_of(m, KEY_LABEL).map(str::to_string),
        waypoints: None,
    })
}

fn parse_points(v: &Value) -> Option<Vec<Point>> {
    let seq = v.as_sequence()?;
    let mut out = Vec::with_capacity(seq.len());
    for p in seq {
        let pt = match p {
            Value::Sequence(xy) if xy.len() == 2 => Point { x: xy[0].as_f64()?, y: xy[1].as_f64()? },
            Value::Mapping(m) => Point { x: f64_of(m, "x")?, y: f64_of(m, "y")? },
            _ => return None,
        };
        out.push(pt);
    }
    Some(out)
}

/// Apply a `layout:` map to an already-built graph: shape ids become pins,
/// edge ids with `points:` become waypoints, anything else is `W416`, and a
/// malformed entry is `E405`. Shared by the manifest and derived sources
/// (`REQ-TRS-VIS-003`: pins survive regeneration).
pub(super) fn apply_layout(graph: &mut DiagramGraph, layout: Option<&Value>, issues: &mut Vec<Issue>) {
    if let Some(v) = layout {
        match v {
            Value::Mapping(m) => {
                for (k, entry) in m {
                    let id = match k.as_str() {
                        Some(s) => s,
                        None => {
                            issues.push(e405("`layout` has a non-string key".to_string()));
                            continue;
                        }
                    };
                    let Value::Mapping(em) = entry else {
                        issues.push(e405(format!("`layout` entry `{id}` is not a map")));
                        continue;
                    };
                    if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == id) {
                        match (f64_of(em, "x"), f64_of(em, "y")) {
                            (Some(x), Some(y)) => {
                                node.pin = Some(Rect { x, y, w: f64_of(em, "w"), h: f64_of(em, "h") });
                            }
                            _ => issues.push(e405(format!("`layout` entry `{id}` needs numeric `x` and `y`"))),
                        }
                    } else if let Some(edge) = graph.edges.iter_mut().find(|e| e.id == id) {
                        match em.get(Value::String(KEY_POINTS.into())).and_then(parse_points) {
                            Some(points) => edge.waypoints = Some(points),
                            None => issues.push(e405(format!("`layout` entry `{id}` for an edge needs a `points` list of `[x, y]` pairs"))),
                        }
                    } else {
                        issues.push(w416(format!("`layout` entry `{id}` names no shape or edge of this diagram")));
                    }
                }
            }
            Value::Null => {}
            _ => issues.push(e405("`layout` must be a map of shape id → {x, y, w, h}".to_string())),
        }
    }

}

/// Build the IR of a manifest-sourced `Diagram` element.
///
/// `kind` is the diagram kind the caller already determined (see
/// [`super::build_graph`]). The returned issues are `E405`/`W416` entries for
/// the validator; the graph is whatever survived them.
pub fn build(
    elem: &RawElement,
    kind: DiagramKind,
    elements: &[RawElement],
    resolver: &Resolver,
) -> (DiagramGraph, Vec<Issue>) {
    let fm = &elem.frontmatter;
    let name = fm
        .name
        .clone()
        .unwrap_or_else(|| elem.qualified_name.rsplit("::").next().unwrap_or(&elem.qualified_name).to_string());
    let mut graph = DiagramGraph::empty(kind, &elem.qualified_name, &name, fm.subject.as_deref());
    let mut issues = Vec::new();

    // ── shapes ──────────────────────────────────────────────────────────
    let mut raw_shapes: Vec<RawShape> = Vec::new();
    if let Some(v) = fm.shapes.as_ref() {
        match entries(KEY_SHAPES, v) {
            Ok(list) => {
                for (id, v) in list {
                    if let Some(s) = parse_shape(&id, v, &mut issues) {
                        raw_shapes.push(s);
                    }
                }
            }
            Err(issue) => issues.push(issue),
        }
    }
    // A `parent:` must name a shape of this diagram, and the chain must be acyclic.
    let ids: std::collections::HashSet<&str> = raw_shapes.iter().map(|s| s.id.as_str()).collect();
    let parent_of: std::collections::HashMap<&str, &str> = raw_shapes
        .iter()
        .filter_map(|s| s.parent.as_deref().map(|p| (s.id.as_str(), p)))
        .collect();
    let mut bad_parent: std::collections::HashSet<String> = std::collections::HashSet::new();
    for s in &raw_shapes {
        let Some(p) = s.parent.as_deref() else { continue };
        if !ids.contains(p) {
            issues.push(e405(format!("shape `{}` has `parent` '{}' which is not a shape of this diagram", s.id, p)));
            bad_parent.insert(s.id.clone());
            continue;
        }
        // Cycle check: walk up at most `len` steps.
        let mut cur = p;
        let mut steps = 0;
        loop {
            if cur == s.id {
                issues.push(e405(format!("shape `{}` is its own ancestor through `parent`", s.id)));
                bad_parent.insert(s.id.clone());
                break;
            }
            match parent_of.get(cur) {
                Some(next) if steps < raw_shapes.len() => {
                    cur = next;
                    steps += 1;
                }
                _ => break,
            }
        }
    }

    for s in raw_shapes {
        let resolved = resolver.resolve_ref(elements, &s.element_ref);
        let element_type = resolved
            .and_then(|e| e.frontmatter.element_type.as_ref().map(|t| t.name().to_string()))
            .or_else(|| s.typed_as.as_ref().map(|t| t.name().to_string()));
        let label = s
            .label
            .clone()
            .or_else(|| resolved.and_then(|e| e.frontmatter.name.clone()))
            .unwrap_or_else(|| s.element_ref.rsplit("::").next().unwrap_or(&s.element_ref).to_string());
        let stereotype = match s.kind {
            // Decorations carry no stereotype of their own.
            NodeKind::Compartment | NodeKind::Label | NodeKind::Swimlane | NodeKind::Activation
            | NodeKind::Fragment | NodeKind::Initial | NodeKind::Final | NodeKind::Choice | NodeKind::History
            | NodeKind::SystemBoundary => None,
            _ => resolved
                .and_then(|e| e.frontmatter.element_type.as_ref().map(stereotype_for_type))
                .or_else(|| s.typed_as.as_ref().map(stereotype_for_type))
                .or_else(|| role_stereotype(s.kind).map(str::to_string)),
        };
        let direction = if s.kind == NodeKind::Port {
            resolved.and_then(|e| e.frontmatter.direction.as_deref()).and_then(PortDirection::parse)
        } else {
            None
        };
        graph.nodes.push(Node {
            id: s.id.clone(),
            element_ref: s.element_ref,
            resolved: resolved.is_some(),
            element_type,
            kind: s.kind,
            label,
            stereotype,
            parent: if bad_parent.contains(&s.id) { None } else { s.parent },
            direction,
            side: None,
            lines: Vec::new(),
            is_abstract: resolved.and_then(|e| e.frontmatter.is_abstract).unwrap_or(false),
            pin: None,
            banners: resolved.map(|e| super::banners_of(e, elements, resolver)).unwrap_or_default(),
        });
    }

    // ── edges ───────────────────────────────────────────────────────────
    if let Some(v) = fm.edges.as_ref() {
        match entries(KEY_EDGES, v) {
            Ok(list) => {
                for (id, v) in list {
                    if let Some(e) = parse_edge(&id, v, &mut issues) {
                        // An endpoint naming no shape is `W403` in the validator;
                        // the edge cannot be drawn, so it is not part of the graph.
                        if graph.node(&e.source).is_some() && graph.node(&e.target).is_some() {
                            graph.edges.push(e);
                        }
                    }
                }
            }
            Err(issue) => issues.push(issue),
        }
    }

    apply_layout(&mut graph, fm.layout.as_ref(), &mut issues);

    (graph, issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::RawFrontmatter;

    fn elem(qname: &str, t: ElementType, extra: impl FnOnce(&mut RawFrontmatter)) -> RawElement {
        let mut fm = RawFrontmatter { element_type: Some(t), ..Default::default() };
        extra(&mut fm);
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("{}.md", qname.replace("::", "/")),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: Vec::new(),
            locale_docs: Default::default(),
            about_notes: Vec::new(),
        }
    }

    fn yaml(s: &str) -> Value {
        serde_yaml::from_str(s).unwrap()
    }

    fn model() -> Vec<RawElement> {
        vec![
            elem("Sys", ElementType::Package, |_| {}),
            elem("Sys::Engine", ElementType::PartDef, |fm| {
                fm.name = Some("Engine".into());
                fm.is_abstract = Some(true);
            }),
            elem("Sys::Motor", ElementType::PartDef, |fm| fm.name = Some("Electric Motor".into())),
            elem("Sys::Engine::powerOut", ElementType::Port, |fm| fm.direction = Some("out".into())),
            elem("Sys::Safety", ElementType::MetadataDef, |fm| fm.name = Some("SafetyCritical".into())),
            elem("Sys::Stereotyped", ElementType::PartDef, |fm| {
                fm.name = Some("Stereotyped".into());
                fm.metadata = Some(vec![
                    yaml("Sys::Safety"),
                    yaml("{type: ModelingMetadata::Rationale, text: because}"),
                    yaml("Nope::Missing"),
                ]);
            }),
        ]
    }

    /// `REQ-TRS-VIS-012`: a resolved element's `metadata:` applications become
    /// banners — the def's `name` when it resolves, else the reference's last
    /// segment (standard-library metadata has no file). Unresolved shapes have none.
    #[test]
    fn applied_stereotypes_become_banners() {
        let d = diagram("BDD", "s: Sys::Stereotyped\nplain: Sys::Engine\nghost: Nope::X\n", None, None);
        let (g, issues) = build_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.node("s").unwrap().banners, vec!["SafetyCritical", "Rationale", "Missing"]);
        assert!(g.node("plain").unwrap().banners.is_empty());
        assert!(g.node("ghost").unwrap().banners.is_empty());
    }

    fn diagram(kind: &str, shapes: &str, edges: Option<&str>, layout: Option<&str>) -> RawElement {
        elem("Diagrams::D", ElementType::Diagram, |fm| {
            fm.diagram_kind = Some(kind.into());
            fm.name = Some("D".into());
            fm.subject = Some("Sys".into());
            fm.shapes = Some(yaml(shapes));
            fm.edges = edges.map(yaml);
            fm.layout = layout.map(yaml);
        })
    }

    fn build_it(d: &RawElement) -> (DiagramGraph, Vec<Issue>) {
        let mut elements = model();
        elements.push(d.clone());
        let resolver = Resolver::new(&elements);
        build(d, DiagramKind::parse(d.frontmatter.diagram_kind.as_deref()).unwrap(), &elements, &resolver)
    }

    #[test]
    fn string_shorthand_and_map_form_both_parse() {
        let d = diagram(
            "BDD",
            "engine: Sys::Engine\nmotor:\n  ref: Sys::Motor\n  kind: block\n",
            Some("inh:\n  source: motor\n  target: engine\n  kind: inheritance\n"),
            None,
        );
        let (g, issues) = build_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.nodes.len(), 2);
        let engine = g.node("engine").unwrap();
        assert_eq!(engine.kind, NodeKind::Block);
        assert!(engine.resolved);
        assert_eq!(engine.element_type.as_deref(), Some("PartDef"));
        assert_eq!(engine.stereotype.as_deref(), Some("part def"));
        assert!(engine.is_abstract);
        assert_eq!(g.node("motor").unwrap().label, "Electric Motor");
        assert_eq!(g.edges.len(), 1);
        assert_eq!(g.edges[0].kind, EdgeKind::Inheritance);
    }

    #[test]
    fn omitted_kind_defaults_to_block_and_omitted_edge_kind_to_connection() {
        let d = diagram("IBD", "a:\n  ref: Sys::Engine\nb:\n  ref: Sys::Motor\n", Some("e:\n  source: a\n  target: b\n"), None);
        let (g, issues) = build_it(&d);
        assert!(issues.is_empty());
        assert_eq!(g.node("a").unwrap().kind, NodeKind::Block);
        assert_eq!(g.edges[0].kind, EdgeKind::Connection);
    }

    #[test]
    fn parent_nesting_and_ports_are_honoured() {
        let d = diagram(
            "IBD",
            "sys:\n  ref: Sys\n  kind: boundary\nengine:\n  ref: Sys::Engine\n  kind: block\n  parent: sys\npout:\n  ref: Sys::Engine::powerOut\n  kind: port\n  parent: engine\n",
            None,
            None,
        );
        let (g, issues) = build_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.node("engine").unwrap().parent.as_deref(), Some("sys"));
        let port = g.node("pout").unwrap();
        assert_eq!(port.kind, NodeKind::Port);
        assert_eq!(port.parent.as_deref(), Some("engine"));
        assert_eq!(port.direction, Some(PortDirection::Out));
        assert_eq!(port.stereotype.as_deref(), Some("port"));
        assert_eq!(g.children_of("sys").count(), 1);
        assert_eq!(g.roots().count(), 1);
    }

    #[test]
    fn element_type_names_are_accepted_as_kinds() {
        let d = diagram("IBD", "a: {ref: Sys::Engine, kind: Part}\nr: {ref: Nope::Req, kind: Requirement}\n", None, None);
        let (g, issues) = build_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.node("a").unwrap().kind, NodeKind::Block);
        // Resolved element's real type wins over the manifest's `Part`.
        assert_eq!(g.node("a").unwrap().element_type.as_deref(), Some("PartDef"));
        let r = g.node("r").unwrap();
        assert_eq!(r.kind, NodeKind::Requirement);
        assert!(!r.resolved);
        assert_eq!(r.element_type.as_deref(), Some("Requirement"));
        assert_eq!(r.stereotype.as_deref(), Some("requirement"));
        assert_eq!(r.label, "Req");
    }

    #[test]
    fn layout_pins_nodes_and_edges_and_flags_stale_keys() {
        let d = diagram(
            "BDD",
            "a: Sys::Engine\nb: Sys::Motor\n",
            Some("e: {source: a, target: b, kind: association}\n"),
            Some("a: {x: 10, y: 20, w: 100}\ne: {points: [[1, 2], {x: 3, y: 4}]}\nzz: {x: 0, y: 0}\n"),
        );
        let (g, issues) = build_it(&d);
        assert_eq!(issues, vec![w416("`layout` entry `zz` names no shape or edge of this diagram".into())]);
        assert_eq!(g.node("a").unwrap().pin, Some(Rect { x: 10.0, y: 20.0, w: Some(100.0), h: None }));
        assert_eq!(g.node("b").unwrap().pin, None);
        assert_eq!(g.edges[0].waypoints.as_ref().unwrap().len(), 2);
        assert!(!g.is_fully_pinned());
        assert_eq!(g.pinned_ids(), vec!["a"]);
    }

    #[test]
    fn malformed_entries_are_e405_and_the_rest_still_builds() {
        let d = diagram(
            "BDD",
            "good: Sys::Engine\nnoref: {kind: block}\nbadkind: {ref: Sys::Motor, kind: gizmo}\nnum: 42\norphan: {ref: Sys::Motor, parent: nowhere}\n",
            Some("ok: {source: good, target: orphan}\nnoends: {kind: flow}\nbadkind: {source: good, target: orphan, kind: teleport}\n"),
            Some("good: {x: 1}\n"),
        );
        let (g, issues) = build_it(&d);
        let codes: Vec<&str> = issues.iter().map(|i| i.code).collect();
        assert_eq!(codes, vec!["E405"; 7], "{issues:?}");
        assert!(issues.iter().any(|i| i.message.contains("edge `badkind` has unknown `kind` 'teleport'")));
        assert!(issues.iter().any(|i| i.message.contains("`noref` has no `ref`")));
        assert!(issues.iter().any(|i| i.message.contains("`badkind` has unknown `kind` 'gizmo'")));
        assert!(issues.iter().any(|i| i.message.contains("`num` is neither")));
        assert!(issues.iter().any(|i| i.message.contains("'nowhere'")));
        assert!(issues.iter().any(|i| i.message.contains("`noends` needs both")));
        assert!(issues.iter().any(|i| i.message.contains("`good` needs numeric")));
        // Survivors: `good`, `orphan` (parent dropped) and the `ok` edge.
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.node("orphan").unwrap().parent, None);
        assert_eq!(g.edges.len(), 1);
    }

    #[test]
    fn non_map_sections_are_e405() {
        let d = diagram("BDD", "- 1\n- 2\n", Some("just a string\n"), Some("- 1\n"));
        let (g, issues) = build_it(&d);
        assert_eq!(issues.iter().filter(|i| i.code == "E405").count(), 3, "{issues:?}");
        assert!(g.nodes.is_empty());
    }

    #[test]
    fn legacy_sequence_form_with_ids_is_accepted() {
        let d = diagram("BDD", "- id: a\n  ref: Sys::Engine\n- id: b\n  ref: Sys::Motor\n", Some("- id: e\n  source: a\n  target: b\n"), None);
        let (g, issues) = build_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.edges.len(), 1);
    }

    #[test]
    fn edge_with_unknown_endpoint_is_left_to_w403_and_not_drawn() {
        let d = diagram("BDD", "a: Sys::Engine\n", Some("e: {source: a, target: ghost}\n"), None);
        let (g, issues) = build_it(&d);
        assert!(issues.is_empty());
        assert!(g.edges.is_empty());
    }

    #[test]
    fn parent_cycle_is_e405() {
        let d = diagram("IBD", "a: {ref: Sys::Engine, parent: b}\nb: {ref: Sys::Motor, parent: a}\n", None, None);
        let (g, issues) = build_it(&d);
        assert_eq!(issues.iter().filter(|i| i.code == "E405").count(), 2, "{issues:?}");
        assert!(g.nodes.iter().all(|n| n.parent.is_none()));
    }
}

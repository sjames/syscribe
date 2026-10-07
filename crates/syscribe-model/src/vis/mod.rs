//! Visualisation (`ADR-SYS-VIS-001`, `REQ-TRS-VIS-*`; design in
//! `docs/design/visualisation.md`).
//!
//! One Diagram IR ([`ir::DiagramGraph`]) is built per `Diagram` element,
//! either from the author's `shapes:`/`edges:`/`layout:` manifest
//! ([`manifest`]) or — when the element declares a `subject:` and no
//! `shapes:` — derived from the model by the generator for its kind. Every
//! renderer and exporter is a pure function of that value. Node sizes come
//! from the shared text metrics ([`metrics`], [`size`]) and automatic layout
//! from the embedded ELK ([`layout`]) — the same engine and sizes the
//! browser client uses (`REQ-TRS-VIS-016`/`-017`).
//!
//! Source selection needs no mode field (`REQ-TRS-VIS-003`): the presence of
//! `shapes:` picks the manifest; a `subject:` alone picks derivation.

pub mod derive;
pub mod ir;
pub mod layout;
pub mod manifest;
pub mod mermaid;
pub mod metrics;
pub mod size;
pub mod sprotty;
pub mod style;
pub mod svg;

use crate::element::{ElementType, RawElement, RawFrontmatter};
use crate::resolver::Resolver;

pub use ir::{DiagramGraph, DiagramKind, Edge, EdgeKind, LayoutHints, Node, NodeKind, Point, PortDirection, Rect, Side};
pub use layout::{layout, Layout, LayoutError};
pub use manifest::Issue;
pub use mermaid::render_mermaid;
pub use metrics::TextMetrics;
pub use size::{size_graph, Sizes};
pub use style::{ArrowHead, EdgeStyle, NodeStyle, PortStyle};
pub use svg::{render_svg, SvgError};

/// The applied-stereotype banners of an element (`REQ-TRS-VIS-012`): one
/// entry per `metadata:` application, naming the applied `MetadataDef` — its
/// `name` when the reference resolves, else the last segment of the reference
/// as written (standard-library metadata such as `ModelingMetadata::Rationale`
/// has no in-model file). Declaration order; never deduplicated.
pub fn banners_of(elem: &RawElement, elements: &[RawElement], resolver: &Resolver) -> Vec<String> {
    crate::element::metadata_applications(&elem.frontmatter.metadata)
        .into_iter()
        .map(|app| {
            resolver
                .resolve_ref(elements, &app.def)
                .filter(|d| matches!(d.frontmatter.element_type, Some(ElementType::MetadataDef)))
                .and_then(|d| d.frontmatter.name.clone())
                .unwrap_or_else(|| app.def.rsplit("::").next().unwrap_or(&app.def).to_string())
        })
        .collect()
}

/// The fallback `diagramKind` used when a `Diagram` element doesn't declare
/// one — the single source of truth for the literal `"SVG"` the server's UI
/// routes report for an un-kinded diagram.
pub const DEFAULT_DIAGRAM_KIND: &str = "SVG";

/// Where a `Diagram` element's content comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The author listed the content in `shapes:` (and `edges:`/`layout:`).
    Manifest,
    /// Only a `subject:` is declared: the generator for the diagram's kind
    /// walks it.
    Derived,
    /// Neither `shapes:` nor `subject:`: nothing to draw (`W400`/`W401`
    /// already cover the missing pieces).
    Empty,
}

/// The source of a `Diagram` element's content (`REQ-TRS-VIS-003`).
pub fn source_of(fm: &RawFrontmatter) -> Source {
    let has_shapes = matches!(fm.shapes.as_ref(), Some(v) if !v.is_null());
    if has_shapes {
        Source::Manifest
    } else if fm.subject.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false) {
        Source::Derived
    } else {
        Source::Empty
    }
}

/// Whether the element is a `Diagram` whose kind has an IR (everything but
/// the hand-authored `Mermaid`/`PlantUML` kinds).
pub fn has_ir(elem: &RawElement) -> bool {
    matches!(elem.frontmatter.element_type, Some(ElementType::Diagram))
        && DiagramKind::parse(elem.frontmatter.diagram_kind.as_deref()).is_some()
}

/// Build the IR of a `Diagram` element, with the issues found on the way:
/// `E405`/`W416` from the manifest parser, `W417`/`W418` from the generators.
/// `None` when the element is not a `Diagram` or its kind has no IR
/// (`Mermaid`, `PlantUML`).
pub fn build_graph(elem: &RawElement, elements: &[RawElement], resolver: &Resolver) -> Option<(DiagramGraph, Vec<Issue>)> {
    if !matches!(elem.frontmatter.element_type, Some(ElementType::Diagram)) {
        return None;
    }
    let kind = DiagramKind::parse(elem.frontmatter.diagram_kind.as_deref())?;
    match source_of(&elem.frontmatter) {
        Source::Manifest => {
            let (graph, mut issues) = manifest::build(elem, kind, elements, resolver);
            let filters = derive::Filters::of(elem);
            if !filters.is_empty() {
                issues.push(derive::w417(
                    "`include`/`exclude` apply only to a derived diagram (one with a `subject` and no `shapes`) — ignored".to_string(),
                ));
            }
            Some((graph, issues))
        }
        Source::Derived => Some(derive::derive(elem, kind, elements, resolver)),
        Source::Empty => {
            let name = elem
                .frontmatter
                .name
                .clone()
                .unwrap_or_else(|| elem.qualified_name.rsplit("::").next().unwrap_or(&elem.qualified_name).to_string());
            Some((
                DiagramGraph::empty(kind, &elem.qualified_name, &name, elem.frontmatter.subject.as_deref()),
                Vec::new(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(qname: &str, fm: RawFrontmatter) -> RawElement {
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

    #[test]
    fn source_is_selected_by_frontmatter_presence() {
        let mut fm = RawFrontmatter::default();
        assert_eq!(source_of(&fm), Source::Empty);
        fm.subject = Some("Sys".into());
        assert_eq!(source_of(&fm), Source::Derived);
        fm.shapes = Some(serde_yaml::from_str("a: Sys::A").unwrap());
        assert_eq!(source_of(&fm), Source::Manifest);
        fm.shapes = Some(serde_yaml::Value::Null);
        assert_eq!(source_of(&fm), Source::Derived, "an empty `shapes:` key does not select the manifest");
    }

    #[test]
    fn hand_authored_kinds_and_non_diagrams_have_no_ir() {
        let mut e = raw("D", RawFrontmatter { element_type: Some(ElementType::Diagram), ..Default::default() });
        assert!(has_ir(&e));
        e.frontmatter.diagram_kind = Some("Mermaid".into());
        assert!(!has_ir(&e));
        e.frontmatter.diagram_kind = Some("IBD".into());
        e.frontmatter.element_type = Some(ElementType::PartDef);
        assert!(!has_ir(&e));
        let elements = vec![e.clone()];
        let resolver = Resolver::new(&elements);
        assert!(build_graph(&e, &elements, &resolver).is_none());
    }

    #[test]
    fn derived_source_yields_an_empty_graph_with_subject() {
        let e = raw(
            "Diagrams::D",
            RawFrontmatter {
                element_type: Some(ElementType::Diagram),
                diagram_kind: Some("IBD".into()),
                subject: Some("Sys::Engine".into()),
                ..Default::default()
            },
        );
        let elements = vec![e.clone()];
        let resolver = Resolver::new(&elements);
        let (g, issues) = build_graph(&e, &elements, &resolver).unwrap();
        assert!(issues.is_empty());
        assert_eq!(g.kind, DiagramKind::Ibd);
        assert_eq!(g.subject.as_deref(), Some("Sys::Engine"));
        assert_eq!(g.name, "D");
        assert!(g.nodes.is_empty());
    }
}

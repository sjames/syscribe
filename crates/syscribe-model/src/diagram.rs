//! Shared diagram-frontmatter parsing (`REQ-TRS-DE-003`, `REQ-TRS-DE-004`).
//!
//! A diagram element's `shapes:`/`edges:`/`layout:` frontmatter is a set of
//! YAML maps keyed by a diagram-local shape/edge id. Before this module, that
//! shape was deserialized independently in `renderer.rs` (private structs,
//! backing the SVG renderer) and `syscribe-server`'s `routes/diagram_model.rs`
//! (its own private, near-identical copy, backing the sprotty-client JSON
//! endpoint) — this module is the single shared source of truth for both.
//!
//! `routes/mutate.rs`'s diagram-sync helpers *write* these same YAML shapes
//! (hand-constructing `serde_yaml::Mapping`s rather than deserializing), so
//! they don't consume the typed structs here, but they do share the
//! `KEY_*` key-name constants below so both the read and write sides agree on
//! one spelling of each field.

use std::collections::HashMap;

use serde::Deserialize;

/// One entry in a diagram's `shapes:` map: a reference to a model element
/// rendered as a node, plus its rendering `kind`.
#[derive(Debug, Deserialize)]
pub struct DiagramShape {
    #[serde(rename = "ref")]
    pub element_ref: String,
    pub kind: String,
}

/// One entry in a diagram's `edges:` map: a connection between two shape ids.
#[derive(Debug, Deserialize)]
pub struct DiagramEdge {
    #[serde(rename = "ref")]
    pub element_ref: Option<String>,
    pub source: String,
    pub target: String,
    pub kind: String,
}

/// One entry in a diagram's `layout:` map: a shape's position and (optional,
/// overriding `default_size`) explicit size.
#[derive(Debug, Deserialize)]
pub struct ShapeLayout {
    pub x: f64,
    pub y: f64,
    pub w: Option<f64>,
    pub h: Option<f64>,
}

/// The fallback `diagramKind` used when a `Diagram` element doesn't declare
/// one — the single source of truth for what used to be the literal `"SVG"`
/// hardcoded independently in `syscribe-server`'s `routes/ui.rs` (twice) and
/// `routes/diagram_model.rs` (once).
pub const DEFAULT_DIAGRAM_KIND: &str = "SVG";

/// Frontmatter key for a diagram's `shapes:` map.
pub const KEY_SHAPES: &str = "shapes";
/// Frontmatter key for a diagram's `edges:` map.
pub const KEY_EDGES: &str = "edges";
/// Frontmatter key for a diagram's `layout:` map.
pub const KEY_LAYOUT: &str = "layout";
/// Key, within one `shapes:`/`edges:` entry, naming the qualified-name it
/// refers to.
pub const KEY_REF: &str = "ref";
/// Key, within one `shapes:`/`edges:` entry, naming its rendering kind.
pub const KEY_KIND: &str = "kind";

/// Default `(width, height)` per element `kind`, used when a `layout:` entry
/// doesn't override `w`/`h` explicitly.
pub fn default_size(kind: &str) -> (f64, f64) {
    match kind {
        "RequirementDef" => (240.0, 56.0),
        "Requirement" => (240.0, 70.0),
        "TestCase" | "TestCaseDef" => (200.0, 56.0),
        "PartDef" | "Part" => (160.0, 46.0),
        _ => (200.0, 50.0),
    }
}

/// Parse a diagram's `shapes:` frontmatter value into `{shapeId: DiagramShape}`,
/// falling back to an empty map on missing/malformed YAML — matching the
/// `.ok().unwrap_or_default()` behavior every call site already relied on.
pub fn parse_shapes(v: Option<&serde_yaml::Value>) -> HashMap<String, DiagramShape> {
    v.and_then(|v| serde_yaml::from_value(v.clone()).ok())
        .unwrap_or_default()
}

/// Parse a diagram's `edges:` frontmatter value into `{edgeId: DiagramEdge}`,
/// falling back to an empty map on missing/malformed YAML.
pub fn parse_edges(v: Option<&serde_yaml::Value>) -> HashMap<String, DiagramEdge> {
    v.and_then(|v| serde_yaml::from_value(v.clone()).ok())
        .unwrap_or_default()
}

/// Parse a diagram's `layout:` frontmatter value into `{shapeId: ShapeLayout}`,
/// falling back to an empty map on missing/malformed YAML.
pub fn parse_layout(v: Option<&serde_yaml::Value>) -> HashMap<String, ShapeLayout> {
    v.and_then(|v| serde_yaml::from_value(v.clone()).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_shapes_missing_is_empty() {
        assert!(parse_shapes(None).is_empty());
    }

    #[test]
    fn parse_shapes_malformed_is_empty() {
        // A sequence where a mapping is expected — `from_value` fails, and the
        // helper must fall back to empty rather than propagate the error.
        let v: serde_yaml::Value = serde_yaml::from_str("[1, 2, 3]").unwrap();
        assert!(parse_shapes(Some(&v)).is_empty());
    }

    #[test]
    fn parse_shapes_well_formed_round_trips() {
        let yaml = r#"
        s1:
          ref: Foo::Bar
          kind: PartDef
        "#;
        let v: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let shapes = parse_shapes(Some(&v));
        assert_eq!(shapes.len(), 1);
        let s = &shapes["s1"];
        assert_eq!(s.element_ref, "Foo::Bar");
        assert_eq!(s.kind, "PartDef");
    }

    #[test]
    fn parse_edges_missing_is_empty() {
        assert!(parse_edges(None).is_empty());
    }

    #[test]
    fn parse_edges_malformed_is_empty() {
        let v: serde_yaml::Value = serde_yaml::from_str("not_a_map: [1,2]").unwrap();
        // This actually parses as a mapping whose value is a sequence, which is
        // fine at the top level but each entry must itself be a DiagramEdge
        // mapping — "not_a_map" -> [1,2] fails to deserialize as DiagramEdge,
        // so the whole map fails and falls back to empty.
        assert!(parse_edges(Some(&v)).is_empty());
    }

    #[test]
    fn parse_edges_well_formed_round_trips() {
        let yaml = r#"
        e1:
          ref: Foo::Conn
          source: s1
          target: s2
          kind: connection
        "#;
        let v: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let edges = parse_edges(Some(&v));
        assert_eq!(edges.len(), 1);
        let e = &edges["e1"];
        assert_eq!(e.element_ref.as_deref(), Some("Foo::Conn"));
        assert_eq!(e.source, "s1");
        assert_eq!(e.target, "s2");
        assert_eq!(e.kind, "connection");
    }

    #[test]
    fn parse_layout_missing_is_empty() {
        assert!(parse_layout(None).is_empty());
    }

    #[test]
    fn parse_layout_malformed_is_empty() {
        let v: serde_yaml::Value = serde_yaml::from_str("[1, 2, 3]").unwrap();
        assert!(parse_layout(Some(&v)).is_empty());
    }

    #[test]
    fn parse_layout_well_formed_round_trips() {
        let yaml = r#"
        s1:
          x: 10.0
          y: 20.0
          w: 100.0
        "#;
        let v: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let layout = parse_layout(Some(&v));
        assert_eq!(layout.len(), 1);
        let l = &layout["s1"];
        assert_eq!(l.x, 10.0);
        assert_eq!(l.y, 20.0);
        assert_eq!(l.w, Some(100.0));
        assert_eq!(l.h, None);
    }

    #[test]
    fn default_size_known_kinds() {
        assert_eq!(default_size("RequirementDef"), (240.0, 56.0));
        assert_eq!(default_size("Requirement"), (240.0, 70.0));
        assert_eq!(default_size("TestCase"), (200.0, 56.0));
        assert_eq!(default_size("TestCaseDef"), (200.0, 56.0));
        assert_eq!(default_size("PartDef"), (160.0, 46.0));
        assert_eq!(default_size("Part"), (160.0, 46.0));
        assert_eq!(default_size("SomethingElse"), (200.0, 50.0));
    }
}

#![allow(dead_code)]

use std::collections::HashMap;

// ── View configuration ────────────────────────────────────────────────────────

/// Which compartments are visible in a given diagram context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewPreset {
    /// All compartments: header, status badges, ports, features, doc/gherkin
    Full,
    /// Header + ports compartment only
    Ports,
    /// Header + features compartment only
    Features,
    /// Header + status badges only
    Compact,
    /// Header only (stereotype + name, no badges)
    Name,
    /// Header + status badges + doc preview (for requirement/ADR diagrams)
    Requirement,
}

impl ViewPreset {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ports" => Self::Ports,
            "features" => Self::Features,
            "compact" => Self::Compact,
            "name" => Self::Name,
            "requirement" | "req" => Self::Requirement,
            _ => Self::Full,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Ports => "ports",
            Self::Features => "features",
            Self::Compact => "compact",
            Self::Name => "name",
            Self::Requirement => "requirement",
        }
    }

    /// Returns true if this compartment kind is visible under this preset.
    pub fn allows(&self, kind: CompartmentKind) -> bool {
        match self {
            Self::Full => true,
            Self::Ports => matches!(kind, CompartmentKind::Header | CompartmentKind::Ports),
            Self::Features => matches!(kind, CompartmentKind::Header | CompartmentKind::Features),
            Self::Compact => matches!(kind, CompartmentKind::Header | CompartmentKind::Status),
            Self::Name => matches!(kind, CompartmentKind::Header),
            Self::Requirement => matches!(
                kind,
                CompartmentKind::Header | CompartmentKind::Status | CompartmentKind::Doc
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompartmentKind {
    Header,
    Status,
    Ports,
    Features,
    Doc,
}

/// Per-compartment name filters. If a list is present, only items whose names
/// are in the list are included. Absent = include all.
#[derive(Debug, Clone, Default)]
pub struct IncludeFilter {
    /// Only include ports with these names (None = all)
    pub ports: Option<Vec<String>>,
    /// Only include features with these names (None = all)
    pub features: Option<Vec<String>>,
}

impl IncludeFilter {
    pub fn is_empty(&self) -> bool {
        self.ports.is_none() && self.features.is_none()
    }

    pub fn port_allowed(&self, name: &str) -> bool {
        self.ports.as_ref().map_or(true, |list| list.iter().any(|n| n == name))
    }

    pub fn feature_allowed(&self, name: &str) -> bool {
        self.features.as_ref().map_or(true, |list| list.iter().any(|n| n == name))
    }

    /// Serialize to a JSON-compatible map for `effective_include` output.
    pub fn to_map(&self) -> HashMap<String, Vec<String>> {
        let mut m = HashMap::new();
        if let Some(ports) = &self.ports {
            m.insert("ports".to_string(), ports.clone());
        }
        if let Some(features) = &self.features {
            m.insert("features".to_string(), features.clone());
        }
        m
    }
}

/// Complete view specification: preset + per-item name filters + optional size override.
#[derive(Debug, Clone)]
pub struct ViewConfig {
    pub preset: ViewPreset,
    pub include: IncludeFilter,
    pub min_width: Option<f64>,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self { preset: ViewPreset::Full, include: IncludeFilter::default(), min_width: None }
    }
}

impl ViewConfig {
    pub fn from_preset(s: &str) -> Self {
        Self { preset: ViewPreset::from_str(s), ..Default::default() }
    }
}

// ── Design tokens — 8px grid ──────────────────────────────────────────────────
pub const MIN_WIDTH: f64 = 160.0;
pub const PAD_H: f64 = 12.0;
pub const PAD_V: f64 = 8.0;
pub const GAP: f64 = 4.0;
pub const SECTION_GAP: f64 = 6.0;
pub const DIVIDER_H: f64 = 1.0;
pub const CORNER_R: f64 = 6.0;
pub const BORDER_W: f64 = 1.5;

// Font sizes
pub const FS_STEREOTYPE: f64 = 9.0;
pub const FS_NAME: f64 = 13.0;
pub const FS_SECTION_TITLE: f64 = 9.0;
pub const FS_ROW: f64 = 11.0;
pub const FS_DOC: f64 = 10.0;
pub const FS_BADGE: f64 = 9.0;

/// RGBA color as CSS hex string.
#[derive(Debug, Clone)]
pub struct Color(pub &'static str);

impl Color {
    pub fn as_str(&self) -> &str {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct DropShadow {
    pub dx: f64,
    pub dy: f64,
    pub blur: f64,
    pub color: &'static str,
}

#[derive(Debug, Clone)]
pub struct ElementTheme {
    pub header_bg: &'static str,
    pub header_fg: &'static str,
    pub stereotype_fg: &'static str,
    pub body_bg: &'static str,
    pub body_fg: &'static str,
    pub muted_fg: &'static str,
    pub border: &'static str,
    pub divider: &'static str,
    pub accent: &'static str,
    pub shadow: Option<DropShadow>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortDirection {
    In,
    Out,
    InOut,
    Undirected,
}

impl PortDirection {
    pub fn arrow(&self) -> &'static str {
        match self {
            PortDirection::In => "▶",
            PortDirection::Out => "◀",
            PortDirection::InOut => "◆",
            PortDirection::Undirected => "○",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            PortDirection::In => "#3a6ea5",
            PortDirection::Out => "#c47a1e",
            PortDirection::InOut => "#7a3ea5",
            PortDirection::Undirected => "#666",
        }
    }
}

#[derive(Debug, Clone)]
pub enum PortSide {
    Left,
    Right,
}

impl From<&PortDirection> for PortSide {
    fn from(d: &PortDirection) -> Self {
        match d {
            PortDirection::In => PortSide::Left,
            PortDirection::Out => PortSide::Right,
            PortDirection::InOut => PortSide::Left,
            PortDirection::Undirected => PortSide::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Badge {
    pub text: String,
    pub bg: &'static str,
    pub fg: &'static str,
    pub mono: bool,
}

#[derive(Debug, Clone)]
pub struct PortRow {
    pub name: String,
    pub type_ref: Option<String>,
    pub direction: PortDirection,
    pub multiplicity: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FeatureRow {
    pub name: String,
    pub type_ref: String,
    pub multiplicity: Option<String>,
    pub unit: Option<String>,
    pub value: Option<String>,
    pub is_derived: bool,
    pub is_constant: bool,
}

#[derive(Debug, Clone)]
pub enum Compartment {
    Header {
        stereotype: Option<String>,
        name: String,
        is_abstract: bool,
        badges: Vec<Badge>,
    },
    StatusRow {
        badges: Vec<Badge>,
    },
    PortsList {
        items: Vec<PortRow>,
    },
    Features {
        items: Vec<FeatureRow>,
    },
    DocPreview {
        lines: Vec<String>,
    },
    GherkinPreview {
        given: Option<String>,
        when: Option<String>,
        then: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct ElementNode {
    pub qualified_name: String,
    pub element_type_label: String,
    pub compartments: Vec<Compartment>,
    pub theme: ElementTheme,
    pub min_width: f64,
}

/// Output of the layout pass — SVG fragment + metadata for assembly
#[derive(Debug, Clone)]
pub struct RenderedElement {
    pub qualified_name: String,
    pub svg: String,
    pub width: f64,
    pub height: f64,
    pub port_anchors: Vec<PortAnchor>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PortAnchor {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub side: String,
    pub direction: String,
}

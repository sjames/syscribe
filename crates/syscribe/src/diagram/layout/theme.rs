use crate::diagram::layout::types::{DropShadow, ElementTheme};
use syscribe_model::element::ElementType;

const SHADOW: DropShadow = DropShadow {
    dx: 0.0,
    dy: 2.0,
    blur: 6.0,
    color: "rgba(0,0,0,0.10)",
};

pub fn theme_for(element_type: &ElementType) -> ElementTheme {
    match element_type {
        ElementType::PartDef | ElementType::Part => ElementTheme {
            header_bg: "#dde2f0",
            header_fg: "#1a1a2e",
            stereotype_fg: "#555",
            body_bg: "#f8f9fc",
            body_fg: "#1a1a2e",
            muted_fg: "#888",
            border: "#3a3a5c",
            divider: "#d0d0e0",
            accent: "#3a3a5c",
            shadow: Some(SHADOW),
        },
        ElementType::ItemDef | ElementType::Item => ElementTheme {
            header_bg: "#dde8f0",
            header_fg: "#1a2030",
            stereotype_fg: "#555",
            body_bg: "#f8fafc",
            body_fg: "#1a2030",
            muted_fg: "#888",
            border: "#3a5070",
            divider: "#ccd8e8",
            accent: "#3a5070",
            shadow: Some(SHADOW),
        },
        ElementType::Requirement | ElementType::RequirementDef => ElementTheme {
            header_bg: "#2d1b54",
            header_fg: "#ffffff",
            stereotype_fg: "#c8b0e8",
            body_bg: "#faf7ff",
            body_fg: "#1a0a30",
            muted_fg: "#888",
            border: "#4a0a8e",
            divider: "#e0d0f8",
            accent: "#7a3ea5",
            shadow: Some(SHADOW),
        },
        ElementType::TestCase => ElementTheme {
            header_bg: "#0f3d1a",
            header_fg: "#ffffff",
            stereotype_fg: "#90e8a8",
            body_bg: "#f4fff6",
            body_fg: "#0a200e",
            muted_fg: "#888",
            border: "#1a6b30",
            divider: "#c8f0d0",
            accent: "#1a6b30",
            shadow: Some(SHADOW),
        },
        ElementType::ADR => ElementTheme {
            header_bg: "#4a3200",
            header_fg: "#ffffff",
            stereotype_fg: "#f0c878",
            body_bg: "#fffbf0",
            body_fg: "#2a1e00",
            muted_fg: "#888",
            border: "#8a6000",
            divider: "#f0e0b0",
            accent: "#8a6000",
            shadow: Some(SHADOW),
        },
        ElementType::Port | ElementType::PortDef => ElementTheme {
            header_bg: "#1a2d5c",
            header_fg: "#ffffff",
            stereotype_fg: "#90b0e8",
            body_bg: "#f4f8ff",
            body_fg: "#0a1530",
            muted_fg: "#888",
            border: "#2a4a8e",
            divider: "#c8d8f8",
            accent: "#2a4a8e",
            shadow: Some(SHADOW),
        },
        ElementType::ActionDef | ElementType::Action => ElementTheme {
            header_bg: "#1a3a1a",
            header_fg: "#ffffff",
            stereotype_fg: "#90d090",
            body_bg: "#f5fff5",
            body_fg: "#0a200a",
            muted_fg: "#888",
            border: "#2a5a2a",
            divider: "#c8e8c8",
            accent: "#2a5a2a",
            shadow: Some(SHADOW),
        },
        ElementType::Allocation | ElementType::AllocationDef => ElementTheme {
            header_bg: "#3a1a3a",
            header_fg: "#ffffff",
            stereotype_fg: "#d090d0",
            body_bg: "#fdf5fd",
            body_fg: "#200a20",
            muted_fg: "#888",
            border: "#6a2a6a",
            divider: "#e8c8e8",
            accent: "#6a2a6a",
            shadow: Some(SHADOW),
        },
        ElementType::InterfaceDef | ElementType::Interface => ElementTheme {
            header_bg: "#1a3a3a",
            header_fg: "#ffffff",
            stereotype_fg: "#90d8d8",
            body_bg: "#f4fdfd",
            body_fg: "#0a2020",
            muted_fg: "#888",
            border: "#2a6a6a",
            divider: "#c8e8e8",
            accent: "#2a6a6a",
            shadow: Some(SHADOW),
        },
        ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace => ElementTheme {
            header_bg: "#e8e8e8",
            header_fg: "#1a1a1a",
            stereotype_fg: "#666",
            body_bg: "#fafafa",
            body_fg: "#1a1a1a",
            muted_fg: "#888",
            border: "#aaa",
            divider: "#ddd",
            accent: "#888",
            shadow: None,
        },
        _ => ElementTheme {
            header_bg: "#2d2d3d",
            header_fg: "#ffffff",
            stereotype_fg: "#aaa",
            body_bg: "#f8f8fb",
            body_fg: "#1a1a2e",
            muted_fg: "#888",
            border: "#555566",
            divider: "#d8d8e8",
            accent: "#555566",
            shadow: Some(SHADOW),
        },
    }
}

pub fn stereotype_label(element_type: &ElementType) -> Option<&'static str> {
    match element_type {
        ElementType::PartDef => Some("part def"),
        ElementType::Part => Some("part"),
        ElementType::ItemDef => Some("item def"),
        ElementType::Item => Some("item"),
        ElementType::PortDef => Some("port def"),
        ElementType::Port => Some("port"),
        ElementType::ConnectionDef => Some("connection def"),
        ElementType::Connection => Some("connection"),
        ElementType::ActionDef => Some("action def"),
        ElementType::Action => Some("action"),
        ElementType::RequirementDef => Some("requirement def"),
        ElementType::Requirement => Some("requirement"),
        ElementType::TestCase => Some("test case"),
        ElementType::ADR => Some("decision"),
        ElementType::InterfaceDef => Some("interface def"),
        ElementType::Interface => Some("interface"),
        ElementType::AllocationDef => Some("allocation def"),
        ElementType::Allocation => Some("allocation"),
        ElementType::Package => Some("package"),
        ElementType::Constraint | ElementType::ConstraintDef => Some("constraint"),
        ElementType::StateDef | ElementType::State => Some("state"),
        ElementType::ViewDef | ElementType::View => Some("view"),
        ElementType::UseCaseDef | ElementType::UseCase => Some("use case"),
        _ => None,
    }
}

pub fn status_badge(status: &str) -> (&'static str, &'static str) {
    match status {
        "draft" => ("#94a3b8", "#1e293b"),
        "review" => ("#3b82f6", "#ffffff"),
        "approved" => ("#22c55e", "#ffffff"),
        "accepted" => ("#22c55e", "#ffffff"),
        "implemented" => ("#10b981", "#ffffff"),
        "verified" => ("#6366f1", "#ffffff"),
        "deprecated" => ("#f97316", "#ffffff"),
        "superseded" => ("#ef4444", "#ffffff"),
        "proposed" => ("#a3a3a3", "#1a1a1a"),
        _ => ("#e5e7eb", "#374151"),
    }
}

pub fn asil_badge(level: &str) -> (&'static str, &'static str) {
    match level {
        "A" | "ASIL-A" => ("#fbbf24", "#1a1a1a"),
        "B" | "ASIL-B" => ("#f97316", "#ffffff"),
        "C" | "ASIL-C" => ("#ef4444", "#ffffff"),
        "D" | "ASIL-D" => ("#b91c1c", "#ffffff"),
        _ => ("#e5e7eb", "#374151"),
    }
}

pub fn sil_badge(level: u8) -> (&'static str, &'static str) {
    match level {
        1 => ("#fbbf24", "#1a1a1a"),
        2 => ("#f97316", "#ffffff"),
        3 => ("#ef4444", "#ffffff"),
        4 => ("#b91c1c", "#ffffff"),
        _ => ("#e5e7eb", "#374151"),
    }
}

pub fn domain_badge(domain: &str) -> (&'static str, &'static str) {
    match domain {
        "system" => ("#6366f1", "#ffffff"),
        "hardware" => ("#f97316", "#ffffff"),
        "software" => ("#3b82f6", "#ffffff"),
        _ => ("#94a3b8", "#1e293b"),
    }
}

pub fn test_level_badge(level: &str) -> (&'static str, &'static str) {
    match level {
        "L1" => ("#e5e7eb", "#374151"),
        "L2" => ("#bfdbfe", "#1e40af"),
        "L3" => ("#6366f1", "#ffffff"),
        "L4" => ("#8b5cf6", "#ffffff"),
        "L5" => ("#7c3aed", "#ffffff"),
        _ => ("#e5e7eb", "#374151"),
    }
}

use crate::diagram::layout::types::ElementTheme;
use syscribe_model::element::ElementType;

pub fn theme_for(element_type: &ElementType) -> ElementTheme {
    match element_type {
        ElementType::PartDef | ElementType::Part => ElementTheme {
            accent: "#3a6ea5",
            header_bg: "#eef3fa",
            header_fg: "#1a2540",
            stereotype_fg: "#4a6ea5",
            body_bg: "#ffffff",
            body_fg: "#1a1a2e",
            muted_fg: "#7a8aaa",
            border: "#a8bcd4",
            divider: "#dde6f0",
        },
        ElementType::ItemDef | ElementType::Item => ElementTheme {
            accent: "#2a6888",
            header_bg: "#edf4f8",
            header_fg: "#0a2030",
            stereotype_fg: "#2a6888",
            body_bg: "#ffffff",
            body_fg: "#0a1e2e",
            muted_fg: "#6a8898",
            border: "#90b8cc",
            divider: "#d0e4ee",
        },
        ElementType::Requirement | ElementType::RequirementDef => ElementTheme {
            accent: "#7040c0",
            header_bg: "#f4effe",
            header_fg: "#2a0a50",
            stereotype_fg: "#7040c0",
            body_bg: "#ffffff",
            body_fg: "#1a0a30",
            muted_fg: "#8a70b0",
            border: "#c0a0e0",
            divider: "#e8d8f8",
        },
        ElementType::TestCase => ElementTheme {
            accent: "#2a7a44",
            header_bg: "#edfbf2",
            header_fg: "#0a2e14",
            stereotype_fg: "#2a7a44",
            body_bg: "#ffffff",
            body_fg: "#0a1e10",
            muted_fg: "#5a8a68",
            border: "#90c8a4",
            divider: "#c8ecd4",
        },
        ElementType::ADR => ElementTheme {
            accent: "#a07010",
            header_bg: "#fdf6e6",
            header_fg: "#2a1a00",
            stereotype_fg: "#a07010",
            body_bg: "#ffffff",
            body_fg: "#1a1200",
            muted_fg: "#8a7030",
            border: "#d4b040",
            divider: "#f0e0a8",
        },
        ElementType::Port | ElementType::PortDef => ElementTheme {
            accent: "#3a5aac",
            header_bg: "#eef1fa",
            header_fg: "#1a2a5c",
            stereotype_fg: "#3a5aac",
            body_bg: "#ffffff",
            body_fg: "#101a3c",
            muted_fg: "#6a7aaa",
            border: "#a0b0d8",
            divider: "#d4daf0",
        },
        ElementType::ActionDef | ElementType::Action => ElementTheme {
            accent: "#306a38",
            header_bg: "#edfaf0",
            header_fg: "#0a2e10",
            stereotype_fg: "#306a38",
            body_bg: "#ffffff",
            body_fg: "#0a1e0a",
            muted_fg: "#5a7a60",
            border: "#90c898",
            divider: "#c8ecd4",
        },
        ElementType::Allocation | ElementType::AllocationDef => ElementTheme {
            accent: "#7a3a7a",
            header_bg: "#f6eef8",
            header_fg: "#280a28",
            stereotype_fg: "#7a3a7a",
            body_bg: "#ffffff",
            body_fg: "#1a0a1a",
            muted_fg: "#8a5a8a",
            border: "#c0a0c0",
            divider: "#e8d0e8",
        },
        ElementType::InterfaceDef | ElementType::Interface => ElementTheme {
            accent: "#2a7a7a",
            header_bg: "#edfafa",
            header_fg: "#0a2e2e",
            stereotype_fg: "#2a7a7a",
            body_bg: "#ffffff",
            body_fg: "#0a1e1e",
            muted_fg: "#5a8a8a",
            border: "#90c8c8",
            divider: "#c8ecec",
        },
        ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace => ElementTheme {
            accent: "#5a5a6a",
            header_bg: "#f2f2f4",
            header_fg: "#1a1a22",
            stereotype_fg: "#5a5a6a",
            body_bg: "#ffffff",
            body_fg: "#1a1a22",
            muted_fg: "#7a7a8a",
            border: "#b0b0c0",
            divider: "#dcdce4",
        },
        _ => ElementTheme {
            accent: "#555566",
            header_bg: "#f2f2f6",
            header_fg: "#1a1a2e",
            stereotype_fg: "#666677",
            body_bg: "#ffffff",
            body_fg: "#1a1a2e",
            muted_fg: "#7a7a8a",
            border: "#b0b0c8",
            divider: "#dcdce8",
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

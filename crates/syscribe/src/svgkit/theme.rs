use crate::svgkit::types::ElementTheme;
use syscribe_model::element::ElementType;

pub fn theme_for(element_type: &ElementType) -> ElementTheme {
    match element_type {
        ElementType::PartDef | ElementType::Part => ElementTheme {
            header_bg: "#eef3fa",
            header_fg: "#1a2540",
            body_bg: "#ffffff",
            body_fg: "#1a1a2e",
            border: "#a8bcd4",
        },
        ElementType::ItemDef | ElementType::Item => ElementTheme {
            header_bg: "#edf4f8",
            header_fg: "#0a2030",
            body_bg: "#ffffff",
            body_fg: "#0a1e2e",
            border: "#90b8cc",
        },
        ElementType::Requirement | ElementType::RequirementDef => ElementTheme {
            header_bg: "#f4effe",
            header_fg: "#2a0a50",
            body_bg: "#ffffff",
            body_fg: "#1a0a30",
            border: "#c0a0e0",
        },
        ElementType::TestCase => ElementTheme {
            header_bg: "#edfbf2",
            header_fg: "#0a2e14",
            body_bg: "#ffffff",
            body_fg: "#0a1e10",
            border: "#90c8a4",
        },
        ElementType::ADR => ElementTheme {
            header_bg: "#fdf6e6",
            header_fg: "#2a1a00",
            body_bg: "#ffffff",
            body_fg: "#1a1200",
            border: "#d4b040",
        },
        ElementType::Port | ElementType::PortDef => ElementTheme {
            header_bg: "#eef1fa",
            header_fg: "#1a2a5c",
            body_bg: "#ffffff",
            body_fg: "#101a3c",
            border: "#a0b0d8",
        },
        ElementType::ActionDef | ElementType::Action => ElementTheme {
            header_bg: "#edfaf0",
            header_fg: "#0a2e10",
            body_bg: "#ffffff",
            body_fg: "#0a1e0a",
            border: "#90c898",
        },
        ElementType::Allocation | ElementType::AllocationDef => ElementTheme {
            header_bg: "#f6eef8",
            header_fg: "#280a28",
            body_bg: "#ffffff",
            body_fg: "#1a0a1a",
            border: "#c0a0c0",
        },
        ElementType::InterfaceDef | ElementType::Interface => ElementTheme {
            header_bg: "#edfafa",
            header_fg: "#0a2e2e",
            body_bg: "#ffffff",
            body_fg: "#0a1e1e",
            border: "#90c8c8",
        },
        ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace => ElementTheme {
            header_bg: "#f2f2f4",
            header_fg: "#1a1a22",
            body_bg: "#ffffff",
            body_fg: "#1a1a22",
            border: "#b0b0c0",
        },
        _ => ElementTheme {
            header_bg: "#f2f2f6",
            header_fg: "#1a1a2e",
            body_bg: "#ffffff",
            body_fg: "#1a1a2e",
            border: "#b0b0c8",
        },
    }
}

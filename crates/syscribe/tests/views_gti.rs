//! `REQ-TRS-VIS-031` (GH #338) on the grid-tie inverter model: the power, protection and firmware views fade or
//! leave out what they are not about, and the variant view of the US M3 and India M2 products marks the arc-fault
//! and rapid-shutdown parts as only the US product's.

use std::path::PathBuf;
use std::process::Command;

fn gti() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../model_gti")
}

fn export(qname: &str, format: &str, extra: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(gti())
        .args(["diagram", "export", qname, "--format", format])
        .args(extra)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn the_power_and_protection_views_fade_what_they_are_not_about() {
    for view in ["Diagrams::GtiPowerView", "Diagrams::GtiProtectionView"] {
        let svg = export(view, "svg", &[]);
        assert!(svg.contains("class=\"faded\"") || svg.contains(" faded"), "{view} fades nothing:\n{svg}");
        assert!(svg.contains("powerStage"), "{view} keeps the whole inverter as context");
    }
}

#[test]
fn the_firmware_view_leaves_the_hardware_out() {
    let mmd = export("Diagrams::GtiSoftwareView", "mermaid", &[]);
    assert!(mmd.contains("firmware"), "{mmd}");
    assert!(mmd.contains("controlboard"), "the board stays as the firmware's frame:\n{mmd}");
    assert!(!mmd.contains("powerStage") && !mmd.contains("faded"), "hidden, not faded:\n{mmd}");
}

#[test]
fn the_variant_view_marks_what_only_the_us_product_has() {
    let mmd = export("Diagrams::GtiIbd", "mermaid", &["--config", "CONF-GTI-US-M3-001", "--config", "CONF-GTI-IN-M2-001"]);
    for part in ["arcFaultFrontEnd", "rapidShutdownDriver"] {
        let line = mmd.lines().find(|l| l.contains(part) && l.contains("[\"")).unwrap_or_else(|| panic!("no {part}:\n{mmd}"));
        assert!(line.contains("only CONF-GTI-US-M3-001"), "{line}");
    }
    assert!(!mmd.contains("only CONF-GTI-IN-M2-001"), "the India M2 product adds no part:\n{mmd}");
    // Mermaid reads a bracket in an unquoted port or edge label as a shape: those labels are quoted
    for l in mmd.lines().filter(|l| l.contains("[only")) {
        assert!(!l.contains("((") || l.contains("((\""), "an unquoted port label: {l}");
        assert!(!l.contains("|[only") && !l.contains("-|[") , "an unquoted edge label: {l}");
    }
}

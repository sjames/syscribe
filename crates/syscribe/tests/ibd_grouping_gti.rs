//! `REQ-TRS-VIS-029` (GH #336) on the grid-tie inverter model: the derived `GtiIbd` draws the firmware inside the
//! STM32 inside the control board, badges the arc-fault and rapid-shutdown parts optional, outlines the safety-rated
//! parts, and `--config CONF-GTI-IN-M2-001` drops the two optional parts.

use std::path::PathBuf;
use std::process::Command;

fn gti() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../model_gti")
}

fn export(extra: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(gti())
        .args(["diagram", "export", "Diagrams::GtiIbd", "--format", "mermaid"])
        .args(extra)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

/// The Mermaid `subgraph` nesting: the ids of the open subgraphs around the line naming `id`.
fn enclosing(mmd: &str, id: &str) -> Vec<String> {
    let mut stack: Vec<String> = Vec::new();
    for line in mmd.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("subgraph ") {
            let sid = rest.split(['[', ' ']).next().unwrap().to_string();
            if sid == id {
                return stack;
            }
            stack.push(sid);
        } else if line == "end" {
            stack.pop();
        }
    }
    panic!("no subgraph {id} in:\n{mmd}");
}

#[test]
fn firmware_sits_in_the_mcu_in_the_control_board() {
    let mmd = export(&[]);
    let around = enclosing(&mmd, "s_system_gridtieinverter_firmware");
    assert!(around.ends_with(&["s_system_gridtieinverter_controlboard".to_string(), "s_system_gridtieinverter_controlboard_mcu".to_string()]), "{around:?}");
}

#[test]
fn optional_and_safety_parts_are_marked_and_the_india_m2_product_drops_the_optional_ones() {
    let mmd = export(&[]);
    assert!(mmd.contains("arcFaultFrontEnd"), "the 150 % picture has the arc-fault part");
    assert!(mmd.contains("optional"), "{mmd}");
    assert!(mmd.contains("SIL 2"), "{mmd}");
    let india = export(&["--config", "CONF-GTI-IN-M2-001"]);
    assert!(!india.contains("arcFaultFrontEnd") && !india.contains("rapidShutdownDriver"), "{india}");
}

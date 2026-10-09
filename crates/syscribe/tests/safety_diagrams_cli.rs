//! GH #223: `safety-case --format dot|mermaid`, `fault-tree render --format …`
//! and `diagram export --format dot` over the shipped `model_auto/` example.
//! The default text outputs are unchanged.

use std::path::Path;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_auto");
    Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().expect("spawn syscribe")
}

fn out(args: &[&str]) -> String {
    let o = run(args);
    assert!(o.status.success(), "{args:?}: {}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn safety_case_default_text_is_unchanged_and_format_selects_diagrams() {
    let text = out(&["safety-case", "SG-ENG-001"]);
    assert!(text.starts_with("[SafetyGoal] SG-ENG-001"), "{text}");
    assert_eq!(out(&["safety-case", "SG-ENG-001", "--format", "text"]), text);
    assert!(out(&["safety-case", "SG-ENG-001", "--json"]).trim_start().starts_with('{'));
    assert!(out(&["safety-case", "SG-ENG-001", "--format", "json"]).trim_start().starts_with('{'));

    let dot = out(&["safety-case", "--format", "dot", "SG-ENG-001"]);
    assert!(dot.starts_with("digraph \"SG-ENG-001\" {"), "{dot}");
    assert!(dot.contains("shape=parallelogram") && dot.contains("__undeveloped") && dot.contains("UNDEVELOPED"), "{dot}");

    let mm = out(&["safety-case", "SG-ENG-001", "--format", "mermaid"]);
    assert!(mm.starts_with("flowchart TD\n") && mm.contains("classDef tone_warn"), "{mm}");
    assert!(mm.contains("-.->"), "InContextOf edges are dashed: {mm}");

    let bad = run(&["safety-case", "--format", "svg"]);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("valid values: text, json, dot, mermaid"));
}

#[test]
fn fault_tree_render_keeps_the_legacy_flowchart_and_format_draws_the_analysed_tree() {
    let legacy = out(&["fault-tree", "render", "FT-ENG-001"]);
    assert!(legacy.starts_with("flowchart TD\n") && legacy.contains("[basic]"), "{legacy}");

    let mm = out(&["fault-tree", "render", "FT-ENG-001", "--format", "mermaid"]);
    assert!(mm.starts_with("flowchart TD\n") && mm.contains("top event") && mm.contains("classDef"), "{mm}");
    let dot = out(&["fault-tree", "render", "--format", "dot", "FT-ENG-001"]);
    assert!(dot.contains("digraph") && dot.contains("top event") && dot.contains("shape=invhouse") || dot.contains("shape=house"), "{dot}");
    let svg = out(&["fault-tree", "render", "FT-ENG-001", "--format", "svg"]);
    assert!(svg.starts_with("<svg") && svg.contains("top event"), "{}", &svg[..svg.len().min(200)]);
    let puml = out(&["fault-tree", "render", "FT-ENG-001", "--format", "plantuml"]);
    assert!(puml.starts_with("@startuml FaultTree") && puml.contains("hexagon"), "{puml}");

    assert!(!run(&["fault-tree", "render", "FT-ENG-001", "--format", "pdf"]).status.success());
    assert!(!run(&["fault-tree", "render", "FT-NOPE-001", "--format", "dot"]).status.success());
}

#[test]
fn diagram_export_writes_dot_for_a_safety_diagram() {
    let dot = out(&["diagram", "export", "Diagrams::AttackTreeTorqueReplay", "--format", "dot"]);
    assert!(dot.starts_with("digraph \"AttackTreeTorqueReplay\"") && dot.contains("feasibility medium"), "{dot}");
    let mm = out(&["diagram", "export", "Diagrams::SafetyCaseEngine", "--format", "mermaid"]);
    assert!(mm.contains("UNDEVELOPED"), "{mm}");
    let p = out(&["diagram", "export", "Diagrams::FaultTreeEngine", "--format", "plantuml"]);
    assert!(p.starts_with("@startuml FaultTreeEngine"), "{p}");
}

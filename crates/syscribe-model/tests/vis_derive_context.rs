//! GH #223 follow-up: the derived safety diagrams read the model root's
//! results sidecar (a GSN test case wears its real verdict) and `[cyber]`
//! configuration (the attack-tree roll-up scores as `W035` does). One test
//! only: the registered model root is process-wide.

use std::path::Path;

use syscribe_model::resolver::Resolver;
use syscribe_model::vis::derive::context::set_model_root;
use syscribe_model::vis::build_graph;
use syscribe_model::walker::walk_model;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn value_of(root: &Path, diagram: &str, node: &str) -> Option<String> {
    let elements = walk_model(root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == diagram).unwrap().clone();
    let (g, _) = build_graph(&d, &elements, &resolver).unwrap();
    g.node(node).unwrap().mark.as_ref().and_then(|m| m.value.clone())
}

#[test]
fn derived_diagrams_read_the_sidecar_verdicts_of_the_registered_root() {
    let root = std::env::temp_dir().join(format!("syscribe-vis-context-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "S/_index.md", "---\ntype: Package\nname: S\n---\n");
    write(&root, "S/SG.md", "---\ntype: SafetyGoal\nid: SG-CX-001\nname: Goal\nstatus: approved\nasilLevel: B\nsafeState: stopped\n---\n\nG.\n");
    write(
        &root,
        "S/REQ.md",
        "---\ntype: Requirement\nid: REQ-CX-001\nname: Req\nstatus: approved\nderivedFromSafetyGoal: SG-CX-001\n---\n\nShall.\n",
    );
    write(
        &root,
        "S/TC.md",
        "---\ntype: TestCase\nid: TC-CX-001\nname: Test\nstatus: active\ntestLevel: L3\nverifies: [REQ-CX-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: the one\n    Given a\n    Then b\n```\n",
    );
    write(&root, "D/_index.md", "---\ntype: Package\nname: D\n---\n");
    write(&root, "D/Gsn.md", "---\ntype: Diagram\nname: Gsn\ndiagramKind: SafetyCase\nsubject: SG-CX-001\n---\n\nGSN.\n");
    set_model_root(&root);

    let tc = "s-s-tc";
    assert_eq!(value_of(&root, "D::Gsn", tc).as_deref(), Some("unknown"), "no sidecar yet");

    write(
        &root,
        ".syscribe/results.json",
        r#"{"schema_version":"1","format":"session-log","source":"x","ingested_at_unix":0,"count":1,"by_leaf":{},"by_scenario":{"TC-CX-001::the one":"fail"}}"#,
    );
    assert_eq!(value_of(&root, "D::Gsn", tc).as_deref(), Some("fail"));
    write(
        &root,
        ".syscribe/results.json",
        r#"{"schema_version":"1","format":"session-log","source":"x","ingested_at_unix":0,"count":1,"by_leaf":{},"by_scenario":{"TC-CX-001::the one":"pass"}}"#,
    );
    assert_eq!(value_of(&root, "D::Gsn", tc).as_deref(), Some("pass"));

    let cfg = syscribe_model::vis::derive::context::cyber_config();
    assert_eq!(cfg, syscribe_model::cyber_config::CyberConfig::load(&root), "the registered root's [cyber] table");
    let _ = std::fs::remove_dir_all(&root);
}

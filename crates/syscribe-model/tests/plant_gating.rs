//! REQ-TRS-FMI-001 (GH #291): with no plant backend registered (a build without the `fmi` feature) the fields
//! parse, no FMI check runs, and one I011 per declaring element says why.

use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

#[test]
fn without_a_backend_the_fields_parse_and_i011_explains() {
    let dir = std::env::temp_dir().join(format!("syscribe-plantgate-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(dir.join("Ball.md"), "---\ntype: Part\nname: Ball\nfmu: {file: ball.fmu, kind: modelExchange, colour: red}\n---\n").unwrap();
    std::fs::write(dir.join("Ctl.md"), "---\ntype: StateDef\nname: Ctl\nsubStates:\n  - {name: A, isInitial: true}\n---\n").unwrap();
    std::fs::write(
        dir.join("TC-FMI-001.md"),
        "---\ntype: TestCase\nid: TC-FMI-001\nname: t\nstatus: active\ntestLevel: L2\nsubject: Ctl\nplants: [Ball]\nplantStep: banana\nplantConnections: [{from: Ball.h, to: nothing}]\n---\n\n```gherkin\nFeature: f\n\n  Scenario: s\n    Given x is 1\n```\n",
    )
    .unwrap();
    let elems = walk_model(&dir).unwrap();
    let f: Vec<String> = {
        let r = validate_with_config(&elems, &ValidateConfig::with_model_root(&dir));
        r.errors().chain(r.warnings()).chain(r.infos()).map(|f| format!("{} {}", f.code, f.message)).collect()
    };
    assert_eq!(f.iter().filter(|x| x.starts_with("I011")).count(), 2, "one per declaring element: {f:?}");
    assert!(f.iter().any(|x| x.starts_with("I011") && x.contains("--features fmi")), "{f:?}");
    for c in ["E888", "E889", "E966", "W047"] {
        assert!(!f.iter().any(|x| x.starts_with(c)), "{c} must not run or fire without the backend: {f:?}");
    }
}

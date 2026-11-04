//! TC-TRS-MXGAP-001 (GH #424): the coverage matrix names what to do about each gap.

mod common;

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(model: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

fn matrix(model: &Path) -> Value {
    let (o, e, c) = run(model, &["matrix", "--json"]);
    assert_eq!(c, 0, "{e}");
    let v: Value = serde_json::from_str(&o).unwrap();
    let s = Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["schema", "matrix"]).output().unwrap();
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let url = "https://syscribe.dev/schemas/matrix.schema.json";
    compiler.add_resource(url, serde_json::from_slice(&s.stdout).unwrap()).unwrap();
    let idx = compiler.compile(url, &mut schemas).unwrap();
    schemas.validate(&v, idx).unwrap_or_else(|e| panic!("{e:#}\n{v}"));
    v
}

fn req(id: &str, extra: &str) -> String {
    format!("---\nid: {id}\ntype: Requirement\nname: R {id}\nstatus: approved\nreqDomain: system\n{extra}---\n\nbody\n")
}
fn tc(id: &str, verifies: &str, status: &str, extra: &str) -> String {
    format!("---\nid: {id}\ntype: TestCase\nname: T {id}\nstatus: {status}\ntestLevel: L4\nverifies: [{verifies}]\n{extra}---\n\nbody\n")
}

/// Features Alpha; C1 selects it, C2 does not. REQ-MG-AAA-001 has no test; -002 only a draft; -003 a test that fails;
/// -004 applies everywhere but its test only when Alpha is on.
fn model(with_features: bool) -> PathBuf {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-mxgap-{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    let w = |rel: &str, c: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    if with_features {
        w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
        w("Features/Alpha.md", "---\nid: FEAT-MG-ALPHA\ntype: FeatureDef\nname: Alpha\n---\n\na\n");
        w("Configurations/_index.md", "---\ntype: Package\nname: Configurations\n---\n");
        w("Configurations/CONF-MG-ONE.md", "---\nid: CONF-MG-ONE\ntype: Configuration\nname: One\nfeatures:\n  FEAT-MG-ALPHA: true\n---\n\nc\n");
        w("Configurations/CONF-MG-TWO.md", "---\nid: CONF-MG-TWO\ntype: Configuration\nname: Two\nfeatures:\n  FEAT-MG-ALPHA: false\n---\n\nc\n");
    }
    w("REQ-MG-AAA-001.md", &req("REQ-MG-AAA-001", ""));
    w("REQ-MG-AAA-002.md", &req("REQ-MG-AAA-002", ""));
    w("TC-MG-AAA-001.md", &tc("TC-MG-AAA-001", "REQ-MG-AAA-002", "draft", ""));
    w("REQ-MG-AAA-003.md", &req("REQ-MG-AAA-003", ""));
    w("TC-MG-AAA-002.md", &tc("TC-MG-AAA-002", "REQ-MG-AAA-003", "active", ""));
    w("REQ-MG-AAA-004.md", &req("REQ-MG-AAA-004", ""));
    w("TC-MG-AAA-003.md", &tc("TC-MG-AAA-003", "REQ-MG-AAA-004", "active", if with_features { "appliesWhen: Features::Alpha\n" } else { "" }));
    root
}

fn gap<'a>(m: &'a Value, req: &str, conf: Option<&str>) -> &'a Value {
    m["gaps"].as_array().unwrap().iter().find(|g| g["requirement"] == req && g["configuration"].as_str() == conf).unwrap_or_else(|| panic!("no gap {req} {conf:?} in {}", m["gaps"]))
}

#[test]
fn each_gap_cell_says_what_to_do_and_with_which_ids() {
    let m = model(true);
    // A failing result for TC-MG-AAA-002 (global), so REQ-003 is `failing` wherever it applies.
    std::fs::write(m.join("r.json"), r#"[{"testCase":"TC-MG-AAA-002","verdict":"fail"}]"#).unwrap();
    let (o, e, c) = run(&m, &["ingest-results", m.join("r.json").to_str().unwrap(), "--format", "records"]);
    assert_eq!(c, 0, "{o}{e}");
    let x = matrix(&m);
    // No test at all: create one, and here is the id and the file of the requirement.
    let g = gap(&x, "REQ-MG-AAA-001", Some("CONF-MG-ONE"));
    assert_eq!((g["state"].as_str(), g["action"].as_str()), (Some("gap"), Some("create-test")));
    assert!(g["tests"].as_array().unwrap().is_empty());
    assert_eq!(g["suggestedTestId"], "TC-MG-AAA-004", "area MG-AAA, after the three existing tests");
    assert!(g["requirementFile"].as_str().unwrap().ends_with("REQ-MG-AAA-001.md"));
    // Only a draft test: activate it, and it is named with its file.
    let g = gap(&x, "REQ-MG-AAA-002", Some("CONF-MG-TWO"));
    assert_eq!(g["action"], "activate-test");
    assert_eq!(g["tests"][0]["id"], "TC-MG-AAA-001");
    assert_eq!(g["tests"][0]["status"], "draft");
    assert!(g["tests"][0]["file"].as_str().unwrap().ends_with("TC-MG-AAA-001.md"));
    assert!(g.get("suggestedTestId").is_none());
    // A failing test.
    let g = gap(&x, "REQ-MG-AAA-003", Some("CONF-MG-ONE"));
    assert_eq!((g["state"].as_str(), g["action"].as_str()), (Some("failing"), Some("fix-failing-test")));
    // The test exists but does not apply in the second configuration: widen it.
    let g = gap(&x, "REQ-MG-AAA-004", Some("CONF-MG-TWO"));
    assert_eq!(g["action"], "widen-test");
    assert_eq!(g["tests"][0]["id"], "TC-MG-AAA-003");
    assert!(x["gaps"].as_array().unwrap().iter().all(|g| !(g["requirement"] == "REQ-MG-AAA-004" && g["configuration"] == "CONF-MG-ONE")), "covered cells are not gaps");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn the_flat_view_without_a_feature_model_also_lists_gaps() {
    let m = model(false);
    let x = matrix(&m);
    assert!(x["gaps"].as_array().unwrap().iter().all(|g| g["configuration"].is_null()));
    let g = gap(&x, "REQ-MG-AAA-001", None);
    assert_eq!(g["action"], "create-test");
    // A draft-only requirement in the flat view has a test to activate rather than create.
    let g = gap(&x, "REQ-MG-AAA-002", None);
    assert_eq!(g["action"], "activate-test", "{g}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn the_mcp_tool_filters_gaps_to_one_configuration() {
    let m = model(true);
    let mut mcp = common::Mcp::start(&m);
    mcp.initialize();
    let all = mcp.call_tool("coverage_matrix", serde_json::json!({}));
    assert_eq!(all, matrix(&m));
    let one = mcp.call_tool("coverage_matrix", serde_json::json!({"config": "CONF-MG-TWO"}));
    let gaps = one["gaps"].as_array().unwrap();
    assert!(!gaps.is_empty() && gaps.iter().all(|g| g["configuration"] == "CONF-MG-TWO"), "{one}");
    let _ = std::fs::remove_dir_all(&m);
}

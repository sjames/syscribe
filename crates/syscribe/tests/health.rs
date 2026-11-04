//! TC-TRS-HEALTH-001 (GH #428): `health`, the status-bar summary.

mod common;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(model: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

fn health(m: &Path) -> Value {
    let (o, e, c) = run(m, &["health", "--json"]);
    assert_eq!(c, 0, "{e}");
    let v: Value = serde_json::from_str(&o).unwrap();
    validate(&v);
    v
}

fn validate(v: &Value) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["schema", "health"]).output().unwrap();
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let url = "https://syscribe.dev/schemas/health.schema.json";
    compiler.add_resource(url, serde_json::from_slice(&o.stdout).unwrap()).unwrap();
    let idx = compiler.compile(url, &mut schemas).unwrap();
    schemas.validate(v, idx).unwrap_or_else(|e| panic!("{e:#}\n{v}"));
}

/// Requirements: one verified, one with no test; an unrelated broken file; a Configuration that violates a `requires`.
fn model(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-health-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let w = |rel: &str, c: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("REQ-HL-001.md", "---\nid: REQ-HL-001\ntype: Requirement\nname: Verified\nstatus: approved\nreqDomain: system\n---\n\nThe system shall be verified.\n");
    w("REQ-HL-002.md", "---\nid: REQ-HL-002\ntype: Requirement\nname: Unverified\nstatus: approved\nreqDomain: system\n---\n\nThe system shall be checked.\n");
    w("TC-HL-001.md", "---\nid: TC-HL-001\ntype: TestCase\nname: T\nstatus: active\ntestLevel: L2\nverifies: [REQ-HL-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/Alpha.md", "---\nid: FEAT-HL-ALPHA\ntype: FeatureDef\nname: Alpha\nrequires: [Features::Beta]\n---\n\na\n");
    w("Features/Beta.md", "---\nid: FEAT-HL-BETA\ntype: FeatureDef\nname: Beta\n---\n\nb\n");
    w("Configurations/_index.md", "---\ntype: Package\nname: Configurations\n---\n");
    w("Configurations/CONF-HL-BAD.md", "---\nid: CONF-HL-BAD\ntype: Configuration\nname: Bad\nstatus: approved\nfeatureModel: Features\nfeatures:\n  FEAT-HL-ALPHA: true\n  FEAT-HL-BETA: false\n---\n\nc\n");
    w("Configurations/CONF-HL-OK.md", "---\nid: CONF-HL-OK\ntype: Configuration\nname: Ok\nstatus: approved\nfeatureModel: Features\nfeatures:\n  FEAT-HL-ALPHA: true\n  FEAT-HL-BETA: true\n---\n\nc\n");
    root
}

#[test]
fn counts_and_click_through() {
    let m = model("counts");
    let h = health(&m);
    // A Configuration that violates the feature model is an error for the project even though `validate` itself is clean.
    assert_eq!((h["status"].clone(), h["errors"].clone()), (json!("error"), json!(0)), "{h}");
    assert_eq!(h["requirements"]["total"], 2);
    assert_eq!(h["requirements"]["orphan"], 2, "neither requirement derives from or is refined by another");
    assert_eq!(h["requirements"]["unverified"], 1);
    assert_eq!(h["configurations"], json!({"total": 2, "failingValidation": 1}));
    let metrics: Vec<&str> = h["clickThrough"].as_array().unwrap().iter().map(|c| c["metric"].as_str().unwrap()).collect();
    for want in ["warnings", "unverifiedRequirements", "failingConfigurations"] {
        assert!(metrics.contains(&want), "{want} in {metrics:?}");
    }
    let c = h["clickThrough"].as_array().unwrap().iter().find(|c| c["metric"] == "failingConfigurations").unwrap();
    assert_eq!((c["count"].clone(), c["command"].clone()), (json!(1), json!("syscribe validate --all-configs")));
    assert!(h["topCodes"].as_array().unwrap().len() <= 5 && h["elapsedMs"].is_u64());
    let (text, _e, code) = run(&m, &["health"]);
    assert_eq!(code, 0);
    assert!(text.contains("Project health: error") && text.contains("1 failing validation") && text.contains("1 unverified"), "{text}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn validation_errors_are_counted_and_listed_first() {
    let m = model("errs");
    std::fs::write(m.join("TC-HL-009.md"), "---\nid: TC-HL-009\ntype: TestCase\nname: Broken\nstatus: nonsense\ntestLevel: L2\nverifies: [REQ-HL-001]\n---\n\nb\n").unwrap();
    let h = health(&m);
    assert!(h["errors"].as_u64().unwrap() >= 1, "{h}");
    assert_eq!(h["clickThrough"][0], json!({"metric": "errors", "count": h["errors"], "command": "syscribe validate"}));
    assert!(h["topCodes"].as_array().unwrap().iter().any(|c| c["code"] == "E007"), "{h}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn fixing_the_problems_clears_the_metrics() {
    let m = model("fix");
    std::fs::write(m.join("Configurations/CONF-HL-BAD.md"), std::fs::read_to_string(m.join("Configurations/CONF-HL-BAD.md")).unwrap().replace("FEAT-HL-BETA: false", "FEAT-HL-BETA: true")).unwrap();
    std::fs::write(m.join("TC-HL-002.md"), "---\nid: TC-HL-002\ntype: TestCase\nname: T2\nstatus: active\ntestLevel: L2\nverifies: [REQ-HL-002]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n").unwrap();
    let h = health(&m);
    assert_eq!((h["requirements"]["unverified"].clone(), h["configurations"]["failingValidation"].clone()), (json!(0), json!(0)), "{h}");
    assert_eq!(h["errors"], 0, "{h}");
    assert_ne!(h["status"], "error");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn it_stays_cheap_on_the_largest_example() {
    let gti = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../model_gti");
    let started = std::time::Instant::now();
    let h = health(&gti);
    // Budget: a few seconds in a debug build (about 1.4 s measured); a status bar polls the cached answer instead.
    assert!(started.elapsed() < std::time::Duration::from_secs(8), "took {:?}", started.elapsed());
    assert_eq!(h["configurations"]["total"], 4);
    assert_eq!(h["errors"], 0);
}

#[test]
fn mcp_answers_are_kept_per_model_version() {
    let m = model("mcp");
    let cli = health(&m);
    let mut mcp = common::Mcp::start(&m);
    mcp.initialize();
    let first = mcp.call_tool("health", json!({}));
    validate(&first);
    assert_eq!(first["cached"], false);
    assert_eq!((first["errors"].clone(), first["requirements"].clone(), first["configurations"].clone()), (cli["errors"].clone(), cli["requirements"].clone(), cli["configurations"].clone()));
    let second = mcp.call_tool("health", json!({}));
    assert_eq!(second["cached"], true);
    assert_eq!(second["elapsedMs"], first["elapsedMs"], "the cost of the call that computed it");
    // A change to the model is a new version: the answer is recomputed.
    std::fs::write(m.join("TC-HL-002.md"), "---\nid: TC-HL-002\ntype: TestCase\nname: T2\nstatus: active\ntestLevel: L2\nverifies: [REQ-HL-002]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n").unwrap();
    mcp.call_tool("reload", json!({}));
    let third = mcp.call_tool("health", json!({}));
    assert_eq!(third["cached"], false, "{third}");
    assert_eq!(third["requirements"]["unverified"], 0);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn the_lsp_request_is_cached_until_a_reload() {
    let m = model("lsp");
    let mut lsp = common::Lsp::start(&m);
    let init = lsp.initialize();
    assert!(init["capabilities"]["experimental"]["syscribe"]["lspRequests"].as_array().unwrap().iter().any(|r| r == "syscribe/health"));
    let first = lsp.custom_request("syscribe/health", json!({}));
    validate(&first);
    assert_eq!(first["cached"], false);
    assert_eq!(first["requirements"]["unverified"], 1);
    assert_eq!(lsp.custom_request("syscribe/health", json!({}))["cached"], true);
    let tc = m.join("TC-HL-002.md");
    std::fs::write(&tc, "---\nid: TC-HL-002\ntype: TestCase\nname: T2\nstatus: active\ntestLevel: L2\nverifies: [REQ-HL-002]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n").unwrap();
    lsp.did_save(&tc, None);
    let after = lsp.custom_request("syscribe/health", json!({}));
    assert_eq!((after["cached"].clone(), after["requirements"]["unverified"].clone()), (json!(false), json!(0)), "{after}");
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn usage() {
    let m = model("usage");
    assert_eq!(run(&m, &["health", "--nope"]).2, 1, "shared option check");
    let _ = std::fs::remove_dir_all(&m);
}

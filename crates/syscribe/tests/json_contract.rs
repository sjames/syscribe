//! TC-TRS-CONTRACT-001 / TC-TRS-CAPS-001 (GH #413, #414): the published JSON contracts and the capabilities handshake.

mod common;

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn repo(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel)
}

fn run(model: Option<&PathBuf>, args: &[&str]) -> (String, String, i32) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_syscribe"));
    if let Some(m) = model {
        c.arg("-m").arg(m);
    }
    let o = c.args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

/// Validate `output` against the published schema `name`, panicking with the validator's message.
fn assert_valid(name: &str, output: &str) -> Value {
    let v: Value = serde_json::from_str(output).unwrap_or_else(|x| panic!("{name}: output is not JSON: {x}\n{output}"));
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let (text, _e, _c) = run(None, &["schema", name]);
    let url = format!("https://syscribe.dev/schemas/{name}.schema.json");
    compiler.add_resource(&url, serde_json::from_str(&text).unwrap()).unwrap();
    let idx = compiler.compile(&url, &mut schemas).unwrap();
    if let Err(e) = schemas.validate(&v, idx) {
        panic!("{name}: output does not match its schema:\n{e:#}\n{output}");
    }
    assert_eq!(v["schemaVersion"], "1.0", "{name} carries schemaVersion");
    v
}

#[test]
fn every_published_contract_matches_the_real_output() {
    let gti = repo("model_gti");
    let conf = "CONF-GTI-US-M3-001";
    let other = "CONF-GTI-IN-M3-001";
    assert_valid("features", &run(Some(&gti), &["features", "--json"]).0);
    let f = run(Some(&gti), &["features", "--json"]).0;
    let first = serde_json::from_str::<Value>(&f).unwrap()["features"][0]["qualifiedName"].as_str().unwrap().to_string();
    assert_valid("feature", &run(Some(&gti), &["feature", &first, "--json"]).0);
    assert_valid("configure", &run(Some(&gti), &["configure", conf, "--json"]).0);
    assert_valid("residual", &run(Some(&gti), &["residual", "--config", conf, "--json"]).0);
    assert_valid("matrix", &run(Some(&gti), &["matrix", "--json"]).0);
    assert_valid("diff", &run(Some(&gti), &["diff", "--config", conf, "--config", other, "--json"]).0);
    assert_valid("feature-check", &run(Some(&gti), &["feature-check", "--json"]).0);
    assert_valid("feature-check", &run(Some(&gti), &["feature-check", "--deep", "--json"]).0);
}

#[test]
fn residual_and_params_check_with_open_parameters_match_their_schemas() {
    let staged = repo("examples/staged-configuration/model");
    let conf = "CONF-UAV-BUILD-001";
    let r = assert_valid("residual", &run(Some(&staged), &["residual", "--config", conf, "--json"]).0);
    assert!(!r["groups"].as_array().unwrap().is_empty() && !r["constraints"].as_array().unwrap().is_empty(), "the fixture exercises groups and constraints");
    assert_valid("configure", &run(Some(&staged), &["configure", conf, "--json"]).0);

    let dir = std::env::temp_dir().join(format!("syscribe-contract-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("params.yaml");
    let (_o, e, c) = run(Some(&staged), &["params", "export", "--config", conf, "--out", file.to_str().unwrap(), "--allow-open"]);
    assert_eq!(c, 0, "{e}");
    let (o, e, code) = run(Some(&staged), &["params", "check", file.to_str().unwrap(), "--config", conf, "--json"]);
    let v = assert_valid("params-check", &o);
    assert_eq!(v["ok"] == true, code == 0, "ok mirrors the exit code ({code}): {e}");
    // A file with an unknown key is reported as an error, with exit 1, in the same document.
    std::fs::write(&file, "Features::Nope.x: 1\n").unwrap();
    let (o, _e, code) = run(Some(&staged), &["params", "check", file.to_str().unwrap(), "--config", conf, "--json"]);
    let v = assert_valid("params-check", &o);
    assert_eq!(code, 1);
    assert_eq!(v["ok"], false);
    assert!(v["errors"].as_u64().unwrap() >= 1 && !v["findings"].as_array().unwrap().is_empty(), "{v}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_schemas_reject_a_wrong_document() {
    // A validator that accepts anything proves nothing: a document missing required keys, or with a wrong type, fails.
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let (text, _e, _c) = run(None, &["schema", "residual"]);
    let url = "https://syscribe.dev/schemas/residual.schema.json";
    compiler.add_resource(url, serde_json::from_str(&text).unwrap()).unwrap();
    let idx = compiler.compile(url, &mut schemas).unwrap();
    for bad in [
        serde_json::json!({}),
        serde_json::json!({"schemaVersion": "2.0", "configuration": "C", "bindingStage": null, "groups": [], "constraints": [], "reliances": [], "retained": []}),
        serde_json::json!({"schemaVersion": "1.0", "configuration": 7, "bindingStage": null, "groups": [], "constraints": [], "reliances": [], "retained": []}),
        serde_json::json!({"schemaVersion": "1.0", "configuration": "C", "bindingStage": null, "groups": [{"stage": null, "closed": "no", "items": []}], "constraints": [], "reliances": [], "retained": []}),
    ] {
        assert!(schemas.validate(&bad, idx).is_err(), "should be rejected: {bad}");
    }
    // An unknown extra property is allowed: a compatible addition must not break a client.
    let ok = serde_json::json!({"schemaVersion": "1.0", "configuration": "C", "bindingStage": null, "groups": [], "constraints": [], "reliances": [], "retained": [], "newField": 1});
    assert!(schemas.validate(&ok, idx).is_ok());
}

#[test]
fn schema_command_lists_prints_and_refuses() {
    let (o, _e, c) = run(None, &["schema"]);
    assert_eq!(c, 0);
    for n in ["features", "feature", "configure", "residual", "matrix", "diff", "feature-check", "params-check"] {
        assert!(o.contains(&format!("| {n} |")), "{n} listed: {o}");
        let (s, _e, c) = run(None, &["schema", n]);
        assert_eq!(c, 0);
        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["properties"]["schemaVersion"]["const"], "1.0", "{n}");
    }
    let (_o, e, c) = run(None, &["schema", "residul"]);
    assert_eq!(c, 1);
    assert!(e.contains("residual") && e.contains("unknown contract"), "{e}");
}

#[test]
fn capabilities_describe_the_installed_binary() {
    let (o, _e, c) = run(None, &["capabilities", "--json"]);
    assert_eq!(c, 0);
    let v: Value = serde_json::from_str(&o).unwrap();
    assert_eq!(v["schemaVersion"], "1.0");
    assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
    // Every published contract is advertised with the version the schema pins, and its command exists.
    for (name, info) in v["jsonSchemas"].as_object().unwrap() {
        assert_eq!(info["version"], "1.0", "{name}");
        assert_eq!(info["available"], true, "{name}: {info}");
    }
    let cmds: Vec<&str> = v["commands"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap()).collect();
    for want in ["residual", "configure", "params", "matrix", "schema", "capabilities", "context"] {
        assert!(cmds.contains(&want), "{want}");
    }
    let matrix = v["commands"].as_array().unwrap().iter().find(|c| c["name"] == "matrix").unwrap();
    assert!(matrix["options"].as_array().unwrap().iter().any(|o| o == "--by"), "{matrix}");
    for f in ["stagedConfiguration", "paramsExport", "variantDiff", "coverageMatrix", "environmentMatrix", "assistedConfiguration", "featureModel"] {
        assert_eq!(v["features"][f], true, "{f}");
    }
    assert!(v["mcpTools"].as_array().unwrap().iter().any(|t| t == "coverage_matrix"));
    assert!(v["lspRequests"].as_array().unwrap().is_empty() || v["lspRequests"].is_array());
    // Needs no model and works from anywhere; the text form summarises.
    let (t, _e, c) = run(None, &["capabilities"]);
    assert_eq!(c, 0);
    assert!(t.contains("stagedConfiguration: yes") || t.contains("- stagedConfiguration: yes"), "{t}");
}

#[test]
fn the_lsp_initialize_result_carries_the_same_document() {
    let model = common::fixture_copy();
    let mut lsp = common::Lsp::start(&model);
    let init = lsp.initialize();
    let doc = &init["capabilities"]["experimental"]["syscribe"];
    let (o, _e, _c) = run(None, &["capabilities", "--json"]);
    let cli: Value = serde_json::from_str(&o).unwrap();
    assert_eq!(doc, &cli);
    lsp.shutdown();
}

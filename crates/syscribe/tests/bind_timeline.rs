//! TC-TRS-BIND-001 / TC-TRS-TIMELINE-001 (GH #418): `bind` (checked, guarded) and `timeline`.

mod common;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(model: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

fn validate(name: &str, v: &Value) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["schema", name]).output().unwrap();
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let url = format!("https://syscribe.dev/schemas/{name}.schema.json");
    compiler.add_resource(&url, serde_json::from_slice(&o.stdout).unwrap()).unwrap();
    let idx = compiler.compile(&url, &mut schemas).unwrap();
    schemas.validate(v, idx).unwrap_or_else(|e| panic!("{e:#}\n{v}"));
}

/// A writable copy of the staged-configuration example.
fn staged(tag: &str) -> PathBuf {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model");
    let dst = std::env::temp_dir().join(format!("syscribe-bind-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dst);
    copy(&src, &dst);
    dst
}
fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy(&e.path(), &p);
        } else {
            std::fs::copy(e.path(), p).unwrap();
        }
    }
}

fn bind(m: &Path, args: &[&str]) -> (Value, i32) {
    let mut a = vec!["bind"];
    a.extend_from_slice(args);
    a.push("--json");
    let (o, e, c) = run(m, &a);
    let v: Value = serde_json::from_str(&o).unwrap_or_else(|x| panic!("{x}: {o} / {e}"));
    validate("bind", &v);
    (v, c)
}

fn codes(v: &Value, k: &str) -> Vec<String> {
    v[k].as_array().unwrap().iter().map(|e| e["code"].as_str().unwrap().to_string()).collect()
}

#[test]
fn timeline_shows_what_each_stage_decided_with_provenance() {
    let m = staged("tl");
    let (o, e, c) = run(&m, &["timeline", "CONF-UAV-DEPLOY-001", "--json"]);
    assert_eq!(c, 0, "{e}");
    let v: Value = serde_json::from_str(&o).unwrap();
    validate("timeline", &v);
    let chain: Vec<&str> = v["chain"].as_array().unwrap().iter().map(|s| s["configuration"].as_str().unwrap()).collect();
    assert_eq!(chain, ["CONF-UAV-BUILD-001", "CONF-UAV-DEPLOY-001"], "root first");
    assert_eq!(v["chain"][0]["bindingStage"], "compile");
    assert_eq!(v["chain"][1]["bindingStage"], "load");
    let deploy = &v["chain"][1]["decided"];
    assert!(deploy["bindings"].to_string().contains("Features::Propulsion.motorKV"), "{deploy}");
    assert_eq!(v["chain"][0]["decided"]["bindings"], json!([]), "the build step bound no parameter");
    // Provenance per value, with the stage that bound it.
    let p = v["effective"]["parameters"].as_array().unwrap().iter().find(|p| p["key"] == "Features::Propulsion.motorKV").unwrap();
    assert_eq!((p["source"]["kind"].as_str(), p["stage"].as_str(), p["value"].as_i64()), (Some("own"), Some("load"), Some(1000)));
    let s = v["effective"]["selections"].as_array().unwrap().iter().find(|s| s["feature"] == "Features::Propulsion").unwrap();
    assert_eq!((s["source"]["kind"].as_str(), s["source"]["from"].as_str(), s["stage"].as_str()), (Some("inherited"), Some("CONF-UAV-BUILD-001"), Some("compile")));
    // What is still open is by stage.
    assert!(v["open"].to_string().contains("rateHz"), "{}", v["open"]);
    // The build configuration alone leaves the motor constant open.
    let b: Value = serde_json::from_str(&run(&m, &["timeline", "CONF-UAV-BUILD-001", "--json"]).0).unwrap();
    assert_eq!(b["chain"].as_array().unwrap().len(), 1);
    assert!(b["open"].to_string().contains("motorKV"));
    let (text, _e, c) = run(&m, &["timeline", "CONF-UAV-DEPLOY-001"]);
    assert_eq!(c, 0);
    assert!(text.contains("## CONF-UAV-BUILD-001 — stage compile") && text.contains("Still open"), "{text}");
    assert_eq!(run(&m, &["timeline", "CONF-NOPE"]).2, 1);
    assert_eq!(run(&m, &["timeline"]).2, 2);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_clean_binding_is_previewed_then_written() {
    let m = staged("ok");
    let before = common::dir_hash(&m);
    let (v, c) = bind(&m, &["CONF-UAV-DEPLOY-001", "Features::Propulsion.motorKV", "1100", "--dry-run"]);
    assert_eq!(c, 0);
    assert_eq!((v["written"].clone(), v["refused"].clone()), (json!(false), json!(null)));
    assert_eq!(v["target"], json!({"kind": "parameter", "key": "Features::Propulsion.motorKV"}));
    assert_eq!((v["previous"]["value"].clone(), v["previous"]["source"].clone()), (json!(1000), json!("own")));
    assert!(v["diff"].as_str().unwrap().contains("-  Features::Propulsion.motorKV: 1000") && v["diff"].as_str().unwrap().contains("+  Features::Propulsion.motorKV: 1100"), "{}", v["diff"]);
    assert_eq!(common::dir_hash(&m), before, "a dry run writes nothing");
    let (v, c) = bind(&m, &["CONF-UAV-DEPLOY-001", "Features::Propulsion.motorKV", "1100"]);
    assert_eq!((c, v["written"].clone()), (0, json!(true)));
    let file = std::fs::read_to_string(m.join("Configurations/CONF-UAV-DEPLOY-001.md")).unwrap();
    assert!(file.contains("motorKV: 1100") && !file.contains("motorKV: 1000"), "{file}");
    // The result is what the timeline now says, and the model still validates.
    let t: Value = serde_json::from_str(&run(&m, &["timeline", "CONF-UAV-DEPLOY-001", "--json"]).0).unwrap();
    assert!(t["effective"]["parameters"].to_string().contains("1100"));
    assert_eq!(run(&m, &["validate"]).2, 0);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn the_models_own_checks_refuse_a_bad_binding_and_nothing_is_written() {
    let m = staged("bad");
    let before = common::dir_hash(&m);
    for (conf, target, value, code) in [
        ("CONF-UAV-DEPLOY-001", "Features::Propulsion.motorKV", "5000", "E205"), // outside the range
        ("CONF-UAV-DEPLOY-001", "Features::Propulsion.motorKV", "1150", "E756"), // inside the range, outside what the chain narrowed it to
        ("CONF-UAV-BUILD-001", "Features::Propulsion.maxCurrentA", "20", "E752"), // not bindable at this stage
        ("CONF-UAV-DEPLOY-001", "Features::Telemetry.mode", "fancy", "E206"),   // not in enumValues
    ] {
        let (v, c) = bind(&m, &[conf, target, value]);
        assert_eq!(c, 1, "{target}={value}: {v}");
        assert_eq!(v["written"], false);
        assert!(v["refused"].as_str().unwrap().contains("would introduce"), "{v}");
        assert!(codes(&v, "newErrors").contains(&code.to_string()), "{target}={value}: {:?}", codes(&v, "newErrors"));
        let c = codes(&v, "newErrors");
        assert_eq!(c.len(), c.iter().collect::<std::collections::BTreeSet<_>>().len(), "each finding once: {c:?}");
    }
    assert_eq!(common::dir_hash(&m), before);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_selection_is_bound_in_the_spelling_the_file_already_uses() {
    let m = staged("sel");
    let path = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    let text = std::fs::read_to_string(&path).unwrap().replace("Features::Telemetry: true", "FEAT-UAV-TELEM: true");
    std::fs::write(&path, text).unwrap();
    let (v, c) = bind(&m, &["CONF-UAV-DEPLOY-001", "Features::Telemetry", "true"]);
    assert_eq!(c, 0, "{v}");
    let file = std::fs::read_to_string(&path).unwrap();
    assert_eq!(file.matches("Telemetry: true").count() + file.matches("TELEM: true").count(), 1, "no duplicate key: {file}");
    assert!(file.contains("FEAT-UAV-TELEM: true") && !file.contains("Features::Telemetry: true"), "the existing spelling is kept: {file}");
    assert_eq!(v["target"]["kind"], "selection");
    // Turning a selection off while its parameters are still bound is the model's own error, and is refused.
    let (v, c) = bind(&m, &["CONF-UAV-DEPLOY-001", "Features::Telemetry", "false"]);
    assert_eq!(c, 1, "{v}");
    assert!(std::fs::read_to_string(&path).unwrap().contains("FEAT-UAV-TELEM: true"));
    // Only the changed line differs: the file's other lines, quoting included, are untouched.
    let (v, _) = bind(&m, &["CONF-UAV-DEPLOY-001", "Features::Propulsion.motorKV", "1100", "--dry-run"]);
    let changed: Vec<&str> = v["diff"].as_str().unwrap().lines().filter(|l| (l.starts_with('+') || l.starts_with('-')) && !l.starts_with("+++") && !l.starts_with("---")).collect();
    assert_eq!(changed.len(), 2, "{changed:?}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn unknown_targets_and_bad_values_are_refused_up_front() {
    let m = staged("err");
    let (_o, e, c) = run(&m, &["bind", "CONF-UAV-DEPLOY-001", "Features::Propulsion.motrKV", "1"]);
    assert_eq!(c, 1);
    assert!(e.contains("has no parameter") && e.contains("motorKV"), "{e}");
    let (_o, e, c) = run(&m, &["bind", "CONF-UAV-DEPLOY-001", "Features::Nope", "true"]);
    assert_eq!(c, 1);
    assert!(e.contains("is not a feature"), "{e}");
    assert_eq!(run(&m, &["bind", "CONF-NOPE", "Features::Telemetry", "true"]).2, 1);
    assert_eq!(run(&m, &["bind", "CONF-UAV-DEPLOY-001", "Features::Telemetry", "maybe"]).2, 2, "a selection is true or false");
    assert_eq!(run(&m, &["bind", "CONF-UAV-DEPLOY-001", "Features::Telemetry"]).2, 2);
    assert_eq!(run(&m, &["bind", "CONF-UAV-DEPLOY-001", "Features::Telemetry", "true", "--nope"]).2, 1, "shared option check");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn the_mcp_tools_and_the_lsp_request() {
    let m = staged("surf");
    let cli: Value = serde_json::from_str(&run(&m, &["timeline", "CONF-UAV-DEPLOY-001", "--json"]).0).unwrap();
    let mut mcp = common::Mcp::start(&m);
    mcp.initialize();
    assert_eq!(mcp.call_tool("timeline", json!({"configuration": "CONF-UAV-DEPLOY-001"})), cli);
    // bind previews by default ...
    let before = common::dir_hash(&m);
    let p = mcp.call_tool("bind", json!({"configuration": "CONF-UAV-DEPLOY-001", "target": "Features::Propulsion.motorKV", "value": 1100}));
    assert_eq!(p["written"], false, "{p}");
    assert_eq!(p["previous"]["value"], 1000);
    assert_eq!(p["target"]["key"], "Features::Propulsion.motorKV");
    assert_eq!(common::dir_hash(&m), before);
    // ... a bad value is refused even when asked to write ...
    let bad = mcp.call_tool("bind", json!({"configuration": "CONF-UAV-DEPLOY-001", "target": "Features::Propulsion.motorKV", "value": 5000, "dry_run": false}));
    assert_eq!(bad["written"], false, "{bad}");
    assert!(bad.to_string().contains("E205"), "{bad}");
    assert!(bad["reason"].as_str().unwrap().contains("would introduce"), "{bad}");
    let codes: Vec<String> = bad["validationDelta"]["newErrors"].as_array().unwrap().iter().map(|e| e["code"].as_str().unwrap().to_string()).collect();
    assert_eq!(codes.len(), codes.iter().collect::<std::collections::BTreeSet<_>>().len(), "each finding once: {codes:?}");
    assert_eq!(common::dir_hash(&m), before);
    // ... and a good one writes.
    let ok = mcp.call_tool("bind", json!({"configuration": "CONF-UAV-DEPLOY-001", "target": "Features::Propulsion.motorKV", "value": 1100, "dry_run": false}));
    assert_eq!(ok["written"], true, "{ok}");
    assert!(std::fs::read_to_string(m.join("Configurations/CONF-UAV-DEPLOY-001.md")).unwrap().contains("motorKV: 1100"));
    assert_eq!(mcp.call_tool_raw("bind", json!({"configuration": "CONF-NOPE", "target": "Features::Telemetry", "value": true}))["isError"], true);
    let mut ro = common::Mcp::start_with_args(&m, &["--read-only"]);
    ro.initialize();
    let names = ro.tools_list().to_string();
    assert!(names.contains("\"timeline\"") && !names.contains("\"bind\""), "bind is a write tool");

    let mut lsp = common::Lsp::start(&m);
    let init = lsp.initialize();
    assert!(init["capabilities"]["experimental"]["syscribe"]["lspRequests"].as_array().unwrap().iter().any(|r| r == "syscribe/timeline"));
    let now: Value = serde_json::from_str(&run(&m, &["timeline", "CONF-UAV-DEPLOY-001", "--json"]).0).unwrap();
    assert_eq!(lsp.custom_request("syscribe/timeline", json!({"configuration": "CONF-UAV-DEPLOY-001"})), now);
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&m);
}

//! TC-TRS-DIFFPARAM-001 (GH #466): `diff --config A --config B` compares effective parameter values too.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn model() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model")
}

fn diff(args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model()).arg("diff").args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), o.status.code().unwrap_or(-1))
}

fn json(a: &str, b: &str, all: bool) -> Value {
    let mut args = vec!["--config", a, "--config", b, "--json"];
    if all {
        args.push("--all");
    }
    let (out, code) = diff(&args);
    assert_eq!(code, 0, "{out}");
    let schema = Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["schema", "diff"]).output().unwrap();
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let url = "https://syscribe.dev/schemas/diff.schema.json";
    compiler.add_resource(url, serde_json::from_slice(&schema.stdout).unwrap()).unwrap();
    let idx = compiler.compile(url, &mut schemas).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    schemas.validate(&v, idx).unwrap_or_else(|e| panic!("{e:#}\n{v}"));
    v
}

#[test]
fn same_features_different_bindings_differ_in_the_parameters_section() {
    // DEPLOY -> FLIGHT select the same features; FLIGHT overrides one binding.
    let (text, code) = diff(&["--config", "CONF-UAV-DEPLOY-001", "--config", "CONF-UAV-FLIGHT-001"]);
    assert_eq!(code, 0);
    assert!(text.contains("Only in CONF-UAV-DEPLOY-001 (0)") && text.contains("Only in CONF-UAV-FLIGHT-001 (0)"), "{text}");
    assert!(text.contains("Parameters that differ (1)"), "{text}");
    assert!(text.contains("Features::Telemetry.rateHz: CONF-UAV-DEPLOY-001 = 10 Hz (default); CONF-UAV-FLIGHT-001 = 20 Hz (own)"), "{text}");
    assert!(!text.contains("motorKV"), "an equal parameter is listed only with --all: {text}");

    let v = json("CONF-UAV-DEPLOY-001", "CONF-UAV-FLIGHT-001", false);
    let rows = v["parameters"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{v}");
    let r = &rows[0];
    assert_eq!((r["key"].as_str(), r["unit"].as_str(), r["differs"].as_bool()), (Some("Features::Telemetry.rateHz"), Some("Hz"), Some(true)));
    assert_eq!((r["a"]["value"].clone(), r["a"]["source"]["kind"].clone()), (10.into(), "default".into()));
    assert_eq!((r["b"]["value"].clone(), r["b"]["source"]["kind"].clone(), r["b"]["stage"].clone()), (20.into(), "own".into(), "runtime".into()));
}

#[test]
fn all_lists_every_parameter_with_its_source_and_a_missing_value_is_null() {
    let v = json("CONF-UAV-DEPLOY-001", "CONF-UAV-FLIGHT-001", true);
    let rows = v["parameters"].as_array().unwrap();
    assert_eq!(rows.len(), 4, "{v}");
    assert_eq!(rows.iter().filter(|r| r["differs"] == true).count(), 1);
    let kv = rows.iter().find(|r| r["key"] == "Features::Propulsion.motorKV").unwrap();
    assert_eq!((kv["unit"].clone(), kv["a"]["source"]["kind"].clone(), kv["b"]["source"]["kind"].clone(), kv["b"]["source"]["from"].clone()), ("rpm/V".into(), "own".into(), "inherited".into(), "CONF-UAV-DEPLOY-001".into()));
    // BUILD binds nothing yet, so a value only the other side has is a difference against null.
    let v = json("CONF-UAV-BUILD-001", "CONF-UAV-DEPLOY-001", false);
    let kv = v["parameters"].as_array().unwrap().iter().find(|r| r["key"] == "Features::Propulsion.motorKV").unwrap().clone();
    assert!(kv["a"].is_null() && kv["differs"] == true, "{kv}");
    // Identical Configurations: an empty list, always present.
    assert_eq!(json("CONF-UAV-DEPLOY-001", "CONF-UAV-DEPLOY-001", false)["parameters"], serde_json::json!([]));
}

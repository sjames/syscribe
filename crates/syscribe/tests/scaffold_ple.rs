//! TC-TRS-SCAFFOLD-001 (GH #427): `new feature | configuration | parameter` — right the first time.

mod common;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn staged(tag: &str) -> PathBuf {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model");
    let dst = std::env::temp_dir().join(format!("syscribe-scaf-{}-{tag}", std::process::id()));
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

fn run(m: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(m).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

fn validate_schema(v: &Value) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["schema", "scaffold"]).output().unwrap();
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let url = "https://syscribe.dev/schemas/scaffold.schema.json";
    compiler.add_resource(url, serde_json::from_slice(&o.stdout).unwrap()).unwrap();
    let idx = compiler.compile(url, &mut schemas).unwrap();
    schemas.validate(v, idx).unwrap_or_else(|e| panic!("{e:#}\n{v}"));
}

fn new(m: &Path, args: &[&str]) -> (Value, i32) {
    let mut a = vec!["new"];
    a.extend_from_slice(args);
    a.push("--json");
    let (o, e, c) = run(m, &a);
    let v: Value = serde_json::from_str(&o).unwrap_or_else(|x| panic!("{x}: {o} / {e}"));
    validate_schema(&v);
    (v, c)
}

/// The set of (code, file-name, message) the model validates with.
fn findings(m: &Path) -> Vec<String> {
    let (o, ..) = run(m, &["validate", "--json"]);
    let v: Value = serde_json::from_str(&o).unwrap();
    let mut f: Vec<String> = v.as_array().unwrap().iter().map(|f| format!("{} {}", f["code"].as_str().unwrap(), f["message"].as_str().unwrap())).collect();
    f.sort();
    f
}

fn feature_check_errors(m: &Path) -> usize {
    let (o, ..) = run(m, &["feature-check", "--json"]);
    let v: Value = serde_json::from_str(&o).unwrap();
    v["findings"].as_array().unwrap().iter().filter(|f| f["severity"] == "error").count()
}

#[test]
fn a_feature_gets_a_free_derived_id_and_adds_no_finding() {
    let m = staged("feat");
    let before = findings(&m);
    let (dry, c) = new(&m, &["feature", "Gimbal", "--under", "Features", "--dry-run"]);
    assert_eq!((c, dry["written"].clone(), dry["target"].clone()), (0, json!(false), json!("FEAT-GIMBAL")));
    assert!(!m.join("Features/Gimbal.md").exists(), "a dry run writes nothing");
    let (v, c) = new(&m, &["feature", "Gimbal", "--under", "Features", "--group", "alternative", "--mandatory"]);
    assert_eq!((c, v["written"].clone()), (0, json!(true)), "{v}");
    assert_eq!((v["newErrors"].clone(), v["newWarnings"].clone()), (json!([]), json!([])), "zero new findings: {v}");
    let text = std::fs::read_to_string(m.join("Features/Gimbal.md")).unwrap();
    assert!(text.starts_with("---\nid: FEAT-GIMBAL\ntype: FeatureDef\nname: Gimbal\nmandatory: true\ngroupKind: alternative\n---"), "{text}");
    assert_eq!(findings(&m), before, "the model validates exactly as before");
    assert_eq!(feature_check_errors(&m), 0);
    // A second feature of the same name elsewhere cannot reuse the id: the names above it are added.
    std::fs::create_dir_all(m.join("Other")).unwrap();
    std::fs::write(m.join("Other/_index.md"), "---\ntype: Package\nname: Other\n---\n").unwrap();
    let (v, c) = new(&m, &["feature", "Gimbal", "--under", "Other"]);
    assert_eq!((c, v["target"].clone()), (0, json!("FEAT-OTHER-GIMBAL")), "{v}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_feature_goes_under_a_group_feature_and_refuses_a_leaf() {
    let m = staged("group");
    std::fs::create_dir_all(m.join("Features/Link")).unwrap();
    std::fs::write(m.join("Features/Link/_index.md"), "---\nid: FEAT-UAV-LINK\ntype: FeatureDef\nname: Link\ngroupKind: alternative\n---\n\nl\n").unwrap();
    let (v, c) = new(&m, &["feature", "Radio", "--under", "Features::Link"]);
    assert_eq!(c, 0, "{v}");
    assert!(m.join("Features/Link/Radio.md").exists());
    let (_o, e, c) = run(&m, &["new", "feature", "Sub", "--under", "Features::Telemetry"]);
    assert_eq!(c, 1);
    assert!(e.contains("leaf feature file"), "{e}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn feature_mistakes_are_refused_before_a_file_exists() {
    let m = staged("fmis");
    for (args, code, needle) in [
        (vec!["new", "feature", "Gim-bal", "--under", "Features"], 2, "basic name"),
        (vec!["new", "feature", "Gimbal", "--under", "Features", "--id", "FEAT-X"], 2, "FEAT-* id"),
        (vec!["new", "feature", "Gimbal", "--under", "Features", "--id", "FEAT-UAV-PROP"], 1, "already used"),
        (vec!["new", "feature", "Telemetry", "--under", "Features"], 1, "already exists"),
        (vec!["new", "feature", "Gimbal", "--under", "Features", "--group", "xor"], 2, "optional, alternative or or"),
        (vec!["new", "feature", "Gimbal", "--under", "Nowhere"], 1, "not a package"),
        (vec!["new", "feature", "Gimbal"], 2, "--under"),
    ] {
        let (_o, e, c) = run(&m, &args);
        assert_eq!(c, code, "{args:?}: {e}");
        assert!(e.contains(needle), "{args:?}: {e}");
    }
    assert!(!m.join("Features/Gimbal.md").exists());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_configuration_derives_at_a_real_stage_no_earlier_than_its_base() {
    let m = staged("conf");
    let before = findings(&m);
    let (v, c) = new(&m, &["configuration", "CONF-UAV-TEST-001", "--from", "CONF-UAV-DEPLOY-001", "--stage", "runtime", "--name", "Bench test"]);
    assert_eq!((c, v["written"].clone()), (0, json!(true)), "{v}");
    assert_eq!((v["newErrors"].clone(), v["newWarnings"].clone()), (json!([]), json!([])), "{v}");
    let text = std::fs::read_to_string(m.join("Configurations/CONF-UAV-TEST-001.md")).unwrap();
    assert!(text.contains("featureModel: Features") && text.contains("bindingStage: runtime") && text.contains("derivedFrom: CONF-UAV-DEPLOY-001") && text.contains("status: draft"), "{text}");
    assert_eq!(findings(&m), before);
    // The earlier stage would be E753, so it is refused up front; so is a stage that does not exist, and a bad or taken id.
    let (_o, e, c) = run(&m, &["new", "configuration", "CONF-UAV-TEST-002", "--from", "CONF-UAV-DEPLOY-001", "--stage", "compile"]);
    assert_eq!(c, 1);
    assert!(e.contains("same or a later stage"), "{e}");
    assert_eq!(run(&m, &["new", "configuration", "CONF-UAV-TEST-002", "--from", "CONF-UAV-DEPLOY-001", "--stage", "loda"]).2, 2);
    assert_eq!(run(&m, &["new", "configuration", "conf-lower", "--from", "CONF-UAV-DEPLOY-001"]).2, 2);
    assert_eq!(run(&m, &["new", "configuration", "CONF-UAV-TEST-001", "--from", "CONF-UAV-DEPLOY-001"]).2, 1, "id already used");
    assert_eq!(run(&m, &["new", "configuration", "CONF-UAV-TEST-003", "--from", "CONF-NOPE"]).2, 1);
    assert!(!m.join("Configurations/CONF-UAV-TEST-002.md").exists());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_standalone_configuration_seeds_its_selection() {
    let m = staged("solo");
    let (v, c) = new(&m, &["configuration", "CONF-UAV-SOLO-001", "--select", "FEAT-UAV-PROP=true", "--select", "Features::Telemetry=false"]);
    assert_eq!(c, 0, "{v}");
    let text = std::fs::read_to_string(m.join("Configurations/CONF-UAV-SOLO-001.md")).unwrap();
    assert!(text.contains("features:\n  Features::Propulsion: true\n  Features::Telemetry: false"), "ids are written as qualified names: {text}");
    assert_eq!(run(&m, &["new", "configuration", "CONF-UAV-SOLO-002", "--select", "Features::Nope=true"]).2, 1);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_parameter_is_appended_in_place_with_a_valid_window() {
    let m = staged("param");
    let before = findings(&m);
    let original = std::fs::read_to_string(m.join("Features/Telemetry.md")).unwrap();
    let (v, c) = new(&m, &["parameter", "Features::Telemetry", "burstMs", "--unit", "ms", "--range", "10..500", "--default", "100", "--stage", "load", "--latest", "runtime"]);
    assert_eq!((c, v["written"].clone()), (0, json!(true)), "{v}");
    assert_eq!((v["newErrors"].clone(), v["newWarnings"].clone()), (json!([]), json!([])), "{v}");
    let text = std::fs::read_to_string(m.join("Features/Telemetry.md")).unwrap();
    assert!(text.contains("  - name: burstMs\n    type: ScalarValues::Real\n    unit: ms\n    range: \"10..500\"\n    default: 100\n    bindingTime: load\n    latestBindingTime: runtime"), "{text}");
    // Only lines were added: every original line is still there, in order.
    let mut it = text.lines();
    assert!(original.lines().all(|l| it.any(|x| x == l)), "nothing else changed");
    assert_eq!(findings(&m), before);
    // The parameter is real: it can be bound.
    assert_eq!(run(&m, &["bind", "CONF-UAV-FLIGHT-001", "Features::Telemetry.burstMs", "200"]).2, 0);
    // An enumeration, a required flag, and a feature with no parameters yet.
    let (v, c) = new(&m, &["feature", "Gimbal", "--under", "Features"]);
    assert_eq!(c, 0, "{v}");
    let (v, c) = new(&m, &["parameter", "Features::Gimbal", "mode", "--type", "String", "--enum", "off,hold,track", "--default", "hold", "--required", "--stage", "compile", "--latest", "load"]);
    assert_eq!(c, 0, "{v}");
    let g = std::fs::read_to_string(m.join("Features/Gimbal.md")).unwrap();
    assert!(g.contains("parameters:\n  - name: mode\n    type: ScalarValues::String\n    enumValues: [off, hold, track]\n    default: hold\n    isRequired: true\n    bindingTime: compile"), "{g}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn parameter_mistakes_are_refused() {
    let m = staged("pmis");
    for (args, code, needle) in [
        (vec!["new", "parameter", "Features::Telemetry", "rateHz"], 1, "already has a parameter"),
        (vec!["new", "parameter", "Features::Nope", "x"], 1, "FeatureDef not found"),
        (vec!["new", "parameter", "Features::Telemetry", "x", "--range", "5..1"], 2, "low end above"),
        (vec!["new", "parameter", "Features::Telemetry", "x", "--range", "abc"], 2, "<lo>..<hi>"),
        (vec!["new", "parameter", "Features::Telemetry", "x", "--range", "1..5", "--default", "9"], 2, "outside the range"),
        (vec!["new", "parameter", "Features::Telemetry", "x", "--stage", "runtime", "--latest", "compile"], 2, "earlier than the earliest"),
        (vec!["new", "parameter", "Features::Telemetry", "x", "--stage", "soon"], 2, "unknown stage"),
        (vec!["new", "parameter", "Features::Telemetry", "x", "--type", "Float"], 2, "Real, Integer"),
        (vec!["new", "parameter", "Features::Telemetry", "x", "--enum", "a,b", "--default", "c", "--type", "String"], 2, "not one of the enumeration"),
        (vec!["new", "parameter", "Features::Telemetry", "bad-name"], 2, "basic name"),
    ] {
        let (_o, e, c) = run(&m, &args);
        assert_eq!(c, code, "{args:?}: {e}");
        assert!(e.contains(needle), "{args:?}: {e}");
    }
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn the_mcp_tool_previews_by_default_and_writes_only_a_clean_scaffold() {
    let m = staged("mcp");
    let mut mcp = common::Mcp::start(&m);
    mcp.initialize();
    let before = common::dir_hash(&m);
    let p = mcp.call_tool("scaffold", json!({"kind": "feature", "name": "Gimbal", "under": "Features"}));
    assert_eq!(p["written"], false, "{p}");
    assert_eq!(p["target"], "FEAT-GIMBAL");
    assert_eq!(common::dir_hash(&m), before);
    let w = mcp.call_tool("scaffold", json!({"kind": "feature", "name": "Gimbal", "under": "Features", "dry_run": false}));
    assert_eq!(w["written"], true, "{w}");
    assert!(m.join("Features/Gimbal.md").exists());
    let c = mcp.call_tool("scaffold", json!({"kind": "configuration", "name": "CONF-UAV-MCP-001", "from": "CONF-UAV-DEPLOY-001", "stage": "runtime", "dry_run": false}));
    assert_eq!(c["written"], true, "{c}");
    let pr = mcp.call_tool("scaffold", json!({"kind": "parameter", "feature": "Features::Gimbal", "name": "gain", "range": "0..10", "default": "5", "dry_run": false}));
    assert_eq!(pr["written"], true, "{pr}");
    assert_eq!(mcp.call_tool_raw("scaffold", json!({"kind": "feature", "name": "Bad-Name", "under": "Features"}))["isError"], true);
    assert_eq!(mcp.call_tool_raw("scaffold", json!({"kind": "feature", "name": "X"}))["isError"], true, "needs under");
    assert_eq!(mcp.call_tool_raw("scaffold", json!({"kind": "gadget", "name": "X"}))["isError"], true);
    let mut ro = common::Mcp::start_with_args(&m, &["--read-only"]);
    ro.initialize();
    assert!(!ro.tools_list().to_string().contains("\"scaffold\""), "a write tool");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn usage_and_capabilities() {
    let m = staged("use");
    assert_eq!(run(&m, &["new"]).2, 2);
    assert_eq!(run(&m, &["new", "gadget", "x"]).2, 2);
    assert_eq!(run(&m, &["new", "feature", "A", "B", "--under", "Features"]).2, 2);
    let (o, ..) = run(&m, &["capabilities", "--json"]);
    let caps: Value = serde_json::from_str(&o).unwrap();
    assert_eq!(caps["features"]["scaffolding"], true);
    assert!(caps["jsonSchemas"]["scaffold"]["available"] == true);
    let _ = std::fs::remove_dir_all(&m);
}

//! TC-TRS-IMPACT-001 (GH #423): `change-impact` and `diff` locations — evaluated on a candidate copy, never on disk.

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

fn impact(model: &Path, args: &[&str]) -> Value {
    let mut a = vec!["change-impact"];
    a.extend_from_slice(args);
    a.push("--json");
    let (o, e, c) = run(model, &a);
    assert_eq!(c, 0, "{a:?}: {e}");
    let v: Value = serde_json::from_str(&o).unwrap();
    validate("change-impact", &v);
    v
}

fn conf<'a>(v: &'a Value, id: &str) -> &'a Value {
    v["configurations"].as_array().unwrap().iter().find(|c| c["configuration"] == id).unwrap_or_else(|| panic!("no {id}: {v}"))
}

/// Alpha and Beta features; ONE selects Alpha, TWO selects Beta; Gadget applies when Alpha is on.
fn model(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-impact-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let w = |rel: &str, c: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/Alpha.md", "---\nid: FEAT-IM-ALPHA\ntype: FeatureDef\nname: Alpha\n---\n\na\n");
    w("Features/Beta.md", "---\nid: FEAT-IM-BETA\ntype: FeatureDef\nname: Beta\n---\n\nb\n");
    w("Configurations/_index.md", "---\ntype: Package\nname: Configurations\n---\n");
    w("Configurations/CONF-IM-ONE.md", "---\nid: CONF-IM-ONE\ntype: Configuration\nname: One\nstatus: approved\nfeatureModel: Features\nfeatures:\n  FEAT-IM-ALPHA: true\n  FEAT-IM-BETA: false\n---\n\nc\n");
    w("Configurations/CONF-IM-TWO.md", "---\nid: CONF-IM-TWO\ntype: Configuration\nname: Two\nstatus: approved\nfeatureModel: Features\nfeatures:\n  FEAT-IM-ALPHA: false\n  FEAT-IM-BETA: true\n---\n\nc\n");
    w("Parts/Gadget.md", "---\ntype: PartDef\nname: Gadget\nappliesWhen: Features::Alpha\n---\n\ng\n");
    root
}

#[test]
fn a_new_requires_makes_the_configuration_that_violates_it_invalid() {
    let m = model("req");
    let v = impact(&m, &["Features::Alpha", "--set", "requires=[Features::Beta]"]);
    let one = conf(&v, "CONF-IM-ONE");
    assert_eq!((one["becameInvalid"].clone(), one["changed"].clone()), (json!(true), json!(true)), "{one}");
    assert_eq!(one["validityBefore"]["errors"], 0);
    assert!(one["validityAfter"]["errors"].as_u64().unwrap() >= 1);
    // Findings from the candidate copy are reported under the real model root, never its temporary one.
    let f = one["newErrors"][0]["file"].as_str().unwrap();
    assert!(f.starts_with(m.canonicalize().unwrap().to_str().unwrap()) && !f.contains("cand"), "{f}");
    assert_eq!(one["newErrors"][0]["code"], "E219");
    // TWO does not select Alpha, so the requirement does not touch it.
    assert_eq!(conf(&v, "CONF-IM-TWO")["changed"], false);
    assert_eq!(v["summary"]["becameInvalid"], 1);
    assert_eq!(v["summary"]["changed"], 1);
    assert_eq!(v["written"], false);
    assert!(v["diff"].as_str().unwrap().contains("requires"), "the unified diff is returned: {}", v["diff"]);
    assert!(v["location"]["file"].as_str().unwrap().ends_with("Alpha.md") && v["location"]["line"].as_u64().unwrap() >= 1);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_changed_applies_when_moves_the_element_between_projections() {
    let m = model("proj");
    let v = impact(&m, &["Parts::Gadget", "--set", "appliesWhen=Features::Beta"]);
    let removed = conf(&v, "CONF-IM-ONE")["projection"]["removed"].to_string();
    let added = conf(&v, "CONF-IM-TWO")["projection"]["added"].to_string();
    assert!(removed.contains("Parts::Gadget"), "{removed}");
    assert!(added.contains("Parts::Gadget"), "{added}");
    assert_eq!(v["summary"]["projectionChanged"], 2);
    assert_eq!(v["summary"]["becameInvalid"], 0);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_parameter_range_edit_is_evaluated_against_the_configurations_narrowings() {
    let staged = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model");
    let v = impact(&staged, &["FEAT-UAV-PROP", "--param", "motorKV", "--param-set", "range=900..1000"]);
    let build = conf(&v, "CONF-UAV-BUILD-001");
    assert_eq!(build["becameInvalid"], true, "{build}");
    assert!(build["newErrors"].to_string().contains("E755"), "{build}");
    // Widening it breaks nothing.
    let w = impact(&staged, &["FEAT-UAV-PROP", "--param", "motorKV", "--param-set", "range=800..1300"]);
    assert_eq!(w["summary"]["becameInvalid"], 0, "{w}");
}

#[test]
fn nothing_is_ever_written() {
    let m = model("ro");
    let before = common::dir_hash(&m);
    let _ = impact(&m, &["Features::Alpha", "--set", "requires=[Features::Beta]"]);
    let _ = impact(&m, &["Parts::Gadget", "--set", "appliesWhen=null"]);
    assert_eq!(common::dir_hash(&m), before);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn errors_and_usage() {
    let m = model("err");
    let (_o, e, c) = run(&m, &["change-impact", "Features::Nope", "--set", "x=1"]);
    assert_eq!(c, 1, "{e}");
    let (_o, e, c) = run(&m, &["change-impact", "Features::Alpha"]);
    assert_eq!(c, 2, "{e}");
    let (_o, e, c) = run(&m, &["change-impact", "Features::Alpha", "--param", "ghost", "--param-set", "range=1..2"]);
    assert_eq!(c, 1, "{e}");
    assert!(e.contains("no parameter 'ghost'"), "{e}");
    for bad in [&["change-impact"][..], &["change-impact", "Features::Alpha", "--set", "noequals"], &["change-impact", "Features::Alpha", "--param-set", "a=1"], &["change-impact", "A", "B"]] {
        assert_eq!(run(&m, bad).2, 2, "{bad:?}");
    }
    let (text, _e, c) = run(&m, &["change-impact", "Features::Alpha", "--set", "requires=[Features::Beta]"]);
    assert_eq!(c, 0);
    assert!(text.contains("become invalid") && text.contains("CONF-IM-ONE") && text.contains("Nothing was written"), "{text}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn diff_changes_carry_jump_to_source_locations() {
    let m = model("diff");
    let (o, e, c) = run(&m, &["diff", "--config", "CONF-IM-ONE", "--config", "CONF-IM-TWO", "--json"]);
    assert_eq!(c, 0, "{e}");
    let v: Value = serde_json::from_str(&o).unwrap();
    validate("diff", &v);
    assert!(v["onlyInA"].to_string().contains("Parts::Gadget"), "existing property unchanged: {v}");
    let g = v["changes"].as_array().unwrap().iter().find(|c| c["qualifiedName"] == "Parts::Gadget").unwrap();
    assert_eq!((g["side"].as_str(), g["type"].as_str()), (Some("onlyInA"), Some("PartDef")));
    assert!(g["location"]["file"].as_str().unwrap().ends_with("Gadget.md"));
    assert!(g["location"]["line"].as_u64().unwrap() >= 1);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn the_mcp_tool_and_the_lsp_request_return_the_same_payload() {
    let m = model("surf");
    let cli = impact(&m, &["Features::Alpha", "--set", "requires=[Features::Beta]"]);
    let args = json!({"element": "Features::Alpha", "set": {"requires": ["Features::Beta"]}});
    let mut mcp = common::Mcp::start(&m);
    mcp.initialize();
    assert_eq!(mcp.call_tool("change_impact", args.clone()), cli);
    assert_eq!(mcp.call_tool_raw("change_impact", json!({"element": "Nope"}))["isError"], true);
    let mut lsp = common::Lsp::start(&m);
    let init = lsp.initialize();
    assert!(init["capabilities"]["experimental"]["syscribe"]["lspRequests"].as_array().unwrap().iter().any(|r| r == "syscribe/changeImpact"));
    assert_eq!(lsp.custom_request("syscribe/changeImpact", args), cli);
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&m);
}

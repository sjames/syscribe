//! TC-TRS-EXPLAIN-001 (GH #417): `explain-feature` — why a feature is forced or excluded, and what to change.

mod common;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(model: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

fn explain(model: &Path, feature: &str, extra: &[&str]) -> Value {
    let mut a = vec!["explain-feature", feature, "--json"];
    a.extend_from_slice(extra);
    let (o, e, c) = run(model, &a);
    assert_eq!(c, 0, "{a:?}: {e}");
    let v: Value = serde_json::from_str(&o).unwrap_or_else(|x| panic!("{x}: {o}"));
    validate_schema(&v);
    v
}

fn validate_schema(v: &Value) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["schema", "explain-feature"]).output().unwrap();
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let url = "https://syscribe.dev/schemas/explain-feature.schema.json";
    compiler.add_resource(url, serde_json::from_slice(&o.stdout).unwrap()).unwrap();
    let idx = compiler.compile(url, &mut schemas).unwrap();
    schemas.validate(v, idx).unwrap_or_else(|e| panic!("{e:#}\n{v}"));
}

/// Root (mandatory); Group G (alternative: exactly one of X, Y) under Root; A excludes B; C requires A;
/// Z requires A and B (so it can never be selected); PRM (with a parameter).
fn model(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-expl-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("Features")).unwrap();
    let w = |rel: &str, c: &str| std::fs::write(root.join(rel), c).unwrap();
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    let f = |name: &str, id: &str, extra: &str| w(&format!("Features/{name}.md"), &format!("---\nid: {id}\ntype: FeatureDef\nname: {name}\n{extra}---\n\n{name}\n"));
    f("Top", "FEAT-EX-TOP", "mandatory: true\n");
    f("A", "FEAT-EX-AA", "excludes: [Features::B]\n");
    f("B", "FEAT-EX-BB", "");
    f("C", "FEAT-EX-CC", "requires: [Features::A]\n");
    f("Z", "FEAT-EX-ZZ", "requires: [Features::A, Features::B]\n");
    f("G", "FEAT-EX-GG", "groupKind: alternative\n");
    std::fs::create_dir_all(root.join("Features/G")).unwrap();
    w("Features/G/_index.md", "---\ntype: Package\nname: G\n---\n");
    root
}

#[test]
fn a_required_feature_is_forced_on_and_the_cause_names_the_choice_and_the_constraint() {
    let m = model("req");
    let e = explain(&m, "Features::A", &["--select", "Features::C=true"]);
    assert_eq!(e["state"], "forcedOn");
    assert_eq!(e["because"]["choices"], json!([{"feature": "Features::C", "selected": true, "file": e["because"]["choices"][0]["file"]}]));
    let c = &e["because"]["constraints"][0];
    assert_eq!(c["kind"], "requires");
    assert_eq!(c["owner"], "Features::C");
    assert_eq!(c["features"], json!(["Features::A", "Features::C"]));
    assert!(c["file"].as_str().unwrap().ends_with("C.md"), "{c}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn an_excluded_feature_is_forced_off_and_the_fix_is_to_drop_a_choice() {
    let m = model("excl");
    let e = explain(&m, "Features::B", &["--select", "Features::C=true"]);
    assert_eq!(e["state"], "forcedOff", "C requires A, A excludes B: {e}");
    // The cause is C's choice and the two constraints on the path.
    let kinds: Vec<&str> = e["because"]["constraints"].as_array().unwrap().iter().map(|c| c["kind"].as_str().unwrap()).collect();
    assert!(kinds.contains(&"requires") && kinds.contains(&"excludes"), "{kinds:?}");
    // To select B, drop the choice on C (dropping it alone suffices).
    let t = &e["toSelect"];
    assert_eq!((t["needed"].as_bool(), t["modelLevel"].as_bool()), (Some(true), Some(false)));
    assert_eq!(t["options"].as_array().unwrap().len(), 1);
    assert_eq!(t["options"][0]["drop"]["feature"], "Features::C");
    // And that is true: without that choice B is no longer forced off.
    let after = explain(&m, "Features::B", &[]);
    assert_eq!(after["state"], "free");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_feature_no_choice_can_select_is_a_model_level_cause() {
    let m = model("dead");
    let e = explain(&m, "Features::Z", &[]);
    assert_eq!(e["toSelect"]["modelLevel"], true, "{e}");
    assert!(e["toSelect"]["options"].as_array().unwrap().is_empty());
    let labels: Vec<&str> = e["toSelect"]["constraints"].as_array().unwrap().iter().map(|c| c["label"].as_str().unwrap()).collect();
    assert!(labels.iter().any(|l| l.contains("excludes")) && labels.iter().any(|l| l.contains("requires")), "{labels:?}");
    // Agrees with feature-check --deep, which reports the same feature dead.
    let (o, _e, _c) = run(&m, &["feature-check", "--deep", "--json"]);
    let deep: Value = serde_json::from_str(&o).unwrap();
    assert!(deep["deadFeatures"].to_string().contains("Features::Z"), "deep says: {}", deep["deadFeatures"]);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn group_cardinality_is_a_cause() {
    let m = model("group");
    // Children of the alternative group G: add X and Y.
    let w = |rel: &str, c: &str| std::fs::write(m.join(rel), c).unwrap();
    w("Features/G/X.md", "---\nid: FEAT-EX-XX\ntype: FeatureDef\nname: X\n---\n\nx\n");
    w("Features/G/Y.md", "---\nid: FEAT-EX-YY\ntype: FeatureDef\nname: Y\n---\n\ny\n");
    // Selecting X leaves Y out (at most one) ...
    let off = explain(&m, "Features::G::Y", &["--select", "Features::G::X=true"]);
    assert_eq!(off["state"], "forcedOff", "{off}");
    assert!(off["because"]["constraints"].as_array().unwrap().iter().any(|c| c["kind"] == "groupAtMost"), "{off}");
    // ... and with the group selected and X out, Y is forced on (at least one).
    let on = explain(&m, "Features::G::Y", &["--select", "Features::G=true", "--select", "Features::G::X=false"]);
    assert_eq!(on["state"], "forcedOn", "{on}");
    assert!(on["because"]["constraints"].as_array().unwrap().iter().any(|c| c["kind"] == "groupAtLeast"), "{on}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn clashing_choices_are_reported_as_a_conflict() {
    let m = model("clash");
    let e = explain(&m, "Features::C", &["--select", "Features::A=true", "--select", "Features::B=true"]);
    assert_eq!(e["state"], "conflict");
    assert_eq!(e["because"]["choices"].as_array().unwrap().len(), 2, "{e}");
    assert!(e["toSelect"].is_null());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn parameter_constraints_that_touch_the_feature_are_listed_with_their_state() {
    let staged = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model");
    let e = explain(&staged, "Features::Propulsion", &["--from", "CONF-UAV-BUILD-001"]);
    let pcs = e["parameterConstraints"].as_array().unwrap();
    assert_eq!(pcs.len(), 1, "{e}");
    assert_eq!((pcs[0]["id"].as_str(), pcs[0]["state"].as_str()), (Some("PC-UAV-CURRENT"), Some("undetermined")));
    assert!(pcs[0]["open"].as_array().unwrap().iter().any(|o| o == "Features::Propulsion.motorKV"), "{pcs:?}");
    // A feature none of whose parameters appear lists none.
    let t = explain(&staged, "Features::Telemetry", &["--from", "CONF-UAV-BUILD-001"]);
    assert!(t["parameterConstraints"].as_array().unwrap().is_empty());
}

#[test]
fn errors_text_form_and_surfaces() {
    let m = model("surf");
    let (_o, e, c) = run(&m, &["explain-feature", "Features::Nope"]);
    assert_eq!(c, 1);
    assert!(e.contains("not a known FeatureDef"), "{e}");
    for bad in [&["explain-feature"][..], &["explain-feature", "A", "B"], &["explain-feature", "A", "--select", "x"], &["explain-feature", "A", "--nope"]] {
        let (_o, e, c) = run(&m, bad);
        assert!(c == 2 || c == 1, "{bad:?} {e}");
    }
    let (_o, _e, c) = run(&m, &["explain-feature", "Features::A", "--from", "CONF-NOPE"]);
    assert_eq!(c, 1);
    let (text, _e, c) = run(&m, &["explain-feature", "Features::B", "--select", "Features::C=true"]);
    assert_eq!(c, 0);
    assert!(text.contains("forcedOff") && text.contains("## Because") && text.contains("Drop any one of these choices") && text.contains("Features::C"), "{text}");

    let args = json!({"feature": "Features::B", "select": {"Features::C": true}});
    let cli = explain(&m, "Features::B", &["--select", "Features::C=true"]);
    let mut mcp = common::Mcp::start(&m);
    mcp.initialize();
    assert_eq!(mcp.call_tool("explain_feature", args.clone()), cli);
    let mut lsp = common::Lsp::start(&m);
    let init = lsp.initialize();
    assert!(init["capabilities"]["experimental"]["syscribe"]["lspRequests"].as_array().unwrap().iter().any(|r| r == "syscribe/explainFeature"));
    assert_eq!(lsp.custom_request("syscribe/explainFeature", args), cli);
    lsp.shutdown();
    let before = common::dir_hash(&m);
    let _ = run(&m, &["explain-feature", "Features::B", "--select", "Features::C=true"]);
    assert_eq!(common::dir_hash(&m), before, "read-only");
    let _ = std::fs::remove_dir_all(&m);
}

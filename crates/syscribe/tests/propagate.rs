//! TC-TRS-PROPAGATE-001 (GH #416): `propagate` — live propagation of a partial selection, read-only.

mod common;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel)
}

fn run(model: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

fn json_of(model: &Path, args: &[&str]) -> Value {
    let (o, e, c) = run(model, args);
    assert_eq!(c, 0, "{args:?}: {e}");
    serde_json::from_str(&o).unwrap_or_else(|x| panic!("{x}: {o}"))
}

fn validate_schema(v: &Value) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["schema", "propagate"]).output().unwrap();
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let url = "https://syscribe.dev/schemas/propagate.schema.json";
    compiler.add_resource(url, serde_json::from_slice(&o.stdout).unwrap()).unwrap();
    let idx = compiler.compile(url, &mut schemas).unwrap();
    schemas.validate(v, idx).unwrap_or_else(|e| panic!("{e:#}\n{v}"));
}

fn names(v: &Value, k: &str) -> Vec<String> {
    let mut o: Vec<String> = v[k].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
    o.sort();
    o
}

/// A small model: Root (mandatory) with A, B, C optional; A excludes B; C requires A; stages differ per feature.
fn small_model(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-prop-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("Features")).unwrap();
    let w = |rel: &str, c: &str| std::fs::write(root.join(rel), c).unwrap();
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/Root.md", "---\nid: FEAT-PR-ROOT\ntype: FeatureDef\nname: Root\nmandatory: true\n---\n\nr\n");
    w("Features/A.md", "---\nid: FEAT-PR-AA\ntype: FeatureDef\nname: A\nexcludes: [Features::B]\nbindingTime: compile\nlatestBindingTime: compile\n---\n\na\n");
    w("Features/B.md", "---\nid: FEAT-PR-BB\ntype: FeatureDef\nname: B\nbindingTime: runtime\nlatestBindingTime: runtime\n---\n\nb\n");
    w("Features/C.md", "---\nid: FEAT-PR-CC\ntype: FeatureDef\nname: C\nrequires: [Features::A]\nbindingTime: runtime\nlatestBindingTime: runtime\n---\n\nc\n");
    std::fs::create_dir_all(root.join("Configurations")).unwrap();
    w("Configurations/_index.md", "---\ntype: Package\nname: Configurations\n---\n");
    w("Configurations/CONF-PR-ONE.md", "---\nid: CONF-PR-ONE\ntype: Configuration\nname: One\nfeatures:\n  FEAT-PR-CC: true\n---\n\nc\n");
    root
}

#[test]
fn it_equals_configure_for_every_configuration_of_the_example_models() {
    for (model, confs) in [("model_gti", vec!["CONF-GTI-US-M3-001", "CONF-GTI-IN-M2-001"]), ("examples/staged-configuration/model", vec!["CONF-UAV-BUILD-001", "CONF-UAV-FLIGHT-001"])] {
        let m = repo(model);
        for c in confs {
            let p = json_of(&m, &["propagate", "--from", c, "--json"]);
            validate_schema(&p);
            let q = json_of(&m, &["configure", c, "--json"]);
            assert_eq!(p["satisfiable"], q["satisfiable"], "{model} {c}");
            for (a, b) in [("forcedTrue", "forcedTrue"), ("forcedFalse", "forcedFalse"), ("free", "free")] {
                assert_eq!(names(&p, a), names(&q, b), "{model} {c}: {a}");
            }
        }
    }
}

#[test]
fn choices_force_exclude_and_leave_free() {
    let m = small_model("force");
    let p = json_of(&m, &["propagate", "--select", "FEAT-PR-CC=true", "--json"]);
    validate_schema(&p);
    assert_eq!(p["satisfiable"], true);
    assert!(names(&p, "forcedTrue").contains(&"Features::A".to_string()), "C requires A: {p}");
    assert!(names(&p, "forcedFalse").contains(&"Features::B".to_string()), "A excludes B: {p}");
    assert_eq!(p["features"]["Features::C"]["state"], "selected");
    assert!(p["selection"]["Features::C"] == true, "ids are canonicalised: {p}");
    // Nothing chosen: A, B and C stay free.
    let none = json_of(&m, &["propagate", "--json"]);
    assert!(names(&none, "free").contains(&"Features::A".to_string()));
    // --select overrides --from.
    let o = json_of(&m, &["propagate", "--from", "CONF-PR-ONE", "--select", "Features::C=false", "--json"]);
    assert_eq!(o["features"]["Features::C"]["state"], "deselected");
    assert_eq!(o["selection"]["Features::C"], false);
    // An unknown feature is reported, not fatal.
    let u = json_of(&m, &["propagate", "--select", "Features::Nope=true", "--json"]);
    assert_eq!(u["unknown"], json!(["Features::Nope"]));
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_clash_names_the_choices_and_the_constraints() {
    let m = small_model("clash");
    let p = json_of(&m, &["propagate", "--select", "Features::A=true", "--select", "Features::B=true", "--json"]);
    validate_schema(&p);
    assert_eq!(p["satisfiable"], false);
    let choices: Vec<&str> = p["conflict"]["choices"].as_array().unwrap().iter().map(|c| c["feature"].as_str().unwrap()).collect();
    assert!(choices.contains(&"Features::A") && choices.contains(&"Features::B"), "{p}");
    assert!(!p["conflict"]["constraints"].as_array().unwrap().is_empty(), "{p}");
    let (text, _e, c) = run(&m, &["propagate", "--select", "Features::A=true", "--select", "Features::B=true"]);
    assert_eq!(c, 0);
    assert!(text.contains("# Conflict") && text.contains("Features::B"), "{text}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_stage_defers_features_bound_elsewhere() {
    let m = small_model("stage");
    let p = json_of(&m, &["propagate", "--select", "Features::C=true", "--stage", "compile", "--json"]);
    validate_schema(&p);
    assert_eq!(p["stage"], "compile");
    // A binds at compile, B and C at runtime: B is deferred, so it drops out of the lists but keeps its true state.
    assert_eq!(p["features"]["Features::B"]["deferred"], true);
    assert_eq!(p["features"]["Features::B"]["state"], "forcedOff");
    assert!(!names(&p, "forcedFalse").contains(&"Features::B".to_string()), "{p}");
    assert!(names(&p, "forcedTrue").contains(&"Features::A".to_string()), "A is decided at compile: {p}");
    assert!(p["features"]["Features::A"].get("deferred").is_none());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn nothing_is_written() {
    let m = small_model("ro");
    let before = common::dir_hash(&m);
    let _ = run(&m, &["propagate", "--from", "CONF-PR-ONE", "--select", "Features::B=true", "--json"]);
    assert_eq!(common::dir_hash(&m), before, "the model directory is unchanged");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn errors_and_usage() {
    let m = small_model("err");
    let (_o, e, c) = run(&m, &["propagate", "--from", "CONF-NOPE"]);
    assert_eq!(c, 1);
    assert!(e.contains("Configuration not found"), "{e}");
    for bad in [&["propagate", "--stage", "nope"][..], &["propagate", "--select", "A"], &["propagate", "--select", "A=maybe"], &["propagate", "extra"]] {
        let (_o, e, c) = run(&m, bad);
        assert_eq!(c, 2, "{bad:?}: {e}");
    }
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn the_mcp_tool_and_the_lsp_request_return_the_same_payload() {
    let m = small_model("surfaces");
    let cli = json_of(&m, &["propagate", "--from", "CONF-PR-ONE", "--select", "Features::B=false", "--stage", "compile", "--json"]);
    let args = json!({"from": "CONF-PR-ONE", "select": {"Features::B": false}, "stage": "compile"});

    let mut mcp = common::Mcp::start(&m);
    mcp.initialize();
    assert_eq!(mcp.call_tool("propagate", args.clone()), cli);
    let bad = mcp.call_tool_raw("propagate", json!({"stage": "nope"}));
    assert_eq!(bad["isError"], true);

    let mut lsp = common::Lsp::start(&m);
    let init = lsp.initialize();
    assert!(init["capabilities"]["experimental"]["syscribe"]["lspRequests"].as_array().unwrap().iter().any(|r| r == "syscribe/propagate"));
    assert_eq!(init["capabilities"]["experimental"]["syscribe"]["features"]["interactiveConfiguration"], true);
    assert_eq!(lsp.custom_request("syscribe/propagate", args), cli);
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&m);
}

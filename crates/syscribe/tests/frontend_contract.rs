//! TC-TRS-FECI-001 (GH #430): the PLE fixture, and contract tests that run it against the real binary over every
//! transport — the CLI's JSON contracts, the language server's custom requests and the MCP server's tools — plus the
//! capabilities handshake a front end gates on. A front end's CI drives the same binary and fixture by whatever
//! means it likes (see docs/guides/frontend-ci.md); these tests are the reference for what it can expect.

mod common;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn fixture() -> PathBuf {
    repo().join("examples/ple-fixture/model")
}
fn expected() -> Value {
    serde_json::from_str(&std::fs::read_to_string(repo().join("examples/ple-fixture/EXPECTED.json")).unwrap()).unwrap()
}
fn run(model: Option<&Path>, args: &[&str]) -> (String, String, i32) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_syscribe"));
    if let Some(m) = model {
        c.arg("-m").arg(m);
    }
    let o = c.args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}
fn sy_json(model: &Path, args: &[&str]) -> Value {
    let (o, e, _) = run(Some(model), args);
    serde_json::from_str(&o).unwrap_or_else(|x| panic!("{args:?}: {x}: {o} / {e}"))
}
fn strings(v: &Value) -> Vec<String> {
    let mut s: Vec<String> = v.as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
    s.sort();
    s
}
fn capabilities() -> Value {
    serde_json::from_str(&run(None, &["capabilities", "--json"]).0).unwrap()
}

/// Validate `v` against the published schema `name` (`syscribe schema <name>`).
fn assert_valid(name: &str, v: &Value) {
    let (text, e, c) = run(None, &["schema", name]);
    assert_eq!(c, 0, "no schema {name}: {e}");
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let url = format!("https://syscribe.dev/schemas/{name}.schema.json");
    compiler.add_resource(&url, serde_json::from_str(&text).unwrap()).unwrap();
    let idx = compiler.compile(&url, &mut schemas).unwrap();
    schemas.validate(v, idx).unwrap_or_else(|e| panic!("{name}: {e:#}\n{v}"));
}

/// A writable copy of the fixture.
fn copy_fixture(tag: &str) -> PathBuf {
    let dst = std::env::temp_dir().join(format!("syscribe-fe-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dst);
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
    copy(&fixture(), &dst);
    dst
}

#[test]
fn the_fixture_contains_exactly_what_expected_json_says() {
    let e = expected();
    let v = sy_json(&fixture(), &["validate", "--json"]);
    let errors = v.as_array().unwrap().iter().filter(|f| f["severity"] == "error").count();
    assert_eq!(errors as u64, e["validate"]["errors"].as_u64().unwrap(), "{v}");
    let f = sy_json(&fixture(), &["features", "--json"]);
    let feats = f["features"].as_array().unwrap();
    assert_eq!(feats.len() as u64, e["features"]["count"].as_u64().unwrap());
    for (q, kind) in e["features"]["groups"].as_object().unwrap() {
        let found = feats.iter().find(|x| x["qualifiedName"] == q.as_str()).unwrap_or_else(|| panic!("{q}"));
        assert_eq!(found["groupKind"], *kind, "{q}");
    }
    for q in e["features"]["abstract"].as_array().unwrap() {
        let text = std::fs::read_to_string(fixture().join(q.as_str().unwrap().replace("::", "/")).join("_index.md")).unwrap();
        assert!(text.contains("isAbstract: true"), "{q}");
    }
    for (q, c) in e["features"]["crossTree"].as_object().unwrap() {
        let d = feats.iter().find(|x| x["qualifiedName"] == q.as_str()).unwrap();
        assert_eq!(strings(&d["requires"]), strings(&c["requires"]), "{q}");
        assert_eq!(strings(&d["excludes"]), strings(&c["excludes"]), "{q}");
    }
    let mut params: Vec<String> = feats.iter().flat_map(|x| x["parameters"].as_array().unwrap().iter().map(move |p| format!("{}.{}", x["qualifiedName"].as_str().unwrap(), p.as_str().unwrap_or_else(|| p["name"].as_str().unwrap())))).collect();
    params.sort();
    assert_eq!(params, strings(&e["features"]["parameters"]));
    for (id, stage) in e["configurations"]["stages"].as_object().unwrap() {
        assert_eq!(sy_json(&fixture(), &["timeline", id, "--json"])["bindingStage"], *stage, "{id}");
    }
    let tree = sy_json(&fixture(), &["children", "--view", "configurations", "--json"]);
    let roots: Vec<&str> = tree["children"].as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap()).collect();
    for id in e["configurations"]["unstaged"].as_array().unwrap() {
        assert!(roots.contains(&id.as_str().unwrap()), "{id} in {roots:?}");
    }
    let text = std::fs::read_to_string(fixture().join("Features/_index.md")).unwrap();
    for id in e["parameterConstraints"].as_array().unwrap() {
        assert!(text.contains(id.as_str().unwrap()), "{id}");
    }
}

#[test]
fn the_documented_samples_give_the_documented_answers() {
    let e = expected();
    let s = &e["samples"];
    let p = sy_json(&fixture(), &["propagate", "--select", "Features::Camera=true", "--json"]);
    assert_eq!(strings(&p["forcedTrue"]), strings(&s["propagate"]["forcedOn"]));
    assert_eq!(strings(&p["forcedFalse"]), strings(&s["propagate"]["forcedOff"]));
    let x = sy_json(&fixture(), &["explain-feature", "Features::Link::Cellular", "--select", "Features::Camera=true", "--json"]);
    assert_eq!(x["state"], s["explainFeature"]["state"]);
    let b = &s["bind"];
    let (conf, target) = (b["configuration"].as_str().unwrap(), b["target"].as_str().unwrap());
    let ok = sy_json(&fixture(), &["bind", conf, target, &b["okValue"].to_string(), "--dry-run", "--json"]);
    assert_eq!(ok["newErrors"], json!([]));
    let bad = sy_json(&fixture(), &["bind", conf, target, &b["badValue"].to_string(), "--dry-run", "--json"]);
    assert!(bad["newErrors"].to_string().contains(b["badCode"].as_str().unwrap()), "{bad}");
}

/// The command each CLI contract comes from, against the fixture.
fn cli_case(name: &str, scratch: &Path) -> Option<Vec<String>> {
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    Some(match name {
        "features" => s(&["features", "--json"]),
        "feature" => s(&["feature", "Features::Propulsion", "--json"]),
        "configure" => s(&["configure", "CONF-PLF-DEPLOY-001", "--json"]),
        "residual" => s(&["residual", "--config", "CONF-PLF-BUILD-001", "--json"]),
        "matrix" => s(&["matrix", "--json"]),
        "diff" => s(&["diff", "--config", "CONF-PLF-BUILD-001", "--config", "CONF-PLF-ALT-001", "--json"]),
        "feature-check" => s(&["feature-check", "--deep", "--json"]),
        "params-check" => {
            let file = scratch.join("params.yaml");
            let (yaml, ..) = run(Some(&fixture()), &["params", "export", "--config", "CONF-PLF-FLIGHT-001", "--allow-open"]);
            std::fs::write(&file, yaml).unwrap();
            s(&["params", "check", file.to_str().unwrap(), "--config", "CONF-PLF-FLIGHT-001", "--json"])
        }
        "propagate" => s(&["propagate", "--select", "Features::Camera=true", "--json"]),
        "explain-feature" => s(&["explain-feature", "Features::Link::Cellular", "--select", "Features::Camera=true", "--json"]),
        "children" => s(&["children", "Features", "--json"]),
        "change-impact" => s(&["change-impact", "Features::Camera", "--set", "requires=[Features::Platform::RiscV]", "--json"]),
        "bind" => s(&["bind", "CONF-PLF-DEPLOY-001", "Features::Propulsion.motorKV", "1100", "--dry-run", "--json"]),
        "scaffold" => s(&["new", "feature", "Gimbal", "--under", "Features", "--dry-run", "--json"]),
        "timeline" => s(&["timeline", "CONF-PLF-FLIGHT-001", "--json"]),
        "health" => s(&["health", "--json"]),
        _ => return None,
    })
}

#[test]
fn every_published_cli_contract_holds_on_the_fixture() {
    let caps = capabilities();
    let scratch = std::env::temp_dir().join(format!("syscribe-fe-scratch-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let mut checked = Vec::new();
    for (name, info) in caps["jsonSchemas"].as_object().unwrap() {
        if info["transport"] == "lsp" {
            continue;
        }
        // A contract added without a fixture case fails here, so the fixture keeps covering everything published.
        let argv = cli_case(name, &scratch).unwrap_or_else(|| panic!("the published contract '{name}' has no case in the fixture suite: add one"));
        let args: Vec<&str> = argv.iter().map(String::as_str).collect();
        let (out, e, _) = run(Some(&fixture()), &args);
        let v: Value = serde_json::from_str(&out).unwrap_or_else(|x| panic!("{name}: {x}: {out} / {e}"));
        assert_valid(name, &v);
        assert_eq!(v["schemaVersion"], info["version"], "{name}: schemaVersion is the advertised one");
        checked.push(name.clone());
    }
    assert!(checked.len() >= 16, "{checked:?}");
    let _ = std::fs::remove_dir_all(&scratch);
}

#[test]
fn every_advertised_lsp_request_answers_with_a_valid_payload() {
    let caps = capabilities();
    let work = copy_fixture("lsp");
    let mut lsp = common::Lsp::start(&work);
    let init = lsp.initialize_with_model_changed();
    // The handshake document is the CLI's.
    assert_eq!(init["capabilities"]["experimental"]["syscribe"], caps);
    assert_eq!(init["capabilities"]["semanticTokensProvider"]["legend"]["tokenTypes"].as_array().unwrap().len(), 4);
    assert!(init["capabilities"]["executeCommandProvider"]["commands"].as_array().unwrap().iter().any(|c| c == "syscribe.apply"));
    let cases: Vec<(&str, Value, Option<&str>)> = vec![
        ("syscribe/propagate", json!({"select": {"Features::Camera": true}}), Some("propagate")),
        ("syscribe/explainFeature", json!({"feature": "Features::Link::Cellular", "select": {"Features::Camera": true}}), Some("explain-feature")),
        ("syscribe/children", json!({"of": "Features"}), Some("children")),
        ("syscribe/changeImpact", json!({"element": "Features::Camera", "set": {"requires": ["Features::Platform::RiscV"]}}), Some("change-impact")),
        ("syscribe/timeline", json!({"configuration": "CONF-PLF-FLIGHT-001"}), Some("timeline")),
        ("syscribe/health", json!({}), Some("health")),
        ("syscribe/preview", json!({"action": "bind", "configuration": "CONF-PLF-DEPLOY-001", "target": "Features::Propulsion.motorKV", "value": 1100}), None),
    ];
    for m in caps["lspRequests"].as_array().unwrap() {
        let m = m.as_str().unwrap();
        let (_, params, schema) = cases.iter().find(|(n, ..)| *n == m).unwrap_or_else(|| panic!("the advertised request '{m}' has no case in the fixture suite: add one"));
        let r = lsp.custom_request(m, params.clone());
        match schema {
            Some(s) => assert_valid(s, &r),
            None => assert!(r["commitable"] == true && r["edit"].is_object() && r["undo"].is_object() && r["report"].is_object(), "{m}: {r}"),
        }
    }
    // A saved change is announced.
    let f = work.join("Requirements/REQ-PLF-099.md");
    std::fs::write(&f, "---\nid: REQ-PLF-099\ntype: Requirement\nname: R99\nstatus: draft\nreqDomain: system\n---\n\nAdded.\n").unwrap();
    lsp.did_save(&f, None);
    let n = lsp.wait_for_notification("syscribe/modelChanged");
    assert_valid("model-changed", &n);
    assert!(n["added"].to_string().contains("REQ-PLF-099"), "{n}");
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&work);
}

#[test]
fn the_mcp_server_offers_every_advertised_read_tool_and_hides_the_write_tools_when_read_only() {
    let caps = capabilities();
    let work = copy_fixture("mcp");
    let mut mcp = common::Mcp::start_with_args(&work, &["--read-only"]);
    mcp.initialize();
    let listed = mcp.tools_list().to_string();
    let write = ["bind", "scaffold", "update_element", "create_element", "delete_element", "move_element", "apply_changes", "question", "edit_feature", "ingest_results", "suspect_accept"];
    for w in write {
        assert!(!listed.contains(&format!("\"{w}\"")), "{w} is a write tool");
    }
    for t in caps["mcpTools"].as_array().unwrap().iter().map(|t| t.as_str().unwrap()).filter(|t| !write.contains(t)) {
        assert!(listed.contains(&format!("\"{t}\"")), "{t} is advertised but not offered");
    }
    for (tool, args, schema) in [
        ("health", json!({}), "health"),
        ("children", json!({"of": "Features"}), "children"),
        ("timeline", json!({"configuration": "CONF-PLF-FLIGHT-001"}), "timeline"),
        ("propagate", json!({"select": {"Features::Camera": true}}), "propagate"),
        ("explain_feature", json!({"feature": "Features::Link::Cellular", "select": {"Features::Camera": true}}), "explain-feature"),
        ("change_impact", json!({"element": "Features::Camera", "set": {"requires": ["Features::Platform::RiscV"]}}), "change-impact"),
    ] {
        assert_valid(schema, &mcp.call_tool(tool, args));
    }
    let _ = std::fs::remove_dir_all(&work);
}

#[test]
fn the_front_end_can_gate_on_capabilities() {
    // What a front end checks first: the version, and each feature it needs, so it degrades against an older binary.
    let caps = capabilities();
    assert!(caps["version"].as_str().unwrap().split('.').count() == 3);
    for f in expected()["frontendRequires"].as_array().unwrap() {
        assert_eq!(caps["features"][f.as_str().unwrap()], true, "{f}");
    }
    // A feature that does not exist is simply absent, never an error: a client can ask about newer capabilities.
    assert!(caps["features"]["notYetInvented"].is_null());
    // Every feature is a boolean, so `features[x] === true` is the whole test.
    assert!(caps["features"].as_object().unwrap().values().all(Value::is_boolean));
}

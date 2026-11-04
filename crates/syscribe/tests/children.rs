//! TC-TRS-CHILDREN-001 (GH #425): `children` — one call per expanded node, with counters and pages.

mod common;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(model: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

fn children(model: &Path, args: &[&str]) -> Value {
    let mut a = vec!["children"];
    a.extend_from_slice(args);
    a.push("--json");
    let (o, e, c) = run(model, &a);
    assert_eq!(c, 0, "{a:?}: {e}");
    let v: Value = serde_json::from_str(&o).unwrap();
    let s = Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["schema", "children"]).output().unwrap();
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let url = "https://syscribe.dev/schemas/children.schema.json";
    compiler.add_resource(url, serde_json::from_slice(&s.stdout).unwrap()).unwrap();
    let idx = compiler.compile(url, &mut schemas).unwrap();
    schemas.validate(&v, idx).unwrap_or_else(|e| panic!("{e:#}\n{v}"));
    v
}

fn ids(v: &Value) -> Vec<String> {
    v["children"].as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap().to_string()).collect()
}

/// Reqs/ (package with _index): REQ-CH-001 (derived children: -002, -003; two tests), Sub/REQ-CH-004 (no _index.md in Sub);
/// a broken requirement (error); Configurations: BASE <- LEAF (derivedFrom), TOP consolidating BASE (subConfigurations).
fn model(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-chld-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let w = |rel: &str, c: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    let req = |id: &str, extra: &str| format!("---\nid: {id}\ntype: Requirement\nname: R {id}\nstatus: draft\nreqDomain: system\n{extra}---\n\nbody\n");
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Reqs/_index.md", "---\ntype: Package\nname: Reqs\n---\n");
    w("Reqs/REQ-CH-001.md", &req("REQ-CH-001", ""));
    w("Reqs/REQ-CH-002.md", &req("REQ-CH-002", "derivedFrom: [REQ-CH-001]\n"));
    w("Reqs/REQ-CH-003.md", &req("REQ-CH-003", "derivedFrom: [REQ-CH-001]\n"));
    w("Reqs/Sub/REQ-CH-004.md", &req("REQ-CH-004", ""));
    w("Reqs/BAD-ONE.md", "---\ntype: Requirement\nid: not-an-id\nname: Broken\nstatus: draft\n---\n\nx\n");
    for i in 1..=2 {
        w(&format!("Tests/TC-CH-00{i}.md"), &format!("---\nid: TC-CH-00{i}\ntype: TestCase\nname: T{i}\nstatus: draft\ntestLevel: L2\nverifies: [REQ-CH-001]\n---\n\nb\n"));
    }
    let conf = |id: &str, extra: &str| format!("---\nid: {id}\ntype: Configuration\nname: {id}\n{extra}---\n\nc\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/Alpha.md", "---\nid: FEAT-CH-ALPHA\ntype: FeatureDef\nname: Alpha\n---\n\na\n");
    w("Configurations/_index.md", "---\ntype: Package\nname: Configurations\n---\n");
    w("Configurations/CONF-CH-BASE.md", &conf("CONF-CH-BASE", "features:\n  FEAT-CH-ALPHA: true\n"));
    w("Configurations/CONF-CH-LEAF.md", &conf("CONF-CH-LEAF", "derivedFrom: CONF-CH-BASE\n"));
    w("Configurations/CONF-CH-TOP.md", &conf("CONF-CH-TOP", "subConfigurations: [CONF-CH-BASE]\n"));
    root
}

#[test]
fn the_root_and_a_package_list_their_direct_children_with_counters() {
    let m = model("tree");
    let root = children(&m, &[]);
    assert_eq!(ids(&root), vec!["Configurations", "Features", "Reqs", "Tests"]);
    assert_eq!(root["total"], 4);
    let reqs = children(&m, &["Reqs"]);
    // Packages first (Sub, virtual: no _index.md), then elements by name.
    assert_eq!(ids(&reqs), vec!["Reqs::Sub", "not-an-id", "REQ-CH-001", "REQ-CH-002", "REQ-CH-003"]);
    let sub = &reqs["children"][0];
    assert_eq!((sub["package"].clone(), sub["virtual"].clone(), sub["hasChildren"].clone()), (json!(true), json!(true), json!(true)));
    let r1 = reqs["children"].as_array().unwrap().iter().find(|c| c["id"] == "REQ-CH-001").unwrap();
    assert_eq!(r1["counters"]["derivedChildren"], 2);
    assert_eq!(r1["counters"]["verifyingTests"], 2);
    assert_eq!(r1["package"], false);
    assert!(r1["file"].as_str().unwrap().ends_with("REQ-CH-001.md"));
    // Errors: the broken requirement carries them; the package rolls its subtree up; a clean package has none.
    let bad = reqs["children"].as_array().unwrap().iter().find(|c| c["id"] == "not-an-id").unwrap();
    assert!(bad["counters"]["errors"].as_u64().unwrap() >= 1, "{bad}");
    let top = children(&m, &[]);
    let reqs_node = top["children"].as_array().unwrap().iter().find(|c| c["id"] == "Reqs").unwrap();
    assert!(reqs_node["counters"]["errors"].as_u64().unwrap() >= bad["counters"]["errors"].as_u64().unwrap(), "subtree roll-up");
    let features = top["children"].as_array().unwrap().iter().find(|c| c["id"] == "Features").unwrap();
    assert_eq!(features["counters"]["errors"], 0);
    // A virtual package expands by its qualified name.
    assert_eq!(ids(&children(&m, &["Reqs::Sub"])), vec!["REQ-CH-004"]);
    // An id works as well as a qualified name.
    assert_eq!(children(&m, &["REQ-CH-001"])["total"], 0);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn pages_cover_the_children_exactly_once() {
    let m = model("page");
    let all = ids(&children(&m, &["Reqs", "--limit", "100"]));
    let mut seen = Vec::new();
    let mut offset = 0usize;
    loop {
        let p = children(&m, &["Reqs", "--limit", "2", "--offset", &offset.to_string()]);
        assert!(p["children"].as_array().unwrap().len() <= 2);
        seen.extend(ids(&p));
        match p["nextOffset"].as_u64() {
            Some(n) => offset = n as usize,
            None => break,
        }
    }
    assert_eq!(seen, all);
    assert_eq!(children(&m, &["Reqs", "--offset", "99"])["children"], json!([]));
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn the_configuration_tree_follows_derived_from_and_sub_configurations() {
    let m = model("conf");
    let roots = children(&m, &["--view", "configurations"]);
    // BASE is derived from by LEAF and consolidated by TOP, so only TOP is a root.
    assert_eq!(ids(&roots), vec!["CONF-CH-TOP"]);
    assert_eq!(roots["children"][0]["relation"], "root");
    let top = children(&m, &["CONF-CH-TOP", "--view", "configurations"]);
    assert_eq!(ids(&top), vec!["CONF-CH-BASE"]);
    assert_eq!(top["children"][0]["relation"], "subConfiguration");
    let base = children(&m, &["CONF-CH-BASE", "--view", "configurations"]);
    assert_eq!(ids(&base), vec!["CONF-CH-LEAF"]);
    assert_eq!(base["children"][0]["relation"], "derivedFrom");
    assert_eq!(top["children"][0]["hasChildren"], true);
    assert_eq!(base["children"][0]["hasChildren"], false);
    let (_o, e, c) = run(&m, &["children", "Reqs::REQ-CH-001", "--view", "configurations"]);
    assert_eq!(c, 1, "{e}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn errors_usage_and_read_only() {
    let m = model("err");
    let (_o, e, c) = run(&m, &["children", "Nope::Nothing"]);
    assert_eq!(c, 1);
    assert!(e.contains("not found"), "{e}");
    for bad in [&["children", "--view", "wide"][..], &["children", "--offset", "x"], &["children", "A", "B"]] {
        let (_o, _e, c) = run(&m, bad);
        assert_eq!(c, 2, "{bad:?}");
    }
    // A missing value is caught by the shared argument check, which exits 1 for every command.
    assert_eq!(run(&m, &["children", "--limit"]).2, 1);
    let before = common::dir_hash(&m);
    let (text, _e, c) = run(&m, &["children", "Reqs"]);
    assert_eq!(c, 0);
    assert!(text.contains("REQ-CH-001") && text.contains("2 err") == false && text.contains("child"), "{text}");
    assert_eq!(common::dir_hash(&m), before);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn the_mcp_tool_and_the_lsp_request_return_the_same_page() {
    let m = model("surf");
    let cli = children(&m, &["Reqs", "--limit", "3", "--offset", "1"]);
    let args = json!({"of": "Reqs", "limit": 3, "offset": 1});
    let mut mcp = common::Mcp::start(&m);
    mcp.initialize();
    assert_eq!(mcp.call_tool("children", args.clone()), cli);
    assert_eq!(mcp.call_tool_raw("children", json!({"of": "Nope::X"}))["isError"], true);
    let mut lsp = common::Lsp::start(&m);
    let init = lsp.initialize();
    assert!(init["capabilities"]["experimental"]["syscribe"]["lspRequests"].as_array().unwrap().iter().any(|r| r == "syscribe/children"));
    assert_eq!(lsp.custom_request("syscribe/children", args), cli);
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&m);
}

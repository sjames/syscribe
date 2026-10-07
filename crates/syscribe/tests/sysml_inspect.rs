//! `REQ-TRS-SYSMLV2-031` (CLI `sysml`) and `REQ-TRS-SYSMLV2-032` (MCP `sysml_submodels`).

mod common;
use common::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// A model with one submodel (a part def, a port def and a metadata def) beside the fixture content.
fn sysml_model() -> PathBuf {
    let root = fixture_copy();
    write(&root, "Sub/_index.md", "---\ntype: Package\nname: Sub\nsysmlSubmodel: true\n---\n");
    write(
        &root,
        "Sub/A.sysml",
        "package P {\n  part def X;\n  port def Pt;\n  metadata def C;\n}\n",
    );
    root
}

fn run(root: &Path, args: &[&str]) -> (String, bool) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.success())
}

#[test]
fn cli_sysml_text_report() {
    let root = sysml_model();
    let (out, ok) = run(&root, &["sysml"]);
    assert!(ok);
    assert!(out.contains("SysMLv2 submodel `Sub`"), "{out}");
    assert!(out.contains("Files parsed: 1/1"), "{out}");
    assert!(out.contains("PartDef: 1"), "{out}");
    assert!(out.contains("metadata def x1"), "{out}");
    assert!(out.contains("W543"), "{out}");
}

#[test]
fn cli_sysml_json_report() {
    let root = sysml_model();
    let (out, ok) = run(&root, &["sysml", "--json"]);
    assert!(ok);
    let v: Value = serde_json::from_str(&out).unwrap();
    let s = &v["submodels"][0];
    assert_eq!(s["package"], "Sub");
    assert_eq!(s["elementsByKind"]["PortDef"], 1);
    assert_eq!(s["unmapped"]["metadata def"], 1);
    assert_eq!(s["findings"][0]["code"], "W543");
}

#[test]
fn cli_sysml_without_submodels_exits_zero() {
    let root = fixture_copy();
    let (out, ok) = run(&root, &["sysml"]);
    assert!(ok);
    assert!(out.contains("No SysMLv2 submodels"), "{out}");
    let (out, ok) = run(&root, &["sysml", "--json"]);
    assert!(ok);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert!(v["submodels"].as_array().unwrap().is_empty());
}

#[test]
fn mcp_sysml_submodels_matches_cli_json_and_is_read_only() {
    let root = sysml_model();
    let (out, _) = run(&root, &["sysml", "--json"]);
    let cli: Value = serde_json::from_str(&out).unwrap();

    let mut mcp = Mcp::start(&root);
    mcp.initialize();
    let tools = mcp.tools_list();
    let tool = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "sysml_submodels")
        .expect("sysml_submodels registered");
    assert_eq!(tool["annotations"]["readOnlyHint"], json!(true));

    let before = dir_hash(&root);
    let res = mcp.call_tool("sysml_submodels", json!({}));
    assert_eq!(res, cli);
    assert_eq!(dir_hash(&root), before, "read-only tool must not touch disk");
}

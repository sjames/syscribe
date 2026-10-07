//! `REQ-TRS-SYSMLV2-037` (CLI `export-sysml`) and `REQ-TRS-SYSMLV2-042` (MCP `export_sysml`).

mod common;
use common::*;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn export_model() -> PathBuf {
    let root = fixture_copy();
    write(&root, "Exp/_index.md", "---\ntype: Package\nname: Exp\n---\n");
    write(&root, "Exp/Motor.md", "---\ntype: PartDef\nname: Motor\n---\nA motor.\n");
    write(&root, "Exp/Other.md", "---\ntype: PartDef\nname: Other\n---\n");
    root
}

fn run(root: &Path, args: &[&str]) -> (String, String, bool) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

#[test]
fn cli_export_sysml_to_stdout_with_stderr_summary() {
    let root = export_model();
    let (out, err, ok) = run(&root, &["export-sysml"]);
    assert!(ok, "{err}");
    assert!(out.contains("package Exp {"), "{out}");
    assert!(out.contains("part def Motor"), "{out}");
    assert!(out.contains("doc /* A motor. */"), "{out}");
    assert!(err.contains("export-sysml: exported"), "{err}");
    assert!(!out.contains("export-sysml: exported"), "summary must not pollute stdout");
}

#[test]
fn cli_export_sysml_scope_and_unknown_scope() {
    let root = export_model();
    let (out, _, ok) = run(&root, &["export-sysml", "Exp::Motor"]);
    assert!(ok);
    assert!(out.contains("part def Motor"), "{out}");
    assert!(!out.contains("Other"), "{out}");
    let (_, err, ok) = run(&root, &["export-sysml", "No::Such"]);
    assert!(!ok);
    assert!(err.contains("No::Such"), "{err}");
}

#[test]
fn cli_export_sysml_out_file_and_dir_leave_model_untouched() {
    let root = export_model();
    let before = dir_hash(&root);
    let dest = root.parent().unwrap().join("out");
    let file = dest.join("all.sysml");
    let (stdout, _, ok) = run(&root, &["export-sysml", "--out", file.to_str().unwrap()]);
    assert!(ok);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(std::fs::read_to_string(&file).unwrap().contains("package Exp {"));

    let dir = dest.join("parts/");
    let (_, _, ok) = run(&root, &["export-sysml", "--out", dir.to_str().unwrap()]);
    assert!(ok);
    let exp = std::fs::read_to_string(dest.join("parts/Exp.sysml")).unwrap();
    assert!(exp.contains("part def Other"), "{exp}");
    assert_eq!(dir_hash(&root), before, "export must not modify the model");
}

#[test]
fn mcp_export_sysml_matches_cli_and_is_read_only() {
    let root = export_model();
    let (cli, _, _) = run(&root, &["export-sysml", "Exp"]);

    let mut mcp = Mcp::start(&root);
    mcp.initialize();
    let tools = mcp.tools_list();
    let tool = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "export_sysml")
        .expect("export_sysml registered");
    assert_eq!(tool["annotations"]["readOnlyHint"], json!(true));

    let before = dir_hash(&root);
    let res = mcp.call_tool_raw("export_sysml", json!({"package": "Exp"}));
    let text = res["content"][0]["text"].as_str().expect("text content");
    assert_eq!(text, cli);
    let bad = mcp.call_tool_raw("export_sysml", json!({"package": "No::Such"}));
    assert_eq!(bad["isError"], json!(true), "{bad}");
    assert_eq!(dir_hash(&root), before, "read-only tool must not touch disk");
}

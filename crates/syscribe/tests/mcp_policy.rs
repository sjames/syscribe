//! TC-TRS-MCPPOLICY-001 (GH #442): every MCP write tool is held to the project's agent policy.

mod common;
use common::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const POLICY: &str = "\n[agents]\nprotected_paths = [\"model/Safety/**\"]\nhuman_only_status = { Requirement = [\"approved\"] }\nrequire_before_done = [\"evidence\"]\n";

fn git(root: &Path, args: &[&str]) {
    let o = Command::new("git").arg("-C").arg(root).args(args).output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

fn req(id: &str, status: &str) -> String {
    format!("---\nid: {id}\ntype: Requirement\nname: {id}\nstatus: {status}\nreqDomain: software\nreqClass: system\n---\n\nShall.\n")
}

/// A fixture model in a git repository (so the policy's `model/…` paths resolve), with the policy appended.
fn setup(policy: &str) -> (PathBuf, PathBuf) {
    let model = fixture_copy();
    let root = model.parent().unwrap().to_path_buf();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.email", "h@example.com"]);
    git(&root, &["config", "user.name", "H"]);
    git(&root, &["config", "commit.gpgsign", "false"]);
    let toml = model.join(".syscribe.toml");
    std::fs::write(&toml, std::fs::read_to_string(&toml).unwrap_or_default() + policy).unwrap();
    let w = |rel: &str, c: &str| {
        let p = model.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("Safety/REQ-SP-002.md", &req("REQ-SP-002", "draft"));
    w("Requirements/REQ-SP-001.md", &req("REQ-SP-001", "draft"));
    w("Parts/Alpha.md", "---\ntype: PartDef\nname: Alpha\n---\n\nA.\n");
    w("Safety/Hazard.md", "---\ntype: PartDef\nname: Hazard\nsupertype: Parts::Alpha\n---\n\nH.\n");
    w("Planning/PI-SP-001.md", "---\nid: PI-SP-001\ntype: PlanningItem\nname: P\nstatus: todo\nachieves: [REQ-SP-001]\nquestions:\n  - id: Q1\n    text: q\n    status: open\n---\n\nx\n");
    w("Safety/PI-SP-009.md", "---\nid: PI-SP-009\ntype: PlanningItem\nname: Protected item\nstatus: todo\nachieves: [REQ-SP-001]\nquestions:\n  - id: Q1\n    text: q\n    status: open\n---\n\nx\n");
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "base"]);
    (root, model)
}

fn refused(r: &Value) -> String {
    let s = serde_json::to_string(r).unwrap();
    assert!(r["isError"] == true || r["written"] == false, "expected a refusal: {s}");
    s
}

#[test]
fn a_protected_path_cannot_be_written_by_any_write_tool() {
    let (_root, model) = setup(POLICY);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let before = std::fs::read_to_string(model.join("Safety/REQ-SP-002.md")).unwrap();
    for (tool, args) in [
        ("update_element", json!({"ref": "REQ-SP-002", "fields": {"status": "review"}, "dry_run": false})),
        ("delete_element", json!({"ref": "REQ-SP-002", "dry_run": false})),
        ("question", json!({"item": "PI-SP-009", "op": "defer", "id": "Q1", "dry_run": false})),
        ("create_element", json!({"type": "Requirement", "parent": "Safety", "fields": {"id": "REQ-SP-003", "name": "n", "status": "draft", "reqDomain": "software", "reqClass": "system"}, "doc": "Shall.\n", "dry_run": false})),
    ] {
        let r = mcp.call_tool_raw(tool, args);
        assert_eq!(r["isError"], true, "{tool}: {r}");
        assert!(serde_json::to_string(&r).unwrap().contains("protected_paths"), "{tool} names the rule: {r}");
    }
    assert_eq!(std::fs::read_to_string(model.join("Safety/REQ-SP-002.md")).unwrap(), before);
    assert!(!model.join("Safety/REQ-SP-003.md").exists());
    // Elsewhere is fine, and reading a protected element is always fine.
    let ok = mcp.call_tool("update_element", json!({"ref": "REQ-SP-001", "fields": {"status": "review"}, "dry_run": false}));
    assert_eq!(ok["written"], true, "{ok}");
    assert!(mcp.call_tool("get_element", json!({"ref": "REQ-SP-002"}))["id"].is_string());
}

#[test]
fn a_move_that_would_rewrite_a_protected_file_is_refused() {
    let (_root, model) = setup(POLICY);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let r = mcp.call_tool_raw("move_element", json!({"ref": "Parts::Alpha", "dest": "Parts::Beta", "dry_run": false}));
    assert_eq!(r["isError"], true, "{r}");
    assert!(serde_json::to_string(&r).unwrap().contains("Hazard.md"), "{r}");
    assert!(model.join("Parts/Alpha.md").exists() && !model.join("Parts/Beta.md").exists());
}

#[test]
fn a_human_only_status_cannot_be_set_over_mcp() {
    let (_root, model) = setup(POLICY);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    for (tool, args) in [
        ("update_element", json!({"ref": "REQ-SP-001", "fields": {"status": "approved"}, "dry_run": false})),
        ("create_element", json!({"type": "Requirement", "parent": "Requirements", "fields": {"id": "REQ-SP-004", "name": "n", "status": "approved", "reqDomain": "software", "reqClass": "system"}, "doc": "Shall.\n", "dry_run": false})),
    ] {
        let r = mcp.call_tool_raw(tool, args);
        assert_eq!(r["isError"], true, "{tool}: {r}");
        assert!(serde_json::to_string(&r).unwrap().contains("human_only_status"), "{tool}: {r}");
    }
    assert!(std::fs::read_to_string(model.join("Requirements/REQ-SP-001.md")).unwrap().contains("status: draft"));
    assert!(!model.join("Requirements/REQ-SP-004.md").exists());
}

#[test]
fn an_item_without_evidence_cannot_be_marked_done_over_mcp() {
    let (_root, model) = setup(POLICY);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let r = mcp.call_tool_raw("update_element", json!({"ref": "PI-SP-001", "fields": {"status": "done"}, "dry_run": false}));
    assert_eq!(r["isError"], true, "{r}");
    assert!(serde_json::to_string(&r).unwrap().contains("require_before_done"), "{r}");
    let ok = mcp.call_tool("update_element", json!({"ref": "PI-SP-001", "fields": {"status": "in_progress"}, "dry_run": false}));
    assert_eq!(ok["written"], true, "{ok}");
}

#[test]
fn a_batch_is_refused_whole_when_any_operation_breaks_the_policy() {
    let (_root, model) = setup(POLICY);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let before = std::fs::read_to_string(model.join("Requirements/REQ-SP-001.md")).unwrap();
    let r = mcp.call_tool(
        "apply_changes",
        json!({"operations": [
            {"op": "update", "ref": "REQ-SP-001", "fields": {"status": "review"}},
            {"op": "update", "ref": "REQ-SP-002", "fields": {"status": "review"}}
        ], "dry_run": false}),
    );
    let s = refused(&r);
    assert!(s.contains("protected_paths"), "{s}");
    assert_eq!(std::fs::read_to_string(model.join("Requirements/REQ-SP-001.md")).unwrap(), before, "all-or-nothing: the permitted first operation was not applied either");

    let r = mcp.call_tool("apply_changes", json!({"operations": [{"op": "update", "ref": "REQ-SP-001", "fields": {"status": "approved"}}], "dry_run": false}));
    assert!(refused(&r).contains("human_only_status"));
    let ok = mcp.call_tool("apply_changes", json!({"operations": [{"op": "update", "ref": "REQ-SP-001", "fields": {"status": "review"}}], "dry_run": false}));
    assert_eq!(ok["written"], true, "{ok}");
}

#[test]
fn the_instructions_state_the_policy_only_when_there_is_one() {
    let (_root, model) = setup(POLICY);
    let mut mcp = Mcp::start(&model);
    let text = mcp.initialize()["instructions"].as_str().unwrap_or_default().to_string();
    assert!(text.contains("Project policy") && text.contains("`model/Safety/**`") && text.contains("Requirement to `approved`"), "{text}");

    let (_r2, plain) = setup("");
    let mut mcp = Mcp::start(&plain);
    let text = mcp.initialize()["instructions"].as_str().unwrap_or_default().to_string();
    assert!(!text.contains("Project policy"), "{text}");
}

#[test]
fn an_item_cannot_be_created_already_done_without_evidence() {
    let (_root, model) = setup(POLICY);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let item = |id: &str, evidence: bool| {
        let mut f = json!({"id": id, "name": "n", "status": "done", "achieves": ["REQ-SP-001"]});
        if evidence {
            f["evidence"] = json!([{"ref": "REQ-SP-001"}]);
        }
        f
    };
    let bad = mcp.call_tool_raw("create_element", json!({"type": "PlanningItem", "parent": "Planning", "fields": item("PI-SP-100", false), "doc": "x\n", "dry_run": false}));
    assert_eq!(bad["isError"], true, "{bad}");
    assert!(serde_json::to_string(&bad).unwrap().contains("require_before_done"), "{bad}");
    assert!(!model.join("Planning/PI-SP-100.md").exists());
    let ok = mcp.call_tool("create_element", json!({"type": "PlanningItem", "parent": "Planning", "fields": item("PI-SP-101", true), "doc": "x\n", "dry_run": false}));
    assert_eq!(ok["written"], true, "{ok}");
    // The same rule inside a batch.
    let batch = mcp.call_tool("apply_changes", json!({"operations": [{"op": "create", "type": "PlanningItem", "parent": "Planning", "fields": item("PI-SP-102", false), "doc": "x\n"}], "dry_run": false}));
    assert!(refused(&batch).contains("require_before_done"));
    assert!(!model.join("Planning/PI-SP-102.md").exists());
}

/// TC-TRS-AGENTPOL-004 (GH #457): no MCP write tool answers a question reserved to a person.
#[test]
fn no_write_tool_answers_a_question_reserved_to_a_person() {
    let (_root, model) = setup("\n[agents]\nhuman_only_answers = true\n");
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let file = model.join("Planning/PI-SP-001.md");
    let before = std::fs::read_to_string(&file).unwrap();
    let answered = json!([{"id": "Q1", "text": "q", "status": "answered", "answer": "a", "answeredBy": "owner"}]);
    for dry_run in [true, false] {
        for (tool, args) in [
            ("question", json!({"item": "PI-SP-001", "op": "answer", "id": "Q1", "text": "a", "dry_run": dry_run})),
            ("update_element", json!({"ref": "PI-SP-001", "fields": {"questions": answered.clone()}, "dry_run": dry_run})),
            ("apply_changes", json!({"operations": [{"op": "update", "ref": "PI-SP-001", "fields": {"questions": answered.clone()}}], "dry_run": dry_run})),
        ] {
            let r = mcp.call_tool_raw(tool, args);
            let s = refused(&r);
            assert!(s.contains("human_only_answers") && s.contains("PI-SP-001#Q1"), "{tool} (dry_run {dry_run}): {s}");
        }
    }
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before, "nothing was written");
    // Recording a question is still fine.
    let ok = mcp.call_tool("question", json!({"item": "PI-SP-001", "op": "add", "text": "another", "dry_run": false}));
    assert_eq!(ok["written"], true, "{ok}");
    // questions exposes who may answer.
    let qs = mcp.call_tool("questions", json!({"item": "PI-SP-001"}));
    assert_eq!(qs["questions"][0]["answerBy"], "human", "{qs}");
}

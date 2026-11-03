//! TC-TRS-PIQ-004 (GH #436): MCP `questions`, `steps` and the guarded `question` tool.

mod common;
use common::*;
use serde_json::json;
use std::collections::BTreeSet;
use std::path::Path;

fn setup() -> std::path::PathBuf {
    let model = fixture_copy();
    let w = |name: &str, body: &str| std::fs::write(model.join(name), body).unwrap();
    w("PI-MQ-001.md", "---\nid: PI-MQ-001\ntype: PlanningItem\nname: Top\nstatus: in_progress\nachieves: [REQ-FX-001]\nquestions:\n  - id: Q1\n    text: Which UART?\n    status: open\n---\n\nx\n");
    w("PI-MQ-002.md", "---\nid: PI-MQ-002\ntype: PlanningItem\nname: Step one\nstatus: done\nparent: PI-MQ-001\nevidence:\n  - ref: REQ-FX-001\n---\n\nx\n");
    w("PI-MQ-003.md", "---\nid: PI-MQ-003\ntype: PlanningItem\nname: Step two\nstatus: todo\nparent: PI-MQ-001\n---\n\nx\n");
    model
}

fn read(model: &Path) -> String {
    std::fs::read_to_string(model.join("PI-MQ-001.md")).unwrap()
}

#[test]
fn questions_and_steps_are_read_tools() {
    let model = setup();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let q = mcp.call_tool("questions", json!({}));
    assert_eq!(q["questions"][0]["item"], "PI-MQ-001");
    assert_eq!(q["questions"][0]["id"], "Q1");
    assert_eq!(mcp.call_tool("questions", json!({"open": true}))["questions"].as_array().unwrap().len(), 1);
    assert!(mcp.call_tool("questions", json!({"status": "answered"}))["questions"].as_array().unwrap().is_empty());
    assert_eq!(mcp.call_tool_raw("questions", json!({"status": "bogus"}))["isError"], true);
    assert_eq!(mcp.call_tool_raw("questions", json!({"item": "REQ-FX-001"}))["isError"], true);

    let s = mcp.call_tool("steps", json!({"item": "PI-MQ-001"}));
    assert_eq!(s["rollup"]["leaves"], 2);
    assert_eq!(s["rollup"]["done"], 1);
    assert_eq!(s["tree"]["openQuestions"], 1);
    assert_eq!(s["tree"]["children"].as_array().unwrap().len(), 2);
    let open = mcp.call_tool("steps", json!({"item": "PI-MQ-001", "open": true}));
    assert_eq!(open["tree"]["children"].as_array().unwrap().len(), 1, "the done step is hidden");
    assert_eq!(open["rollup"]["leaves"], 2, "the roll-up still counts it");
    assert_eq!(mcp.call_tool_raw("steps", json!({"item": "nope"}))["isError"], true);
}

#[test]
fn the_question_tool_dry_runs_by_default_then_commits_through_the_guarded_write() {
    let model = setup();
    let before = read(&model);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let dry = mcp.call_tool("question", json!({"item": "PI-MQ-001", "op": "add", "text": "Which pin?", "recommended": "PA9"}));
    assert_eq!(dry["written"], false, "{dry}");
    assert!(dry["diff"].as_str().unwrap().contains("+  - id: Q2"), "{dry}");
    assert_eq!(read(&model), before, "a dry run writes nothing");

    let done = mcp.call_tool("question", json!({"item": "PI-MQ-001", "op": "add", "text": "Which pin?", "recommended": "PA9", "dry_run": false}));
    assert_eq!(done["written"], true, "{done}");
    assert!(read(&model).contains("  - id: Q2\n    text: Which pin?\n    recommended: PA9\n    status: open\n"));
    assert_eq!(mcp.call_tool("questions", json!({"open": true}))["questions"].as_array().unwrap().len(), 2, "the server sees its own write");

    let ans = mcp.call_tool("question", json!({"item": "PI-MQ-001", "op": "answer", "id": "Q1", "text": "UART2", "by": "agent-7", "dry_run": false}));
    assert_eq!(ans["written"], true, "{ans}");
    let q1 = mcp.call_tool("questions", json!({"status": "answered"}));
    assert_eq!(q1["questions"][0]["answer"], "UART2");
    assert_eq!(q1["questions"][0]["answeredBy"], "agent-7");

    let defer = mcp.call_tool("question", json!({"item": "PI-MQ-001", "op": "defer", "id": "Q2", "dry_run": false}));
    assert_eq!(defer["written"], true, "{defer}");
    assert!(mcp.call_tool("questions", json!({"open": true}))["questions"].as_array().unwrap().is_empty());
}

#[test]
fn the_question_tool_refuses_bad_input_and_writes_nothing() {
    let model = setup();
    let before = read(&model);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    for args in [
        json!({"item": "PI-MQ-001", "op": "answer", "id": "Q9", "text": "x", "dry_run": false}),
        json!({"item": "PI-MQ-001", "op": "answer", "id": "Q1", "dry_run": false}),
        json!({"item": "PI-MQ-001", "op": "add", "dry_run": false}),
        json!({"item": "PI-MQ-001", "op": "explode", "dry_run": false}),
        json!({"item": "REQ-FX-001", "op": "add", "text": "x", "dry_run": false}),
        json!({"item": "nope", "op": "add", "text": "x", "dry_run": false}),
    ] {
        let r = mcp.call_tool_raw("question", args.clone());
        assert_eq!(r["isError"], true, "{args}: {r}");
    }
    assert_eq!(read(&model), before);
}

#[test]
fn a_read_only_server_offers_the_read_tools_but_not_question() {
    let model = setup();
    let mut mcp = Mcp::start_with_args(&model, &["--read-only"]);
    let text = mcp.initialize()["instructions"].as_str().unwrap_or_default().to_string();
    let tools: BTreeSet<String> = mcp.tools_list()["tools"].as_array().unwrap().iter().filter_map(|t| t["name"].as_str().map(String::from)).collect();
    assert!(tools.contains("questions") && tools.contains("steps"));
    assert!(!tools.contains("question"), "{tools:?}");
    assert!(!text.contains("`question` tool"), "no write advice on a read-only server: {text}");
    assert_eq!(mcp.call_tool_raw("question", json!({"item": "PI-MQ-001", "op": "add", "text": "x", "dry_run": false}))["isError"], true);
}

#[test]
fn instructions_and_describe_type_mention_questions() {
    let model = setup();
    let mut mcp = Mcp::start(&model);
    let text = mcp.initialize()["instructions"].as_str().unwrap_or_default().to_string();
    assert!(text.contains("record it with the `question` tool"), "{text}");
    let d = mcp.call_tool("describe_type", json!({"type": "PlanningItem"}));
    assert!(d["fields"].as_array().unwrap().iter().any(|f| f["name"] == "questions"), "{d}");
}

//! TC-TRS-MCPGUIDE-001 (GH #440): generated server instructions, workflow prompts, and
//! `mcp --print-instructions`; plus the `changes` tool and `validate new_since` the review
//! prompt relies on.

mod common;
use common::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

fn instructions(res: &Value) -> String {
    res.get("instructions").and_then(|i| i.as_str()).unwrap_or_default().to_string()
}

fn prompt_args() -> Vec<(&'static str, Value)> {
    vec![
        ("add-requirement-with-test", json!({"statement": "The system shall log faults.", "parent": "REQ-FX-001"})),
        ("record-test-evidence", json!({"results_path": "results.xml", "format": "junit"})),
        ("review-change-set", json!({"since": "main", "until": "HEAD"})),
        ("close-planning-item", json!({"item": "PI-FX-001"})),
        ("check-variant", json!({"config": "CONF-FX-001"})),
    ]
}

#[test]
fn initialize_instructions_name_the_projects_real_prefixes_and_statuses() {
    let model = fixture_copy(); // its .syscribe.toml adds the Requirement prefix FXREQ
    let mut mcp = Mcp::start(&model);
    let text = instructions(&mcp.initialize());
    assert!(text.contains("Requirement REQ (also FXREQ)"), "{text}");
    assert!(text.contains("Requirement draft|review|approved|implemented|verified"), "{text}");
    assert!(text.contains("PlanningItem todo|in_progress|blocked|done"), "{text}");
    // The model-independent guidance is still there.
    assert!(text.contains("reloads automatically") && text.contains("next_id"), "{text}");
    assert!(text.contains("claimedBy"), "ownership rule: {text}");
    assert!(!text.contains("READ-ONLY"));
}

#[test]
fn a_read_only_server_says_so_and_offers_no_write_sequence() {
    let model = fixture_copy();
    let mut mcp = Mcp::start_with_args(&model, &["--read-only"]);
    let text = instructions(&mcp.initialize());
    assert!(text.contains("READ-ONLY"), "{text}");
    assert!(!text.contains("create_element"), "no write sequence advertised: {text}");
}

#[test]
fn print_instructions_matches_the_initialize_response() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    let served = instructions(&mcp.initialize());
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&model).args(["mcp", "--print-instructions"]).output().unwrap();
    assert!(o.status.success());
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim_end(), served.trim_end());
}

#[test]
fn the_workflow_prompts_are_listed_with_their_arguments() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let list = mcp.prompts_list();
    let prompts = list["prompts"].as_array().expect("prompts array");
    for (name, _) in prompt_args() {
        let p = prompts.iter().find(|p| p["name"] == name).unwrap_or_else(|| panic!("{name} not listed"));
        assert!(p["description"].as_str().is_some_and(|d| !d.is_empty()));
    }
    let args_of = |n: &str| -> Vec<String> {
        prompts.iter().find(|p| p["name"] == n).unwrap()["arguments"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x["name"].as_str().map(String::from)).collect())
            .unwrap_or_default()
    };
    assert_eq!(args_of("review-change-set"), ["since", "until"]);
    assert_eq!(args_of("close-planning-item"), ["item"]);
    // The older prompts remain.
    assert!(prompts.iter().any(|p| p["name"] == "add-requirement"));
}

/// Every snake_case token in backticks that a prompt names must be a real tool, so a prompt can
/// never send a client to a tool the server does not have.
#[test]
fn every_tool_a_prompt_names_exists() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let tools: BTreeSet<String> = mcp.tools_list()["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();
    // Arguments, not tools.
    let not_tools = ["dry_run", "new_since", "results_path"];
    let must_use: &[(&str, &[&str])] = &[
        ("add-requirement-with-test", &["next_id", "template", "create_element", "validate_element", "validate", "trace"]),
        ("record-test-evidence", &["ingest_results", "evidence", "coverage", "validate"]),
        ("review-change-set", &["changes", "validate", "impact", "trace"]),
        ("close-planning-item", &["get_element", "evidence", "update_element", "validate_element"]),
        ("check-variant", &["configure", "project", "feature_check", "diff_configs", "why_active"]),
    ];
    let snake = regex::Regex::new(r"`([a-z]+(?:_[a-z]+)+)`|`([a-z]+(?:_[a-z]+)*)`").unwrap();
    for (name, args) in prompt_args() {
        let got = mcp.prompts_get_with(name, args);
        let text = serde_json::to_string(&got).unwrap().replace("\\n", "\n");
        assert!(!text.contains("<target>"), "{name}: placeholder leaked");
        for want in must_use.iter().find(|(n, _)| *n == name).unwrap().1 {
            assert!(text.contains(&format!("`{want}`")), "{name} should name `{want}`:\n{text}");
            assert!(tools.contains(*want), "{name} names `{want}`, which is not a tool");
        }
        for c in snake.captures_iter(&text) {
            let tok = c.get(1).or(c.get(2)).unwrap().as_str();
            // Only judge the snake_case ones strictly; single words may be fields or CLI words.
            if tok.contains('_') && !not_tools.contains(&tok) && !["in_progress"].contains(&tok) {
                assert!(tools.contains(tok), "{name} names `{tok}`, which is not a tool");
            }
        }
    }
}

#[test]
fn prompt_arguments_are_substituted() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let t = |r: Value| serde_json::to_string(&r).unwrap();
    assert!(t(mcp.prompts_get_with("close-planning-item", json!({"item": "PI-FX-001"}))).contains("PI-FX-001"));
    assert!(t(mcp.prompts_get_with("review-change-set", json!({"since": "v1.2"}))).contains("from=v1.2"));
    let withparent = t(mcp.prompts_get_with("add-requirement-with-test", json!({"parent": "REQ-FX-001"})));
    assert!(withparent.contains("derivedFrom: [REQ-FX-001]"), "{withparent}");
}

#[test]
#[should_panic(expected = "requires the argument")]
fn a_required_prompt_argument_is_enforced() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    mcp.prompts_get_with("close-planning-item", json!({}));
}

/// Run the add-requirement-with-test sequence for real, tool by tool.
#[test]
fn the_add_requirement_with_test_sequence_runs_end_to_end() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let next = mcp.call_tool("next_id", json!({"prefix": "REQ-FX"}));
    let req_id = next["id"].as_str().or(next["nextId"].as_str()).unwrap_or_else(|| panic!("no id in {next}")).to_string();
    let tc = mcp.call_tool("next_id", json!({"prefix": "TC-FX"}));
    let tc_id = tc["id"].as_str().or(tc["nextId"].as_str()).unwrap_or_else(|| panic!("no id in {tc}")).to_string();
    assert!(mcp.call_tool("template", json!({"type": "Requirement"}))["content"].is_string());

    let fields = json!({"id": req_id, "name": "Fault logging", "status": "draft", "reqDomain": "software", "reqClass": "system"});
    let dry = mcp.call_tool("create_element", json!({"type": "Requirement", "parent": "", "fields": fields, "doc": "The system shall log faults.\n", "dry_run": true}));
    assert_eq!(dry["written"], false, "a dry run writes nothing: {dry}");
    let done = mcp.call_tool("create_element", json!({"type": "Requirement", "parent": "", "fields": fields, "doc": "The system shall log faults.\n", "dry_run": false}));
    assert_eq!(done["written"], true, "{done}");

    let tfields = json!({"id": tc_id, "name": "Fault logging test", "status": "draft", "testLevel": "L2", "verifies": [req_id]});
    let gherkin = "```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n";
    let done = mcp.call_tool("create_element", json!({"type": "TestCase", "parent": "", "fields": tfields, "doc": gherkin, "dry_run": false}));
    assert_eq!(done["written"], true, "{done}");

    let trace = mcp.call_tool("trace", json!({"ref": req_id}));
    assert!(serde_json::to_string(&trace).unwrap().contains(&tc_id), "the test appears under the requirement: {trace}");
}

// ── the tools the review prompt relies on (git-backed) ───────────────────────

fn git(root: &Path, args: &[&str]) {
    let o = Command::new("git").arg("-C").arg(root).args(args).output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

fn req(id: &str, status: &str) -> String {
    format!("---\nid: {id}\ntype: Requirement\nname: {id} name\nstatus: {status}\nreqDomain: software\nreqClass: system\n---\n\nThe system shall do {id}.\n")
}

#[test]
fn changes_tool_and_validate_new_since_work_against_git() {
    let model = fixture_copy();
    let root = model.parent().unwrap().to_path_buf();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.email", "t@example.com"]);
    git(&root, &["config", "user.name", "T"]);
    git(&root, &["config", "commit.gpgsign", "false"]);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "base"]);

    std::fs::write(model.join("Requirements/REQ-NEWCHG-001.md"), req("REQ-NEWCHG-001", "approved")).unwrap();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let ch = mcp.call_tool("changes", json!({"from": "HEAD"}));
    let list = ch["changes"].as_array().unwrap_or_else(|| panic!("{ch}"));
    assert!(list.iter().any(|c| c["id"] == "REQ-NEWCHG-001" && c["kind"] == "added"), "{ch}");
    assert_eq!(ch["to"], "working tree");
    let only = mcp.call_tool("changes", json!({"from": "HEAD", "types": ["TestCase"]}));
    assert!(only["changes"].as_array().unwrap().is_empty(), "{only}");
    let bad = mcp.call_tool_raw("changes", json!({"from": "no-such-rev"}));
    assert_eq!(bad["isError"], true, "{bad}");

    let all = mcp.call_tool("validate", json!({"severity": "info"}));
    let news = mcp.call_tool("validate", json!({"severity": "info", "new_since": "HEAD"}));
    let total = all["findings"].as_array().unwrap().len();
    let new = news["findings"].as_array().unwrap().len();
    let pre = news["preexisting"].as_u64().expect("preexisting reported");
    assert!(new >= 1 && new < total, "only the change's findings: {new} of {total}");
    assert_eq!(new as u64 + pre, total as u64);
    assert!(news["findings"].as_array().unwrap().iter().all(|f| f["file"].as_str().unwrap_or("").contains("REQ-NEWCHG-001")), "{news}");
    assert!(all.get("preexisting").is_none(), "absent without new_since");
}

#[test]
fn a_read_only_server_still_offers_the_review_tools_but_no_writes() {
    let model = fixture_copy();
    let mut mcp = Mcp::start_with_args(&model, &["--read-only"]);
    mcp.initialize();
    let tools: BTreeSet<String> = mcp.tools_list()["tools"].as_array().unwrap().iter().filter_map(|t| t["name"].as_str().map(String::from)).collect();
    assert!(tools.contains("changes") && tools.contains("validate"));
    assert!(!tools.contains("create_element") && !tools.contains("update_element"));
}

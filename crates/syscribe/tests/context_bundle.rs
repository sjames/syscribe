//! TC-TRS-CONTEXT-001 (GH #435): `context`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

mod common;

struct Model {
    root: PathBuf,
}

impl Model {
    fn new() -> Model {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-ctx-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&root).unwrap();
        let m = Model { root };
        m.w("_index.md", "---\ntype: Package\nname: Root\n---\n");
        m.w("TE-RIG-001.md", "---\nid: TE-RIG-001\ntype: TestEnvironment\nname: Rig\nstatus: available\nenvironmentKind: bench\n---\n\nrig\n");
        m.w("REQ-CX-001.md", "---\nid: REQ-CX-001\ntype: Requirement\nname: Parent need\nstatus: approved\nreqDomain: system\n---\n\nThe system shall be briefed.\n");
        let long = "Detail line that makes the body large. ".repeat(60);
        m.w("REQ-CX-002.md", &format!("---\nid: REQ-CX-002\ntype: Requirement\nname: Child need\nstatus: approved\nreqDomain: system\nderivedFrom: [REQ-CX-001]\n---\n\n{long}\n"));
        for i in 1..=12 {
            m.w(&format!("REQ-CX-1{i:02}.md"), &format!("---\nid: REQ-CX-1{i:02}\ntype: Requirement\nname: Grandchild {i}\nstatus: draft\nreqDomain: system\nderivedFrom: [REQ-CX-002]\n---\n\nG{i}.\n"));
        }
        m.w("TC-CX-001.md", "---\nid: TC-CX-001\ntype: TestCase\nname: Briefing test\nstatus: active\ntestLevel: L4\nverifies: [REQ-CX-002]\nrunsOn: [TE-RIG-001]\n---\n\nbody\n");
        m.w("Planning/PI-CX-001.md", "---\nid: PI-CX-001\ntype: PlanningItem\nname: Brief everyone\nstatus: in_progress\nitemType: epic\nachieves: [REQ-CX-002]\nclaimedBy: agent-1\nclaimedAt: \"2026-10-01T08:00:00Z\"\nquestions:\n  - id: Q1\n    text: Which format?\n    status: open\n---\n\nEpic.\n");
        m.w("Planning/PI-CX-002.md", "---\nid: PI-CX-002\ntype: PlanningItem\nname: Step one\nstatus: todo\nitemType: task\nparent: PI-CX-001\n---\n\nStep.\n");
        m
    }
    fn w(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&self.root).args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
    }
    fn ingest(&self) {
        std::fs::write(
            self.root.join("r.json"),
            r#"[{"testCase":"TC-CX-001","verdict":"pass","provenance":{"environment":"TE-RIG-001","build":{"commit":"abc1234"},"finishedAt":"2026-10-01T09:00:00Z"}}]"#,
        )
        .unwrap();
        let f = self.root.join("r.json");
        let (o, e, c) = self.run(&["ingest-results", f.to_str().unwrap(), "--format", "records"]);
        assert_eq!(c, 0, "{o}{e}");
    }
}
impl Drop for Model {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn a_requirement_shows_neighbours_verification_and_findings() {
    let m = Model::new();
    m.ingest();
    let (o, e, c) = m.run(&["context", "REQ-CX-002", "--format", "json"]);
    assert_eq!(c, 0, "{e}");
    let j: serde_json::Value = serde_json::from_str(&o).unwrap();
    assert_eq!(j["element"]["id"], "REQ-CX-002");
    assert!(j["body"].as_str().unwrap().contains("Detail line"));
    let ids = |k: &str| j[k].as_array().unwrap().iter().map(|x| x["id"].as_str().unwrap().to_string()).collect::<Vec<_>>();
    assert_eq!(ids("upstream"), vec!["REQ-CX-001"]);
    assert!(ids("downstream").contains(&"REQ-CX-101".to_string()) && ids("downstream").contains(&"TC-CX-001".to_string()));
    let v = &j["verification"][0];
    assert_eq!((v["id"].as_str(), v["verdict"].as_str(), v["environment"].as_str(), v["build"].as_str()), (Some("TC-CX-001"), Some("pass"), Some("TE-RIG-001"), Some("abc1234")));
    assert!(j.get("truncated").is_none());
    // The text form carries the same headline facts.
    let (t, _e, _c) = m.run(&["context", "REQ-CX-002"]);
    assert!(t.contains("# REQ-CX-002") && t.contains("## Verification") && t.contains("TC-CX-001") && t.contains("pass"), "{t}");
    // Depth two reaches the grandparent-free chain downstream of the parent.
    let (o, _e, _c) = m.run(&["context", "REQ-CX-001", "--depth", "2", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&o).unwrap();
    assert!(j["downstream"].as_array().unwrap().iter().any(|x| x["id"] == "REQ-CX-101" && x["depth"] == 2 && x["via"] == "REQ-CX-002"), "{j}");
    let (o, _e, _c) = m.run(&["context", "REQ-CX-001", "--depth", "1", "--json"]);
    assert!(!o.contains("REQ-CX-101"));
}

#[test]
fn findings_on_the_element_and_its_neighbours_are_listed() {
    let m = Model::new();
    // A draft-status leaf deriving from a parent without an ADR raises findings on the grandchildren.
    let (o, _e, _c) = m.run(&["context", "REQ-CX-002", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&o).unwrap();
    let f = j["findings"].as_array().unwrap();
    assert!(!f.is_empty(), "{j}");
    assert!(f.iter().any(|x| x["element"] == "REQ-CX-002"), "{f:?}");
    // Errors come before warnings.
    let sev: Vec<_> = f.iter().map(|x| x["severity"].as_str().unwrap()).collect();
    let first_warn = sev.iter().position(|s| *s == "warning").unwrap_or(sev.len());
    assert!(sev[first_warn..].iter().all(|s| *s == "warning"), "{sev:?}");
}

#[test]
fn a_planning_item_shows_its_work_state() {
    let m = Model::new();
    let (o, e, c) = m.run(&["context", "PI-CX-001", "--json"]);
    assert_eq!(c, 0, "{e}");
    let j: serde_json::Value = serde_json::from_str(&o).unwrap();
    let roles: Vec<String> = j["planning"].as_array().unwrap().iter().map(|x| format!("{}:{}", x["role"].as_str().unwrap(), x["id"].as_str().or(x["by"].as_str()).unwrap_or(""))).collect();
    for want in ["child:PI-CX-002", "achieves:REQ-CX-002", "claim:agent-1", "openQuestion:PI-CX-001#Q1"] {
        assert!(roles.iter().any(|r| r == want), "{want} not in {roles:?}");
    }
}

#[test]
fn output_never_exceeds_the_bound_and_json_stays_valid() {
    let m = Model::new();
    m.ingest();
    let full = m.run(&["context", "REQ-CX-002", "--depth", "3", "--max-bytes", "1000000"]).0;
    assert!(full.len() > 2500, "fixture should be large enough to need truncating: {}", full.len());
    assert!(!full.contains("Truncated"));
    for max in [1024usize, 1300, 1700, 2000, 2500] {
        for fmt in ["json", "markdown"] {
            let (o, e, c) = m.run(&["context", "REQ-CX-002", "--depth", "3", "--max-bytes", &max.to_string(), "--format", fmt]);
            assert_eq!(c, 0, "{e}");
            let body = o.strip_suffix('\n').unwrap_or(&o);
            assert!(body.len() <= max, "{fmt} at {max}: {} bytes", body.len());
            if fmt == "json" {
                let j: serde_json::Value = serde_json::from_str(body).unwrap_or_else(|x| panic!("invalid JSON at {max}: {x}\n{body}"));
                assert!(j["truncated"]["message"].as_str().unwrap().contains("Truncated"), "{j}");
                assert_eq!(j["element"]["id"], "REQ-CX-002");
            } else {
                assert!(body.contains("Truncated"), "{body}");
            }
        }
    }
    // What goes first: downstream before verification, and the omissions are counted.
    let (o, _e, _c) = m.run(&["context", "REQ-CX-002", "--depth", "3", "--max-bytes", "2500", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&o).unwrap();
    assert!(j["truncated"]["omitted"]["downstream"].as_u64().unwrap() > 0, "{j}");
    assert!(!j["verification"].as_array().unwrap().is_empty(), "verification outlasts downstream: {j}");
}

#[test]
fn errors_and_usage() {
    let m = Model::new();
    let (_o, e, c) = m.run(&["context", "REQ-CX-00"]);
    assert_eq!(c, 1);
    assert!(e.contains("not found") && e.contains("REQ-CX-00"), "{e}");
    for bad in [&["context"][..], &["context", "REQ-CX-001", "--max-bytes", "10"], &["context", "REQ-CX-001", "--format", "xml"], &["context", "REQ-CX-001", "--depth", "x"]] {
        let (_o, e, c) = m.run(bad);
        assert_eq!(c, 2, "{bad:?}: {e}");
    }
    // An unknown option is caught by the shared argument check, which exits 1 for every command.
    let (_o, e, c) = m.run(&["context", "REQ-CX-001", "--nope"]);
    assert_eq!(c, 1, "{e}");
    assert!(e.contains("--max-bytes"), "{e}");
}

#[test]
fn the_mcp_tool_matches_the_cli_and_honours_the_bound() {
    let m = Model::new();
    m.ingest();
    let cli: serde_json::Value = serde_json::from_str(&m.run(&["context", "REQ-CX-002", "--json"]).0).unwrap();
    let mut mcp = common::Mcp::start(&m.root);
    mcp.initialize();
    let v = mcp.call_tool("context", serde_json::json!({"element": "REQ-CX-002"}));
    assert_eq!(v, cli);
    let small = mcp.call_tool("context", serde_json::json!({"element": "REQ-CX-002", "depth": 3, "max_bytes": 1500}));
    assert!(serde_json::to_string(&small).unwrap().len() <= 1600 && small["truncated"].is_object(), "{small}");
    let bad = mcp.call_tool_raw("context", serde_json::json!({"element": "REQ-CX-002", "max_bytes": 5}));
    assert_eq!(bad["isError"], true);
}

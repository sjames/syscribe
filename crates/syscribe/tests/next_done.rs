//! GH #485 (`REQ-TRS-NEXT-001`, `REQ-TRS-DONE-001`): `syscribe next` and `syscribe done <PI>`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct M {
    root: PathBuf,
}

impl M {
    fn new(toml: &str) -> M {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-nd-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
        std::fs::create_dir_all(&root).unwrap();
        let m = M { root };
        m.w("_index.md", "---\ntype: Package\nname: Root\n---\n");
        m.w("Req/REQ-TST-001.md", "---\ntype: Requirement\nid: REQ-TST-001\nname: R\nstatus: draft\nreqDomain: software\nreqClass: system\n---\nThe tool shall do it.\n");
        m.w("Test/TC-TST-001.md", "---\nid: TC-TST-001\ntype: TestCase\ntestLevel: L3\nstatus: active\nname: T\nverifies:\n  - REQ-TST-001\n---\n\n```gherkin\nFeature: F\n  Scenario: S\n    Given a\n    When b\n    Then c\n```\n");
        if !toml.is_empty() {
            m.w(".syscribe.toml", toml);
        }
        m
    }
    fn pi(&self, id: &str, extra: &str) {
        self.w(&format!("Plan/{id}.md"), &format!("---\ntype: PlanningItem\nid: {id}\nname: {id}\nitemType: task\n{extra}---\n"));
    }
    fn w(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn run(&self, args: &[&str]) -> (String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&self.root).args(args).output().unwrap();
        (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
    }
    fn json(&self, args: &[&str]) -> (serde_json::Value, i32) {
        let (out, code) = self.run(args);
        (serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}")), code)
    }
}

fn board() -> M {
    let m = M::new("");
    m.pi("PI-TST-001", "status: todo\nachieves: [REQ-TST-001]\n");
    m.pi("PI-TST-002", "status: in_progress\nachieves: [REQ-TST-001]\n");
    m.pi("PI-TST-003", "status: todo\nachieves: [REQ-TST-001]\nclaimedBy: agent-x\nclaimedAt: 2026-11-17T00:00:00Z\n");
    m.pi("PI-TST-004", "status: todo\nachieves: [REQ-TST-001]\nblockedBy: [PI-TST-005]\n");
    m.pi("PI-TST-005", "status: todo\nachieves: [REQ-TST-001]\n");
    m.pi("PI-TST-006", "status: todo\nachieves: [REQ-TST-001]\n");
    m.pi("PI-TST-007", "status: todo\nparent: PI-TST-006\n");
    m.pi("PI-TST-008", "status: todo\nachieves: [REQ-TST-001]\nquestions:\n  - { id: Q1, text: \"Which?\", status: open, answerBy: human }\n");
    m.pi("PI-TST-009", "status: todo\nachieves: [REQ-TST-001]\nquestions:\n  - { id: Q1, text: \"Which?\", status: open, answerBy: any }\n");
    m.pi("PI-TST-010", "status: done\nachieves: [REQ-TST-001]\nevidence:\n  - { ref: TC-TST-001 }\n");
    m
}

#[test]
fn next_offers_unblocked_unclaimed_leaves_resumable_first() {
    let (v, code) = board().json(&["next", "--json", "--limit", "20"]);
    assert_eq!(code, 0);
    let ids: Vec<&str> = v["candidates"].as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap()).collect();
    // The unclaimed in_progress item first; then by id. PI-TST-004 waits on PI-TST-005.
    assert_eq!(ids, ["PI-TST-002", "PI-TST-001", "PI-TST-005", "PI-TST-007", "PI-TST-009"], "{v}");
    assert_eq!(v["next"]["id"], "PI-TST-002");
    let skipped = |id: &str| v["skipped"].as_array().unwrap().iter().find(|s| s["id"] == id).map(|s| s["reason"].as_str().unwrap().to_string()).unwrap_or_default();
    assert!(skipped("PI-TST-003").contains("claimed by agent-x"));
    assert!(skipped("PI-TST-004").contains("blocked by PI-TST-005 (todo)"));
    assert!(skipped("PI-TST-006").contains("composite"));
    assert!(skipped("PI-TST-008").contains("waiting on a person") && skipped("PI-TST-008").contains("PI-TST-008#Q1"));
    assert!(v["skipped"].as_array().unwrap().iter().all(|s| s["id"] != "PI-TST-010"), "a done item is not listed");
}

#[test]
fn next_reports_hazards_and_exits_2_when_nothing_is_offered() {
    let m = board();
    let (v, _) = m.json(&["next", "--json", "--limit", "20"]);
    let hz = |id: &str| v["candidates"].as_array().unwrap().iter().find(|c| c["id"] == id).unwrap()["hazards"].to_string();
    assert!(hz("PI-TST-001").contains("overlaps active PI-TST-002") && hz("PI-TST-001").contains("W311"), "{}", hz("PI-TST-001"));
    assert!(hz("PI-TST-009").contains("PI-TST-009#Q1"), "{}", hz("PI-TST-009"));
    let empty = M::new("");
    empty.pi("PI-EMP-001", "status: done\nachieves: [REQ-TST-001]\nevidence:\n  - { ref: TC-TST-001 }\n");
    let (out, code) = empty.run(&["next"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("Nothing to take"), "{out}");
    let (_, code) = empty.run(&["next", "--bogus"]);
    assert_eq!(code, 1);
}

#[test]
fn done_names_exactly_what_is_missing() {
    let m = M::new("");
    m.pi("PI-DNE-001", "status: in_progress\nachieves: [REQ-TST-001]\nquestions:\n  - { id: Q1, text: \"Which?\", status: open }\nblockedBy: [PI-DNE-002]\n");
    m.pi("PI-DNE-002", "status: todo\nachieves: [REQ-TST-001]\n");
    let (v, code) = m.json(&["done", "PI-DNE-001", "--json"]);
    assert_eq!(code, 2);
    assert_eq!(v["ready"], false);
    let result = |c: &str| v["checks"].as_array().unwrap().iter().find(|x| x["check"] == c).unwrap()["result"].as_str().unwrap().to_string();
    assert_eq!(result("questions"), "fail");
    assert_eq!(result("blockers"), "fail");
    assert_eq!(result("evidence"), "fail");
    let missing = v["missing"].to_string();
    assert!(missing.contains("PI-DNE-001#Q1") && missing.contains("PI-DNE-002") && missing.contains("E719"), "{missing}");
    // The evidence failure is not counted a second time under validate.
    assert_eq!(result("validate"), "pass");
}

#[test]
fn done_passes_when_everything_holds_and_never_writes() {
    let m = M::new("");
    m.pi("PI-DNE-001", "status: in_progress\nachieves: [REQ-TST-001]\nevidence:\n  - { ref: TC-TST-001 }\n");
    let before = std::fs::read_to_string(m.root.join("Plan/PI-DNE-001.md")).unwrap();
    let (out, code) = m.run(&["done", "PI-DNE-001"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("ready to be done") && out.contains("status=done"), "{out}");
    assert_eq!(before, std::fs::read_to_string(m.root.join("Plan/PI-DNE-001.md")).unwrap());
    let (v, code) = m.json(&["done", "PI-DNE-001", "--json"]);
    assert_eq!((v["ready"].clone(), v["needsPerson"].clone(), code), (serde_json::json!(true), serde_json::json!(false), 0), "{v}");
}

#[test]
fn done_judges_a_composite_by_its_steps() {
    let m = M::new("");
    m.pi("PI-DNE-001", "status: in_progress\nachieves: [REQ-TST-001]\nevidence:\n  - { ref: TC-TST-001 }\n");
    m.pi("PI-DNE-002", "status: todo\nparent: PI-DNE-001\n");
    let (v, code) = m.json(&["done", "PI-DNE-001", "--json"]);
    assert_eq!(code, 2);
    assert!(v["missing"].to_string().contains("steps: 1 step(s) not done: PI-DNE-002"), "{v}");
}

#[test]
fn done_reports_a_handoff_when_the_policy_reserves_done_for_a_person() {
    let m = M::new("[agents]\nhuman_only_status = { PlanningItem = [\"done\"] }\n");
    m.pi("PI-DNE-001", "status: in_progress\nachieves: [REQ-TST-001]\nevidence:\n  - { ref: TC-TST-001 }\n");
    let (v, code) = m.json(&["--by", "agent-7", "done", "PI-DNE-001", "--json"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!((v["ready"].clone(), v["needsPerson"].clone()), (serde_json::json!(true), serde_json::json!(true)), "{v}");
    let (out, _) = m.run(&["--by", "agent-7", "done", "PI-DNE-001"]);
    assert!(out.contains("a person sets `status: done`"), "{out}");
    // A person is not handed off.
    let (v, _) = m.json(&["--as-human", "done", "PI-DNE-001", "--json"]);
    assert_eq!(v["needsPerson"], false, "{v}");
}

#[test]
fn done_rejects_an_unknown_item_and_a_non_planning_item() {
    let m = M::new("");
    for r in ["NOPE", "REQ-TST-001"] {
        let (out, code) = m.run(&["done", r]);
        assert_eq!(code, 1, "{r}: {out}");
        assert!(out.contains("is not a PlanningItem"), "{r}: {out}");
    }
    let (_, code) = m.run(&["done"]);
    assert_eq!(code, 1);
}

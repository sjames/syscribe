//! TC-TRS-PIQ-001..003 and TC-TRS-PISTEP-001 (GH #436): inline PlanningItem questions and the
//! steps view.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn run(model: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

/// A model with a requirement and a small PlanningItem tree. `extra` is appended to PI-QQQ-001's frontmatter.
fn model(extra: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-piq-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("REQ-QQQ-001.md", "---\nid: REQ-QQQ-001\ntype: Requirement\nname: R\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nShall.\n");
    w("PI-QQQ-001.md", &format!("---\nid: PI-QQQ-001\ntype: PlanningItem\nname: Top item\nstatus: in_progress\nachieves: [REQ-QQQ-001]\n{extra}---\n\nBody.\n"));
    w("PI-QQQ-002.md", "---\nid: PI-QQQ-002\ntype: PlanningItem\nname: First step\nstatus: done\nparent: PI-QQQ-001\nevidence:\n  - ref: REQ-QQQ-001\n---\n\nx\n");
    w("PI-QQQ-003.md", "---\nid: PI-QQQ-003\ntype: PlanningItem\nname: Second step\nstatus: in_progress\nparent: PI-QQQ-001\nassignedTo: alice\nclaimedBy: agent-1\nclaimedAt: 2026-11-03T10:00:00Z\n---\n\nx\n");
    w("PI-QQQ-004.md", "---\nid: PI-QQQ-004\ntype: PlanningItem\nname: Third step\nstatus: blocked\nparent: PI-QQQ-001\nblockedBy: [PI-QQQ-003]\n---\n\nx\n");
    d
}

const TWO: &str = "questions:\n  # decisions for hw\n  - id: Q1\n    text: Which UART?   # ask hw\n    recommended: UART2\n    status: open\n  - id: Q2\n    text: Baud rate\n    status: answered\n    answer: \"115200\"\n    answeredBy: alice\n";

fn codes(out: &str) -> Vec<String> {
    out.lines().filter_map(|l| l.split('|').nth(1)).map(|c| c.trim().to_string()).filter(|c| c.len() >= 4 && c.chars().skip(1).all(|x| x.is_ascii_digit())).collect()
}

// ── validation (TC-TRS-PIQ-001) ──────────────────────────────────────────────

#[test]
fn well_formed_questions_raise_nothing_and_items_without_them_are_unaffected() {
    let (out, _e, _c) = run(&model(TWO), &["validate"]);
    let cs = codes(&out);
    assert!(!cs.iter().any(|c| matches!(c.as_str(), "E724" | "E725" | "E726" | "E727" | "W317")), "{out}");
    let (out, _e, _c) = run(&model(""), &["validate"]);
    assert!(!out.contains("E724") && !out.contains("W317"), "{out}");
}

#[test]
fn malformed_entries_are_each_reported() {
    let bad = "questions:\n  - id: Q1\n    text: ok\n    status: open\n  - id: Q1\n    text: duplicate\n    status: open\n  - id: Q2\n    text: bad status\n    status: pending\n  - id: Q3\n    text: no answer\n    status: answered\n  - id: Q4\n    status: open\n  - just a string\n  - id: question-five\n    text: bad id\n    status: open\n";
    let (out, _e, code) = run(&model(bad), &["validate"]);
    let cs = codes(&out);
    let n = |c: &str| cs.iter().filter(|x| *x == c).count();
    assert_eq!(n("E725"), 1, "{out}");
    assert_eq!(n("E726"), 1, "{out}");
    assert_eq!(n("E727"), 1, "{out}");
    assert_eq!(n("E724"), 3, "no text, not a mapping, bad id:\n{out}");
    assert_eq!(code, 1);
    let (out, _e, _c) = run(&model("questions: just text\n"), &["validate"]);
    assert!(out.contains("E724") && out.contains("must be a list"), "{out}");
}

#[test]
fn done_with_an_open_question_warns_but_deferred_and_answered_do_not() {
    let dir = model("");
    let write = |status: &str, qstatus: &str| {
        std::fs::write(
            dir.join("PI-QQQ-001.md"),
            format!("---\nid: PI-QQQ-001\ntype: PlanningItem\nname: Top item\nstatus: {status}\nachieves: [REQ-QQQ-001]\nquestions:\n  - id: Q1\n    text: q\n    status: {qstatus}\n    answer: a\n---\n\nBody.\n"),
        )
        .unwrap();
        run(&dir, &["validate"]).0
    };
    let w317 = |out: &str| out.lines().find(|l| l.contains("| W317 |")).map(String::from);
    let line = w317(&write("done", "open")).expect("W317 for done + open");
    assert!(line.contains("PI-QQQ-001#Q1") && line.contains("question.defer"), "{line}");
    assert!(w317(&write("done", "deferred")).is_none());
    assert!(w317(&write("done", "answered")).is_none());
    assert!(w317(&write("in_progress", "open")).is_none(), "only a done item is stale");
    // An open question does not make an item `blocked`, and a blocked item is not flagged for lacking one.
    let out = write("in_progress", "open");
    assert!(!out.contains("W308"), "{out}");
}

#[test]
fn questions_on_another_type_warn_and_are_not_counted() {
    let dir = model(TWO);
    // A Requirement and a TestCase that (wrongly) carry the field.
    std::fs::write(
        dir.join("REQ-QQQ-001.md"),
        "---\nid: REQ-QQQ-001\ntype: Requirement\nname: R\nstatus: draft\nreqDomain: software\nreqClass: system\nquestions:\n  - id: Q1\n    text: misplaced\n    status: open\n---\n\nShall.\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("TC-QQQ-001.md"),
        "---\nid: TC-QQQ-001\ntype: TestCase\nname: T\nstatus: draft\ntestLevel: L2\nverifies: [REQ-QQQ-001]\nquestions: not even a list\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n",
    )
    .unwrap();
    let (out, _e, _c) = run(&dir, &["validate"]);
    let w318: Vec<&str> = out.lines().filter(|l| l.contains("| W318 |")).collect();
    assert_eq!(w318.len(), 2, "one per misplaced field, whatever its shape:\n{out}");
    assert!(w318.iter().any(|l| l.contains("REQ-QQQ-001.md") && l.contains("REQ-QQQ-001") && l.contains("only a PlanningItem")), "{w318:?}");
    assert!(w318.iter().any(|l| l.contains("TC-QQQ-001.md")), "{w318:?}");
    assert!(!out.contains("E724"), "a misplaced field is not parsed, so it is not malformed: {out}");
    // The PlanningItem that really has questions is unaffected, and the misplaced ones are not listed.
    assert!(!w318.iter().any(|l| l.contains("PI-QQQ-001")));
    let (q, _e, _c) = run(&dir, &["questions", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&q).unwrap();
    assert_eq!(v["questions"].as_array().unwrap().len(), 2, "only the PlanningItem's: {q}");
    // And the warning is non-fatal: it does not change the exit status by itself.
    let clean = model(TWO);
    assert_eq!(run(&clean, &["validate"]).2, 0);
}

// ── surfacing (TC-TRS-PIQ-002) ───────────────────────────────────────────────

#[test]
fn questions_command_lists_filters_and_reports_json() {
    let m = model(TWO);
    let (out, _e, code) = run(&m, &["questions"]);
    assert_eq!(code, 0);
    assert!(out.contains("| PI-QQQ-001#Q1 | open | Which UART? | UART2 |") && out.contains("| PI-QQQ-001#Q2 | answered | Baud rate |"), "{out}");
    assert!(out.contains("115200 — alice"), "{out}");

    let (open, _e, _c) = run(&m, &["questions", "--open"]);
    assert!(open.contains("#Q1") && !open.contains("#Q2"), "{open}");
    assert!(run(&m, &["questions", "--status", "answered"]).0.contains("#Q2"));
    assert!(run(&m, &["questions", "--item", "PI-QQQ-001", "--open"]).0.contains("#Q1"));
    assert_eq!(run(&m, &["questions", "--item", "PI-QQQ-003"]).0.trim(), "No questions match.");

    let (j, _e, code) = run(&m, &["questions", "--json"]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["questions"].as_array().unwrap().len(), 2);
    assert_eq!(v["questions"][0]["item"], "PI-QQQ-001");
    assert_eq!(v["questions"][0]["recommended"], "UART2");

    let (_o, err, code) = run(&m, &["questions", "--status", "opne"]);
    assert_eq!(code, 1);
    assert!(err.contains("Did you mean 'open'?"), "{err}");
    assert_eq!(run(&m, &["questions", "--item", "nope"]).2, 1);
    assert_eq!(run(&m, &["questions", "--item", "REQ-QQQ-001"]).2, 1, "not a PlanningItem");
    assert_eq!(run(&m, &["questions", "--open", "--status", "answered"]).2, 1);
    assert!(run(&model(""), &["questions"]).0.starts_with("No questions."));
}

#[test]
fn list_and_show_surface_questions() {
    let m = model(TWO);
    let (out, _e, _c) = run(&m, &["list", "PlanningItem", "--open-questions"]);
    assert!(out.contains("PI-QQQ-001") && !out.contains("PI-QQQ-002"), "{out}");
    let (j, _e, _c) = run(&m, &["list", "PlanningItem", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    let count = |id: &str| v.as_array().unwrap().iter().find(|e| e["id"] == id).unwrap()["openQuestions"].as_u64().unwrap();
    assert_eq!((count("PI-QQQ-001"), count("PI-QQQ-002")), (1, 0));
    let (show, _e, _c) = run(&m, &["show", "PI-QQQ-001"]);
    assert!(show.contains("| **Q1 (open)** | Which UART? — recommended: UART2 |"), "{show}");
    assert!(show.contains("| **Q2 (answered)** | Baud rate — answer: 115200 |"), "{show}");
    assert!(!run(&m, &["show", "PI-QQQ-002"]).0.contains("(open)"));
}

#[test]
fn hint_for_a_prose_questions_section() {
    let dir = model("");
    std::fs::write(
        dir.join("PI-QQQ-001.md"),
        "---\nid: PI-QQQ-001\ntype: PlanningItem\nname: Top item\nstatus: todo\nachieves: [REQ-QQQ-001]\n---\n\n## Open questions\n\n| # | Question |\n|---|---|\n| 1 | Which UART? |\n",
    )
    .unwrap();
    assert!(!run(&dir, &["validate"]).0.contains("I913"), "off by default");
    let (out, _e, _c) = run(&dir, &["validate", "--hints"]);
    let row = out.lines().find(|l| l.contains("| I913 |")).unwrap_or_else(|| panic!("{out}"));
    assert!(row.contains("question.add"), "{row}");
    // Structured questions silence it.
    let (out, _e, _c) = run(&model(TWO), &["validate", "--hints"]);
    assert!(!out.contains("I913"));
}

// ── writing (TC-TRS-PIQ-003) ─────────────────────────────────────────────────

fn read(m: &Path, f: &str) -> String {
    std::fs::read_to_string(m.join(f)).unwrap()
}

#[test]
fn add_answer_and_defer_edit_only_their_own_entry() {
    let m = model(TWO);
    let before = read(&m, "PI-QQQ-001.md");

    let (out, err, code) = run(&m, &["set", "PI-QQQ-001", "question.add", "text=Which pin?", "recommended=PA9"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("Added question PI-QQQ-001#Q3"), "{out}");
    let after = read(&m, "PI-QQQ-001.md");
    assert!(after.contains("  - id: Q3\n    text: Which pin?\n    recommended: PA9\n    status: open\n"), "{after}");
    assert!(after.contains("# decisions for hw") && after.contains("text: Which UART?   # ask hw"), "comments kept:\n{after}");
    // Nothing but the new entry was added: remove it and the file is byte-identical to before.
    let added = "  - id: Q3\n    text: Which pin?\n    recommended: PA9\n    status: open\n";
    assert_eq!(after.replacen(added, "", 1), before, "only the new entry was added");

    let (out, err, code) = run(&m, &["set", "PI-QQQ-001", "question.answer", "Q1", "text=UART2", "by=alice"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("Answered question PI-QQQ-001#Q1"));
    let after = read(&m, "PI-QQQ-001.md");
    let q1 = &after[after.find("- id: Q1").unwrap()..after.find("- id: Q2").unwrap()];
    assert!(q1.contains("status: answered") && q1.contains("answer: UART2") && q1.contains("answeredBy: alice") && q1.contains("answeredAt: "), "{q1}");
    assert!(q1.contains("text: Which UART?   # ask hw"), "{q1}");
    let q2 = &after[after.find("- id: Q2").unwrap()..after.find("- id: Q3").unwrap()];
    assert!(q2.contains("answer: \"115200\""), "Q2 byte-identical:\n{q2}");

    assert_eq!(run(&m, &["set", "PI-QQQ-001", "question.defer", "Q3"]).2, 0);
    assert!(read(&m, "PI-QQQ-001.md").contains("text: Which pin?\n    recommended: PA9\n    status: deferred"));
    // Re-answering replaces the answer.
    assert_eq!(run(&m, &["set", "PI-QQQ-001", "question.answer", "Q1", "text=UART3"]).2, 0);
    assert!(read(&m, "PI-QQQ-001.md").contains("answer: UART3"));

    // The result validates clean and `questions` reads it back.
    let (v, _e, _c) = run(&m, &["validate"]);
    assert!(!v.contains("E724") && !v.contains("E725") && !v.contains("E726") && !v.contains("E727"), "{v}");
    assert!(!run(&m, &["questions", "--open"]).0.contains("#Q"), "none open now");
}

#[test]
fn add_creates_the_list_when_absent_and_dry_run_writes_nothing() {
    let m = model("");
    let before = read(&m, "PI-QQQ-001.md");
    let (out, _e, code) = run(&m, &["set", "PI-QQQ-001", "question.add", "text=First?", "--dry-run"]);
    assert_eq!(code, 0);
    assert!(out.contains("+questions:") && out.contains("+  - id: Q1") && out.contains("+    status: open"), "{out}");
    assert_eq!(read(&m, "PI-QQQ-001.md"), before, "dry run leaves the file byte-identical");
    assert_eq!(run(&m, &["set", "PI-QQQ-001", "question.add", "text=First?"]).2, 0);
    assert!(read(&m, "PI-QQQ-001.md").contains("questions:\n  - id: Q1\n    text: First?\n    status: open"));
}

#[test]
fn a_flow_style_list_falls_back_to_reserialising_and_still_round_trips() {
    let m = model("questions: [{id: Q1, text: flow, status: open}]\n");
    assert_eq!(run(&m, &["set", "PI-QQQ-001", "question.answer", "Q1", "text=done"]).2, 0);
    let (j, _e, _c) = run(&m, &["questions", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["questions"][0]["status"], "answered");
    assert_eq!(v["questions"][0]["answer"], "done");
}

#[test]
fn refusals_exit_one_and_write_nothing() {
    let m = model(TWO);
    let before = read(&m, "PI-QQQ-001.md");
    let cases: &[&[&str]] = &[
        &["set", "PI-QQQ-001", "question.answer", "Q9", "text=x"],
        &["set", "PI-QQQ-001", "question.answer", "Q1"],
        &["set", "PI-QQQ-001", "question.add", "text="],
        &["set", "PI-QQQ-001", "question.add"],
        &["set", "PI-QQQ-001", "question.defer"],
        &["set", "PI-QQQ-001", "question.bogus", "Q1"],
        &["set", "REQ-QQQ-001", "question.add", "text=x"],
    ];
    for c in cases {
        let (_o, err, code) = run(&m, c);
        assert_eq!(code, 1, "{c:?}: {err}");
        assert!(!err.is_empty(), "{c:?} explains itself");
    }
    assert_eq!(read(&m, "PI-QQQ-001.md"), before);
    let (_o, err, _c) = run(&m, &["set", "PI-QQQ-001", "question.answer", "Q9", "text=x"]);
    assert!(err.contains("has no question Q9") && err.contains("Q1, Q2"), "{err}");
}

// ── steps (TC-TRS-PISTEP-001) ────────────────────────────────────────────────

#[test]
fn steps_renders_children_with_status_owner_claim_blockers_and_a_rollup() {
    let m = model(TWO);
    let (out, err, code) = run(&m, &["steps", "PI-QQQ-001"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("1 of 3 leaves done · 1 in progress · 1 blocked"), "{out}");
    assert!(out.contains("├─ PI-QQQ-002  First step  [done]"), "{out}");
    assert!(out.contains("PI-QQQ-003  Second step  [in_progress]  @alice  claimed by agent-1"), "{out}");
    assert!(out.contains("└─ PI-QQQ-004  Third step  [blocked]  blocked by PI-QQQ-003"), "{out}");
    assert!(out.contains("?1"), "the root shows its open question: {out}");
}

#[test]
fn steps_open_depth_json_and_errors() {
    let m = model("");
    let (open, _e, _c) = run(&m, &["steps", "PI-QQQ-001", "--open"]);
    assert!(!open.contains("First step") && open.contains("Second step"), "{open}");
    assert!(open.contains("1 of 3 leaves done"), "the roll-up still counts what is hidden:\n{open}");
    let (d0, _e, _c) = run(&m, &["steps", "PI-QQQ-001", "--depth", "0"]);
    assert!(d0.contains("3 more below") && !d0.contains("Second step"), "{d0}");

    let (j, _e, code) = run(&m, &["steps", "PI-QQQ-001", "--json"]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["rollup"]["leaves"], 3);
    assert_eq!(v["rollup"]["done"], 1);
    assert_eq!(v["tree"]["children"].as_array().unwrap().len(), 3);
    assert_eq!(v["tree"]["children"][1]["assignedTo"], "alice");

    assert_eq!(run(&m, &["steps", "nope"]).2, 1);
    assert_eq!(run(&m, &["steps", "REQ-QQQ-001"]).2, 1);
    assert_eq!(run(&m, &["steps"]).2, 1);
    assert_eq!(run(&m, &["steps", "PI-QQQ-001", "--depth", "x"]).2, 1);
    let (leaf, _e, code) = run(&m, &["steps", "PI-QQQ-002"]);
    assert_eq!(code, 0);
    assert!(leaf.contains("No child items"), "{leaf}");
}

#[test]
fn steps_notes_disagreement_without_raising_a_finding() {
    let dir = model("");
    // All leaves done, the parent still in progress.
    for (f, id) in [("PI-QQQ-003.md", "PI-QQQ-003"), ("PI-QQQ-004.md", "PI-QQQ-004")] {
        std::fs::write(dir.join(f), format!("---\nid: {id}\ntype: PlanningItem\nname: step\nstatus: done\nparent: PI-QQQ-001\nevidence:\n  - ref: REQ-QQQ-001\n---\n\nx\n")).unwrap();
    }
    let (out, _e, _c) = run(&dir, &["steps", "PI-QQQ-001"]);
    assert!(out.contains("Notes (not findings)") && out.contains("all 3 leaves are done but the item is `in_progress`"), "{out}");
    let (v, _e, _c) = run(&dir, &["validate"]);
    assert!(!v.contains("all 3 leaves"), "validate stays silent: {v}");
}

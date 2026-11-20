//! GH #500 a) (`REQ-TRS-RESOUTLINE-001`): rows of a Scenario Outline recorded through `ingest-results --format
//! records` as `<title> [row N]` reach the TestCase's verdict, as they already do for a Cucumber report.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(gherkin: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-outline-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("REQ-HTR-001.md", "---\nid: REQ-HTR-001\ntype: Requirement\nname: \"Heater response\"\nstatus: approved\nreqDomain: system\nreqClass: system\n---\n\nThe heater shall reach the setpoint within the limit.\n");
    w("TE-HTR-001.md", "---\nid: TE-HTR-001\ntype: TestEnvironment\nname: Rig\nstatus: available\nenvironmentKind: bench\n---\n\nRig.\n");
    w("TC-HTR-001.md", &format!("---\nid: TC-HTR-001\ntype: TestCase\ntestLevel: L4\nstatus: active\nname: \"Heater response\"\nverifies: [REQ-HTR-001]\n---\n\n```gherkin\n{gherkin}```\n"));
    root
}

const OUTLINE: &str = "Feature: Heater\n\n  Scenario Outline: Response within limit\n    Given the ambient is <ambient>\n    When the heater is switched on\n    Then it reaches the setpoint within the limit\n\n    Examples:\n      | ambient  |\n      | -20 degC |\n      | 23 degC  |\n      | 70 degC  |\n";

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn rec(scenario: &str, verdict: &str) -> String {
    format!(r#"{{"testCase":"TC-HTR-001","scenario":"{scenario}","verdict":"{verdict}","rationale":{},"provenance":{{"environment":"TE-HTR-001"}}}}"#, if verdict == "waived" { "\"accepted deviation\"" } else { "null" })
}

fn row(n: usize, verdict: &str) -> String {
    rec(&format!("Response within limit [row {n}]"), verdict)
}

fn ingest(root: &Path, records: &[String]) {
    let file = root.join("r.json");
    std::fs::write(&file, format!("[{}]", records.join(","))).unwrap();
    let (out, code) = run(root, &["ingest-results", "--format", "records", file.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
}

/// The verdict `trace` shows for the TestCase.
fn verdict(root: &Path) -> String {
    let (out, _) = run(root, &["trace", "REQ-HTR-001"]);
    let line = out.lines().find(|l| l.contains("TC-HTR-001")).unwrap_or_else(|| panic!("no TestCase row in:\n{out}"));
    // With no results at all `trace` shows no verdict.
    let Some(open) = line.find('[') else { return "none".to_string() };
    line[open + 1..line[open..].find(']').map(|i| open + i).unwrap()].to_string()
}

#[test]
fn a_failing_row_fails_the_outline_whatever_else_was_recorded() {
    let m = model(OUTLINE);
    assert_eq!(verdict(&m), "none", "nothing recorded");
    ingest(&m, &[row(1, "fail"), row(2, "pass")]);
    assert_eq!(verdict(&m), "fail", "row 1 failed, row 2 passed, row 3 never ran");
}

#[test]
fn a_later_pass_at_one_condition_does_not_hide_a_fail_at_another() {
    let m = model(OUTLINE);
    ingest(&m, &[row(1, "fail"), row(2, "pass"), row(3, "pass")]);
    assert_eq!(verdict(&m), "fail");
    ingest(&m, &[row(2, "pass"), row(3, "pass")]);
    assert_eq!(verdict(&m), "fail", "re-running the other rows leaves row 1's failure standing");
    ingest(&m, &[row(1, "pass")]);
    assert_eq!(verdict(&m), "pass", "only a new result for row 1 itself replaces it");
}

#[test]
fn the_outline_passes_only_when_every_row_has_passed() {
    let m = model(OUTLINE);
    ingest(&m, &[row(1, "pass"), row(2, "pass")]);
    assert_eq!(verdict(&m), "unknown", "row 3 never ran: not a pass");
    ingest(&m, &[row(3, "pass")]);
    assert_eq!(verdict(&m), "pass");
}

#[test]
fn a_row_that_did_not_conclude_is_not_a_pass() {
    for v in ["not-run", "blocked", "inconclusive", "ignored", "flaky"] {
        let m = model(OUTLINE);
        ingest(&m, &[row(1, "pass"), row(2, "pass"), row(3, v)]);
        assert_eq!(verdict(&m), "unknown", "{v}");
    }
}

#[test]
fn a_waived_row_shows_as_a_waiver_never_as_a_pass() {
    let m = model(OUTLINE);
    ingest(&m, &[row(1, "pass"), row(2, "pass"), row(3, "waived")]);
    assert_eq!(verdict(&m), "waived");
    ingest(&m, &[row(1, "fail")]);
    assert_eq!(verdict(&m), "fail");
}

#[test]
fn the_outlines_own_record_still_speaks_when_no_row_was_recorded() {
    let m = model(OUTLINE);
    ingest(&m, &[rec("Response within limit", "pass")]);
    assert_eq!(verdict(&m), "pass", "a record of the outline title, as before");
    ingest(&m, &[rec("Response within limit", "fail")]);
    assert_eq!(verdict(&m), "fail");
}

#[test]
fn an_outline_record_of_fail_survives_passing_rows() {
    let m = model(OUTLINE);
    ingest(&m, &[rec("Response within limit", "fail"), row(1, "pass"), row(2, "pass"), row(3, "pass")]);
    assert_eq!(verdict(&m), "fail", "the explicit fail is not outvoted by the rows");
}

#[test]
fn a_row_the_examples_table_does_not_have_does_not_count_towards_the_outline() {
    let m = model(OUTLINE);
    // The table has three rows. Rows 1 and 2 pass; "row 7" is not a condition of this outline.
    ingest(&m, &[row(1, "pass"), row(2, "pass"), row(7, "pass")]);
    assert_eq!(verdict(&m), "unknown", "row 7 cannot stand in for row 3");
    ingest(&m, &[row(4, "fail")]);
    assert_eq!(verdict(&m), "unknown", "and a failing phantom row cannot fail the outline");
}

#[test]
fn rows_are_numbered_across_several_examples_tables() {
    let two = "Feature: Heater\n\n  Scenario Outline: Response within limit\n    Given the ambient is <ambient>\n    Then it reaches the setpoint\n\n    Examples: cold\n      | ambient |\n      | -20 degC |\n\n    Examples: hot\n      | ambient |\n      | 70 degC |\n      | 85 degC |\n";
    let m = model(two);
    ingest(&m, &[row(1, "pass"), row(2, "pass")]);
    assert_eq!(verdict(&m), "unknown", "three rows in all");
    ingest(&m, &[row(3, "pass")]);
    assert_eq!(verdict(&m), "pass");
}

#[test]
fn plain_scenarios_and_outlines_are_judged_together() {
    let both = format!("{OUTLINE}\n  Scenario: Cold start\n    Given the ambient is -20 degC\n    Then it starts\n");
    let m = model(&both);
    ingest(&m, &[row(1, "pass"), row(2, "pass"), row(3, "pass")]);
    assert_eq!(verdict(&m), "unknown", "the plain scenario has no result yet");
    ingest(&m, &[rec("Cold start", "pass")]);
    assert_eq!(verdict(&m), "pass");
    ingest(&m, &[row(2, "fail")]);
    assert_eq!(verdict(&m), "fail");
}

#[test]
fn the_verdict_is_judged_per_rig_when_a_plan_names_several() {
    let m = model(OUTLINE);
    std::fs::write(m.join("TE-HTR-002.md"), "---\nid: TE-HTR-002\ntype: TestEnvironment\nname: Rig 2\nstatus: available\nenvironmentKind: bench\n---\n\nRig.\n").unwrap();
    let on = |env: &str, n: usize, v: &str| format!(r#"{{"testCase":"TC-HTR-001","scenario":"Response within limit [row {n}]","verdict":"{v}","provenance":{{"environment":"{env}"}}}}"#);
    // Every row passes on rig 1; rig 2 ran only row 1.
    ingest(&m, &[on("TE-HTR-001", 1, "pass"), on("TE-HTR-001", 2, "pass"), on("TE-HTR-001", 3, "pass"), on("TE-HTR-002", 1, "pass")]);
    std::fs::write(m.join("TC-HTR-001.md"), std::fs::read_to_string(m.join("TC-HTR-001.md")).unwrap().replace("verifies: [REQ-HTR-001]", "verifies: [REQ-HTR-001]\nrunsOn: [TE-HTR-001, TE-HTR-002]")).unwrap();
    assert_eq!(verdict(&m), "unknown", "rig 2 has not run every condition");
    ingest(&m, &[on("TE-HTR-002", 2, "pass"), on("TE-HTR-002", 3, "pass")]);
    assert_eq!(verdict(&m), "pass");
    ingest(&m, &[on("TE-HTR-002", 3, "fail")]);
    assert_eq!(verdict(&m), "fail");
}

#[test]
fn testplan_and_audit_see_the_rows_too() {
    let m = model(OUTLINE);
    std::fs::create_dir_all(m.join("Plans")).unwrap();
    std::fs::write(m.join("Plans/TP-HTR-001.md"), "---\nid: TP-HTR-001\ntype: TestPlan\nname: Heater plan\nstatus: approved\ntestCases: [TC-HTR-001]\nrunsOn: [TE-HTR-001]\n---\n\nPlan.\n").unwrap();
    ingest(&m, &[row(1, "fail"), row(2, "pass")]);
    let (plan, _) = run(&m, &["testplan", "TP-HTR-001"]);
    assert!(plan.contains("Verdict: fail"), "{plan}");
    let (audit, _) = run(&m, &["audit"]);
    assert!(audit.contains("Active tests: 0 pass, 1 fail, 0 unknown"), "{audit}");
    ingest(&m, &[row(1, "pass"), row(3, "pass")]);
    let (plan, _) = run(&m, &["testplan", "TP-HTR-001"]);
    assert!(plan.contains("Verdict: pass"), "{plan}");
    let (audit, _) = run(&m, &["audit"]);
    assert!(audit.contains("Active tests: 1 pass, 0 fail, 0 unknown"), "{audit}");
}

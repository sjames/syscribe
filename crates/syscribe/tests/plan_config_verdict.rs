//! `REQ-TRS-PLANCFG-001` (GH #528): a plan that names its `configurations:` is judged on the results recorded
//! for those Configurations, each on its own; evidence of a Configuration it does not name never reaches it.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// A heater with three candidate variants and one plan; `plan` is the plan's extra frontmatter.
fn model(plan: &str, status: &str, tc_extra: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-plancfg-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/Opt.md", "---\ntype: FeatureDef\nid: FEAT-OPT-001\nname: Opt\ngroupKind: optional\n---\n\nO.\n");
    for (n, v) in [("001", "true"), ("002", "false"), ("003", "true")] {
        w(&format!("Configurations/CONF-HTR-{n}.md"), &format!("---\ntype: Configuration\nid: CONF-HTR-{n}\nname: Variant {n}\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: {v}\n---\n\nC.\n"));
    }
    w("REQ-HTR-001.md", "---\nid: REQ-HTR-001\ntype: Requirement\nname: \"Heater response\"\nstatus: approved\nreqDomain: system\nreqClass: system\n---\n\nThe heater shall reach the setpoint.\n");
    w("TE-HTR-001.md", "---\nid: TE-HTR-001\ntype: TestEnvironment\nname: Rig\nstatus: available\nenvironmentKind: bench\n---\n\nRig.\n");
    w("TC-HTR-001.md", &format!("---\nid: TC-HTR-001\ntype: TestCase\ntestLevel: L4\nstatus: active\nname: \"Heater response\"\nverifies: [REQ-HTR-001]\n{tc_extra}---\n\n```gherkin\nFeature: F\n  Scenario: Response\n    Given a heater\n```\n"));
    w("TP-HTR-001.md", &format!("---\ntype: TestPlan\nid: TP-HTR-001\nname: \"Confirm a heater variant\"\nstatus: {status}\nscope: hil\ntestCases: [TC-HTR-001]\n{plan}---\n\nPlan.\n"));
    root
}

/// The plan of the issue: one variant, one rig, two ambient points, two units.
const ISSUE: &str = "configurations: [CONF-HTR-001]\nrunsOn: [TE-HTR-001]\nconditions:\n  ambient: [{name: cold}, {name: room}]\nminUnits: 2\n";

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn rec(verdict: &str, cond: &str, unit: &str) -> String {
    let c = if cond.is_empty() { String::new() } else { format!(r#","condition":{{"ambient":"{cond}"}}"#) };
    let u = if unit.is_empty() { String::new() } else { format!(r#","unit":{{"id":"{unit}"}}"#) };
    format!(r#"{{"testCase":"TC-HTR-001","verdict":"{verdict}","provenance":{{"environment":"TE-HTR-001"{u}}}{c}}}"#)
}

fn ingest(root: &Path, config: Option<&str>, recs: &[String]) {
    let f = root.join("r.json");
    std::fs::write(&f, format!("[{}]", recs.join(","))).unwrap();
    let mut a = vec!["ingest-results", "--format", "records"];
    if let Some(c) = config {
        a.extend(["--config", c]);
    }
    a.push(f.to_str().unwrap());
    let (out, code) = run(root, &a);
    assert_eq!(code, 0, "{out}");
}

fn detail(root: &Path) -> serde_json::Value {
    let (out, _) = run(root, &["testplan", "TP-HTR-001", "--json"]);
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}\n{out}"))
}

fn verdict(root: &Path) -> String {
    detail(root)["verdict"].as_str().unwrap().to_string()
}

/// Passes at both points on two units.
fn full_pass() -> Vec<String> {
    ["cold", "room"].iter().flat_map(|c| ["S1", "S2"].map(|u| rec("pass", c, u))).collect()
}

#[test]
fn a_failure_recorded_for_another_configuration_does_not_touch_the_plan() {
    let m = model(ISSUE, "draft", "");
    ingest(&m, Some("CONF-HTR-001"), &full_pass());
    assert_eq!(verdict(&m), "pass");
    ingest(&m, Some("CONF-HTR-002"), &[rec("fail", "cold", "S1")]);
    let d = detail(&m);
    assert_eq!(d["verdict"], "pass", "CONF-HTR-002 is not this plan's variant: {d}");
    assert_eq!(d["requiredCoverage"]["gaps"].as_array().unwrap().len(), 0, "{d}");
    let (list, _) = run(&m, &["testplan"]);
    assert!(list.contains("| pass |"), "{list}");
}

#[test]
fn a_failure_recorded_for_the_plans_own_configuration_fails_it() {
    let m = model(ISSUE, "draft", "");
    ingest(&m, Some("CONF-HTR-001"), &full_pass());
    ingest(&m, Some("CONF-HTR-001"), &[rec("fail", "cold", "S1")]);
    let d = detail(&m);
    assert_eq!(d["verdict"], "fail");
    assert_eq!(d["requiredCoverage"]["gaps"][0]["configuration"], "CONF-HTR-001", "{d}");
    let (text, _) = run(&m, &["testplan", "TP-HTR-001"]);
    assert!(text.contains("TC-HTR-001 @ TE-HTR-001 [ambient=cold] in CONF-HTR-001 (FAILED)"), "{text}");
}

#[test]
fn a_pass_recorded_for_another_configuration_does_not_complete_the_plan() {
    let m = model(ISSUE, "draft", "");
    ingest(&m, Some("CONF-HTR-002"), &full_pass());
    assert_eq!(verdict(&m), "incomplete", "nothing was recorded for CONF-HTR-001");
}

#[test]
fn results_ingested_without_a_configuration_still_count_for_the_plan() {
    let m = model(ISSUE, "draft", "");
    ingest(&m, None, &full_pass());
    assert_eq!(verdict(&m), "pass");
    // another variant's failure still does not reach it, and no gap is tagged: no view is per Configuration
    ingest(&m, Some("CONF-HTR-002"), &[rec("fail", "cold", "S1")]);
    let d = detail(&m);
    assert_eq!(d["verdict"], "pass", "{d}");
    assert!(d["verdictByConfiguration"].is_null(), "{d}");
}

#[test]
fn a_plan_without_configurations_keeps_the_conservative_verdict() {
    let m = model("runsOn: [TE-HTR-001]\nconditions:\n  ambient: [{name: cold}, {name: room}]\nminUnits: 2\n", "draft", "");
    ingest(&m, Some("CONF-HTR-001"), &full_pass());
    assert_eq!(verdict(&m), "pass");
    ingest(&m, Some("CONF-HTR-002"), &[rec("fail", "cold", "S1")]);
    assert_eq!(verdict(&m), "fail", "a config-agnostic plan hides no variant's failure");
}

#[test]
fn several_configurations_are_each_judged_and_shown() {
    let m = model("configurations: [CONF-HTR-001, CONF-HTR-003]\nrunsOn: [TE-HTR-001]\nconditions:\n  ambient: [{name: cold}, {name: room}]\nminUnits: 2\n", "draft", "");
    ingest(&m, Some("CONF-HTR-001"), &full_pass());
    let d = detail(&m);
    assert_eq!(d["verdict"], "incomplete", "CONF-HTR-003 has not run: {d}");
    let by: Vec<(String, String)> = d["verdictByConfiguration"].as_array().unwrap().iter().map(|x| (x["configuration"].as_str().unwrap().to_string(), x["verdict"].as_str().unwrap().to_string())).collect();
    assert_eq!(by, vec![("CONF-HTR-001".to_string(), "pass".to_string()), ("CONF-HTR-003".to_string(), "incomplete".to_string())]);
    assert!(d["requiredCoverage"]["gaps"].as_array().unwrap().iter().all(|g| g["configuration"] == "CONF-HTR-003"), "{d}");
    let (text, _) = run(&m, &["testplan", "TP-HTR-001"]);
    assert!(text.contains("  - CONF-HTR-001: pass") && text.contains("  - CONF-HTR-003: incomplete"), "{text}");
    assert!(text.contains("in CONF-HTR-003 (missing)"), "{text}");

    ingest(&m, Some("CONF-HTR-003"), &full_pass());
    assert_eq!(verdict(&m), "pass");
    ingest(&m, Some("CONF-HTR-003"), &[rec("fail", "room", "S2")]);
    let d = detail(&m);
    assert_eq!(d["verdict"], "fail");
    assert_eq!(d["verdictByConfiguration"][0]["verdict"], "pass", "{d}");
}

#[test]
fn a_plan_without_required_coverage_is_scoped_the_same_way() {
    let m = model("configurations: [CONF-HTR-001]\n", "draft", "");
    ingest(&m, Some("CONF-HTR-001"), &[rec("pass", "", "")]);
    ingest(&m, Some("CONF-HTR-002"), &[rec("fail", "", "")]);
    assert_eq!(verdict(&m), "pass");
    ingest(&m, Some("CONF-HTR-001"), &[rec("fail", "", "")]);
    assert_eq!(verdict(&m), "fail");
}

#[test]
fn a_member_is_judged_only_in_the_configurations_it_is_part_of() {
    // The test applies with the option only: CONF-HTR-002 (option off) has nothing to run.
    let m = model("configurations: [CONF-HTR-001, CONF-HTR-002]\n", "draft", "appliesWhen: Features::Opt\n");
    ingest(&m, Some("CONF-HTR-001"), &[rec("pass", "", "")]);
    let d = detail(&m);
    assert_eq!(d["verdict"], "pass", "{d}");
}

#[test]
fn validate_gates_an_approved_plan_on_its_own_configurations_only() {
    let m = model(ISSUE, "approved", "");
    ingest(&m, Some("CONF-HTR-001"), &full_pass());
    ingest(&m, Some("CONF-HTR-002"), &[rec("fail", "cold", "S1")]);
    let (out, _) = run(&m, &["validate"]);
    assert!(!out.contains("W615") && !out.contains("W617"), "{out}");
    ingest(&m, Some("CONF-HTR-001"), &[rec("fail", "cold", "S1")]);
    let (out, _) = run(&m, &["validate"]);
    assert!(out.contains("W617") && out.contains("in CONF-HTR-001 (FAILED)"), "{out}");
    assert!(out.contains("W615") && out.contains("@ TE-HTR-001 in CONF-HTR-001 (FAILED)"), "{out}");
}

#[test]
fn audit_counts_the_plan_by_the_same_verdict() {
    let m = model(ISSUE, "draft", "");
    ingest(&m, Some("CONF-HTR-001"), &full_pass());
    ingest(&m, Some("CONF-HTR-002"), &[rec("fail", "cold", "S1")]);
    let (out, _) = run(&m, &["audit", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}\n{out}"));
    let plans = v.pointer("/verification/plans").unwrap_or_else(|| panic!("{out}"));
    assert_eq!(plans["pass"], 1, "{plans}");
    assert_eq!(plans["fail"], 0, "{plans}");
}

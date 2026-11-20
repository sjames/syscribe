//! `REQ-TRS-PLANCOV-002` (ADR-SYS-COVERAGE-001, GH #500/#501, slice 2b): a plan passes only when every cell of
//! its declared grid (member × rig × condition point, on `minUnits` units) has a pass.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(plan_extra: &str, status: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-plancov2-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("REQ-HTR-001.md", "---\nid: REQ-HTR-001\ntype: Requirement\nname: \"Heater response\"\nstatus: approved\nreqDomain: system\nreqClass: system\n---\n\nThe heater shall reach the setpoint.\n");
    for n in ["001", "002"] {
        w(&format!("TE-HTR-{n}.md"), &format!("---\nid: TE-HTR-{n}\ntype: TestEnvironment\nname: Rig {n}\nstatus: available\nenvironmentKind: bench\n---\n\nRig.\n"));
    }
    w("TC-HTR-001.md", "---\nid: TC-HTR-001\ntype: TestCase\ntestLevel: L4\nstatus: active\nname: \"Heater response\"\nverifies: [REQ-HTR-001]\n---\n\n```gherkin\nFeature: F\n  Scenario: Response\n    Given a heater\n```\n");
    w("TP-HTR-001.md", &format!("---\ntype: TestPlan\nid: TP-HTR-001\nname: \"Heater plan\"\nstatus: {status}\ntestCases: [TC-HTR-001]\n{plan_extra}---\n\nPlan.\n"));
    root
}

const AMB: &str = "conditions:\n  ambient: [cold, nominal, hot]\n";

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn rec(verdict: &str, cond: &str, env: &str, unit: &str) -> String {
    let c = if cond.is_empty() { String::new() } else { format!(r#","condition":{{"ambient":"{cond}"}}"#) };
    let mut prov = Vec::new();
    if !env.is_empty() {
        prov.push(format!(r#""environment":"{env}""#));
    }
    if !unit.is_empty() {
        prov.push(format!(r#""unit":{{"id":"{unit}"}}"#));
    }
    format!(r#"{{"testCase":"TC-HTR-001","verdict":"{verdict}","provenance":{{{}}}{c}}}"#, prov.join(","))
}

fn ingest(root: &Path, recs: &[String]) {
    let f = root.join("r.json");
    std::fs::write(&f, format!("[{}]", recs.join(","))).unwrap();
    let (out, code) = run(root, &["ingest-results", "--format", "records", f.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
}

fn verdict(root: &Path) -> String {
    let (out, _) = run(root, &["testplan", "TP-HTR-001", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}\n{out}"));
    v["verdict"].as_str().unwrap().to_string()
}

#[test]
fn a_plan_that_never_ran_one_of_its_conditions_is_incomplete_not_pass() {
    let m = model(AMB, "draft");
    ingest(&m, &[rec("pass", "cold", "", ""), rec("pass", "nominal", "", "")]);
    assert_eq!(verdict(&m), "incomplete", "hot never ran");
    ingest(&m, &[rec("pass", "hot", "", "")]);
    assert_eq!(verdict(&m), "pass");
}

#[test]
fn a_failure_at_any_point_fails_the_plan() {
    let m = model(AMB, "draft");
    ingest(&m, &[rec("pass", "cold", "", ""), rec("fail", "hot", "", "")]);
    assert_eq!(verdict(&m), "fail");
}

#[test]
fn without_a_declaration_the_verdict_is_what_it_was() {
    let m = model("", "draft");
    ingest(&m, &[rec("pass", "", "", "")]);
    assert_eq!(verdict(&m), "pass");
}

#[test]
fn min_units_needs_distinct_passing_units() {
    let m = model(&format!("{AMB}minUnits: 2\n"), "draft");
    let all = |unit: &str| ["cold", "nominal", "hot"].map(|c| rec("pass", c, "", unit)).to_vec();
    ingest(&m, &all("SN-1"));
    assert_eq!(verdict(&m), "incomplete", "one unit");
    ingest(&m, &all("SN-2"));
    assert_eq!(verdict(&m), "pass", "two units");
}

#[test]
fn every_rig_needs_every_point() {
    let m = model(&format!("{AMB}runsOn: [TE-HTR-001, TE-HTR-002]\n"), "draft");
    let on = |env: &str| ["cold", "nominal", "hot"].map(|c| rec("pass", c, env, "")).to_vec();
    ingest(&m, &on("TE-HTR-001"));
    assert_eq!(verdict(&m), "incomplete", "rig 2 has nothing");
    ingest(&m, &on("TE-HTR-002"));
    assert_eq!(verdict(&m), "pass");
}

#[test]
fn testplan_detail_lists_the_gaps_and_json_carries_them() {
    let m = model(&format!("{AMB}minUnits: 2\n"), "draft");
    ingest(&m, &[rec("pass", "cold", "", "SN-1"), rec("pass", "cold", "", "SN-2"), rec("pass", "nominal", "", "SN-1")]);
    let (out, _) = run(&m, &["testplan", "TP-HTR-001"]);
    assert!(out.contains("## Required coverage"), "{out}");
    assert!(out.contains("TC-HTR-001 [ambient=nominal] (1 of 2 units)"), "{out}");
    assert!(out.contains("TC-HTR-001 [ambient=hot] (missing)"), "{out}");
    let (j, _) = run(&m, &["testplan", "TP-HTR-001", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    let gaps = v["requiredCoverage"]["gaps"].as_array().unwrap();
    assert_eq!(gaps.len(), 2, "{j}");
    assert_eq!(v["requiredCoverage"]["minUnits"], 2);
    assert!(gaps.iter().any(|g| g["reason"] == "units" && g["units"]["have"] == 1 && g["units"]["need"] == 2));
}

#[test]
fn an_approved_plan_with_gaps_raises_w617_and_a_complete_one_does_not() {
    let m = model(AMB, "approved");
    ingest(&m, &[rec("pass", "cold", "", "")]);
    let (out, _) = run(&m, &["validate"]);
    assert!(out.contains("W617") && out.contains("[ambient=hot] (missing)") && out.contains("[ambient=nominal] (missing)"), "{out}");
    ingest(&m, &[rec("pass", "nominal", "", ""), rec("pass", "hot", "", "")]);
    let (out, _) = run(&m, &["validate"]);
    assert!(!out.contains("W617"), "{out}");
    // a draft plan is not held to it
    let d = model(AMB, "draft");
    ingest(&d, &[rec("pass", "cold", "", "")]);
    assert!(!run(&d, &["validate"]).0.contains("W617"));
}

#[test]
fn the_plan_list_shows_the_same_verdict() {
    let m = model(AMB, "draft");
    ingest(&m, &[rec("pass", "cold", "", "")]);
    let (out, _) = run(&m, &["testplan"]);
    assert!(out.contains("incomplete"), "{out}");
}

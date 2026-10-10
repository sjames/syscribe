//! TC-TRS-AUDRES-001 / GH #256.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(results: Option<&str>) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir()
        .join(format!("syscribe-audres-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)))
        .join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w(
        "HE-AU-001.md",
        "---\ntype: HazardousEvent\nid: HE-AU-001\nname: h\nstatus: approved\nseverity: S3\nexposure: E4\ncontrollability: C3\nasilLevel: D\n---\n\nHazard.\n",
    );
    w(
        "SG-AU-001.md",
        "---\ntype: SafetyGoal\nid: SG-AU-001\nname: g\nstatus: approved\nasilLevel: D\nsafeState: stop\nftti: 50ms\nhazardousEvents: [HE-AU-001]\n---\n\nGoal.\n",
    );
    w(
        "REQ-AU-001.md",
        "---\ntype: Requirement\nid: REQ-AU-001\nname: r\nstatus: approved\nreqDomain: software\nasilLevel: D\nderivedFromSafetyGoal: SG-AU-001\n---\n\nThe system shall.\n",
    );
    let tc = |id: &str, func: &str| {
        format!("---\ntype: TestCase\nid: {id}\nname: t\nstatus: active\ntestLevel: L4\nverifies: [REQ-AU-001]\ntestFunctions:\n  - function: \"{func}\"\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n")
    };
    w("TC/TC-AU-001.md", &tc("TC-AU-001", "fn_fail"));
    w("TC/TC-AU-002.md", &tc("TC-AU-002", "fn_pass"));
    w(
        "TP/TP-AUDIT-001.md",
        "---\ntype: TestPlan\nid: TP-AUDIT-001\nname: p\nstatus: approved\nscope: integration\ntestCases: [TC-AU-001, TC-AU-002]\n---\n\nPlan.\n",
    );
    if let Some(res) = results {
        w(".syscribe/results.json", res);
    }
    r
}

const FAILING: &str = r#"{"schema_version":"1.0","format":"junit","source":"x","ingested_at_unix":1,"count":2,"by_leaf":{"fn_fail":"fail","fn_pass":"pass"}}"#;
const PASSING: &str = r#"{"schema_version":"1.0","format":"junit","source":"x","ingested_at_unix":1,"count":2,"by_leaf":{"fn_fail":"pass","fn_pass":"pass"}}"#;

fn audit(root: &Path) -> serde_json::Value {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(["audit", "--json"]).output().unwrap();
    let s = String::from_utf8_lossy(&o.stdout);
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("{e}: {s}"))
}

fn reasons(v: &serde_json::Value) -> String {
    v["verdict"]["reasons"].as_array().unwrap().iter().map(|r| r.as_str().unwrap()).collect::<Vec<_>>().join(" | ")
}

#[test]
fn failing_goal_fails_the_verdict_and_is_named() {
    let v = audit(&model(Some(FAILING)));
    assert_eq!(v["verdict"]["pass"], false, "{v}");
    let r = reasons(&v);
    assert!(r.contains("FAILING") && r.contains("SG-AU-001"), "{r}");
}

#[test]
fn verification_section_counts_tests_goals_and_plans() {
    let v = audit(&model(Some(FAILING)));
    let s = &v["verification"];
    assert_eq!(s["tests"]["pass"], 1, "{s}");
    assert_eq!(s["tests"]["fail"], 1, "{s}");
    assert_eq!(s["tests"]["unknown"], 0, "{s}");
    assert_eq!(s["goals"]["failing"], 1, "{s}");
    assert_eq!(s["plans"]["fail"], 1, "{s}");
}

#[test]
fn approved_requirement_with_failing_verifier_is_a_default_audit_reason() {
    let r = reasons(&audit(&model(Some(FAILING))));
    assert!(r.contains("W312"), "{r}");
}

#[test]
fn passing_results_show_the_section_but_no_failing_reasons() {
    let v = audit(&model(Some(PASSING)));
    assert_eq!(v["verification"]["tests"]["pass"], 2, "{v}");
    assert_eq!(v["verification"]["goals"]["failing"], 0, "{v}");
    assert_eq!(v["verification"]["goals"]["supported"], 1, "{v}");
    let r = reasons(&v);
    assert!(!r.contains("FAILING") && !r.contains("W312"), "{r}");
}

#[test]
fn without_results_the_section_is_null() {
    let v = audit(&model(None));
    assert!(v["verification"].is_null(), "{v}");
    assert!(!reasons(&v).contains("FAILING"));
}

#[test]
fn text_output_has_the_section() {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model(Some(FAILING))).arg("audit").output().unwrap();
    let s = String::from_utf8_lossy(&o.stdout);
    assert!(s.contains("Verification results") && s.contains("FAILING"), "{s}");
    assert!(s.contains("Verdict: **FAIL**"), "{s}");
}

#[test]
fn plan_scoped_audit_reports_tests_only_and_does_not_apply_goal_verdicts() {
    // Goals and plans are model-level; a plan-scoped audit counts the plan's tests and
    // leaves goal/plan verdicts out rather than silently reporting zeros.
    let root = model(Some(FAILING));
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(&root)
        .args(["audit", "--json", "--plan", "TP-AUDIT-001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&o.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).unwrap_or_else(|e| panic!("{e}: {s}"));
    assert_eq!(v["verification"]["tests"]["fail"], 1, "{v}");
    assert!(v["verification"]["goals"].is_null(), "{v}");
    assert!(v["verification"]["plans"].is_null(), "{v}");
    assert!(!reasons(&v).contains("FAILING"), "{}", reasons(&v));
}

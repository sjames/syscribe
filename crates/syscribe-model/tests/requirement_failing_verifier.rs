//! TC-TRS-VRES-001 / GH #257 (a): W312 / E319 on a requirement with a failing active verifier.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::{validate_with_config, Finding};
use syscribe_model::walker::walk_model;

fn run(req_status: &str, tcs: &[(&str, &str, Option<&str>)], results: Option<&str>) -> Vec<Finding> {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-vres-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    let w = |rel: &str, c: &str| {
        let p: PathBuf = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w(
        "REQ-VR-001.md",
        &format!("---\ntype: Requirement\nid: REQ-VR-001\nname: r\nstatus: {req_status}\nreqDomain: software\n---\n\nThe system shall.\n"),
    );
    for (id, status, func) in tcs {
        let tf = func.map(|f| format!("testFunctions:\n  - function: \"{f}\"\n")).unwrap_or_default();
        w(
            &format!("TC/{id}.md"),
            &format!("---\ntype: TestCase\nid: {id}\nname: t\nstatus: {status}\ntestLevel: L3\nverifies: [REQ-VR-001]\n{tf}---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n"),
        );
    }
    if let Some(res) = results {
        w(".syscribe/results.json", res);
    }
    let els = walk_model(&r).unwrap();
    validate_with_config(&els, &ValidateConfig::with_model_root(&r)).findings
}

const RES: &str = r#"{"schema_version":"1.0","format":"junit","source":"x","ingested_at_unix":1,"count":2,
 "by_leaf":{"fn_bad":"fail","fn_ok":"pass"}}"#;

fn n(f: &[Finding], code: &str) -> usize {
    f.iter().filter(|x| x.code == code).count()
}

#[test]
fn approved_and_implemented_get_w312() {
    for st in ["approved", "implemented"] {
        let f = run(st, &[("TC-VR-001", "active", Some("fn_bad"))], Some(RES));
        assert_eq!(n(&f, "W312"), 1, "{st}: {f:?}");
        assert_eq!(n(&f, "E319"), 0);
    }
}

#[test]
fn verified_gets_e319_not_w312() {
    let f = run("verified", &[("TC-VR-001", "active", Some("fn_bad"))], Some(RES));
    assert_eq!(n(&f, "E319"), 1, "{f:?}");
    assert_eq!(n(&f, "W312"), 0);
}

#[test]
fn all_failing_ids_are_named_even_with_a_passing_verifier() {
    let f = run(
        "approved",
        &[("TC-VR-002", "active", Some("fn_bad")), ("TC-VR-001", "active", Some("fn_ok")), ("TC-VR-003", "active", Some("fn_bad"))],
        Some(RES),
    );
    let w: Vec<_> = f.iter().filter(|x| x.code == "W312").collect();
    assert_eq!(w.len(), 1, "{f:?}");
    assert!(w[0].message.contains("TC-VR-002, TC-VR-003"), "{}", w[0].message);
    assert!(!w[0].message.contains("TC-VR-001"), "{}", w[0].message);
}

#[test]
fn quiet_when_not_failing_or_not_applicable() {
    // passing
    assert_eq!(n(&run("verified", &[("TC-VR-001", "active", Some("fn_ok"))], Some(RES)), "E319"), 0);
    // draft requirement
    let f = run("draft", &[("TC-VR-001", "active", Some("fn_bad"))], Some(RES));
    assert_eq!(n(&f, "W312") + n(&f, "E319"), 0);
    // failing but not active
    let f = run("approved", &[("TC-VR-001", "draft", Some("fn_bad"))], Some(RES));
    assert_eq!(n(&f, "W312"), 0);
    // missing function is Unknown, not a failure
    let f = run("approved", &[("TC-VR-001", "active", Some("fn_absent"))], Some(RES));
    assert_eq!(n(&f, "W312"), 0);
    // no results ingested
    let f = run("verified", &[("TC-VR-001", "active", Some("fn_bad"))], None);
    assert_eq!(n(&f, "W312") + n(&f, "E319"), 0);
}

#[test]
fn session_log_scenario_failures_are_caught_too() {
    // A TestCase with no testFunctions is scored from per-scenario session-log verdicts.
    let res = r#"{"schema_version":"1.0","format":"session-log","source":"x","ingested_at_unix":1,"count":1,
     "by_leaf":{}, "by_scenario":{"TC-VR-001::s":"fail"}}"#;
    let f = run("approved", &[("TC-VR-001", "active", None)], Some(res));
    assert_eq!(n(&f, "W312"), 1, "{f:?}");
}

#[test]
fn a_failing_verifier_beside_a_passing_one_still_flags_verified() {
    let f = run("verified", &[("TC-VR-001", "active", Some("fn_ok")), ("TC-VR-002", "active", Some("fn_bad"))], Some(RES));
    assert_eq!(n(&f, "E319"), 1, "{f:?}");
}

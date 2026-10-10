//! TC-TRS-TPNOISE-001 / GH #255, #260.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::{validate_with_config, Finding};
use syscribe_model::walker::walk_model;

fn model(files: &[(String, String)], results: Option<&str>) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-tpn-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (rel, c) in files {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    if let Some(res) = results {
        std::fs::create_dir_all(r.join(".syscribe")).unwrap();
        std::fs::write(r.join(".syscribe/results.json"), res).unwrap();
    }
    r
}

fn tc(n: u32, func: Option<&str>) -> (String, String) {
    let tf = func.map(|f| format!("testFunctions:\n  - function: \"{f}\"\n")).unwrap_or_default();
    (
        format!("TC/TC-TPN-{n:03}.md"),
        format!("---\ntype: TestCase\nid: TC-TPN-{n:03}\nname: t{n}\nstatus: active\ntestLevel: L3\nverifies: [REQ-TPN-001]\n{tf}---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n"),
    )
}

fn plan(id: &str, scope: &str, cases: &[u32]) -> (String, String) {
    let list = cases.iter().map(|n| format!("TC-TPN-{n:03}")).collect::<Vec<_>>().join(", ");
    (
        format!("TP/{id}.md"),
        format!("---\ntype: TestPlan\nid: {id}\nname: {id}\nstatus: approved\nscope: {scope}\ntestCases: [{list}]\n---\n\nPlan.\n"),
    )
}

fn req() -> (String, String) {
    (
        "REQ-TPN-001.md".into(),
        "---\ntype: Requirement\nid: REQ-TPN-001\nname: r\nstatus: approved\nreqDomain: software\n---\n\nThe system shall.\n".into(),
    )
}

fn findings(files: Vec<(String, String)>, results: Option<&str>) -> Vec<Finding> {
    let r = model(&files, results);
    let els = walk_model(&r).unwrap();
    validate_with_config(&els, &ValidateConfig::with_model_root(&r)).findings
}

fn count(f: &[Finding], code: &str) -> usize {
    f.iter().filter(|x| x.code == code).count()
}

#[test]
fn w616_ignores_distinct_plans_that_share_a_scope() {
    let mut files = vec![req()];
    for n in 1..=6 {
        files.push(tc(n, None));
    }
    files.push(plan("TP-ALPHA-001", "security", &[1, 2]));
    files.push(plan("TP-BRAVO-001", "security", &[3, 4]));
    files.push(plan("TP-CHARLIE-001", "security", &[5, 6]));
    assert_eq!(count(&findings(files, None), "W616"), 0);
}

#[test]
fn w616_flags_identical_and_subset_plans_once() {
    let mut files = vec![req()];
    for n in 1..=4 {
        files.push(tc(n, None));
    }
    files.push(plan("TP-ALPHA-001", "smoke", &[1, 2, 3, 4]));
    files.push(plan("TP-BRAVO-001", "smoke", &[1, 2, 3, 4]));
    let f = findings(files, None);
    assert_eq!(count(&f, "W616"), 1, "{f:?}");
    assert!(f.iter().any(|x| x.code == "W616" && x.message.contains("4 of 4")), "{f:?}");

    let mut files = vec![req()];
    for n in 1..=4 {
        files.push(tc(n, None));
    }
    files.push(plan("TP-ALPHA-001", "smoke", &[1, 2, 3, 4]));
    files.push(plan("TP-BRAVO-001", "smoke", &[1, 2]));
    assert_eq!(count(&findings(files, None), "W616"), 1);
}

#[test]
fn w616_threshold_is_jaccard_half() {
    // {1,2,3} vs {3,4,5}: 1/5 overlap, neither contains the other -> no finding.
    let mut files = vec![req()];
    for n in 1..=5 {
        files.push(tc(n, None));
    }
    files.push(plan("TP-ALPHA-001", "regression", &[1, 2, 3]));
    files.push(plan("TP-BRAVO-001", "regression", &[3, 4, 5]));
    assert_eq!(count(&findings(files.clone(), None), "W616"), 0);
    // {1,2,3,4} vs {2,3,4,5}: 3/5 = 0.6 -> flagged.
    let mut files = vec![req()];
    for n in 1..=5 {
        files.push(tc(n, None));
    }
    files.push(plan("TP-ALPHA-001", "regression", &[1, 2, 3, 4]));
    files.push(plan("TP-BRAVO-001", "regression", &[2, 3, 4, 5]));
    assert_eq!(count(&findings(files, None), "W616"), 1);
}

const RESULTS: &str = r#"{"schema_version":"1.0","format":"junit","source":"x","ingested_at_unix":1,"count":2,
 "by_leaf":{"fn_one":"fail"}}"#;

#[test]
fn w615_is_one_finding_per_plan() {
    let mut files = vec![req()];
    files.push(tc(1, Some("fn_one")));
    files.push(tc(2, Some("fn_two")));
    files.push(plan("TP-ALPHA-001", "integration", &[1, 2]));
    files.push(plan("TP-BRAVO-001", "hil", &[1, 2]));
    let f = findings(files, Some(RESULTS));
    assert_eq!(count(&f, "W615"), 2, "{:?}", f.iter().filter(|x| x.code == "W615").collect::<Vec<_>>());
    for x in f.iter().filter(|x| x.code == "W615") {
        assert!(x.message.contains("TC-TPN-001") && x.message.contains("fn_one"), "{}", x.message);
        assert!(x.message.contains("TC-TPN-002") && x.message.contains("fn_two"), "{}", x.message);
        assert!(x.message.contains("FAILED") && x.message.contains("missing"), "{}", x.message);
    }
}

#[test]
fn w615_caps_the_listed_functions() {
    let mut files = vec![req()];
    for n in 1..=12 {
        files.push(tc(n, Some(&format!("fn_missing_{n}"))));
    }
    files.push(plan("TP-ALPHA-001", "integration", &(1..=12).collect::<Vec<_>>()));
    let f = findings(files, Some(RESULTS));
    let w: Vec<_> = f.iter().filter(|x| x.code == "W615").collect();
    assert_eq!(w.len(), 1);
    assert!(w[0].message.contains("12 "), "{}", w[0].message);
    assert!(w[0].message.contains("and 2 more"), "{}", w[0].message);
}

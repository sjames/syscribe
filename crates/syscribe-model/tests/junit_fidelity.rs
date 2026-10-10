//! TC-TRS-JUNIT-001 / GH #259.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::config::ValidateConfig;
use syscribe_model::results::{FnVerdict, ResultsData};
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

fn junit(body: &str) -> ResultsData {
    ResultsData::parse_junit(&format!("<testsuite>{body}</testsuite>"), "t.xml")
}

#[test]
fn same_test_name_in_two_classes_is_kept_apart() {
    let d = junit(
        r#"<testcase classname="com.A" name="test_x"/>
           <testcase classname="com.B" name="test_x"><failure message="m"/></testcase>"#,
    );
    assert_eq!(d.verdict_for("com.A#test_x"), FnVerdict::Pass);
    assert_eq!(d.verdict_for("com.B#test_x"), FnVerdict::Fail);
    assert_eq!(d.verdict_for("com.A::test_x"), FnVerdict::Pass);
    // A leaf-only reference is conservative: the worst of the colliding tests.
    assert_eq!(d.verdict_for("test_x"), FnVerdict::Fail);
}

#[test]
fn a_flaky_failure_is_flaky_not_pass() {
    let d = junit(r#"<testcase classname="c.T" name="test_f"><flakyFailure message="x"/></testcase>"#);
    assert_eq!(d.verdict_for("test_f"), FnVerdict::Flaky);
    assert_eq!(d.verdict_for("c.T#test_f"), FnVerdict::Flaky);
}

#[test]
fn rerun_children_without_a_failure_are_flaky_and_with_a_failure_are_fail() {
    let d = junit(
        r#"<testcase classname="c.T" name="a"><rerunError message="x"/></testcase>
           <testcase classname="c.T" name="b"><failure message="x"/><rerunFailure message="y"/></testcase>"#,
    );
    assert_eq!(d.verdict_for("a"), FnVerdict::Flaky);
    assert_eq!(d.verdict_for("b"), FnVerdict::Fail);
}

#[test]
fn plain_results_without_classname_behave_as_before() {
    let d = junit(r#"<testcase name="t1"/><testcase name="t2"><skipped/></testcase>"#);
    assert_eq!(d.verdict_for("t1"), FnVerdict::Pass);
    assert_eq!(d.verdict_for("t2"), FnVerdict::Ignored);
    assert_eq!(d.count, 2);
}

#[test]
fn the_testcase_count_is_not_inflated_by_qualified_keys() {
    let d = junit(r#"<testcase classname="c.T" name="a"/><testcase classname="c.T" name="b"/>"#);
    assert_eq!(d.count, 2);
}

#[test]
fn flaky_function_raises_w010_and_does_not_count_as_passing() {
    static N: AtomicU64 = AtomicU64::new(0);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-flaky-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(r.join("TC")).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(
        r.join("TC/TC-FL-001.md"),
        "---\ntype: TestCase\nid: TC-FL-001\nname: t\nstatus: active\ntestLevel: L3\ntestFunctions:\n  - function: \"c.T#test_f\"\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n",
    )
    .unwrap();
    junit(r#"<testcase classname="c.T" name="test_f"><flakyFailure message="x"/></testcase>"#).merge_into_sidecar(&r).unwrap();
    let els = walk_model(&r).unwrap();
    let f = validate_with_config(&els, &ValidateConfig::with_model_root(&r)).findings;
    let w: Vec<_> = f.iter().filter(|x| x.code == "W010").collect();
    assert_eq!(w.len(), 1, "{f:?}");
    assert!(w[0].message.contains("flaky") && w[0].message.contains("retry"), "{}", w[0].message);
    let tc = els.iter().find(|e| e.frontmatter.id.as_deref() == Some("TC-FL-001")).unwrap();
    let results = ResultsData::load_sidecar(&r);
    assert_ne!(
        syscribe_model::results::testcase_verdict(tc, results.as_ref()),
        syscribe_model::safety_case::Verdict::Pass
    );
}

#[test]
fn dotted_references_are_class_qualified_too() {
    let d = junit(
        r#"<testcase classname="com.A" name="test_x"/>
           <testcase classname="com.B" name="test_x"><failure message="m"/></testcase>"#,
    );
    assert_eq!(d.verdict_for("com.A.test_x"), FnVerdict::Pass);
    assert_eq!(d.verdict_for("com.B.test_x"), FnVerdict::Fail);
}

#[test]
fn flaky_error_and_a_class_less_flaky_case() {
    let d = junit(r#"<testcase classname="c.T" name="e"><flakyError message="x"/></testcase><testcase name="n"><flakyFailure/></testcase>"#);
    assert_eq!(d.verdict_for("c.T#e"), FnVerdict::Flaky);
    assert_eq!(d.verdict_for("n"), FnVerdict::Flaky);
}

#[test]
fn an_approved_plan_with_a_flaky_member_raises_w615() {
    static N: AtomicU64 = AtomicU64::new(0);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-flakyplan-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(r.join("TC")).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(
        r.join("TC/TC-FL-001.md"),
        "---\ntype: TestCase\nid: TC-FL-001\nname: t\nstatus: active\ntestLevel: L3\ntestFunctions:\n  - function: \"test_f\"\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n",
    )
    .unwrap();
    std::fs::write(
        r.join("TC/TP-FLAKY-001.md"),
        "---\ntype: TestPlan\nid: TP-FLAKY-001\nname: p\nstatus: approved\nscope: integration\ntestCases: [TC-FL-001]\n---\n\nP.\n",
    )
    .unwrap();
    junit(r#"<testcase name="test_f"><flakyFailure/></testcase>"#).merge_into_sidecar(&r).unwrap();
    let els = walk_model(&r).unwrap();
    let f = validate_with_config(&els, &ValidateConfig::with_model_root(&r)).findings;
    let w: Vec<_> = f.iter().filter(|x| x.code == "W615").collect();
    assert_eq!(w.len(), 1, "{f:?}");
    assert!(w[0].message.contains("flaky"), "{}", w[0].message);
}

#[test]
fn an_older_sidecar_without_flaky_values_still_loads() {
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-oldside-{}", std::process::id()));
    std::fs::create_dir_all(r.join(".syscribe")).unwrap();
    std::fs::write(
        r.join(".syscribe/results.json"),
        r#"{"schema_version":"1.0","format":"junit","source":"x","ingested_at_unix":1,"count":1,"by_leaf":{"t":"pass"}}"#,
    )
    .unwrap();
    let d = ResultsData::load_sidecar(&r).expect("loads");
    assert_eq!(d.verdict_for("t"), FnVerdict::Pass);
}

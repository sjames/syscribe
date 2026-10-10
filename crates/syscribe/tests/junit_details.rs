//! TC-TRS-JUNITDET-001 / GH #259.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-jdet-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let m = d.join("model");
    std::fs::create_dir_all(&m).unwrap();
    std::fs::write(m.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(m.join("REQ-JD-001.md"), "---\nid: REQ-JD-001\ntype: Requirement\nname: r\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\nShall.\n").unwrap();
    std::fs::write(
        m.join("TC-JD-001.md"),
        "---\nid: TC-JD-001\ntype: TestCase\nname: t\nstatus: active\ntestLevel: L3\nverifies: [REQ-JD-001]\ntestFunctions:\n  - function: t_pass\n  - function: t_skip\n  - function: t_gone\n  - function: t_fail\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n",
    )
    .unwrap();
    d
}

fn xml(d: &Path) -> String {
    let p = d.join("r.xml");
    std::fs::write(
        &p,
        r#"<testsuite>
<testcase classname="c.T" name="t_pass" time="0.5"/>
<testcase classname="c.T" name="t_fail" time="1.25"><failure message="latency 61 ms &gt; 50 ms limit"/></testcase>
<testcase classname="c.T" name="t_skip" time="0"><skipped message="rig offline"/></testcase>
<testcase classname="c.T" name="t_flaky" time="2"><flakyFailure message="first attempt timed out"/></testcase>
</testsuite>"#,
    )
    .unwrap();
    p.to_string_lossy().into_owned()
}

fn run(d: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d.join("model")).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

#[test]
fn details_are_retained_for_non_passing_cases_only() {
    let d = model();
    let x = xml(&d);
    assert_eq!(run(&d, &["ingest-results", "--format", "junit", &x]).1, 0);
    let s: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(d.join("model/.syscribe/results.json")).unwrap()).unwrap();
    let det = &s["details"];
    assert_eq!(det["t_fail"]["message"], "latency 61 ms > 50 ms limit", "{s}");
    assert_eq!(det["t_fail"]["time"], 1.25);
    assert_eq!(det["c.T::t_fail"]["message"], "latency 61 ms > 50 ms limit");
    assert_eq!(det["t_skip"]["message"], "rig offline");
    assert_eq!(det["t_flaky"]["message"], "first attempt timed out");
    assert!(det.get("t_pass").is_none(), "{s}");
}

#[test]
fn failures_lists_message_and_time() {
    let d = model();
    let x = xml(&d);
    run(&d, &["ingest-results", "--format", "junit", &x]);
    let (o, c) = run(&d, &["results", "failures"]);
    assert_eq!(c, 0, "{o}");
    for s in ["t_fail", "fail", "latency 61 ms > 50 ms limit", "1.25", "t_skip", "rig offline", "t_flaky", "flaky"] {
        assert!(o.contains(s), "{s}: {o}");
    }
    assert!(!o.contains("t_pass"), "{o}");
    let (j, _) = run(&d, &["results", "failures", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["failures"].as_array().unwrap().len(), 3, "{j}");
}

#[test]
fn ingest_summarises_missing_and_skipped_expected_functions() {
    let d = model();
    let x = xml(&d);
    let (o, c) = run(&d, &["ingest-results", "--format", "junit", &x]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("Expected functions: 4; not run: 1 missing, 1 skipped"), "{o}");
    assert!(o.contains("t_gone") && o.contains("t_skip"), "{o}");
}

#[test]
fn a_session_log_ingest_keeps_function_details() {
    let d = model();
    let x = xml(&d);
    run(&d, &["ingest-results", "--format", "junit", &x]);
    let log = d.join("log.json");
    std::fs::write(&log, r#"[{"testCase":"TC-JD-001","scenario":"s","steps":[{"cmd":"x"}],"result":"pass"}]"#).unwrap();
    assert_eq!(run(&d, &["ingest-results", "--format", "session-log", log.to_str().unwrap()]).1, 0);
    let (o, _) = run(&d, &["results", "failures"]);
    assert!(o.contains("latency 61 ms"), "{o}");
}

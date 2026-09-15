//! Regression tests for `ingest-results --format session-log` (issue #113):
//! manual/exploratory verification evidence, machine-checkable the same way
//! `cargo-json`/`junit` results already are.
//!
//! Black-box: builds a minimal, self-contained model in a fresh temp
//! directory per test and drives the `syscribe` binary against it.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_model() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir()
        .join(format!("syscribe-sesslog-{}-{}-{}", std::process::id(), nanos, n))
        .join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: SessionLog\n---\n\nModel root.\n");
    root
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn run(root: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn syscribe");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

/// A leaf, approved Requirement verified by one TestCase with two Gherkin
/// scenarios and no `testFunctions:` (the norm for a manually-verified one).
fn seed_req_and_tc(root: &Path) {
    write(
        root,
        "REQ-SL-001.md",
        "---\nid: REQ-SL-001\ntype: Requirement\nname: \"A requirement\"\nstatus: approved\nreqDomain: system\n---\n\nBody.\n",
    );
    write(
        root,
        "TC-SL-001.md",
        "---\nid: TC-SL-001\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: \"Manually verified via a live session\"\nverifies: [REQ-SL-001]\n---\n\n```gherkin\nFeature: manual verification\n  Scenario: An empty allowlist denies everything\n    Given an empty allowlist\n    When a command is issued\n    Then it is denied\n\n  Scenario: A configured allowlist permits listed commands\n    Given a configured allowlist\n    When an allowed command is issued\n    Then it succeeds\n```\n",
    );
}

fn write_session_log(root: &Path, rel: &str, records: &str) {
    write(root, rel, records);
}

#[test]
fn ingests_and_writes_the_sidecar() {
    let root = new_model();
    seed_req_and_tc(&root);
    write_session_log(
        &root,
        "session.json",
        r#"[
            {"testCase": "TC-SL-001", "scenario": "An empty allowlist denies everything", "steps": [{"cmd": "curl"}], "result": "pass"},
            {"testCase": "TC-SL-001", "scenario": "A configured allowlist permits listed commands", "steps": [{"cmd": "curl"}], "result": "pass"}
        ]"#,
    );
    let (stdout, _stderr, code) = run(&root, &["ingest-results", "--format", "session-log", root.join("session.json").to_str().unwrap()]);
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("2 pass, 0 fail, 0 ignored"), "{stdout}");
    assert!(root.join(".syscribe/results.json").exists());
}

#[test]
fn trace_annotates_the_testcase_pass_after_ingest_but_not_before() {
    let root = new_model();
    seed_req_and_tc(&root);

    let (before, _err, _code) = run(&root, &["trace", "REQ-SL-001"]);
    assert!(before.contains("TC-SL-001"), "{before}");
    assert!(!before.contains("TC-SL-001 [pass]"), "must not show a verdict before any ingest: {before}");

    write_session_log(
        &root,
        "session.json",
        r#"[
            {"testCase": "TC-SL-001", "scenario": "An empty allowlist denies everything", "steps": [{"cmd": "curl"}], "result": "pass"},
            {"testCase": "TC-SL-001", "scenario": "A configured allowlist permits listed commands", "steps": [{"cmd": "curl"}], "result": "pass"}
        ]"#,
    );
    run(&root, &["ingest-results", "--format", "session-log", root.join("session.json").to_str().unwrap()]);

    let (after, _err, _code) = run(&root, &["trace", "REQ-SL-001"]);
    assert!(after.contains("TC-SL-001 [pass]"), "{after}");
}

#[test]
fn a_failing_scenario_marks_the_whole_testcase_fail() {
    let root = new_model();
    seed_req_and_tc(&root);
    write_session_log(
        &root,
        "session.json",
        r#"[
            {"testCase": "TC-SL-001", "scenario": "An empty allowlist denies everything", "steps": [{"cmd": "curl"}], "result": "fail"},
            {"testCase": "TC-SL-001", "scenario": "A configured allowlist permits listed commands", "steps": [{"cmd": "curl"}], "result": "pass"}
        ]"#,
    );
    run(&root, &["ingest-results", "--format", "session-log", root.join("session.json").to_str().unwrap()]);
    let (out, _err, _code) = run(&root, &["trace", "REQ-SL-001"]);
    assert!(out.contains("TC-SL-001 [fail]"), "{out}");
}

#[test]
fn an_incomplete_session_leaves_the_testcase_unknown_not_pass() {
    // Only one of the TestCase's two scenarios has a recorded verdict --
    // mirrors the existing testFunctions rule (every function must pass for
    // the whole TC to read Pass): partial coverage must not read as Pass.
    let root = new_model();
    seed_req_and_tc(&root);
    write_session_log(
        &root,
        "session.json",
        r#"[{"testCase": "TC-SL-001", "scenario": "An empty allowlist denies everything", "steps": [{"cmd": "curl"}], "result": "pass"}]"#,
    );
    run(&root, &["ingest-results", "--format", "session-log", root.join("session.json").to_str().unwrap()]);
    let (out, _err, _code) = run(&root, &["trace", "REQ-SL-001"]);
    assert!(!out.contains("[pass]"), "partial coverage must not read as Pass: {out}");
    assert!(!out.contains("[fail]"), "{out}");
}

#[test]
fn malformed_steps_fails_ingestion_with_a_clear_error_and_does_not_clobber_the_sidecar() {
    let root = new_model();
    seed_req_and_tc(&root);
    // A good ingest first, so we can prove the bad one doesn't overwrite it.
    write_session_log(
        &root,
        "good.json",
        r#"[{"testCase": "TC-SL-001", "scenario": "An empty allowlist denies everything", "steps": [{"cmd": "curl"}], "result": "pass"}]"#,
    );
    run(&root, &["ingest-results", "--format", "session-log", root.join("good.json").to_str().unwrap()]);
    let before = std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap();

    write_session_log(
        &root,
        "bad.json",
        r#"[{"testCase": "TC-SL-001", "scenario": "Missing steps", "steps": [], "result": "pass"}]"#,
    );
    let (_out, stderr, code) = run(&root, &["ingest-results", "--format", "session-log", root.join("bad.json").to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(stderr.contains("steps"), "{stderr}");

    let after = std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap();
    assert_eq!(before, after, "a failed ingest must not clobber the existing sidecar");
}

#[test]
fn unrecognized_result_value_fails_ingestion() {
    let root = new_model();
    seed_req_and_tc(&root);
    write_session_log(
        &root,
        "bad.json",
        r#"[{"testCase": "TC-SL-001", "scenario": "S", "steps": [{"cmd": "x"}], "result": "maybe"}]"#,
    );
    let (_out, stderr, code) = run(&root, &["ingest-results", "--format", "session-log", root.join("bad.json").to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(stderr.contains("maybe"), "{stderr}");
}

#[test]
fn empty_array_fails_ingestion() {
    let root = new_model();
    seed_req_and_tc(&root);
    write_session_log(&root, "empty.json", "[]");
    let (_out, stderr, code) = run(&root, &["ingest-results", "--format", "session-log", root.join("empty.json").to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(stderr.contains("no records"), "{stderr}");
}

#[test]
fn a_testcase_with_test_functions_is_scored_only_by_those_ignoring_session_log_data() {
    // Hand-crafts a single sidecar with BOTH a by_leaf (function) verdict and a
    // by_scenario (session-log) verdict for the same TestCase, so this isolates
    // "does tc_verdict consult by_scenario when testFunctions exist" from
    // ingest-results' own (separate, pre-existing) one-report-at-a-time sidecar
    // semantics -- two sequential `ingest-results` calls each overwrite the
    // whole sidecar, which would conflate the two concerns.
    let root = new_model();
    write(
        &root,
        "REQ-SL-002.md",
        "---\nid: REQ-SL-002\ntype: Requirement\nname: \"Another requirement\"\nstatus: approved\nreqDomain: system\n---\n\nBody.\n",
    );
    write(
        &root,
        "TC-SL-002.md",
        "---\nid: TC-SL-002\ntype: TestCase\ntestLevel: L1\nstatus: active\nname: \"Automated\"\nverifies: [REQ-SL-002]\ntestFunctions:\n  - function: \"crate::tests::it_works\"\n    scenario: \"A scenario\"\n---\n\n```gherkin\nFeature: automated\n  Scenario: A scenario\n    Given x\n    When y\n    Then z\n```\n",
    );
    write(
        &root,
        ".syscribe/results.json",
        r#"{
            "schema_version": "1.0",
            "format": "mixed",
            "source": "test",
            "ingested_at_unix": 0,
            "count": 2,
            "by_leaf": {"it_works": "pass"},
            "by_scenario": {"TC-SL-002::A scenario": "fail"}
        }"#,
    );
    let (out, _err, _code) = run(&root, &["trace", "REQ-SL-002"]);
    assert!(out.contains("TC-SL-002 [pass]"), "a TestCase with testFunctions must be scored only by those, ignoring by_scenario: {out}");
}

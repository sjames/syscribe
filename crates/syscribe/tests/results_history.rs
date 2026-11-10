//! TC-TRS-RUNHIST-001 / GH #258.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn dir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-runhist-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(d.join("model")).unwrap();
    std::fs::write(d.join("model/_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    d
}

fn junit(d: &Path, name: &str, cases: &[(&str, &str)]) -> String {
    let body: String = cases
        .iter()
        .map(|(n, v)| match *v {
            "pass" => format!("<testcase classname=\"C\" name=\"{n}\"/>"),
            "fail" => format!("<testcase classname=\"C\" name=\"{n}\"><failure message=\"x\"/></testcase>"),
            _ => format!("<testcase classname=\"C\" name=\"{n}\"><skipped/></testcase>"),
        })
        .collect();
    let p = d.join(name);
    std::fs::write(&p, format!("<testsuite>{body}</testsuite>")).unwrap();
    p.to_string_lossy().into_owned()
}

fn run(d: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d.join("model")).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn two_runs() -> PathBuf {
    let d = dir();
    let a = junit(&d, "a.xml", &[("t_pass", "pass"), ("t_regress", "pass"), ("t_fixed", "fail"), ("t_still", "fail"), ("t_gone", "pass")]);
    let b = junit(&d, "b.xml", &[("t_pass", "pass"), ("t_regress", "fail"), ("t_fixed", "pass"), ("t_still", "fail"), ("t_new", "fail"), ("t_skip", "skip")]);
    assert_eq!(run(&d, &["ingest-results", "--format", "junit", "--run", "R1", &a]).1, 0);
    assert_eq!(run(&d, &["ingest-results", "--format", "junit", "--run", "R2", &b]).1, 0);
    d
}

#[test]
fn without_a_run_id_no_history_is_written() {
    let d = dir();
    let a = junit(&d, "a.xml", &[("t1", "pass")]);
    assert_eq!(run(&d, &["ingest-results", "--format", "junit", &a]).1, 0);
    assert!(!d.join("model/.syscribe/results-history.json").exists());
    assert!(d.join("model/.syscribe/results.json").exists());
}

#[test]
fn runs_are_listed_and_diffed() {
    let d = two_runs();
    let (o, c) = run(&d, &["results", "runs"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.find("R1").unwrap() < o.find("R2").unwrap(), "{o}");
    let (o, c) = run(&d, &["results", "diff", "R1", "R2", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let ids = |k: &str| -> Vec<String> { v[k].as_array().unwrap().iter().map(|x| x["test"].as_str().unwrap().to_string()).collect() };
    let mut reg = ids("regressions");
    reg.sort();
    assert_eq!(reg, vec!["t_new", "t_regress"], "{o}");
    assert_eq!(ids("fixed"), vec!["t_fixed"], "{o}");
    assert_eq!(ids("stillFailing"), vec!["t_still"], "{o}");
    let mut other = ids("otherChanges");
    other.sort();
    assert_eq!(other, vec!["t_gone", "t_skip"], "{o}");
}

#[test]
fn text_diff_names_the_sections() {
    let d = two_runs();
    let (o, _) = run(&d, &["results", "diff", "R1", "R2"]);
    for s in ["Regressions (2)", "Fixed (1)", "Still failing (1)", "t_regress", "t_fixed"] {
        assert!(o.contains(s), "{s}: {o}");
    }
}

#[test]
fn fail_on_regression_gates_and_unknown_runs_error() {
    let d = two_runs();
    assert_eq!(run(&d, &["results", "diff", "R1", "R2", "--fail-on-regression"]).1, 1);
    assert_eq!(run(&d, &["results", "diff", "R2", "R1", "--fail-on-regression"]).1, 1, "t_gone fails? no: R1 has t_fixed failing");
    assert_eq!(run(&d, &["results", "diff", "R1", "R1", "--fail-on-regression"]).1, 0);
    let (o, c) = run(&d, &["results", "diff", "R1", "NOPE"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("NOPE"), "{o}");
}

#[test]
fn re_ingesting_a_run_replaces_it_and_keeps_the_others() {
    let d = two_runs();
    let a2 = junit(&d, "a2.xml", &[("only", "pass")]);
    assert_eq!(run(&d, &["ingest-results", "--format", "junit", "--run", "R1", &a2]).1, 0);
    let (o, _) = run(&d, &["results", "runs", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let runs = v["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 2, "{o}");
    let r1 = runs.iter().find(|r| r["run"] == "R1").unwrap();
    assert_eq!(r1["functions"], 1, "{o}");
}

#[test]
fn bad_run_arguments_are_usage_errors() {
    let d = dir();
    let a = junit(&d, "a.xml", &[("t1", "pass")]);
    for args in [vec!["--run", "--json", a.as_str()], vec!["--run", "", a.as_str()], vec![a.as_str(), "--run"], vec!["--runs", "R1", a.as_str()]] {
        let mut full = vec!["ingest-results", "--format", "junit"];
        full.extend(args.iter());
        let (o, c) = run(&d, &full);
        assert_eq!(c, 1, "{args:?}: {o}");
    }
    assert!(!d.join("model/.syscribe/results-history.json").exists());
}

#[test]
fn a_vanished_failure_trips_the_gate_and_only_shared_sections_are_compared() {
    let d = dir();
    let a = junit(&d, "a.xml", &[("t_fail", "fail"), ("t_ok", "pass")]);
    let b = junit(&d, "b.xml", &[("t_ok", "pass")]);
    run(&d, &["ingest-results", "--format", "junit", "--run", "R1", &a]);
    run(&d, &["ingest-results", "--format", "junit", "--run", "R2", &b]);
    assert_eq!(run(&d, &["results", "diff", "R1", "R2", "--fail-on-regression"]).1, 1);
    // R3 adds a session-log section R1 lacks: its scenarios are not "new regressions" against R1
    let log = d.join("log.json");
    std::fs::write(&log, r#"[{"testCase":"TC-X-001","scenario":"S","steps":[{"cmd":"x"}],"result":"fail"}]"#).unwrap();
    run(&d, &["ingest-results", "--format", "junit", "--run", "R3", &a]);
    run(&d, &["ingest-results", "--format", "session-log", "--run", "R3", log.to_str().unwrap()]);
    let (o, _) = run(&d, &["results", "diff", "R1", "R3", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    assert!(v["regressions"].as_array().unwrap().is_empty(), "{o}");
}

// ── the history goes through the write layer (GH #469) ──────────────────────────────────────────────────────────

#[test]
fn a_dry_run_writes_no_run_history() {
    let d = dir();
    let a = junit(&d, "a.xml", &[("t_pass", "pass")]);
    let (out, code) = run(&d, &["--dry-run", "ingest-results", "--format", "junit", "--run", "R1", &a]);
    assert_eq!(code, 0, "{out}");
    assert!(!d.join("model/.syscribe/results-history.json").exists(), "a dry run must not write the history: {out}");
    assert!(!d.join("model/.syscribe/results.json").exists(), "nor the sidecar: {out}");
    assert!(out.contains("would write") && out.contains("results-history.json"), "the history is reported as declined: {out}");
}

#[test]
fn a_protected_history_is_refused_before_the_sidecar_is_written() {
    let d = dir();
    // Only the history is protected, not the sidecar: the case the sidecar's own refusal could not catch.
    std::fs::write(d.join("model/.syscribe.toml"), "[agents]\nprotected_paths = [\".syscribe/results-history.json\"]\n").unwrap();
    let a = junit(&d, "a.xml", &[("t_pass", "pass")]);
    let (out, code) = run(&d, &["ingest-results", "--format", "junit", "--run", "R1", "--by", "agent-1", &a]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("protected_paths") && out.contains("results-history.json"), "the rule and the file are named: {out}");
    assert!(!d.join("model/.syscribe/results.json").exists(), "nothing was written, so the sidecar and history stay in step: {out}");
    assert!(!d.join("model/.syscribe/results-history.json").exists(), "{out}");
    // A person, or a run without --run, is unaffected.
    assert_eq!(run(&d, &["ingest-results", "--format", "junit", "--run", "R1", "--by", "alice", "--as-human", &a]).1, 0);
    assert!(d.join("model/.syscribe/results-history.json").exists());
    assert_eq!(run(&d, &["ingest-results", "--format", "junit", "--by", "agent-1", &a]).1, 0, "no --run, no history write");
}

// ── records retained under a run (GH #471) ──────────────────────────────────────────────────────────────────────

fn history(d: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(d.join("model/.syscribe/results-history.json")).unwrap()).unwrap()
}

fn run_with_a_test_case() -> PathBuf {
    let d = dir();
    std::fs::write(d.join("model/REQ-RH-001.md"), "---\nid: REQ-RH-001\ntype: Requirement\nname: R\nstatus: approved\nreqDomain: system\n---\n\nBody.\n").unwrap();
    std::fs::write(d.join("model/TC-RH-001.md"), "---\nid: TC-RH-001\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: T\nverifies: [REQ-RH-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: One\n    Given x\n    Then y\n```\n").unwrap();
    d
}

fn records(d: &Path, name: &str, value: &str) -> String {
    let p = d.join(name);
    std::fs::write(&p, format!("[{{\"testCase\": \"TC-RH-001\", \"scenario\": \"One\", \"verdict\": \"pass\", \"measurements\": [{{\"name\": \"closeTime\", \"value\": \"{value}\"}}]}}]")).unwrap();
    p.to_string_lossy().into_owned()
}

#[test]
fn a_records_run_retains_its_measurements_and_a_later_ingest_of_the_same_run_supersedes_them() {
    let d = run_with_a_test_case();
    let a = records(&d, "a.json", "1650 ms");
    assert_eq!(run(&d, &["ingest-results", "--format", "records", "--run", "R1", &a]).1, 0);
    let h = history(&d);
    let r = &h["runs"][0]["records"];
    assert_eq!((r.as_array().unwrap().len(), r[0]["testCase"].clone(), r[0]["measurements"][0]["value"].clone()), (1, "TC-RH-001".into(), "1650 ms".into()), "{h}");
    assert!(r[0]["tcHash"].is_string(), "the retained record keeps the hash it was stamped with: {r}");
    // A later ingest of the same run and key replaces the record; the sidecar and the history agree.
    let b = records(&d, "b.json", "1700 ms");
    assert_eq!(run(&d, &["ingest-results", "--format", "records", "--run", "R1", &b]).1, 0);
    let r = history(&d)["runs"][0]["records"].clone();
    assert_eq!((r.as_array().unwrap().len(), r[0]["measurements"][0]["value"].clone()), (1, "1700 ms".into()), "{r}");
}

#[test]
fn function_level_runs_retain_no_records_and_a_history_without_them_keeps_its_shape() {
    let d = dir();
    let a = junit(&d, "a.xml", &[("t_pass", "pass")]);
    assert_eq!(run(&d, &["ingest-results", "--format", "junit", "--run", "R1", &a]).1, 0);
    let h = history(&d);
    assert!(h["runs"][0].get("records").is_none(), "no records key for a verdict-only run: {h}");
    // A history file written before records were retained still loads and is listed.
    let (out, code) = run(&d, &["results", "runs"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("R1"), "{out}");
}

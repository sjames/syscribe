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

// ── records retained under a run, one file per run (GH #471) ────────────────────────────────────────────────────

fn history(d: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(d.join("model/.syscribe/results-history.json")).unwrap()).unwrap()
}

fn run_file(d: &Path, name: &str) -> PathBuf {
    d.join("model/.syscribe/runs").join(format!("{name}.json"))
}

fn run_records(d: &Path, name: &str) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(run_file(d, name)).unwrap()).unwrap()
}

fn run_with_a_test_case() -> PathBuf {
    let d = dir();
    std::fs::write(d.join("model/REQ-RH-001.md"), "---\nid: REQ-RH-001\ntype: Requirement\nname: R\nstatus: approved\nreqDomain: system\n---\n\nBody.\n").unwrap();
    std::fs::write(d.join("model/TC-RH-001.md"), "---\nid: TC-RH-001\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: T\nverifies: [REQ-RH-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: One\n    Given x\n    Then y\n```\n").unwrap();
    d
}

/// A records file with everything a record can carry, so what the history drops is visible.
fn records(d: &Path, name: &str, value: &str) -> String {
    let p = d.join(name);
    std::fs::write(
        &p,
        format!(
            "[{{\"testCase\": \"TC-RH-001\", \"scenario\": \"One\", \"verdict\": \"pass\", \"steps\": [{{\"text\": \"a step\"}}],\
             \"measurements\": [{{\"name\": \"closeTime\", \"value\": \"{value}\"}}],\
             \"provenance\": {{\"operator\": \"alice\", \"tool\": \"rig 1\", \"build\": {{\"commit\": \"abc\"}}, \"startedAt\": \"2026-11-01T10:00:00Z\", \"finishedAt\": \"2026-11-01T10:05:00Z\", \
             \"artefacts\": [{{\"uri\": \"file:///x\", \"hash\": \"sha256:{}\"}}]}}}}]",
            "0".repeat(64)
        ),
    )
    .unwrap();
    p.to_string_lossy().into_owned()
}

fn ingest_run(d: &Path, run: &str, value: &str) -> (String, i32) {
    let f = records(d, &format!("{}-{value}.json", run.replace(['/', ' '], "_")), value);
    run_cli(d, &["ingest-results", "--format", "records", "--run", run, &f])
}

fn run_cli(d: &Path, args: &[&str]) -> (String, i32) {
    run(d, args)
}

#[test]
fn a_records_run_keeps_slim_records_in_its_own_file_and_the_index_counts_them() {
    let d = run_with_a_test_case();
    let (out, code) = ingest_run(&d, "R1", "1650 ms");
    assert_eq!(code, 0, "{out}");
    // The index holds counts and verdicts, never the records.
    let h = history(&d);
    assert_eq!(h["runs"][0]["record_count"], 1, "{h}");
    assert!(h["runs"][0].get("records").is_none(), "no records in the index: {h}");
    // The run's file holds the slim record: the measurements, hashes and finish time, not the steps, artefacts or operator.
    let f = run_records(&d, "R1");
    let r = &f["records"][0];
    assert_eq!((f["run"].clone(), r["measurements"][0]["value"].clone(), r["provenance"]["finishedAt"].clone()), ("R1".into(), "1650 ms".into(), "2026-11-01T10:05:00Z".into()), "{f}");
    assert!(r["tcHash"].is_string() && r["reqHashes"]["REQ-RH-001"].is_string(), "the hashes the record was stamped with are kept: {r}");
    for gone in ["steps"] {
        assert!(r.get(gone).is_none(), "{gone} is not kept: {r}");
    }
    for gone in ["operator", "tool", "build", "artefacts"] {
        assert!(r["provenance"].get(gone).is_none(), "provenance.{gone} is not kept: {r}");
    }
    // The sidecar keeps the full record.
    let sidecar: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(d.join("model/.syscribe/results.json")).unwrap()).unwrap();
    assert!(sidecar["records"][0]["steps"].as_array().is_some_and(|s| !s.is_empty()), "{sidecar}");
}

#[test]
fn a_later_ingest_of_the_same_run_supersedes_its_keys_and_other_runs_have_their_own_files() {
    let d = run_with_a_test_case();
    assert_eq!(ingest_run(&d, "R1", "1650 ms").1, 0);
    let r1_before = std::fs::read_to_string(run_file(&d, "R1")).unwrap();
    assert_eq!(ingest_run(&d, "R2", "1800 ms").1, 0);
    assert_eq!(std::fs::read_to_string(run_file(&d, "R1")).unwrap(), r1_before, "another run's ingest does not touch R1's file");
    // The same run and key again replaces the record: one record, the new value; the index agrees.
    assert_eq!(ingest_run(&d, "R1", "1700 ms").1, 0);
    let f = run_records(&d, "R1");
    assert_eq!((f["records"].as_array().unwrap().len(), f["records"][0]["measurements"][0]["value"].clone()), (1, "1700 ms".into()), "{f}");
    assert_eq!(history(&d)["runs"][0]["record_count"], 1);
}

#[test]
fn function_level_runs_write_no_run_file_and_keep_the_indexs_shape() {
    let d = dir();
    let a = junit(&d, "a.xml", &[("t_pass", "pass")]);
    assert_eq!(run(&d, &["ingest-results", "--format", "junit", "--run", "R1", &a]).1, 0);
    assert!(history(&d)["runs"][0].get("record_count").is_none(), "no count for a verdict-only run");
    assert!(!d.join("model/.syscribe/runs").exists(), "no run file, so no directory");
    let (out, code) = run(&d, &["results", "runs"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("R1") && !out.contains("record(s)"), "{out}");
}

fn with_keep(d: &Path, n: &str) {
    std::fs::write(d.join("model/.syscribe.toml"), format!("[results]\nkeep_run_records = {n}\n")).unwrap();
}

#[test]
fn the_cap_drops_the_oldest_runs_records_and_keeps_their_verdicts_and_says_so() {
    let d = run_with_a_test_case();
    with_keep(&d, "2");
    assert_eq!(ingest_run(&d, "R1", "1600 ms").1, 0);
    assert_eq!(ingest_run(&d, "R2", "1700 ms").1, 0);
    let (out, code) = ingest_run(&d, "R3", "1800 ms");
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("Dropped the records of run R1") && out.contains("keeping the last 2") && out.contains("their verdicts are kept"), "{out}");
    assert!(!run_file(&d, "R1").exists() && run_file(&d, "R2").exists() && run_file(&d, "R3").exists(), "the oldest run's file is deleted");
    let h = history(&d);
    assert_eq!((h["runs"][0]["record_count"].clone(), h["runs"][0]["records_dropped"].clone()), (serde_json::Value::Null, true.into()), "{h}");
    // The verdicts stay, so the runs still compare.
    let (out, code) = run(&d, &["results", "diff", "R1", "R2"]);
    assert_eq!(code, 0, "{out}");
    // `results runs` shows what each run holds.
    let (out, _) = run(&d, &["results", "runs"]);
    let line = |id: &str| out.lines().find(|l| l.starts_with(id)).unwrap().to_string();
    assert!(line("R1").contains("records dropped") && line("R2").contains("1 record(s)") && line("R3").contains("1 record(s)"), "{out}");
    let (j, _) = run(&d, &["results", "runs", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!((v["runs"][0]["recordsDropped"].clone(), v["runs"][2]["records"].clone()), (true.into(), 1.into()), "{v}");
}

#[test]
fn a_run_re_ingested_after_a_newer_one_counts_as_newest_even_within_the_same_second() {
    let d = run_with_a_test_case();
    with_keep(&d, "2");
    for (r, v) in [("R1", "1"), ("R2", "2"), ("R3", "3")] {
        assert_eq!(ingest_run(&d, r, &format!("{v}00 ms")).1, 0);
    }
    // R1's records were dropped. Re-ingesting it now keeps them; the oldest of the others (R2) goes.
    let (out, code) = ingest_run(&d, "R1", "999 ms");
    assert_eq!(code, 0, "{out}");
    assert!(run_file(&d, "R1").exists() && run_file(&d, "R3").exists() && !run_file(&d, "R2").exists(), "R1 is newest: {out}");
    assert!(out.contains("Dropped the records of run R2"), "{out}");
}

#[test]
fn keeping_none_and_a_malformed_setting() {
    let d = run_with_a_test_case();
    with_keep(&d, "0");
    let (out, code) = ingest_run(&d, "R1", "1600 ms");
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("no run keeps records") && !run_file(&d, "R1").exists(), "{out}");
    assert_eq!(history(&d)["runs"][0]["records_dropped"], true);
    // A malformed value is a warning and the default applies: nothing is dropped.
    let d = run_with_a_test_case();
    with_keep(&d, "\"many\"");
    let (out, code) = ingest_run(&d, "R1", "1600 ms");
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("keep_run_records must be a whole number") && run_file(&d, "R1").exists(), "{out}");
}

#[test]
fn an_unreadable_run_file_is_never_overwritten() {
    let d = run_with_a_test_case();
    assert_eq!(ingest_run(&d, "R1", "1600 ms").1, 0);
    std::fs::write(run_file(&d, "R1"), "{ not json").unwrap();
    let (out, code) = ingest_run(&d, "R1", "1700 ms");
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("is not valid") && out.contains("fix or remove"), "{out}");
    assert_eq!(std::fs::read_to_string(run_file(&d, "R1")).unwrap(), "{ not json", "left for a person to fix");
}

#[test]
fn a_run_id_becomes_a_safe_unique_file_name() {
    let d = run_with_a_test_case();
    assert_eq!(ingest_run(&d, "rc/1 2", "1600 ms").1, 0);
    assert!(run_file(&d, "rc%2F1%202").exists(), "{:?}", std::fs::read_dir(d.join("model/.syscribe/runs")).unwrap().map(|e| e.unwrap().file_name()).collect::<Vec<_>>());
    assert_eq!(history(&d)["runs"][0]["run"], "rc/1 2", "the index keeps the id as given");
}

#[test]
fn a_protected_run_file_is_refused_before_anything_is_written() {
    let d = run_with_a_test_case();
    std::fs::write(d.join("model/.syscribe.toml"), "[agents]\nprotected_paths = [\".syscribe/runs/**\"]\n").unwrap();
    let f = records(&d, "a.json", "1600 ms");
    let (out, code) = run(&d, &["ingest-results", "--format", "records", "--run", "R1", "--by", "agent-1", &f]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("protected_paths") && out.contains("runs"), "{out}");
    assert!(!d.join("model/.syscribe/results.json").exists() && !d.join("model/.syscribe/results-history.json").exists(), "nothing was written: {out}");
}

#[test]
fn a_moved_runs_dir_holds_the_run_files_and_the_index_stays_in_syscribe() {
    let d = run_with_a_test_case();
    // `../evidence/runs` is relative to the model root: <dir>/evidence/runs, outside the model.
    std::fs::write(d.join("model/.syscribe.toml"), "[results]\nruns_dir = \"../evidence/runs\"\n").unwrap();
    let (out, code) = ingest_run(&d, "R1", "1600 ms");
    assert_eq!(code, 0, "{out}");
    assert!(d.join("evidence/runs/R1.json").is_file(), "the run file is where runs_dir says: {out}");
    assert!(!d.join("model/.syscribe/runs").exists(), "and not in the default place");
    assert_eq!(history(&d)["runs"][0]["record_count"], 1, "the index is still in .syscribe");
    let (list, code) = run(&d, &["results", "runs"]);
    assert_eq!(code, 0, "{list}");
    assert!(list.contains("R1") && list.contains("1 record(s)"), "{list}");
    // A cap drop deletes the file from the moved directory.
    std::fs::write(d.join("model/.syscribe.toml"), "[results]\nruns_dir = \"../evidence/runs\"\nkeep_run_records = 1\n").unwrap();
    let (out, code) = ingest_run(&d, "R2", "1700 ms");
    assert_eq!(code, 0, "{out}");
    assert!(!d.join("evidence/runs/R1.json").exists() && d.join("evidence/runs/R2.json").is_file(), "{out}");
}

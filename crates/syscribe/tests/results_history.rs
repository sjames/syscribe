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

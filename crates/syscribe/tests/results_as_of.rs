//! TC-TRS-RESASOF-001 / GH #258.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-asof-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let m = d.join("model");
    let w = |rel: &str, c: &str| {
        let p = m.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("R/REQ-AO-001.md", "---\nid: REQ-AO-001\ntype: Requirement\nname: r\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\nShall.\n");
    w("R/TC-AO-001.md", "---\nid: TC-AO-001\ntype: TestCase\nname: t\nstatus: active\ntestLevel: L3\nverifies: [REQ-AO-001]\ntestFunctions:\n  - function: t_one\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    let junit = |name: &str, verdict: &str| {
        let body = if verdict == "fail" { "<failure message=\"x\"/>" } else { "" };
        let p = d.join(name);
        std::fs::write(&p, format!("<testsuite><testcase classname=\"C\" name=\"t_one\">{body}</testcase></testsuite>")).unwrap();
        p
    };
    let run = |args: &[&str]| {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&m).args(args).output().unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    };
    run(&["ingest-results", "--format", "junit", "--run", "R1", junit("a.xml", "pass").to_str().unwrap()]);
    run(&["ingest-results", "--format", "junit", "--run", "R2", junit("b.xml", "fail").to_str().unwrap()]);
    m
}

fn run(m: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(m).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

#[test]
fn a_retained_run_is_evaluated_instead_of_the_latest_sidecar() {
    let m = model();
    // the latest sidecar is R2 (fail): W010 fires
    assert!(run(&m, &["validate"]).0.contains("W010"));
    // as of R1 (pass) it does not — in either spelling and position
    assert!(!run(&m, &["validate", "--results-as-of", "R1"]).0.contains("W010"));
    for args in [vec!["--results-as-of=R1", "validate"], vec!["--results-as-of", "R1", "validate"]] {
        let (o, c) = run(&m, &args);
        assert_eq!(c, 0, "{args:?}: {o}");
        assert!(!o.contains("W010") && o.contains("errors"), "{args:?}: {o}");
    }
    // `results failures` reads the same lens: R1 had no failure
    let (f, _) = run(&m, &["results", "failures", "--results-as-of", "R1"]);
    assert!(!f.contains("t_one"), "{f}");
    // mcp / lsp / ingest-results refuse the flag
    assert_eq!(run(&m, &["ingest-results", "--results-as-of", "R1"]).1, 1);
    assert_eq!(run(&m, &["lsp", "--results-as-of", "R1"]).1, 1);
    assert_eq!(run(&m, &["mcp", "--results-as-of", "R1"]).1, 1);
    // and as of R2 it does
    assert!(run(&m, &["validate", "--results-as-of", "R2"]).0.contains("W010"));
    // the verdicts show up in a results-reading view
    let (t, c) = run(&m, &["trace", "REQ-AO-001", "--results-as-of", "R1"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("pass"), "{t}");
    // the sidecar itself is untouched
    assert!(run(&m, &["validate"]).0.contains("W010"));
}

#[test]
fn an_unknown_run_is_an_error_naming_the_retained_ones() {
    let m = model();
    let (o, c) = run(&m, &["validate", "--results-as-of", "NOPE"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("NOPE") && o.contains("R1") && o.contains("R2"), "{o}");
    assert_eq!(run(&m, &["validate", "--results-as-of"]).1, 1);
}

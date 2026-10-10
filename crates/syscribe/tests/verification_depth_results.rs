//! TC-TRS-VRES-002 / GH #257 (b, c).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(with_results: bool) -> PathBuf {
    model_with(with_results, &[])
}

/// Extra `(id, level, testFunctions-yaml-or-empty)` TestCases on top of the defaults.
fn model_with(with_results: bool, extra: &[(&str, &str, &str)]) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir()
        .join(format!("syscribe-vdr-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)))
        .join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("REQ-VD-001.md", "---\ntype: Requirement\nid: REQ-VD-001\nname: r\nstatus: approved\nreqDomain: software\nsilLevel: 2\n---\n\nThe system shall.\n");
    let tc = |id: &str, lvl: &str, func: Option<&str>| {
        let tf = func.map(|f| format!("testFunctions:\n  - function: \"{f}\"\n")).unwrap_or_default();
        format!("---\ntype: TestCase\nid: {id}\nname: t\nstatus: active\ntestLevel: {lvl}\nverifies: [REQ-VD-001]\n{tf}---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n")
    };
    w("TC/TC-VD-001.md", &tc("TC-VD-001", "L3", Some("fn_pass")));
    w("TC/TC-VD-002.md", &tc("TC-VD-002", "L4", Some("fn_fail")));
    w("TC/TC-VD-003.md", &tc("TC-VD-003", "L5", Some("fn_absent")));
    w("TC/TC-VD-004.md", &tc("TC-VD-004", "L2", None));
    for (id, lvl, tf) in extra {
        w(
            &format!("TC/{id}.md"),
            &format!("---\ntype: TestCase\nid: {id}\nname: t\nstatus: active\ntestLevel: {lvl}\nverifies: [REQ-VD-001]\n{tf}---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n"),
        );
    }
    if with_results {
        w(
            ".syscribe/results.json",
            r#"{"schema_version":"1.0","format":"junit","source":"x","ingested_at_unix":1,"count":2,"by_leaf":{"fn_pass":"pass","fn_fail":"fail","fn_skip":"ignored"}}"#,
        );
    }
    r
}

fn depth(root: &Path) -> serde_json::Value {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .args(["verification-depth", "--json"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&o.stdout);
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("{e}: {s}"))
}

#[test]
fn failing_and_unrun_tests_do_not_count_and_are_listed() {
    let v = depth(&model(true));
    let row = &v[0];
    // L3 passes, L4 fails, L5 not run, L2 is manual (no functions) and still counts.
    assert_eq!(row["levels"], serde_json::json!(["L2", "L3"]), "{row}");
    assert_eq!(row["failing"], serde_json::json!(["TC-VD-002"]), "{row}");
    assert_eq!(row["notRun"], serde_json::json!(["TC-VD-003"]), "{row}");
}

#[test]
fn without_results_everything_counts_as_before() {
    let v = depth(&model(false));
    let row = &v[0];
    assert_eq!(row["levels"], serde_json::json!(["L2", "L3", "L4", "L5"]), "{row}");
    assert_eq!(row["failing"], serde_json::json!([]), "{row}");
    assert_eq!(row["notRun"], serde_json::json!([]), "{row}");
}

#[test]
fn text_report_shows_the_failing_column() {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(model(true))
        .arg("verification-depth")
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&o.stdout);
    assert!(s.contains("Failing") && s.contains("TC-VD-002"), "{s}");
}

#[test]
fn a_skipped_function_is_not_run() {
    let r = model_with(true, &[("TC-VD-005", "L1", "testFunctions:\n  - function: \"fn_skip\"\n")]);
    let v = depth(&r);
    assert!(v[0]["notRun"].as_array().unwrap().contains(&serde_json::json!("TC-VD-005")), "{}", v[0]);
    assert!(!v[0]["levels"].as_array().unwrap().contains(&serde_json::json!("L1")), "{}", v[0]);
}

#[test]
fn test_functions_without_a_function_key_are_not_treated_as_automated() {
    // No scoreable function: stays counted like a manual test.
    let r = model_with(true, &[("TC-VD-006", "L1", "testFunctions:\n  - scenario: \"only a scenario\"\n")]);
    let v = depth(&r);
    assert!(v[0]["levels"].as_array().unwrap().contains(&serde_json::json!("L1")), "{}", v[0]);
    assert!(!v[0]["notRun"].as_array().unwrap().contains(&serde_json::json!("TC-VD-006")), "{}", v[0]);
}

#[test]
fn min_levels_gate_uses_the_counted_levels() {
    let root = model(true);
    let run = |n: &str| {
        Command::new(env!("CARGO_BIN_EXE_syscribe"))
            .arg("-m")
            .arg(&root)
            .args(["verification-depth", "--min-levels", n])
            .output()
            .unwrap()
            .status
            .code()
    };
    assert_eq!(run("2"), Some(0), "L2 + L3 count");
    assert_eq!(run("3"), Some(2), "failing L4 and unrun L5 must not satisfy a 3-level gate");
}

#[test]
fn only_failing_tests_leave_the_requirement_unverified() {
    let r = model(true);
    for id in ["TC-VD-001", "TC-VD-003", "TC-VD-004"] {
        std::fs::remove_file(r.join(format!("TC/{id}.md"))).unwrap();
    }
    let v = depth(&r);
    assert_eq!(v[0]["flag"], "none", "{}", v[0]);
    assert_eq!(v[0]["failing"], serde_json::json!(["TC-VD-002"]), "{}", v[0]);
}

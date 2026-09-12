//! Regression test for the `validate` text-mode leading summary line
//! (issue #116): "did this pass" must be answerable from the first line of
//! output, not from the absence of an `Errors (N):` section, and the summary
//! must appear before the per-severity tables and reflect any gating flag
//! that changed the effective pass/fail verdict.
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
        .join(format!("syscribe-w116-{}-{}-{}", std::process::id(), nanos, n))
        .join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: SummaryLine\n---\n\nModel root.\n");
    root
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn syscribe");
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.code().unwrap_or(-1))
}

/// A `ViewDef` with `svgMode: companion` but no `<img` tag reliably fires
/// W405 (and only W405 — no error, no gating) in an otherwise-empty model.
fn write_warning_source(root: &Path) {
    write(
        root,
        "Diag.md",
        "---\ntype: ViewDef\nname: Diag\nsvgMode: companion\n---\n\nNo img tag here on purpose.\n",
    );
    // Avoid the unrelated E402 (missing companion SVG on disk) so this model
    // produces exactly one warning (W405) and zero errors.
    write(root, "Diag.svg", "");
}

#[test]
fn clean_model_prints_explicit_zero_zero_line() {
    let root = new_model();
    let (stdout, code) = run(&root, &["validate"]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "0 errors, 0 warnings — model is valid.");
}

#[test]
fn warnings_only_run_has_summary_before_the_warnings_table_and_no_fail_suffix() {
    let root = new_model();
    write_warning_source(&root);
    let (stdout, code) = run(&root, &["validate"]);
    assert_eq!(code, 0, "ungated warnings must not fail the run:\n{stdout}");

    let summary_pos = stdout.find("errors,").expect("summary line present");
    let table_pos = stdout.find("Warnings (").expect("Warnings (N): table present");
    assert!(summary_pos < table_pos, "summary line must come before the Warnings table:\n{stdout}");

    let first_line = stdout.lines().next().unwrap();
    assert!(first_line.starts_with("0 errors, "), "got: {first_line}");
    assert!(first_line.ends_with("warnings"), "ungated run must have no gating suffix: {first_line}");
    assert!(!first_line.contains("FAIL"), "ungated warnings-only run must not say FAIL: {first_line}");
}

#[test]
fn deny_gate_trip_is_attributed_and_fails() {
    let root = new_model();
    write_warning_source(&root);
    let (stdout, code) = run(&root, &["validate", "--deny", "W405"]);
    assert_eq!(code, 2);
    let first_line = stdout.lines().next().unwrap();
    assert!(first_line.contains("1 gated by --deny W405"), "got: {first_line}");
    assert!(first_line.ends_with("— FAIL"), "got: {first_line}");
}

#[test]
fn max_warnings_gate_trip_is_attributed_and_fails() {
    let root = new_model();
    write_warning_source(&root);
    let (stdout, code) = run(&root, &["validate", "--max-warnings", "0"]);
    assert_eq!(code, 2);
    let first_line = stdout.lines().next().unwrap();
    assert!(first_line.contains("exceeds --max-warnings 0"), "got: {first_line}");
    assert!(first_line.ends_with("— FAIL"), "got: {first_line}");
}

#[test]
fn warnings_as_errors_gate_trip_is_attributed_and_fails() {
    let root = new_model();
    write_warning_source(&root);
    let (stdout, code) = run(&root, &["validate", "--warnings-as-errors"]);
    assert_eq!(code, 2);
    let first_line = stdout.lines().next().unwrap();
    assert!(first_line.contains("all warnings promoted to errors (--warnings-as-errors)"), "got: {first_line}");
    assert!(first_line.ends_with("— FAIL"), "got: {first_line}");
}

#[test]
fn profile_gate_trip_is_attributed_distinctly_from_deny() {
    let root = new_model();
    write_warning_source(&root);
    write(&root, ".syscribe.toml", "[profiles.strict]\npromote = [\"W405\"]\n");
    let (stdout, code) = run(&root, &["validate", "--profile", "strict"]);
    assert_eq!(code, 2, "{stdout}");
    let first_line = stdout.lines().next().unwrap();
    assert!(first_line.contains("1 gated by --profile"), "got: {first_line}");
    assert!(!first_line.contains("--deny"), "profile-only gating must not blame --deny: {first_line}");
    assert!(first_line.ends_with("— FAIL"), "got: {first_line}");
}

#[test]
fn json_output_is_unchanged_by_the_summary_line() {
    let root = new_model();
    write_warning_source(&root);
    let (stdout, code) = run(&root, &["validate", "--json"]);
    assert_eq!(code, 0);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON, no leading summary text");
    assert!(parsed.is_array());
}

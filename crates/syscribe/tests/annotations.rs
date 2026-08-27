//! Black-box CLI harness for `annotations scan <qname-or-label> --dry-run`
//! (`ADR-SYS-ANNOTATE-001`). Drives the real `syscribe` binary, mirroring
//! `plugins.rs`'s harness shape for the sibling mechanism.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-annotations-cli-test-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn run(model: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(model)
        .args(args)
        .output()
        .expect("spawn syscribe");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn new_model_with_marker() -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Firmware/_index.md",
        "---\ntype: Package\nname: Firmware\nannotationFormat: c-linecomment\nmarker: '//\\s*@syscribe\\b'\ninclude: [\"**/*.c\"]\n---\n",
    );
    write(
        &root,
        "Firmware/engine.c",
        "// @syscribe\n// type: Part\n// name: EngineController\n// doc: x\n",
    );
    root
}

#[test]
fn scan_dry_run_by_qname_prints_json_report_and_exits_zero() {
    let model = new_model_with_marker();
    let (stdout, stderr, code) = run(&model, &["annotations", "scan", "Firmware", "--dry-run"]);
    assert_eq!(code, 0, "stdout: {stdout}\nstderr: {stderr}");
    assert!(stdout.contains("\"Firmware::EngineController\""), "expected synthesized qname in stdout: {stdout}");
}

#[test]
fn scan_dry_run_by_label_matches_the_same_package() {
    let model = new_model_with_marker();
    let (stdout, stderr, code) = run(&model, &["annotations", "scan", "c-linecomment", "--dry-run"]);
    assert_eq!(code, 0, "stdout: {stdout}\nstderr: {stderr}");
    assert!(stdout.contains("\"Firmware::EngineController\""), "expected synthesized qname in stdout: {stdout}");
}

#[test]
fn scan_without_dry_run_flag_is_a_usage_error() {
    let model = new_model_with_marker();
    let (_stdout, stderr, code) = run(&model, &["annotations", "scan", "Firmware"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Usage"), "stderr: {stderr}");
}

#[test]
fn scan_unknown_selector_fails_clearly() {
    let model = new_model_with_marker();
    let (_stdout, stderr, code) = run(&model, &["annotations", "scan", "NotAPackage", "--dry-run"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("no package"), "stderr: {stderr}");
}

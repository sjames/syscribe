//! Black-box CLI harness for `plugins run <alias> --dry-run`
//! (`ADR-SYS-PLUGIN-002`). Drives the real `syscribe` binary, mirroring
//! `baseline.rs`'s harness shape for a simpler, non-git-anchored feature.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-plugins-cli-test-{}-{}",
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

fn new_model_with_plugin(plugin_shell_cmd: &str) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Legacy/_index.md",
        "---\ntype: Package\nname: Legacy\nforeignFormat: toydsl\n---\n",
    );
    write(
        &root,
        ".syscribe.toml",
        &format!("[plugins.toydsl]\ncommand = \"/bin/sh\"\nargs = [\"-c\", \"{plugin_shell_cmd}\"]\n"),
    );
    root
}

#[test]
fn run_dry_run_prints_raw_envelope_and_exits_zero() {
    let model = new_model_with_plugin(
        "cat >/dev/null; echo '{\\\"elements\\\":[{\\\"qname\\\":\\\"Widget\\\",\\\"type\\\":\\\"PartDef\\\"}]}'",
    );
    let (stdout, _stderr, code) = run(&model, &["plugins", "run", "toydsl", "--dry-run"]);
    assert_eq!(code, 0, "stdout: {stdout}");
    assert!(stdout.contains("\"Widget\""), "expected raw envelope in stdout: {stdout}");
}

#[test]
fn run_without_dry_run_flag_is_a_usage_error() {
    let model = new_model_with_plugin("echo '{}'");
    let (_stdout, stderr, code) = run(&model, &["plugins", "run", "toydsl"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Usage"), "stderr: {stderr}");
}

#[test]
fn run_missing_alias_entry_fails_clearly() {
    let model = new_model_with_plugin("echo '{}'");
    let (_stdout, stderr, code) = run(&model, &["plugins", "run", "not-configured", "--dry-run"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("no [plugins.not-configured] entry"), "stderr: {stderr}");
}

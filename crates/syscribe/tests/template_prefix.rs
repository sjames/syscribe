//! Realises TC-TRS-TMPL-001 (GH #246): `template Requirement` honours `[ids.prefixes]`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn model(toml: Option<&str>) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir()
        .join(format!("syscribe-tmplprefix-{}-{}", std::process::id(), n))
        .join("model");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    if let Some(t) = toml {
        std::fs::write(root.join(".syscribe.toml"), t).unwrap();
    }
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .arg("template")
        .args(args)
        .output()
        .unwrap()
}

fn id_line(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).lines().find(|l| l.starts_with("id:")).unwrap_or("").to_string()
}

const CFG: &str = "[ids.prefixes]\nRequirement = [\"STK\", \"SYS\"]\n";

#[test]
fn default_prefix_is_first_configured() {
    let m = model(Some(CFG));
    let o = run(&m, &["Requirement"]);
    assert!(o.status.success());
    assert!(id_line(&o).starts_with("id: STK-"), "{}", id_line(&o));
}

#[test]
fn explicit_prefix_is_used() {
    let m = model(Some(CFG));
    let o = run(&m, &["Requirement", "--prefix", "SYS"]);
    assert!(o.status.success());
    assert!(id_line(&o).starts_with("id: SYS-"), "{}", id_line(&o));
}

#[test]
fn unknown_prefix_is_rejected_and_valid_ones_listed() {
    let m = model(Some(CFG));
    let o = run(&m, &["Requirement", "--prefix", "NOPE"]);
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr);
    for p in ["REQ", "STK", "SYS"] {
        assert!(err.contains(p), "stderr should list {p}: {err}");
    }
}

#[test]
fn unconfigured_model_keeps_builtin_prefix() {
    let m = model(None);
    let o = run(&m, &["Requirement"]);
    assert!(o.status.success());
    assert!(id_line(&o).starts_with("id: REQ-"), "{}", id_line(&o));
}

#[test]
fn malformed_configured_prefix_is_never_offered() {
    let m = model(Some("[ids.prefixes]\nRequirement = [\"stk\", \"SYS\"]\n"));
    let o = run(&m, &["Requirement"]);
    assert!(id_line(&o).starts_with("id: SYS-"), "{}", id_line(&o));
    let o = run(&m, &["Requirement", "--prefix", "stk"]);
    assert!(!o.status.success());
}

#[test]
fn prefix_without_value_or_unknown_flag_is_an_error() {
    let m = model(Some(CFG));
    assert!(!run(&m, &["Requirement", "--prefix"]).status.success());
    assert!(!run(&m, &["Requirement", "--prefx", "SYS"]).status.success());
}

#[test]
fn prefix_may_precede_the_type() {
    let m = model(Some(CFG));
    let o = run(&m, &["--prefix", "SYS", "Requirement"]);
    assert!(o.status.success());
    assert!(id_line(&o).starts_with("id: SYS-"), "{}", id_line(&o));
}

#[test]
fn prefix_on_another_type_is_rejected() {
    let m = model(Some(CFG));
    assert!(!run(&m, &["TestCase", "--prefix", "SYS"]).status.success());
}

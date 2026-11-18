//! GH #496 (`REQ-TRS-MINVER-001`): `[syscribe] minVersion` in `.syscribe.toml`.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

const RUNNING: &str = env!("CARGO_PKG_VERSION");

struct M {
    root: PathBuf,
}

impl M {
    fn new(toml: &str) -> M {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-mv-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
        std::fs::create_dir_all(&root).unwrap();
        let m = M { root };
        m.w("_index.md", "---\ntype: Package\nname: Root\n---\n");
        m.w("Features/Disk.md", "---\ntype: FeatureDef\nid: FEAT-DISK\nname: Disk\nmandatory: true\nparameters:\n  - { name: slotMiB, type: ScalarValues::Integer, default: 1000 }\n  - { name: superMiB, type: ScalarValues::Integer, derivedFrom: \"Features::Disk.slotMiB * 2\" }\n---\n");
        if !toml.is_empty() {
            m.w(".syscribe.toml", toml);
        }
        m
    }
    fn w(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn run(&self, args: &[&str]) -> (String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&self.root).args(args).stdin(Stdio::null()).output().unwrap();
        (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
    }
}

fn newer() -> String {
    let major: u64 = RUNNING.split('.').next().unwrap().parse().unwrap();
    format!("{}.0.0", major + 1)
}

#[test]
fn an_older_syscribe_refuses_the_model_naming_both_versions() {
    for form in [newer(), format!(">={}", newer())] {
        let m = M::new(&format!("[syscribe]\nminVersion = \"{form}\"\n"));
        for args in [&["validate"][..], &["build-config", "--config", "X"][..], &["list", "FeatureDef"][..], &["mcp"][..], &["lsp"][..]] {
            let (out, code) = m.run(args);
            assert_eq!(code, 1, "{args:?}: {out}");
            assert!(out.contains(&form) && out.contains(&format!("this is syscribe {RUNNING}")), "{args:?}: {out}");
        }
    }
}

#[test]
fn a_model_whose_minimum_is_met_loads_and_commands_without_a_model_still_run() {
    for form in [RUNNING.to_string(), format!(">={RUNNING}"), "0.1.0".to_string()] {
        let m = M::new(&format!("[syscribe]\nminVersion = \"{form}\"\n"));
        let (out, code) = m.run(&["validate"]);
        assert_eq!(code, 0, "{form}: {out}");
    }
    let refusing = M::new(&format!("[syscribe]\nminVersion = \"{}\"\n", newer()));
    for args in [&["--version"][..], &["help"][..], &["capabilities"][..]] {
        let (out, code) = refusing.run(args);
        assert_eq!(code, 0, "{args:?}: {out}");
    }
}

#[test]
fn a_malformed_minimum_is_an_error_not_a_pass() {
    for bad in ["banana", "<2.0.0", "1.2.3.4"] {
        let m = M::new(&format!("[syscribe]\nminVersion = \"{bad}\"\n"));
        let (out, code) = m.run(&["validate"]);
        assert_eq!(code, 1, "{bad}: {out}");
        assert!(out.contains("minVersion must be a version"), "{bad}: {out}");
    }
}

#[test]
fn policy_and_capabilities_report_the_declared_minimum() {
    let m = M::new("[syscribe]\nminVersion = \"0.1.0\"\n");
    let (out, _) = m.run(&["policy", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["minVersion"], "0.1.0", "{out}");
    let (out, _) = m.run(&["capabilities", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["modelMinVersion"], "0.1.0", "{out}");
    let none = M::new("");
    let (out, _) = none.run(&["policy", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(v["minVersion"].is_null(), "{out}");
}

#[test]
fn validate_warns_when_the_model_uses_a_feature_newer_than_its_minimum() {
    let old = M::new("[syscribe]\nminVersion = \"0.60.0\"\n");
    let (out, code) = old.run(&["validate"]);
    assert!(out.contains("W329") && out.contains("derivedFrom") && out.contains("raise it to at least 0.61.0"), "{out}");
    assert_eq!(code, 0, "a warning, not an error: {out}");
    let (out, code) = old.run(&["validate", "--deny", "W329"]);
    assert_eq!(code, 2, "gate with --deny: {out}");
    for toml in ["[syscribe]\nminVersion = \"0.61.0\"\n", ""] {
        let m = M::new(toml);
        let (out, _) = m.run(&["validate"]);
        assert!(!out.contains("W329"), "{toml:?}: {out}");
    }
}

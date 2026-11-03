//! TC-TRS-VALBASE-001 (GH #431): `validate --new-since` / `--baseline-file`, against a throwaway repository.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Repo {
    root: PathBuf,
}

impl Repo {
    fn new() -> Repo {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-rev-it-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { root };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "t@example.com"]);
        r.git(&["config", "user.name", "T"]);
        r.git(&["config", "commit.gpgsign", "false"]);
        r.write("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
        r
    }
    fn model(&self) -> PathBuf {
        self.root.join("model")
    }
    fn git(&self, args: &[&str]) {
        let o = Command::new("git").arg("-C").arg(&self.root).args(args).output().unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    }
    fn write(&self, rel: &str, content: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }
    fn commit(&self, msg: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", msg]);
    }
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(self.model()).args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
    }
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn req(id: &str, status: &str) -> String {
    format!("---\nid: {id}\ntype: Requirement\nname: {id} name\nstatus: {status}\nreqDomain: software\nreqClass: system\n---\n\nThe system shall do {id}.\n")
}
fn tc(id: &str, verifies: &str) -> String {
    format!("---\nid: {id}\ntype: TestCase\nname: {id} name\nstatus: active\ntestLevel: L2\nverifies: [{verifies}]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n")
}

/// A model with two draft requirements and a baseline of accepted warnings.
fn base() -> Repo {
    let r = Repo::new();
    r.write("model/R/REQ-AAA-001.md", &req("REQ-AAA-001", "draft"));
    r.write("model/R/REQ-AAA-002.md", &req("REQ-AAA-002", "draft"));
    r.write("model/R/REQ-AAA-003.md", &req("REQ-AAA-003", "approved")); // pre-existing warnings (uncovered)
    r.commit("base");
    r
}

// ── GH #431: validate --new-since / --baseline-file ──────────────────────────

fn warns_as_errors(r: &Repo, extra: &[&str]) -> (String, String, i32) {
    let mut a = vec!["validate", "--warnings-as-errors"];
    a.extend_from_slice(extra);
    r.run(&a)
}

#[test]
fn pre_existing_warnings_do_not_trip_the_gate_but_a_new_one_does() {
    let r = base();
    // The accepted warnings alone trip a plain gate…
    let (_o, _e, c) = warns_as_errors(&r, &[]);
    assert_eq!(c, 2, "fixture must carry pre-existing warnings");
    // …but not against the revision that already had them.
    let (out, _e, c) = warns_as_errors(&r, &["--new-since", "HEAD"]);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("pre-existing finding(s) accepted"), "{out}");
    // One change introduces one more warning.
    r.write("model/R/REQ-AAA-004.md", &req("REQ-AAA-004", "approved"));
    let (out, _e, c) = warns_as_errors(&r, &["--new-since", "HEAD"]);
    assert_eq!(c, 2, "{out}");
    assert!(out.contains("REQ-AAA-004"), "{out}");
    assert!(!out.contains("| REQ-AAA-003"), "pre-existing findings must not be listed: {out}");
}

#[test]
fn json_flags_every_finding_new_or_not() {
    let r = base();
    r.write("model/R/REQ-AAA-004.md", &req("REQ-AAA-004", "approved"));
    let (out, _e, _c) = r.run(&["validate", "--new-since", "HEAD", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let items = v.as_array().unwrap();
    assert!(items.iter().any(|f| f["new"] == true && f["file"].as_str().unwrap().contains("REQ-AAA-004")), "{out}");
    assert!(items.iter().any(|f| f["new"] == false && f["file"].as_str().unwrap().contains("REQ-AAA-003")), "{out}");
    // Without a baseline the field is absent (output unchanged).
    let (plain, _e, _c) = r.run(&["validate", "--json"]);
    assert!(!plain.contains("\"new\""), "{plain}");
}

#[test]
fn moving_or_renaming_a_file_does_not_make_its_findings_new() {
    let r = base();
    std::fs::create_dir_all(r.root.join("model/Moved")).unwrap();
    r.git(&["mv", "model/R/REQ-AAA-003.md", "model/Moved/REQ-AAA-003.md"]);
    let (out, _e, c) = warns_as_errors(&r, &["--new-since", "HEAD"]);
    assert_eq!(c, 0, "{out}");
}

#[test]
fn a_second_copy_of_an_accepted_finding_is_new() {
    let r = base();
    // A sibling with an identical shape raises a message differing only by id → new.
    r.write("model/R/REQ-AAA-005.md", &req("REQ-AAA-005", "approved"));
    let (_o, _e, c) = warns_as_errors(&r, &["--new-since", "HEAD"]);
    assert_eq!(c, 2);
}

#[test]
fn baseline_file_round_trip() {
    let r = base();
    let bl = r.root.join("accepted.json");
    let (_o, e, c) = r.run(&["validate", "--write-baseline", bl.to_str().unwrap()]);
    assert_eq!(c, 0, "{e}");
    assert!(e.contains("Wrote"), "{e}");
    let saved: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&bl).unwrap()).unwrap();
    assert_eq!(saved["schemaVersion"], "1.0");
    assert!(!saved["findings"].as_array().unwrap().is_empty());
    let (out, _e, c) = warns_as_errors(&r, &["--baseline-file", bl.to_str().unwrap()]);
    assert_eq!(c, 0, "{out}");
    r.write("model/R/REQ-AAA-004.md", &req("REQ-AAA-004", "approved"));
    let (_o, _e, c) = warns_as_errors(&r, &[&format!("--baseline-file={}", bl.display())]);
    assert_eq!(c, 2);
}

#[test]
fn usage_errors_exit_one() {
    let r = base();
    let (_o, e, c) = r.run(&["validate", "--new-since", "no-such-rev"]);
    assert_eq!(c, 1);
    assert!(e.contains("does not name a commit"), "{e}");
    let (_o, e, c) = r.run(&["validate", "--new-since", "HEAD", "--baseline-file", "x.json"]);
    assert_eq!(c, 1);
    assert!(e.contains("mutually exclusive"), "{e}");
    let (_o, _e, c) = r.run(&["validate", "--baseline-file", "/no/such/file.json"]);
    assert_eq!(c, 1);
    let (_o, e, c) = r.run(&["validate", "--all-configs", "--new-since", "HEAD"]);
    assert_eq!(c, 1);
    assert!(e.contains("--all-configs"), "{e}");
}

#[allow(dead_code)]
fn _unused(_: &Path) {}

//! TC-TRS-CHECKSTAGED-001 (GH #445): `check-staged` and its pre-commit hook, against throwaway git repos.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Repo {
    root: PathBuf,
}

const GOOD_TC: &str = "---\nid: TC-CS-001\ntype: TestCase\nname: ok test\nstatus: active\ntestLevel: L2\nverifies: [REQ-CS-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n";
// Lists testFunctions but has no Gherkin block at all.
const BAD_TC: &str = "---\nid: TC-CS-001\ntype: TestCase\nname: bad test\nstatus: active\ntestLevel: L2\nverifies: [REQ-CS-001]\ntestFunctions:\n  - function: test_it\n    scenario: Does the thing\n---\n\nNo gherkin here.\n";

fn req(domain: &str) -> String {
    format!("---\nid: REQ-CS-001\ntype: Requirement\nname: R\nstatus: draft\nreqDomain: {domain}\nreqClass: system\n---\n\nThe system shall do it.\n")
}

impl Repo {
    fn new() -> Repo {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-cs-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { root };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "t@example.com"]);
        r.git(&["config", "user.name", "T"]);
        r.git(&["config", "commit.gpgsign", "false"]);
        r.write("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
        r.write("model/REQ-CS-001.md", &req("software"));
        r.write("model/Part.md", "---\ntype: PartDef\nname: Part\ndomain: software\nsatisfies: [REQ-CS-001]\n---\n\nPart.\n");
        r.write("README.md", "hello\n");
        r.commit("base");
        r
    }
    fn model(&self) -> PathBuf {
        self.root.join("model")
    }
    fn git(&self, args: &[&str]) {
        let o = Command::new("git").arg("-C").arg(&self.root).args(args).output().unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    }
    fn write(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn commit(&self, msg: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", msg]);
    }
    fn stage(&self, rel: &str) {
        self.git(&["add", rel]);
    }
    fn check(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).current_dir(&self.root).arg("-m").arg(self.model()).arg("check-staged").args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
    }
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn json(out: &str) -> Value {
    serde_json::from_str(out).unwrap_or_else(|e| panic!("{e}: {out}"))
}

#[test]
fn a_staged_test_case_without_gherkin_fails_and_names_the_fix() {
    let r = Repo::new();
    r.write("model/TC-CS-001.md", BAD_TC);
    r.stage("model/TC-CS-001.md");
    let (out, err, code) = r.check(&[]);
    assert_eq!(code, 1, "{out}\n{err}");
    assert!(out.contains("| E106 |") && out.contains("model/TC-CS-001.md") && out.contains("scaffold-gherkin TC-CS-001 --fix"), "{out}");
    assert!(out.contains("| E011 |"), "{out}");
    let (hinted, _e, _c) = r.check(&["--fix-hints"]);
    let tail = &hinted[hinted.find("How to fix:").expect("a How to fix section")..];
    assert!(tail.contains("E106") && tail.contains("syscribe help scaffold-gherkin"), "{tail}");
    // JSON.
    let (j, _e, code) = r.check(&["--json"]);
    assert_eq!(code, 1);
    let v = json(&j);
    assert_eq!(v["ok"], false);
    assert_eq!(v["stagedFiles"][0], "model/TC-CS-001.md");
    assert!(v["findings"].as_array().unwrap().iter().any(|f| f["code"] == "E106" && f["severity"] == "error"));
}

#[test]
fn staging_only_unrelated_files_is_a_quick_no_op() {
    let r = Repo::new();
    r.write("README.md", "changed\n");
    r.write("src/a.c", "int a;\n");
    r.stage("README.md");
    r.stage("src/a.c");
    let started = std::time::Instant::now();
    let (out, _e, code) = r.check(&[]);
    assert_eq!(code, 0);
    assert!(out.contains("no staged model files"), "{out}");
    assert!(started.elapsed().as_secs() < 2, "a no-op must not load the model");
    // Nothing staged at all behaves the same.
    r.git(&["reset", "-q"]);
    assert_eq!(r.check(&[]).2, 0);
    let (j, _e, _c) = r.check(&["--json"]);
    assert_eq!(json(&j)["ok"], true);
}

#[test]
fn the_check_is_about_the_staged_content_not_the_working_tree() {
    let r = Repo::new();
    // A valid test staged, then spoiled (unstaged): the commit would still be fine.
    r.write("model/TC-CS-001.md", GOOD_TC);
    r.stage("model/TC-CS-001.md");
    r.write("model/TC-CS-001.md", BAD_TC);
    let (out, err, code) = r.check(&[]);
    assert_eq!(code, 0, "{out}\n{err}");
    assert!(out.contains("judged on the staged content"), "{out}");
    // A broken test staged, then fixed in the working tree only: the commit would still be broken.
    r.git(&["reset", "-q"]);
    r.write("model/TC-CS-001.md", BAD_TC);
    r.stage("model/TC-CS-001.md");
    r.write("model/TC-CS-001.md", GOOD_TC);
    let (out, _e, code) = r.check(&[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("| E106 |"), "{out}");
    // An untracked model file is not part of the commit either: it cannot satisfy a reference.
    r.git(&["reset", "-q"]);
    r.write("model/TC-CS-001.md", "---\nid: TC-CS-001\ntype: TestCase\nname: t\nstatus: active\ntestLevel: L2\nverifies: [REQ-CS-009]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    r.write("model/REQ-CS-009.md", &req("software").replace("REQ-CS-001", "REQ-CS-009"));
    r.stage("model/TC-CS-001.md");
    let (out, _e, code) = r.check(&[]);
    assert_eq!(code, 1, "the reference is not in the commit: {out}");
    assert!(out.contains("REQ-CS-009"), "{out}");
}

#[test]
fn direct_dependants_of_a_staged_element_are_checked() {
    let r = Repo::new();
    // Making the requirement hardware-domain breaks the software Part that satisfies it (E313 on the Part's file).
    r.write("model/REQ-CS-001.md", &req("hardware"));
    r.stage("model/REQ-CS-001.md");
    let (out, _e, code) = r.check(&[]);
    assert_eq!(code, 1, "{out}");
    let row = out.lines().find(|l| l.contains("| E313 |")).unwrap_or_else(|| panic!("{out}"));
    assert!(row.contains("model/Part.md"), "the finding is on the dependant, which was not staged: {row}");
    assert!(out.contains("1 direct dependant(s)"), "{out}");
    // Unrelated findings elsewhere in the model are not reported.
    let r2 = Repo::new();
    r2.write("model/Broken.md", "---\ntype: PartDef\nname: Broken\nsupertype: Nope::Missing\n---\n\nx\n");
    r2.commit("an existing problem");
    r2.write("model/REQ-CS-001.md", &req("software").replace("The system shall do it.", "The system shall do it well."));
    r2.stage("model/REQ-CS-001.md");
    let (out, _e, code) = r2.check(&[]);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("Broken"), "{out}");
}

#[test]
fn a_staged_deletion_reports_the_dependants_it_leaves_dangling() {
    let r = Repo::new();
    r.git(&["rm", "-q", "model/REQ-CS-001.md"]);
    let (out, _e, code) = r.check(&[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("model/Part.md"), "the Part now satisfies nothing: {out}");
}

#[test]
fn warnings_fail_only_when_asked_to() {
    let r = Repo::new();
    // An approved leaf requirement with no test: a warning, not an error.
    r.write("model/REQ-CS-002.md", &req("software").replace("REQ-CS-001", "REQ-CS-002").replace("draft", "approved"));
    r.stage("model/REQ-CS-002.md");
    let (out, _e, code) = r.check(&[]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("warning(s)") && !out.contains("0 warning(s)"), "there is a warning: {out}");
    assert_eq!(r.check(&["--warnings-as-errors"]).2, 2);
    assert_eq!(r.check(&["--max-warnings", "0"]).2, 2);
    assert_eq!(r.check(&["--max-warnings", "50"]).2, 0);
}

#[test]
fn staging_the_project_config_checks_everything() {
    let r = Repo::new();
    r.write("model/Broken.md", "---\ntype: PartDef\nname: Broken\nsupertype: Nope::Missing\n---\n\nx\n");
    r.commit("an existing problem");
    r.write("model/.syscribe.toml", "[ids.prefixes]\nRequirement = [\"STK\"]\n");
    r.stage("model/.syscribe.toml");
    let (out, _e, code) = r.check(&[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("Broken"), "a config change can affect any element: {out}");
}

#[test]
fn outside_a_git_repository_it_says_so() {
    let d = std::env::temp_dir().join(format!("syscribe-cs-nogit-{}", std::process::id())).join("model");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&d).arg("check-staged").output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("not inside a git repository"));
}

// ── the hook ─────────────────────────────────────────────────────────────────

fn commit(r: &Repo, path_with_syscribe: bool, msg: &str) -> (i32, String) {
    let bin = std::path::Path::new(env!("CARGO_BIN_EXE_syscribe")).parent().unwrap().to_path_buf();
    let path = if path_with_syscribe { format!("{}:/usr/bin:/bin", bin.display()) } else { "/usr/bin:/bin".to_string() };
    let o = Command::new("git").arg("-C").arg(&r.root).args(["commit", "-q", "-m", msg]).env("PATH", path).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

fn head(r: &Repo) -> String {
    String::from_utf8_lossy(&Command::new("git").arg("-C").arg(&r.root).args(["rev-parse", "HEAD"]).output().unwrap().stdout).trim().to_string()
}

#[test]
fn the_installed_hook_blocks_a_bad_commit_and_lets_a_good_one_through() {
    let r = Repo::new();
    let (out, err, code) = r.check(&["--install-hook"]);
    assert_eq!(code, 0, "{out}\n{err}");
    let hook = r.root.join(".git/hooks/pre-commit");
    let script = std::fs::read_to_string(&hook).unwrap();
    assert!(script.contains("# syscribe:check-staged") && script.contains("check-staged"), "{script}");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert!(std::fs::metadata(&hook).unwrap().permissions().mode() & 0o111 != 0, "the hook is executable");
    }

    let before = head(&r);
    r.write("model/TC-CS-001.md", BAD_TC);
    r.stage("model/TC-CS-001.md");
    let (code, out) = commit(&r, true, "bad");
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("E106") && out.contains("scaffold-gherkin"), "the developer sees why: {out}");
    assert_eq!(head(&r), before, "nothing was committed");

    r.write("model/TC-CS-001.md", GOOD_TC);
    r.stage("model/TC-CS-001.md");
    let (code, out) = commit(&r, true, "good");
    assert_eq!(code, 0, "{out}");
    assert_ne!(head(&r), before);

    // A commit that stages no model file is not slowed down or blocked.
    r.write("README.md", "again\n");
    r.stage("README.md");
    assert_eq!(commit(&r, true, "docs").0, 0);
}

#[test]
fn the_hook_lets_a_commit_through_when_syscribe_is_not_installed() {
    let r = Repo::new();
    assert_eq!(r.check(&["--install-hook"]).2, 0);
    r.write("model/TC-CS-001.md", BAD_TC);
    r.stage("model/TC-CS-001.md");
    let (code, out) = commit(&r, false, "no tool");
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("syscribe not found"), "it says so rather than silently passing: {out}");
}

#[test]
fn install_hook_never_overwrites_someone_elses_hook() {
    let r = Repo::new();
    let hook = r.root.join(".git/hooks/pre-commit");
    std::fs::create_dir_all(hook.parent().unwrap()).unwrap();
    std::fs::write(&hook, "#!/bin/sh\necho mine\n").unwrap();
    let (_o, err, code) = r.check(&["--install-hook"]);
    assert_eq!(code, 1);
    assert!(err.contains("not installed by syscribe") && err.contains("check-staged"), "{err}");
    assert_eq!(std::fs::read_to_string(&hook).unwrap(), "#!/bin/sh\necho mine\n");
    // Its own hook can be reinstalled.
    std::fs::remove_file(&hook).unwrap();
    assert_eq!(r.check(&["--install-hook"]).2, 0);
    assert_eq!(r.check(&["--install-hook"]).2, 0);
}

#[test]
fn a_hooks_path_is_respected() {
    let r = Repo::new();
    r.git(&["config", "core.hooksPath", ".githooks"]);
    assert_eq!(r.check(&["--install-hook"]).2, 0);
    assert!(r.root.join(".githooks/pre-commit").exists());
    assert!(!r.root.join(".git/hooks/pre-commit").exists());
}

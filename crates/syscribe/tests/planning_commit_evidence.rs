//! TC-TRS-PICOMMIT-001 (GH #455): PlanningItem `evidence: - commit: <sha>` resolved against the git
//! history — `E728`, counting for `E719`, the `[planning]` gates `W320`/`W321`, and the read and
//! write surfaces (`steps --json`, `context`, `set … evidence.add commit=`, `log --item`).

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Repo {
    root: PathBuf,
}

fn scratch(tag: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-picommit-{tag}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

impl Repo {
    /// A repository with the model under `model/` and two commits: `c1` adds `src/a.rs`, `c2` adds `src/b.rs`.
    fn new(toml: &str) -> Repo {
        let r = Repo { root: scratch("git") };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "dev@example.com"]);
        r.git(&["config", "user.name", "Dev"]);
        r.git(&["config", "commit.gpgsign", "false"]);
        if !toml.is_empty() {
            r.write("model/.syscribe.toml", toml);
        }
        r.write("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
        r.write("model/R/REQ-CM-001.md", "---\nid: REQ-CM-001\ntype: Requirement\nname: R1\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nThe tool shall work.\n");
        r.write(
            "model/T/TC-CM-001.md",
            "---\nid: TC-CM-001\ntype: TestCase\nname: T1\nstatus: draft\ntestLevel: L2\nsourceFile: repo:src/a.rs\nverifies: [REQ-CM-001]\n---\n\n```gherkin\nFeature: F\n\n  Scenario: S\n    Then it works\n```\n",
        );
        r.commit("model");
        r.write("src/a.rs", "fn a() {}\n");
        r.commit("add a");
        r.write("src/b.rs", "fn b() {}\n");
        r.commit("add b");
        r
    }
    fn sha(&self, rev: &str) -> String {
        self.git(&["rev-parse", rev])
    }
    fn model(&self) -> PathBuf {
        self.root.join("model")
    }
    fn git(&self, args: &[&str]) -> String {
        let o = Command::new("git").arg("-C").arg(&self.root).args(args).output().unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }
    fn write(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.root.join(rel)).unwrap()
    }
    fn commit(&self, msg: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", msg]);
    }
    fn pi(&self, id: &str, status: &str, evidence: &str) {
        self.write(&format!("model/P/{id}.md"), &pi(id, status, evidence));
    }
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        run_in(&self.model(), args)
    }
    fn codes(&self) -> Vec<(String, String)> {
        codes_in(&self.model())
    }
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn pi(id: &str, status: &str, evidence: &str) -> String {
    format!("---\nid: {id}\ntype: PlanningItem\nname: {id}\nstatus: {status}\nachieves: [REQ-CM-001]\nevidence:\n{evidence}---\n\nBody.\n")
}

fn run_in(model: &std::path::Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

/// `(code, message)` of every finding.
fn codes_in(model: &std::path::Path) -> Vec<(String, String)> {
    let (out, _e, _c) = run_in(model, &["validate", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    v.as_array().unwrap().iter().map(|f| (f["code"].as_str().unwrap().to_string(), f["message"].as_str().unwrap_or("").to_string())).collect()
}

fn with<'a>(fs: &'a [(String, String)], code: &str) -> Vec<&'a str> {
    fs.iter().filter(|(c, _)| c == code).map(|(_, m)| m.as_str()).collect()
}

// ── resolution: E728 and E719 ────────────────────────────────────────────────

#[test]
fn a_resolving_commit_is_evidence_for_a_done_leaf() {
    let r = Repo::new("");
    let short = &r.sha("HEAD~1")[..9];
    r.pi("PI-CM-001", "done", &format!("  - commit: \"{short}\"\n"));
    let fs = r.codes();
    assert!(with(&fs, "E728").is_empty(), "{fs:?}");
    assert!(with(&fs, "E719").is_empty(), "{fs:?}");
}

#[test]
fn malformed_and_unknown_commits_are_e728_and_do_not_count() {
    let r = Repo::new("");
    r.pi("PI-CM-001", "done", "  - commit: xyz1234\n  - commit: abc12\n  - commit: \"0123456789abcdef0123456789abcdef01234567\"\n");
    let fs = r.codes();
    let e = with(&fs, "E728");
    assert_eq!(e.len(), 3, "{fs:?}");
    assert!(e.iter().any(|m| m.contains("xyz1234") && m.contains("malformed")), "{e:?}");
    assert!(e.iter().any(|m| m.contains("abc12") && m.contains("fewer than 7")), "{e:?}");
    assert!(e.iter().any(|m| m.contains("unknown")), "{e:?}");
    assert_eq!(with(&fs, "E719").len(), 1, "{fs:?}");
}

#[test]
fn a_waived_commit_entry_is_not_checked() {
    let r = Repo::new("");
    r.pi("PI-CM-001", "todo", "  - commit: \"0123456789abcdef0123456789abcdef01234567\"\n    rationale: lost in a force-push\n");
    assert!(with(&r.codes(), "E728").is_empty());
}

#[test]
fn outside_a_repository_resolution_is_skipped() {
    let root = scratch("nogit");
    let w = |rel: &str, c: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("R/REQ-CM-001.md", "---\nid: REQ-CM-001\ntype: Requirement\nname: R1\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nThe tool shall work.\n");
    w("P/PI-CM-001.md", &pi("PI-CM-001", "done", "  - commit: \"0123456789abcdef0123456789abcdef01234567\"\n"));
    let fs = codes_in(&root);
    assert!(with(&fs, "E728").is_empty(), "{fs:?}");
    // Not checkable, so not held against the item either.
    assert!(with(&fs, "E719").is_empty(), "{fs:?}");
    let _ = std::fs::remove_dir_all(&root);
}

// ── the [planning] gates ─────────────────────────────────────────────────────

#[test]
fn w320_a_commit_that_touches_none_of_the_items_paths() {
    let r = Repo::new("[planning]\ncheck_commit_paths = true\n");
    let (c1, c2) = (r.sha("HEAD~1"), r.sha("HEAD"));
    // Names src/a.rs by path; c2 only touched src/b.rs.
    r.pi("PI-CM-001", "todo", &format!("  - path: repo:src/a.rs\n  - commit: \"{}\"\n", &c2[..10]));
    // Names src/a.rs through the TestCase's sourceFile; c1 touched it.
    r.pi("PI-CM-002", "todo", &format!("  - ref: TC-CM-001\n  - commit: \"{}\"\n", &c1[..10]));
    // Names no path at all: nothing to compare against.
    r.pi("PI-CM-003", "todo", &format!("  - commit: \"{}\"\n", &c2[..10]));
    let fs = r.codes();
    let w = with(&fs, "W320");
    assert_eq!(w.len(), 1, "{fs:?}");
    assert!(w[0].contains(&c2[..7]) && w[0].contains("add b"), "{w:?}");
}

#[test]
fn w320_is_off_by_default() {
    let r = Repo::new("");
    let c2 = r.sha("HEAD");
    r.pi("PI-CM-001", "todo", &format!("  - path: repo:src/a.rs\n  - commit: \"{c2}\"\n"));
    assert!(with(&r.codes(), "W320").is_empty());
}

#[test]
fn w321_a_done_leaf_without_commit_evidence() {
    let r = Repo::new("[planning]\nrequire_commit_evidence = true\n");
    let c1 = r.sha("HEAD~1");
    r.pi("PI-CM-001", "done", "  - path: repo:src/a.rs\n");
    r.pi("PI-CM-002", "done", &format!("  - path: repo:src/a.rs\n  - commit: \"{c1}\"\n"));
    r.pi("PI-CM-003", "in_progress", "  - path: repo:src/a.rs\n");
    let fs = r.codes();
    let w = with(&fs, "W321");
    assert_eq!(w.len(), 1, "{fs:?}");
    assert!(w[0].contains("PI-CM-001"), "{w:?}");
    // Off by default.
    let r = Repo::new("");
    r.pi("PI-CM-001", "done", "  - path: repo:src/a.rs\n");
    assert!(with(&r.codes(), "W321").is_empty());
}

// ── read and write surfaces ──────────────────────────────────────────────────

#[test]
fn steps_and_context_show_the_resolved_commit() {
    let r = Repo::new("");
    let c1 = r.sha("HEAD~1");
    r.pi("PI-CM-001", "done", &format!("  - commit: \"{}\"\n", &c1[..8]));
    let (out, e, c) = r.run(&["steps", "PI-CM-001", "--json"]);
    assert_eq!(c, 0, "{e}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let commit = &v["tree"]["commits"][0];
    assert_eq!(commit["sha"], c1.as_str(), "{out}");
    assert_eq!(commit["subject"], "add a", "{out}");
    assert_eq!(commit["status"], "resolved", "{out}");
    let (out, e, c) = r.run(&["context", "PI-CM-001", "--format", "json"]);
    assert_eq!(c, 0, "{e}");
    assert!(out.contains(&c1) && out.contains("add a"), "{out}");
}

#[test]
fn set_evidence_add_commit_checks_and_writes_the_entry() {
    let r = Repo::new("");
    r.pi("PI-CM-001", "todo", "  - path: repo:src/a.rs\n");
    let c1 = r.sha("HEAD~1");
    let (_o, e, c) = r.run(&["set", "PI-CM-001", "evidence.add", &format!("commit={}", &c1[..7])]);
    assert_eq!(c, 0, "{e}");
    let text = r.read("model/P/PI-CM-001.md");
    // `set` quotes a hash YAML would read as a number (e.g. `7666202e104`), so accept either form.
    let short = &c1[..7];
    assert!(text.contains(&format!("- commit: {short}")) || text.contains(&format!("- commit: \"{short}\"")) || text.contains(&format!("- commit: '{short}'")), "{text}");
    // Unknown and malformed hashes are refused, nothing written.
    let (_o, e, c) = r.run(&["set", "PI-CM-001", "evidence.add", "commit=0123456789abcdef0123456789abcdef01234567"]);
    assert_eq!(c, 1, "{e}");
    assert!(e.contains("unknown"), "{e}");
    let (_o, e, c) = r.run(&["set", "PI-CM-001", "evidence.add", "commit=nothex!"]);
    assert_eq!(c, 1, "{e}");
    assert!(e.contains("malformed"), "{e}");
    assert_eq!(r.read("model/P/PI-CM-001.md"), text);
}

#[test]
fn log_item_lists_the_commits_named_as_evidence() {
    let r = Repo::new("");
    let c1 = r.sha("HEAD~1");
    r.pi("PI-CM-001", "done", &format!("  - commit: \"{}\"\n", &c1[..8]));
    let (out, e, c) = r.run(&["log", "--item", "PI-CM-001", "--json"]);
    assert_eq!(c, 0, "{e}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let entries = v["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1, "{out}");
    assert_eq!(entries[0]["subject"], "add a", "{out}");
    let (out, _e, c) = r.run(&["log", "--item", "PI-CM-001"]);
    assert_eq!(c, 0);
    assert!(out.contains("add a"), "{out}");
    let (_o, e, c) = r.run(&["log", "--item", "REQ-CM-001"]);
    assert_eq!(c, 1, "{e}");
}

#[test]
fn an_unquoted_hash_yaml_reads_as_a_number_is_e728_asking_for_quotes() {
    let r = Repo::new("");
    r.pi("PI-CM-001", "todo", "  - commit: 7666202e104\n");
    let fs = r.codes();
    let e = with(&fs, "E728");
    assert_eq!(e.len(), 1, "{fs:?}");
    assert!(e[0].contains("7.666202e110") && e[0].contains("quote it"), "{fs:?}");
}

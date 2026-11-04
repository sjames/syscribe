//! TC-TRS-PROVLOG-001 (GH #443) and TC-TRS-WRITEGUARD-001 (GH #444): provenance stored in git,
//! `log`, `--expect-hash`, `--json` envelopes and the exit-code contract.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Repo {
    root: PathBuf,
}

fn pi(id: &str, status: &str, extra: &str) -> String {
    format!("---\nid: {id}\ntype: PlanningItem\nname: Item {id}\nstatus: {status}\nachieves: [REQ-PV-001]\n{extra}---\n\nBody.\n")
}

impl Repo {
    fn new() -> Repo {
        Repo::new_dated(None)
    }

    /// The base commit is dated `date` when given, so later commits are chronologically after it.
    fn new_dated(date: Option<&str>) -> Repo {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-prov-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { root };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "human@example.com"]);
        r.git(&["config", "user.name", "Human"]);
        r.git(&["config", "commit.gpgsign", "false"]);
        r.write("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
        r.write("model/REQ-PV-001.md", "---\nid: REQ-PV-001\ntype: Requirement\nname: R\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nShall.\n");
        r.write("model/PI-PV-001.md", &pi("PI-PV-001", "todo", "questions:\n  - id: Q1\n    text: Which port?\n    status: open\n"));
        r.write("model/PI-PV-002.md", &pi("PI-PV-002", "todo", ""));
        r.write("notes.txt", "unrelated\n");
        r.git(&["add", "-A"]);
        let mut c = Command::new("git");
        c.arg("-C").arg(&r.root).args(["commit", "-q", "-m", "base"]);
        if let Some(d) = date {
            c.env("GIT_AUTHOR_DATE", d).env("GIT_COMMITTER_DATE", d);
        }
        assert!(c.output().unwrap().status.success());
        r
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
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).current_dir(&self.root).arg("-m").arg(self.model()).args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
    }
    fn commits(&self) -> usize {
        self.git(&["rev-list", "--count", "HEAD"]).parse().unwrap()
    }
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn json(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or_else(|e| panic!("{e}: {s}"))
}

// ── --commit: provenance lands in git (GH #443) ──────────────────────────────

#[test]
fn commit_records_the_actor_the_reason_and_the_command_in_git() {
    let r = Repo::new();
    let n = r.commits();
    let (out, err, code) = r.run(&["set", "PI-PV-001", "status=in_progress", "--by", "agent-1", "--reason", "starting work", "--commit"]);
    assert_eq!(code, 0, "{out}\n{err}");
    assert!(out.contains("Committed "), "{out}");
    assert_eq!(r.commits(), n + 1);
    assert_eq!(r.git(&["log", "-1", "--format=%an"]), "agent-1", "the actor is the author");
    assert_eq!(r.git(&["log", "-1", "--format=%cn"]), "Human", "the committer is the configured git user");
    let msg = r.git(&["log", "-1", "--format=%B"]);
    assert!(msg.starts_with("syscribe: set PI-PV-001 status=in_progress"), "{msg}");
    assert!(msg.contains("starting work"), "{msg}");
    assert!(msg.contains("Syscribe-Command: set PI-PV-001 status=in_progress"), "{msg}");
    assert!(msg.contains("Syscribe-Actor: agent-1") && msg.contains("Syscribe-Reason: starting work"), "{msg}");
    assert_eq!(r.git(&["status", "--porcelain"]), "", "the working tree is clean afterwards");
}

#[test]
fn only_the_changed_files_are_committed_never_another_agents_pending_work() {
    let r = Repo::new();
    // Another agent has a staged change and an unstaged one, in the same working tree.
    r.write("model/PI-PV-002.md", &pi("PI-PV-002", "in_progress", ""));
    r.git(&["add", "model/PI-PV-002.md"]);
    r.write("notes.txt", "someone else is editing this\n");
    let (_o, err, code) = r.run(&["set", "PI-PV-001", "status=in_progress", "--by", "agent-1", "--commit"]);
    assert_eq!(code, 0, "{err}");
    let files = r.git(&["show", "--name-only", "--format=", "HEAD"]);
    assert_eq!(files, "model/PI-PV-001.md", "exactly the file the command changed");
    let status = r.git(&["status", "--porcelain"]);
    assert!(status.contains("M  model/PI-PV-002.md"), "the other agent's staged change is still staged: {status}");
    assert!(status.contains(" M notes.txt"), "and the unstaged one untouched: {status}");
}

#[test]
fn a_multi_file_command_is_one_commit() {
    let r = Repo::new();
    r.write("model/Parts/Alpha.md", "---\ntype: PartDef\nname: Alpha\n---\n\nA.\n");
    r.write("model/Parts/User.md", "---\ntype: PartDef\nname: User\nsupertype: Parts::Alpha\n---\n\nU.\n");
    r.commit("parts");
    let n = r.commits();
    let (_o, err, code) = r.run(&["move", "Parts::Alpha", "Parts::Beta", "--by", "agent-2", "--commit"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(r.commits(), n + 1, "one command, one commit");
    let files = r.git(&["show", "--name-only", "--format=", "HEAD"]);
    assert!(files.contains("Beta") && files.contains("User.md"), "the moved file and the rewritten reference: {files}");
}

#[test]
fn the_actor_is_optional() {
    let r = Repo::new();
    let (_o, err, code) = r.run(&["set", "PI-PV-001", "status=in_progress", "--commit"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(r.git(&["log", "-1", "--format=%an"]), "Human", "no actor: the configured git identity");
    let msg = r.git(&["log", "-1", "--format=%B"]);
    assert!(msg.contains("Syscribe-Command:") && !msg.contains("Syscribe-Actor"), "{msg}");
}

#[test]
fn nothing_is_committed_unless_asked_and_a_dry_run_never_commits() {
    let r = Repo::new();
    let n = r.commits();
    let (_o, _e, code) = r.run(&["set", "PI-PV-001", "status=in_progress", "--by", "agent-1"]);
    assert_eq!(code, 0);
    assert_eq!(r.commits(), n, "no --commit, no commit");
    assert!(r.git(&["status", "--porcelain"]).contains("model/PI-PV-001.md"));
    r.git(&["checkout", "--", "."]);
    let (_o, _e, code) = r.run(&["set", "PI-PV-001", "status=in_progress", "--commit", "--dry-run"]);
    assert_eq!(code, 0);
    assert_eq!(r.commits(), n, "a dry run does not commit");
    assert_eq!(r.git(&["status", "--porcelain"]), "");
}

#[test]
fn the_config_switch_commits_without_the_flag() {
    let r = Repo::new();
    r.write("model/.syscribe.toml", "[provenance]\ncommit = true\n");
    r.commit("enable");
    let n = r.commits();
    let (_o, err, code) = r.run(&["set", "PI-PV-001", "status=in_progress", "--by", "agent-3"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(r.commits(), n + 1);
    assert_eq!(r.git(&["log", "-1", "--format=%an"]), "agent-3");
}

#[test]
fn a_failed_commit_leaves_the_change_in_place_and_says_so() {
    let r = Repo::new();
    let hook = r.root.join(".git/hooks/pre-commit");
    std::fs::create_dir_all(hook.parent().unwrap()).unwrap();
    std::fs::write(&hook, "#!/bin/sh\necho 'rejected by policy' >&2\nexit 1\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let n = r.commits();
    let (_o, err, code) = r.run(&["set", "PI-PV-001", "status=in_progress", "--commit"]);
    assert_eq!(code, 1);
    assert!(err.contains("written, but committing it failed") && err.contains("rejected by policy"), "{err}");
    assert_eq!(r.commits(), n);
    assert!(r.read("model/PI-PV-001.md").contains("status: in_progress"), "the edit itself was made");
}

#[test]
fn a_bad_actor_id_is_a_usage_error() {
    let r = Repo::new();
    let before = r.read("model/PI-PV-001.md");
    for by in ["bad actor", "a<b>", ""] {
        let (_o, err, code) = r.run(&["set", "PI-PV-001", "status=in_progress", "--by", by]);
        assert_eq!(code, 1, "{by:?}");
        assert!(err.contains("not a valid actor id"), "{err}");
    }
    assert_eq!(r.read("model/PI-PV-001.md"), before);
}

#[test]
fn answering_a_question_records_the_actor_as_the_answerer() {
    let r = Repo::new();
    let (_o, err, code) = r.run(&["set", "PI-PV-001", "question.answer", "Q1", "text=UART2", "--by", "alice"]);
    assert_eq!(code, 0, "{err}");
    assert!(r.read("model/PI-PV-001.md").contains("answeredBy: alice"));
}

// ── log: read the history back (GH #443) ─────────────────────────────────────

#[test]
fn log_for_an_element_shows_who_did_what_and_why() {
    let r = Repo::new();
    r.run(&["set", "PI-PV-001", "status=in_progress", "--by", "agent-1", "--reason", "picking this up", "--commit"]);
    r.run(&["set", "PI-PV-001", "status=done", "--by", "agent-2", "--reason", "all evidence in", "--commit"]);
    // A hand edit, committed by a person.
    r.write("model/PI-PV-001.md", &r.read("model/PI-PV-001.md").replace("name: Item PI-PV-001", "name: Renamed by hand"));
    r.commit("tweak the name");

    let (out, err, code) = r.run(&["log", "PI-PV-001"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("agent-2 (as agent-2)") || out.contains("agent-2"), "{out}");
    assert!(out.contains("status in_progress → done") && out.contains("reason: all evidence in"), "{out}");
    assert!(out.contains("status todo → in_progress") && out.contains("reason: picking this up"), "{out}");
    assert!(out.contains("tweak the name") && out.contains("[not made by syscribe]"), "{out}");
    assert!(out.contains("fields name"), "the hand edit's field change is shown: {out}");

    let (j, _e, _c) = r.run(&["log", "PI-PV-001", "--json"]);
    let v = json(&j);
    let e = v["entries"].as_array().unwrap();
    assert_eq!(e[0]["attributed"], false);
    assert_eq!(e[1]["actor"], "agent-2");
    assert_eq!(e[1]["change"], "status in_progress → done");
    assert_eq!(e[2]["command"], "set PI-PV-001 status=in_progress");
}

#[test]
fn log_filters_by_actor_unattributed_since_and_limit() {
    let r = Repo::new();
    r.run(&["set", "PI-PV-001", "status=in_progress", "--by", "agent-1", "--commit"]);
    r.run(&["set", "PI-PV-002", "status=in_progress", "--by", "agent-2", "--commit"]);
    let (out, _e, _c) = r.run(&["log", "--by", "agent-1"]);
    assert!(out.contains("agent-1") && !out.contains("agent-2"), "{out}");
    let (out, _e, _c) = r.run(&["log", "--unattributed"]);
    assert!(out.contains("base") && !out.contains("syscribe: set"), "only the commits that were not made by syscribe: {out}");
    let (j, _e, _c) = r.run(&["log", "--limit", "1", "--json"]);
    assert_eq!(json(&j)["entries"].as_array().unwrap().len(), 1);
    assert_eq!(r.run(&["log", "--limit", "x"]).2, 1);
}

#[test]
fn log_since_excludes_older_commits() {
    let r = Repo::new_dated(Some("2020-01-01T00:00:00"));
    r.run(&["set", "PI-PV-001", "status=in_progress", "--by", "agent-1", "--commit"]);
    let (out, _e, _c) = r.run(&["log", "--since", "2021-01-01"]);
    assert!(out.contains("syscribe: set") && !out.contains("base"), "the 2020 base commit is older than --since: {out}");
    let (out, _e, _c) = r.run(&["log"]);
    assert!(out.contains("base") && out.contains("syscribe: set"), "without --since both are there: {out}");
    let (out, _e, _c) = r.run(&["log", "--since", "2999-01-01"]);
    assert!(out.contains("History of"), "a far-future date is accepted: {out}");
}

#[test]
fn log_notes_uncommitted_changes_and_follows_a_rename() {
    let r = Repo::new();
    r.write("model/PI-PV-002.md", &pi("PI-PV-002", "in_progress", ""));
    let (out, _e, _c) = r.run(&["log"]);
    assert!(out.contains("Uncommitted changes in 1 file(s)") && out.contains("PI-PV-002.md"), "{out}");
    r.git(&["checkout", "--", "."]);
    r.git(&["mv", "model/PI-PV-002.md", "model/PI-PV-002-renamed.md"]);
    r.commit("rename");
    let (j, _e, _c) = r.run(&["log", "PI-PV-002", "--json"]);
    let entries = json(&j)["entries"].as_array().unwrap().clone();
    assert!(entries.len() >= 2, "history continues past the rename: {entries:?}");
}

#[test]
fn log_outside_git_or_for_an_unknown_element_exits_one() {
    let d = std::env::temp_dir().join(format!("syscribe-prov-nogit-{}", std::process::id())).join("model");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&d).arg("log").output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("not inside a git repository"));
    let r = Repo::new();
    assert_eq!(r.run(&["log", "NOPE-001"]).2, 1);
}

// ── --expect-hash (GH #444) ──────────────────────────────────────────────────

fn hash_of(r: &Repo, id: &str) -> String {
    let (out, _e, _c) = r.run(&["show", id]);
    let line = out.lines().find(|l| l.contains("contentHash")).unwrap_or_else(|| panic!("no contentHash in:\n{out}"));
    line.split('|').nth(2).unwrap().trim().to_string()
}

#[test]
fn expect_hash_refuses_a_stale_edit_and_accepts_a_current_one() {
    let r = Repo::new();
    let h = hash_of(&r, "PI-PV-001");
    assert!(h.starts_with("blake3:"));
    // A concurrent change lands after the read.
    r.write("model/PI-PV-001.md", &r.read("model/PI-PV-001.md").replace("Body.", "Body, edited by someone else."));
    let before = r.read("model/PI-PV-001.md");
    for args in [
        vec!["set", "PI-PV-001", "status=in_progress"],
        vec!["set", "PI-PV-001", "question.add", "text=x"],
        vec!["claim", "PI-PV-001", "--by", "a1"],
        vec!["move", "PI-PV-001", "PI-PV-009"],
    ] {
        let mut a = args.clone();
        a.extend(["--expect-hash", &h]);
        let (_o, err, code) = r.run(&a);
        assert_eq!(code, 1, "{args:?}");
        assert!(err.contains("has changed since it was read"), "{args:?}: {err}");
    }
    assert_eq!(r.read("model/PI-PV-001.md"), before, "a stale edit writes nothing");
    let fresh = hash_of(&r, "PI-PV-001");
    assert_ne!(fresh, h);
    let (_o, err, code) = r.run(&["set", "PI-PV-001", "status=in_progress", "--expect-hash", &fresh]);
    assert_eq!(code, 0, "{err}");
    // And the hash moved with the write: the old one is now stale again.
    assert_eq!(r.run(&["set", "PI-PV-001", "status=blocked", "--expect-hash", &fresh]).2, 1);
}

// ── --json envelope and the exit-code contract (GH #444) ─────────────────────

#[test]
fn json_envelope_lists_what_was_written_and_the_commit() {
    let r = Repo::new();
    let (out, err, code) = r.run(&["set", "PI-PV-001", "status=in_progress", "--json", "--by", "agent-1", "--commit"]);
    assert_eq!(code, 0, "{err}");
    let v = json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["dryRun"], false);
    assert!(v["wrote"][0].as_str().unwrap().ends_with("model/PI-PV-001.md"), "{v}");
    assert!(v["commit"].as_str().is_some_and(|c| c.len() >= 7), "{v}");
    assert!(v["messages"][0].as_str().unwrap().contains("Set status"), "{v}");
    assert!(v["diff"].as_str().unwrap().contains("+status: in_progress"), "{v}");
    assert!(!out.contains("Committed"), "stdout is only the envelope");

    let (out, _e, code) = r.run(&["set", "PI-PV-001", "status=done", "--json", "--dry-run"]);
    assert_eq!(code, 0);
    let v = json(&out);
    assert_eq!(v["dryRun"], true);
    assert_eq!(v["wrote"].as_array().unwrap().len(), 0);
    assert!(v["diff"].as_str().unwrap().contains("+status: done"), "{v}");
    assert!(r.read("model/PI-PV-001.md").contains("status: in_progress"));
}

#[test]
fn json_is_unchanged_for_commands_that_are_not_writers() {
    let r = Repo::new();
    let (out, _e, code) = r.run(&["questions", "--json"]);
    assert_eq!(code, 0);
    assert!(json(&out).get("questions").is_some(), "{out}");
}

#[test]
fn the_exit_code_contract() {
    let r = Repo::new();
    // 0: success.
    assert_eq!(r.run(&["questions"]).2, 0);
    // 1: a usage error, an unknown element, a refused write.
    assert_eq!(r.run(&["questions", "--bogus"]).2, 1, "unknown option");
    assert_eq!(r.run(&["show", "NOPE-1"]).2, 1, "unknown element");
    assert_eq!(r.run(&["set", "PI-PV-001", "status=nonsense"]).2, 1, "refused value");
    assert_eq!(r.run(&["set", "PI-PV-001", "status=todo", "--expect-hash", "blake3:00"]).2, 1, "stale hash");
    // 2: a tripped gate.
    r.write("model/REQ-PV-009.md", "---\nid: REQ-PV-009\ntype: Requirement\nname: R9\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\nShall.\n");
    assert_eq!(r.run(&["validate", "--warnings-as-errors"]).2, 2, "a gated warning");
    // The documented exceptions: these exit 2 on a usage error.
    assert_eq!(r.run(&["impact"]).2, 2, "impact with no element");
}

// ── review follow-ups: every success path finishes, flags are checked, no-ops are not failures ───────────────

#[test]
fn applies_when_commits_like_every_other_writer() {
    let r = Repo::new();
    r.write("model/Feat.md", "---\nid: FEAT-PV-A\ntype: FeatureDef\nname: Feat\n---\n\nA feature.\n");
    r.commit("feature");
    let n = r.commits();
    let (out, err, code) = r.run(&["applies-when", "REQ-PV-001", "--set", "Feat", "--by", "agent-4", "--commit"]);
    assert_eq!(code, 0, "{out}\n{err}");
    assert_eq!(r.commits(), n + 1, "applies-when ends the process itself, yet it must still commit");
    assert_eq!(r.git(&["log", "-1", "--format=%an"]), "agent-4");
    assert!(r.read("model/REQ-PV-001.md").contains("appliesWhen"));
    assert_eq!(r.git(&["show", "--name-only", "--format=", "HEAD"]), "model/REQ-PV-001.md");
}

#[test]
fn a_claimant_id_stays_free_form_unless_it_is_also_the_commit_author() {
    let r = Repo::new();
    // `claim --by` is an opaque claimant id, as it always was.
    let (_o, err, code) = r.run(&["claim", "PI-PV-002", "--by", "alice@corp session 7"]);
    assert_eq!(code, 0, "{err}");
    assert!(r.read("model/PI-PV-002.md").contains("alice@corp session 7"), "{}", r.read("model/PI-PV-002.md"));
    // But as a commit author it has to be a safe name.
    r.git(&["checkout", "--", "."]);
    let n = r.commits();
    let (_o, err, code) = r.run(&["claim", "PI-PV-002", "--by", "alice@corp session 7", "--commit"]);
    assert_eq!(code, 1);
    assert!(err.contains("not a valid actor id"), "{err}");
    assert_eq!(r.commits(), n);
}

#[test]
fn a_global_flag_without_its_value_is_a_usage_error() {
    let r = Repo::new();
    let before = r.read("model/PI-PV-001.md");
    for args in [
        vec!["set", "PI-PV-001", "status=in_progress", "--reason"],
        vec!["set", "PI-PV-001", "status=in_progress", "--by"],
        vec!["set", "PI-PV-001", "status=in_progress", "--expect-hash"],
        vec!["set", "PI-PV-001", "status=in_progress", "--by", "--commit"],
    ] {
        let (_o, err, code) = r.run(&args);
        assert_eq!(code, 1, "{args:?}");
        assert!(err.contains("expects a value"), "{args:?}: {err}");
    }
    assert_eq!(r.read("model/PI-PV-001.md"), before, "nothing was written");
}

#[test]
fn expect_hash_on_a_command_that_does_not_check_it_is_refused_not_ignored() {
    let r = Repo::new();
    let (_o, err, code) = r.run(&["questions", "--expect-hash", "blake3:00"]);
    assert_eq!(code, 1);
    assert!(err.contains("--expect-hash is not supported by `questions`") && err.contains("set"), "{err}");
    let (_o, err, code) = r.run(&["suspect", "accept", "--all", "--expect-hash", "blake3:00"]);
    assert_eq!(code, 1, "{err}");
    // The commands that do check it accept it.
    let h = hash_of(&r, "PI-PV-001");
    assert_eq!(r.run(&["set", "PI-PV-001", "status=in_progress", "--expect-hash", &h]).2, 0);
}

#[test]
fn rewriting_a_file_with_identical_content_is_a_successful_no_op_not_a_failed_commit() {
    let r = Repo::new();
    r.write("model/.syscribe.toml", "[provenance]\ncommit = true\n");
    r.commit("enable");
    let (_o, err, code) = r.run(&["export-reqif", "--output", "out.reqif", "--by", "agent-1"]);
    assert_eq!(code, 0, "{err}");
    let n = r.commits();
    // Identical output again: nothing differs from HEAD, so there is nothing to commit — and nothing failed.
    let (_o, err, code) = r.run(&["export-reqif", "--output", "out.reqif", "--by", "agent-1"]);
    assert_eq!(code, 0, "an unchanged re-run must not fail: {err}");
    assert_eq!(r.commits(), n, "and must not create an empty commit");
}

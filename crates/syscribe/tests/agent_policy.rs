//! TC-TRS-AGENTPOL-001..003 (GH #442): the project's agent policy — enforced by the writers, audited from git,
//! and shown to agents.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const POLICY: &str = "[agents]\nprotected_paths = [\"model/Safety/**\"]\nhuman_only_status = { Requirement = [\"approved\"] }\nrequire_before_done = [\"evidence\"]\nactors = [\"bot-*\"]\n";

struct Repo {
    root: PathBuf,
}

impl Repo {
    fn new(policy: &str) -> Repo {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-agpol-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { root };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "human@example.com"]);
        r.git(&["config", "user.name", "Human"]);
        r.git(&["config", "commit.gpgsign", "false"]);
        r.write("model/.syscribe.toml", policy);
        r.write("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
        r.write("model/Requirements/REQ-AP-001.md", &req("REQ-AP-001", "draft"));
        r.write("model/Safety/REQ-AP-002.md", &req("REQ-AP-002", "draft"));
        r.write("model/Parts/Alpha.md", "---\ntype: PartDef\nname: Alpha\n---\n\nA.\n");
        r.write("model/Safety/Hazard.md", "---\ntype: PartDef\nname: Hazard\nsupertype: Parts::Alpha\n---\n\nH.\n");
        r.write("model/Planning/PI-AP-001.md", &pi("PI-AP-001", "todo", ""));
        r.write("model/Planning/PI-AP-002.md", &pi("PI-AP-002", "todo", "evidence:\n  - ref: REQ-AP-001\n"));
        r.commit("base");
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
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn req(id: &str, status: &str) -> String {
    format!("---\nid: {id}\ntype: Requirement\nname: {id}\nstatus: {status}\nreqDomain: software\nreqClass: system\n---\n\nShall.\n")
}
fn pi(id: &str, status: &str, extra: &str) -> String {
    format!("---\nid: {id}\ntype: PlanningItem\nname: {id}\nstatus: {status}\nachieves: [REQ-AP-001]\n{extra}---\n\nBody.\n")
}

// ── enforcement (TC-TRS-AGENTPOL-001) ────────────────────────────────────────

#[test]
fn an_agent_may_not_write_a_protected_path_but_a_person_may() {
    let r = Repo::new(POLICY);
    let before = r.read("model/Safety/REQ-AP-002.md");
    let (_o, err, code) = r.run(&["set", "REQ-AP-002", "status=review", "--by", "agent-1"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("protected_paths") && err.contains("model/Safety/REQ-AP-002.md") && err.contains("--as-human"), "{err}");
    assert_eq!(r.read("model/Safety/REQ-AP-002.md"), before, "nothing was written");

    // A person: no actor, or an explicit declaration.
    assert_eq!(r.run(&["set", "REQ-AP-002", "status=review"]).2, 0, "a person at a terminal is unaffected");
    r.git(&["checkout", "--", "."]);
    let (_o, err, code) = r.run(&["set", "REQ-AP-002", "status=review", "--by", "alice", "--as-human"]);
    assert_eq!(code, 0, "{err}");
    // An agent may write elsewhere.
    assert_eq!(r.run(&["set", "REQ-AP-001", "status=review", "--by", "agent-1"]).2, 0);
}

#[test]
fn a_dry_run_is_refused_exactly_as_a_real_run_is() {
    let r = Repo::new(POLICY);
    let (_o, err, code) = r.run(&["set", "REQ-AP-002", "status=review", "--by", "agent-1", "--dry-run"]);
    assert_eq!(code, 1, "a dry run must report what a real run would be refused: {err}");
    assert!(err.contains("protected_paths"), "{err}");
    let (_o, err, code) = r.run(&["set", "REQ-AP-001", "status=approved", "--by", "agent-1", "--dry-run"]);
    assert_eq!(code, 1, "{err}");
}

#[test]
fn a_move_that_rewrites_a_protected_file_is_refused_before_anything_is_touched() {
    let r = Repo::new(POLICY);
    // Alpha is not protected, but Safety/Hazard.md refers to it, so renaming Alpha would edit a protected file.
    let before = r.git(&["status", "--porcelain"]);
    let (_o, err, code) = r.run(&["move", "Parts::Alpha", "Parts::Beta", "--by", "agent-1"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("protected_paths") && err.contains("Hazard.md"), "{err}");
    assert_eq!(r.git(&["status", "--porcelain"]), before, "no file was moved or rewritten");
    assert!(r.root.join("model/Parts/Alpha.md").exists());
    // A person may.
    assert_eq!(r.run(&["move", "Parts::Alpha", "Parts::Beta", "--as-human", "--by", "alice"]).2, 0);
}

#[test]
fn a_human_only_status_cannot_be_set_by_an_agent() {
    let r = Repo::new(POLICY);
    let (_o, err, code) = r.run(&["set", "REQ-AP-001", "status=approved", "--by", "agent-1"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("human_only_status") && err.contains("Requirement") && err.contains("`approved`"), "{err}");
    assert!(r.read("model/Requirements/REQ-AP-001.md").contains("status: draft"));
    // Other statuses are fine; another type is unaffected; a person may.
    assert_eq!(r.run(&["set", "REQ-AP-001", "status=review", "--by", "agent-1"]).2, 0);
    assert_eq!(r.run(&["set", "REQ-AP-001", "status=approved", "--by", "alice", "--as-human"]).2, 0);
    assert!(r.read("model/Requirements/REQ-AP-001.md").contains("status: approved"));
}

#[test]
fn an_item_without_evidence_cannot_be_marked_done_by_an_agent() {
    let r = Repo::new(POLICY);
    let (_o, err, code) = r.run(&["set", "PI-AP-001", "status=done", "--by", "agent-1"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("require_before_done") && err.contains("evidence") && err.contains("E719"), "{err}");
    assert!(r.read("model/Planning/PI-AP-001.md").contains("status: todo"));
    // With evidence it goes through; a person is never held to it.
    let (_o, err, code) = r.run(&["set", "PI-AP-002", "status=done", "--by", "agent-1"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(r.run(&["set", "PI-AP-001", "status=done"]).2, 0);
}

#[test]
fn require_before_done_validate_refuses_a_change_that_introduces_an_error() {
    let r = Repo::new("[agents]\nrequire_before_done = [\"validate\"]\n");
    // Marking an evidence-less leaf done introduces E719: an error the change would add.
    let (_o, err, code) = r.run(&["set", "PI-AP-001", "status=done", "--by", "agent-1"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("validate") && err.contains("introduces 1 error"), "{err}");
    assert_eq!(r.run(&["set", "PI-AP-002", "status=done", "--by", "agent-1"]).2, 0);
}

#[test]
fn require_by_refuses_a_writer_that_names_no_actor() {
    let r = Repo::new("[agents]\nrequire_by = true\n");
    let before = r.read("model/Requirements/REQ-AP-001.md");
    let (_o, err, code) = r.run(&["set", "REQ-AP-001", "status=review"]);
    assert_eq!(code, 1);
    assert!(err.contains("require_by") && err.contains("--by") && err.contains("--as-human"), "{err}");
    assert_eq!(r.read("model/Requirements/REQ-AP-001.md"), before);
    assert_eq!(r.run(&["set", "REQ-AP-001", "status=review", "--by", "agent-1"]).2, 0);
    r.git(&["checkout", "--", "."]);
    assert_eq!(r.run(&["set", "REQ-AP-001", "status=review", "--as-human"]).2, 0);
    // Reads are never refused, and a command that does not write the model is not a model write.
    assert_eq!(r.run(&["show", "REQ-AP-001"]).2, 0);
    assert_eq!(r.run(&["validate"]).2, 0);
    assert_eq!(r.run(&["applies-when", "REQ-AP-001"]).2, 0, "the read form of applies-when");
}

#[test]
fn the_write_layer_backstops_commands_that_have_no_check_of_their_own() {
    let r = Repo::new(POLICY);
    let (_o, err, code) = r.run(&["export-reqif", "--output", "model/Safety/out.reqif", "--by", "agent-1"]);
    assert_ne!(code, 0, "{err}");
    assert!(err.contains("protected"), "{err}");
    assert!(!r.root.join("model/Safety/out.reqif").exists());
    // Elsewhere, and for a person, it writes.
    assert_eq!(r.run(&["export-reqif", "--output", "out.reqif", "--by", "agent-1"]).2, 0);
    assert_eq!(r.run(&["export-reqif", "--output", "model/Safety/out.reqif"]).2, 0);
}

#[test]
fn as_human_is_recorded_in_the_commit() {
    let r = Repo::new(POLICY);
    let (_o, err, code) = r.run(&["set", "REQ-AP-001", "status=approved", "--by", "alice", "--as-human", "--commit"]);
    assert_eq!(code, 0, "{err}");
    let msg = r.git(&["log", "-1", "--format=%B"]);
    assert!(msg.contains("Syscribe-Actor: alice") && msg.contains("Syscribe-As: human"), "{msg}");
}

// ── audit (TC-TRS-AGENTPOL-002) ──────────────────────────────────────────────

fn commit_as(r: &Repo, author: &str, msg: &str) {
    r.git(&["add", "-A"]);
    let o = Command::new("git").arg("-C").arg(&r.root).args(["-c", &format!("user.name={author}"), "commit", "-q", "-m", msg]).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}

fn findings(r: &Repo, code: &str) -> Vec<String> {
    r.run(&["validate"]).0.lines().filter(|l| l.contains(&format!("| {code} |"))).map(String::from).collect()
}

#[test]
fn validate_reports_agent_commits_that_bypassed_the_writers() {
    let r = Repo::new(POLICY);
    // (1) Tool-attributed: a Syscribe-Actor trailer, but the change was made by hand (the writers did not see it).
    r.write("model/Safety/REQ-AP-002.md", &req("REQ-AP-002", "review"));
    commit_as(&r, "Human", "edit safety\n\nSyscribe-Command: hand edit\nSyscribe-Actor: agent-9");
    // (2) An agent author (matches `actors`) sets a human-only status.
    r.write("model/Requirements/REQ-AP-001.md", &req("REQ-AP-001", "approved"));
    commit_as(&r, "bot-7", "approve");

    let w760 = findings(&r, "W760");
    assert_eq!(w760.len(), 1, "{w760:?}");
    assert!(w760[0].contains("model/Safety/REQ-AP-002.md") && w760[0].contains("agent-9") && w760[0].contains("protected_paths"), "{}", w760[0]);
    let w761 = findings(&r, "W761");
    assert_eq!(w761.len(), 1, "{w761:?}");
    assert!(w761[0].contains("REQ-AP-001.md") && w761[0].contains("`approved`") && w761[0].contains("bot-7"), "{}", w761[0]);

    // They are warnings: exit 0 by default, gated by --deny.
    assert_eq!(r.run(&["validate"]).2, 0);
    assert_eq!(r.run(&["validate", "--deny", "W760"]).2, 2);
}

#[test]
fn people_and_declared_humans_raise_nothing() {
    let r = Repo::new(POLICY);
    r.write("model/Safety/REQ-AP-002.md", &req("REQ-AP-002", "review"));
    commit_as(&r, "Human", "a person edits the safety folder");
    r.write("model/Requirements/REQ-AP-001.md", &req("REQ-AP-001", "approved"));
    commit_as(&r, "bot-7", "approve, but a person told it to\n\nSyscribe-Command: set\nSyscribe-Actor: alice\nSyscribe-As: human");
    assert!(findings(&r, "W760").is_empty() && findings(&r, "W761").is_empty());
}

#[test]
fn a_malformed_entry_is_reported_and_the_rest_of_the_policy_still_applies() {
    let r = Repo::new("[agents]\nprotected_paths = [\"model/Safety/**\"]\nrequire_by = \"yes\"\nhuman_only_status = { Requirement = [\"aproved\"] }\nbogus = 1\n");
    let w762 = findings(&r, "W762");
    assert_eq!(w762.len(), 3, "{w762:?}");
    assert!(w762.iter().any(|l| l.contains("`aproved` is not a Requirement status")), "a typo'd status is caught: {w762:?}");
    assert!(w762.iter().any(|l| l.contains("unknown key `bogus`")));
    // The good entry is intact.
    let (_o, err, code) = r.run(&["set", "REQ-AP-002", "status=review", "--by", "agent-1"]);
    assert_eq!(code, 1, "{err}");
}

// ── showing it (TC-TRS-AGENTPOL-003) ─────────────────────────────────────────

#[test]
fn the_policy_command_states_the_rules_and_how_this_call_is_treated() {
    let r = Repo::new(POLICY);
    let (out, _e, code) = r.run(&["policy"]);
    assert_eq!(code, 0);
    assert!(out.contains("`model/Safety/**`") && out.contains("Requirement to `approved`") && out.contains("resolving evidence"), "{out}");
    assert!(out.contains("treated as a person (no --by)"), "{out}");
    let (out, _e, _c) = r.run(&["policy", "--by", "agent-1"]);
    assert!(out.contains("treated as an agent"), "{out}");
    let (out, _e, _c) = r.run(&["policy", "--by", "agent-1", "--as-human"]);
    assert!(out.contains("treated as a person (--as-human)"), "{out}");
    let (j, _e, _c) = r.run(&["policy", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["configured"], true);
    assert_eq!(v["protectedPaths"][0], "model/Safety/**");
    assert_eq!(v["humanOnlyStatus"]["Requirement"][0], "approved");

    let none = Repo::new("[ids]\nmax_digits = 4\n");
    assert!(none.run(&["policy"]).0.contains("No [agents] policy is configured"));
    assert_eq!(json(&none.run(&["policy", "--json"]).0)["configured"], false);
}

fn json(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or_else(|e| panic!("{e}: {s}"))
}

#[test]
fn agent_guide_states_the_policy_only_when_there_is_one() {
    let r = Repo::new(POLICY);
    let (out, _e, code) = r.run(&["agent-guide"]);
    assert_eq!(code, 0);
    assert!(out.contains("### Project policy") && out.contains("`model/Safety/**`") && out.contains("--as-human"), "{out}");
    assert!(r.run(&["agent-guide", "--sections", "ids"]).0.find("Project policy").is_none(), "only when the section is asked for");
    let none = Repo::new("[ids]\nmax_digits = 4\n");
    assert!(!none.run(&["agent-guide"]).0.contains("Project policy"));
}

// ── review follow-ups ────────────────────────────────────────────────────────

#[test]
fn the_audit_understands_a_model_relative_protected_glob_like_the_writers_do() {
    let r = Repo::new("[agents]\nprotected_paths = [\"Safety/**\"]\nactors = [\"bot-*\"]\n");
    r.write("model/Safety/REQ-AP-002.md", &req("REQ-AP-002", "review"));
    commit_as(&r, "bot-7", "touch the safety folder");
    // The writers refuse `model/Safety/…` because `Safety/**` matches its model-relative spelling; so must the audit.
    let (_o, err, code) = r.run(&["set", "REQ-AP-002", "status=draft", "--by", "agent-1"]);
    assert_eq!(code, 1, "{err}");
    let w760 = findings(&r, "W760");
    assert_eq!(w760.len(), 1, "{w760:?}");
    assert!(w760[0].contains("model/Safety/REQ-AP-002.md") && w760[0].contains("`Safety/**`"), "{}", w760[0]);
}

#[test]
fn only_files_whose_diff_adds_a_reserved_status_are_read_by_the_audit() {
    let r = Repo::new(POLICY);
    // Many files changed by an agent, one of which gains a reserved status: exactly that one is found.
    for i in 0..12 {
        r.write(&format!("model/Requirements/REQ-AP-1{i:02}.md"), &req(&format!("REQ-AP-1{i:02}"), "draft"));
    }
    r.commit("many requirements");
    for i in 0..12 {
        let st = if i == 7 { "approved" } else { "review" };
        r.write(&format!("model/Requirements/REQ-AP-1{i:02}.md"), &req(&format!("REQ-AP-1{i:02}"), st));
    }
    commit_as(&r, "bot-3", "bulk status change");
    let w761 = findings(&r, "W761");
    assert_eq!(w761.len(), 1, "{w761:?}");
    assert!(w761[0].contains("REQ-AP-107.md"), "{}", w761[0]);
}

#[test]
fn require_by_is_enforced_in_the_write_layer_for_commands_nobody_listed() {
    let r = Repo::new("[agents]\nrequire_by = true\n");
    // `export-reqif` is not in the list of known model writers, but writing into the model directory is a model write.
    // The tool's own regenerable cache is not a model write.
    assert_eq!(r.run(&["summarize"]).2, 0);
    // An export into the model directory is a write to the model: refused without an actor, allowed with one.
    let (_o, err, code) = r.run(&["export-reqif", "--output", "model/out.reqif"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("require_by"), "{err}");
    assert!(!r.root.join("model/out.reqif").exists());
    assert_eq!(r.run(&["export-reqif", "--output", "model/out.reqif", "--by", "agent-1"]).2, 0);
    // Outside the model it is not a model write.
    assert_eq!(r.run(&["export-reqif", "--output", "elsewhere.reqif"]).2, 0);
}

#[test]
fn a_dry_run_move_is_refused_like_the_real_one() {
    let r = Repo::new(POLICY);
    let (_o, err, code) = r.run(&["move", "Parts::Alpha", "Parts::Beta", "--by", "agent-1", "--dry-run"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("protected_paths") && err.contains("Hazard.md"), "{err}");
}

#[test]
fn the_guide_tells_agents_to_pass_by_unless_it_is_mandatory() {
    let r = Repo::new(POLICY);
    assert!(r.run(&["agent-guide"]).0.contains("always pass `--by <you>`"));
    let mandatory = Repo::new("[agents]\nrequire_by = true\nprotected_paths = [\"model/Safety/**\"]\n");
    let out = mandatory.run(&["agent-guide"]).0;
    assert!(out.contains("Every model-writing command must name its actor") && !out.contains("always pass `--by <you>`"), "{out}");
}

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

// ── questions reserved to a person (TC-TRS-AGENTPOL-004, GH #457) ───────────

fn pi_q(id: &str, q: &str) -> String {
    pi(id, "todo", &format!("questions:\n  - id: Q1\n    text: Which UART?\n    status: open\n{q}"))
}

#[test]
fn human_only_answers_refuses_an_agent_answer_also_under_dry_run() {
    let r = Repo::new("[agents]\nhuman_only_answers = true\nactors = [\"bot-*\"]\n");
    r.write("model/Planning/PI-AP-003.md", &pi_q("PI-AP-003", ""));
    r.commit("question");
    let before = r.read("model/Planning/PI-AP-003.md");
    for extra in [&[][..], &["--dry-run"][..]] {
        let mut args = vec!["set", "PI-AP-003", "question.answer", "Q1", "text=UART2", "--by", "agent-1"];
        args.extend_from_slice(extra);
        let (_o, err, code) = r.run(&args);
        assert_eq!(code, 1, "{extra:?}: {err}");
        assert!(err.contains("human_only_answers") && err.contains("PI-AP-003#Q1") && err.contains("--as-human"), "{err}");
    }
    assert_eq!(r.read("model/Planning/PI-AP-003.md"), before, "nothing was written");
    // An agent may still record a question; a person may answer.
    assert_eq!(r.run(&["set", "PI-AP-003", "question.add", "text=Baud?", "--by", "agent-1"]).2, 0);
    let (_o, err, code) = r.run(&["set", "PI-AP-003", "question.answer", "Q1", "text=UART2", "--by", "alice", "--as-human"]);
    assert_eq!(code, 0, "{err}");
    assert!(r.read("model/Planning/PI-AP-003.md").contains("answer: UART2"));
    assert_eq!(r.run(&["set", "PI-AP-003", "question.answer", "Q2", "text=115200"]).2, 0, "no --by: a person at a terminal");
}

#[test]
fn answer_by_on_the_question_decides_and_wins_over_the_policy_default() {
    // No [agents] table at all: `answerBy: human` still reserves that one question.
    let r = Repo::new("[ids]\nmax_digits = 4\n");
    r.write("model/Planning/PI-AP-003.md", &pi_q("PI-AP-003", "    answerBy: human\n"));
    r.write("model/Planning/PI-AP-004.md", &pi_q("PI-AP-004", ""));
    r.commit("questions");
    let (_o, err, code) = r.run(&["set", "PI-AP-003", "question.answer", "Q1", "text=x", "--by", "agent-1"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("human_only_answers") && err.contains("answerBy: human"), "{err}");
    assert_eq!(r.run(&["set", "PI-AP-004", "question.answer", "Q1", "text=x", "--by", "agent-1"]).2, 0, "unreserved");
    // With the policy on, `answerBy: any` opens one question to agents.
    let r = Repo::new("[agents]\nhuman_only_answers = true\n");
    r.write("model/Planning/PI-AP-003.md", &pi_q("PI-AP-003", "    answerBy: any\n"));
    r.commit("q");
    assert_eq!(r.run(&["set", "PI-AP-003", "question.answer", "Q1", "text=x", "--by", "agent-1"]).2, 0);
    // A bad value is E724 and counts as `human`.
    r.write("model/Planning/PI-AP-004.md", &pi_q("PI-AP-004", "    answerBy: owner\n"));
    assert!(findings(&r, "E724").iter().any(|l| l.contains("answerBy `owner`")), "{:?}", findings(&r, "E724"));
    assert_eq!(r.run(&["set", "PI-AP-004", "question.answer", "Q1", "text=x", "--by", "agent-1"]).2, 1);
}

fn answered(id: &str, by: &str, answer_by: &str) -> String {
    pi(id, "todo", &format!("questions:\n  - id: Q1\n    text: Which UART?\n    status: answered\n    answer: UART2\n    answeredBy: {by}\n{answer_by}"))
}

#[test]
fn validate_flags_a_reserved_answer_an_agent_wrote() {
    let r = Repo::new("[agents]\nhuman_only_answers = true\nactors = [\"bot-*\"]\n");
    r.write("model/Planning/PI-AP-003.md", &pi_q("PI-AP-003", ""));
    r.write("model/Planning/PI-AP-004.md", &pi_q("PI-AP-004", ""));
    r.write("model/Planning/PI-AP-005.md", &pi_q("PI-AP-005", ""));
    r.write("model/Planning/PI-AP-006.md", &pi_q("PI-AP-006", "    answerBy: any\n"));
    r.commit("questions");
    // (1) A commit attributed to an agent (Syscribe-Actor trailer) wrote the answer, claiming the owner answered.
    r.write("model/Planning/PI-AP-003.md", &answered("PI-AP-003", "owner", ""));
    commit_as(&r, "Human", "answer\n\nSyscribe-Command: hand edit\nSyscribe-Actor: agent-9");
    // (2) A person wrote this one.
    r.write("model/Planning/PI-AP-004.md", &answered("PI-AP-004", "owner", ""));
    commit_as(&r, "Human", "owner answers");
    // (3) `answeredBy` itself names an agent.
    r.write("model/Planning/PI-AP-005.md", &answered("PI-AP-005", "bot-2", ""));
    commit_as(&r, "Human", "pasted");
    // (4) Open to agents: an agent answer is fine.
    r.write("model/Planning/PI-AP-006.md", &answered("PI-AP-006", "bot-2", "    answerBy: any\n"));
    commit_as(&r, "bot-2", "agent answers an open question");

    let w323 = findings(&r, "W323");
    assert_eq!(w323.len(), 2, "{w323:?}");
    assert!(w323.iter().any(|l| l.contains("PI-AP-003#Q1") && l.contains("agent-9")), "{w323:?}");
    assert!(w323.iter().any(|l| l.contains("PI-AP-005#Q1") && l.contains("bot-2")), "{w323:?}");
    assert_eq!(r.run(&["validate"]).2, 0, "a warning");
    assert_eq!(r.run(&["validate", "--deny", "W323"]).2, 2);

    // An answer not yet committed cannot be attributed: nothing is said about it.
    r.write("model/Planning/PI-AP-004.md", &answered("PI-AP-004", "carol", ""));
    assert!(!findings(&r, "W323").iter().any(|l| l.contains("PI-AP-004")));

    // questions --json: answerBy and the answer's provenance.
    let v = json(&r.run(&["questions", "--json"]).0);
    let q = |item: &str| v["questions"].as_array().unwrap().iter().find(|q| q["item"] == item).cloned().unwrap();
    assert_eq!(q("PI-AP-003")["answerBy"], "human");
    assert_eq!(q("PI-AP-006")["answerBy"], "any");
    assert_eq!(q("PI-AP-003")["answerProvenance"]["by"], "agent-9");
    assert_eq!(q("PI-AP-003")["answerProvenance"]["agent"], true);
    assert_eq!(q("PI-AP-006")["answerProvenance"]["agent"], true, "bot-2 matches [agents] actors");
    assert!(q("PI-AP-003")["answerProvenance"]["commit"].is_string() && q("PI-AP-003")["answerProvenance"]["at"].is_string());
    assert!(q("PI-AP-004").get("answerProvenance").is_none(), "uncommitted: unknown");
    // The text table marks the reserved ones.
    assert!(r.run(&["questions"]).0.contains("| PI-AP-003#Q1 *(for a person)* |"));
}

#[test]
fn without_a_policy_only_answer_by_human_questions_are_audited() {
    let r = Repo::new("[ids]\nmax_digits = 4\n");
    r.write("model/Planning/PI-AP-003.md", &answered("PI-AP-003", "owner", "    answerBy: human\n"));
    r.write("model/Planning/PI-AP-004.md", &answered("PI-AP-004", "owner", ""));
    commit_as(&r, "Human", "answers\n\nSyscribe-Command: set\nSyscribe-Actor: agent-1");
    let w323 = findings(&r, "W323");
    assert_eq!(w323.len(), 1, "{w323:?}");
    assert!(w323[0].contains("PI-AP-003#Q1"), "{}", w323[0]);
}

#[test]
fn the_policy_states_human_only_answers() {
    let r = Repo::new("[agents]\nhuman_only_answers = true\n");
    assert!(r.run(&["policy"]).0.contains("answerBy: any"));
    assert_eq!(json(&r.run(&["policy", "--json"]).0)["humanOnlyAnswers"], true);
    assert!(r.run(&["agent-guide"]).0.contains("Only a person answers a PlanningItem question"));
}

// ── the evidence sidecar (TC-TRS-AGENTPOL-005, GH #469) ──────────────────────

fn sidecar_repo(policy: &str) -> Repo {
    let r = Repo::new(policy);
    r.write("model/Verification/TC-AP-001.md", "---\nid: TC-AP-001\ntype: TestCase\nname: one\nstatus: active\ntestLevel: L1\nverifies: [REQ-AP-001]\ntestFunctions:\n  - function: t_one\n---\n\nScenario.\n");
    r.write("report.xml", "<testsuite name=\"s\"><testcase classname=\"c\" name=\"t_one\"/></testsuite>");
    r.commit("tc");
    r
}

#[test]
fn a_protected_sidecar_refuses_ingest_with_exit_1_and_the_rule_named() {
    let r = sidecar_repo("[agents]\nprotected_paths = [\".syscribe/**\"]\n");
    let (_o, err, code) = r.run(&["ingest-results", "report.xml", "--by", "agent-1"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("protected_paths") && err.contains(".syscribe/results.json"), "{err}");
    assert!(!r.model().join(".syscribe/results.json").exists(), "nothing was written");
}

#[test]
fn sidecar_writers_lets_only_the_named_command_write_the_protected_sidecar() {
    let r = sidecar_repo("[agents]\nprotected_paths = [\".syscribe/**\"]\nsidecar_writers = [\"ingest-results\"]\n");
    let (_o, err, code) = r.run(&["ingest-results", "report.xml", "--by", "agent-1"]);
    assert_eq!(code, 0, "{err}");
    assert!(r.model().join(".syscribe/results.json").exists());
    // The policy says so in words, so an agent knows to ingest rather than edit.
    let (out, _, _) = r.run(&["policy"]);
    assert!(out.contains("`ingest-results` may write the evidence sidecar"), "{out}");
    // Another command is not named, so the same path stays protected for it.
    let other = sidecar_repo("[agents]\nprotected_paths = [\".syscribe/**\"]\nsidecar_writers = [\"set\"]\n");
    let (_o, err, code) = other.run(&["ingest-results", "report.xml", "--by", "agent-1"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("protected_paths"), "{err}");
    // A malformed entry is W762, never a silent grant.
    let bad = sidecar_repo("[agents]\nprotected_paths = [\".syscribe/**\"]\nsidecar_writers = \"ingest-results\"\n");
    assert_eq!(bad.run(&["ingest-results", "report.xml", "--by", "agent-1"]).2, 1, "a string is not a list");
}

// ── the audit and the evidence sidecar (TC-TRS-AGENTPOL-005, GH #469) ────────────────────────────────────────

/// An agent's commit of the sidecar an ingest wrote, as `ingest-results --commit` makes it.
fn agent_commit_of_a_sidecar(policy: &str) -> Repo {
    let r = sidecar_repo(policy);
    let (_o, err, code) = r.run(&["ingest-results", "report.xml", "--by", "agent-1", "--reason", "ci", "--commit"]);
    assert_eq!(code, 0, "{err}");
    assert!(r.git(&["show", "--stat", "--format=%an", "HEAD"]).contains(".syscribe/results.json"), "the ingest was committed");
    r
}

#[test]
fn the_audit_does_not_flag_a_sidecar_a_listed_command_wrote() {
    let r = agent_commit_of_a_sidecar("[agents]\nprotected_paths = [\".syscribe/**\"]\nsidecar_writers = [\"ingest-results\"]\nactors = [\"agent-*\"]\n");
    assert!(findings(&r, "W760").is_empty(), "{:?}", findings(&r, "W760"));
}

#[test]
fn without_sidecar_writers_the_same_commit_is_still_flagged_and_other_protected_paths_always_are() {
    // No writer is named, so the ingest cannot even run for an agent; the sidecar arrives by a person's hand and
    // an agent commits it: that is what the audit exists to report.
    let r = sidecar_repo("[agents]\nprotected_paths = [\".syscribe/**\", \"Safety/**\"]\nactors = [\"agent-*\"]\n");
    r.write("model/.syscribe/results.json", "{}");
    commit_as(&r, "agent-1", "commit the sidecar");
    let w = findings(&r, "W760");
    assert!(w.iter().any(|l| l.contains(".syscribe/results.json")), "{w:?}");
    // With a writer named, a protected path that is not the sidecar is still reported, in the same commit.
    let s = sidecar_repo("[agents]\nprotected_paths = [\".syscribe/**\", \"Safety/**\"]\nsidecar_writers = [\"ingest-results\"]\nactors = [\"agent-*\"]\n");
    s.write("model/.syscribe/results.json", "{}");
    s.write("model/Safety/REQ-AP-002.md", &req("REQ-AP-002", "review"));
    commit_as(&s, "agent-1", "sidecar and a safety file");
    let w = findings(&s, "W760");
    assert!(w.iter().any(|l| l.contains("Safety/REQ-AP-002.md")), "{w:?}");
    assert!(!w.iter().any(|l| l.contains(".syscribe/results.json")), "the sidecar is exempt, the safety file is not: {w:?}");
    // The scripts directory is tooling, not evidence: never exempt.
    s.write("model/.syscribe/scripts/x.rhai", "1");
    commit_as(&s, "agent-1", "a script");
    assert!(findings(&s, "W760").iter().any(|l| l.contains(".syscribe/scripts/x.rhai")));
}

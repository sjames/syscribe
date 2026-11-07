//! TC-TRS-AMEND-001..004 (GH #454): amendment records on accepted ADRs and confirmed requirements —
//! the `amendments:` field (E330), edits without an amendment (W324), scope changes without an owner
//! reference (W325), and the surfaces (`set amendment.add`, `show`, `export-html`, suspect links,
//! template and help).

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const REQUIRE: &str = "[traceability]\nrequire_amendments = true\n";

struct Repo {
    root: PathBuf,
}

impl Repo {
    /// A model in a fresh git repository (`git` true) or a plain directory, with `.syscribe.toml` = `config`.
    fn new(config: &str, git: bool) -> Repo {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-amend-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { root };
        if git {
            r.git(&["init", "-q", "-b", "main"]);
            r.git(&["config", "user.email", "owner@example.com"]);
            r.git(&["config", "user.name", "Owner"]);
            r.git(&["config", "commit.gpgsign", "false"]);
        }
        r.write("model/.syscribe.toml", config);
        r.write("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
        r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "We use protocol X for message A.", ""));
        r.write("model/Requirements/REQ-AM-001.md", &req("REQ-AM-001", "approved", "The system shall send A.", ""));
        r.write("model/Parts/Alpha.md", "---\ntype: PartDef\nname: Alpha\n---\n\nA.\n");
        r.write(
            "model/Planning/PI-AM-001.md",
            "---\nid: PI-AM-001\ntype: PlanningItem\nname: Patch\nstatus: todo\nachieves: [REQ-AM-001]\nquestions:\n  - id: Q1\n    text: Cover message B too?\n    status: answered\n    answer: yes\n  - id: Q2\n    text: And message C?\n    status: open\n---\n\nWork.\n",
        );
        r.write(
            "model/Reviews/RR-AM-001.md",
            "---\ntype: ReviewRecord\nid: RR-AM-001\nname: Decision review\nstatus: closed\nreviewType: design_review\nreviewDate: \"2026-01-01\"\nreviews:\n  - ADR-AM-001\n---\n\nReviewed.\n",
        );
        if git {
            r.commit("base");
        }
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
    /// The `validate` lines mentioning `code`.
    fn findings(&self, code: &str) -> Vec<String> {
        let (out, err, _) = self.run(&["validate"]);
        out.lines().chain(err.lines()).filter(|l| l.contains(code)).map(String::from).collect()
    }
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn adr(status: &str, body: &str, amendments: &str) -> String {
    format!("---\ntype: ADR\nid: ADR-AM-001\nname: Message protocol\nstatus: {status}\n{amendments}---\n\n## Decision\n\n{body}\n")
}
fn req(id: &str, status: &str, body: &str, extra: &str) -> String {
    format!("---\nid: {id}\ntype: Requirement\nname: {id}\nstatus: {status}\nreqDomain: software\nreqClass: system\n{extra}---\n\n{body}\n")
}
fn amendment(date: &str, kind: &str, r: Option<&str>) -> String {
    let r = r.map(|r| format!("    ref: {r}\n")).unwrap_or_default();
    format!("amendments:\n  - date: {date}\n    by: owner\n    reason: covers message B too\n{r}    kind: {kind}\n")
}
fn today() -> String {
    // The tool dates working-tree edits in UTC; ask git for the same day.
    let o = Command::new("date").args(["-u", "+%Y-%m-%d"]).output().unwrap();
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

// ── the field (TC-TRS-AMEND-001) ─────────────────────────────────────────────

#[test]
fn a_well_formed_amendment_list_is_clean() {
    let r = Repo::new("", true);
    r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "X.", &amendment("2026-03-01", "clarification", Some("RR-AM-001"))));
    assert!(r.findings("E330").is_empty(), "{:?}", r.findings("E330"));
    assert!(r.findings("W047").is_empty());
    // A Requirement hosts amendments too.
    r.write("model/Requirements/REQ-AM-001.md", &req("REQ-AM-001", "approved", "S.", &amendment("2026-03-01", "clarification", None)));
    assert!(r.findings("E330").is_empty());
}

#[test]
fn malformed_amendments_are_e330() {
    let r = Repo::new("", true);
    for bad in [
        "amendments: just text\n",
        "amendments:\n  - date: 2026-03-01\n    by: owner\n    kind: clarification\n",
        "amendments:\n  - date: 2026-03-01\n    by: owner\n    reason: r\n    kind: tweak\n",
        "amendments:\n  - date: 03/01/2026\n    by: owner\n    reason: r\n    kind: clarification\n",
        "amendments:\n  - date: 2026-03-01\n    by: owner\n    reason: r\n    kind: clarification\n    ref: ADR-NOPE-999\n",
        "amendments:\n  - date: 2026-03-01\n    by: owner\n    reason: r\n    kind: clarification\n    ref: PI-AM-001#Q9\n",
        "amendments:\n  - date: 2026-03-01\n    by: owner\n    reason: r\n    kind: clarification\n    ref: 0123456789abcdef\n",
    ] {
        r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "X.", bad));
        assert_eq!(r.findings("E330").len(), 1, "{bad}: {:?}", r.findings("E330"));
    }
}

#[test]
fn refs_resolve_to_questions_elements_and_commits() {
    let r = Repo::new("", true);
    let head = r.git(&["rev-parse", "HEAD"]);
    for good in ["PI-AM-001#Q1", "RR-AM-001", "REQ-AM-001", &head[..10]] {
        r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "X.", &amendment("2026-03-01", "clarification", Some(good))));
        assert!(r.findings("E330").is_empty(), "{good}: {:?}", r.findings("E330"));
    }
}

#[test]
fn amendments_on_another_type_are_w047() {
    let r = Repo::new("", true);
    r.write("model/Parts/Alpha.md", &format!("---\ntype: PartDef\nname: Alpha\n{}---\n\nA.\n", amendment("2026-03-01", "clarification", None)));
    let w = r.findings("W047");
    assert!(w.iter().any(|l| l.contains("amendments")), "{w:?}");
    assert!(r.findings("E330").is_empty());
}

// ── edits without an amendment (TC-TRS-AMEND-002) ────────────────────────────

#[test]
fn an_uncommitted_edit_of_an_accepted_adr_needs_an_amendment() {
    let r = Repo::new(REQUIRE, true);
    assert!(r.findings("W324").is_empty());
    r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "We use protocol X for messages A and B.", ""));
    let w = r.findings("W324");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("ADR-AM-001") && w[0].contains("uncommitted"), "{w:?}");
    // An amendment dated today records it.
    r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "We use protocol X for messages A and B.", &amendment(&today(), "clarification", None)));
    assert!(r.findings("W324").is_empty(), "{:?}", r.findings("W324"));
}

#[test]
fn a_committed_edit_needs_an_amendment_dated_on_or_after_it() {
    let r = Repo::new(REQUIRE, true);
    r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "We use protocol X for messages A and B.", &amendment("2000-01-01", "clarification", None)));
    r.commit("widen the decision");
    let sha = r.git(&["rev-parse", "--short=10", "HEAD"]);
    let w = r.findings("W324");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains(&sha), "names the changing commit: {w:?}");
    r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "We use protocol X for messages A and B.", &amendment(&today(), "clarification", None)));
    assert!(r.findings("W324").is_empty());
}

#[test]
fn edits_before_acceptance_status_only_and_amendment_only_changes_are_not_flagged() {
    let r = Repo::new(REQUIRE, true);
    // Re-run the life cycle: proposed, edited while proposed, then accepted.
    r.write("model/Decisions/ADR-AM-001.md", &adr("proposed", "Draft.", ""));
    r.commit("propose");
    r.write("model/Decisions/ADR-AM-001.md", &adr("proposed", "We use protocol X.", ""));
    r.commit("edit while proposed");
    r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "We use protocol X.", ""));
    r.commit("accept");
    assert!(r.findings("W324").is_empty(), "{:?}", r.findings("W324"));
    // Adding an amendment entry alone is not a normative change.
    r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "We use protocol X.", &amendment("2026-01-01", "clarification", None)));
    assert!(r.findings("W324").is_empty(), "{:?}", r.findings("W324"));
    // Accepting in the working tree: nothing has changed since.
    r.write("model/Requirements/REQ-AM-002.md", &req("REQ-AM-002", "draft", "S.", ""));
    r.commit("new requirement");
    r.write("model/Requirements/REQ-AM-002.md", &req("REQ-AM-002", "approved", "S, reworded.", ""));
    assert!(r.findings("W324").is_empty(), "{:?}", r.findings("W324"));
}

#[test]
fn a_confirmed_requirement_is_guarded_too() {
    let r = Repo::new(REQUIRE, true);
    r.write("model/Requirements/REQ-AM-001.md", &req("REQ-AM-001", "approved", "The system shall send A and B.", ""));
    let w = r.findings("W324");
    assert!(w.iter().any(|l| l.contains("REQ-AM-001")), "{w:?}");
    // Moving on to `implemented` keeps the guard from the approval, not from the newer status.
    r.commit("edit");
    r.write("model/Requirements/REQ-AM-001.md", &req("REQ-AM-001", "implemented", "The system shall send A and B.", ""));
    r.commit("implemented");
    assert!(r.findings("W324").iter().any(|l| l.contains("REQ-AM-001")));
}

#[test]
fn w324_is_opt_in_and_needs_git() {
    let off = Repo::new("", true);
    off.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "Changed.", ""));
    assert!(off.findings("W324").is_empty());
    let nogit = Repo::new(REQUIRE, false);
    nogit.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "Changed.", ""));
    assert!(nogit.findings("W324").is_empty());
    let bad = Repo::new("[traceability]\nrequire_amendments = \"yes\"\n", true);
    assert!(bad.findings("W632").iter().any(|l| l.contains("require_amendments")));
}

// ── scope changes need an owner reference (TC-TRS-AMEND-003) ─────────────────

#[test]
fn a_scope_change_needs_an_owner_reference() {
    let r = Repo::new("[agents]\nactors = [\"bot-*\"]\n", true);
    let w325 = |r: &Repo, reference: Option<&str>| {
        r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "X.", &amendment("2026-03-01", "scope-change", reference)));
        r.findings("W325")
    };
    assert_eq!(w325(&r, None).len(), 1);
    assert!(w325(&r, Some("PI-AM-001#Q2"))[0].contains("not answered"));
    assert!(w325(&r, Some("PI-AM-001#Q1")).is_empty());
    assert!(w325(&r, Some("RR-AM-001")).is_empty());
    assert!(w325(&r, Some("REQ-AM-001"))[0].contains("not a ReviewRecord"));
    // A person's commit is an owner reference; an agent's is not.
    let human = r.git(&["rev-parse", "HEAD"]);
    assert!(w325(&r, Some(&human[..12])).is_empty());
    r.write("model/Parts/Beta.md", "---\ntype: PartDef\nname: Beta\n---\n\nB.\n");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "agent edit\n\nSyscribe-Actor: bot-1"]);
    let agent = r.git(&["rev-parse", "HEAD"]);
    assert!(w325(&r, Some(&agent[..12]))[0].contains("agent"), "{:?}", w325(&r, Some(&agent[..12])));
    // A clarification needs none.
    r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "X.", &amendment("2026-03-01", "clarification", None)));
    assert!(r.findings("W325").is_empty());
}

// ── surfaces (TC-TRS-AMEND-004) ──────────────────────────────────────────────

#[test]
fn set_amendment_add_fills_date_and_by_and_checks_its_values() {
    let r = Repo::new("", true);
    let (_o, err, code) = r.run(&["set", "ADR-AM-001", "amendment.add", "kind=scope-change", "reason=covers message B too", "ref=PI-AM-001#Q1", "--by", "alice"]);
    assert_eq!(code, 0, "{err}");
    let text = r.read("model/Decisions/ADR-AM-001.md");
    assert!(text.contains(&format!("date: {}", today())) || text.contains(&format!("date: '{}'", today())) || text.contains(&format!("date: \"{}\"", today())), "{text}");
    assert!(text.contains("by: alice") && text.contains("kind: scope-change") && text.contains("ref: PI-AM-001#Q1"), "{text}");
    assert!(r.findings("E330").is_empty() && r.findings("W325").is_empty());
    // Refusals: bad kind, unresolved ref, no author, a type without amendments.
    let before = r.read("model/Decisions/ADR-AM-001.md");
    for args in [
        vec!["set", "ADR-AM-001", "amendment.add", "kind=tweak", "reason=r", "--by", "alice"],
        vec!["set", "ADR-AM-001", "amendment.add", "kind=clarification", "reason=r", "ref=ADR-NOPE-999", "--by", "alice"],
        vec!["set", "ADR-AM-001", "amendment.add", "kind=clarification", "reason=r"],
        vec!["set", "Parts::Alpha", "amendment.add", "kind=clarification", "reason=r", "--by", "alice"],
    ] {
        let (_o, err, code) = r.run(&args);
        assert_eq!(code, 1, "{args:?}: {err}");
    }
    assert_eq!(r.read("model/Decisions/ADR-AM-001.md"), before);
    // A scope change without a ref is written, with the W325 note.
    let (_o, err, code) = r.run(&["set", "ADR-AM-001", "amendment.add", "kind=scope-change", "reason=also C", "--by", "alice"]);
    assert_eq!(code, 0, "{err}");
    assert!(err.contains("W325"), "{err}");
}

#[test]
fn show_and_export_html_render_the_amendment_history() {
    let r = Repo::new("", true);
    r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "X.", &amendment("2026-03-01", "scope-change", Some("RR-AM-001"))));
    let (out, err, code) = r.run(&["show", "ADR-AM-001"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("## Amendments") && out.contains("| 2026-03-01 | scope-change | owner | covers message B too | RR-AM-001 |"), "{out}");
    let site = r.root.join("site");
    let (_o, err, code) = r.run(&["export-html", "--out", site.to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    let page = std::fs::read_dir(site.join("elements"))
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("ADR-AM-001"))
        .map(|e| std::fs::read_to_string(e.path()).unwrap())
        .expect("ADR page");
    assert!(page.contains("<h2>Amendments</h2>") && page.contains("covers message B too") && page.contains("RR-AM-001.html"), "{page}");
}

#[test]
fn an_amendment_alone_does_not_make_links_suspect_but_the_edit_does() {
    let r = Repo::new("", true);
    r.write("model/Requirements/REQ-AM-002.md", &req("REQ-AM-002", "draft", "Child.", "derivedFrom: [REQ-AM-001]\nbreakdownAdr: ADR-AM-001\n"));
    let (_o, err, code) = r.run(&["suspect", "accept", "--all-unbaselined"]);
    assert_eq!(code, 0, "{err}");
    assert!(r.findings("W090").is_empty());
    r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "We use protocol X for message A.", &amendment("2026-03-01", "clarification", None)));
    assert!(r.findings("W090").is_empty(), "{:?}", r.findings("W090"));
    r.write("model/Decisions/ADR-AM-001.md", &adr("accepted", "We use protocol X for messages A and B.", &amendment("2026-03-01", "clarification", None)));
    let w = r.findings("W090");
    assert!(w.iter().any(|l| l.contains("ADR-AM-001")), "{w:?}");
}

#[test]
fn template_and_help_teach_amendments() {
    let r = Repo::new("", false);
    for ty in ["ADR", "Requirement"] {
        let (out, _e, code) = r.run(&["template", ty]);
        assert_eq!(code, 0);
        assert!(out.contains("# amendments:") && out.contains("scope-change"), "{ty}: {out}");
        let (out, _e, _c) = r.run(&["help", ty]);
        assert!(out.contains("amendments") && out.contains("amendment.add"), "{ty}: {out}");
    }
    for term in ["amend", "amendment", "scope-change"] {
        let (out, _e, _c) = r.run(&["help", "--search", term]);
        assert!(out.to_lowercase().contains("amend"), "{term}: {out}");
    }
}

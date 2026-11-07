//! TC-TRS-ITEMREF-001..005 (GH #453): requirement item anchors, checked references to
//! items and PlanningItem questions, refs/impact/check-ref/lint-docs, and item-granular
//! suspect links.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Model {
    root: PathBuf,
}

impl Model {
    fn new() -> Model {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-itemref-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&root).unwrap();
        let m = Model { root };
        m.write("_index.md", "---\ntype: Package\nname: Root\n---\n");
        m.write(
            "Requirements/REQ-IR-001.md",
            &req("REQ-IR-001", "draft", "The server shall:\n\n1. {#grant-keys} **Server keys.** Hold the grant keys.\n2. {#rotate} **Rotation.** Rotate keys yearly.\n3. Log every use.\n"),
        );
        m.write("Planning/PI-IR-001.md", &pi("PI-IR-001"));
        m
    }
    fn write(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.root.join(rel)).unwrap()
    }
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&self.root).args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
    }
    fn validate(&self) -> String {
        let (o, e, _) = self.run(&["validate"]);
        format!("{o}{e}")
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn req(id: &str, status: &str, body: &str) -> String {
    format!("---\nid: {id}\ntype: Requirement\nname: {id}\nstatus: {status}\nreqDomain: software\nreqClass: system\n---\n\n{body}")
}

fn pi(id: &str) -> String {
    format!(
        "---\nid: {id}\ntype: PlanningItem\nname: {id}\nstatus: todo\nachieves: [REQ-IR-001]\nquestions:\n  - id: Q1\n    text: Which HSM?\n    status: open\n  - id: Q2\n    text: Key length?\n    status: answered\n    answer: 256 bits\n---\n\nBody.\n"
    )
}

fn lines_with<'a>(out: &'a str, code: &str) -> Vec<&'a str> {
    out.lines().filter(|l| l.contains(code)).collect()
}

// ── TC-TRS-ITEMREF-001: anchors, numbering, display ──────────────────────────

#[test]
fn show_and_export_html_drop_the_anchor_token_and_keep_the_number() {
    let m = Model::new();
    let (out, _, code) = m.run(&["show", "REQ-IR-001"]);
    assert_eq!(code, 0);
    assert!(out.contains("1. **Server keys.** Hold the grant keys."), "{out}");
    assert!(!out.contains("{#grant-keys}"), "{out}");

    let dest = m.root.join("site");
    let (_o, err, code) = m.run(&["export-html", "--out", dest.to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    let mut html = String::new();
    for e in walk(&dest) {
        let t = std::fs::read_to_string(&e).unwrap_or_default();
        if t.contains("Server keys") {
            html = t;
        }
    }
    assert!(html.contains("<a id=\"item-grant-keys\"></a>"), "export-html adds an HTML anchor");
    assert!(!html.contains("{#grant-keys}"));
}

fn walk(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(walk(&p));
        } else if p.extension().is_some_and(|x| x == "html") {
            out.push(p);
        }
    }
    out
}

#[test]
fn duplicate_and_malformed_anchors_are_w092() {
    let m = Model::new();
    m.write("Requirements/REQ-IR-002.md", &req("REQ-IR-002", "draft", "1. {#a} One.\n2. {#a} Two.\n3. {#Bad} Three.\n"));
    let out = m.validate();
    let w = lines_with(&out, "W092");
    assert!(w.iter().any(|l| l.contains("defined twice")), "{out}");
    assert!(w.iter().any(|l| l.contains("malformed anchor")), "{out}");
}

// ── TC-TRS-ITEMREF-002: reference grammar and W092 ──────────────────────────

#[test]
fn dangling_item_and_question_references_are_w092_and_valid_ones_are_silent() {
    let m = Model::new();
    m.write(
        "Requirements/REQ-IR-003.md",
        &req(
            "REQ-IR-003",
            "draft",
            "Relies on REQ-IR-001#grant-keys, REQ-IR-001#3, REQ-IR-001 item 2 and PI-IR-001#Q2.\n\nAlso REQ-IR-001#nope, REQ-IR-001#9, REQ-IR-001 Item 7 and PI-IR-001#Q9.\n\nGHOST-ID-001#x is not checked.\n\n```\nREQ-IR-001#also-nope\n```\n",
        ),
    );
    let out = m.validate();
    let w = lines_with(&out, "W092");
    assert_eq!(w.len(), 4, "{out}");
    for needle in ["REQ-IR-001#nope", "REQ-IR-001#9", "REQ-IR-001 Item 7", "PI-IR-001#Q9"] {
        assert!(w.iter().any(|l| l.contains(needle)), "{needle} not reported: {out}");
    }
    assert!(!out.contains("also-nope"), "fenced code is not scanned: {out}");
    assert!(!out.contains("GHOST-ID-001"), "an unresolved element is not reported here: {out}");
}

// ── TC-TRS-ITEMREF-003: open / answered question citations ──────────────────

#[test]
fn a_committed_element_citing_an_open_question_is_w093() {
    let m = Model::new();
    m.write("Requirements/REQ-IR-004.md", &req("REQ-IR-004", "approved", "HSM per PI-IR-001#Q1.\n"));
    m.write("Requirements/REQ-IR-005.md", &req("REQ-IR-005", "draft", "HSM per PI-IR-001#Q1.\n"));
    let out = m.validate();
    let w = lines_with(&out, "W093");
    assert_eq!(w.len(), 1, "{out}");
    assert!(w[0].contains("REQ-IR-004") && w[0].contains("PI-IR-001#Q1"), "{out}");
}

#[test]
fn an_answered_question_cited_as_to_confirm_is_w094_and_markers_are_configurable() {
    let m = Model::new();
    m.write("Requirements/REQ-IR-006.md", &req("REQ-IR-006", "draft", "Key length to be confirmed (PI-IR-001#Q2). Fine: PI-IR-001#Q2 says 256.\n"));
    let out = m.validate();
    let w = lines_with(&out, "W094");
    assert_eq!(w.len(), 1, "{out}");
    assert!(w[0].contains("to be confirmed"), "{out}");

    m.write(".syscribe.toml", "[references]\nopen_markers = [\"pending owner\"]\n");
    let out = m.validate();
    assert!(lines_with(&out, "W094").is_empty(), "the configured list replaces the defaults: {out}");
    m.write("Requirements/REQ-IR-006.md", &req("REQ-IR-006", "draft", "Pending owner, PI-IR-001#Q2.\n"));
    assert_eq!(lines_with(&m.validate(), "W094").len(), 1);

    m.write(".syscribe.toml", "[references]\nopen_markers = \"oops\"\n");
    let out = m.validate();
    assert!(lines_with(&out, "W094").iter().any(|l| l.contains(".syscribe.toml") && l.contains("open_markers")), "{out}");
}

// ── TC-TRS-ITEMREF-004: refs / impact / check-ref / lint-docs ───────────────

#[test]
fn refs_impact_and_check_ref_understand_item_references() {
    let m = Model::new();
    m.write("Requirements/REQ-IR-007.md", &req("REQ-IR-007", "draft", "Uses REQ-IR-001#grant-keys.\n"));
    m.write("Requirements/REQ-IR-008.md", &req("REQ-IR-008", "draft", "Uses REQ-IR-001#1 and PI-IR-001#Q1.\n"));
    m.write("Requirements/REQ-IR-009.md", &req("REQ-IR-009", "draft", "Uses REQ-IR-001#rotate.\n"));

    let (out, _, code) = m.run(&["refs", "REQ-IR-001#grant-keys"]);
    assert_eq!(code, 0);
    assert!(out.contains("REQ-IR-007") && out.contains("REQ-IR-008"), "anchor and number name the same item: {out}");
    assert!(!out.contains("REQ-IR-009"), "{out}");

    let (out, _, _) = m.run(&["refs", "REQ-IR-001"]);
    assert!(out.contains("Item and question citations") && out.contains("REQ-IR-009"), "{out}");

    let (out, _, _) = m.run(&["refs", "PI-IR-001#Q1"]);
    assert!(out.contains("REQ-IR-008"), "{out}");

    let (out, _, code) = m.run(&["impact", "REQ-IR-001#rotate"]);
    assert_eq!(code, 0);
    assert!(out.contains("REQ-IR-009") && !out.contains("REQ-IR-007"), "{out}");

    let (out, _, code) = m.run(&["check-ref", "REQ-IR-001#rotate"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("item      2 {#rotate}"), "{out}");
    let (out, _, code) = m.run(&["check-ref", "REQ-IR-001 item 3"]);
    assert_eq!(code, 0, "{out}");
    let (out, _, code) = m.run(&["check-ref", "REQ-IR-001#missing"]);
    assert_eq!(code, 1, "{out}");
    let (_, _, code) = m.run(&["check-ref", "PI-IR-001#Q2"]);
    assert_eq!(code, 0);
}

#[test]
fn lint_docs_reports_dangling_item_references_in_external_docs() {
    let m = Model::new();
    let doc = m.root.join("notes.md");
    std::fs::write(&doc, "See REQ-IR-001#grant-keys and REQ-IR-001#gone.\n").unwrap();
    let (out, _, code) = m.run(&["lint-docs", doc.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    let w = lines_with(&out, "W092");
    assert_eq!(w.len(), 1, "{out}");
    assert!(w[0].contains("REQ-IR-001#gone"), "{out}");
}

// ── TC-TRS-ITEMREF-005: item-granular suspect links ─────────────────────────

#[test]
fn an_item_baseline_goes_suspect_only_when_that_item_changes() {
    let m = Model::new();
    m.write("Requirements/REQ-IR-010.md", &req("REQ-IR-010", "draft", "Relies on REQ-IR-001#grant-keys.\n"));
    let (out, err, code) = m.run(&["suspect", "accept", "REQ-IR-010", "REQ-IR-001#grant-keys"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(m.read("Requirements/REQ-IR-010.md").contains("REQ-IR-001#grant-keys: \"blake3:"));
    assert!(lines_with(&m.validate(), "W090").is_empty());

    // Edit a different item, and insert one above: still not suspect.
    m.write(
        "Requirements/REQ-IR-001.md",
        &req("REQ-IR-001", "draft", "The server shall:\n\n1. New first item.\n2. {#grant-keys} **Server keys.** Hold the grant keys.\n3. {#rotate} **Rotation.** Rotate keys monthly.\n4. Log every use.\n"),
    );
    assert!(lines_with(&m.validate(), "W090").is_empty(), "{}", m.validate());

    // Edit the cited item: suspect.
    m.write(
        "Requirements/REQ-IR-001.md",
        &req("REQ-IR-001", "draft", "The server shall:\n\n1. New first item.\n2. {#grant-keys} **Server keys.** Hold and escrow the grant keys.\n"),
    );
    let out = m.validate();
    let w = lines_with(&out, "W090");
    assert_eq!(w.len(), 1, "{out}");
    assert!(w[0].contains("REQ-IR-001#grant-keys"), "{out}");
    let (out, _, _) = m.run(&["suspect", "list"]);
    assert!(out.contains("SUSPECT  REQ-IR-010 cites REQ-IR-001#grant-keys"), "{out}");
}

#[test]
fn a_by_number_reference_cannot_be_baselined() {
    let m = Model::new();
    m.write("Requirements/REQ-IR-011.md", &req("REQ-IR-011", "draft", "Relies on REQ-IR-001#3.\n"));
    let (_o, err, code) = m.run(&["suspect", "accept", "REQ-IR-011", "REQ-IR-001#3"]);
    assert_eq!(code, 1);
    assert!(err.contains("by number") && err.contains("anchor"), "{err}");
}

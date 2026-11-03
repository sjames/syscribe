//! TC-TRS-HELPDISC-001 (GH #438) and TC-TRS-HINTS-001 (GH #439): type-centric help, keyword
//! search, `explain`, model-free `template`, did-you-mean, and `validate --hints`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn run_in(dir: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).current_dir(dir).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

/// An empty directory with no model in it.
fn empty_dir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-helpdisc-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn model() -> PathBuf {
    let d = empty_dir().join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("REQ-AAA-001.md", "---\nid: REQ-AAA-001\ntype: Requirement\nname: R\nstatus: approvd\nreqDomain: software\nreqClass: system\n---\n\nShall.\n");
    w(
        "PI-AAA-001.md",
        "---\nid: PI-AAA-001\ntype: PlanningItem\nname: P\nstatus: todo\nachieves: [REQ-AAA-001]\nclaimedBy: bob\nevidence:\n  - ref: REQ-AAA-001\n  - ref: REQ-AAA-001\n---\n\nx\n",
    );
    d
}

// ── GH #438 ──────────────────────────────────────────────────────────────────

#[test]
fn help_for_a_type_lists_its_commands_without_a_model() {
    let (out, _e, code) = run_in(&empty_dir(), &["help", "PlanningItem"]);
    assert_eq!(code, 0);
    for want in ["    claim ", "    release ", "    set ", "todo | in_progress | blocked | done", "## EXAMPLE", "E708"] {
        assert!(out.contains(want), "missing {want:?}:\n{out}");
    }
    // Case-insensitive, and the same page from `-m <root> help`.
    assert_eq!(run_in(&empty_dir(), &["help", "planningitem"]).0, out);
}

#[test]
fn keyword_search_finds_commands_by_what_they_do() {
    let (out, _e, code) = run_in(&empty_dir(), &["help", "--search", "evidence"]);
    assert_eq!(code, 0);
    for want in ["set ", "ingest-results ", "results "] {
        assert!(out.lines().any(|l| l.trim_start().starts_with(want)), "{want:?} missing:\n{out}");
    }
    assert!(out.contains("evidence.add"), "the matching line is shown:\n{out}");
    assert_eq!(run_in(&empty_dir(), &["help", "-k", "evidence"]).0, out);
    let (none, _e, c) = run_in(&empty_dir(), &["help", "--search", "zzqqxx"]);
    assert_eq!(c, 0);
    assert!(none.contains("No help matches"), "{none}");
    assert_eq!(run_in(&empty_dir(), &["help", "--search"]).2, 1);
}

#[test]
fn tasks_page_and_unknown_topics() {
    let (out, _e, code) = run_in(&empty_dir(), &["help", "tasks"]);
    assert_eq!(code, 0);
    assert!(out.contains("claim <PI-id> --by"), "{out}");
    let (_o, err, code) = run_in(&empty_dir(), &["help", "clam"]);
    assert_eq!(code, 1);
    assert!(err.contains("Did you mean") && err.contains("claim "), "{err}");
}

#[test]
fn template_needs_no_model() {
    let d = empty_dir();
    let (out, err, code) = run_in(&d, &["template", "Requirement"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("type: Requirement"), "{out}");
    let (out, _e, code) = run_in(&d, &["-m", "/no/such/model", "template", "TestCase"]);
    assert_eq!(code, 0);
    assert!(out.contains("type: TestCase"));
    let (_o, err, code) = run_in(&d, &["template", "Requirment"]);
    assert_eq!(code, 1);
    assert!(err.contains("Did you mean 'Requirement'?"), "{err}");
    // Every other command still needs the model.
    assert_eq!(run_in(&d, &["-m", "/no/such/model", "validate"]).2, 1);
}

// ── GH #439 ──────────────────────────────────────────────────────────────────

#[test]
fn explain_prints_the_catalogue_text_and_matches_help() {
    let (out, _e, code) = run_in(&empty_dir(), &["explain", "E106"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("E106 (error)") && out.contains("Gherkin"), "{out}");
    assert_eq!(run_in(&empty_dir(), &["help", "e106"]).0, out);
    let (_o, err, code) = run_in(&empty_dir(), &["explain", "E999"]);
    assert_eq!(code, 1);
    assert!(err.contains("Unknown finding code"), "{err}");
    assert_eq!(run_in(&empty_dir(), &["explain"]).2, 1);
}

#[test]
fn set_answers_a_typo_with_the_nearest_field_and_value() {
    let d = model();
    let root = d.to_str().unwrap();
    let (_o, err, code) = run_in(d.parent().unwrap(), &["-m", root, "set", "PI-AAA-001", "stauts=done"]);
    assert_eq!(code, 1);
    assert!(err.contains("Did you mean 'status'?"), "{err}");
    let (_o, err, code) = run_in(d.parent().unwrap(), &["-m", root, "set", "PI-AAA-001", "status=in-progress"]);
    assert_eq!(code, 1);
    assert!(err.contains("Did you mean 'in_progress'?"), "{err}");
}

#[test]
fn unknown_commands_and_options_name_the_nearest() {
    let d = model();
    let root = d.to_str().unwrap();
    let (_o, err, code) = run_in(d.parent().unwrap(), &["zvalidate"]);
    assert_ne!(code, 0);
    assert!(err.contains("unrecognized command 'zvalidate'") && err.contains("validate "), "{err}");
    assert!(err.matches("\n  ").count() <= 3, "at most three suggestions:\n{err}");
    let (_o, err, code) = run_in(d.parent().unwrap(), &["-m", root, "validate", "--dney", "W1"]);
    assert_eq!(code, 1);
    assert!(err.contains("Did you mean '--deny'?"), "{err}");
}

#[test]
fn hints_are_opt_in_info_findings_that_name_the_command() {
    let d = model();
    let root = d.to_str().unwrap();
    let cwd = d.parent().unwrap();
    let (plain, _e, plain_code) = run_in(cwd, &["-m", root, "validate"]);
    assert!(!plain.contains("I91"), "hints must be off by default:\n{plain}");

    let (out, _e, code) = run_in(cwd, &["-m", root, "validate", "--hints"]);
    let row = |c: &str| out.lines().find(|l| l.contains(&format!("| {c} |"))).unwrap_or_else(|| panic!("{c} missing:\n{out}")).to_string();
    let i910 = row("I910");
    assert!(i910.contains("PI-AAA-001") && i910.contains("claim") && i910.contains("release"), "{i910}");
    assert!(row("I911").contains("evidence"), "{}", row("I911"));
    let i912 = row("I912");
    assert!(i912.contains("approved") && i912.contains("set REQ-AAA-001 status=approved"), "{i912}");
    // Informational: the exit status is the same with and without them.
    assert_eq!(code, plain_code);
}

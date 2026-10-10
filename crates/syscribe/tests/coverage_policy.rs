//! TC-TRS-COVPOL-001 / GH #253.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// 001 ← 002, 003 (all leaves verified), no direct test on 001.
fn model(toml: &str, extra_fm: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-covpol-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w(".syscribe.toml", toml);
    let req = |id: &str, parent: Option<&str>, extra: &str| {
        let d = parent.map(|p| format!("derivedFrom: [{p}]\n")).unwrap_or_default();
        format!("---\nid: {id}\ntype: Requirement\nname: \"{id}\"\nstatus: approved\nreqDomain: software\nreqClass: system\ntags: [function]\n{d}{extra}---\n\nShall.\n")
    };
    let tc = |id: &str, v: &str| format!("---\nid: {id}\ntype: TestCase\nname: \"{id}\"\nstatus: active\ntestLevel: L3\nverifies: [{v}]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    w("REQ-CP-001.md", &req("REQ-CP-001", None, extra_fm));
    w("REQ-CP-002.md", &req("REQ-CP-002", Some("REQ-CP-001"), ""));
    w("REQ-CP-003.md", &req("REQ-CP-003", Some("REQ-CP-001"), ""));
    w("TC-CP-002.md", &tc("TC-CP-002", "REQ-CP-002"));
    w("TC-CP-003.md", &tc("TC-CP-003", "REQ-CP-003"));
    r
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn top(root: &Path) -> (String, i32) {
    let (o, c) = run(root, &["coverage", "tree", "REQ-CP-001"]);
    (o.lines().next().unwrap_or("").to_string(), c)
}

#[test]
fn default_is_both_and_prints_the_rule() {
    let (l, c) = top(&model("", ""));
    assert_eq!(c, 0, "{l}");
    assert!(l.starts_with("◐ REQ-CP-001") && l.contains("(rule: both)"), "{l}");
}

#[test]
fn a_rollup_rule_completes_a_parent_with_all_leaves_verified() {
    let r = model("[[coverage.rule]]\nreqClass = \"system\"\nparent_rule = \"rollup\"\n", "");
    let (l, _) = top(&r);
    assert!(l.starts_with("● REQ-CP-001") && l.contains("(rule: rollup)"), "{l}");
    let (j, _) = run(&r, &["coverage", "tree", "REQ-CP-001", "--json"]);
    assert_eq!(serde_json::from_str::<serde_json::Value>(&j).unwrap()["rule"], "rollup", "{j}");
}

#[test]
fn selectors_are_anded_and_the_first_matching_rule_wins() {
    // tag matches but sil="QM" and tag "function": rated? no -> rollup applies
    let t = "[[coverage.rule]]\nsil = \"QM\"\ntag = [\"function\"]\nparent_rule = \"rollup\"\n[[coverage.rule]]\nparent_rule = \"both\"\n";
    assert!(top(&model(t, "")).0.contains("(rule: rollup)"));
    // tag that does not match falls through to the second rule
    let t = "[[coverage.rule]]\ntag = [\"other\"]\nparent_rule = \"rollup\"\n[[coverage.rule]]\nparent_rule = \"direct\"\n";
    assert!(top(&model(t, "")).0.contains("(rule: direct)"));
    // default applies when no rule matches
    let t = "[coverage]\ndefault = \"rollup\"\n[[coverage.rule]]\nreqClass = \"stakeholder\"\nparent_rule = \"both\"\n";
    assert!(top(&model(t, "")).0.contains("(rule: rollup)"));
}

#[test]
fn loosening_an_integrity_rated_requirement_is_refused() {
    let t = "[[coverage.rule]]\nreqClass = \"system\"\nparent_rule = \"rollup\"\n";
    let (o, c) = run(&model(t, "asilLevel: B\n"), &["coverage", "tree", "REQ-CP-001"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("REQ-CP-001") && o.contains("rollup") && o.contains("integrity"), "{o}");
    // an ASIL-scoped strict rule ahead of the loose one keeps it valid
    let t = "[[coverage.rule]]\nasil = [\"A\",\"B\",\"C\",\"D\"]\nparent_rule = \"both\"\n[[coverage.rule]]\nreqClass = \"system\"\nparent_rule = \"rollup\"\n";
    let (o, c) = run(&model(t, "asilLevel: B\n"), &["coverage", "tree", "REQ-CP-001"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.lines().next().unwrap().contains("(rule: both)"), "{o}");
}

#[test]
fn invalid_values_and_unknown_selectors_are_errors() {
    for t in [
        "[coverage]\ndefault = \"sometimes\"\n",
        "[[coverage.rule]]\nparent_rule = \"nope\"\n",
        "[[coverage.rule]]\ncolour = \"red\"\nparent_rule = \"both\"\n",
        "[[coverage.rule]]\nasil = [\"Z\"]\nparent_rule = \"both\"\n",
    ] {
        let (o, c) = run(&model(t, ""), &["coverage", "tree", "REQ-CP-001"]);
        assert_eq!(c, 1, "{t}: {o}");
        assert!(o.contains("[coverage"), "{t}: {o}");
    }
}

fn stdout_of(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into_owned(), o.status.code().unwrap_or(-1))
}

#[test]
fn a_config_error_prints_nothing_on_stdout_and_a_syntax_error_is_not_ignored() {
    let t = "[[coverage.rule]]\nreqClass = \"system\"\nparent_rule = \"rollup\"\n";
    let (o, c) = stdout_of(&model(t, "asilLevel: B\n"), &["coverage", "tree", "REQ-CP-001", "--json"]);
    assert_eq!((o.as_str(), c), ("", 1));
    let (o, c) = run(&model("[coverage\ndefault = \"rollup\"\n", ""), &["coverage", "tree", "REQ-CP-001"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("does not parse"), "{o}");
    let (o, c) = run(&model("[[coverage.rule]]\ntag = []\nparent_rule = \"both\"\n", ""), &["coverage", "tree", "REQ-CP-001"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("never match"), "{o}");
}

#[test]
fn cal_and_sil_rated_requirements_are_guarded_and_sil_zero_is_qm() {
    let t = "[[coverage.rule]]\nreqClass = \"system\"\nparent_rule = \"direct\"\n";
    for fm in ["calLevel: CAL2\n", "silLevel: 2\n"] {
        let (o, c) = run(&model(t, fm), &["coverage", "tree", "REQ-CP-001"]);
        assert_eq!(c, 1, "{fm}: {o}");
    }
    let (o, c) = run(&model(t, "silLevel: 0\n"), &["coverage", "tree", "REQ-CP-001"]);
    assert_eq!(c, 0, "{o}");
}

//! TC-TRS-DOCFIX-001 / GH #248, #249.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-docfix-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("REQ-DF-001.md", "---\nid: REQ-DF-001\ntype: Requirement\nname: r\nstatus: approved\nreqDomain: software\n---\n\nShall.\n");
    w(
        "TC-DF-001.md",
        "---\nid: TC-DF-001\ntype: TestCase\nname: t\nstatus: approved\ntestLevel: L3\nverifies: [REQ-DF-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n",
    );
    w("TP-DFX-001.md", "---\ntype: TestPlan\nid: TP-DFX-001\nname: p\nstatus: approved\nscope: integration\ntestCases: [TC-DF-001]\n---\n\nP.\n");
    w(
        "CSG-DF-001.md",
        "---\ntype: CybersecurityGoal\nid: CSG-DF-001\nname: g\nstatus: approved\ncalLevel: CAL3\n---\n\nG.\n",
    );
    w(
        "CM-DF-001.md",
        "---\ntype: ConfirmationMeasure\nid: CM-DF-001\nname: cm\nstatus: draft\nmeasureType: confirmation_review\nindependenceLevel: I1\nconfirms: [REQ-DF-001]\n---\n\nR.\n",
    );
    r
}

fn run(root: &Path, args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

#[test]
fn verification_depth_names_the_counted_status() {
    let out = run(&model(), &["verification-depth"]);
    assert!(out.contains("active"), "{out}");
    assert!(out.to_lowercase().contains("counts") || out.contains("Counted"), "{out}");
}

#[test]
fn testplan_explains_its_coverage_percentage() {
    let out = run(&model(), &["testplan"]);
    assert!(out.contains("Coverage ="), "{out}");
}

#[test]
fn the_cal3_w039_message_says_i2_or_higher() {
    let out = run(&model(), &["validate"]);
    let line = out.lines().find(|l| l.contains("W039") && l.contains("CAL3")).unwrap_or_else(|| panic!("{out}"));
    assert!(line.contains("I2 or higher"), "{line}");
}

#[test]
fn the_testplan_legend_does_not_contain_verdict_words_that_would_confuse_text_checks() {
    // qual/tests/tc/TC-TRS-PLAN-005.sh greps the text output for the verdict word.
    let out = run(&model(), &["testplan"]);
    let legend = out.lines().find(|l| l.starts_with("Coverage =")).unwrap();
    assert!(!legend.to_lowercase().contains("pass") && !legend.to_lowercase().contains("fail"), "{legend}");
    assert!(legend.contains("non-draft"), "{legend}");
}

#[test]
fn the_verification_depth_note_is_accurate_about_manual_tests() {
    let out = run(&model(), &["verification-depth"]);
    assert!(out.contains("`active` TestCases only"), "{out}");
}

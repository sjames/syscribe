//! TC-TRS-QUANT-001 / GH #237.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(files: &[(&str, String)]) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-quant-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    std::fs::create_dir_all(d.join("R")).unwrap();
    std::fs::write(d.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(d.join("R/_index.md"), "---\ntype: Package\nname: R\n---\n").unwrap();
    for (n, c) in files {
        std::fs::write(d.join("R").join(n), c).unwrap();
    }
    d
}

fn validate(d: &Path) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d).arg("validate").output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn req(id: &str, parent: Option<&str>, q: &str, extra: &str) -> (String, String) {
    let d = parent.map(|p| format!("derivedFrom: [{p}]\n")).unwrap_or_default();
    (format!("{id}.md"), format!("---\nid: {id}\ntype: Requirement\nname: {id}\nstatus: approved\nreqDomain: software\nreqClass: system\n{d}{extra}quantities:\n{q}---\n\nShall.\n"))
}

fn files(v: Vec<(String, String)>) -> Vec<(&'static str, String)> {
    v.into_iter().map(|(a, b)| (Box::leak(a.into_boxed_str()) as &'static str, b)).collect()
}

#[test]
fn malformed_entries_are_e895() {
    for q in ["  - {kind: speed, value: 5, unit: ms}\n", "  - {kind: latency, value: 5, unit: parsecs}\n", "  - {kind: latency, value: -1, unit: ms}\n", "  - {kind: latency, value: fast, unit: ms}\n", "  - nonsense\n"] {
        let d = model(&files(vec![req("REQ-QT-001", None, q, "")]));
        assert!(validate(&d).contains("E895"), "{q}");
    }
    let d = model(&files(vec![req("REQ-QT-001", None, "  - {kind: latency, value: 5, unit: ms}\n  - {kind: wcet, value: 200, unit: us}\n", "")]));
    let v = validate(&d);
    assert!(!v.contains("E895") && !v.contains("W893"), "{v}");
}

#[test]
fn fitting_and_overrunning_chains() {
    let fit = files(vec![
        req("REQ-QT-001", None, "  - {kind: latency, value: 100, unit: ms}\n", ""),
        req("REQ-QT-002", Some("REQ-QT-001"), "  - {kind: latency, value: 40, unit: ms}\n", ""),
        req("REQ-QT-003", Some("REQ-QT-001"), "  - {kind: latency, value: 0.05, unit: s}\n", ""),
    ]);
    assert!(!validate(&model(&fit)).contains("W893"));
    let over = files(vec![
        req("REQ-QT-001", None, "  - {kind: latency, value: 100, unit: ms}\n", ""),
        req("REQ-QT-002", Some("REQ-QT-001"), "  - {kind: latency, value: 60, unit: ms}\n", ""),
        req("REQ-QT-003", Some("REQ-QT-001"), "  - {kind: latency, value: 0.05, unit: s}\n", ""),
    ]);
    let v = validate(&model(&over));
    assert!(v.contains("W893") && v.contains("REQ-QT-001"), "{v}");
    // a parent with no child quantity of the kind raises nothing
    let none = files(vec![req("REQ-QT-001", None, "  - {kind: latency, value: 100, unit: ms}\n", ""), req("REQ-QT-002", Some("REQ-QT-001"), "  - {kind: wcet, value: 500, unit: ms}\n", "")]);
    assert!(!validate(&model(&none)).contains("W893"));
}

#[test]
fn a_reaction_over_the_goals_ftti_is_w893() {
    let sg = |ftti: &str| ("SG-QT-001.md".to_string(), format!("---\nid: SG-QT-001\ntype: SafetyGoal\nname: sg\nstatus: approved\nasilLevel: B\nftti: {ftti}\n---\n\nGoal.\n"));
    let child = |q: &str| req("REQ-QT-001", None, q, "derivedFromSafetyGoal: SG-QT-001\n");
    let over = files(vec![sg("50 ms"), child("  - {kind: latency, value: 20, unit: ms}\n  - {kind: reaction, value: 40, unit: ms}\n")]);
    let v = validate(&model(&over));
    assert!(v.contains("W893") && v.contains("SG-QT-001"), "{v}");
    let fits = files(vec![sg("50 ms"), child("  - {kind: latency, value: 20, unit: ms}\n  - {kind: reaction, value: 20, unit: ms}\n")]);
    assert!(!validate(&model(&fits)).contains("W893"));
    // an `ftti` quantity on the goal works like the ftti: string
    let qgoal = ("SG-QT-001.md".to_string(), "---\nid: SG-QT-001\ntype: SafetyGoal\nname: sg\nstatus: approved\nasilLevel: B\nquantities:\n  - {kind: ftti, value: 0.05, unit: s}\n---\n\nGoal.\n".to_string());
    let over = files(vec![qgoal, child("  - {kind: reaction, value: 60, unit: ms}\n")]);
    assert!(validate(&model(&over)).contains("W893"));
}

#[test]
fn budgets_roll_up_through_intermediates_and_skip_drafts_alternatives_and_duplicates() {
    // G (100) -> M (nothing) -> L1, L2 (80 each): the leaves sum to 160
    let over = files(vec![
        req("REQ-QT-001", None, "  - {kind: latency, value: 100, unit: ms}\n", ""),
        req("REQ-QT-002", Some("REQ-QT-001"), "  - {kind: wcet, value: 1, unit: ms}\n", ""),
        req("REQ-QT-003", Some("REQ-QT-002"), "  - {kind: latency, value: 80, unit: ms}\n", ""),
        req("REQ-QT-004", Some("REQ-QT-002"), "  - {kind: latency, value: 80, unit: ms}\n", ""),
    ]);
    assert!(validate(&model(&over)).contains("W893"));
    // a draft child does not count
    let draft = files(vec![
        req("REQ-QT-001", None, "  - {kind: latency, value: 100, unit: ms}\n", ""),
        req("REQ-QT-002", Some("REQ-QT-001"), "  - {kind: latency, value: 60, unit: ms}\n", ""),
        req("REQ-QT-003", Some("REQ-QT-001"), "  - {kind: latency, value: 60, unit: ms}\n", "").tap_draft(),
    ]);
    assert!(!validate(&model(&draft)).contains("W893"));
    // a gated (variant) child is left to the per-variant check
    let gated = files(vec![
        req("REQ-QT-001", None, "  - {kind: latency, value: 100, unit: ms}\n", ""),
        req("REQ-QT-002", Some("REQ-QT-001"), "  - {kind: latency, value: 80, unit: ms}\n", ""),
        req("REQ-QT-003", Some("REQ-QT-001"), "  - {kind: latency, value: 80, unit: ms}\n", "appliesWhen: FEAT-NOPE-001\n"),
    ]);
    assert!(!validate(&model(&gated)).contains("W893"));
    // a duplicate kind is E895; a duplicated derivedFrom edge does not double a child
    let dup = files(vec![req("REQ-QT-001", None, "  - {kind: latency, value: 5, unit: ms}\n  - {kind: latency, value: 6, unit: ms}\n", "")]);
    assert!(validate(&model(&dup)).contains("E895"));
    let twice = vec![
        req("REQ-QT-001", None, "  - {kind: latency, value: 100, unit: ms}\n", ""),
        {
            let (n, c) = req("REQ-QT-002", Some("REQ-QT-001"), "  - {kind: latency, value: 60, unit: ms}\n", "");
            (n, c.replace("derivedFrom: [REQ-QT-001]", "derivedFrom: [REQ-QT-001, REQ-QT-001]"))
        },
    ];
    assert!(!validate(&model(&files(twice))).contains("W893"));
}

trait TapDraft {
    fn tap_draft(self) -> (String, String);
}
impl TapDraft for (String, String) {
    fn tap_draft(self) -> (String, String) {
        (self.0, self.1.replace("status: approved", "status: draft"))
    }
}

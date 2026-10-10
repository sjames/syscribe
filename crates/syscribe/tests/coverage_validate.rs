//! TC-TRS-COVVAL-001 / GH #253 (validator part).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(toml: &str, leaf3_tested: bool, parent_extra: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-covval-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    if !toml.is_empty() {
        w(".syscribe.toml", toml);
    }
    let req = |id: &str, parent: Option<&str>, extra: &str| {
        let d = parent.map(|p| format!("derivedFrom: [{p}]\n")).unwrap_or_default();
        format!("---\nid: {id}\ntype: Requirement\nname: {id}\nstatus: approved\nreqDomain: software\nreqClass: system\n{d}{extra}---\n\nShall.\n")
    };
    let tc = |id: &str, v: &str| format!("---\nid: {id}\ntype: TestCase\nname: {id}\nstatus: active\ntestLevel: L2\nverifies: [{v}]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    w("R/REQ-CV-001.md", &req("REQ-CV-001", None, parent_extra));
    w("R/REQ-CV-002.md", &req("REQ-CV-002", Some("REQ-CV-001"), ""));
    w("R/REQ-CV-003.md", &req("REQ-CV-003", Some("REQ-CV-001"), ""));
    w("R/TC-CV-002.md", &tc("TC-CV-002", "REQ-CV-002"));
    if leaf3_tested {
        w("R/TC-CV-003.md", &tc("TC-CV-003", "REQ-CV-003"));
    }
    d
}

fn validate(d: &Path) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d).arg("validate").output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

const ROLLUP: &str = "[[coverage.rule]]\nreqClass = \"system\"\nparent_rule = \"rollup\"\n";

#[test]
fn w305_names_the_rule_and_rollup_silences_a_fully_verified_parent() {
    let v = validate(&model("", true, ""));
    assert!(v.contains("W305") && v.contains("(rule: both)"), "{v}");
    let v = validate(&model(ROLLUP, true, ""));
    assert!(!v.contains("W305"), "{v}");
    // one leaf unverified: the parent is not covered through its children
    let v = validate(&model(ROLLUP, false, ""));
    assert!(v.contains("W305") && v.contains("(rule: rollup)"), "{v}");
    // direct behaves as before
    let v = validate(&model("[coverage]\ndefault = \"direct\"\n", true, ""));
    assert!(v.contains("W305") && v.contains("(rule: direct)"), "{v}");
}

#[test]
fn an_invalid_policy_or_a_loosened_rated_parent_is_e898() {
    let v = validate(&model("[coverage]\ndefault = \"sometimes\"\n", true, ""));
    assert!(v.contains("E898") && v.contains(".syscribe.toml"), "{v}");
    let v = validate(&model(ROLLUP, true, "asilLevel: B\n"));
    assert!(v.contains("E898") && v.contains("REQ-CV-001"), "{v}");
    // a strict rule for rated items first keeps it valid
    let ok = "[[coverage.rule]]\nasil = [\"A\",\"B\",\"C\",\"D\"]\nparent_rule = \"both\"\n[[coverage.rule]]\nreqClass = \"system\"\nparent_rule = \"rollup\"\n";
    assert!(!validate(&model(ok, true, "asilLevel: B\n")).contains("E898"));
    assert!(!validate(&model("", true, "asilLevel: B\n")).contains("E898"));
}

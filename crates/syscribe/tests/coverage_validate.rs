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
    assert!(v.contains("W305") && v.contains("rule: both"), "{v}");
    let v = validate(&model(ROLLUP, true, ""));
    assert!(!v.contains("W305"), "{v}");
    // one leaf unverified: the parent is not covered through its children
    let v = validate(&model(ROLLUP, false, ""));
    assert!(v.contains("W305") && v.contains("rule: rollup"), "{v}");
    // direct behaves as before
    let v = validate(&model("[coverage]\ndefault = \"direct\"\n", true, ""));
    assert!(v.contains("W305") && v.contains("rule: direct"), "{v}");
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

#[test]
fn w310_and_draft_leaves_follow_the_same_rollup() {
    let with_pi = |toml: &str, tested: bool| {
        let d = model(toml, tested, "");
        std::fs::create_dir_all(d.join("P")).unwrap();
        std::fs::write(d.join("P/PI-CV-001.md"), "---\nid: PI-CV-001\ntype: PlanningItem\nname: p\nstatus: done\nitemType: task\nachieves: [REQ-CV-001]\nevidence:\n  - ref: TC-CV-002\n---\n\nDone.\n").unwrap();
        d
    };
    assert!(validate(&with_pi("", true)).contains("W310"));
    assert!(!validate(&with_pi(ROLLUP, true)).contains("W310"));
    // a draft leaf without a test does not keep a rolled-up parent uncovered
    let d = model(ROLLUP, false, "");
    let p = d.join("R/REQ-CV-003.md");
    std::fs::write(&p, std::fs::read_to_string(&p).unwrap().replace("status: approved", "status: draft")).unwrap();
    assert!(!validate(&d).contains("W305"), "{}", validate(&d));
}

#[test]
fn an_invalid_policy_is_reported_once_not_per_configuration() {
    let d = model("[coverage]\ndefault = \"sometimes\"\n", true, "");
    std::fs::create_dir_all(d.join("Features")).unwrap();
    std::fs::write(d.join("Features/_index.md"), "---\ntype: Package\nname: Features\n---\n").unwrap();
    std::fs::write(d.join("Features/Opt.md"), "---\ntype: FeatureDef\nid: FEAT-OPT-001\nname: Opt\ngroupKind: optional\n---\n\nO.\n").unwrap();
    for (id, v) in [("CONF-ON-001", "true"), ("CONF-OFF-001", "false")] {
        std::fs::create_dir_all(d.join("Configurations")).unwrap();
        std::fs::write(d.join(format!("Configurations/{id}.md")), format!("---\ntype: Configuration\nid: {id}\nname: {id}\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: {v}\n---\n\nC.\n")).unwrap();
    }
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&d).args(["validate", "--all-configs"]).output().unwrap();
    let out = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    let plain = validate(&d);
    assert!(plain.contains("E898"));
    assert!(out.matches("E898").count() <= 1, "{out}");
}

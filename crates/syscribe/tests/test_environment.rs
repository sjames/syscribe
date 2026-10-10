//! TC-TRS-TESTENV-001 / GH #238.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(te: &str, tc_extra: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-te-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Requirements/_index.md", "---\ntype: Package\nname: Requirements\n---\n");
    w("Requirements/REQ-TE-001.md", "---\nid: REQ-TE-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nShall.\n");
    w("Rigs/_index.md", "---\ntype: Package\nname: Rigs\n---\n");
    if !te.is_empty() {
        w("Rigs/TE-HIL-001.md", te);
    }
    w("Requirements/TC-TE-001.md", &format!("---\nid: TC-TE-001\ntype: TestCase\nname: t\nstatus: active\ntestLevel: L4\nverifies: [REQ-TE-001]\n{tc_extra}---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n"));
    d
}

fn validate(d: &Path) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d).arg("validate").output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

const GOOD: &str = "---\nid: TE-HIL-001\ntype: TestEnvironment\nname: Cluster HIL rig\nstatus: available\nenvironmentKind: hil\ncapabilities: [CAN-FD, display-camera, 12V-supply]\ncalibrationStatus: valid\ncalibrationDue: 2027-03-01\n---\n\nThe rig.\n";
const RUNS: &str = "runsOn: [TE-HIL-001]\nrequiresCapabilities: [can-fd, display-camera]\n";

#[test]
fn a_valid_environment_and_runs_on_are_clean() {
    let v = validate(&model(GOOD, RUNS));
    for c in ["E893", "E894", "W891", "W892"] {
        assert!(!v.contains(c), "{c}: {v}");
    }
}

#[test]
fn defects_are_reported() {
    assert!(validate(&model(&GOOD.replace("status: available", "status: bogus"), RUNS)).contains("E893"));
    assert!(validate(&model(&GOOD.replace("environmentKind: hil", "environmentKind: toaster"), RUNS)).contains("E893"));
    assert!(validate(&model(&GOOD.replace("2027-03-01", "next spring"), RUNS)).contains("E893"));
    assert!(validate(&model(GOOD, "runsOn: [REQ-TE-001]\n")).contains("E894"));
    assert!(validate(&model(GOOD, "runsOn: [TE-NOPE-999]\n")).contains("E894"));
    let missing = validate(&model(GOOD, "runsOn: [TE-HIL-001]\nrequiresCapabilities: [climatic-chamber]\n"));
    assert!(missing.contains("W891") && missing.contains("climatic-chamber"), "{missing}");
    assert!(validate(&model(&GOOD.replace("calibrationStatus: valid", "calibrationStatus: expired"), RUNS)).contains("W892"));
    assert!(validate(&model(&GOOD.replace("status: available", "status: retired"), RUNS)).contains("W892"));
}

#[test]
fn odd_shapes_dates_and_unresolved_runs_on_are_reported_precisely() {
    // wrong shapes give the specific code, not a whole-file parse failure
    let v = validate(&model(&GOOD.replace("calibrationDue: 2027-03-01", "calibrationDue: 20270301").replace("calibrationStatus: valid", "calibrationStatus: yes"), RUNS));
    assert!(v.contains("E893") && !v.contains("E002"), "{v}");
    assert!(validate(&model(&GOOD.replace("2027-03-01", "2027-13-45"), RUNS)).contains("E893"));
    assert!(validate(&model(&GOOD.replace("2027-03-01", "2027-02-30"), RUNS)).contains("E893"));
    assert!(!validate(&model(&GOOD.replace("2027-03-01", "2028-02-29"), RUNS)).contains("E893"));
    // an unresolved runsOn is E894 only — no W891 pile-on
    let v = validate(&model(GOOD, "runsOn: [TE-NOPE-999]\nrequiresCapabilities: [x]\n"));
    assert!(v.contains("E894") && !v.contains("W891"), "{v}");
    // show and refs know the new edge
    let d = model(GOOD, RUNS);
    let run = |a: &[&str]| String::from_utf8_lossy(&Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&d).args(a).output().unwrap().stdout).into_owned();
    assert!(run(&["show", "TC-TE-001"]).contains("TE-HIL-001"));
    assert!(run(&["show", "TE-HIL-001"]).contains("CAN-FD"));
    assert!(run(&["refs", "TE-HIL-001"]).contains("TC-TE-001"), "{}", run(&["refs", "TE-HIL-001"]));
}

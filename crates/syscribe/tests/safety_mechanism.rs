//! TC-TRS-SAFEMECH-001 / GH #236.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(sm: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-sm-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("S/_index.md", "---\ntype: Package\nname: S\n---\n");
    w("S/SG-MX-001.md", "---\nid: SG-MX-001\ntype: SafetyGoal\nname: show speed\nstatus: approved\nasilLevel: B\nftti: 50 ms\n---\n\nGoal.\n");
    w("S/REQ-MX-001.md", "---\nid: REQ-MX-001\ntype: Requirement\nname: r\nstatus: approved\nreqDomain: software\nreqClass: system\nderivedFromSafetyGoal: SG-MX-001\n---\n\nShall.\n");
    w("S/FM-MX-001.md", "---\nid: FM-MX-001\ntype: FMEAEntry\nname: Stuck pixel\nstatus: draft\nfailureMode: stuck\nfmeaSeverity: 5\noccurrence: 3\ndetection: 4\nrpn: 60\n---\n\nb\n");
    w("S/FM-MX-002.md", "---\nid: FM-MX-002\ntype: FMEAEntry\nname: Frozen frame\nstatus: draft\nfailureMode: frozen\nfmeaSeverity: 5\noccurrence: 3\ndetection: 4\nrpn: 60\n---\n\nb\n");
    w("S/Chip.md", "---\ntype: PartDef\nname: Chip\n---\n\nC.\n");
    if !sm.is_empty() {
        w("S/SM-MX-001.md", sm);
    }
    d
}

fn run(d: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

const GOOD: &str = "---\nid: SM-MX-001\ntype: SafetyMechanism\nname: Frame CRC monitor\nstatus: approved\ncovers: [FM-MX-001, REQ-MX-001]\ndiagnosticCoverage: 0.99\nlatentDiagnosticCoverage: 0.9\nreactionTime: 20 ms\nsafeState: blank display\nallocatedTo: [S::Chip]\n---\n\nCRC over each frame.\n";

#[test]
fn a_valid_mechanism_is_clean() {
    let (v, _) = run(&model(GOOD), &["validate"]);
    for c in ["E896", "E897", "W894"] {
        assert!(!v.contains(c), "{c}: {v}");
    }
}

#[test]
fn defects_are_reported() {
    let v = |s: &str| run(&model(s), &["validate"]).0;
    assert!(v(&GOOD.replace("status: approved", "status: bogus")).contains("E896"));
    assert!(v(&GOOD.replace("id: SM-MX-001", "id: XX-MX-001")).contains("E896"));
    // a coverage outside 0..1 or not a number is the generic E846 — reported once, not also as E896
    let range = v(&GOOD.replace("diagnosticCoverage: 0.99", "diagnosticCoverage: 1.5"));
    assert!(range.contains("E846") && !range.contains("E896"), "{range}");
    assert!(v(&GOOD.replace("latentDiagnosticCoverage: 0.9", "latentDiagnosticCoverage: -0.1")).contains("E846"));
    let text = v(&GOOD.replace("diagnosticCoverage: 0.99", "diagnosticCoverage: \"99%\""));
    assert!(text.contains("E846") && !text.contains("E002"), "{text}");
    assert!(v(&GOOD.replace("reactionTime: 20 ms", "reactionTime: fast")).contains("E896"));
    assert!(v(&GOOD.replace("[FM-MX-001, REQ-MX-001]", "[FM-NOPE-999]")).contains("E897"));
    assert!(v(&GOOD.replace("[FM-MX-001, REQ-MX-001]", "[S::Chip]")).contains("E897"));
}

#[test]
fn a_reaction_time_over_the_ftti_is_w894() {
    let over = GOOD.replace("reactionTime: 20 ms", "reactionTime: 80 ms");
    let (v, _) = run(&model(&over), &["validate"]);
    assert!(v.contains("W894") && v.contains("SG-MX-001"), "{v}");
    // a covered goal directly
    let direct = over.replace("[FM-MX-001, REQ-MX-001]", "[SG-MX-001]");
    assert!(run(&model(&direct), &["validate"]).0.contains("W894"));
    // a draft mechanism is not checked
    assert!(!run(&model(&over.replace("status: approved", "status: draft")), &["validate"]).0.contains("W894"));
    // a retired mechanism and a duplicated covers entry
    assert!(!run(&model(&over.replace("status: approved", "status: retired")), &["validate"]).0.contains("W894"));
    let dup = run(&model(&GOOD.replace("[FM-MX-001, REQ-MX-001]", "[S::Chip, S::Chip]")), &["validate"]).0;
    assert_eq!(dup.matches("E897").count(), 1, "{dup}");
    // a requirement two derivedFrom hops below the goal is followed
    let d = model(&over);
    std::fs::write(d.join("S/REQ-MX-002.md"), "---\nid: REQ-MX-002\ntype: Requirement\nname: r2\nstatus: approved\nreqDomain: software\nreqClass: system\nderivedFrom: [REQ-MX-001]\n---\n\nShall.\n").unwrap();
    let chain = std::fs::read_to_string(d.join("S/SM-MX-001.md")).unwrap().replace("[FM-MX-001, REQ-MX-001]", "[REQ-MX-002]");
    std::fs::write(d.join("S/SM-MX-001.md"), chain).unwrap();
    assert!(run(&d, &["validate"]).0.contains("W894"));
    // 1 s expressed in seconds also compares
    assert!(run(&model(&GOOD.replace("20 ms", "1 s")), &["validate"]).0.contains("W894"));
}

#[test]
fn mechanisms_lists_coverage_and_uncovered_failure_modes() {
    let d = model(GOOD);
    let (j, c) = run(&d, &["mechanisms", "--json"]);
    assert_eq!(c, 0, "{j}");
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    let m = &v["mechanisms"][0];
    assert_eq!(m["id"], "SM-MX-001");
    assert_eq!(m["diagnosticCoverage"], 0.99);
    assert_eq!(m["reactionTime"], "20 ms");
    assert!(m["covers"].as_array().unwrap().iter().any(|c| c == "FM-MX-001"));
    let (u, _) = run(&d, &["mechanisms", "--uncovered"]);
    assert!(u.contains("FM-MX-002") && !u.contains("FM-MX-001"), "{u}");
    let (t, _) = run(&d, &["mechanisms"]);
    assert!(t.contains("SM-MX-001") && t.contains("99"), "{t}");
}

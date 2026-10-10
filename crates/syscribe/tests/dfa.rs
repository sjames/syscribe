//! TC-TRS-DFA-001 / GH #235.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(dfa: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-dfa-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
    w("Sys/Cluster.md", "---\ntype: PartDef\nname: Cluster\nasilLevel: B\nallocatedTo: [Sys::Chip]\n---\n\nASIL B partition.\n");
    w("Sys/Android.md", "---\ntype: PartDef\nname: Android\nallocatedTo: [Sys::Chip]\n---\n\nQM partition.\n");
    w("Sys/Chip.md", "---\ntype: PartDef\nname: Chip\n---\n\nShared SoC.\n");
    w("Safety/_index.md", "---\ntype: Package\nname: Safety\n---\n");
    if !dfa.is_empty() {
        w("Safety/DFA-CK-001.md", dfa);
    }
    d
}

fn run(d: &Path, args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d).args(args).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

const GOOD: &str = "---\nid: DFA-CK-001\ntype: DependentFailureAnalysis\nname: Cluster vs Android\nstatus: approved\nanalyses: [Sys::Cluster, Sys::Android]\nsharedResources:\n  - resource: SoC power rail\n    kind: power\n    initiators: [brown-out]\n    couplingFactor: 0.1\n    mitigation: Separate regulators with supervisor\n---\n\nIndependence argument.\n";

#[test]
fn a_valid_analysis_has_no_dfa_findings() {
    let d = model(GOOD);
    let v = run(&d, &["validate"]);
    for c in ["E890", "E891", "E892", "W890"] {
        assert!(!v.contains(c), "{c}: {v}");
    }
}

#[test]
fn defects_are_reported_with_their_codes() {
    let one = GOOD.replace("[Sys::Cluster, Sys::Android]", "[Sys::Cluster]");
    assert!(run(&model(&one), &["validate"]).contains("E890"));
    let dangling = GOOD.replace("Sys::Android]", "Sys::Nope]");
    assert!(run(&model(&dangling), &["validate"]).contains("E891"));
    let kind = GOOD.replace("kind: power", "kind: magic");
    assert!(run(&model(&kind), &["validate"]).contains("E892"));
    let cf = GOOD.replace("couplingFactor: 0.1", "couplingFactor: 2");
    assert!(run(&model(&cf), &["validate"]).contains("E892"));
    let nores = GOOD.replace("resource: SoC power rail", "foo: bar");
    assert!(run(&model(&nores), &["validate"]).contains("E892"));
    let unmit = GOOD.replace("    mitigation: Separate regulators with supervisor\n", "");
    assert!(run(&model(&unmit), &["validate"]).contains("W890"));
    let bad_status = GOOD.replace("status: approved", "status: bogus");
    assert!(run(&model(&bad_status), &["validate"]).contains("E890"));
}

#[test]
fn an_approved_dfa_excuses_w034_a_draft_one_does_not() {
    let none = run(&model(""), &["validate"]);
    assert!(none.contains("W034"), "baseline must show the mixed-criticality sharing: {none}");
    let approved = run(&model(GOOD), &["validate"]);
    assert!(!approved.contains("W034"), "{approved}");
    let draft = run(&model(&GOOD.replace("status: approved", "status: draft")), &["validate"]);
    assert!(draft.contains("W034"), "{draft}");
}

#[test]
fn the_element_is_addressable_by_its_id() {
    let d = model(GOOD);
    let s = run(&d, &["show", "DFA-CK-001"]);
    assert!(s.contains("DependentFailureAnalysis") && s.contains("DFA-CK-001"), "{s}");
}

#[test]
fn odd_shapes_duplicates_and_empty_resources_are_precise() {
    let v = run(&model(&GOOD.replace("sharedResources:\n  - resource: SoC power rail\n    kind: power\n    initiators: [brown-out]\n    couplingFactor: 0.1\n    mitigation: Separate regulators with supervisor\n", "sharedResources: SoC power rail\n")), &["validate"]);
    assert!(v.contains("E892") && !v.contains("E002"), "{v}");
    assert!(run(&model(&GOOD.replace("analyses: [Sys::Cluster, Sys::Android]", "analyses: 5")), &["validate"]).contains("analyses"));
    let dup = GOOD.replace("[Sys::Cluster, Sys::Android]", "[Sys::Cluster, Sys::Cluster]");
    assert!(run(&model(&dup), &["validate"]).contains("E890"));
    let empty = "---\nid: DFA-CK-001\ntype: DependentFailureAnalysis\nname: n\nstatus: approved\nanalyses: [Sys::Cluster, Sys::Android]\n---\n\nNothing shared?\n";
    assert!(run(&model(empty), &["validate"]).contains("W890"));
    let d = model(GOOD);
    let refs = run(&d, &["refs", "Sys::Cluster"]);
    assert!(refs.contains("DFA-CK-001"), "{refs}");
}

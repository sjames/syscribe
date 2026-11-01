//! Filters and tables of `list`, `trace`, `refs` and friends (query.rs), run against one
//! small model that has a Requirement, TestCase, AssumptionOfUse, a gated PartDef, a
//! feature with a parameter consumed by a part, and a safety goal.

use std::path::PathBuf;
use std::process::{Command, Output};

fn model() -> PathBuf {
    static M: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    M.get_or_init(build).clone()
}

fn build() -> PathBuf {
    let d = std::env::temp_dir().join(format!("syscribe_qcli_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let w = |p: &str, c: &str| {
        let f = d.join(p);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: M\n---\n");
    w("Reqs/REQ-QC-001.md", "---\nid: REQ-QC-001\ntype: Requirement\nname: Top\nstatus: approved\nreqDomain: software\nsilLevel: 2\ntags: [alpha, beta]\nderivedFromSafetyGoal: SG-QC-001\n---\nThe system shall work.\n");
    w("Reqs/REQ-QC-002.md", "---\nid: REQ-QC-002\ntype: Requirement\nname: Child\nstatus: draft\nreqDomain: software\nderivedFrom: [REQ-QC-001]\ntags: [alpha]\n---\nThe system shall also work.\n");
    w("Tests/TC-QC-001.md", "---\nid: TC-QC-001\ntype: TestCase\nname: T\ntestLevel: L2\nstatus: active\ntags: [alpha]\nverifies: [REQ-QC-002]\n---\n");
    w("Safety/SG-QC-001.md", "---\nid: SG-QC-001\ntype: SafetyGoal\nname: Goal\nasilLevel: B\n---\nGoal.\n");
    w("Assume/AOU-QC-001.md", "---\nid: AOU-QC-001\ntype: AssumptionOfUse\nname: Assume\nappliesTo: [Arch::Gated]\nstatus: draft\n---\n");
    w("Feat/Cam.md", "---\ntype: FeatureDef\nid: FEAT-QC-CAM\nname: Cam\ngroupKind: optional\nparameters:\n  - name: res\n    type: ScalarValues::Real\n    default: 1\n    consumedBy: Arch::Gated\n---\n");
    w("Arch/Gated.md", "---\ntype: PartDef\nname: Gated\nappliesWhen: Feat::Cam\nsatisfies: [REQ-QC-002]\nimplementedBy: src/x.rs\n---\n");
    w("Arch/ById.md", "---\ntype: PartDef\nname: ById\nappliesWhen: FEAT-QC-CAM\n---\n");
    w("Arch/Plain.md", "---\ntype: PartDef\nname: Plain\n---\n");
    d
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model()).args(args).output().unwrap()
}
fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn list_filters() {
    let all = out(&run(&["list", "Requirement"]));
    assert!(all.contains("REQ-QC-001") && all.contains("REQ-QC-002"));
    let tag = out(&run(&["list", "Requirement", "--tag", "alpha", "--tag", "beta"]));
    assert!(tag.contains("REQ-QC-001") && !tag.contains("REQ-QC-002"), "{tag}");
    let st = out(&run(&["list", "Requirement", "--status", "draft"]));
    assert!(st.contains("REQ-QC-002") && !st.contains("REQ-QC-001"), "{st}");
    let sil = out(&run(&["list", "Requirement", "--sil", "2"]));
    assert!(sil.contains("REQ-QC-001") && !sil.contains("REQ-QC-002"), "{sil}");
    let feat = out(&run(&["list", "PartDef", "--feature", "Feat::Cam"]));
    assert!(feat.contains("Gated") && !feat.contains("Plain"), "{feat}");
    // The feature may be named by its FEAT-* id as well as its qualified name, and the
    // gating `appliesWhen:` may use either form (Gated uses the qname, ById the id).
    let by_id = out(&run(&["list", "PartDef", "--feature", "FEAT-QC-CAM"]));
    assert!(by_id.contains("Gated") && by_id.contains("ById") && !by_id.contains("Plain"), "{by_id}");
    assert!(feat.contains("ById"), "{feat}");
    let bad = run(&["list", "PartDef", "--feature", "Nope"]);
    assert_eq!(bad.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&bad.stderr).contains("unknown feature"));
    let wcet = run(&["list", "PartDef", "--has-wcet"]);
    assert!(wcet.status.success());
    let j = run(&["list", "Requirement", "--json"]);
    serde_json::from_slice::<serde_json::Value>(&j.stdout).unwrap();
}

#[test]
fn list_type_specific_tables() {
    let tc = out(&run(&["list", "TestCase"]));
    assert!(tc.contains("| ID | Name | Level | Status | Verifies | Tags |") && tc.contains("REQ-QC-002"), "{tc}");
    let aou = out(&run(&["list", "AssumptionOfUse"]));
    assert!(aou.contains("Applies To") && aou.contains("AOU-QC-001"), "{aou}");
}

#[test]
fn trace_shows_goal_and_parameter_consumers() {
    let t = out(&run(&["trace", "REQ-QC-001"]));
    assert!(t.contains("SG-QC-001"), "{t}");
    let t2 = out(&run(&["trace", "REQ-QC-002"]));
    assert!(t2.contains("TC-QC-001") && t2.contains("Gated"), "{t2}");
    let f = out(&run(&["trace", "FEAT-QC-CAM"]));
    assert!(f.contains("Parameter consumers") && f.contains("Arch::Gated"), "{f}");
    for cmd in ["trace", "show", "links", "why", "who-verifies"] {
        let nope = run(&[cmd, "REQ-NOPE-001"]);
        assert_eq!(nope.status.code(), Some(1), "{cmd} on an unknown element must exit 1");
        assert!(String::from_utf8_lossy(&nope.stderr).contains("Element not found"), "{cmd}");
    }
}

#[test]
fn refs_who_verifies_and_find() {
    let r = run(&["refs", "REQ-QC-001"]);
    assert!(r.status.success());
    assert!(out(&r).contains("REQ-QC-002"), "{}", out(&r));
    let v = run(&["who-verifies", "REQ-QC-002"]);
    assert!(out(&v).contains("TC-QC-001"), "{}", out(&v));
    let f = run(&["find", "Gated"]);
    assert!(out(&f).contains("Gated"));
    let ext = run(&["extref", "DNG:1"]);
    assert!(ext.status.code().is_some());
}

#[test]
fn feature_check_and_configure_on_the_model() {
    let fc = run(&["feature-check"]);
    assert!(fc.status.code().is_some());
    let fcd = run(&["feature-check", "--deep"]);
    assert!(fcd.status.code().is_some());
}

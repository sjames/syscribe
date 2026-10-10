//! TC-TRS-PLEREF-001 / GH #234.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-pleref-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/CloudSync.md", "---\ntype: FeatureDef\nid: FEAT-CLOUD-SYNC\nname: CloudSync\ngroupKind: optional\n---\n\nCloud.\n");
    w("Requirements/_index.md", "---\ntype: Package\nname: Requirements\n---\n");
    w("Requirements/REQ-PL-001.md", "---\nid: REQ-PL-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nShall.\n");
    // gated test case
    w("Tests/_index.md", "---\ntype: Package\nname: Tests\n---\n");
    w("Tests/TC-PL-001.md", "---\nid: TC-PL-001\ntype: TestCase\nname: t\nstatus: draft\ntestLevel: L3\nverifies: [REQ-PL-001]\nappliesWhen: FEAT-CLOUD-SYNC\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    // ungated referrers
    w("Planning/_index.md", "---\ntype: Package\nname: Planning\n---\n");
    w("Planning/PI-PL-001.md", "---\nid: PI-PL-001\ntype: PlanningItem\nname: p\nstatus: todo\nitemType: task\nachieves: [REQ-PL-001]\nevidence:\n  - ref: TC-PL-001\n---\n\nWork.\n");
    w("Reviews/_index.md", "---\ntype: Package\nname: Reviews\n---\n");
    w("Reviews/RR-PL-001.md", "---\nid: RR-PL-001\ntype: ReviewRecord\nname: rr\nstatus: open\nreviewType: design_review\nreviews: [TC-PL-001]\n---\n\nReview.\n");
    // allocation between an ungated and a gated part
    w("Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
    w("Sys/Base.md", "---\ntype: PartDef\nname: Base\n---\n\nB.\n");
    w("Sys/Cloud.md", "---\ntype: PartDef\nname: Cloud\nappliesWhen: FEAT-CLOUD-SYNC\n---\n\nC.\n");
    w("Sys/BaseToCloud.md", "---\ntype: Allocation\nname: BaseToCloud\nallocatedFrom: [Sys::Base]\nallocatedTo: [Sys::Cloud]\n---\n\nA.\n");
    w("Configurations/CONF-ON-001.md", "---\ntype: Configuration\nid: CONF-ON-001\nname: On\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::CloudSync: true\n---\n\nOn.\n");
    w("Configurations/CONF-OFF-001.md", "---\ntype: Configuration\nid: CONF-OFF-001\nname: Off\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::CloudSync: false\n---\n\nOff.\n");
    d
}

fn run(d: &Path, args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d).args(args).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

#[test]
fn list_references_to_a_gated_element_are_w019_in_the_variant_that_drops_it() {
    let d = model();
    let off = run(&d, &["validate", "--config", "CONF-OFF-001"]);
    for bad in ["E716", "E704", "E226"] {
        assert!(!off.contains(bad), "{bad}: {off}");
    }
    assert!(off.contains("W019"), "{off}");
    assert!(off.contains("PI-PL-001") && off.contains("RR-PL-001"), "both referrers are named: {off}");
    let on = run(&d, &["validate", "--config", "CONF-ON-001"]);
    assert!(!on.contains("W019") && !on.contains("E716") && !on.contains("E704"), "{on}");
}

#[test]
fn an_allocation_inherits_the_gate_of_its_endpoints() {
    let d = model();
    let off = run(&d, &["validate", "--config", "CONF-OFF-001"]);
    assert!(!off.contains("BaseToCloud"), "the allocation is inactive in the variant, so no finding names it: {off}");
    let all = run(&d, &["validate", "--all-configs"]);
    assert!(!all.contains("E226"), "{all}");
}

#[test]
fn plain_validate_still_flags_a_reference_that_resolves_nowhere() {
    let d = model();
    std::fs::write(
        d.join("Planning/PI-PL-002.md"),
        "---\nid: PI-PL-002\ntype: PlanningItem\nname: p2\nstatus: todo\nitemType: task\nevidence:\n  - ref: TC-NOPE-999\n---\n\nWork.\n",
    )
    .unwrap();
    assert!(run(&d, &["validate"]).contains("E716"));
}

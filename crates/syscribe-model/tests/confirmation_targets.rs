//! TC-TRS-CONFM-001 / GH #233.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn findings(target_file: &str, target_md: &str) -> Vec<(String, String)> {
    static N: AtomicU64 = AtomicU64::new(0);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-confm-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(r.join(target_file), target_md).unwrap();
    std::fs::write(
        r.join("CM-X-001.md"),
        "---\ntype: ConfirmationMeasure\nid: CM-X-001\nname: cm\nstatus: draft\nmeasureType: confirmation_review\nindependenceLevel: I2\nconfirms:\n  - TARGET\n---\n\nReview.\n"
            .replace("TARGET", &target_file.trim_end_matches(".md").replace('/', "::")),
    )
    .unwrap();
    let els = walk_model(&r).unwrap();
    validate(&els).findings.into_iter().filter(|f| f.code == "E860" || f.code == "E851").map(|f| (f.code.to_string(), f.message)).collect()
}

#[test]
fn analysis_and_plan_work_products_are_accepted() {
    let cases: &[(&str, &str)] = &[
        ("FaultTree.md", "---\ntype: FaultTree\nid: FT-X-001\nname: ft\nstatus: draft\n---\n\nFT.\n"),
        ("FmeaSheet.md", "---\ntype: FMEASheet\nid: FMEA-X-001\nname: f\nstatus: draft\n---\n\nF.\n"),
        ("TaraSheet.md", "---\ntype: TARASheet\nid: TARA-X-001\nname: t\nstatus: draft\n---\n\nT.\n"),
        ("Adr.md", "---\ntype: ADR\nid: ADR-X-001\nname: a\nstatus: accepted\n---\n\nA.\n"),
        ("Plan.md", "---\ntype: TestPlan\nid: TP-XXX-001\nname: p\nstatus: draft\nscope: unit\n---\n\nP.\n"),
        ("Alloc.md", "---\ntype: Allocation\nname: Alloc\n---\n\nA.\n"),
    ];
    for (file, md) in cases {
        let f = findings(file, md);
        assert!(f.is_empty(), "{file}: {f:?}");
    }
}

#[test]
fn argument_is_accepted() {
    let f = findings("Arg.md", "---\ntype: Argument\nid: ARG-X-001\nname: a\nstatus: draft\nargumentType: claim\n---\n\nA.\n");
    assert!(f.is_empty(), "{f:?}");
}

#[test]
fn other_types_are_still_e860_and_the_message_lists_the_accepted_types() {
    let f = findings("Part.md", "---\ntype: PartDef\nname: Part\n---\n");
    assert_eq!(f.len(), 1, "{f:?}");
    assert_eq!(f[0].0, "E860");
    for t in ["SafetyGoal", "Requirement", "FaultTree", "FMEASheet", "TARASheet", "ADR", "Argument", "TestPlan", "Allocation"] {
        assert!(f[0].1.contains(t), "{}", f[0].1);
    }
}

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

fn with_confirms(files: &[(&str, &str)], confirms: &str) -> Vec<(String, String)> {
    static N: AtomicU64 = AtomicU64::new(500);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-confm2-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (f, c) in files {
        std::fs::write(r.join(f), c).unwrap();
    }
    std::fs::write(
        r.join("CM-X-001.md"),
        format!("---\ntype: ConfirmationMeasure\nid: CM-X-001\nname: cm\nstatus: draft\nmeasureType: confirmation_review\nindependenceLevel: I2\nconfirms: {confirms}\n---\n\nReview.\n"),
    )
    .unwrap();
    let els = walk_model(&r).unwrap();
    validate(&els).findings.into_iter().filter(|f| f.code == "E860" || f.code == "E851").map(|f| (f.code.to_string(), f.message)).collect()
}

const FT: (&str, &str) = ("FT-X-001.md", "---\ntype: FaultTree\nid: FT-X-001\nname: ft\nstatus: draft\n---\n\nFT.\n");
const SG: (&str, &str) = ("SG-X-001.md", "---\ntype: SafetyGoal\nid: SG-X-001\nname: g\nstatus: draft\nasilLevel: B\nsafeState: s\n---\n\nG.\n");
const PART: (&str, &str) = ("Part.md", "---\ntype: PartDef\nname: Part\n---\n");

#[test]
fn confirming_by_stable_id_works_for_the_new_types() {
    assert!(with_confirms(&[FT], "[FT-X-001]").is_empty());
}

#[test]
fn goals_hazards_and_requirements_are_still_accepted_and_a_mixed_list_reports_only_the_bad_entry() {
    let f = with_confirms(&[FT, SG, PART], "[SG-X-001, FT-X-001, Part]");
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].1.contains("'Part'"), "{f:?}");
}

#[test]
fn exploded_child_rows_are_not_confirmation_targets() {
    // You confirm the sheet, not one of its rows.
    let sheet = ("FMEA-X-001.md", "---\ntype: FMEASheet\nid: FMEA-X-001\nname: f\nstatus: draft\nentries:\n  - id: FM-XXX-001\n    failureMode: x\n---\n\nF.\n");
    let f = with_confirms(&[sheet], "[FM-XXX-001]");
    assert!(f.iter().any(|x| x.0 == "E860"), "{f:?}");
}

//! TC-TRS-PIACH-001 / GH #240.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::validator::{validate, Finding};
use syscribe_model::walker::walk_model;

fn run(target_file: &str, target_md: &str, pi_status: &str) -> Vec<Finding> {
    static N: AtomicU64 = AtomicU64::new(0);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-piach-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(r.join(target_file), target_md).unwrap();
    let target = target_file.trim_end_matches(".md");
    std::fs::write(
        r.join("PI-PA-001.md"),
        format!("---\ntype: PlanningItem\nid: PI-PA-001\nname: p\nstatus: {pi_status}\nitemType: task\nachieves: [{target}]\n{}---\n\nWork.\n",
            if pi_status == "done" { "evidence:\n  - path: repo:README.md\n" } else { "" }),
    )
    .unwrap();
    let els = walk_model(&r).unwrap();
    validate(&els).findings
}

fn n(f: &[Finding], c: &str) -> usize {
    f.iter().filter(|x| x.code == c).count()
}

#[test]
fn the_new_target_types_are_accepted() {
    let cases: &[(&str, &str)] = &[
        ("SG-PA-001.md", "---\ntype: SafetyGoal\nid: SG-PA-001\nname: g\nstatus: approved\nasilLevel: B\nsafeState: s\n---\n\nG.\n"),
        ("CSG-PA-001.md", "---\ntype: CybersecurityGoal\nid: CSG-PA-001\nname: g\nstatus: approved\ncalLevel: CAL2\n---\n\nG.\n"),
        ("ADR-PA-001.md", "---\ntype: ADR\nid: ADR-PA-001\nname: a\nstatus: accepted\n---\n\nA.\n"),
        ("ARG-PA-001.md", "---\ntype: Argument\nid: ARG-PA-001\nname: a\nstatus: approved\nargumentType: claim\n---\n\nA.\n"),
        ("TP-PAX-001.md", "---\ntype: TestPlan\nid: TP-PAX-001\nname: p\nstatus: approved\nscope: unit\n---\n\nP.\n"),
    ];
    for (file, md) in cases {
        let f = run(file, md, "in_progress");
        assert_eq!(n(&f, "E715") + n(&f, "E714"), 0, "{file}: {f:?}");
    }
}

#[test]
fn other_types_are_e715_and_the_message_lists_the_accepted_types() {
    let f = run("Part.md", "---\ntype: PartDef\nname: Part\n---\n", "in_progress");
    let m = &f.iter().find(|x| x.code == "E715").unwrap_or_else(|| panic!("{f:?}")).message;
    for t in ["Requirement", "SafetyGoal", "CybersecurityGoal", "ADR", "Argument", "TestPlan", "Baseline"] {
        assert!(m.contains(t), "{m}");
    }
}

#[test]
fn a_done_item_achieving_an_unfinished_work_product_raises_w315() {
    let draft = "---\ntype: ADR\nid: ADR-PA-001\nname: a\nstatus: proposed\n---\n\nA.\n";
    let done = "---\ntype: ADR\nid: ADR-PA-001\nname: a\nstatus: accepted\n---\n\nA.\n";
    assert_eq!(n(&run("ADR-PA-001.md", draft, "done"), "W315"), 1);
    assert_eq!(n(&run("ADR-PA-001.md", done, "done"), "W315"), 0);
    // An item still in progress is not expected to have a finished target.
    assert_eq!(n(&run("ADR-PA-001.md", draft, "in_progress"), "W315"), 0);
}

#[test]
fn a_requirement_target_keeps_the_w310_bar_and_never_w315() {
    let req = "---\ntype: Requirement\nid: REQ-PA-001\nname: r\nstatus: approved\nreqDomain: software\n---\n\nShall.\n";
    let f = run("REQ-PA-001.md", req, "done");
    assert_eq!(n(&f, "W315"), 0, "{f:?}");
    assert_eq!(n(&f, "W310"), 1, "{f:?}");
}

#[test]
fn a_baseline_target_is_accepted_and_a_missing_status_counts_as_unfinished() {
    let bl = "---\ntype: Baseline\nid: BL-PA-1\nname: b\ngitTag: v1\nseal:\n  aggregateHash: blake3:00\n  elementCount: 0\n  manifest: baselines/x.json\n---\n\nB.\n";
    let f = run("BL-PA-1.md", bl, "done");
    assert_eq!(n(&f, "E715"), 0, "{f:?}");
    assert_eq!(n(&f, "W315"), 1, "no status is unfinished: {f:?}");
}

#[test]
fn draft_and_review_goals_and_plans_are_unfinished_targets() {
    for (file, md) in [
        ("SG-PA-001.md", "---\ntype: SafetyGoal\nid: SG-PA-001\nname: g\nstatus: draft\nasilLevel: B\nsafeState: s\n---\n\nG.\n"),
        ("TP-PAX-001.md", "---\ntype: TestPlan\nid: TP-PAX-001\nname: p\nstatus: review\nscope: unit\n---\n\nP.\n"),
    ] {
        assert_eq!(n(&run(file, md, "done"), "W315"), 1, "{file}");
    }
}

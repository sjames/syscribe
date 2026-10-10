//! TC-TRS-ADRSUP-001 / GH #232.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::{validate_with_config, Finding};
use syscribe_model::walker::walk_model;

fn model(files: &[(&str, String)]) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-adrsup-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (rel, c) in files {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    r
}

fn adr(id: &str, status: &str, extra: &str) -> (String, String) {
    (
        format!("Decisions/{id}.md"),
        format!("---\ntype: ADR\nid: {id}\nname: d\nstatus: {status}\n{extra}---\n\nDecision.\n"),
    )
}

fn run(files: Vec<(String, String)>) -> Vec<Finding> {
    let fs: Vec<(&str, String)> = files.iter().map(|(a, b)| (a.as_str(), b.clone())).collect();
    let r = model(&fs);
    let els = walk_model(&r).unwrap();
    validate_with_config(&els, &ValidateConfig::with_model_root(&r)).findings
}

fn n(f: &[Finding], c: &str) -> usize {
    f.iter().filter(|x| x.code == c).count()
}

#[test]
fn supersedes_accepts_a_string_or_a_list_and_a_superseded_target_is_clean() {
    let f = run(vec![
        adr("ADR-AA-001", "superseded", ""),
        adr("ADR-AA-002", "superseded", ""),
        adr("ADR-AA-003", "accepted", "supersedes: ADR-AA-001\n"),
        adr("ADR-AA-004", "accepted", "supersedes: [ADR-AA-002]\n"),
    ]);
    assert_eq!(n(&f, "E320") + n(&f, "E321") + n(&f, "W313"), 0, "{f:?}");
    assert_eq!(n(&f, "W047"), 0, "{f:?}");
}

#[test]
fn unresolved_and_non_adr_targets_are_e320() {
    let f = run(vec![
        adr("ADR-AA-001", "accepted", "supersedes: [ADR-NOPE-001]\n"),
        (
            "REQ-AA-001.md".into(),
            "---\ntype: Requirement\nid: REQ-AA-001\nname: r\nstatus: draft\n---\n\nShall.\n".into(),
        ),
        adr("ADR-AA-002", "accepted", "supersedes: REQ-AA-001\n"),
    ]);
    assert_eq!(n(&f, "E320"), 2, "{f:?}");
}

#[test]
fn cycles_and_self_supersession_are_e321() {
    let f = run(vec![
        adr("ADR-AA-001", "superseded", "supersedes: ADR-AA-001\n"),
        adr("ADR-AA-002", "superseded", "supersedes: ADR-AA-003\n"),
        adr("ADR-AA-003", "superseded", "supersedes: ADR-AA-002\n"),
    ]);
    assert!(n(&f, "E321") >= 2, "{f:?}");
}

#[test]
fn superseding_an_adr_that_is_not_superseded_is_w313() {
    let f = run(vec![adr("ADR-AA-001", "accepted", ""), adr("ADR-AA-002", "accepted", "supersedes: ADR-AA-001\n")]);
    assert_eq!(n(&f, "W313"), 1, "{f:?}");
}

#[test]
fn a_breakdown_adr_that_is_superseded_is_w314_for_non_draft_requirements_only() {
    let req = |id: &str, status: &str| {
        (
            format!("{id}.md"),
            format!("---\ntype: Requirement\nid: {id}\nname: r\nstatus: {status}\nderivedFrom: [REQ-AA-000]\nbreakdownAdr: ADR-AA-001\n---\n\nShall.\n"),
        )
    };
    let parent = (
        "REQ-AA-000.md".to_string(),
        "---\ntype: Requirement\nid: REQ-AA-000\nname: p\nstatus: draft\n---\n\nShall.\n".to_string(),
    );
    let f = run(vec![
        adr("ADR-AA-001", "superseded", ""),
        adr("ADR-AA-002", "accepted", "supersedes: ADR-AA-001\n"),
        parent,
        req("REQ-AA-010", "approved"),
        req("REQ-AA-011", "draft"),
    ]);
    assert_eq!(n(&f, "W314"), 1, "{f:?}");
}

#[test]
fn baseline_supersedes_is_unchanged() {
    let f = run(vec![
        (
            "Baselines/BL-AA-1.md".into(),
            "---\ntype: Baseline\nid: BL-AA-1\nname: b\nstatus: draft\ngitTag: v1\nsupersedes: BL-NOPE-9\nseal:\n  aggregateHash: blake3:00\n  elementCount: 0\n  manifest: baselines/x.json\n---\n\nB.\n".into(),
        ),
    ]);
    assert_eq!(n(&f, "E522"), 1, "{f:?}");
}

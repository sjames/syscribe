//! TC-TRS-REQKIND-001 / GH #250.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn codes(kind: &str, status: &str) -> Vec<String> {
    static N: AtomicU64 = AtomicU64::new(0);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-rk-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(
        r.join("REQ-RK-001.md"),
        format!("---\ntype: Requirement\nid: REQ-RK-001\nname: r\nstatus: {status}\nrequirementKind: {kind}\nreqDomain: system\n---\n\nThe supplier shall.\n"),
    )
    .unwrap();
    let els = walk_model(&r).unwrap();
    validate(&els).findings.into_iter().map(|f| f.code.to_string()).collect()
}

#[test]
fn the_three_new_kinds_are_accepted_and_unknown_kinds_still_rejected() {
    for k in ["process", "regulatory", "deliverable"] {
        assert!(!codes(k, "draft").contains(&"E022".to_string()), "{k}");
    }
    assert!(codes("bogus", "draft").contains(&"E022".to_string()));
}

#[test]
fn approved_non_allocatable_leaves_do_not_raise_w300() {
    for k in ["process", "regulatory", "deliverable"] {
        assert!(!codes(k, "approved").contains(&"W300".to_string()), "{k}");
    }
    assert!(codes("system", "approved").contains(&"W300".to_string()));
}

#[test]
fn implemented_non_allocatable_leaves_do_not_raise_w302() {
    for k in ["process", "regulatory", "deliverable"] {
        assert!(!codes(k, "implemented").contains(&"W302".to_string()), "{k}");
    }
    assert!(codes("system", "implemented").contains(&"W302".to_string()));
}

#[test]
fn verification_coverage_is_unchanged() {
    assert!(codes("process", "approved").contains(&"W002".to_string()));
}

#[test]
fn verified_status_also_skips_w302_and_e022_lists_every_kind() {
    assert!(!codes("process", "verified").contains(&"W302".to_string()));
    assert!(codes("system", "verified").contains(&"W302".to_string()));
}

#[test]
fn e022_names_every_accepted_kind() {
    static N: AtomicU64 = AtomicU64::new(100);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-rk-msg-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(r.join("REQ-RK-001.md"), "---\ntype: Requirement\nid: REQ-RK-001\nname: r\nstatus: draft\nrequirementKind: bogus\n---\n\nShall.\n").unwrap();
    let els = walk_model(&r).unwrap();
    let f = validate(&els).findings;
    let m = &f.iter().find(|x| x.code == "E022").unwrap().message;
    for k in ["stakeholder", "system", "software", "hardware", "process", "regulatory", "deliverable"] {
        assert!(m.contains(k), "{m}");
    }
}

#[test]
fn the_traceability_diagram_does_not_badge_a_non_allocatable_leaf_as_unsatisfied() {
    use syscribe_model::resolver::Resolver;
    use syscribe_model::vis::build_graph;
    let badges = |kind: &str| -> Vec<String> {
        static N: AtomicU64 = AtomicU64::new(200);
        let r: PathBuf = std::env::temp_dir().join(format!("syscribe-rk-tr-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir_all(&r).unwrap();
        std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
        std::fs::write(r.join("SG.md"), "---\ntype: SafetyGoal\nid: SG-RK-001\nname: g\nstatus: approved\nasilLevel: B\nsafeState: stop\n---\n\nG.\n").unwrap();
        std::fs::write(
            r.join("REQ-RK-001.md"),
            format!("---\ntype: Requirement\nid: REQ-RK-001\nname: r\nstatus: approved\nrequirementKind: {kind}\nreqDomain: system\nderivedFromSafetyGoal: SG-RK-001\n---\n\nShall.\n"),
        )
        .unwrap();
        std::fs::write(r.join("D.md"), "---\ntype: Diagram\nname: D\ndiagramKind: Traceability\nsubject: SG-RK-001\n---\n\nT.\n").unwrap();
        let els = walk_model(&r).unwrap();
        let res = Resolver::new(&els);
        let d = els.iter().find(|e| e.qualified_name == "D").unwrap().clone();
        let (g, _) = build_graph(&d, &els, &res).unwrap();
        g.nodes.iter().filter_map(|n| n.mark.as_ref()).flat_map(|m| m.badges.clone()).collect()
    };
    assert!(badges("system").iter().any(|b| b.contains("W300")), "control: {:?}", badges("system"));
    assert!(!badges("process").iter().any(|b| b.contains("W300")), "{:?}", badges("process"));
}

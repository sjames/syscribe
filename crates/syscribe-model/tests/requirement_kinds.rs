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

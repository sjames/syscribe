//! GH #251 / REQ-TRS-SUS-LINKS-002: workflow-state fields are excluded from the
//! suspect-link projection; content fields are not.

use std::path::Path;
use syscribe_model::suspect::projection_hash;
use syscribe_model::walker::walk_model;

fn hash_of(fm: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-susw-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    write(&r, "PI-TST-001.md", &format!("---\ntype: PlanningItem\nid: PI-TST-001\nname: \"Item\"\nitemType: task\nachieves: [REQ-X-001]\n{fm}---\n\nBody.\n"));
    let els = walk_model(&r).unwrap();
    let e = els.iter().find(|e| e.frontmatter.id.as_deref() == Some("PI-TST-001")).expect("element");
    projection_hash(e)
}

fn write(root: &Path, rel: &str, c: &str) {
    std::fs::write(root.join(rel), c).unwrap();
}

#[test]
fn workflow_fields_do_not_change_the_hash() {
    let base = hash_of("status: todo\n");
    assert_eq!(base, hash_of("status: in_progress\n"));
    assert_eq!(base, hash_of("status: in_progress\nclaimedBy: agent-1\nclaimedAt: \"2026-01-01T00:00:00Z\"\n"));
    assert_eq!(base, hash_of("status: todo\nassignedTo: alice\n"));
}

#[test]
fn content_fields_still_change_the_hash() {
    assert_ne!(hash_of("status: todo\n"), hash_of("status: todo\ntags: [x]\n"));
}

#[test]
fn retiring_or_disposing_status_stays_in_the_projection() {
    // Progress through the lifecycle is silent, but retiring a target (or a
    // disposition such as wont_fix) is a change links must see.
    let live = hash_of("status: todo\n");
    for st in ["deprecated", "superseded", "obsolete", "rejected", "withdrawn", "wont_fix", "false_positive", "not_affected"] {
        assert_ne!(live, hash_of(&format!("status: {st}\n")), "status {st} must change the hash");
    }
    assert_ne!(hash_of("status: deprecated\n"), hash_of("status: superseded\n"));
}

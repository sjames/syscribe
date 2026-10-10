//! GH #262 review: the shared `verify_baseline` (used by the MCP tool) agrees with the CLI —
//! a superseded baseline's content drift is skipped, not failed.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::walker::walk_model;

fn model(status: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-bvs-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(r.join("Baselines")).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(
        r.join("REQ-VS-001.md"),
        "---\ntype: Requirement\nid: REQ-VS-001\nname: r\nstatus: draft\nreqDomain: software\n---\n\nThe system shall.\n",
    )
    .unwrap();
    std::fs::write(
        r.join("Baselines/BL-VS-1.md"),
        format!(
            "---\ntype: Baseline\nid: BL-VS-1\nname: b\nstatus: {status}\ngitTag: v1\nseal:\n  aggregateHash: blake3:00\n  elementCount: 1\n  manifest: baselines/none.json\n---\n\nBaseline.\n"
        ),
    )
    .unwrap();
    r
}

fn verify(status: &str) -> syscribe_model::baseline::VerifyResult {
    let r = model(status);
    let els = walk_model(&r).unwrap();
    let b = els.iter().find(|e| e.frontmatter.id.as_deref() == Some("BL-VS-1")).unwrap();
    syscribe_model::baseline::verify_baseline(&els, b, &r)
}

#[test]
fn drift_fails_a_live_baseline() {
    let v = verify("draft");
    assert!(!v.passed, "{v:?}");
    assert!(v.messages.iter().any(|m| m.contains("content drift")), "{v:?}");
}

#[test]
fn drift_is_skipped_for_a_superseded_baseline() {
    let v = verify("superseded");
    assert!(v.passed, "{v:?}");
    assert!(v.skipped, "{v:?}");
}

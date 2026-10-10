//! TC-TRS-ADRSUP-001 (show and links) / GH #232.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-adrcli-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    std::fs::create_dir_all(r.join("Decisions")).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(r.join("Decisions/ADR-AA-001.md"), "---\ntype: ADR\nid: ADR-AA-001\nname: Old\nstatus: superseded\n---\n\nOld.\n").unwrap();
    std::fs::write(
        r.join("Decisions/ADR-AA-002.md"),
        "---\ntype: ADR\nid: ADR-AA-002\nname: New\nstatus: accepted\nsupersedes: [ADR-AA-001]\n---\n\nNew.\n",
    )
    .unwrap();
    r
}

fn run(root: &Path, args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn show_prints_both_directions() {
    let r = model();
    let old = run(&r, &["show", "ADR-AA-001"]);
    assert!(old.contains("supersededBy") && old.contains("ADR-AA-002"), "{old}");
    let new = run(&r, &["show", "ADR-AA-002"]);
    assert!(new.contains("supersedes") && new.contains("ADR-AA-001"), "{new}");
}

#[test]
fn links_lists_the_supersedes_edge_both_ways() {
    let r = model();
    let new = run(&r, &["links", "ADR-AA-002"]);
    assert!(new.contains("supersedes") && new.contains("ADR-AA-001"), "{new}");
    let old = run(&r, &["links", "ADR-AA-001"]);
    assert!(old.contains("ADR-AA-002") && old.contains("supersedes"), "inbound: {old}");
}

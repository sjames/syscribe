//! TC-TRS-CLIFIX-001 / GH #231.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir()
        .join(format!("syscribe-clifix-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)))
        .join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Requirements/_index.md", "---\ntype: Package\nname: Requirements\n---\n");
    w("Requirements/System/_index.md", "---\ntype: Package\nname: System\n---\n");
    w(
        "Requirements/System/REQ-A-001.md",
        "---\ntype: Requirement\nid: REQ-A-001\nname: A\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nBody.\n",
    );
    w(
        "Arch/Thing.md",
        "---\ntype: PartDef\nname: Thing\nimplementedBy:\n  - https://example.com/x\n  - crates.io:serde@1.0.200\n---\n",
    );
    r
}

fn run(root: &Path, args: &[&str]) -> (String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned())
}

#[test]
fn behavioral_coverage_header_has_no_placeholder() {
    let (out, _) = run(&model(), &["behavioral-coverage"]);
    assert!(out.contains("Behavioral Coverage"), "{out}");
    assert!(!out.contains("<model>"), "{out}");
}

#[test]
fn links_marks_remote_implemented_by_as_external() {
    let (out, _) = run(&model(), &["links", "Arch::Thing"]);
    assert!(out.contains("https://example.com/x | external"), "{out}");
    assert!(out.contains("crates.io:serde@1.0.200 | external"), "{out}");
    assert!(!out.contains("(unresolved)"), "{out}");
}

#[test]
fn ls_accepts_a_slash_path() {
    let (out, _) = run(&model(), &["ls", "Requirements/System"]);
    assert!(out.contains("REQ-A-001") || out.contains("Requirements::System::REQ-A-001"), "{out}");
}

#[test]
fn ls_unknown_scope_hints_at_the_qualified_form() {
    let (_, err) = run(&model(), &["ls", "Requirements/Syste"]);
    assert!(err.contains("No children found"), "{err}");
    let (_, err) = run(&model(), &["ls", "Requirements/System/Nope"]);
    assert!(err.contains("Requirements::System"), "hint expected: {err}");
}

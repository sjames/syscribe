//! Regression test for `show`'s "Related:" footer (issue #117): surfacing
//! the traceability commands (`trace`/`impact`/`who-verifies`/`refs`) that
//! already exist and answer the natural next questions about the shown
//! element, type-appropriately, suppressible with `--no-related`.
//!
//! Black-box: builds a minimal, self-contained model in a fresh temp
//! directory per test and drives the `syscribe` binary against it.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_model() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir()
        .join(format!("syscribe-w117-{}-{}-{}", std::process::id(), nanos, n))
        .join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: ShowRelated\n---\n\nModel root.\n");
    root
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn run(root: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn syscribe");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn requirement_suggests_trace_who_verifies_impact_refs() {
    let root = new_model();
    write(
        &root,
        "REQ-SR-001.md",
        "---\ntype: Requirement\nid: REQ-SR-001\nname: \"A requirement\"\nstatus: draft\nreqDomain: system\n---\n\nBody.\n",
    );
    let stdout = run(&root, &["show", "REQ-SR-001"]);
    assert!(stdout.contains("Related:"), "{stdout}");
    assert!(stdout.contains("syscribe trace          REQ-SR-001"), "{stdout}");
    assert!(stdout.contains("syscribe who-verifies   REQ-SR-001"), "{stdout}");
    assert!(stdout.contains("syscribe impact         REQ-SR-001"), "{stdout}");
    assert!(stdout.contains("syscribe refs           REQ-SR-001"), "{stdout}");
    // Architecture-only commands must not appear on a Requirement.
    assert!(!stdout.contains("connectivity"), "{stdout}");
    assert!(!stdout.contains(" n2 "), "{stdout}");
}

#[test]
fn architecture_element_suggests_impact_connectivity_n2_refs_not_requirement_specific() {
    let root = new_model();
    write(&root, "Widget.md", "---\ntype: PartDef\nname: Widget\n---\n\nBody.\n");
    let stdout = run(&root, &["show", "Widget"]);
    assert!(stdout.contains("Related:"), "{stdout}");
    assert!(stdout.contains("syscribe impact         Widget"), "{stdout}");
    assert!(stdout.contains("syscribe connectivity   Widget"), "{stdout}");
    assert!(stdout.contains("syscribe n2             Widget"), "{stdout}");
    assert!(stdout.contains("syscribe refs           Widget"), "{stdout}");
    // Requirement-only commands must not appear on a PartDef.
    assert!(!stdout.contains(" trace "), "{stdout}");
    assert!(!stdout.contains("who-verifies"), "{stdout}");
}

#[test]
fn no_related_flag_suppresses_the_footer() {
    let root = new_model();
    write(
        &root,
        "REQ-SR-002.md",
        "---\ntype: Requirement\nid: REQ-SR-002\nname: \"Another requirement\"\nstatus: draft\nreqDomain: system\n---\n\nBody.\n",
    );
    let stdout = run(&root, &["show", "REQ-SR-002", "--no-related"]);
    assert!(!stdout.contains("Related:"), "{stdout}");
    assert!(!stdout.contains("syscribe trace"), "{stdout}");
}

//! Regression coverage for task #12: the validate-report's `## 7. Satisfaction
//! Links` / "7.1 Elements with `satisfies`" table filtered its displayed
//! `satisfies` targets through `is_req_id`, silently hiding a qname-form
//! target even though it resolves correctly and correctly suppresses W300
//! elsewhere. The display column must show a `satisfies:` target regardless
//! of whether it is id-form (`REQ-*`) or qname-form (`Package::Element`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-report-satisfies-test-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// Run `syscribe -m <root>` (default: prints the markdown validation report to
/// stdout) and return stdout.
fn run_report(root: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .output()
        .expect("spawn syscribe");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn qname_form_satisfies_target_appears_in_report_table() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Requirements/SomeRequirement.md",
        "---\ntype: Requirement\nid: REQ-SOME-001\nname: Some Requirement\nreqDomain: software\nstatus: approved\n---\n\nBody.\n",
    );
    write(
        &root,
        "Arch/SomePart.md",
        "---\ntype: PartDef\nname: SomePart\ndomain: software\nsatisfies:\n  - Requirements::SomeRequirement\n---\n",
    );

    let stdout = run_report(&root);
    assert!(
        stdout.contains("## 7. Satisfaction Links"),
        "report should contain the Satisfaction Links section:\n{stdout}"
    );
    // The row for Arch::SomePart must show the qname-form target, not "—".
    let row = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("| Arch::SomePart |"))
        .unwrap_or_else(|| panic!("expected a table row for Arch::SomePart in report:\n{stdout}"));
    assert!(
        row.contains("Requirements::SomeRequirement"),
        "qname-form satisfies target should be displayed in the row, got: {row}"
    );
    assert!(
        !row.trim_end().ends_with("| — |"),
        "row should not show '—' when a qname-form satisfies target exists: {row}"
    );
}

/// Control: the id-form (`REQ-*`) target must keep displaying as before.
#[test]
fn id_form_satisfies_target_still_appears_in_report_table() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Requirements/SomeRequirement.md",
        "---\ntype: Requirement\nid: REQ-SOME-002\nname: Some Requirement\nreqDomain: software\nstatus: approved\n---\n\nBody.\n",
    );
    write(
        &root,
        "Arch/SomePart2.md",
        "---\ntype: PartDef\nname: SomePart2\ndomain: software\nsatisfies:\n  - REQ-SOME-002\n---\n",
    );

    let stdout = run_report(&root);
    let row = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("| Arch::SomePart2 |"))
        .unwrap_or_else(|| panic!("expected a table row for Arch::SomePart2 in report:\n{stdout}"));
    assert!(
        row.contains("REQ-SOME-002"),
        "id-form satisfies target should still be displayed in the row, got: {row}"
    );
}

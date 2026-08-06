//! Regression coverage for task #13: `query.rs`'s `type_label` had a match arm
//! for `ElementType::AttributeDef` but none for `ElementType::Attribute`, so
//! `syscribe types`/`list` mislabeled every `Attribute`-typed (usage, not def)
//! element as "Other" (falling through the catch-all arm).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-type-label-test-{}-{}",
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

fn run(root: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn syscribe");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn model_with_attribute() -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Arch/Mass.md",
        "---\ntype: Attribute\nname: mass\n---\n",
    );
    root
}

#[test]
fn types_command_labels_attribute_correctly() {
    let root = model_with_attribute();
    let (stdout, stderr, code) = run(&root, &["types"]);
    assert_eq!(code, 0, "types exits 0 (stderr: {stderr})");
    assert!(
        stdout.lines().any(|l| l.trim_start().starts_with("| Attribute |")),
        "expected an 'Attribute' row in `types` output, got:\n{stdout}"
    );
    assert!(
        !stdout.lines().any(|l| l.trim_start().starts_with("| Other |")),
        "the Attribute-typed element should not fall through to 'Other':\n{stdout}"
    );
}

#[test]
fn list_command_finds_attribute_typed_element() {
    let root = model_with_attribute();
    let (stdout, stderr, code) = run(&root, &["list", "Attribute"]);
    assert_eq!(code, 0, "list exits 0 (stderr: {stderr})");
    assert!(
        stdout.contains("Arch::mass") || stdout.contains("mass"),
        "expected the Attribute-typed element 'mass' to be found by `list Attribute`, got:\n{stdout}"
    );
}

//! Integration tests for native SysML v2/KerML submodel scoping and ingestion
//! (`ADR-SYS-SYSMLV2-001`, `REQ-TRS-SYSMLV2-001`, `REQ-TRS-SYSMLV2-002`,
//! `REQ-TRS-SYSMLV2-007`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

/// A unique throwaway directory under the OS temp dir (mirrors the WASM-plugin
/// tests' hand-rolled helper — this crate does not depend on `tempfile`).
fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-test-{}-{}",
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

#[test]
fn unmapped_sysml_constructs_stay_invisible_while_the_package_anchor_still_parses() {
    // REQ-TRS-SYSMLV2-007's fixed mapped set is Package, Part(Def/Usage),
    // Attribute(Def/Usage), Port(Def/Usage), Connection(Def/Usage),
    // Interface(Def/Usage), Item(Def/Usage), Requirement(Def/Usage),
    // AllocationUsage, variation/variant membership, and — as of
    // REQ-TRS-SYSMLV2-018/-019 — State(Def/Usage)/Action(Def/Usage). `calc
    // def`/`case def`/`analysis def`/`verification def`/`constraint def`
    // stay outside that set (`REQ-TRS-SYSMLV2-000`'s explicitly deferred
    // scope) — `calc def` here must keep synthesizing zero elements no
    // matter how much of the fixed set later commits add support for.
    // Wrapped in a real `package { ... }` (not bare file-root content) since
    // `merge_root` only merges `RootElement::Package` in the first place —
    // a bare root-level construct is invisible for that separate, unrelated
    // reason regardless of its own kind, so it wouldn't isolate this test's
    // actual subject (kind-mapping, not root-level wrapping).
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Sensor.sysml",
        "package Behavior {\n    calc def ComputeMargin;\n}\n",
    );

    let elements = walk_model(&root).unwrap();

    assert!(
        !elements.iter().any(|e| e.qualified_name.ends_with("ComputeMargin")),
        "calc def is outside the fixed mapped set and must stay invisible: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    // The package's own _index.md, and the Behavior package the calc def
    // lives in, are still normal, fully-mapped elements.
    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy"));
    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy::Behavior"));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn state_def_and_action_def_are_now_mapped_when_package_wrapped() {
    // The mirror image of the test above: REQ-TRS-SYSMLV2-018/-019 moved
    // State/Action into the fixed mapped set, so — unlike `calc def` —
    // these now synthesize real elements once actually reached (i.e. found
    // inside a real `package { ... }`, not at bare file root; see the
    // comment above on why bare-root content is invisible for an unrelated
    // reason).
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Behavior.sysml",
        "package Behavior {\n    state def Idle;\n    action def DoNothing;\n}\n",
    );

    let elements = walk_model(&root).unwrap();
    let qnames: Vec<&str> = elements.iter().map(|e| e.qualified_name.as_str()).collect();
    assert!(qnames.contains(&"SysML2Legacy::Behavior::Idle"), "{qnames:#?}");
    assert!(qnames.contains(&"SysML2Legacy::Behavior::DoNothing"), "{qnames:#?}");

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn non_sysml_files_are_ignored_without_error() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(&root, "SysML2Legacy/README.txt", "not a model file\n");
    write(&root, "SysML2Legacy/diagram.svg", "<svg></svg>\n");

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
    assert!(elements
        .iter()
        .all(|e| !e.file_path.ends_with("README.txt") && !e.file_path.ends_with("diagram.svg")));
}

#[test]
fn stray_nested_index_md_warns_and_is_not_processed_as_a_package() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    // A stray nested _index.md, several levels deep, inside the marked subtree.
    write(
        &root,
        "SysML2Legacy/Nested/Deeper/_index.md",
        "---\ntype: Package\nname: Deeper\n---\n",
    );

    let elements = walk_model(&root).unwrap();

    // Not processed as a package: no element carries the nested qualified name.
    assert!(!elements
        .iter()
        .any(|e| e.qualified_name.contains("Deeper")));

    let result = validate(&elements);
    let w540: Vec<_> = result.findings.iter().filter(|f| f.code == "W540").collect();
    assert_eq!(w540.len(), 1, "expected exactly one W540, got: {w540:#?}");
    assert!(w540[0].file.contains("Deeper"));
    assert_eq!(result.errors().count(), 0);
}

#[test]
fn a_sysml_submodel_package_nested_inside_another_escapes_exclusion_entirely_is_fixed() {
    // Regression: a sysmlSubmodel: true package nested inside another
    // sysmlSubmodel: true package must not survive as its own live, processed
    // Package graph element — it is a stray within the outer subtree, exactly
    // like an unmarked nested _index.md would be (REQ-TRS-SYSMLV2-001: "no
    // nested _index.md is expected or processed anywhere inside the marked
    // subtree").
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Outer/_index.md",
        "---\ntype: Package\nname: Outer\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "Outer/Inner/_index.md",
        "---\ntype: Package\nname: Inner\nsysmlSubmodel: true\n---\n",
    );
    write(&root, "Outer/Inner/Thing.sysml", "part def Thing {\n}\n");

    let elements = walk_model(&root).unwrap();

    // Inner does not survive as a live, separately-processed Package element.
    assert!(
        !elements.iter().any(|e| e.qualified_name == "Outer::Inner"),
        "Outer::Inner should have been excluded as a stray, not processed as its own package: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    // The outer anchor itself is still a normal native element.
    assert!(elements.iter().any(|e| e.qualified_name == "Outer"));

    let result = validate(&elements);
    let w540: Vec<_> = result.findings.iter().filter(|f| f.code == "W540").collect();
    assert_eq!(w540.len(), 1, "expected exactly one W540 for the nested anchor, got: {w540:#?}");
    assert!(w540[0].file.contains("Inner"));
    assert_eq!(result.errors().count(), 0);
}

#[test]
fn hand_authored_md_siblings_still_parse_normally() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Sensor.sysml",
        "part def PressureSensor {\n}\n",
    );
    write(
        &root,
        "SysML2Legacy/HandWritten.md",
        "---\ntype: PartDef\nname: HandWritten\n---\n",
    );

    let elements = walk_model(&root).unwrap();

    let hand_written = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::HandWritten");
    assert!(
        hand_written.is_some(),
        "hand-authored .md sibling should still parse and join the package namespace: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn model_with_no_sysml_submodel_package_is_unaffected() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Plain/_index.md",
        "---\ntype: Package\nname: Plain\n---\n",
    );
    write(
        &root,
        "Plain/Nested/_index.md",
        "---\ntype: Package\nname: Nested\n---\n",
    );
    write(
        &root,
        "Plain/Nested/Thing.md",
        "---\ntype: PartDef\nname: Thing\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    // No sysmlSubmodel package anywhere: no W540 findings, nested packages parse normally.
    assert!(result.findings.iter().all(|f| f.code != "W540"));
    assert!(elements
        .iter()
        .any(|e| e.qualified_name == "Plain::Nested::Thing"));
    assert!(elements.iter().any(|e| e.qualified_name == "Plain::Nested"));
}

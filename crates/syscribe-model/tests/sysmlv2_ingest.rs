//! Integration tests for native SysML v2/KerML parsing + graph merge
//! (`REQ-TRS-SYSMLV2-002`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::ElementType;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-ingest-test-{}-{}",
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
fn a_sysml_package_becomes_a_qname_mapped_package_element() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(&root, "SysML2Legacy/Sensors.sysml", "package Sensors { }\n");

    let elements = walk_model(&root).unwrap();

    let pkg = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Sensors")
        .unwrap_or_else(|| {
            panic!(
                "expected SysML2Legacy::Sensors, got: {:#?}",
                elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
            )
        });
    assert_eq!(pkg.frontmatter.element_type, Some(ElementType::Package));
    assert!(pkg.file_path.ends_with("Sensors.sysml"));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn nested_sysml_packages_derive_a_double_colon_qname() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Nested.sysml",
        "package Outer { package Inner { } }\n",
    );

    let elements = walk_model(&root).unwrap();

    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy::Outer"));
    assert!(elements
        .iter()
        .any(|e| e.qualified_name == "SysML2Legacy::Outer::Inner"));
}

#[test]
fn two_files_declaring_the_same_package_merge_into_one_namespace() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    // Two separate files both contribute to the same `Shared` SysML v2 package.
    write(&root, "SysML2Legacy/PartA.sysml", "package Shared { package Left { } }\n");
    write(&root, "SysML2Legacy/PartB.sysml", "package Shared { package Right { } }\n");

    let elements = walk_model(&root).unwrap();

    // Exactly one Shared package element — not two colliding on qname.
    let shared: Vec<_> = elements
        .iter()
        .filter(|e| e.qualified_name == "SysML2Legacy::Shared")
        .collect();
    assert_eq!(
        shared.len(),
        1,
        "same-named package across two files should merge into one element, got: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    // Both files' nested content landed inside the merged namespace.
    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy::Shared::Left"));
    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy::Shared::Right"));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors (e.g. a spurious E108): {:#?}", result.findings);
}

#[test]
fn a_parse_failure_in_one_file_does_not_abort_the_rest_of_the_subtree() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    // Malformed: unbalanced braces, should fail to parse.
    write(&root, "SysML2Legacy/Broken.sysml", "package Broken { part def X {\n");
    // A second, well-formed file in the same subtree.
    write(&root, "SysML2Legacy/Good.sysml", "package Good { }\n");

    let elements = walk_model(&root).unwrap();

    // The good file's package still made it into the graph.
    assert!(
        elements.iter().any(|e| e.qualified_name == "SysML2Legacy::Good"),
        "good file's package should still be ingested: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    // Nothing at all came from the broken file.
    assert!(!elements.iter().any(|e| e.qualified_name.contains("Broken")));

    let result = validate(&elements);
    let w541: Vec<_> = result.findings.iter().filter(|f| f.code == "W541").collect();
    assert_eq!(w541.len(), 1, "expected exactly one W541 for the broken file, got: {w541:#?}");
    assert!(w541[0].file.contains("Broken.sysml"));
    // A parse failure is a warning, never an error — never aborts the rest of validate.
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

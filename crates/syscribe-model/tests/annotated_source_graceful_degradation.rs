//! End-to-end tests for annotated-source comment-marker ingestion
//! (`ADR-SYS-ANNOTATE-001`) through the public `walk_model`/`validate` API —
//! mirrors `stdio_plugins_graceful_degradation.rs`'s shape for the sibling
//! mechanism. Unit-level coverage of `apply_annotation_scans` itself lives in
//! `crates/syscribe-model/src/annotations.rs`'s own `#[cfg(test)]` module;
//! these tests confirm the whole pipeline (walker hook -> validator) holds.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-annotated-source-degrade-test-{}-{}",
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
fn malformed_annotation_config_is_e560_rest_of_model_validates_normally() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Firmware/_index.md",
        // annotationFormat: set, but no marker:/include: — fatal config error.
        "---\ntype: Package\nname: Firmware\nannotationFormat: c-linecomment\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    let e560: Vec<_> = result.findings.iter().filter(|f| f.code == "E560").collect();
    assert_eq!(e560.len(), 1, "expected exactly one E560: {:#?}", result.findings);
    assert_eq!(
        result.errors().count(),
        1,
        "only the E560 itself, nothing else: {:#?}",
        result.findings
    );
}

#[test]
fn well_formed_marker_merges_and_the_model_validates_clean() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Firmware/_index.md",
        "---\ntype: Package\nname: Firmware\nannotationFormat: c-linecomment\nmarker: '//\\s*@syscribe\\b'\ninclude: [\"**/*.c\"]\n---\n",
    );
    write(
        &root,
        "Firmware/engine.c",
        "// @syscribe\n// type: Part\n// name: EngineController\n// doc: Engine controller.\n",
    );

    let elements = walk_model(&root).unwrap();
    assert!(
        elements.iter().any(|e| e.qualified_name == "Firmware::EngineController"),
        "marker-synthesized element should be present: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn one_malformed_marker_degrades_gracefully_never_aborts_validate() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Firmware/_index.md",
        "---\ntype: Package\nname: Firmware\nannotationFormat: c-linecomment\nmarker: '//\\s*@syscribe\\b'\ninclude: [\"**/*.c\"]\n---\n",
    );
    // One marker with unterminated YAML, one well-formed sibling.
    write(
        &root,
        "Firmware/engine.c",
        "// @syscribe\n// type: Part\n// name: [unterminated\nint x;\n// @syscribe\n// type: Part\n// name: Good\n",
    );
    // An unrelated, ordinary requirement elsewhere in the model — must be
    // unaffected by the malformed marker living in a different package.
    write(
        &root,
        "Requirements/REQ-100.md",
        "---\ntype: Requirement\nid: REQ-100\nname: \"Unrelated requirement\"\nstatus: draft\nreqDomain: system\nreqClass: system\n---\n\nSomething shall happen.\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    let e561: Vec<_> = result.findings.iter().filter(|f| f.code == "E561").collect();
    assert_eq!(e561.len(), 1, "expected exactly one E561: {:#?}", result.findings);
    assert!(
        elements.iter().any(|e| e.qualified_name == "Firmware::Good"),
        "well-formed sibling marker still merges despite its neighbor's malformed YAML: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    assert!(
        elements.iter().any(|e| e.qualified_name == "Requirements::REQ-100"),
        "an unrelated package elsewhere in the model is unaffected: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
}

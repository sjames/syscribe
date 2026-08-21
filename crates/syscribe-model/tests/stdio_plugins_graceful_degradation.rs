//! End-to-end tests for stdio-subprocess foreign-format plugins
//! (`ADR-SYS-PLUGIN-002`) through the public `walk_model`/`validate` API —
//! mirrors `sysmlv2_graceful_degradation.rs`'s shape for the sibling
//! mechanism. Unit-level coverage of `apply_foreign_plugins` itself lives in
//! `crates/syscribe-model/src/plugins/mod.rs`'s own `#[cfg(test)]` module;
//! these tests confirm the whole pipeline (walker hook -> validator) holds.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-stdio-plugins-degrade-test-{}-{}",
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
fn missing_plugins_entry_is_e551_rest_of_model_validates_normally() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Legacy/_index.md",
        "---\ntype: Package\nname: Legacy\nforeignFormat: toydsl\n---\n",
    );
    // No `.syscribe.toml` at all -> no `[plugins.toydsl]` entry.

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    let e551: Vec<_> = result.findings.iter().filter(|f| f.code == "E551").collect();
    assert_eq!(e551.len(), 1, "expected exactly one E551: {:#?}", result.findings);
    // Never panics, and doesn't cascade into unrelated errors.
    assert_eq!(
        result.errors().count(),
        1,
        "only the E551 itself, nothing else: {:#?}",
        result.findings
    );
}

#[test]
fn well_formed_plugin_merges_and_the_model_validates_clean() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Legacy/_index.md",
        "---\ntype: Package\nname: Legacy\nforeignFormat: toydsl\n---\n",
    );
    write(
        &root,
        ".syscribe.toml",
        "[plugins.toydsl]\ncommand = \"/bin/sh\"\nargs = [\"-c\", \"cat >/dev/null; echo '{\\\"elements\\\":[{\\\"qname\\\":\\\"Widget\\\",\\\"type\\\":\\\"PartDef\\\"}]}'\"]\n",
    );

    let elements = walk_model(&root).unwrap();
    assert!(
        elements.iter().any(|e| e.qualified_name == "Legacy::Widget"),
        "plugin-synthesized element should be present: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn plugin_process_failure_degrades_gracefully_never_aborts_validate() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Legacy/_index.md",
        "---\ntype: Package\nname: Legacy\nforeignFormat: toydsl\n---\n",
    );
    write(
        &root,
        ".syscribe.toml",
        "[plugins.toydsl]\ncommand = \"/bin/sh\"\nargs = [\"-c\", \"cat >/dev/null; exit 1\"]\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    let w550: Vec<_> = result.findings.iter().filter(|f| f.code == "W550").collect();
    assert_eq!(w550.len(), 1, "expected exactly one W550: {:#?}", result.findings);
    assert_eq!(
        result.errors().count(),
        0,
        "a plugin failure is a warning, not an error, and never aborts validate: {:#?}",
        result.findings
    );
}

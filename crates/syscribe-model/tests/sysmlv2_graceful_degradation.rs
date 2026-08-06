//! Audit tests for `REQ-TRS-SYSMLV2-006`: a SysMLv2 ingestion failure degrades
//! gracefully, under its own error/warning code range, and never aborts
//! `validate`.
//!
//! This is deliberately an *audit*, not new feature work — `W540` (stray
//! nested `_index.md`, `sysmlv2/mod.rs`) and `W541` (parse/read failure,
//! `sysmlv2/ingest.rs`) already existed and are already distinct from the
//! WASM-plugin family (`E530`–`E532`/`W530`–`W534`, which don't even exist on
//! this branch — that family shipped on `feat/wasm-plugins`, a separate,
//! unmerged branch this one forked before). These tests confirm the existing
//! mechanisms actually hold rather than building anything new.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-degrade-test-{}-{}",
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
fn malformed_sysml_submodel_string_value_degrades_via_the_generic_e002_path() {
    // `sysmlSubmodel:` is `Option<bool>` on `RawFrontmatter`; a non-bool YAML
    // value fails the *whole file's* `serde_yaml` deserialize (not a
    // SysMLv2-specific check at all), which `walker.rs` already turns into
    // `ParseIssue::YamlError` -> `E002` for any file, unconditionally. No new
    // handling needed or added — this just proves the existing generic path
    // covers this case, exactly as REQ-TRS-SYSMLV2-006 expects.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: \"yes\"\n---\n",
    );
    write(&root, "SysML2Legacy/Sensor.sysml", "part def PressureSensor {\n}\n");

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    let e002: Vec<_> = result.findings.iter().filter(|f| f.code == "E002").collect();
    assert_eq!(e002.len(), 1, "expected exactly one E002, got: {:#?}", result.findings);
    assert!(e002[0].file.ends_with("SysML2Legacy/_index.md"));
    // Degrades gracefully: the package's own element still exists (frontmatter
    // defaults to empty on parse failure, per the existing generic path), and
    // — since `sysmlSubmodel` didn't parse as `true` — the subtree is simply
    // not treated as a SysMLv2 submodel at all; nothing from Sensor.sysml is
    // synthesized, but nothing panics or aborts the rest of validate either.
    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy"));
}

#[test]
fn malformed_sysml_submodel_integer_value_degrades_the_same_way() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: 1\n---\n",
    );
    write(&root, "SysML2Legacy/Sensor.sysml", "part def PressureSensor {\n}\n");

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    let e002: Vec<_> = result.findings.iter().filter(|f| f.code == "E002").collect();
    assert_eq!(e002.len(), 1, "expected exactly one E002, got: {:#?}", result.findings);
}

#[test]
fn empty_sysml_file_parses_cleanly_to_zero_elements() {
    // Not a failure at all -- an empty file is a syntactically valid, empty
    // RootNamespace. Included as a no-panic guard for a genuinely weird input.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(&root, "SysML2Legacy/Empty.sysml", "");

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    assert!(!result.findings.iter().any(|f| f.code == "W541"));
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn whitespace_only_sysml_file_parses_cleanly() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(&root, "SysML2Legacy/Whitespace.sysml", "   \n\n\t\n  ");

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    assert!(!result.findings.iter().any(|f| f.code == "W541"));
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn sysml_submodel_package_with_zero_sysml_files_is_fine() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    // Zero .sysml/.kerml files anywhere in the subtree.

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy"));
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn invalid_utf8_file_degrades_via_w541_read_error_not_the_parse_error_branch() {
    // Distinct code path from a syntax parse failure: `find_sysml_files`'s
    // `std::fs::read_to_string` itself fails first. Confirms the *other* half
    // of `ingest_subtree`'s two W541 call sites (read failure vs. parse
    // failure) also degrades gracefully, and that a sibling good file in the
    // same subtree is unaffected.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(&root, "SysML2Legacy/Good.sysml", "package Good { }\n");
    std::fs::write(
        root.join("SysML2Legacy/Bad.sysml"),
        [0x50u8, 0x61, 0x63, 0xFF, 0xFE, 0x00, 0x81],
    )
    .unwrap();

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    let w541: Vec<_> = result.findings.iter().filter(|f| f.code == "W541").collect();
    assert_eq!(w541.len(), 1, "expected exactly one W541, got: {:#?}", result.findings);
    assert!(w541[0].file.ends_with("Bad.sysml"));
    assert!(w541[0].message.contains("could not read"));
    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy::Good"));
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn multi_file_merge_can_silently_duplicate_a_qname_known_gap_not_fixed_here() {
    // Explicitly out of scope for REQ-TRS-SYSMLV2-006 (per this task's
    // instructions): `E108` (duplicate qname, any origin) doesn't exist on
    // this branch at all -- it shipped with the WASM-plugin feature on a
    // separate, unmerged branch this one forked before. This test documents
    // a genuine duplicate-qname collision possible purely within SysMLv2
    // multi-file merge itself (two files each contributing a same-named
    // `part def` to the same merged package produce two `RawElement`s at the
    // identical qname, with nothing today to catch it) as a known finding
    // for later -- NOT a regression to fix as part of this task. It does not
    // panic or abort validate either way.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(&root, "SysML2Legacy/A.sysml", "package Foo {\npart def Bar;\n}\n");
    write(&root, "SysML2Legacy/B.sysml", "package Foo {\npart def Bar;\n}\n");

    let elements = walk_model(&root).unwrap();
    let matches: Vec<_> = elements
        .iter()
        .filter(|e| e.qualified_name == "SysML2Legacy::Foo::Bar")
        .collect();
    assert_eq!(
        matches.len(),
        2,
        "documents the known gap: two files each contributing 'part def Bar' \
         under the same merged package currently produce two elements at the \
         same qname with no diagnostic; got: {matches:#?}"
    );

    // Whatever the eventual fix, it must not panic today.
    let result = validate(&elements);
    assert_eq!(
        result.errors().count(),
        0,
        "no error is currently raised for this collision (that's the gap): {:#?}",
        result.findings
    );
}

//! Integration tests for mapping `enum def`/`enum` onto the native
//! `EnumerationDef`/`Enumeration` schema (`REQ-TRS-SYSMLV2-025`).
//!
//! Mirrors `sysmlv2_flows.rs`. Covers the real, distinguishing fact of this
//! increment: `EnumerationBody` carries no `Doc` variant at all, unlike
//! every other body type mapped so far — confirmed empirically, not
//! assumed.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::ElementType;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-enums-test-{}-{}",
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

fn base_model(root: &Path) {
    write(root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
}

#[test]
fn an_enum_def_lifts_values_and_supertype_with_no_doc() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Enums.sysml",
        "package Enums {\n\
         enum def BaseKind;\n\
         enum def ArmStatus :> BaseKind {\n\
         doc /* Arming state. */\n\
         enum disarmed;\n\
         armed;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Enums::ArmStatus")
        .expect("ArmStatus should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::EnumerationDef));
    assert_eq!(el.frontmatter.supertype.as_ref().and_then(|v| v.as_str()), Some("BaseKind"));
    let values = el.frontmatter.values.as_deref().expect("values present");
    let names: Vec<&str> = values.iter().filter_map(|v| v.get("name").and_then(|n| n.as_str())).collect();
    assert_eq!(names, vec!["disarmed", "armed"], "{values:#?}");
    // EnumerationBody carries no Doc variant at all -- confirmed empirically,
    // a doc member is structurally unreachable, so `doc` stays "".
    assert_eq!(el.doc, "", "{:?}", el.doc);
}

#[test]
fn an_enum_literal_with_an_initializer_keeps_only_its_name() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Enums.sysml",
        "package Enums {\n\
         enum def LevelEnum { low = 0.25; medium = 0.5; high = 0.75; }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements.iter().find(|e| e.qualified_name == "SysML2Legacy::Enums::LevelEnum").unwrap();
    let values = el.frontmatter.values.as_deref().unwrap();
    assert_eq!(values.len(), 3, "{values:#?}");
    for v in values {
        let m = v.as_mapping().expect("each value is a map");
        assert_eq!(m.len(), 1, "only `name:` should survive the initializer: {m:#?}");
    }
    let names: Vec<&str> = values.iter().filter_map(|v| v.get("name").and_then(|n| n.as_str())).collect();
    assert_eq!(names, vec!["low", "medium", "high"]);
}

#[test]
fn a_named_enum_usage_lifts_typed_by_and_doc() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Enums.sysml",
        "package Enums {\n\
         enum def FillLevel { low; high; }\n\
         part def FilterSystem {\n\
         enum fillLevel : FillLevel {\n\
         doc /* Current filter fill level. */\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Enums::FilterSystem::fillLevel")
        .expect("fillLevel should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::Enumeration));
    assert_eq!(el.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("FillLevel"));
    assert!(el.doc.contains("Current filter fill level."), "{:?}", el.doc);
}

#[test]
fn an_enum_def_and_usage_nested_in_a_part_usage_body_are_also_reachable() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Enums.sysml",
        "package Enums {\n\
         part def Housing;\n\
         part housing : Housing {\n\
         enum def LocalKind { a; b; }\n\
         enum k : LocalKind;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let def = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Enums::housing::LocalKind")
        .expect("LocalKind should be a real element");
    assert_eq!(def.frontmatter.element_type, Some(ElementType::EnumerationDef));

    let usage = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Enums::housing::k")
        .expect("k should be a real element");
    assert_eq!(usage.frontmatter.element_type, Some(ElementType::Enumeration));
    assert_eq!(usage.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("LocalKind"));
}

//! Integration tests for lifting SysML v2 `doc /* ... */` comments into the
//! synthesized element's `doc` body (`REQ-TRS-SYSMLV2-009`).
//!
//! Mirrors `sysmlv2_feature.rs`/`sysmlv2_metadata.rs`'s structure. Unlike
//! those two, the lifted value lands on `RawElement.doc` directly, not on
//! `RawFrontmatter` — there's no YAML field involved, so assertions read
//! `element.doc` and drive `validate` to confirm the existing `W600` check
//! reacts to it exactly like a hand-authored element's body text would.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-doc-test-{}-{}",
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
fn a_single_doc_block_lifts_onto_a_part_def_and_clears_w600() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def DocumentedPart {\n\
         doc /* Explanation. */\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::DocumentedPart")
        .unwrap();
    assert_eq!(part.doc, "Explanation.");

    let result = validate(&elements);
    assert!(
        !result.findings.iter().any(|f| f.code == "W600"),
        "expected no W600 for a documented PartDef: {:#?}",
        result.findings
    );
}

#[test]
fn two_doc_blocks_concatenate_in_source_order() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def TwoDocPart {\n\
         doc /* First. */\n\
         doc /* Second. */\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::TwoDocPart")
        .unwrap();
    assert_eq!(part.doc, "First.\n\nSecond.");
}

#[test]
fn no_doc_block_means_empty_doc_and_w600_still_fires_no_regression() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def PlainPart;\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::PlainPart")
        .unwrap();
    assert_eq!(part.doc, "");

    let result = validate(&elements);
    assert!(
        result.findings.iter().any(|f| f.code == "W600"),
        "expected W600 for an undocumented PartDef (no regression): {:#?}",
        result.findings
    );
}

#[test]
fn a_part_usage_also_lifts_its_own_doc_block() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def Ecu;\n\
         part safetyEcu : Ecu {\n\
         doc /* Usage-level explanation. */\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::safetyEcu")
        .unwrap();
    assert_eq!(part.doc, "Usage-level explanation.");
}

#[test]
fn port_def_port_usage_connection_def_interface_def_attribute_def_and_item_def_all_lift_their_own_doc() {
    // One test covering every remaining element kind REQ-TRS-SYSMLV2-009
    // scopes in, rather than six near-duplicate tests.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         port def DocPortDef {\n\
         doc /* port def doc. */\n\
         }\n\
         connection def DocConnDef {\n\
         doc /* connection def doc. */\n\
         }\n\
         interface def DocIfaceDef {\n\
         doc /* interface def doc. */\n\
         }\n\
         attribute def DocAttrDef {\n\
         doc /* attribute def doc. */\n\
         }\n\
         item def DocItemDef {\n\
         doc /* item def doc. */\n\
         }\n\
         part def Holder {\n\
         port docPort : DocPortDef {\n\
         doc /* port usage doc. */\n\
         }\n\
         attribute docAttr : DocAttrDef {\n\
         doc /* attribute usage doc. */\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let doc_of = |qname: &str| -> String {
        elements
            .iter()
            .find(|e| e.qualified_name == qname)
            .unwrap_or_else(|| panic!("missing element {qname}"))
            .doc
            .clone()
    };

    assert_eq!(doc_of("SysML2Legacy::CarOS::DocPortDef"), "port def doc.");
    assert_eq!(doc_of("SysML2Legacy::CarOS::DocConnDef"), "connection def doc.");
    assert_eq!(doc_of("SysML2Legacy::CarOS::DocIfaceDef"), "interface def doc.");
    assert_eq!(doc_of("SysML2Legacy::CarOS::DocAttrDef"), "attribute def doc.");
    assert_eq!(doc_of("SysML2Legacy::CarOS::DocItemDef"), "item def doc.");
    assert_eq!(doc_of("SysML2Legacy::CarOS::Holder::docPort"), "port usage doc.");
    assert_eq!(doc_of("SysML2Legacy::CarOS::Holder::docAttr"), "attribute usage doc.");
}

#[test]
fn a_variant_part_attribute_and_port_usage_each_lift_their_own_doc() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Config.sysml",
        "package Config {\n\
         variation part def RotorConfig {\n\
         variant part quadConfig : QuadRotor {\n\
         doc /* quad variant doc. */\n\
         }\n\
         variant attribute quadAttr : ThrustRating {\n\
         doc /* attribute variant doc. */\n\
         }\n\
         variant port quadPort : FuelPort {\n\
         doc /* port variant doc. */\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let doc_of = |qname: &str| -> String {
        elements
            .iter()
            .find(|e| e.qualified_name == qname)
            .unwrap_or_else(|| panic!("missing element {qname}"))
            .doc
            .clone()
    };

    assert_eq!(doc_of("SysML2Legacy::Config::RotorConfig::quadConfig"), "quad variant doc.");
    assert_eq!(doc_of("SysML2Legacy::Config::RotorConfig::quadAttr"), "attribute variant doc.");
    assert_eq!(doc_of("SysML2Legacy::Config::RotorConfig::quadPort"), "port variant doc.");
}

#[test]
fn an_item_usage_has_nowhere_for_doc_to_attach_and_stays_empty() {
    // ItemUsage carries no `body` field in this grammar (unlike ItemDef) —
    // there's nowhere for a `doc` member to attach at all. Pinning that this
    // is a real grammar limitation, not a mapper oversight: an ItemUsage
    // simply has no doc, always, regardless of this feature.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         item def Fuel;\n\
         part def Tank {\n\
         item fuelItem : Fuel;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let item = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::Tank::fuelItem")
        .unwrap();
    assert_eq!(item.doc, "");
}

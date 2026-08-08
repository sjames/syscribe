//! Integration tests for the fixed set of `@Syscribe*` metadata annotations
//! that lift `domain:`/`asilLevel:`/`silLevel:`/`plLevel:`/`shortName:`/
//! `implementedBy:` onto a SysMLv2-authored `part def`/`part`
//! (`REQ-TRS-SYSMLV2-008`).
//!
//! Mirrors `sysmlv2_feature.rs`'s structure and its core finding: every field
//! lifted here already exists on `RawFrontmatter` and is already validated
//! for a hand-authored element, so this is purely a mapping concern — no
//! validator changes are exercised here beyond confirming the existing rules
//! fire identically for a SysMLv2-originated element.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-metadata-test-{}-{}",
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
fn all_four_annotations_lift_onto_a_part_def() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def CarSafetyServices {\n\
         @SyscribeDomain {\n\
         value = 'software';\n\
         }\n\
         @SyscribeIntegrity {\n\
         asil = 'B';\n\
         }\n\
         @SyscribeShortName {\n\
         value = 'car-safety-services';\n\
         }\n\
         @SyscribeImplementedBy {\n\
         path = 'services/car-safety-services/';\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::CarSafetyServices")
        .unwrap();

    assert_eq!(part.frontmatter.domain.as_deref(), Some("software"));
    assert_eq!(part.frontmatter.asil_level.as_deref(), Some("B"));
    assert_eq!(part.frontmatter.sil_level, None);
    assert_eq!(part.frontmatter.pl_level, None);
    assert_eq!(part.frontmatter.short_name.as_deref(), Some("car-safety-services"));
    assert_eq!(
        part.frontmatter.implemented_by,
        Some(vec!["services/car-safety-services/".to_string()])
    );

    // W023 is expected (the implementedBy path doesn't exist on disk in this
    // fixture) — that's the existing check firing, not a defect. No error.
    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
    assert!(
        result.findings.iter().any(|f| f.code == "W023"),
        "expected the existing W023 disk-check to fire on the lifted implementedBy path: {:#?}",
        result.findings
    );
}

#[test]
fn all_four_annotations_lift_onto_a_part_usage() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def Ecu;\n\
         part safetyEcu : Ecu {\n\
         @SyscribeDomain {\n\
         value = 'hardware';\n\
         }\n\
         @SyscribeIntegrity {\n\
         asil = 'D';\n\
         }\n\
         @SyscribeShortName {\n\
         value = 'safety-ecu';\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::safetyEcu")
        .unwrap();

    assert_eq!(part.frontmatter.domain.as_deref(), Some("hardware"));
    assert_eq!(part.frontmatter.asil_level.as_deref(), Some("D"));
    assert_eq!(part.frontmatter.short_name.as_deref(), Some("safety-ecu"));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn syscribe_integrity_sil_and_pl_forms_parse_independently() {
    // sil is an unquoted integer literal (Expression::LiteralInteger), unlike
    // asil/pl/domain/shortName/implementedBy's quoted "restricted name"
    // (Expression::FeatureRef) values — this is the one lift in the fixed set
    // that reads a different AST expression variant.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def SilPart {\n\
         @SyscribeIntegrity {\n\
         sil = 3;\n\
         }\n\
         }\n\
         part def PlPart {\n\
         @SyscribeIntegrity {\n\
         pl = 'd';\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let sil_part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::SilPart")
        .unwrap();
    let pl_part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::PlPart")
        .unwrap();

    assert_eq!(sil_part.frontmatter.sil_level, Some(3));
    assert_eq!(sil_part.frontmatter.asil_level, None);
    assert_eq!(pl_part.frontmatter.pl_level.as_deref(), Some("d"));
}

#[test]
fn syscribe_integrity_with_both_asil_and_sil_raises_the_existing_w006() {
    // REQ-TRS-SYSMLV2-008's acceptance criterion: more than one of
    // asil/sil/pl on the same @SyscribeIntegrity annotation is not specially
    // rejected by the mapper — both fields are simply written, and the
    // pre-existing asilLevel/silLevel mutual-exclusion warning fires exactly
    // as it would for a hand-authored element carrying both. No new
    // validation code exists for this — this test proves that by exercising
    // only the ingestion + the *unmodified* validator.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def BothPart {\n\
         @SyscribeIntegrity {\n\
         asil = 'D';\n\
         sil = 2;\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::BothPart")
        .unwrap();
    assert_eq!(part.frontmatter.asil_level.as_deref(), Some("D"));
    assert_eq!(part.frontmatter.sil_level, Some(2));

    let result = validate(&elements);
    let w006: Vec<_> = result.findings.iter().filter(|f| f.code == "W006").collect();
    assert_eq!(w006.len(), 1, "expected exactly one W006, got: {:#?}", result.findings);
}

#[test]
fn a_variant_part_usage_also_lifts_the_fixed_field_set() {
    // The same PartUsageBody shape @SyscribeFeature already reaches inside a
    // `variant part name : Type { ... }` usage — REQ-TRS-SYSMLV2-008's lift
    // reuses that exact reach.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Config.sysml",
        "package Config {\n\
         variation part def RotorConfig {\n\
         variant part quad : QuadRotor {\n\
         @SyscribeDomain {\n\
         value = 'hardware';\n\
         }\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let quad = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Config::RotorConfig::quad")
        .unwrap();
    assert_eq!(quad.frontmatter.domain.as_deref(), Some("hardware"));
    assert_eq!(quad.frontmatter.is_variant, Some(true));
}

#[test]
fn no_annotation_means_no_lifted_fields_no_regression() {
    // REQ-TRS-SYSMLV2-008's stated scope: a part def/part with none of these
    // annotations behaves exactly as it does today.
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
    assert_eq!(part.frontmatter.domain, None);
    assert_eq!(part.frontmatter.asil_level, None);
    assert_eq!(part.frontmatter.sil_level, None);
    assert_eq!(part.frontmatter.pl_level, None);
    assert_eq!(part.frontmatter.short_name, None);
    assert_eq!(part.frontmatter.implemented_by, None);
}

#[test]
fn syscribe_feature_and_the_fixed_field_set_coexist_on_the_same_part() {
    // The two annotation families (REQ-TRS-SYSMLV2-005's @SyscribeFeature and
    // REQ-TRS-SYSMLV2-008's fixed four) are independent scans over the same
    // body — both must be readable side by side on one element.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "Features/Rotor.md",
        "---\ntype: FeatureDef\nid: FEAT-ROTOR\nname: Rotor\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Config.sysml",
        "package Config {\n\
         variation part def RotorConfig {\n\
         @SyscribeFeature {\n\
         featureId = 'FEAT-ROTOR';\n\
         }\n\
         @SyscribeDomain {\n\
         value = 'software';\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let rotor = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Config::RotorConfig")
        .unwrap();
    assert_eq!(
        rotor.frontmatter.applies_when,
        Some(serde_yaml::Value::String("FEAT-ROTOR".to_string()))
    );
    assert_eq!(rotor.frontmatter.domain.as_deref(), Some("software"));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

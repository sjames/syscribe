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
fn double_quoted_string_values_lift_the_same_as_single_quoted() {
    // Regression: an earlier version of attribute_body_string only handled
    // Expression::FeatureRef (single-quoted "restricted name" tokens), so a
    // syntactically valid double-quoted SysML v2 string literal
    // (Expression::LiteralString) silently produced no lifted field and no
    // diagnostic at all. Found by manual smoke-testing, fixed before this
    // test was written.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def DoubleQuotedPart {\n\
         @SyscribeDomain {\n\
         value = \"software\";\n\
         }\n\
         @SyscribeShortName {\n\
         value = \"double-quoted-part\";\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::DoubleQuotedPart")
        .unwrap();
    assert_eq!(part.frontmatter.domain.as_deref(), Some("software"));
    assert_eq!(part.frontmatter.short_name.as_deref(), Some("double-quoted-part"));
}

#[test]
fn an_out_of_range_sil_saturates_and_still_raises_e009() {
    // Regression: an earlier version wrote `u8::try_from(v).ok()`, which
    // silently dropped the entire field (no silLevel:, no diagnostic at all)
    // for any sil value outside 0..=255 — worse than a hand-authored
    // `silLevel: 999`, which at least reaches the existing E009 "out of
    // range 1-4" check. Saturating instead means the value still lands on
    // frontmatter and the existing check still catches it.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def OutOfRangeSilPart {\n\
         @SyscribeIntegrity {\n\
         sil = 999;\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::OutOfRangeSilPart")
        .unwrap();
    assert_eq!(part.frontmatter.sil_level, Some(255));

    let result = validate(&elements);
    let e009: Vec<_> = result.findings.iter().filter(|f| f.code == "E009").collect();
    assert_eq!(e009.len(), 1, "expected exactly one E009, got: {:#?}", result.findings);
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

#[test]
fn syscribe_domain_alone_lifts_with_no_other_annotation_present() {
    // Every other test in this file bundles two or more annotations on the
    // same part def — this one isolates @SyscribeDomain to confirm it lifts
    // correctly with nothing else present (not just "domain ends up right
    // when three other annotations are also being folded in the same pass").
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def DomainOnlyPart {\n\
         @SyscribeDomain {\n\
         value = 'hardware';\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::DomainOnlyPart")
        .unwrap();
    assert_eq!(part.frontmatter.domain.as_deref(), Some("hardware"));
    assert_eq!(part.frontmatter.asil_level, None);
    assert_eq!(part.frontmatter.sil_level, None);
    assert_eq!(part.frontmatter.pl_level, None);
    assert_eq!(part.frontmatter.short_name, None);
    assert_eq!(part.frontmatter.implemented_by, None);
}

#[test]
fn a_negative_sil_saturates_via_unary_minus_and_still_raises_e009() {
    // Regression: sysml-v2-parser has no negative-integer-literal token of
    // its own — `sil = -1;` parses one level deeper, as
    // Expression::UnaryOp { op: Minus, operand: LiteralInteger(1) }, not a
    // single LiteralInteger(-1). An earlier version of attribute_body_i64
    // only matched LiteralInteger directly, so this silently produced no
    // silLevel: and no diagnostic at all. Fixed by also matching the
    // UnaryOp/Minus/LiteralInteger shape and clamping into u8 range (same
    // saturate-so-E009-still-catches-it strategy as the too-large case).
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def NegSilPart {\n\
         @SyscribeIntegrity {\n\
         sil = -1;\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::NegSilPart")
        .unwrap();
    assert_eq!(part.frontmatter.sil_level, Some(0));

    let result = validate(&elements);
    let e009: Vec<_> = result.findings.iter().filter(|f| f.code == "E009").collect();
    assert_eq!(e009.len(), 1, "expected exactly one E009, got: {:#?}", result.findings);
}

#[test]
fn a_non_integer_sil_is_a_documented_silent_gap_not_a_crash() {
    // A float value (Expression::LiteralReal) isn't a shape
    // attribute_body_i64 recognizes at all — silLevel is inherently an
    // integer scale (1-4), so there's no sensible truncate-or-round
    // recovery. Pinning current (documented, module-level-noted) behavior:
    // no silLevel: is lifted, no diagnostic is raised, and — most
    // importantly — ingestion doesn't panic or otherwise misbehave on a
    // legally-parsed-but-unrecognized value shape.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def FloatSilPart {\n\
         @SyscribeIntegrity {\n\
         sil = 2.5;\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::FloatSilPart")
        .unwrap();
    assert_eq!(part.frontmatter.sil_level, None);

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn a_requirement_usage_does_not_pick_up_the_fixed_field_set() {
    // Scope check: unlike @SyscribeFeature (which reaches into
    // RequirementDef/RequirementUsage bodies too, since variation isn't
    // Part-exclusive), REQ-TRS-SYSMLV2-008's fixed four are dispatched only
    // from convert_part_def/convert_part_usage/the variant-part branch —
    // convert_requirement_def/convert_requirement_usage never call
    // fold_syscribe_meta_annotation. A RequirementUsage carrying
    // @SyscribeDomain should be ingested as an ordinary Requirement with no
    // domain: field at all, not silently coerced into one.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Config.sysml",
        "package Config {\n\
         requirement reqFoo : ReqChoice {\n\
         @SyscribeDomain {\n\
         value = 'software';\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let req = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Config::reqFoo")
        .unwrap();
    assert_eq!(req.frontmatter.element_type, Some(syscribe_model::element::ElementType::Requirement));
    assert_eq!(
        req.frontmatter.domain, None,
        "@SyscribeDomain on a RequirementUsage should not lift a domain: field"
    );
}

// ── REQ-TRS-SYSMLV2-014: doc-comment directives for interface def/port def/ ──
// ── connection def, since their body grammars carry no MetadataAnnotation ───
// ── slot for the real @Name{...} form (#100).                              ──

#[test]
fn a_doc_directive_lifts_implemented_by_onto_an_interface_def() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         interface def IPowerInterface {\n\
         doc /*\n\
         Real prose stays.\n\
         @SyscribeShortName: power-if\n\
         @SyscribeImplementedBy: aidl/interfaces/car/power/IPowerInterface.aidl\n\
         More prose after.\n\
         */\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let iface = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::IPowerInterface")
        .unwrap();

    assert_eq!(iface.frontmatter.short_name.as_deref(), Some("power-if"));
    assert_eq!(
        iface.frontmatter.implemented_by,
        Some(vec!["aidl/interfaces/car/power/IPowerInterface.aidl".to_string()])
    );
    // Directive lines are stripped; surrounding prose survives, with no
    // stray double-blank-line left where they were removed.
    assert!(iface.doc.contains("Real prose stays."));
    assert!(iface.doc.contains("More prose after."));
    assert!(!iface.doc.contains("@SyscribeShortName"));
    assert!(!iface.doc.contains("@SyscribeImplementedBy"));
    assert!(!iface.doc.contains("\n\n\n"), "no stray triple-newline: {:?}", iface.doc);

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
    assert!(
        result.findings.iter().any(|f| f.code == "W023"),
        "expected W023 to fire on the lifted implementedBy path exactly like the real-annotation form: {:#?}",
        result.findings
    );
}

#[test]
fn a_doc_directive_lifts_domain_and_integrity_onto_a_port_def() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         port def PowerPort {\n\
         doc /* @SyscribeDomain: hardware\n\
         @SyscribeIntegrity: asil=D, sil=3 */\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let port = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::PowerPort")
        .unwrap();

    assert_eq!(port.frontmatter.domain.as_deref(), Some("hardware"));
    assert_eq!(port.frontmatter.asil_level.as_deref(), Some("D"));
    assert_eq!(port.frontmatter.sil_level, Some(3));

    // W006: asilLevel and silLevel together fire the same mutual-exclusion
    // warning a hand-authored element carrying both would — no new
    // validation code, exactly as REQ-TRS-SYSMLV2-008's addendum establishes
    // for the real-annotation form.
    let result = validate(&elements);
    assert!(
        result.findings.iter().any(|f| f.code == "W006"),
        "expected W006 for asilLevel+silLevel both set: {:#?}",
        result.findings
    );
}

#[test]
fn a_doc_directive_lifts_short_name_onto_a_connection_def() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         connection def PowerLink {\n\
         doc /* @SyscribeShortName: power-link */\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let conn = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::PowerLink")
        .unwrap();
    assert_eq!(conn.frontmatter.short_name.as_deref(), Some("power-link"));
    assert_eq!(conn.doc, "", "a doc block that is only a directive line leaves no doc text");
}

#[test]
fn an_unrecognized_at_line_is_left_in_the_doc_text_untouched() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         interface def IFoo {\n\
         doc /* @SomethingElse: not a directive */\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let iface = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::IFoo")
        .unwrap();
    assert_eq!(iface.doc, "@SomethingElse: not a directive");
    assert_eq!(iface.frontmatter.short_name, None);
    assert_eq!(iface.frontmatter.implemented_by, None);
}

#[test]
fn a_later_directive_for_the_same_field_wins_over_an_earlier_one() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         interface def IFoo {\n\
         doc /* @SyscribeShortName: first\n\
         @SyscribeShortName: second */\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let iface = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::IFoo")
        .unwrap();
    assert_eq!(iface.frontmatter.short_name.as_deref(), Some("second"));
}

#[test]
fn an_interface_def_with_no_doc_comment_at_all_is_unaffected_no_regression() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         interface def IPlain;\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let iface = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::IPlain")
        .unwrap();
    assert_eq!(iface.doc, "");
    assert_eq!(iface.frontmatter.short_name, None);
    assert_eq!(iface.frontmatter.implemented_by, None);
    assert_eq!(iface.frontmatter.domain, None);
}

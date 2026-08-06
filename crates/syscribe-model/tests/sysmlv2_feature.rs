//! Integration tests for a SysMLv2 `variation`/`variant` element's
//! `@SyscribeFeature { featureId = '...'; }` metadata annotation
//! (`REQ-TRS-SYSMLV2-005`).
//!
//! Investigation finding (see this task's report): `appliesWhen:`/the
//! feature-model machinery (`variability.rs`/`feature_model.rs`/
//! `projection.rs`) is already fully origin-agnostic — it reads
//! `RawFrontmatter.applies_when` off any `RawElement` regardless of how it
//! entered `elements`, the same way `satisfies:`/`verifies:` already were.
//! So this task is purely about (a) finding `@SyscribeFeature` in the AST and
//! (b) writing the *existing* `applies_when` field — no new gating logic, no
//! solver changes.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::feature_model;
use syscribe_model::projection;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-feature-test-{}-{}",
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
fn syscribe_feature_annotation_lifts_into_applies_when() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/Rotor.md",
        "---\ntype: FeatureDef\nid: FEAT-ROTOR\nname: Rotor\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Config.sysml",
        "package Config {\n\
         variation part def RotorConfig {\n\
         @SyscribeFeature {\n\
         featureId = 'FEAT-ROTOR';\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let rotor = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Config::RotorConfig")
        .unwrap();

    // The mapper writes the exact same field a native element's appliesWhen:
    // uses — verbatim, as a bare string (the FEAT-* id, quote-stripped).
    assert_eq!(
        rotor.frontmatter.applies_when,
        Some(serde_yaml::Value::String("FEAT-ROTOR".to_string()))
    );

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn syscribe_feature_on_a_typed_variant_lifts_into_applies_when() {
    // The annotation also attaches to a `variant part name : Type { ... }`'s
    // own inner usage body, not just the enclosing `variation part def`.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/Quad.md",
        "---\ntype: FeatureDef\nid: FEAT-QUAD\nname: Quad\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Config.sysml",
        "package Config {\n\
         variation part def RotorConfig {\n\
         variant part quad : QuadRotor {\n\
         @SyscribeFeature {\n\
         featureId = 'FEAT-QUAD';\n\
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
    assert_eq!(
        quad.frontmatter.applies_when,
        Some(serde_yaml::Value::String("FEAT-QUAD".to_string()))
    );
    assert_eq!(quad.frontmatter.is_variant, Some(true));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn no_annotation_means_purely_structural_no_feature_participation() {
    // REQ-TRS-SYSMLV2-005's stated scope: a variation/variant with no
    // @SyscribeFeature is ingested normally and simply doesn't participate in
    // the feature-model graph, same as a native element with no appliesWhen:.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Config.sysml",
        "package Config {\n\
         variation part def RotorConfig {\n\
         variant part quad : QuadRotor;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let rotor = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Config::RotorConfig")
        .unwrap();
    let quad = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Config::RotorConfig::quad")
        .unwrap();

    assert_eq!(rotor.frontmatter.applies_when, None);
    assert_eq!(quad.frontmatter.applies_when, None);
    // Still fully structural: is_variation/is_variant are unaffected.
    assert_eq!(rotor.frontmatter.is_variation, Some(true));
    assert_eq!(quad.frontmatter.is_variant, Some(true));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn unresolvable_feature_id_raises_the_normal_e209_finding() {
    // Verified empirically (not assumed): E209 is a fully generic check —
    // "for any element with appliesWhen: set, every operand must resolve to
    // a FeatureDef" — with no special-casing by element type or origin. A
    // dangling featureId hits the exact same code path as a hand-authored
    // element's dangling appliesWhen:, confirmed by reading
    // `validator.rs`'s E209 block (unconditional over `elements`).
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Config.sysml",
        "package Config {\n\
         variation part def RotorConfig {\n\
         @SyscribeFeature {\n\
         featureId = 'FEAT-DOES-NOT-EXIST';\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    let e209: Vec<_> = result.findings.iter().filter(|f| f.code == "E209").collect();
    assert_eq!(e209.len(), 1, "expected exactly one E209, got: {:#?}", result.findings);
    assert!(e209[0].message.contains("FEAT-DOES-NOT-EXIST"));
}

#[test]
fn feature_gated_sysmlv2_element_projects_in_and_out_like_a_native_one() {
    // "Same clause behavior, not just doesn't crash": the existing
    // `--config` projection engine (`projection::project`, the same code
    // `configure`/`validate --config` use) includes or excludes the
    // SysMLv2-authored element based purely on its lifted `applies_when`,
    // exactly as it would for a native element with the same appliesWhen:.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/Rotor.md",
        "---\ntype: FeatureDef\nid: FEAT-ROTOR\nname: Rotor\n---\n",
    );
    write(
        &root,
        "Configs/CONF-ON.md",
        "---\ntype: Configuration\nid: CONF-ON\nname: On\nfeatures:\n  FEAT-ROTOR: true\n---\n",
    );
    write(
        &root,
        "Configs/CONF-OFF.md",
        "---\ntype: Configuration\nid: CONF-OFF\nname: Off\nfeatures:\n  FEAT-ROTOR: false\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Config.sysml",
        "package Config {\n\
         variation part def RotorConfig {\n\
         @SyscribeFeature {\n\
         featureId = 'FEAT-ROTOR';\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let conf_on = elements
        .iter()
        .find(|e| e.frontmatter.id.as_deref() == Some("CONF-ON"))
        .unwrap();
    let conf_off = elements
        .iter()
        .find(|e| e.frontmatter.id.as_deref() == Some("CONF-OFF"))
        .unwrap();

    let sel_on = projection::canonical_selection(&elements, conf_on);
    let sel_off = projection::canonical_selection(&elements, conf_off);
    let proj_on = projection::project(&elements, &sel_on);
    let proj_off = projection::project(&elements, &sel_off);

    assert!(
        proj_on
            .iter()
            .any(|e| e.qualified_name == "SysML2Legacy::Config::RotorConfig"),
        "the SysMLv2 element should be present when its gating feature is selected true"
    );
    assert!(
        !proj_off
            .iter()
            .any(|e| e.qualified_name == "SysML2Legacy::Config::RotorConfig"),
        "the SysMLv2 element should be excluded when its gating feature is selected false"
    );
}

#[test]
fn feature_referenced_only_via_a_sysmlv2_element_is_not_flagged_orphan() {
    // Same-clause-behavior proof via a second, independent consumer of
    // appliesWhen: (`feature_model::check_feature_model`'s W024 "orphan
    // FeatureDef" check) — a FeatureDef referenced by NO element's
    // appliesWhen: and selected by no Configuration is flagged. When the
    // *only* reference is our SysMLv2 element's lifted applies_when, W024
    // must NOT fire, exactly as it wouldn't for a native referencing element.
    //
    // Uses the qualified-name form of featureId (`Config::Rotor`) rather than
    // the bare FEAT-* id: a real, pre-existing, origin-independent quirk was
    // found in W024's own check (it doesn't canonicalize an id-form
    // appliesWhen: operand to the FeatureDef's qualified name before
    // comparing — confirmed to reproduce identically for a hand-authored
    // native element using `appliesWhen: FEAT-ROTOR`, so it is not a SysMLv2
    // defect and out of this task's scope to fix). The qname form sidesteps
    // that pre-existing gap and still fully proves this task's actual claim:
    // the SysMLv2-lifted `applies_when` is read by W024's check exactly like
    // any other element's.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/Rotor.md",
        "---\ntype: FeatureDef\nid: FEAT-ROTOR\nname: Rotor\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Config.sysml",
        "package Config {\n\
         variation part def RotorConfig {\n\
         @SyscribeFeature {\n\
         featureId = Features::Rotor;\n\
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
        Some(serde_yaml::Value::String("Features::Rotor".to_string()))
    );

    let findings = feature_model::check_feature_model(&elements);
    assert!(
        !findings.iter().any(|f| f.code == "W024"),
        "FEAT-ROTOR should not be flagged orphan — it's referenced by the SysMLv2 element: {:#?}",
        findings
    );
}

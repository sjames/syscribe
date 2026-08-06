//! Integration tests for `PlanningItem` + the existing, universal `appliesWhen:`
//! product-line gating mechanism (`REQ-TRS-PLANITEM-004`).
//!
//! Investigation finding (confirmed here with tests, not just by inspection):
//! `appliesWhen:`/the feature-model machinery (`variability.rs`/
//! `feature_model.rs`/`projection.rs`) is already fully type-agnostic — it
//! reads `RawFrontmatter.applies_when` off any `RawElement` by field alone,
//! with zero `ElementType` filtering anywhere in `is_active_canon`/`project`/
//! `check_feature_model`'s W024 pass/`validator.rs`'s E209 block. Since
//! `PlanningItem` is a normal `ElementType` backed by the same
//! `RawFrontmatter`, no new mechanism was needed — this file exists purely to
//! prove that claim empirically, mirroring the pattern
//! `tests/sysmlv2_feature.rs` used for the same claim about SysMLv2-lifted
//! elements (`REQ-TRS-SYSMLV2-005`). Zero changes were required in
//! `feature_model.rs`, `projection.rs`, `variability.rs`, or `validator.rs`
//! to make these tests pass.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::feature_model;
use syscribe_model::projection;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-planning-item-applies-when-test-{}-{}",
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
fn planning_item_with_applies_when_projects_in_and_out_across_configurations() {
    // Same claim as sysmlv2_feature.rs's
    // feature_gated_sysmlv2_element_projects_in_and_out_like_a_native_one:
    // the *same* projection::project engine `configure`/`validate --config`
    // use includes/excludes the element based purely on its `appliesWhen:`,
    // with no special-casing for PlanningItem.
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
        "Requirements/RotorReq.md",
        "---\ntype: Requirement\nid: REQ-ROTOR-001\nname: Rotor requirement\nstatus: draft\n---\nThe system shall have a rotor.\n",
    );
    write(
        &root,
        "Planning/BuildRotor.md",
        "---\ntype: PlanningItem\nid: PI-ROTOR-001\nname: Build the rotor\nstatus: todo\nachieves: REQ-ROTOR-001\nappliesWhen: FEAT-ROTOR\n---\n",
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
            .any(|e| e.qualified_name == "Planning::BuildRotor"),
        "the PlanningItem should be present when its gating feature is selected true: {:#?}",
        proj_on.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    assert!(
        !proj_off
            .iter()
            .any(|e| e.qualified_name == "Planning::BuildRotor"),
        "the PlanningItem should be excluded when its gating feature is selected false: {:#?}",
        proj_off.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
}

#[test]
fn planning_item_referenced_only_via_applies_when_is_not_flagged_orphan() {
    // Same-clause-behavior proof via check_feature_model's W024 "orphan
    // FeatureDef" check: a FeatureDef referenced by NO element's appliesWhen:
    // and selected by no Configuration is flagged. When the *only* reference
    // is a PlanningItem's appliesWhen:, W024 must NOT fire.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/Rotor.md",
        "---\ntype: FeatureDef\nid: FEAT-ROTOR\nname: Rotor\n---\n",
    );
    write(
        &root,
        "Requirements/RotorReq.md",
        "---\ntype: Requirement\nid: REQ-ROTOR-002\nname: Rotor requirement\nstatus: draft\n---\nThe system shall have a rotor.\n",
    );
    write(
        &root,
        "Planning/BuildRotor.md",
        "---\ntype: PlanningItem\nid: PI-ROTOR-002\nname: Build the rotor\nstatus: todo\nachieves: REQ-ROTOR-002\nappliesWhen: FEAT-ROTOR\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let findings = feature_model::check_feature_model(&elements);
    assert!(
        !findings.iter().any(|f| f.code == "W024"),
        "FEAT-ROTOR should not be flagged orphan — it's referenced by the PlanningItem's appliesWhen: {:#?}",
        findings
    );
}

#[test]
fn dangling_applies_when_on_planning_item_raises_e209() {
    // The normal E209 dangling/wrong-type appliesWhen check, exercised on a
    // PlanningItem exactly as it would be on any other element type.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Requirements/SomeReq.md",
        "---\ntype: Requirement\nid: REQ-SOME-001\nname: Some requirement\nstatus: draft\n---\nThe system shall do the thing.\n",
    );
    write(
        &root,
        "Planning/Dangling.md",
        "---\ntype: PlanningItem\nid: PI-DANGLE-001\nname: Dangling\nstatus: todo\nachieves: REQ-SOME-001\nappliesWhen: FEAT-NOPE-DOES-NOT-EXIST\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    assert!(
        result.findings.iter().any(|f| f.code == "E209"),
        "expected E209 for a dangling appliesWhen target on a PlanningItem: {:#?}",
        result.findings
    );
}

//! Regression coverage for three defects the REQ-TRS-FM-005 (single-file
//! feature model) review surfaced — all pre-existing, not specific to the new
//! `featureTree:`/`crossTreeConstraints:` mechanism, but exposed by exercising
//! a full-fledged feature model end to end:
//!
//! 1. `parameterConstraints:` was never a typed `RawFrontmatter` field, so
//!    declaring it (on a `Package` *or* the new `FeatureModel` sheet) always
//!    misfired `W047` ("unrecognized field") despite being read and enforced.
//! 2. `requires:`/`excludes:` resolution (`E212`, and the `feature-check
//!    --deep` SAT encoding) never canonicalized a bare `FEAT-*` stable id
//!    through the id→qname alias `appliesWhen:`/`Configuration.features:`
//!    already use — so an id-form reference (the natural shorthand in
//!    `crossTreeConstraints:`) always raised a bogus `E212` and was silently
//!    dropped from deep analysis.
//! 3. `syscribe features`' tree indented purely by raw qname-segment count,
//!    not real FeatureDef ancestry — misrendering a feature under a
//!    non-FeatureDef namespace directory as nested under an unrelated
//!    sibling, and a `parentFeature:`-relocated feature as top-level instead
//!    of under its real parent.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-fm-review-fixes-{}-{}",
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

fn run(model: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(model)
        .args(args)
        .output()
        .expect("run syscribe");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// `parameterConstraints:` on a plain `Package` must not raise `W047` — it is
/// a recognized, enforced field (§9.7), not author-defined data.
#[test]
fn parameter_constraints_on_package_does_not_raise_w047() {
    let root = tempdir();
    write(
        &root,
        "_index.md",
        r#"---
type: Package
name: Root
parameterConstraints:
  - id: PC-1
    expression: "Features::X.p >= 1"
    severity: error
---
"#,
    );
    write(
        &root,
        "Features/X.md",
        r#"---
type: FeatureDef
id: FEAT-X-001
name: X
groupKind: optional
parameters:
  - { name: p, type: ScalarValues::Integer, range: "0..10", default: 1 }
---
"#,
    );

    let out = run(&root, &["validate"]);
    assert!(!out.contains("W047"), "unexpected W047 on a recognized field: {out}");
}

/// A `requires:`/`excludes:` entry may be a bare `FEAT-*` stable id — resolved
/// the same way `appliesWhen:`/`Configuration.features:` already accept ids —
/// with no `E212`, and it is genuinely enforced (`E219` when violated).
#[test]
fn requires_by_stable_id_resolves_and_is_enforced() {
    let root = tempdir();
    write(
        &root,
        "Features/A.md",
        r#"---
type: FeatureDef
id: FEAT-AA-001
name: A
groupKind: optional
requires: [FEAT-BB-001]
---
"#,
    );
    write(
        &root,
        "Features/B.md",
        r#"---
type: FeatureDef
id: FEAT-BB-001
name: B
groupKind: optional
---
"#,
    );
    write(
        &root,
        "Configurations/Bad.md",
        r#"---
type: Configuration
id: CONF-BAD-001
name: "A without B"
status: draft
featureModel: Features
features:
  Features::A: true
  Features::B: false
---
"#,
    );

    let out = run(&root, &["feature-check"]);
    assert!(!out.contains("E212"), "id-form requires: must resolve, no E212: {out}");
    assert!(out.contains("E219"), "A selected without required B must still be caught: {out}");
}

/// `feature-check --deep`'s SAT encoding must also honor an id-form
/// `requires:` — not just the parse-time `E212` check — else the constraint
/// is silently absent from void/dead/core analysis.
#[test]
fn requires_by_stable_id_is_enforced_in_deep_analysis() {
    let root = tempdir();
    write(
        &root,
        "Features/A.md",
        r#"---
type: FeatureDef
id: FEAT-AA-001
name: A
mandatory: true
groupKind: optional
requires: [FEAT-BB-001]
---
"#,
    );
    write(
        &root,
        "Features/B.md",
        r#"---
type: FeatureDef
id: FEAT-BB-001
name: B
excludes: [FEAT-AA-001]
groupKind: optional
---
"#,
    );

    // A is mandatory (always selected) and requires B; B excludes A — so no
    // valid configuration can exist. If the id-form requires:/excludes: were
    // silently dropped from the SAT encoding, this would wrongly report sound.
    let out = run(&root, &["feature-check", "--deep"]);
    assert!(out.contains("void model: true"), "contradictory id-form requires/excludes must be seen by deep analysis: {out}");
}

/// `syscribe features`: a feature under a plain namespace directory (no
/// FeatureDef of its own) must print as the top-level feature it is, not
/// nested under an unrelated sibling by raw qname-segment count.
#[test]
fn features_tree_does_not_misnest_under_a_non_feature_namespace() {
    let root = tempdir();
    write(
        &root,
        "Features/Alpha.md",
        r#"---
type: FeatureDef
id: FEAT-ALPHA-001
name: Alpha
groupKind: optional
---
"#,
    );
    // "Legacy" is a plain directory, not itself a FeatureDef.
    write(
        &root,
        "Features/Legacy/Orphan.md",
        r#"---
type: FeatureDef
id: FEAT-ORPHAN-001
name: Orphan
groupKind: optional
---
"#,
    );

    let out = run(&root, &["features"]);
    let orphan_line = out
        .lines()
        .find(|l| l.contains("Features::Legacy::Orphan"))
        .unwrap_or_else(|| panic!("Orphan feature not listed: {out}"));
    assert!(
        orphan_line.starts_with("- "),
        "a feature under a non-FeatureDef namespace must print top-level (no leading indent): {orphan_line:?}"
    );
}

/// `syscribe features`: an explicit `parentFeature:` override must nest the
/// feature under its real parent, not print it as top-level.
#[test]
fn features_tree_nests_under_explicit_parent_feature_override() {
    let root = tempdir();
    write(
        &root,
        "Features/Wdt.md",
        r#"---
type: FeatureDef
id: FEAT-WDT-001
name: Wdt
groupKind: optional
---
"#,
    );
    write(
        &root,
        "Features/Relocated.md",
        r#"---
type: FeatureDef
id: FEAT-RELOCATED-001
name: Relocated
groupKind: optional
parentFeature: Features::Wdt
---
"#,
    );

    let out = run(&root, &["features"]);
    let relocated_line = out
        .lines()
        .find(|l| l.contains("Features::Relocated"))
        .unwrap_or_else(|| panic!("Relocated feature not listed: {out}"));
    assert!(
        relocated_line.starts_with("  - "),
        "parentFeature: override must nest the feature one level under Wdt: {relocated_line:?}"
    );
}

//! REQ-TRS-FM-005: a `type: FeatureModel` sheet's flat, dot-named
//! `featureTree:` explodes into ordinary `FeatureDef` `RawElement`s with the
//! same qnames a directory-per-feature layout would produce, and its
//! `crossTreeConstraints:` merge into the matching synthesized `FeatureDef`'s
//! own `requires`/`excludes`. `walker::explode_feature_model_trees` is the
//! pass under test (invoked internally by `walk_model`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::ElementType;
use syscribe_model::feature_model;
use syscribe_model::validator::{self, Severity};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-fm-single-file-test-{}-{}",
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

fn has_code(findings: &[validator::Finding], code: &str) -> bool {
    findings.iter().any(|f| f.code == code)
}

fn errors_of(result: &validator::ValidationResult) -> Vec<&validator::Finding> {
    result.findings.iter().filter(|f| f.severity == Severity::Error).collect()
}

fn warnings_of(result: &validator::ValidationResult) -> Vec<&validator::Finding> {
    result.findings.iter().filter(|f| f.severity == Severity::Warning).collect()
}

/// A flat, dot-named `featureTree:` produces `FeatureDef` elements at exactly
/// the qnames a directory-per-feature layout would, with the leaf-only `name:`
/// rewritten, `mandatory`/`groupKind` carried through, and no findings.
#[test]
fn flat_dotted_tree_explodes_to_expected_qnames() {
    let root = tempdir();
    write(
        &root,
        "Features/_index.md",
        r#"---
type: FeatureModel
name: Features
featureTree:
  - name: Platform
    id: FEAT-PLATFORM-001
    mandatory: true
    groupKind: alternative
  - name: Platform.CortexM
    id: FEAT-CORTEXM-001
    groupKind: optional
  - name: Platform.RiscV
    id: FEAT-RISCV-001
    groupKind: optional
---
"#,
    );

    let elements = walk_model(&root).unwrap();
    let findings = validator::validate(&elements);
    assert!(errors_of(&findings).is_empty(), "unexpected errors: {:#?}", errors_of(&findings));

    let cortex_m = elements
        .iter()
        .find(|e| e.qualified_name == "Features::Platform::CortexM")
        .expect("Features::Platform::CortexM should be synthesized");
    assert_eq!(cortex_m.frontmatter.element_type, Some(ElementType::FeatureDef));
    assert_eq!(cortex_m.frontmatter.name.as_deref(), Some("CortexM"), "name: must be rewritten to just the leaf segment");
    assert_eq!(cortex_m.frontmatter.group_kind.as_deref(), Some("optional"));

    let platform = elements
        .iter()
        .find(|e| e.qualified_name == "Features::Platform")
        .expect("Features::Platform should be synthesized");
    assert_eq!(platform.frontmatter.mandatory, Some(true));
    assert_eq!(platform.frontmatter.group_kind.as_deref(), Some("alternative"));
}

/// `crossTreeConstraints:` merges into the matching synthesized `FeatureDef`'s
/// own `requires:`, equivalent to writing it inline.
#[test]
fn cross_tree_constraints_merge_into_synthesized_requires() {
    let root = tempdir();
    write(
        &root,
        "Features/_index.md",
        r#"---
type: FeatureModel
name: Features
featureTree:
  - name: Platform
    id: FEAT-PLATFORM-001
    mandatory: true
    groupKind: alternative
  - name: Platform.CortexM
    id: FEAT-CORTEXM-001
    groupKind: optional
  - name: Wdt
    id: FEAT-WDT-001
    groupKind: optional
crossTreeConstraints:
  - feature: Wdt
    requires: [Platform.CortexM]
---
"#,
    );

    let elements = walk_model(&root).unwrap();
    let findings = validator::validate(&elements);
    assert!(errors_of(&findings).is_empty(), "unexpected errors: {:#?}", errors_of(&findings));

    let wdt = elements.iter().find(|e| e.qualified_name == "Features::Wdt").unwrap();
    let requires: Vec<String> = wdt
        .frontmatter
        .requires
        .as_ref()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    assert_eq!(requires, vec!["Features::Platform::CortexM".to_string()]);
}

/// A `featureTree:` entry with no `name:` is dropped and flagged `E231`; it
/// contributes no `FeatureDef`.
#[test]
fn entry_with_no_name_is_e231_and_dropped() {
    let root = tempdir();
    write(
        &root,
        "Features/_index.md",
        r#"---
type: FeatureModel
name: Features
featureTree:
  - id: FEAT-NONAME-001
    groupKind: optional
---
"#,
    );

    let elements = walk_model(&root).unwrap();
    let findings = validator::validate(&elements);
    assert!(has_code(&findings.findings, "E231"));
    assert!(
        !elements.iter().any(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef)),
        "no FeatureDef should be synthesized from a nameless entry"
    );
}

/// A dotted path with an empty segment (leading/trailing/doubled `.`) is also `E231`.
#[test]
fn empty_path_segment_is_e231() {
    let root = tempdir();
    write(
        &root,
        "Features/_index.md",
        r#"---
type: FeatureModel
name: Features
featureTree:
  - name: "Platform..CortexM"
    id: FEAT-BAD-001
    groupKind: optional
---
"#,
    );

    let elements = walk_model(&root).unwrap();
    let findings = validator::validate(&elements);
    assert!(has_code(&findings.findings, "E231"));
}

/// Two `featureTree:` entries resolving to the same qname are `E232`; only the
/// first is kept.
#[test]
fn duplicate_qname_is_e232() {
    let root = tempdir();
    write(
        &root,
        "Features/_index.md",
        r#"---
type: FeatureModel
name: Features
featureTree:
  - name: Wdt
    id: FEAT-WDT-A
    groupKind: optional
  - name: Wdt
    id: FEAT-WDT-B
    groupKind: optional
---
"#,
    );

    let elements = walk_model(&root).unwrap();
    let findings = validator::validate(&elements);
    assert!(has_code(&findings.findings, "E232"));
    let wdts: Vec<_> = elements.iter().filter(|e| e.qualified_name == "Features::Wdt").collect();
    assert_eq!(wdts.len(), 1, "only the first colliding entry should be kept");
}

/// A `crossTreeConstraints:` entry whose `feature:` doesn't resolve to a
/// `FeatureDef` synthesized from this same sheet is `E233`.
#[test]
fn cross_tree_constraint_unresolved_feature_is_e233() {
    let root = tempdir();
    write(
        &root,
        "Features/_index.md",
        r#"---
type: FeatureModel
name: Features
featureTree:
  - name: Wdt
    id: FEAT-WDT-001
    groupKind: optional
crossTreeConstraints:
  - feature: DoesNotExist
    requires: [Wdt]
---
"#,
    );

    let elements = walk_model(&root).unwrap();
    let findings = validator::validate(&elements);
    assert!(has_code(&findings.findings, "E233"));
}

/// `featureTree:` on a non-`FeatureModel` element is inert and flagged `W048`;
/// it contributes no `FeatureDef`.
#[test]
fn feature_tree_on_wrong_type_is_w048_and_inert() {
    let root = tempdir();
    write(
        &root,
        "Features/_index.md",
        r#"---
type: Package
name: Features
featureTree:
  - name: Wdt
    id: FEAT-WDT-001
    groupKind: optional
---
"#,
    );

    let elements = walk_model(&root).unwrap();
    let findings = validator::validate(&elements);
    assert!(warnings_of(&findings).iter().any(|f| f.code == "W048"));
    assert!(
        !elements.iter().any(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef)),
        "featureTree: on a non-FeatureModel type must not synthesize any FeatureDef"
    );
}

/// `parameterConstraints:` declared directly on a `type: FeatureModel` sheet
/// is evaluated by `feature-check`, exactly as it already is on a `Package`
/// `_index.md` — no `E213` for a path that does resolve.
#[test]
fn parameter_constraints_on_feature_model_sheet_is_evaluated() {
    let root = tempdir();
    write(
        &root,
        "Features/_index.md",
        r#"---
type: FeatureModel
name: Features
featureTree:
  - name: Wdt
    id: FEAT-WDT-001
    groupKind: optional
    parameters:
      - { name: timeoutMs, type: ScalarValues::Integer, range: "10..=5000", default: 1000 }
parameterConstraints:
  - id: PC-001
    expression: "Features::Wdt.timeoutMs <= 5000"
    appliesWhen: Features::Wdt
    severity: error
---
"#,
    );

    let elements = walk_model(&root).unwrap();
    let findings = feature_model::check_feature_model(&elements);
    assert!(
        !findings.iter().any(|f| f.code == "E213"),
        "parameterConstraints: on a FeatureModel sheet should resolve the parameter path: {:#?}",
        findings
    );
}

/// Corner case (review pass): a `FeatureModel` sheet placed directly at the
/// model root has qname `""` — `featureTree:` entries must not pick up a
/// leading `::` from naively joining onto that empty prefix.
#[test]
fn feature_model_sheet_at_model_root_produces_clean_qnames() {
    let root = tempdir();
    write(
        &root,
        "_index.md",
        r#"---
type: FeatureModel
name: Root
featureTree:
  - name: Wdt
    id: FEAT-WDT-001
    groupKind: optional
---
"#,
    );

    let elements = walk_model(&root).unwrap();
    assert!(
        elements.iter().any(|e| e.qualified_name == "Wdt"),
        "expected qname 'Wdt', not '::Wdt' or similar: {:?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    let findings = validator::validate(&elements);
    let errors = errors_of(&findings);
    assert!(errors.is_empty(), "unexpected errors: {errors:#?}");
}

/// Corner case (review pass): a type-mismatched field anywhere in a
/// `featureTree:` entry (e.g. `mandatory: "yes"`, a string not a bool) must
/// not silently discard every other field in that entry via a swallowed
/// deserialize error — it surfaces as the same `E002` a malformed real `.md`
/// file would, naming the entry and the concrete error, and the entry is not
/// synthesized with corrupted (all-default) data.
#[test]
fn type_mismatched_field_in_entry_is_e002_not_silently_discarded() {
    let root = tempdir();
    write(
        &root,
        "Features/_index.md",
        r#"---
type: FeatureModel
name: Features
featureTree:
  - name: Wdt
    id: FEAT-WDT-001
    groupKind: optional
    mandatory: "yes"
---
"#,
    );

    let elements = walk_model(&root).unwrap();
    let findings = validator::validate(&elements);
    assert!(has_code(&findings.findings, "E002"), "expected E002: {:#?}", findings.findings);
    assert!(
        !has_code(&findings.findings, "E201"),
        "must not fall through to the confusing 'id is required' symptom: {:#?}",
        findings.findings
    );
    let wdt = elements.iter().find(|e| e.qualified_name == "Features::Wdt");
    assert!(
        wdt.is_none() || wdt.unwrap().frontmatter.element_type != Some(ElementType::FeatureDef),
        "a broken entry must not be synthesized as a (corrupted) FeatureDef"
    );
}

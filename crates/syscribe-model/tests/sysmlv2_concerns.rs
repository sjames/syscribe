//! Integration tests for mapping `concern def`/`concern` onto the native
//! `ConcernDef`/`Concern` schema (`REQ-TRS-SYSMLV2-023`).
//!
//! Mirrors `sysmlv2_views.rs`: raw YAML shape assertions on
//! `frontmatter.subject`/`.stakeholders`/`.supertype`/`.typedBy`, plus
//! confirming the explicit descopes (`requires:`/`assume:` stay unset) and
//! the genuine parser-level absence of `Concern*` from both
//! `PartDefBodyElement` and `PartUsageBodyElement` (broader than the
//! View-in-part-usage-only gap from the prior increment).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::ElementType;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-concerns-test-{}-{}",
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

fn codes(findings: &[syscribe_model::validator::Finding]) -> Vec<&str> {
    findings.iter().map(|f| f.code).collect()
}

#[test]
fn a_concern_def_becomes_a_real_concerndef_with_supertype() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Concerns.sysml",
        "package Concerns {\n\
         concern def BaseConcern;\n\
         concern def SafetyConcern : BaseConcern {\n\
         doc /* Safety-related stakeholder concern. */\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Concerns::SafetyConcern")
        .expect("SafetyConcern should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::ConcernDef));
    // `concern def X : Y` -- `: Y` is a supertype, never a typedBy, for the
    // definition form.
    assert_eq!(el.frontmatter.supertype.as_ref().and_then(|v| v.as_str()), Some("BaseConcern"));
    assert!(el.frontmatter.typed_by.is_none(), "{:#?}", el.frontmatter.typed_by);
    assert!(el.doc.contains("Safety-related stakeholder concern."));
}

#[test]
fn a_bare_concern_usage_becomes_a_real_concern_with_typed_by() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Concerns.sysml",
        "package Concerns {\n\
         concern def MassConcernType;\n\
         concern massConcern : MassConcernType;\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Concerns::massConcern")
        .expect("massConcern should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::Concern));
    // A bare `concern x : Y` usage -- `: Y` is a typedBy, never a
    // supertype, for the usage form.
    assert_eq!(el.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("MassConcernType"));
    assert!(el.frontmatter.supertype.is_none(), "{:#?}", el.frontmatter.supertype);
}

#[test]
fn subject_and_stakeholders_lift_from_a_concern_def_body() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Concerns.sysml",
        "package Concerns {\n\
         part def Vehicle;\n\
         concern def MassConcern {\n\
         subject target : Vehicle;\n\
         stakeholder ChiefEngineer;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Concerns::MassConcern")
        .expect("MassConcern should be a real element");
    assert_eq!(el.frontmatter.subject.as_deref(), Some("Vehicle"));
    assert_eq!(el.frontmatter.stakeholders.as_deref(), Some(&["ChiefEngineer".to_string()][..]));
    // `ConcernDef` has no `concerns:` self-field -- confirm it's never
    // populated by this mapping.
    assert!(el.frontmatter.concerns.is_none(), "{:#?}", el.frontmatter.concerns);
}

#[test]
fn requires_and_assume_stay_unset_an_explicit_descope() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Concerns.sysml",
        "package Concerns {\n\
         concern def BudgetConcern {\n\
         require constraint massBudget;\n\
         assume constraint nominalLoad;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Concerns::BudgetConcern")
        .expect("BudgetConcern should be a real element");
    // Explicit descope (REQ-TRS-SYSMLV2-023's Scope section), not an
    // implicit/accidental absence -- confirmed against a fixture that
    // actually declares both.
    assert!(el.frontmatter.requires.is_none(), "{:#?}", el.frontmatter.requires);
    assert!(el.frontmatter.assume.is_none(), "{:#?}", el.frontmatter.assume);
}

#[test]
fn a_concern_nested_inside_a_part_def_body_stays_invisible() {
    let root = tempdir();
    base_model(&root);
    // `ConcernUsage` is reachable only from `PackageBodyElement` in this
    // parser version -- confirmed absent from `PartDefBodyElement` too
    // (broader than View/Viewpoint/Rendering, which at least reached
    // `PartDefBodyElement`). Fails to parse outright, gracefully degrading
    // to W541 rather than a crash or a synthesized element.
    write(
        &root,
        "SysML2Legacy/Nested.sysml",
        "package Concerns {\n\
         concern def SomeConcern;\n\
         part def Housing {\n\
         concern innerConcern : SomeConcern;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    assert!(
        !elements.iter().any(|e| e.qualified_name.ends_with("::innerConcern")),
        "a concern nested inside a part def body should stay invisible: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    let result = validate(&elements);
    assert!(codes(&result.findings).contains(&"W541"), "{:#?}", result.findings);
}

#[test]
fn a_concern_nested_inside_a_part_usage_body_stays_invisible() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Nested.sysml",
        "package Concerns {\n\
         concern def SomeConcern;\n\
         part def Housing;\n\
         part housing : Housing {\n\
         concern innerConcern : SomeConcern;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    assert!(
        !elements.iter().any(|e| e.qualified_name.ends_with("::innerConcern")),
        "a concern nested inside a part usage body should stay invisible: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    let result = validate(&elements);
    assert!(codes(&result.findings).contains(&"W541"), "{:#?}", result.findings);
}

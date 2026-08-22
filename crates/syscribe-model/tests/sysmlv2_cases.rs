//! Integration tests for mapping `case def`/`case`, `analysis def`/
//! `analysis`, and `verification def`/`verification` onto the native
//! `CaseDef`/`Case`, `AnalysisCaseDef`/`AnalysisCase`,
//! `VerificationCaseDef`/`VerificationCase` schema
//! (`REQ-TRS-SYSMLV2-026`/`-027`/`-028`).
//!
//! Mirrors `sysmlv2_flows.rs`. Covers the real asymmetry this family has
//! that no prior increment did: only `AnalysisCaseDef`/`AnalysisCaseUsage`
//! are reachable inside a `part` *usage* body — `case`/`verification`
//! nested there fail to parse outright (`W541`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::ElementType;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-cases-test-{}-{}",
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
fn a_case_def_and_usage_lift_subject_actors_objectives_result_and_doc() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Cases.sysml",
        "package Cases {\n\
         part def System;\n\
         part def Pilot;\n\
         attribute def Outcome;\n\
         case def BaseCase;\n\
         case def MissionCase :> BaseCase {\n\
         doc /* Top-level mission case. */\n\
         subject sys : System;\n\
         actor pilot : Pilot;\n\
         objective missionObjective : Outcome;\n\
         return result : Outcome;\n\
         }\n\
         case c : MissionCase;\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let def = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Cases::MissionCase")
        .expect("MissionCase should be a real element");
    assert_eq!(def.frontmatter.element_type, Some(ElementType::CaseDef));
    assert_eq!(def.frontmatter.supertype.as_ref().and_then(|v| v.as_str()), Some("BaseCase"));
    assert_eq!(def.frontmatter.subject.as_deref(), Some("System"));
    assert_eq!(def.frontmatter.actors.as_deref(), Some(&["Pilot".to_string()][..]));
    let objectives = def.frontmatter.objectives.as_deref().expect("objectives present");
    assert_eq!(objectives.len(), 1, "{objectives:#?}");
    assert_eq!(objectives[0].as_str(), Some("missionObjective"));
    assert_eq!(def.frontmatter.result_type.as_deref(), Some("Outcome"));
    assert!(def.doc.contains("Top-level mission case."));

    let usage = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Cases::c")
        .expect("c should be a real element");
    assert_eq!(usage.frontmatter.element_type, Some(ElementType::Case));
    assert_eq!(usage.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("MissionCase"));
}

#[test]
fn an_analysis_def_is_reachable_at_all_three_nesting_levels() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Cases.sysml",
        "package Cases {\n\
         part def System;\n\
         analysis def TopAnalysis {\n\
         subject sys : System;\n\
         }\n\
         part def Housing {\n\
         analysis def NestedInDef {\n\
         subject sys : System;\n\
         }\n\
         }\n\
         part housing : Housing {\n\
         analysis def NestedInUsage {\n\
         subject sys : System;\n\
         }\n\
         analysis a : NestedInUsage;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    for qname in [
        "SysML2Legacy::Cases::TopAnalysis",
        "SysML2Legacy::Cases::Housing::NestedInDef",
        "SysML2Legacy::Cases::housing::NestedInUsage",
    ] {
        let el = elements.iter().find(|e| e.qualified_name == qname).unwrap_or_else(|| panic!("{qname} should be a real element"));
        assert_eq!(el.frontmatter.element_type, Some(ElementType::AnalysisCaseDef), "{qname}");
    }
    let usage = elements.iter().find(|e| e.qualified_name == "SysML2Legacy::Cases::housing::a").unwrap();
    assert_eq!(usage.frontmatter.element_type, Some(ElementType::AnalysisCase));
}

#[test]
fn a_verification_def_with_multiple_returns_takes_the_first_typed_one() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Cases.sysml",
        "package Cases {\n\
         part def System;\n\
         attribute def VerdictKind;\n\
         attribute def Score;\n\
         verification def MyVerification {\n\
         subject sys : System;\n\
         return verdict : VerdictKind;\n\
         return attribute score : Score;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Cases::MyVerification")
        .expect("MyVerification should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::VerificationCaseDef));
    assert_eq!(el.frontmatter.result_type.as_deref(), Some("VerdictKind"));
    // Explicit descope: no AST source for verify statements or verdict
    // semantics on this body shape.
    assert!(el.frontmatter.verifies.is_none(), "{:#?}", el.frontmatter.verifies);
    assert!(el.frontmatter.verdict_expression.is_none(), "{:#?}", el.frontmatter.verdict_expression);
    assert!(el.frontmatter.verdict_type.is_none(), "{:#?}", el.frontmatter.verdict_type);
}

#[test]
fn case_and_verification_nested_in_a_part_usage_body_stay_invisible() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Cases.sysml",
        "package Cases {\n\
         case def SomeCase;\n\
         part def Housing;\n\
         part housing : Housing {\n\
         case innerCase : SomeCase;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    assert!(
        !elements.iter().any(|e| e.qualified_name.ends_with("::innerCase")),
        "a case nested inside a part usage body should stay invisible: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    let result = validate(&elements);
    assert!(codes(&result.findings).contains(&"W541"), "{:#?}", result.findings);
}

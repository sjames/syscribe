//! Integration tests for mapping `constraint def`/`constraint`, `calc def`/`calc`,
//! `use case def`/`use case` and package-level `doc` onto native elements
//! (`REQ-TRS-SYSMLV2-033`..`-036`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-cc-test-{}-{}",
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

fn load(sysml: &str) -> (PathBuf, Vec<RawElement>) {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "S/_index.md", "---\ntype: Package\nname: S\nsysmlSubmodel: true\n---\n");
    write(&root, "S/A.sysml", sysml);
    let elements = walk_model(&root).unwrap();
    (root, elements)
}

fn find<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter().find(|e| e.qualified_name == q).unwrap_or_else(|| panic!("missing {q}"))
}

fn w543(els: &[RawElement]) -> Vec<String> {
    validate(els).findings.iter().filter(|f| f.code == "W543").map(|f| f.message.clone()).collect()
}

#[test]
fn constraint_def_and_usage_are_mapped_with_expression_and_parameters() {
    let (_r, els) = load(
        "package P {\n\
         constraint def Base;\n\
         constraint def MassLimit : Base {\n\
           doc /* Mass must stay under the limit. */\n\
           in actualMass : Real;\n\
           in maxMass : Real;\n\
           actualMass <= maxMass\n\
         }\n\
         constraint massCheck : MassLimit;\n\
         }\n",
    );
    let d = find(&els, "S::P::MassLimit");
    assert_eq!(d.frontmatter.element_type, Some(ElementType::ConstraintDef));
    assert_eq!(d.frontmatter.supertype.as_ref().and_then(|v| v.as_str()), Some("Base"));
    assert_eq!(d.frontmatter.expression.as_deref(), Some("actualMass <= maxMass"));
    let params = d.frontmatter.parameters.as_ref().expect("parameters lifted");
    assert_eq!(params.len(), 2);
    assert_eq!(params[0]["name"].as_str(), Some("actualMass"));
    assert_eq!(params[0]["typedBy"].as_str(), Some("Real"));
    assert_eq!(params[0]["direction"].as_str(), Some("in"));
    assert!(d.doc.contains("Mass must stay under the limit."));

    let u = find(&els, "S::P::massCheck");
    assert_eq!(u.frontmatter.element_type, Some(ElementType::Constraint));
    assert_eq!(u.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("MassLimit"));
    assert!(u.frontmatter.supertype.is_none());
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn constraint_usage_inside_part_def_is_scoped_under_the_part() {
    let (_r, els) = load(
        "package P {\n constraint def K { in x : Real; x > 0 }\n part def Eng { constraint chk : K; }\n}\n",
    );
    let u = find(&els, "S::P::Eng::chk");
    assert_eq!(u.frontmatter.element_type, Some(ElementType::Constraint));
}

#[test]
fn calc_def_is_mapped_with_parameters_return_type_and_body() {
    let (_r, els) = load(
        "package P {\n\
         calc def FuelEconomy {\n\
           doc /* Distance over fuel. */\n\
           in distance : Real;\n\
           in fuel : Real;\n\
           return economy : Real;\n\
           distance / fuel\n\
         }\n\
         calc def Bare;\n\
         part def Veh { calc eco : FuelEconomy; }\n\
         }\n",
    );
    let c = find(&els, "S::P::FuelEconomy");
    assert_eq!(c.frontmatter.element_type, Some(ElementType::CalculationDef));
    assert_eq!(c.frontmatter.return_type.as_deref(), Some("Real"));
    assert_eq!(c.frontmatter.body.as_deref(), Some("distance / fuel"));
    assert_eq!(c.frontmatter.body_language.as_deref(), Some("kerml"));
    let params = c.frontmatter.parameters.as_ref().unwrap();
    assert_eq!(params.len(), 3);
    assert_eq!(params[2]["direction"].as_str(), Some("return"));
    assert!(c.doc.contains("Distance over fuel."));

    let bare = find(&els, "S::P::Bare");
    assert!(bare.frontmatter.body.is_none() && bare.frontmatter.body_language.is_none());

    let u = find(&els, "S::P::Veh::eco");
    assert_eq!(u.frontmatter.element_type, Some(ElementType::Calculation));
    assert_eq!(u.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("FuelEconomy"));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn use_case_def_and_usage_are_mapped_with_case_fields() {
    let (_r, els) = load(
        "package P {\n\
         part def Vehicle;\n\
         part def Driver;\n\
         use case def Drive : Base {\n\
           doc /* Drive the vehicle. */\n\
           subject v : Vehicle;\n\
           actor d : Driver;\n\
         }\n\
         use case def Base;\n\
         use case drive1 : Drive;\n\
         }\n",
    );
    let d = find(&els, "S::P::Drive");
    assert_eq!(d.frontmatter.element_type, Some(ElementType::UseCaseDef));
    assert_eq!(d.frontmatter.supertype.as_ref().and_then(|v| v.as_str()), Some("Base"));
    assert_eq!(d.frontmatter.subject.as_deref(), Some("Vehicle"));
    assert_eq!(d.frontmatter.actors.as_ref().unwrap(), &vec!["Driver".to_string()]);
    assert!(d.doc.contains("Drive the vehicle."));
    let u = find(&els, "S::P::drive1");
    assert_eq!(u.frontmatter.element_type, Some(ElementType::UseCase));
    assert_eq!(u.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("Drive"));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn package_level_doc_lifts_onto_the_package_and_is_no_longer_unmapped() {
    let (_r, els) = load(
        "package P {\n doc /* Top-level package docs. */\n part def X;\n package In { doc /* Inner docs. */ part def Y; }\n}\n",
    );
    assert!(find(&els, "S::P").doc.contains("Top-level package docs."));
    assert!(find(&els, "S::P::In").doc.contains("Inner docs."));
    assert!(!find(&els, "S::P").doc.contains("Inner docs."));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn mapped_kinds_add_no_validation_errors() {
    let (_r, els) = load(
        "package P {\n constraint def K { in x : Real; x > 0 }\n calc def C { in a : Real; return r : Real; a * 2 }\n use case def U;\n}\n",
    );
    let result = validate(&els);
    assert_eq!(result.errors().count(), 0, "{:#?}", result.findings);
}

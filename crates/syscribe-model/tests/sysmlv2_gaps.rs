//! Integration tests for `REQ-TRS-SYSMLV2-043`..`-048` (ingestion gap closure) and
//! `REQ-TRS-SYSMLV2-049`..`-052` (export round trips).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::sysmlv2::export::export_sysml;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-gaps-test-{}-{}",
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

fn load(sysml: &str) -> Vec<RawElement> {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "S/_index.md", "---\ntype: Package\nname: S\nsysmlSubmodel: true\n---\n");
    write(&root, "S/A.sysml", sysml);
    walk_model(&root).unwrap()
}

fn find<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter().find(|e| e.qualified_name == q).unwrap_or_else(|| panic!("missing {q}"))
}

fn w543(els: &[RawElement]) -> Vec<String> {
    validate(els).findings.iter().filter(|f| f.code == "W543").map(|f| f.message.clone()).collect()
}

fn s(v: &Option<serde_yaml::Value>) -> Option<&str> {
    v.as_ref().and_then(|v| v.as_str())
}

fn errors(els: &[RawElement]) -> Vec<String> {
    validate(els)
        .findings
        .iter()
        .filter(|f| f.code.starts_with('E') && f.file.ends_with(".sysml"))
        .map(|f| format!("{} {}", f.code, f.message))
        .collect()
}

#[test]
fn alias_in_named_package_lifts_onto_package_aliases() {
    let els = load("package P {\n part def Engine;\n alias Motor for Engine;\n alias <m> Mot2 for P::Engine;\n}\n");
    let p = find(&els, "S::P");
    let aliases = p.frontmatter.aliases.as_ref().expect("aliases lifted");
    assert_eq!(aliases.len(), 2);
    assert_eq!(aliases[0]["name"].as_str(), Some("Motor"));
    assert_eq!(aliases[0]["for"].as_str(), Some("Engine"));
    assert_eq!(aliases[1]["name"].as_str(), Some("Mot2"));
    assert_eq!(aliases[1]["shortName"].as_str(), Some("m"));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn library_package_and_namespace_become_packages() {
    let els = load(
        "library package Lib {\n part def Gear;\n library package Inner { part def Cog; }\n}\n\
         namespace Ns { part def Bolt; }\n\
         package Plain { library package Nested { part def Nut; } }\n",
    );
    for (q, t) in [
        ("S::Lib", ElementType::Package),
        ("S::Lib::Gear", ElementType::PartDef),
        ("S::Lib::Inner", ElementType::Package),
        ("S::Lib::Inner::Cog", ElementType::PartDef),
        ("S::Ns", ElementType::Package),
        ("S::Ns::Bolt", ElementType::PartDef),
        ("S::Plain::Nested::Nut", ElementType::PartDef),
    ] {
        assert_eq!(find(&els, q).frontmatter.element_type, Some(t), "{q}");
    }
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn metadata_def_becomes_a_metadata_def_element() {
    let els = load(
        "package P {\n metadata def Base;\n abstract metadata def Safety :> Base {\n doc /* Safety tag. */\n }\n}\n",
    );
    let m = find(&els, "S::P::Safety");
    assert_eq!(m.frontmatter.element_type, Some(ElementType::MetadataDef));
    assert_eq!(m.frontmatter.is_abstract, Some(true));
    assert_eq!(s(&m.frontmatter.supertype), Some("Base"));
    assert!(m.doc.contains("Safety tag."));
    assert_eq!(find(&els, "S::P::Base").frontmatter.element_type, Some(ElementType::MetadataDef));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
    assert!(errors(&els).is_empty(), "{:?}", errors(&els));
}

#[test]
fn package_level_satisfy_lifts_into_the_subject_when_it_resolves() {
    let els = load(
        "package P {\n requirement def R1;\n requirement def R2;\n part def Engine;\n part eng : Engine;\n\
         satisfy R1 by Engine;\n satisfy R2 by eng;\n satisfy R1 by Missing;\n satisfy R2;\n}\n",
    );
    assert_eq!(find(&els, "S::P::Engine").frontmatter.satisfies.as_deref(), Some(&["R1".to_string()][..]));
    assert_eq!(find(&els, "S::P::eng").frontmatter.satisfies.as_deref(), Some(&["R2".to_string()][..]));
    let w = w543(&els);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("satisfy x2"), "unresolved subject + bare shorthand stay counted: {w:?}");
}

#[test]
fn doc_on_requirement_def_and_requirement_lifts() {
    let els = load(
        "package P {\n requirement def R {\n doc /* The engine shall run. */\n }\n requirement r : R {\n doc /* Usage doc. */\n }\n}\n",
    );
    assert!(find(&els, "S::P::R").doc.contains("The engine shall run."));
    assert!(find(&els, "S::P::r").doc.contains("Usage doc."));
}

#[test]
fn usage_multiplicity_subsets_and_redefines_are_ingested() {
    let els = load(
        "package P {\n part def Axle;\n part def Car {\n part wheels : Axle [2];\n part spare : Axle [0..1] :> wheels;\n\
         part any : Axle [*];\n part many : Axle [1..*];\n part wheels2 : Axle :>> wheels;\n\
         attribute mass : Real :>> base;\n port p : Axle [3];\n item cargo [0..*] : Axle;\n }\n}\n",
    );
    let f = |q: &str| find(&els, q).frontmatter.clone();
    assert_eq!(f("S::P::Car::wheels").multiplicity.as_deref(), Some("2"));
    assert_eq!(f("S::P::Car::spare").multiplicity.as_deref(), Some("0..1"));
    assert_eq!(f("S::P::Car::spare").subsets, Some(vec!["wheels".to_string()]));
    assert_eq!(f("S::P::Car::any").multiplicity.as_deref(), Some("*"));
    assert_eq!(f("S::P::Car::many").multiplicity.as_deref(), Some("1..*"));
    assert_eq!(s(&f("S::P::Car::wheels2").redefines), Some("wheels"));
    assert_eq!(s(&f("S::P::Car::mass").redefines), Some("base"));
    assert_eq!(f("S::P::Car::p").multiplicity.as_deref(), Some("3"));
    assert_eq!(f("S::P::Car::cargo").multiplicity.as_deref(), Some("0..*"));
    assert!(find(&els, "S::P::Car::wheels").frontmatter.subsets.is_none());
}

/// Export `root`'s model, re-import the text as a submodel `Imp`, and return (text, elements).
fn round_trip(root: &Path) -> (String, Vec<RawElement>) {
    let model = walk_model(root).unwrap();
    let text = export_sysml(&model, None).unwrap().text;
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Imp/_index.md", "---\ntype: Package\nname: Imp\nsysmlSubmodel: true\n---\n");
    write(&r, "Imp/out.sysml", &text);
    let els = walk_model(&r).unwrap();
    let w541: Vec<_> = els.iter().flat_map(|e| e.derive_findings.iter()).filter(|(c, _, _)| c == "W541").collect();
    assert!(w541.is_empty(), "parse-back failed: {w541:?}\n{text}");
    (text, els)
}

fn native_root() -> PathBuf {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    r
}

#[test]
fn export_round_trips_multiplicity_subsets_and_redefines() {
    let r = native_root();
    write(&r, "Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
    write(&r, "Sys/Axle.md", "---\ntype: PartDef\nname: Axle\n---\n");
    write(&r, "Sys/Car.md", "---\ntype: PartDef\nname: Car\n---\n");
    write(&r, "Sys/Car/wheels.md", "---\ntype: Part\nname: wheels\ntypedBy: Sys::Axle\nmultiplicity: \"2\"\n---\n");
    write(
        &r,
        "Sys/Car/spare.md",
        "---\ntype: Part\nname: spare\ntypedBy: Sys::Axle\nmultiplicity: \"0..1\"\nsubsets: [Sys::Car::wheels]\n---\n",
    );
    write(
        &r,
        "Sys/Car/front.md",
        "---\ntype: Part\nname: front\ntypedBy: Sys::Axle\nmultiplicity: \"1..*\"\nredefines: Sys::Car::wheels\n---\n",
    );
    let (text, els) = round_trip(&r);
    assert!(text.contains(":> Sys::Car::wheels") && text.contains(":>> Sys::Car::wheels"), "{text}");
    let f = |q: &str| find(&els, q).frontmatter.clone();
    assert_eq!(f("Imp::Sys::Car::wheels").multiplicity.as_deref(), Some("2"));
    let spare = f("Imp::Sys::Car::spare");
    assert_eq!(spare.multiplicity.as_deref(), Some("0..1"));
    assert_eq!(spare.subsets, Some(vec!["Sys::Car::wheels".to_string()]));
    let front = f("Imp::Sys::Car::front");
    assert_eq!(front.multiplicity.as_deref(), Some("1..*"));
    assert_eq!(s(&front.redefines), Some("Sys::Car::wheels"));
}

#[test]
fn export_attribute_unit_is_a_literal_with_unit() {
    let r = native_root();
    write(&r, "Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
    write(&r, "Sys/Mass.md", "---\ntype: AttributeDef\nname: Mass\n---\n");
    write(
        &r,
        "Sys/Engine.md",
        "---\ntype: PartDef\nname: Engine\nfeatures:\n  - name: mass\n    typedBy: Sys::Mass\n    value: 5\n    unit: kg\n  - name: limit\n    typedBy: Sys::Mass\n    unit: kg\n---\n",
    );
    let (text, _els) = round_trip(&r);
    assert!(text.contains("attribute mass : Sys::Mass = 5 [kg];"), "{text}");
    assert!(!text.contains("unit: kg\n    attribute mass"), "{text}");
    assert!(text.contains("attribute limit : Sys::Mass; // unit: kg"), "{text}");
    // The literal really is a literal-with-unit to the parser.
    let root = sysml_v2_parser::parse(&text).expect("parses");
    let dump = format!("{root:?}");
    assert!(dump.contains("LiteralWithUnit"), "{dump}");
}

#[test]
fn export_satisfy_and_verify_become_real_statements() {
    let r = native_root();
    write(&r, "Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
    write(&r, "Sys/Fuel.md", "---\ntype: ItemDef\nname: Fuel\nsatisfies: [Sys::R1]\n---\n");
    write(&r, "Sys/R1.md", "---\ntype: RequirementDef\nname: R1\nverifies: [Sys::Fuel]\n---\nDoc of R1.\n");
    let (text, els) = round_trip(&r);
    assert!(text.contains("satisfy Sys::R1 by Sys::Fuel;"), "{text}");
    assert!(!text.contains("// satisfies"), "{text}");
    assert!(text.contains("verify Sys::Fuel;"), "{text}");
    assert_eq!(find(&els, "Imp::Sys::Fuel").frontmatter.satisfies, Some(vec!["Sys::R1".to_string()]));
    let r1 = find(&els, "Imp::Sys::R1");
    assert_eq!(r1.frontmatter.verifies, Some(vec!["Sys::Fuel".to_string()]));
    assert!(r1.doc.contains("Doc of R1."), "requirement doc round-trips via REQ-047");
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn export_constraint_and_calc_bodies_round_trip() {
    let r = native_root();
    write(&r, "Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
    write(
        &r,
        "Sys/Limit.md",
        "---\ntype: ConstraintDef\nname: Limit\nparameters:\n  - {name: actual, typedBy: Real, direction: in}\n  - {name: maxv, typedBy: Real, direction: in}\nexpression: actual <= maxv\n---\nLimit doc.\n",
    );
    write(
        &r,
        "Sys/Eco.md",
        "---\ntype: CalculationDef\nname: Eco\nparameters:\n  - {name: d, typedBy: Real, direction: in}\n  - {name: f, typedBy: Real, direction: in}\n  - {name: economy, typedBy: Real, direction: return}\nbody: d / f\nbodyLanguage: kerml\n---\n",
    );
    write(&r, "Sys/Odd.md", "---\ntype: ConstraintDef\nname: Odd\nexpression: \"x &&& y ??\"\n---\n");
    write(&r, "Sys/Ph.md", "---\ntype: CalculationDef\nname: Ph\nbody: \"<conditional expression>\"\n---\n");
    let (text, els) = round_trip(&r);
    let limit = find(&els, "Imp::Sys::Limit");
    assert_eq!(limit.frontmatter.expression.as_deref(), Some("actual <= maxv"));
    assert_eq!(limit.frontmatter.parameters.as_ref().map(|p| p.len()), Some(2));
    assert_eq!(limit.frontmatter.parameters.as_ref().unwrap()[0]["typedBy"].as_str(), Some("Real"));
    let eco = find(&els, "Imp::Sys::Eco");
    assert_eq!(eco.frontmatter.body.as_deref(), Some("d / f"));
    assert_eq!(eco.frontmatter.return_type.as_deref(), Some("Real"));
    let params = eco.frontmatter.parameters.as_ref().unwrap();
    assert_eq!(params.len(), 3);
    assert_eq!(params[2]["direction"].as_str(), Some("return"));
    // Text that is not valid SysML v2, or a placeholder, is a comment, never source.
    assert!(text.contains("// expression (not valid SysML v2 text): x &&& y ??"), "{text}");
    assert!(text.contains("// expression (not valid SysML v2 text): <conditional expression>"), "{text}");
    assert!(find(&els, "Imp::Sys::Odd").frontmatter.expression.is_none());
}


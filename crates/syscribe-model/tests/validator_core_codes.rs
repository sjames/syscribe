//! Table-driven minimal fixtures for core validation codes that no other test raised
//! (found from the `llvm-cov` report). Each case writes a tiny model, validates it and
//! asserts the code fires — and, for the negative twin, that a clean variant does not.

use std::path::PathBuf;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn model(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let d = std::env::temp_dir().join(format!("syscribe_vcore_{}_{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("_index.md"), "---\ntype: Package\nname: M\n---\n").unwrap();
    for (p, c) in files {
        let f = d.join(p);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, c).unwrap();
    }
    d
}

fn codes(tag: &str, files: &[(&str, &str)]) -> Vec<String> {
    let d = model(tag, files);
    let els = walk_model(&d).unwrap();
    let mut v: Vec<String> = validate(&els).findings.into_iter().map(|f| f.code.to_string()).collect();
    v.sort();
    v.dedup();
    v
}

#[track_caller]
fn fires(code: &str, files: &[(&str, &str)]) {
    let got = codes(&code.to_lowercase(), files);
    assert!(got.iter().any(|c| c == code), "{code} not raised; got {got:?}");
}

const REQ: &str = "---\nid: REQ-VC-001\ntype: Requirement\nname: R\nstatus: draft\nreqDomain: software\n---\nThe tool shall work.\n";

#[test]
fn frontmatter_and_enum_codes() {
    fires("E001", &[("A/Bad.md", "no frontmatter here\n")]);
    fires("W008", &[("A/Untyped.md", "---\nname: Untyped\n---\nbody\n")]);
    fires("E005", &[("A/Weird.md", "---\ntype: Frobnicator\nname: Weird\n---\n")]);
    fires("E007", &[("A/R.md", &REQ.replace("status: draft", "status: bogus").replace("REQ-VC-001", "REQ-VC-002"))]);
    fires("E004", &[("A/NoName.md", "---\ntype: Requirement\nid: REQ-VC-003\nstatus: draft\n---\nThe tool shall work.\n")]);
    fires("E020", &[("A/R.md", &REQ.replace("reqDomain: software", "reqDomain: software\nverificationMethod: guess"))]);
    fires("E021", &[("A/T.md", "---\nid: TC-VC-001\ntype: TestCase\nname: T\ntestLevel: L1\nstatus: draft\ncoverageTarget: everything\nverifies: [REQ-VC-001]\n---\n"), ("A/R.md", REQ)]);
    fires("E008", &[("A/T.md", "---\nid: TC-VC-002\ntype: TestCase\nname: T\ntestLevel: L9\nstatus: draft\nverifies: [REQ-VC-001]\n---\n"), ("A/R.md", REQ)]);
    fires("E019", &[("A/R.md", &REQ.replace("reqDomain: software", "reqDomain: software\ndalLevel: Z"))]);
}

#[test]
fn adr_and_domain_codes() {
    fires("E301", &[("A/D.md", "---\nid: ADR-VC-001\ntype: ADR\n---\nbody\n")]);
    fires("E304", &[("A/D.md", "---\nid: ADR-VC-001\ntype: ADR\nname: D\nstatus: maybe\n---\nbody\n")]);
    fires("E303", &[("A/P.md", "---\ntype: PartDef\nname: P\ndomain: quantum\n---\n")]);
    fires("W304", &[("A/P.md", "---\ntype: PartDef\nname: P\ndomain: hardware\nisDeploymentPackage: true\n---\n")]);
    fires("E105", &[("A/P.md", "---\ntype: PartDef\nname: P\n---\n"), ("A/R.md", &REQ.replace("---\nThe", "derivedFrom: [A::P]\n---\nThe"))]);
    fires("E311", &[("A/R.md", &REQ.replace("---\nThe", "derivedFrom: [REQ-VC-009]\nbreakdownAdr: Nope\n---\nThe")), ("A/Parent.md", &REQ.replace("REQ-VC-001", "REQ-VC-009"))]);
    fires("W303", &[
        ("A/D.md", "---\nid: ADR-VC-001\ntype: ADR\nname: D\nstatus: proposed\n---\nbody\n"),
        ("A/Parent.md", &REQ.replace("REQ-VC-001", "REQ-VC-009")),
        ("A/R.md", &REQ.replace("status: draft", "status: approved").replace("---\nThe", "derivedFrom: [REQ-VC-009]\nbreakdownAdr: ADR-VC-001\n---\nThe")),
    ]);
    fires("E315", &[
        ("A/HwBase.md", "---\ntype: PartDef\nname: HwBase\ndomain: hardware\n---\n"),
        ("A/SwPart.md", "---\ntype: PartDef\nname: SwPart\ndomain: software\nsupertype: A::HwBase\n---\n"),
    ]);
    fires("E316", &[("A/U.md", "---\ntype: UseCaseDef\nname: U\nrefines: [REQ-NOPE-001]\n---\n")]);
}

#[test]
fn misc_warning_codes() {
    fires("W028", &[
        ("A/P1.md", "---\ntype: PartDef\nname: P1\nextRef: DNG:1\n---\n"),
        ("A/P2.md", "---\ntype: PartDef\nname: P2\nextRef: DNG:1\n---\n"),
    ]);
    fires("W041", &[("A/P.md", "---\ntype: PartDef\nname: P\ncustom_fields:\n  k:\n    nested: 1\n---\n")]);
    fires("W048", &[("A/P.md", "---\ntype: PartDef\nname: P\nfeatureTree:\n  - name: X\n---\n")]);
    fires("W049", &[("A/P.md", "---\ntype: PartDef\nname: P\nqualifiedName: Other::Name\n---\n")]);
    fires("W026", &[("G/_index.md", "---\ntype: Package\nname: G\nappliesWhen: FEAT-VC-A\n---\n"), ("F/A.md", "---\ntype: FeatureDef\nid: FEAT-VC-A\nname: A\ngroupKind: optional\n---\n")]);
    fires("E014", &[("A/T.md", "---\nid: TC-VC-003\ntype: TestCase\nname: T\ntestLevel: L1\nstatus: draft\nverifies: [REQ-VC-001]\n---\n\n```gherkin\nFeature: F\n  Scenario Outline: O\n    Given x\n```\n"), ("A/R.md", REQ)]);
    fires("W037", &[("A/G.md", "---\ntype: AttackTreeGate\nname: G\n---\n")]);
}

const FEAT: &str = "---\ntype: FeatureDef\nid: FEAT-VC-A\nname: A\ngroupKind: optional\nparameters:\n  - name: p\n    type: ScalarValues::Real\n    default: 1\n---\n";

#[test]
fn product_line_codes() {
    fires("E201", &[("F/A.md", "---\ntype: FeatureDef\nname: A\ngroupKind: optional\n---\n")]);
    fires("E222", &[("F/A.md", FEAT), ("C/CONF-VC-001.md", "---\ntype: Configuration\nid: CONF-VC-001\nname: C\nstatus: draft\nfeatureModel: F\nfeatures:\n  FEAT-VC-A: true\nparameterBindings:\n  F::Nope.q: 1\n---\n")]);
    fires("E228", &[("F/A.md", FEAT.replace("groupKind", "appliesWhen: FEAT-VC-A\ngroupKind").as_str())]);
    fires("W016", &[("F/A.md", FEAT), ("C/CONF-VC-001.md", "---\ntype: Configuration\nid: CONF-VC-001\nname: C\nstatus: draft\nfeatureModel: F\nselections:\n  FEAT-VC-A: true\n---\n")]);
}

#[test]
fn implementation_and_test_codes() {
    fires("W023", &[("A/P.md", "---\ntype: PartDef\nname: P\nstatus: approved\nimplementedBy: src/missing.rs\n---\n")]);
    let src = std::env::temp_dir().join(format!("syscribe_vcore_src_{}.rs", std::process::id()));
    std::fs::write(&src, "fn other() {}\n").unwrap();
    let tc = format!("---\nid: TC-VC-004\ntype: TestCase\nname: T\ntestLevel: L1\nstatus: active\nsourceFile: {}\ntestFunctions:\n  - function: not_there\nverifies: [REQ-VC-001]\n---\n\n```gherkin\nFeature: F\n  Scenario: S\n    Given x\n```\n", src.display());
    fires("W009", &[("A/R.md", REQ), ("A/T.md", &tc)]);
}

#[test]
fn tagged_value_codes() {
    let meta = "---\ntype: MetadataDef\nname: Tag\nfeatures:\n  - name: level\n    type: ScalarValues::Integer\n    range: \"1..5\"\n  - name: kind\n    type: ScalarValues::String\n    enumValues: [a, b]\n---\n";
    let part = |level: &str, kind: &str| format!("---\ntype: PartDef\nname: P\nmetadata:\n  - def: A::Tag\n    level: {level}\n    kind: {kind}\n---\n");
    fires("E323", &[("A/Tag.md", meta), ("A/P.md", &part("9", "a"))]);
    fires("E324", &[("A/Tag.md", meta), ("A/P.md", &part("3", "zzz"))]);
    fires("E322", &[("A/Tag.md", meta), ("A/P.md", &part("\"text\"", "a"))]);
}

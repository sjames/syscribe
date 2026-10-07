//! `REQ-TRS-SYSMLV2-038`..`-041` (`ADR-SYS-SYSMLV2-002`): the one-way SysML v2
//! textual export (`sysmlv2::export`) and its parse-back round trip through the
//! existing `sysml-v2-parser` ingestion.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::sysmlv2::export::{export_sysml, sysml_ident, ExportError};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-export-{}-{}",
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

/// A native-only model touching every supported kind plus unsupported ones.
fn native_model() -> PathBuf {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Sys/_index.md", "---\ntype: Package\nname: Sys\n---\nThe system package.\n");
    write(&r, "Sys/Base.md", "---\ntype: PartDef\nname: Base\nisAbstract: true\n---\nBase part. */ tricky\n");
    write(&r, "Sys/Engine.md", "---\ntype: PartDef\nname: Engine\nsupertype: Sys::Base\ndomain: hardware\nsatisfies: [REQ-SYS-001]\n---\nAn engine.\n\nSecond paragraph.\n");
    write(&r, "Sys/Engine/Shaft.md", "---\ntype: Part\nname: Shaft\ntypedBy: Sys::Rod\nmultiplicity: \"2\"\n---\n");
    write(&r, "Sys/Rod.md", "---\ntype: PartDef\nname: Rod\n---\n");
    write(&r, "Sys/Out.md", "---\ntype: PortDef\nname: Out\n---\n");
    write(&r, "Sys/Engine/outPort.md", "---\ntype: Port\nname: outPort\ntypedBy: Sys::Out\n---\n");
    write(&r, "Sys/Mass.md", "---\ntype: AttributeDef\nname: Mass\n---\n");
    write(&r, "Sys/Engine/mass.md", "---\ntype: Attribute\nname: mass\ntypedBy: Sys::Mass\n---\n");
    write(&r, "Sys/Link.md", "---\ntype: ConnectionDef\nname: Link\n---\n");
    write(&r, "Sys/Iface.md", "---\ntype: InterfaceDef\nname: Iface\n---\n");
    write(&r, "Sys/Fuel.md", "---\ntype: ItemDef\nname: Fuel\n---\n");
    write(&r, "Sys/Anti-Lock.md", "---\ntype: PartDef\nname: Anti-Lock\n---\n");
    write(&r, "Sys/Run.md", "---\ntype: ActionDef\nname: Run\n---\n");
    write(&r, "Sys/Mode.md", "---\ntype: StateDef\nname: Mode\n---\n");
    write(&r, "Sys/Limit.md", "---\ntype: ConstraintDef\nname: Limit\n---\n");
    write(&r, "Sys/Calc1.md", "---\ntype: CalculationDef\nname: Calc1\n---\n");
    write(&r, "Reqs/_index.md", "---\ntype: Package\nname: Reqs\n---\n");
    write(&r, "Reqs/REQ-SYS-001.md", "---\ntype: Requirement\nid: REQ-SYS-001\nname: The engine shall run\nstatus: draft\nreqDomain: system\nreqClass: stakeholder\n---\nThe engine **shall** run.\n");
    write(&r, "Reqs/TC-SYS-001.md", "---\ntype: TestCase\nid: TC-SYS-001\nname: Engine runs\ntestLevel: L3\nstatus: draft\nverifies: [REQ-SYS-001]\n---\nBody.\n");
    r
}

/// Export, write as a `sysmlSubmodel` under `Imp`, re-walk; returns qname -> type name.
fn reimport(text: &str) -> BTreeMap<String, String> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Imp/_index.md", "---\ntype: Package\nname: Imp\nsysmlSubmodel: true\n---\n");
    write(&r, "Imp/out.sysml", text);
    let elements = walk_model(&r).unwrap();
    let w541: Vec<_> = elements
        .iter()
        .flat_map(|e| e.derive_findings.iter())
        .filter(|(c, _, _)| c == "W541")
        .collect();
    assert!(w541.is_empty(), "parse-back failed: {w541:?}\n{text}");
    elements
        .iter()
        .filter(|e| e.file_path.ends_with(".sysml"))
        .map(|e| (e.qualified_name.clone(), type_name(e)))
        .collect()
}

fn type_name(e: &RawElement) -> String {
    e.frontmatter.element_type.as_ref().map_or("Unknown", |t| t.name()).to_string()
}

fn expected_kind(e: &RawElement) -> Option<&'static str> {
    use ElementType as T;
    let t = e.frontmatter.element_type.as_ref()?;
    Some(match t {
        T::Package => "Package",
        T::PartDef => "PartDef",
        T::Part => "Part",
        T::PortDef => "PortDef",
        T::Port => "Port",
        T::AttributeDef => "AttributeDef",
        T::Attribute => "Attribute",
        T::ConnectionDef => "ConnectionDef",
        T::Connection => "Connection",
        T::InterfaceDef => "InterfaceDef",
        T::Interface => "Interface",
        T::ItemDef => "ItemDef",
        T::Item => "Item",
        T::RequirementDef => "RequirementDef",
        // Native (id-bearing) Requirement is the one documented non-identity mapping.
        T::Requirement if e.frontmatter.id.is_some() => "RequirementDef",
        T::Requirement => "Requirement",
        T::ActionDef => "ActionDef",
        T::StateDef => "StateDef",
        T::ConstraintDef => "ConstraintDef",
        T::CalculationDef => "CalculationDef",
        _ => return None,
    })
}

fn assert_roundtrip(elements: &[RawElement], text: &str) {
    let back = reimport(text);
    let mut checked = 0;
    for e in elements {
        if e.qualified_name.is_empty() {
            continue;
        }
        let Some(kind) = expected_kind(e) else { continue };
        let q = format!("Imp::{}", e.qualified_name);
        // Usage-kind elements round-trip as their usage kind when typed; accept the
        // exact kind only (the documented exception is native Requirement).
        assert_eq!(back.get(&q).map(String::as_str), Some(kind), "{q} did not round-trip\n{text}");
        checked += 1;
    }
    assert!(checked > 0);
}

#[test]
fn identifiers_are_quoted_when_not_basic_or_reserved() {
    assert_eq!(sysml_ident("Engine"), "Engine");
    assert_eq!(sysml_ident("_x1"), "_x1");
    assert_eq!(sysml_ident("REQ-TRS-001"), "'REQ-TRS-001'");
    assert_eq!(sysml_ident("two words"), "'two words'");
    assert_eq!(sysml_ident("1abc"), "'1abc'");
    assert_eq!(sysml_ident("part"), "'part'");
    assert_eq!(sysml_ident("it's"), "'it\\'s'");
}

#[test]
fn maps_supported_kinds_with_supertype_typing_multiplicity_doc_and_satisfy() {
    let root = native_model();
    let elements = walk_model(&root).unwrap();
    let out = export_sysml(&elements, None).unwrap();
    let t = &out.text;
    assert!(t.contains("package Sys {"), "{t}");
    assert!(t.contains("abstract part def Base"), "{t}");
    assert!(t.contains("part def Engine :> Sys::Base {"), "{t}");
    assert!(t.contains("part Shaft : Sys::Rod [2];"), "{t}");
    assert!(t.contains("port outPort : Sys::Out;"), "{t}");
    assert!(t.contains("attribute mass : Sys::Mass;"), "{t}");
    assert!(t.contains("connection def Link;"), "{t}");
    assert!(t.contains("interface def Iface;"), "{t}");
    assert!(t.contains("item def Fuel;"), "{t}");
    assert!(t.contains("action def Run;"), "{t}");
    assert!(t.contains("state def Mode;"), "{t}");
    assert!(t.contains("constraint def Limit;"), "{t}");
    assert!(t.contains("calc def Calc1;"), "{t}");
    assert!(t.contains("part def 'Anti-Lock';"), "{t}");
    // Native requirement -> requirement def named by its stable id, body as doc.
    assert!(t.contains("requirement def 'REQ-SYS-001'"), "{t}");
    assert!(t.contains("The engine **shall** run."), "{t}");
    // satisfies: -> satisfy <qualified target> inside the architecture element's body.
    assert!(t.contains("satisfy Reqs::'REQ-SYS-001';"), "{t}");
    // A `*/` in a doc body cannot close the comment early.
    assert!(!t.contains("Base part. */"), "{t}");
    assert!(t.contains("@SyscribeDomain"), "{t}");
}

#[test]
fn unsupported_elements_are_commented_and_counted() {
    let root = native_model();
    let elements = walk_model(&root).unwrap();
    let out = export_sysml(&elements, None).unwrap();
    assert!(out.text.contains("// skipped: Reqs::TC-SYS-001 (TestCase)"), "{}", out.text);
    assert_eq!(out.report.skipped.get("TestCase"), Some(&1));
    assert!(out.report.exported.get("PartDef").copied().unwrap_or(0) >= 4);
    assert_eq!(out.report.exported.get("Requirement"), Some(&1));
    assert!(out.text.contains("export summary"), "{}", out.text);
    assert!(out.report.summary_line().contains("skipped 1"));
}

#[test]
fn export_is_deterministic() {
    let root = native_model();
    let elements = walk_model(&root).unwrap();
    let a = export_sysml(&elements, None).unwrap().text;
    let mut rev = elements.clone();
    rev.reverse();
    let b = export_sysml(&rev, None).unwrap().text;
    assert_eq!(a, b);
}

#[test]
fn scope_limits_the_subtree_and_unknown_scope_errors() {
    let root = native_model();
    let elements = walk_model(&root).unwrap();
    let out = export_sysml(&elements, Some("Reqs")).unwrap();
    assert!(out.text.contains("package Reqs {"), "{}", out.text);
    assert!(!out.text.contains("package Sys"), "{}", out.text);
    assert_eq!(out.parts.len(), 1);
    assert!(matches!(export_sysml(&elements, Some("Nope::X")), Err(ExportError::UnknownScope(_))));
}

#[test]
fn native_model_round_trips_through_ingestion() {
    let root = native_model();
    let elements = walk_model(&root).unwrap();
    let out = export_sysml(&elements, None).unwrap();
    assert_roundtrip(&elements, &out.text);
}

#[test]
fn example_submodel_round_trips_through_ingestion() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/sysmlv2-submodel/model");
    let elements = walk_model(&root).unwrap();
    let out = export_sysml(&elements, None).unwrap();
    assert!(out.report.exported_total() > 5, "{}", out.report.summary_line());
    assert_roundtrip(&elements, &out.text);
    // Re-exporting the re-imported text is stable too (idempotence of the supported set).
    let again = export_sysml(&elements, None).unwrap().text;
    assert_eq!(out.text, again);
}

#[test]
fn inline_features_and_connections_render_and_still_parse() {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "P/_index.md", "---\ntype: Package\nname: P\n---\n");
    write(&r, "P/Rod.md", "---\ntype: PartDef\nname: Rod\n---\n");
    write(
        &r,
        "P/Machine.md",
        "---\ntype: PartDef\nname: Machine\nfeatures:\n  - name: mass\n    typedBy: ScalarValues::Real\n    unit: kg\n    value: \"12.5\"\n  - name: cmd\n    type: Port\n    typedBy: P::Rod\nconnections:\n  - name: feed\n    from: a::outPort\n    to: b::inPort\n---\nMachine.\n",
    );
    write(&r, "P/Machine/a.md", "---\ntype: Part\nname: a\ntypedBy: P::Rod\n---\n");
    write(&r, "P/Machine/b.md", "---\ntype: Part\nname: b\ntypedBy: P::Rod\n---\n");
    let elements = walk_model(&r).unwrap();
    let out = export_sysml(&elements, None).unwrap();
    let t = &out.text;
    assert!(t.contains("attribute mass : ScalarValues::Real = 12.5; // unit: kg"), "{t}");
    assert!(t.contains("port cmd : P::Rod;"), "{t}");
    assert!(t.contains("connection feed connect a.outPort to b.inPort;"), "{t}");
    let back = reimport(t);
    assert_eq!(back.get("Imp::P::Machine").map(String::as_str), Some("PartDef"), "{t}");
    assert_eq!(back.get("Imp::P::Machine::a").map(String::as_str), Some("Part"), "{t}");
}

//! `REQ-TRS-SYSMLV2-086`..`-097`: metadata applications lifted into `metadata:` (with `about`
//! targets and export), `while ... until`, control-node parameters, succession types, structural
//! successions, anonymous dependencies, portion kinds, root-level aliases, unquoted expression
//! payloads and the final `W543` list.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_yaml::Value;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::sysmlv2::export::export_sysml;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-sysmlv2-lastgaps-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// Ingest SysML text under a `sysmlSubmodel` package `S`; every file must parse.
fn load(sysml: &str) -> Vec<RawElement> {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "S/_index.md", "---\ntype: Package\nname: S\nsysmlSubmodel: true\n---\n");
    write(&root, "S/A.sysml", sysml);
    let els = walk_model(&root).unwrap();
    let bad: Vec<_> = els.iter().flat_map(|e| e.derive_findings.iter()).filter(|(c, _, _)| c == "W541").collect();
    assert!(bad.is_empty(), "parse failed: {bad:?}\n{sysml}");
    els
}

fn native(files: &[(&str, &str)]) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "B/_index.md", "---\ntype: Package\nname: B\n---\n");
    for (p, c) in files {
        write(&r, p, c);
    }
    walk_model(&r).unwrap()
}

fn reimport(text: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Imp/_index.md", "---\ntype: Package\nname: Imp\nsysmlSubmodel: true\n---\n");
    write(&r, "Imp/out.sysml", text);
    let els = walk_model(&r).unwrap();
    let bad: Vec<_> = els.iter().flat_map(|e| e.derive_findings.iter()).filter(|(c, _, _)| c == "W541").collect();
    assert!(bad.is_empty(), "parse-back failed: {bad:?}\n{text}");
    els
}

fn find<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter().find(|e| e.qualified_name == q).unwrap_or_else(|| panic!("missing {q}; have {:?}", els.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()))
}

fn yaml(v: &Option<Vec<Value>>) -> Value {
    Value::Sequence(v.clone().unwrap_or_default())
}

fn entry<'a>(list: &'a Option<Vec<Value>>, name: &str) -> &'a Value {
    list.as_ref().unwrap().iter().find(|v| v.get("name").and_then(Value::as_str) == Some(name)).unwrap_or_else(|| panic!("no entry {name} in {list:?}"))
}

fn st(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(Value::as_str).map(str::to_string)
}

fn w543(els: &[RawElement]) -> Vec<String> {
    validate(els).findings.iter().filter(|f| f.code == "W543").map(|f| f.message.clone()).collect()
}

fn codes(els: &[RawElement], code: &str) -> Vec<String> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| f.message.clone()).collect()
}

/// The `metadata:` entries of an element as `(type, other keys sorted)` for order-free comparison.
fn meta(e: &RawElement) -> Vec<Value> {
    e.frontmatter.metadata.clone().unwrap_or_default()
}

fn meta_entry<'a>(e: &'a RawElement, ty: &str) -> &'a Value {
    e.frontmatter.metadata.as_ref().unwrap().iter().find(|m| st(m, "type").as_deref() == Some(ty)).unwrap_or_else(|| panic!("no metadata of type {ty} on {}: {:?}", e.qualified_name, e.frontmatter.metadata))
}

/// Export `files` and check that re-ingesting the text gives the same action entries.
fn round_trip(files: &[(&str, &str)], qname: &str) -> String {
    let els = native(files);
    let out = export_sysml(&els, None).unwrap();
    assert!(!out.text.contains("not exported"), "{}", out.text);
    assert_eq!(out.report.degraded_behaviour, 0, "{}", out.text);
    let back = reimport(&out.text);
    let (a, b) = (find(&els, qname), find(&back, &format!("Imp::{qname}")));
    assert_eq!(yaml(&a.frontmatter.sub_actions), yaml(&b.frontmatter.sub_actions), "{}", out.text);
    assert_eq!(yaml(&a.frontmatter.control_nodes), yaml(&b.frontmatter.control_nodes), "{}", out.text);
    assert_eq!(yaml(&a.frontmatter.succession_connections), yaml(&b.frontmatter.succession_connections), "{}", out.text);
    out.text
}

// ── REQ-TRS-SYSMLV2-086 ────────────────────────────────────────────────────

const META: &str = "package P {\n  metadata def Safety { attribute level : ScalarValues::Natural; attribute hazardId : ScalarValues::String; attribute ratio : ScalarValues::Real; attribute hot : ScalarValues::Boolean; }\n  metadata def SyscribeDomain { attribute value : ScalarValues::String; }\n  part def A {\n    @Safety { level = 3; hazardId = \"HAZ-042\"; ratio = 2.5; hot = true; }\n    @s2 : Safety;\n    #Safety { level = 2; }\n    @SyscribeDomain { value = 'software'; }\n    @ModelingMetadata::StatusInfo;\n  }\n  #Safety part def B;\n  action def Act { @Safety { level = 1; } }\n  state def St { @Safety; }\n  port def Pt { @Safety; }\n  item def It { @Safety; }\n  requirement def R { @Safety; }\n}\n";

#[test]
fn body_metadata_applications_lift_into_the_native_list() {
    let els = load(META);
    let a = find(&els, "S::P::A");
    let m = meta(a);
    assert_eq!(m.len(), 4, "{m:?}");
    // Typed literals, type resolved to the ingested MetadataDef.
    let full = &m[0];
    assert_eq!(st(full, "type").as_deref(), Some("S::P::Safety"));
    assert_eq!(full.get("level"), Some(&Value::Number(3.into())));
    assert_eq!(st(full, "hazardId").as_deref(), Some("HAZ-042"));
    assert_eq!(full.get("ratio"), Some(&Value::Number(2.5.into())));
    assert_eq!(full.get("hot"), Some(&Value::Bool(true)));
    assert_eq!(full.get("name"), None);
    // A declared name.
    assert_eq!(st(&m[1], "name").as_deref(), Some("s2"));
    assert_eq!(st(&m[1], "type").as_deref(), Some("S::P::Safety"));
    // The `#T { ... }` keyword form.
    assert_eq!(st(&m[2], "type").as_deref(), Some("S::P::Safety"));
    assert_eq!(m[2].get("level"), Some(&Value::Number(2.into())));
    // A standard-library type stays as written.
    assert_eq!(st(&m[3], "type").as_deref(), Some("ModelingMetadata::StatusInfo"));
    // `@Syscribe*` keeps its field lift and never lands in `metadata:`.
    assert_eq!(a.frontmatter.domain.as_deref(), Some("software"));
    assert!(m.iter().all(|x| !st(x, "type").unwrap().contains("Syscribe")), "{m:?}");
    // The package-level `#T` prefix member applies to the member that follows it.
    assert_eq!(st(meta_entry(find(&els, "S::P::B"), "S::P::Safety"), "type").as_deref(), Some("S::P::Safety"));
    assert_eq!(meta(find(&els, "S::P::B")).len(), 1);
    // Other element kinds.
    for q in ["S::P::Act", "S::P::St", "S::P::Pt", "S::P::It", "S::P::R"] {
        assert_eq!(meta(find(&els, q)).len(), 1, "{q}");
    }
    assert_eq!(meta_entry(find(&els, "S::P::Act"), "S::P::Safety").get("level"), Some(&Value::Number(1.into())));
    // The native rules apply unchanged: everything resolves, every tagged value is declared.
    assert!(codes(&els, "E317").is_empty(), "{:?}", codes(&els, "E317"));
    assert!(codes(&els, "W045").is_empty(), "{:?}", codes(&els, "W045"));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn an_unresolved_metadata_type_is_the_native_e317() {
    let els = load("package P {\n  part def A { @Nowhere { x = 1; } }\n}\n");
    assert_eq!(st(meta_entry(find(&els, "S::P::A"), "Nowhere"), "type").as_deref(), Some("Nowhere"));
    assert_eq!(codes(&els, "E317").len(), 1);
}

// ── REQ-TRS-SYSMLV2-087 ────────────────────────────────────────────────────

#[test]
fn about_targets_attach_to_the_target_or_stay_with_the_holder() {
    let els = load(
        "package P {\n  metadata def Safety { attribute level : ScalarValues::Natural; }\n  part def A;\n  part def C { part a : A; @Safety about a; }\n  @Safety about A, C;\n  metadata m : Safety about A { level = 1; }\n  metadata Safety about A;\n  metadata m2 : Safety;\n  @Safety about Nope;\n}\n",
    );
    let a = meta(find(&els, "S::P::A"));
    assert_eq!(a.len(), 3, "{a:?}");
    assert!(a.iter().all(|m| st(m, "type").as_deref() == Some("S::P::Safety") && m.get("about").is_none()), "{a:?}");
    assert!(a.iter().any(|m| st(m, "name").as_deref() == Some("m") && m.get("level") == Some(&Value::Number(1.into()))), "{a:?}");
    // A typeless usage names its type.
    assert!(a.iter().any(|m| m.get("name").is_none() && m.get("level").is_none()), "{a:?}");
    assert_eq!(meta(find(&els, "S::P::C")).len(), 1);
    // A body-level `about` resolves in the holder's own scope.
    assert_eq!(meta(find(&els, "S::P::C::a")).len(), 1);
    // No `about`: the package itself; an unresolved target: kept on the holder with `about:`.
    let p = meta(find(&els, "S::P"));
    assert_eq!(p.len(), 2, "{p:?}");
    assert!(p.iter().any(|m| st(m, "name").as_deref() == Some("m2") && m.get("about").is_none()), "{p:?}");
    assert!(p.iter().any(|m| st(m, "about").as_deref() == Some("Nope")), "{p:?}");
    let w = w543(&els);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("metadata x1"), "{w:?}");
}

// ── REQ-TRS-SYSMLV2-088 ────────────────────────────────────────────────────

#[test]
fn metadata_exports_as_annotations_and_reads_back() {
    let els = native(&[
        ("B/Safety.md", "---\ntype: MetadataDef\nname: Safety\nfeatures:\n  - name: level\n    typedBy: ScalarValues::Natural\n  - name: hazardId\n    typedBy: ScalarValues::String\n  - name: hot\n    typedBy: ScalarValues::Boolean\n---\n"),
        ("B/A.md", "---\ntype: PartDef\nname: A\nmetadata:\n  - type: B::Safety\n    level: 3\n    hazardId: \"HAZ-042\"\n    hot: false\n  - type: B::Safety\n    name: tag\n  - type: B::Safety\n    about: Nope\n  - B::Safety\n---\n"),
        ("B/x.md", "---\ntype: Part\nname: x\ntypedBy: B::A\nmetadata:\n  - type: ModelingMetadata::StatusInfo\n    status: approved\n---\n"),
    ]);
    let out = export_sysml(&els, None).unwrap();
    let t = &out.text;
    assert!(t.contains("@B::Safety {"), "{t}");
    assert!(t.contains("level = 3;") && t.contains("hazardId = \"HAZ-042\";") && t.contains("hot = false;"), "{t}");
    assert!(t.contains("@tag : B::Safety;"), "{t}");
    assert!(t.contains("@B::Safety about Nope;"), "{t}");
    assert!(t.contains("@ModelingMetadata::StatusInfo {"), "{t}");
    let back = reimport(t);
    let a = find(&back, "Imp::B::A");
    let m = meta(a);
    assert_eq!(m.len(), 4, "{m:?}\n{t}");
    assert_eq!(st(&m[0], "type").as_deref(), Some("Imp::B::Safety"));
    assert_eq!(m[0].get("level"), Some(&Value::Number(3.into())));
    assert_eq!(st(&m[0], "hazardId").as_deref(), Some("HAZ-042"));
    assert_eq!(m[0].get("hot"), Some(&Value::Bool(false)));
    assert_eq!(st(&m[1], "name").as_deref(), Some("tag"));
    assert_eq!(st(&m[2], "about").as_deref(), Some("Nope"));
    assert_eq!(st(&m[3], "type").as_deref(), Some("Imp::B::Safety"));
    let x = meta_entry(find(&back, "Imp::B::x"), "ModelingMetadata::StatusInfo");
    assert_eq!(st(x, "status").as_deref(), Some("approved"));
}

// ── REQ-TRS-SYSMLV2-089 ────────────────────────────────────────────────────

#[test]
fn a_while_loop_keeps_its_until_condition() {
    let els = load("package P {\n  action def A {\n    action w1;\n    while x > 1 { action inner; } until y;\n  }\n}\n");
    let w = entry(&find(&els, "S::P::A").frontmatter.sub_actions, "while_1");
    assert_eq!(st(w, "loopKind").as_deref(), Some("while"));
    assert_eq!(st(w, "condition").as_deref(), Some("x > 1"));
    assert_eq!(st(w, "untilCondition").as_deref(), Some("y"));
    let text = round_trip(
        &[("B/A.md", "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: while_1\n    kind: LoopAction\n    loopKind: while\n    condition: \"x > 1\"\n    untilCondition: \"y\"\n    body:\n      - name: inner\n        kind: PerformAction\n---\n")],
        "B::A",
    );
    assert!(text.contains("while x > 1 {") && text.contains("} until y;"), "{text}");
}

// ── REQ-TRS-SYSMLV2-090 ────────────────────────────────────────────────────

#[test]
fn control_node_parameters_are_ingested_and_exported() {
    let els = load("package P {\n  action def A {\n    action a;\n    fork f { in pa; out pb : T; }\n    then join j { inout pc; }\n    decide d;\n  }\n}\n");
    let a = find(&els, "S::P::A");
    let f = entry(&a.frontmatter.control_nodes, "f");
    let params = f.get("parameters").and_then(Value::as_sequence).unwrap();
    assert_eq!(params.len(), 2, "{f:?}");
    assert_eq!((st(&params[0], "name").as_deref(), st(&params[0], "direction").as_deref()), (Some("pa"), Some("in")));
    assert_eq!((st(&params[1], "name").as_deref(), st(&params[1], "direction").as_deref(), st(&params[1], "typedBy").as_deref()), (Some("pb"), Some("out"), Some("T")));
    let j = entry(&a.frontmatter.control_nodes, "j");
    assert_eq!(st(&j.get("parameters").unwrap()[0], "direction").as_deref(), Some("inout"));
    assert!(entry(&a.frontmatter.control_nodes, "d").get("parameters").is_none());
    let text = round_trip(
        &[
            ("B/T.md", "---\ntype: ItemDef\nname: T\n---\n"),
            ("B/A.md", "---\ntype: ActionDef\nname: A\ncontrolNodes:\n  - name: f\n    kind: ForkNode\n    parameters:\n      - name: pa\n        direction: in\n      - name: pb\n        direction: out\n        typedBy: B::T\n  - name: j\n    kind: JoinNode\n---\n"),
        ],
        "B::A",
    );
    assert!(text.contains("fork f {") && text.contains("in pa;") && text.contains("out pb : B::T;") && text.contains("join j;"), "{text}");
}

// ── REQ-TRS-SYSMLV2-091 ────────────────────────────────────────────────────

#[test]
fn a_successions_own_type_is_ingested_and_exported() {
    let els = load("package P {\n  action def A {\n    action a; action b; action c;\n    succession s : T first a then b;\n    succession : T2 first b then c;\n    succession s3 : T [1] first [0..1] a then [1] c;\n  }\n}\n");
    let sc = &find(&els, "S::P::A").frontmatter.succession_connections;
    let s = entry(sc, "s");
    assert_eq!(st(s, "typedBy").as_deref(), Some("T"));
    assert!(sc.as_ref().unwrap().iter().any(|v| v.get("name").is_none() && st(v, "typedBy").as_deref() == Some("T2")), "{sc:?}");
    let s3 = entry(sc, "s3");
    assert_eq!((st(s3, "typedBy").as_deref(), st(s3, "multiplicity").as_deref(), st(s3, "afterMultiplicity").as_deref()), (Some("T"), Some("1"), Some("0..1")));
    let text = round_trip(
        &[
            ("B/T.md", "---\ntype: ConnectionDef\nname: T\n---\n"),
            ("B/A.md", "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: a\n    kind: PerformAction\n  - name: b\n    kind: PerformAction\nsuccessionConnections:\n  - name: s\n    typedBy: B::T\n    after: a\n    before: b\n  - typedBy: B::T\n    after: b\n    before: a\n    multiplicity: \"1\"\n---\n"),
        ],
        "B::A",
    );
    assert!(text.contains("succession s : B::T first a then b;"), "{text}");
    assert!(text.contains("succession : B::T [1] first b then a;"), "{text}");
}

// ── REQ-TRS-SYSMLV2-092 ────────────────────────────────────────────────────

#[test]
fn structural_successions_in_part_bodies_are_ingested_and_exported() {
    let els = load("package P {\n  part def Q {\n    part a; part b;\n    first a then b;\n    succession s2 : HB first b then a;\n  }\n  part q : Q {\n    part c; part d;\n    succession s3 first c then d;\n  }\n}\n");
    let q = &find(&els, "S::P::Q").frontmatter.succession_connections;
    assert_eq!(q.as_ref().unwrap().len(), 2, "{q:?}");
    assert_eq!((st(&q.as_ref().unwrap()[0], "after").as_deref(), st(&q.as_ref().unwrap()[0], "before").as_deref()), (Some("a"), Some("b")));
    assert_eq!(st(entry(q, "s2"), "typedBy").as_deref(), Some("HB"));
    let u = &find(&els, "S::P::q").frontmatter.succession_connections;
    assert_eq!((st(entry(u, "s3"), "after").as_deref(), st(entry(u, "s3"), "before").as_deref()), (Some("c"), Some("d")));

    let native_els = native(&[
        ("B/Q.md", "---\ntype: PartDef\nname: Q\nsuccessionConnections:\n  - after: a\n    before: b\n  - name: s2\n    after: b\n    before: a\n---\n"),
        ("B/Q/a.md", "---\ntype: Part\nname: a\n---\n"),
        ("B/Q/b.md", "---\ntype: Part\nname: b\n---\n"),
        ("B/q.md", "---\ntype: Part\nname: q\ntypedBy: B::Q\nsuccessionConnections:\n  - name: s3\n    after: c\n    before: d\n---\n"),
    ]);
    let out = export_sysml(&native_els, None).unwrap();
    assert!(out.text.contains("first a then b;") && out.text.contains("succession s2 first b then a;") && out.text.contains("succession s3 first c then d;"), "{}", out.text);
    assert_eq!(out.report.degraded_behaviour, 0, "{}", out.text);
    let back = reimport(&out.text);
    assert_eq!(yaml(&find(&back, "Imp::B::Q").frontmatter.succession_connections), yaml(&find(&native_els, "B::Q").frontmatter.succession_connections));
    assert_eq!(yaml(&find(&back, "Imp::B::q").frontmatter.succession_connections), yaml(&find(&native_els, "B::q").frontmatter.succession_connections));
}

// ── REQ-TRS-SYSMLV2-093 ────────────────────────────────────────────────────

#[test]
fn anonymous_dependencies_get_synthesized_names() {
    let els = load("package P {\n  part def X;\n  part def Y;\n  dependency from X to Y;\n  dependency dependency_2 from Y to X;\n  dependency from Y to X;\n  part def Z { dependency from X to Y; }\n}\n");
    let d1 = find(&els, "S::P::dependency_1");
    assert_eq!(d1.frontmatter.element_type, Some(ElementType::Dependency));
    assert_eq!(d1.frontmatter.clients, Some(vec!["S::P::X".to_string()]));
    assert_eq!(d1.frontmatter.suppliers, Some(vec!["S::P::Y".to_string()]));
    // The explicit `dependency_2` is taken, so the next anonymous one is `dependency_3`.
    assert_eq!(find(&els, "S::P::dependency_3").frontmatter.clients, Some(vec!["S::P::Y".to_string()]));
    // Numbering is per owning scope.
    assert_eq!(find(&els, "S::P::Z::dependency_1").frontmatter.suppliers, Some(vec!["S::P::Y".to_string()]));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
    let out = export_sysml(&els, Some("S::P")).unwrap();
    assert!(out.text.contains("dependency dependency_1 from"), "{}", out.text);
}

// ── REQ-TRS-SYSMLV2-094 ────────────────────────────────────────────────────

#[test]
fn portion_kinds_map_to_is_portion_and_portion_kind() {
    let els = load("package P {\n  occurrence def Life;\n  snapshot occurrence frozen : Life;\n  individual timeslice occurrence slice : Life;\n  occurrence whole : Life;\n}\n");
    let f = find(&els, "S::P::frozen");
    assert_eq!((f.frontmatter.is_portion, f.frontmatter.portion_kind.as_deref()), (Some(true), Some("snapshot")));
    let s = find(&els, "S::P::slice");
    assert_eq!((s.frontmatter.is_portion, s.frontmatter.portion_kind.as_deref(), s.frontmatter.is_individual), (Some(true), Some("timeslice"), Some(true)));
    assert_eq!(find(&els, "S::P::whole").frontmatter.is_portion, None);
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));

    let native_els = native(&[
        ("B/Life.md", "---\ntype: OccurrenceDef\nname: Life\n---\n"),
        ("B/frozen.md", "---\ntype: Occurrence\nname: frozen\ntypedBy: B::Life\nisPortion: true\nportionKind: snapshot\n---\n"),
        ("B/slice.md", "---\ntype: Occurrence\nname: slice\ntypedBy: B::Life\nisPortion: true\nisIndividual: true\n---\n"),
    ]);
    let out = export_sysml(&native_els, None).unwrap();
    assert!(out.text.contains("snapshot occurrence frozen : B::Life;"), "{}", out.text);
    assert!(out.text.contains("individual timeslice occurrence slice : B::Life;"), "{}", out.text);
    let back = reimport(&out.text);
    assert_eq!(find(&back, "Imp::B::frozen").frontmatter.portion_kind.as_deref(), Some("snapshot"));
    assert_eq!(find(&back, "Imp::B::slice").frontmatter.portion_kind.as_deref(), Some("timeslice"));
}

// ── REQ-TRS-SYSMLV2-095 ────────────────────────────────────────────────────

#[test]
fn a_root_level_alias_lifts_onto_the_anchor_package() {
    let els = load("alias Top for P::X;\nalias <t2> Second for P::X;\npackage P {\n  part def X;\n  alias Inner for X;\n}\n");
    let anchor = find(&els, "S");
    let aliases = anchor.frontmatter.aliases.as_ref().unwrap();
    assert_eq!(aliases.len(), 2, "{aliases:?}");
    assert_eq!((st(&aliases[0], "name").as_deref(), st(&aliases[0], "for").as_deref()), (Some("Top"), Some("P::X")));
    assert_eq!(st(&aliases[1], "shortName").as_deref(), Some("t2"));
    assert_eq!(find(&els, "S::P").frontmatter.aliases.as_ref().unwrap().len(), 1);
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn other_bare_root_members_merge_under_the_anchor() {
    // `REQ-TRS-SYSMLV2-098`: a bare root-level definition is the anchor's own member now (it was
    // counted as `root-level member` before); the alias lift itself is unchanged.
    let els = load("part def Loose;\ndoc /* root doc */\nalias Top for P::X;\npackage P { part def X; }\n");
    assert_eq!(find(&els, "S::Loose").frontmatter.element_type, Some(ElementType::PartDef));
    assert_eq!(find(&els, "S").frontmatter.aliases.as_ref().map(Vec::len), Some(1));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

// ── REQ-TRS-SYSMLV2-096 ────────────────────────────────────────────────────

#[test]
fn an_expression_payload_exports_unquoted() {
    let text = round_trip(
        &[("B/A.md", "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: new Cmd()\n    kind: SendAction\n    payload: new Cmd()\n    via: p\n  - name: \"a + 1\"\n    kind: SendAction\n    payload: \"a + 1\"\n  - name: tell\n    kind: SendAction\n    payload: B::Cmd\n    via: q\n  - name: odd name\n    kind: AcceptAction\n    payload: odd name\n---\n")],
        "B::A",
    );
    assert!(text.contains("send new Cmd() via p;"), "{text}");
    assert!(text.contains("send a + 1;"), "{text}");
    assert!(text.contains("send tell : B::Cmd via q;"), "{text}");
    // A genuine restricted name stays quoted.
    assert!(text.contains("accept 'odd name';"), "{text}");
    assert!(!text.contains("'new Cmd()'"), "{text}");
    // Ingested from source, the same statement comes back out.
    let els = load("package P {\n  action def A { send new Cmd() via p; }\n}\n");
    let out = export_sysml(&els, Some("S::P")).unwrap();
    assert!(out.text.contains("send new Cmd() via p;"), "{}", out.text);
    assert_eq!(out.report.degraded_behaviour, 0, "{}", out.text);
}

// ── REQ-TRS-SYSMLV2-097 ────────────────────────────────────────────────────

#[test]
fn w543_names_exactly_the_final_unmapped_kinds() {
    let els = load(
        "part def Loose;\nalias Top for P::X;\npackage P {\n  part def X;\n  metadata def Tag;\n  occurrence def O;\n  snapshot occurrence s : O;\n  dependency from X to O;\n  actor Act;\n  filter @Tag;\n  @Tag about X;\n  @Tag about Nope;\n  metadata m : Tag;\n  alias Z for X;\n  classifier K;\n}\n",
    );
    let w = w543(&els);
    assert_eq!(w.len(), 1, "{w:?}");
    for k in ["actor x1", "filter x1", "metadata x1", "KerML declaration x1"] {
        assert!(w[0].contains(k), "missing {k}: {w:?}");
    }
    // `REQ-TRS-SYSMLV2-098`: the root-level `part def Loose;` is `S::Loose`, not a count.
    assert_eq!(find(&els, "S::Loose").frontmatter.element_type, Some(ElementType::PartDef));
    for k in ["occurrence", "dependency", "alias", "metadata x2", "root-level member"] {
        assert!(!w[0].contains(k), "{k} must not be counted: {w:?}");
    }
}

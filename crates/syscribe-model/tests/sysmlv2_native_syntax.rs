//! `REQ-TRS-SYSMLV2-077`..`-085`: native 0.57 syntax for step fields (`via`/`to`, triggers,
//! `assign` referents, `until` loops), nested kinds in part usages, succession names and
//! multiplicities, `then` control forms, occurrences, dependencies and the accurate `W543` list.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_yaml::Value;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::sysmlv2::export::export_sysml;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-sysmlv2-native-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
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

// ── REQ-TRS-SYSMLV2-077 ────────────────────────────────────────────────────

#[test]
fn accept_via_and_send_via_to_are_ingested() {
    let els = load(
        "package P {\n  action def A {\n    accept listen : Fix via gpsIn;\n    accept Short via port1;\n    send tell : Cmd via out1 to tgt;\n    send ping : Cmd to tgt2;\n    send new Cmd() via out2;\n  }\n}\n",
    );
    let a = &find(&els, "S::P::A").frontmatter.sub_actions;
    let listen = entry(a, "listen");
    assert_eq!(st(listen, "kind").as_deref(), Some("AcceptAction"));
    assert_eq!(st(listen, "via").as_deref(), Some("gpsIn"));
    assert_eq!(st(entry(a, "Short"), "via").as_deref(), Some("port1"));
    let tell = entry(a, "tell");
    assert_eq!((st(tell, "via").as_deref(), st(tell, "to").as_deref()), (Some("out1"), Some("tgt")));
    let ping = entry(a, "ping");
    assert_eq!((st(ping, "via"), st(ping, "to").as_deref()), (None, Some("tgt2")));
    assert!(a.as_ref().unwrap().iter().any(|v| st(v, "kind").as_deref() == Some("SendAction") && st(v, "via").as_deref() == Some("out2")));
}

#[test]
fn via_and_to_export_as_native_statements_and_read_back() {
    let text = round_trip(
        &[(
            "B/A.md",
            "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: tell\n    kind: SendAction\n    payload: B::Cmd\n    via: controlOut\n    to: ground\n  - name: listen\n    kind: AcceptAction\n    payload: B::Fix\n    via: gpsIn\n---\n",
        )],
        "B::A",
    );
    assert!(text.contains("send tell : B::Cmd via controlOut to ground;"), "{text}");
    assert!(text.contains("accept listen : B::Fix via gpsIn;"), "{text}");
    assert!(!text.contains("@SyscribeStep"), "{text}");
}

#[test]
fn the_deprecated_step_annotation_is_still_ingested() {
    let els = load(
        "package P {\n  metadata def SyscribeStep { attribute via : String; }\n  action def A {\n    send tell : Cmd { @SyscribeStep { via = 'controlOut'; } }\n  }\n}\n",
    );
    assert_eq!(st(entry(&find(&els, "S::P::A").frontmatter.sub_actions, "tell"), "via").as_deref(), Some("controlOut"));
}

// ── REQ-TRS-SYSMLV2-078 ────────────────────────────────────────────────────

#[test]
fn time_and_change_triggers_are_ingested() {
    let els = load(
        "package P {\n  action def A {\n    accept after 5;\n    accept when d < 2;\n    accept at t0;\n    action waitHere accept when ready;\n  }\n}\n",
    );
    let a = &find(&els, "S::P::A").frontmatter.sub_actions;
    let trig = |n: &str| entry(a, n).get("trigger").cloned().unwrap_or_else(|| panic!("no trigger on {n}"));
    assert_eq!(trig("accept_1"), serde_yaml::from_str::<Value>("{kind: timeOut, when: '5'}").unwrap());
    assert_eq!(trig("accept_2"), serde_yaml::from_str::<Value>("{kind: change, condition: 'd < 2'}").unwrap());
    assert_eq!(trig("accept_3"), serde_yaml::from_str::<Value>("{kind: at, when: t0}").unwrap());
    assert_eq!(st(entry(a, "waitHere"), "kind").as_deref(), Some("AcceptAction"));
    assert!(entry(a, "waitHere").get("payload").is_none());
}

#[test]
fn triggers_export_as_native_statements_and_read_back() {
    let text = round_trip(
        &[(
            "B/A.md",
            "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: accept_1\n    kind: AcceptAction\n    trigger:\n      kind: timeOut\n      when: \"5\"\n  - name: ready\n    kind: AcceptAction\n    trigger:\n      kind: change\n      condition: \"d < 2\"\n  - name: arrival\n    kind: AcceptAction\n    trigger:\n      kind: at\n      when: t0\n---\n",
        )],
        "B::A",
    );
    assert!(text.contains("accept after 5;"), "{text}");
    assert!(text.contains("action ready accept when d < 2;"), "{text}");
    assert!(text.contains("action arrival accept at t0;"), "{text}");
    assert!(!text.contains("@SyscribeStep"), "{text}");
}

#[test]
fn a_trigger_beside_a_payload_keeps_the_deprecated_annotation() {
    let text = round_trip(
        &[(
            "B/A.md",
            "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: listen\n    kind: AcceptAction\n    payload: B::Fix\n    trigger:\n      kind: change\n      condition: \"d < 2.0\"\n---\n",
        )],
        "B::A",
    );
    assert!(text.contains("@SyscribeStep") && text.contains("triggerKind = 'change';"), "{text}");
}

#[test]
fn a_trigger_kind_with_no_syntax_and_no_payload_still_degrades() {
    let els = native(&[(
        "B/A.md",
        "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: m\n    kind: AcceptAction\n    trigger:\n      kind: message\n      payload: B::Cmd\n---\n",
    )]);
    let out = export_sysml(&els, None).unwrap();
    assert_eq!(out.report.degraded_behaviour, 1, "{}", out.text);
}

// ── REQ-TRS-SYSMLV2-079 ────────────────────────────────────────────────────

#[test]
fn assign_referent_and_until_loops_are_ingested() {
    let els = load(
        "package P {\n  action def A {\n    assign self.throttle := 0.6;\n    assign plain := 1;\n    loop { action poll; } until self.alt <= 0.1;\n  }\n}\n",
    );
    let a = &find(&els, "S::P::A").frontmatter.sub_actions;
    let asg = entry(a, "assign_1");
    assert_eq!((st(asg, "target").as_deref(), st(asg, "referent").as_deref()), (Some("self"), Some("throttle")));
    assert_eq!(st(entry(a, "assign_2"), "referent"), None);
    let lp = entry(a, "loop_1");
    assert_eq!((st(lp, "loopKind").as_deref(), st(lp, "condition").as_deref()), (Some("until"), Some("self.alt <= 0.1")));
}

#[test]
fn referent_and_until_export_natively_and_a_dotted_target_is_the_same_value() {
    let text = round_trip(
        &[(
            "B/A.md",
            "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: assign_1\n    kind: AssignmentAction\n    target: self\n    referent: throttle\n    value: \"0.6\"\n  - name: wait\n    kind: LoopAction\n    loopKind: until\n    condition: \"self.alt <= 0.1\"\n    body:\n      - name: poll\n        kind: PerformAction\n---\n",
        )],
        "B::A",
    );
    assert!(text.contains("assign self.throttle := 0.6;"), "{text}");
    assert!(text.contains("until self.alt <= 0.1;"), "{text}");
    assert!(!text.contains("@SyscribeStep"), "{text}");

    // `target: ctrl.mode` with no `referent` is the same statement as `target: ctrl, referent: mode`.
    let els = native(&[(
        "B/D.md",
        "---\ntype: ActionDef\nname: D\nsubActions:\n  - name: assign_1\n    kind: AssignmentAction\n    target: ctrl.mode\n    value: \"1\"\n---\n",
    )]);
    let out = export_sysml(&els, None).unwrap();
    assert_eq!(out.report.degraded_behaviour, 0, "{}", out.text);
    assert!(out.text.contains("assign ctrl.mode := 1;"), "{}", out.text);
    let back = reimport(&out.text);
    let e = entry(&find(&back, "Imp::B::D").frontmatter.sub_actions, "assign_1").clone();
    assert_eq!((st(&e, "target").as_deref(), st(&e, "referent").as_deref()), (Some("ctrl"), Some("mode")));
}

#[test]
fn value_kind_is_the_one_assign_field_that_still_needs_the_annotation() {
    let text = round_trip(
        &[(
            "B/A.md",
            "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: setIt\n    kind: AssignmentAction\n    target: self\n    referent: throttle\n    value: \"0.6\"\n    valueKind: initial\n---\n",
        )],
        "B::A",
    );
    assert!(text.contains("valueKind = 'initial';") && !text.contains("feature = "), "{text}");
}

// ── REQ-TRS-SYSMLV2-080 ────────────────────────────────────────────────────

#[test]
fn views_and_other_kinds_nested_in_a_part_usage_are_native_elements() {
    let els = load(
        "package P {\n  part def Eng;\n  part e : Eng {\n    view v;\n    viewpoint vp;\n    view def VD;\n    viewpoint def VPD;\n    rendering r;\n    constraint def Lim { in x : Real; x > 0 }\n    metadata def Tag;\n  }\n}\n",
    );
    for (q, t) in [
        ("S::P::e::v", ElementType::View),
        ("S::P::e::vp", ElementType::View),
        ("S::P::e::VD", ElementType::ViewDef),
        ("S::P::e::VPD", ElementType::ViewpointDef),
        ("S::P::e::Lim", ElementType::ConstraintDef),
        ("S::P::e::Tag", ElementType::MetadataDef),
    ] {
        assert_eq!(find(&els, q).frontmatter.element_type, Some(t), "{q}");
    }
}

// ── REQ-TRS-SYSMLV2-081 ────────────────────────────────────────────────────

#[test]
fn succession_names_and_multiplicities_are_ingested_and_exported() {
    let els = load(
        "package P {\n  action def A {\n    action a;\n    action b;\n    action c;\n    succession s1 first a then b;\n    succession s2 [1] first [0..1] b then [1..*] c;\n    succession s3 first a if ok then c;\n  }\n}\n",
    );
    let sc = &find(&els, "S::P::A").frontmatter.succession_connections;
    let s1 = sc.as_ref().unwrap().iter().find(|v| st(v, "name").as_deref() == Some("s1")).expect("s1");
    assert_eq!((st(s1, "after").as_deref(), st(s1, "before").as_deref()), (Some("a"), Some("b")));
    let s2 = sc.as_ref().unwrap().iter().find(|v| st(v, "name").as_deref() == Some("s2")).expect("s2");
    assert_eq!(
        (st(s2, "multiplicity").as_deref(), st(s2, "afterMultiplicity").as_deref(), st(s2, "beforeMultiplicity").as_deref()),
        (Some("1"), Some("0..1"), Some("1..*"))
    );
    let s3 = sc.as_ref().unwrap().iter().find(|v| st(v, "name").as_deref() == Some("s3")).expect("s3");
    assert_eq!(st(s3, "guard").as_deref(), Some("ok"));
}

#[test]
fn named_and_multiplicity_successions_read_back_identically() {
    let text = round_trip(
        &[(
            "B/A.md",
            "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: a\n    kind: PerformAction\n  - name: b\n    kind: PerformAction\nsuccessionConnections:\n  - name: s1\n    after: a\n    before: b\n  - name: s2\n    after: b\n    before: a\n    multiplicity: \"1\"\n    afterMultiplicity: \"0..1\"\n    beforeMultiplicity: \"1..*\"\n  - name: s3\n    after: a\n    before: b\n    guard: ok\n---\n",
        )],
        "B::A",
    );
    assert!(text.contains("succession s2 [1] first [0..1] b then [1..*] a;"), "{text}");
}

// ── REQ-TRS-SYSMLV2-082 ────────────────────────────────────────────────────

#[test]
fn then_control_forms_produce_nodes_and_edges() {
    let els = load(
        "package P {\n  action def A {\n    action a;\n    then fork f;\n    then decide d;\n    then join j;\n    then accept Foo;\n    then send new S() to b;\n    then if x > 1 { action w; }\n  }\n}\n",
    );
    let a = find(&els, "S::P::A");
    let nodes: Vec<_> = a.frontmatter.control_nodes.as_ref().unwrap().iter().map(|v| (st(v, "name").unwrap(), st(v, "kind").unwrap())).collect();
    assert_eq!(nodes, vec![("f".into(), "ForkNode".into()), ("d".into(), "DecisionNode".into()), ("j".into(), "JoinNode".into())]);
    let edges: Vec<_> = a.frontmatter.succession_connections.as_ref().unwrap().iter().map(|v| (st(v, "after").unwrap(), st(v, "before").unwrap())).collect();
    assert_eq!(edges.len(), 6, "{edges:?}");
    assert_eq!(edges[0], ("a".to_string(), "f".to_string()));
    assert_eq!(edges[1], ("f".to_string(), "d".to_string()));
    assert_eq!(edges[2], ("d".to_string(), "j".to_string()));
    assert_eq!(edges[3].0, "j");
    let kinds: Vec<_> = a.frontmatter.sub_actions.as_ref().unwrap().iter().filter_map(|v| st(v, "kind")).collect();
    assert!(kinds.contains(&"AcceptAction".into()) && kinds.contains(&"SendAction".into()) && kinds.contains(&"IfAction".into()), "{kinds:?}");
}

// ── REQ-TRS-SYSMLV2-083 ────────────────────────────────────────────────────

const OCC: &str = "package P {\n  part def Eng;\n  occurrence def Life :> Eng;\n  individual def Unit1 :> Eng;\n  occurrence phase : Life [0..1] {\n    doc /* The running phase. */\n  }\n  individual occurrence special : Life;\n  event occurrence boom : Life;\n  snapshot occurrence frozen : Life;\n}\n";

#[test]
fn occurrences_and_individuals_are_native_elements() {
    let els = load(OCC);
    assert_eq!(find(&els, "S::P::Life").frontmatter.element_type, Some(ElementType::OccurrenceDef));
    assert_eq!(find(&els, "S::P::Unit1").frontmatter.element_type, Some(ElementType::IndividualDef));
    let phase = find(&els, "S::P::phase");
    assert_eq!(phase.frontmatter.element_type, Some(ElementType::Occurrence));
    assert_eq!(phase.frontmatter.multiplicity.as_deref(), Some("0..1"));
    assert!(phase.doc.contains("running phase"));
    assert_eq!(find(&els, "S::P::special").frontmatter.is_individual, Some(true));
    assert_eq!(find(&els, "S::P::boom").frontmatter.element_type, Some(ElementType::EventOccurrence));
    // A portion kind is the native `isPortion`/`portionKind` since `REQ-TRS-SYSMLV2-094`.
    assert_eq!(find(&els, "S::P::frozen").frontmatter.portion_kind.as_deref(), Some("snapshot"));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn native_occurrence_types_export_and_read_back() {
    let els = native(&[
        ("B/Life.md", "---\ntype: OccurrenceDef\nname: Life\nsupertype: B::Base\n---\n"),
        ("B/Base.md", "---\ntype: OccurrenceDef\nname: Base\n---\n"),
        ("B/Unit.md", "---\ntype: IndividualDef\nname: Unit\n---\nSpecific unit.\n"),
        ("B/phase.md", "---\ntype: Occurrence\nname: phase\ntypedBy: B::Life\nmultiplicity: \"0..1\"\n---\n"),
        ("B/special.md", "---\ntype: Occurrence\nname: special\ntypedBy: B::Life\nisIndividual: true\n---\n"),
        ("B/boom.md", "---\ntype: EventOccurrence\nname: boom\ntypedBy: B::Life\n---\n"),
    ]);
    let out = export_sysml(&els, None).unwrap();
    assert!(out.text.contains("occurrence def Life :> B::Base;"), "{}", out.text);
    assert!(out.text.contains("individual def Unit"), "{}", out.text);
    assert!(out.text.contains("individual occurrence special : B::Life;"), "{}", out.text);
    let back = reimport(&out.text);
    for (q, t) in [("Life", ElementType::OccurrenceDef), ("Unit", ElementType::IndividualDef), ("phase", ElementType::Occurrence), ("special", ElementType::Occurrence), ("boom", ElementType::EventOccurrence)] {
        assert_eq!(find(&back, &format!("Imp::B::{q}")).frontmatter.element_type, Some(t), "{q}\n{}", out.text);
    }
    assert_eq!(find(&back, "Imp::B::phase").frontmatter.multiplicity.as_deref(), Some("0..1"));
    assert_eq!(find(&back, "Imp::B::special").frontmatter.is_individual, Some(true));
    assert!(find(&back, "Imp::B::Unit").doc.contains("Specific unit"));
}

// ── REQ-TRS-SYSMLV2-084 ────────────────────────────────────────────────────

#[test]
fn a_named_dependency_is_ingested_with_resolved_ends() {
    let els = load("package P {\n  part def X;\n  part def Y;\n  part def Z;\n  dependency d from X, Y to Z;\n  dependency from X to Y;\n}\n");
    let d = find(&els, "S::P::d");
    assert_eq!(d.frontmatter.element_type, Some(ElementType::Dependency));
    assert_eq!(d.frontmatter.clients, Some(vec!["S::P::X".to_string(), "S::P::Y".to_string()]));
    assert_eq!(d.frontmatter.suppliers, Some(vec!["S::P::Z".to_string()]));
    // An anonymous one is `dependency_N` since `REQ-TRS-SYSMLV2-093`.
    assert_eq!(find(&els, "S::P::dependency_1").frontmatter.clients, Some(vec!["S::P::X".to_string()]));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn a_native_dependency_exports_and_reads_back() {
    let els = native(&[
        ("B/X.md", "---\ntype: PartDef\nname: X\n---\n"),
        ("B/Y.md", "---\ntype: PartDef\nname: Y\n---\n"),
        ("B/d.md", "---\ntype: Dependency\nname: d\nclients: [B::X]\nsuppliers: [B::Y]\n---\nWhy.\n"),
    ]);
    let out = export_sysml(&els, None).unwrap();
    assert!(out.text.contains("dependency d from B::X to B::Y {"), "{}", out.text);
    let back = reimport(&out.text);
    let d = find(&back, "Imp::B::d");
    assert_eq!(d.frontmatter.clients, Some(vec!["Imp::B::X".to_string()]));
    assert_eq!(d.frontmatter.suppliers, Some(vec!["Imp::B::Y".to_string()]));
    assert!(d.doc.contains("Why."));
}

// ── REQ-TRS-SYSMLV2-085 ────────────────────────────────────────────────────

#[test]
fn w543_names_exactly_the_remaining_unmapped_kinds() {
    // `REQ-TRS-SYSMLV2-087`/`-097`: a metadata usage whose `about` resolves is lifted; only an
    // unresolved target still counts.
    let els = load(
        "package P {\n  part def X;\n  occurrence def O;\n  individual def I;\n  dependency d from X to O;\n  actor Act;\n  filter @Tag;\n  metadata m about X;\n  metadata m2 about Nope;\n  alias Z for X;\n}\n",
    );
    let w = w543(&els);
    assert_eq!(w.len(), 1, "{w:?}");
    for k in ["actor x1", "filter x1", "metadata x1"] {
        assert!(w[0].contains(k), "missing {k}: {w:?}");
    }
    for k in ["occurrence", "individual", "dependency", "alias"] {
        assert!(!w[0].contains(k), "{k} must not be counted: {w:?}");
    }
}

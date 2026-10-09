//! GH #203 -- SysML v2 ingestion no longer silently drops body constructs: anonymous `connect`,
//! `bind`, `perform`, `exhibit state`, `interface ... connect`, `interface def` `end` features,
//! port directions, `~` conjugation and `ref`; one unparsable member no longer discards the whole
//! file; anything still unmapped is counted in `W543`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::RawElement;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-ingest-gaps-{}-{}",
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

fn ingest(src: &str) -> Vec<RawElement> {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Sys/_index.md", "---\ntype: Package\nname: Sys\nsysmlSubmodel: true\n---\n");
    write(&root, "Sys/m.sysml", src);
    walk_model(&root).unwrap()
}

fn get<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter().find(|e| e.qualified_name == q).unwrap_or_else(|| panic!("no element {q}"))
}

fn yaml(e: &RawElement) -> serde_yaml::Value {
    serde_yaml::to_value(&e.frontmatter).unwrap()
}

fn field(e: &RawElement, k: &str) -> serde_yaml::Value {
    yaml(e).get(k).cloned().unwrap_or(serde_yaml::Value::Null)
}

fn findings(els: &[RawElement], code: &str) -> Vec<String> {
    els.iter()
        .flat_map(|e| e.derive_findings.iter())
        .filter(|f| f.0 == code)
        .map(|f| f.2.clone())
        .collect()
}

const PROBE: &str = "package P {\n\
    port def PP { in item x : Real; out item y : Real; }\n\
    part def Eng { port pwr : PP; port inp : ~PP; ref part helper : Eng; }\n\
    part def Car {\n\
      part eng : Eng; part wheel : Eng;\n\
      connect eng.pwr to wheel.inp;\n\
      bind eng.pwr = wheel.pwr;\n\
      perform action drive;\n\
      exhibit state running;\n\
      action drive2;\n\
    }\n\
    interface def PI { end a : PP; end b : ~PP; }\n\
    part def Sys2 { part e1 : Eng; part e2 : Eng; interface i1 : PI connect a ::> e1.pwr to b ::> e2.inp; }\n\
}\n";

#[test]
fn anonymous_connect_lifts_onto_the_owner_connections() {
    let els = ingest(PROBE);
    let car = get(&els, "Sys::P::Car");
    let conns = field(car, "connections");
    let conns = conns.as_sequence().expect("connections");
    assert_eq!(conns.len(), 1);
    // GH #206: the dotted endpoint keeps its full path rather than collapsing to the head part.
    assert_eq!(conns[0]["from"].as_str(), Some("Sys::P::Car::eng::pwr"));
    assert_eq!(conns[0]["to"].as_str(), Some("Sys::P::Car::wheel::inp"));
    assert!(findings(&els, "W542").is_empty());
}

#[test]
fn bind_lifts_onto_binding_connections() {
    let els = ingest(PROBE);
    let b = field(get(&els, "Sys::P::Car"), "bindingConnections");
    let b = b.as_sequence().expect("bindingConnections");
    assert_eq!(b.len(), 1);
    assert_eq!(b[0]["left"].as_str(), Some("Sys::P::Car::eng::pwr"));
    assert_eq!(b[0]["right"].as_str(), Some("Sys::P::Car::wheel::pwr"));
}

#[test]
fn perform_lifts_onto_performs() {
    let els = ingest(PROBE);
    let p = field(get(&els, "Sys::P::Car"), "performs");
    let p = p.as_sequence().expect("performs");
    assert_eq!(p.len(), 1);
    assert_eq!(p[0]["name"].as_str(), Some("drive"));
}

#[test]
fn perform_of_a_typed_action_uses_the_type() {
    let els = ingest(
        "package P { action def Drive; part def Car { perform action d : Drive; perform Drive; } }",
    );
    let p = field(get(&els, "Sys::P::Car"), "performs");
    let p = p.as_sequence().unwrap();
    assert_eq!(p[0]["name"].as_str(), Some("d"));
    assert_eq!(p[0]["typedBy"].as_str(), Some("Drive"));
    assert_eq!(p[1].as_str(), Some("Drive"));
}

#[test]
fn exhibit_state_lifts_onto_exhibits_states_and_declares_the_state() {
    let els = ingest(PROBE);
    let e = field(get(&els, "Sys::P::Car"), "exhibitsStates");
    let e = e.as_sequence().expect("exhibitsStates");
    assert_eq!(e[0].as_str(), Some("Sys::P::Car::running"));
    // The usage declares a state of its own, so the entry resolves.
    assert!(els.iter().any(|x| x.qualified_name == "Sys::P::Car::running"));
}

#[test]
fn exhibit_of_a_state_def_resolves_relative_to_the_subtree() {
    let els = ingest("package P { state def Run; part def Car { exhibit state r : Run; } }");
    assert_eq!(field(get(&els, "Sys::P::Car"), "exhibitsStates")[0].as_str(), Some("Sys::P::Car::r"));
    assert_eq!(field(get(&els, "Sys::P::Car::r"), "typedBy").as_str(), Some("Run"));
}

#[test]
fn interface_usage_with_connect_becomes_a_typed_named_connection() {
    let els = ingest(PROBE);
    let sys2 = get(&els, "Sys::P::Sys2");
    let conns = field(sys2, "connections");
    let conns = conns.as_sequence().expect("connections");
    assert_eq!(conns.len(), 1);
    assert_eq!(conns[0]["name"].as_str(), Some("i1"));
    assert_eq!(conns[0]["typedBy"].as_str(), Some("PI"));
    let ends = conns[0]["ends"].as_sequence().unwrap();
    assert_eq!(ends[0]["end"].as_str(), Some("a"));
    assert_eq!(ends[0]["binds"].as_str(), Some("Sys::P::Sys2::e1::pwr"));
    assert_eq!(ends[1]["end"].as_str(), Some("b"));
    assert_eq!(ends[1]["binds"].as_str(), Some("Sys::P::Sys2::e2::inp"));
    // ... and the interface usage itself is an element.
    assert_eq!(field(get(&els, "Sys::P::Sys2::i1"), "typedBy").as_str(), Some("PI"));
}

#[test]
fn interface_def_ends_become_ends() {
    let els = ingest(PROBE);
    let ends = field(get(&els, "Sys::P::PI"), "ends");
    let ends = ends.as_sequence().expect("ends");
    assert_eq!(ends.len(), 2);
    assert_eq!(ends[0]["name"].as_str(), Some("a"));
    assert_eq!(ends[0]["typedBy"].as_str(), Some("PP"));
    assert_eq!(ends[1]["name"].as_str(), Some("b"));
    assert_eq!(ends[1]["isConjugated"].as_bool(), Some(true));
}

#[test]
fn a_single_interface_end_is_counted_not_emitted() {
    let els = ingest("package P { port def PP; interface def Half { end a : PP; } }");
    assert!(field(get(&els, "Sys::P::Half"), "ends").is_null());
    assert!(findings(&els, "W543").iter().any(|m| m.contains("interface end")));
    assert!(findings(&els, "E125").is_empty());
}

#[test]
fn feature_directions_are_kept() {
    let els = ingest(PROBE);
    assert_eq!(field(get(&els, "Sys::P::PP::x"), "direction").as_str(), Some("in"));
    assert_eq!(field(get(&els, "Sys::P::PP::y"), "direction").as_str(), Some("out"));
}

#[test]
fn conjugated_port_typing_sets_is_conjugated() {
    let els = ingest(PROBE);
    let inp = get(&els, "Sys::P::Eng::inp");
    assert_eq!(field(inp, "isConjugated").as_bool(), Some(true));
    assert_eq!(field(inp, "typedBy").as_str(), Some("PP"));
    assert!(field(get(&els, "Sys::P::Eng::pwr"), "isConjugated").is_null());
}

#[test]
fn ref_usage_sets_is_reference() {
    let els = ingest(PROBE);
    assert_eq!(field(get(&els, "Sys::P::Eng::helper"), "isReference").as_bool(), Some(true));
    assert!(field(get(&els, "Sys::P::Car::eng"), "isReference").is_null());
}

#[test]
fn nested_port_features_are_walked() {
    let els = ingest("package P { port def PP; part def A { port q : PP { attribute rate; item pkt; } } }");
    assert!(els.iter().any(|e| e.qualified_name == "Sys::P::A::q::rate"));
    assert!(els.iter().any(|e| e.qualified_name == "Sys::P::A::q::pkt"));
}

#[test]
fn unmapped_body_members_are_counted_in_w543() {
    let els = ingest("package P { part def A { constraint def K; alias z for A; } }");
    let w = findings(&els, "W543");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("constraint def x1") && w[0].contains("alias x1"), "{}", w[0]);
}

#[test]
fn one_bad_member_does_not_discard_the_file() {
    let els = ingest(
        "package P {\n\
         part def Good1;\n\
         part def Bad { this is not sysml ;;; }\n\
         part def Good2;\n\
         }\n",
    );
    assert!(els.iter().any(|e| e.qualified_name == "Sys::P::Good1"));
    assert!(els.iter().any(|e| e.qualified_name == "Sys::P::Good2"));
    let w = findings(&els, "W541");
    assert!(!w.is_empty(), "the bad member must raise W541");
    assert!(w.iter().any(|m| m.contains("line 3")), "{w:?}");
}

#[test]
fn a_clean_file_raises_no_parse_findings() {
    let els = ingest(PROBE);
    assert!(findings(&els, "W541").is_empty());
}

#[test]
fn dotted_connect_endpoints_resolve_in_validation_through_inherited_ports() {
    // GH #206: `connect eng.pwr to wheel.inp` -- `pwr`/`inp` are ports of `Eng`, inherited by the
    // `eng`/`wheel` usages through `typedBy:`; the full path validates clean.
    let src = "package P {\n\
        port def PP;\n\
        part def Eng { port pwr : PP; port inp : PP; }\n\
        part def Car { part eng : Eng; part wheel : Eng; connect eng.pwr to wheel.inp; }\n\
        }\n";
    let els = ingest(src);
    let result = syscribe_model::validator::validate(&els);
    let bad: Vec<_> = result.findings.iter().filter(|f| matches!(f.code, "E127" | "W056" | "W542")).collect();
    assert!(bad.is_empty(), "{bad:#?}");

    // ... and a tail that is not a member of the head's type is reported, not silently dropped.
    let els = ingest(&src.replace("wheel.inp", "wheel.nope"));
    let result = syscribe_model::validator::validate(&els);
    assert!(result.findings.iter().any(|f| f.code == "W056" && f.message.contains("nope")), "{:#?}", result.findings);
}

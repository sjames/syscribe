//! `REQ-TRS-SYSMLV2-099`/`-100`: the section 6 audit — a `#T` prefix on a usage applies metadata to
//! that usage wherever it is declared and in both parser shapes; a package-level `calc` usage shape
//! maps; a package declared with a qualified name is counted instead of vanishing.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_yaml::Value;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::sysmlv2::export::export_sysml;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-sysmlv2-audit-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// Ingest SysML text under a `sysmlSubmodel` package named `anchor`; the file must parse.
fn load_as(anchor: &str, sysml: &str) -> Vec<RawElement> {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, &format!("{anchor}/_index.md"), &format!("---\ntype: Package\nname: {anchor}\nsysmlSubmodel: true\n---\n"));
    write(&root, &format!("{anchor}/A.sysml"), sysml);
    let els = walk_model(&root).unwrap();
    let bad: Vec<_> = els.iter().flat_map(|e| e.derive_findings.iter()).filter(|(c, _, _)| c == "W541").collect();
    assert!(bad.is_empty(), "parse failed: {bad:?}\n{sysml}");
    els
}

fn load(sysml: &str) -> Vec<RawElement> {
    load_as("S", sysml)
}

fn find<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter().find(|e| e.qualified_name == q).unwrap_or_else(|| panic!("missing {q}; have {:?}", els.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()))
}

fn st(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(Value::as_str).map(str::to_string)
}

fn w543(els: &[RawElement]) -> Vec<String> {
    validate(els).findings.iter().filter(|f| f.code == "W543").map(|f| f.message.clone()).collect()
}

/// The `type:` of every `metadata:` entry on the element, in order.
fn meta_types(e: &RawElement) -> Vec<String> {
    e.frontmatter.metadata.clone().unwrap_or_default().iter().filter_map(|m| st(m, "type")).collect()
}

// ── REQ-TRS-SYSMLV2-099 ────────────────────────────────────────────────────

const PREFIXED_BODY: &str = "package P {\n  metadata def Tag;\n  part def A2;\n  port def Pt;\n  item def It;\n  part def A {\n    #Tag part a : A2;\n    #Tag attribute x : ScalarValues::Real;\n    #Tag action act;\n    #Tag port pt : Pt;\n    #Tag item it : It;\n    #Tag occurrence o;\n    part plain : A2;\n    @Tag;\n  }\n  part u : A { #Tag part b : A2; }\n  action def Act { #Tag part ap : A2; #Tag action step; }\n}\n";

#[test]
fn a_prefix_on_a_usage_inside_a_definition_body_applies_metadata_in_both_parser_shapes() {
    let els = load(PREFIXED_BODY);
    // Extension-keyword shape (part/port/item/occurrence) and bodiless-member shape
    // (attribute/action) both land on the prefixed usage, resolved to the ingested def.
    for q in ["S::P::A::a", "S::P::A::x", "S::P::A::act", "S::P::A::pt", "S::P::A::it", "S::P::A::o", "S::P::u::b", "S::P::Act::ap"] {
        assert_eq!(meta_types(find(&els, q)), vec!["S::P::Tag".to_string()], "{q}");
    }
    // The holder's own `@Tag;` is still the holder's, and an unprefixed sibling gets nothing.
    assert_eq!(meta_types(find(&els, "S::P::A")), vec!["S::P::Tag".to_string()]);
    assert!(find(&els, "S::P::A::plain").frontmatter.metadata.is_none());
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn a_prefix_on_a_package_level_or_root_level_usage_applies_metadata() {
    let els = load("metadata def Tag;\npart def X;\n#Tag part p : X;\npackage P {\n  #Tag part q : X;\n  #Tag attribute y : ScalarValues::Real;\n  #Tag part def D;\n}\n");
    for q in ["S::p", "S::P::q", "S::P::y", "S::P::D"] {
        assert_eq!(meta_types(find(&els, q)), vec!["S::Tag".to_string()], "{q}");
    }
    assert!(find(&els, "S::X").frontmatter.metadata.is_none());
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn a_prefix_on_a_member_that_becomes_no_element_is_dropped() {
    let els = load("package P {\n  metadata def Tag;\n  #Tag actor Act;\n  part def A;\n}\n");
    // The prefix is consumed by the actor, which maps to nothing; the next member is untouched.
    assert!(find(&els, "S::P::A").frontmatter.metadata.is_none());
    let w = w543(&els);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("actor x1") && w[0].contains("1 parsed construct"), "{w:?}");
}

#[test]
fn a_prefixed_usage_round_trips_through_export() {
    let els = load(PREFIXED_BODY);
    let out = export_sysml(&els, Some("S::P")).unwrap();
    assert!(!out.text.contains("not exported"), "{}", out.text);
    // Written back as a body annotation on the usage.
    assert!(out.text.contains("part a : A2 {\n            @S::P::Tag;\n        }"), "{}", out.text);
    assert!(out.text.contains("attribute x : ScalarValues::Real {\n            @S::P::Tag;\n        }"), "{}", out.text);
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Imp/_index.md", "---\ntype: Package\nname: Imp\nsysmlSubmodel: true\n---\n");
    write(&root, "Imp/out.sysml", &out.text);
    let back = walk_model(&root).unwrap();
    let bad: Vec<_> = back.iter().flat_map(|e| e.derive_findings.iter()).filter(|(c, _, _)| c == "W541").collect();
    assert!(bad.is_empty(), "parse-back failed: {bad:?}\n{}", out.text);
    // Every re-ingested usage carries exactly the entry its holder's own `@Tag;` re-ingests to.
    let expected = meta_types(find(&back, "Imp::P::A"));
    assert_eq!(expected.len(), 1, "{expected:?}\n{}", out.text);
    for q in ["A::a", "A::x", "A::act", "A::pt", "A::it", "A::o", "u::b", "Act::ap"] {
        assert_eq!(meta_types(find(&back, &format!("Imp::P::{q}"))), expected, "{q}\n{}", out.text);
    }
    assert!(find(&back, "Imp::P::A::plain").frontmatter.metadata.is_none());
}

// ── REQ-TRS-SYSMLV2-100 ────────────────────────────────────────────────────

#[test]
fn a_package_level_calc_usage_shape_maps_to_a_calculation() {
    let els = load("package P {\n  calc estimate [1];\n  calc def Est;\n}\n");
    assert_eq!(find(&els, "S::P::estimate").frontmatter.element_type, Some(ElementType::Calculation));
    assert_eq!(find(&els, "S::P::Est").frontmatter.element_type, Some(ElementType::CalculationDef));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn a_qualified_name_package_is_counted_once_and_its_members_are_not_ingested() {
    let els = load("package P { part def X; }\npackage A::B {\n  part def Y;\n  alias N for X;\n  actor Z;\n  package Inner { part def W; }\n}\nlibrary package L::M { part def V; }\n");
    assert_eq!(find(&els, "S::P::X").frontmatter.element_type, Some(ElementType::PartDef));
    assert!(els.iter().all(|e| !e.qualified_name.ends_with("::Y") && !e.qualified_name.ends_with("::W") && !e.qualified_name.ends_with("::V")), "{:?}", els.iter().map(|e| &e.qualified_name).collect::<Vec<_>>());
    let w = w543(&els);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("qualified package x2") && w[0].contains("2 parsed construct"), "{w:?}");
    for k in ["alias", "actor", "root-level"] {
        assert!(!w[0].contains(k), "{k} must not be counted: {w:?}");
    }
}

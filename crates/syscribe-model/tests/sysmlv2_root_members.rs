//! `REQ-TRS-SYSMLV2-098`: a definition or usage declared at a `.sysml` file's root, outside every
//! `package`, merges under the `sysmlSubmodel:` anchor package exactly as if it were a member of a
//! package with the anchor's qualified name — including the `#T` prefix, `satisfy`, metadata, `doc`
//! and `alias` forms — and exports back as a direct member of the anchor's body.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_yaml::Value;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::sysmlv2::export::export_sysml;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-sysmlv2-rootmembers-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// Ingest the given `.sysml` files under a `sysmlSubmodel` package `S` (whose `_index.md` body is
/// `index_doc`); every file must parse.
fn load_files(index_doc: &str, files: &[(&str, &str)]) -> Vec<RawElement> {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "S/_index.md", &format!("---\ntype: Package\nname: S\nsysmlSubmodel: true\n---\n{index_doc}"));
    for (rel, text) in files {
        write(&root, &format!("S/{rel}"), text);
    }
    let els = walk_model(&root).unwrap();
    let bad: Vec<_> = els.iter().flat_map(|e| e.derive_findings.iter()).filter(|(c, _, _)| c == "W541").collect();
    assert!(bad.is_empty(), "parse failed: {bad:?}");
    els
}

fn load(sysml: &str) -> Vec<RawElement> {
    load_files("", &[("A.sysml", sysml)])
}

fn find<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter().find(|e| e.qualified_name == q).unwrap_or_else(|| panic!("missing {q}; have {:?}", els.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()))
}

fn st(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(Value::as_str).map(str::to_string)
}

fn s(v: &Option<Value>) -> Option<String> {
    v.as_ref().and_then(Value::as_str).map(str::to_string)
}

fn w543(els: &[RawElement]) -> Vec<String> {
    validate(els).findings.iter().filter(|f| f.code == "W543").map(|f| f.message.clone()).collect()
}

fn meta(e: &RawElement) -> Vec<Value> {
    e.frontmatter.metadata.clone().unwrap_or_default()
}

// ── Root-level definitions and usages ──────────────────────────────────────

#[test]
fn root_level_definitions_and_usages_merge_under_the_anchor() {
    let els = load(
        "part def Loose { attribute mass : ScalarValues::Real; port p : Pt; }\npart loose : Loose;\nport def Pt;\nrequirement def R;\nattribute def A;\nitem def I;\naction def Act { action step1; }\nstate def St;\npackage P { part def X :> Loose; }\n",
    );
    let loose = find(&els, "S::Loose");
    assert_eq!(loose.frontmatter.element_type, Some(ElementType::PartDef));
    assert!(loose.file_path.ends_with("A.sysml"), "{}", loose.file_path);
    // Nested body members carry on under the root-level definition's own qname.
    assert_eq!(find(&els, "S::Loose::p").frontmatter.element_type, Some(ElementType::Port));
    assert_eq!(find(&els, "S::Loose::mass").frontmatter.element_type, Some(ElementType::Attribute));
    let usage = find(&els, "S::loose");
    assert_eq!(usage.frontmatter.element_type, Some(ElementType::Part));
    assert_eq!(s(&usage.frontmatter.typed_by).as_deref(), Some("Loose"));
    for (q, t) in [
        ("S::Pt", ElementType::PortDef),
        ("S::R", ElementType::RequirementDef),
        ("S::A", ElementType::AttributeDef),
        ("S::I", ElementType::ItemDef),
        ("S::Act", ElementType::ActionDef),
        ("S::St", ElementType::StateDef),
    ] {
        assert_eq!(find(&els, q).frontmatter.element_type, Some(t), "{q}");
    }
    // A package in the same file is still a sibling of the root-level members, under the anchor.
    assert_eq!(s(&find(&els, "S::P::X").frontmatter.supertype).as_deref(), Some("Loose"));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn root_level_members_merge_across_files_and_resolve_like_package_members() {
    let els = load_files(
        "",
        &[
            ("one.sysml", "part def Engine;\nrequirement def Thrust;\n"),
            ("two.sysml", "part engine : Engine;\nsatisfy Thrust by engine;\npackage P { part def Rotor { part e : Engine; } }\n"),
        ],
    );
    assert!(find(&els, "S::Engine").file_path.ends_with("one.sysml"));
    let engine = find(&els, "S::engine");
    assert!(engine.file_path.ends_with("two.sysml"));
    // A root-level `satisfy R by X;` lifts onto `X`, the same as a package-level one.
    assert_eq!(engine.frontmatter.satisfies.as_deref(), Some(&["Thrust".to_string()][..]));
    // A nested usage inside a package resolves its type to the root-level definition.
    assert_eq!(s(&find(&els, "S::P::Rotor::e").frontmatter.typed_by).as_deref(), Some("Engine"));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn root_level_prefix_metadata_doc_and_alias_lift_like_a_package_body() {
    let els = load_files(
        "Hand-written purpose.\n",
        &[(
            "A.sysml",
            "doc /* Root documentation. */\nmetadata def Tag;\n#Tag part def B;\nmetadata m : Tag;\n@Tag about B;\n@Tag about Nope;\nalias Top for P::X;\npackage P { part def X; }\n",
        )],
    );
    let anchor = find(&els, "S");
    // The anchor keeps its own body and gains the root `doc`.
    assert!(anchor.doc.starts_with("Hand-written purpose."), "{:?}", anchor.doc);
    assert!(anchor.doc.contains("Root documentation."), "{:?}", anchor.doc);
    // `#Tag part def B;` prefixes the member that follows; `@Tag about B;` lands on `B` too.
    let b = find(&els, "S::B");
    let b_meta = meta(b);
    assert_eq!(b_meta.len(), 2, "{b_meta:?}");
    assert!(b_meta.iter().all(|m| st(m, "type").as_deref() == Some("S::Tag")), "{b_meta:?}");
    // `metadata m : Tag;` is the anchor's own application; the dangling `about` stays on it.
    let a_meta = meta(anchor);
    assert_eq!(a_meta.len(), 2, "{a_meta:?}");
    let named = a_meta.iter().find(|m| st(m, "name").as_deref() == Some("m")).unwrap_or_else(|| panic!("{a_meta:?}"));
    assert_eq!(st(named, "type").as_deref(), Some("S::Tag"));
    let dangling = a_meta.iter().find(|m| st(m, "about").is_some()).unwrap_or_else(|| panic!("{a_meta:?}"));
    assert_eq!(st(dangling, "about").as_deref(), Some("Nope"));
    // `REQ-TRS-SYSMLV2-095` unchanged: the alias lifts onto the anchor.
    let aliases = anchor.frontmatter.aliases.as_ref().unwrap();
    assert_eq!((st(&aliases[0], "name").as_deref(), st(&aliases[0], "for").as_deref()), (Some("Top"), Some("P::X")));
    // Only the dangling `about` counts; nothing is a `root-level member` any more.
    let w = w543(&els);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("1 parsed construct") && w[0].contains("metadata x1"), "{w:?}");
    assert!(!w[0].contains("root-level member"), "{w:?}");
}

#[test]
fn root_level_members_count_under_their_package_kind_in_w543() {
    let els = load("part def Loose;\nactor Act;\nfilter @Nothing;\nclassifier K;\n");
    assert_eq!(find(&els, "S::Loose").frontmatter.element_type, Some(ElementType::PartDef));
    let w = w543(&els);
    assert_eq!(w.len(), 1, "{w:?}");
    for k in ["actor x1", "filter x1", "KerML declaration x1"] {
        assert!(w[0].contains(k), "missing {k}: {w:?}");
    }
    assert!(!w[0].contains("root-level member"), "{w:?}");
}

// ── Export round trip ──────────────────────────────────────────────────────

#[test]
fn root_level_members_export_as_direct_members_of_the_anchor_and_round_trip() {
    let els = load("part def Loose { attribute mass : ScalarValues::Real; }\npart loose : Loose;\nrequirement def R;\npackage P { part def X :> Loose; }\n");
    let out = export_sysml(&els, Some("S")).unwrap();
    assert!(!out.text.contains("not exported"), "{}", out.text);
    // Direct members of the anchor's body: one indentation level inside `package S {`.
    assert!(out.text.contains("package S {\n"), "{}", out.text);
    assert!(out.text.contains("\n    part def Loose {\n"), "{}", out.text);
    assert!(out.text.contains("\n    part loose : Loose;\n"), "{}", out.text);
    assert!(out.text.contains("\n    requirement def R"), "{}", out.text);
    assert!(out.text.contains("\n    package P {\n"), "{}", out.text);

    // Re-ingesting the export under a fresh anchor reproduces the same elements.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Imp/_index.md", "---\ntype: Package\nname: Imp\nsysmlSubmodel: true\n---\n");
    write(&root, "Imp/out.sysml", &out.text);
    let back = walk_model(&root).unwrap();
    let bad: Vec<_> = back.iter().flat_map(|e| e.derive_findings.iter()).filter(|(c, _, _)| c == "W541").collect();
    assert!(bad.is_empty(), "parse-back failed: {bad:?}\n{}", out.text);
    for q in ["Loose", "loose", "R", "P", "P::X"] {
        let (a, b) = (find(&els, &format!("S::{q}")), find(&back, &format!("Imp::S::{q}")));
        assert_eq!(a.frontmatter.element_type, b.frontmatter.element_type, "{q}");
        assert_eq!(a.frontmatter.typed_by, b.frontmatter.typed_by, "{q}");
        assert_eq!(a.frontmatter.supertype, b.frontmatter.supertype, "{q}");
    }
    assert_eq!(find(&back, "Imp::S::Loose::mass").frontmatter.typed_by, find(&els, "S::Loose::mass").frontmatter.typed_by);
    assert!(w543(&back).is_empty(), "{:?}", w543(&back));
}

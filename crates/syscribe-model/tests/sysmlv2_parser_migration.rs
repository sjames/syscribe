//! `REQ-TRS-SYSMLV2-073`: ingestion and export read the same things on `sysml-v2-parser` 0.55+
//! (span-handle AST) that they read on 0.54 (owned strings). These tests lock the readings the
//! migration was most likely to disturb: handles resolved against the *right* document when
//! several files merge into one package, quoted names, qualified references, literals and
//! units, rendered expression text and doc comments.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::RawElement;
use syscribe_model::sysmlv2::report::PARSER_VERSION;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-migr-{}-{}",
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

fn load_files(files: &[(&str, &str)]) -> Vec<RawElement> {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "S/_index.md", "---\ntype: Package\nname: S\nsysmlSubmodel: true\n---\n");
    for (name, text) in files {
        write(&root, &format!("S/{name}"), text);
    }
    walk_model(&root).unwrap()
}

fn find<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter()
        .find(|e| e.qualified_name == q)
        .unwrap_or_else(|| panic!("missing {q}; have {:?}", els.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()))
}

#[test]
fn the_pinned_parser_is_zero_point_fifty_seven_or_later() {
    let mut it = PARSER_VERSION.split('.').map(|p| p.parse::<u32>().unwrap());
    let (major, minor) = (it.next().unwrap(), it.next().unwrap());
    assert!(major > 0 || minor >= 57, "{PARSER_VERSION}");
}

#[test]
fn names_resolve_against_their_own_file_when_packages_merge() {
    // Two files contribute to one package; every span handle must be read from its own source,
    // so a long name in the first file and a short one in the second cannot cross-contaminate.
    let els = load_files(&[
        ("A.sysml", "package P {\n  part def AVeryLongPartDefinitionNameIndeed :> Base;\n  part def Base;\n}\n"),
        ("B.sysml", "package P {\n  part def Z :> Base;\n  part k : P::Base;\n}\n"),
    ]);
    assert_eq!(
        find(&els, "S::P::AVeryLongPartDefinitionNameIndeed").frontmatter.supertype.as_ref().and_then(|v| v.as_str()),
        Some("Base")
    );
    assert_eq!(find(&els, "S::P::Z").frontmatter.supertype.as_ref().and_then(|v| v.as_str()), Some("Base"));
    assert_eq!(find(&els, "S::P::k").frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("P::Base"));
}

#[test]
fn quoted_names_and_qualified_references_are_decoded() {
    let els = load_files(&[(
        "A.sysml",
        "package P {\n  part def 'Rotor Blade';\n  part def Wing :> 'Rotor Blade';\n  part w : P::'Rotor Blade';\n}\n",
    )]);
    find(&els, "S::P::Rotor Blade");
    assert_eq!(find(&els, "S::P::Wing").frontmatter.supertype.as_ref().and_then(|v| v.as_str()), Some("Rotor Blade"));
    assert_eq!(find(&els, "S::P::w").frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("P::Rotor Blade"));
}

#[test]
fn doc_comments_and_literal_values_with_units_are_read() {
    let els = load_files(&[(
        "A.sysml",
        "package P {\n  part def Engine {\n    doc /* Burns fuel. */\n    attribute mass : Real = 12.5 [kg];\n    attribute label : String = \"hi\";\n    attribute on : Boolean = true;\n  }\n}\n",
    )]);
    assert!(find(&els, "S::P::Engine").doc.contains("Burns fuel."));
    let mass = find(&els, "S::P::Engine::mass");
    assert_eq!(mass.frontmatter.unit.as_deref(), Some("kg"));
    assert_eq!(mass.frontmatter.value, Some(serde_yaml::Value::Number(serde_yaml::Number::from(12.5))));
    assert_eq!(
        find(&els, "S::P::Engine::label").frontmatter.value,
        Some(serde_yaml::Value::String("hi".into()))
    );
    assert_eq!(find(&els, "S::P::Engine::on").frontmatter.value, Some(serde_yaml::Value::Bool(true)));
}

#[test]
fn guard_and_assignment_expressions_render_as_before() {
    let els = load_files(&[(
        "A.sysml",
        "package P {\n  state def M {\n    state a;\n    state b;\n    transition first a accept Go if x > 1 and (y == 2) then b;\n  }\n  action def Act {\n    for i in 1..3 { action t; }\n    assign v := f(a, 2) + w.z;\n  }\n}\n",
    )]);
    let m = find(&els, "S::P::M");
    let t = &m.frontmatter.transitions.as_ref().unwrap()[0];
    assert_eq!(t.get("guard").and_then(|v| v.as_str()), Some("x > 1 && (y == 2)"));
    let a = find(&els, "S::P::Act");
    let subs = a.frontmatter.sub_actions.as_ref().unwrap();
    let text = serde_yaml::to_string(subs).unwrap();
    assert!(text.contains("1..3"), "{text}");
    assert!(text.contains("f(a, 2) + w.z"), "{text}");
}

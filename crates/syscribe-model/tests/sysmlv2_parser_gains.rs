//! What the 0.57 parser makes possible (`REQ-TRS-SYSMLV2-074`..`-076`): guarded successions,
//! qualified/declaring `include` targets, and bare package-level usages with their values.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::sysmlv2::export::export_sysml;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-gains-{}-{}",
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
    els.iter()
        .find(|e| e.qualified_name == q)
        .unwrap_or_else(|| panic!("missing {q}; have {:?}", els.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()))
}

fn s(v: &serde_yaml::Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(str::to_string)
}

// ── REQ-TRS-SYSMLV2-074 ─────────────────────────────────────────────────────

#[test]
fn guarded_succession_is_ingested_with_its_guard() {
    let els = load(
        "package P {\n  action def Seq {\n    action a;\n    action b;\n    first a if sig.ok and x > 1 then b;\n    first b then a;\n  }\n}\n",
    );
    let seq = find(&els, "S::P::Seq");
    let succ = seq.frontmatter.succession_connections.as_ref().unwrap();
    assert_eq!(succ.len(), 2, "{succ:?}");
    assert_eq!(s(&succ[0], "after").as_deref(), Some("a"));
    assert_eq!(s(&succ[0], "before").as_deref(), Some("b"));
    assert_eq!(s(&succ[0], "guard").as_deref(), Some("sig.ok && x > 1"));
    assert!(succ[1].get("guard").is_none(), "an unguarded succession carries no guard: {succ:?}");
}

#[test]
fn guarded_succession_exports_and_reads_back_identically() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "A/Seq.md",
        "---\ntype: ActionDef\nname: Seq\nsubActions:\n  - name: a\n    kind: PerformAction\n  - name: b\n    kind: PerformAction\nsuccessionConnections:\n  - after: a\n    before: b\n    guard: \"signaller.routeRequest.isValid\"\n---\n",
    );
    write(&root, "A/_index.md", "---\ntype: Package\nname: A\n---\n");
    let els = walk_model(&root).unwrap();
    let out = export_sysml(&els, None).unwrap();
    assert!(out.text.contains("first a if signaller.routeRequest.isValid then b;"), "{}", out.text);
    assert_eq!(out.report.degraded_behaviour, 0, "{}", out.text);

    // Re-ingesting the exported text gives the same succession entry.
    let back_root = tempdir();
    write(&back_root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&back_root, "S/_index.md", "---\ntype: Package\nname: S\nsysmlSubmodel: true\n---\n");
    write(&back_root, "S/A.sysml", &out.text);
    let back = walk_model(&back_root).unwrap();
    let seq = back.iter().find(|e| e.qualified_name.ends_with("::Seq")).expect("Seq");
    let succ = seq.frontmatter.succession_connections.as_ref().unwrap();
    assert_eq!(s(&succ[0], "guard").as_deref(), Some("signaller.routeRequest.isValid"));
}

#[test]
fn a_guard_that_is_not_an_expression_still_degrades_to_a_comment() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "A/Seq.md",
        "---\ntype: ActionDef\nname: Seq\nsubActions:\n  - name: a\n    kind: PerformAction\n  - name: b\n    kind: PerformAction\nsuccessionConnections:\n  - after: a\n    before: b\n    guard: \"(((\"\n---\n",
    );
    write(&root, "A/_index.md", "---\ntype: Package\nname: A\n---\n");
    let els = walk_model(&root).unwrap();
    let out = export_sysml(&els, None).unwrap();
    assert!(out.text.contains("// successionConnection not exported"), "{}", out.text);
    assert_eq!(out.report.degraded_behaviour, 1);
}

// ── REQ-TRS-SYSMLV2-075 ─────────────────────────────────────────────────────

#[test]
fn include_accepts_qualified_and_declaring_forms() {
    let els = load(
        "package P {\n  use case def Fly;\n  package Q { use case def Land; }\n  use case def Mission {\n    include Fly;\n    include Q::Land;\n  }\n  use case def Other {\n    include use case l2 : Q::Land;\n  }\n}\n",
    );
    let inc = |q: &str| find(&els, q).frontmatter.includes.clone().unwrap();
    assert_eq!(inc("S::P::Mission"), vec!["S::P::Fly".to_string(), "S::P::Q::Land".to_string()]);
    assert_eq!(inc("S::P::Other"), vec!["S::P::Q::Land".to_string()]);
}

#[test]
fn an_unresolved_qualified_include_is_counted_in_w543() {
    let els = load("package P {\n  use case def Mission {\n    include Nowhere::Missing;\n  }\n}\n");
    let w: Vec<String> =
        validate(&els).findings.iter().filter(|f| f.code == "W543").map(|f| f.message.clone()).collect();
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("include x1"), "{w:?}");
}

// ── REQ-TRS-SYSMLV2-076 ─────────────────────────────────────────────────────

#[test]
fn bare_package_level_usages_are_usages_with_values_and_units() {
    let els = load(
        "package P {\n  part def Engine;\n  attribute maxMass : Real = 12.5 [kg];\n  attribute label : String = \"x\";\n  port p : Engine;\n  item i : Engine;\n}\n",
    );
    let m = find(&els, "S::P::maxMass");
    assert_eq!(m.frontmatter.element_type, Some(ElementType::Attribute));
    assert_eq!(m.frontmatter.unit.as_deref(), Some("kg"));
    assert_eq!(m.frontmatter.value, Some(serde_yaml::Value::Number(serde_yaml::Number::from(12.5))));
    assert_eq!(find(&els, "S::P::label").frontmatter.value, Some(serde_yaml::Value::String("x".into())));
    let p = find(&els, "S::P::p");
    assert_eq!(p.frontmatter.element_type, Some(ElementType::Port));
    assert_eq!(p.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("Engine"));
    assert_eq!(find(&els, "S::P::i").frontmatter.element_type, Some(ElementType::Item));
}

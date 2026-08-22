//! Integration tests for mapping `flow def`/`flow` onto the native
//! `FlowDef`/`Flow` schema, plus the `flowConnections:` lift onto an owning
//! `PartDef`/`Part` (`REQ-TRS-SYSMLV2-024`).
//!
//! Mirrors `sysmlv2_concerns.rs`. Covers the dual pattern mirrored from
//! `REQ-TRS-SYSMLV2-010`'s connection lift: a named flow usage becomes both
//! its own `Flow` element *and* a `flowConnections:` entry on the owning
//! part; an anonymous one only ever contributes the entry.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::ElementType;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-flows-test-{}-{}",
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

fn base_model(root: &Path) {
    write(root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
}

fn codes(findings: &[syscribe_model::validator::Finding]) -> Vec<&str> {
    findings.iter().map(|f| f.code).collect()
}

#[test]
fn a_flow_def_becomes_a_real_flowdef_with_supertype_and_no_ends_or_itemtype() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flows.sysml",
        "package Flows {\n\
         flow def BaseFlow;\n\
         flow def PowerFlow : BaseFlow {\n\
         doc /* Power transfer flow. */\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Flows::PowerFlow")
        .expect("PowerFlow should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::FlowDef));
    assert_eq!(el.frontmatter.supertype.as_ref().and_then(|v| v.as_str()), Some("BaseFlow"));
    assert!(el.doc.contains("Power transfer flow."));
    // Explicit descope: no AST source for these on a `flow def` body.
    assert!(el.frontmatter.ends.is_none(), "{:#?}", el.frontmatter.ends);
    assert!(el.frontmatter.item_type.is_none(), "{:#?}", el.frontmatter.item_type);
}

#[test]
fn a_named_top_level_flow_usage_becomes_a_real_flow_with_item_type() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flows.sysml",
        "package Flows {\n\
         part def Src;\n\
         part def Dst;\n\
         attribute def Fuel;\n\
         part src : Src;\n\
         part dst : Dst;\n\
         flow transfer : Fuel from src to dst;\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Flows::transfer")
        .expect("transfer should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::Flow));
    assert_eq!(el.frontmatter.item_type.as_deref(), Some("Fuel"));
}

#[test]
fn an_anonymous_flow_nested_in_a_part_def_becomes_a_flow_connections_entry_only() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flows.sysml",
        "package Flows {\n\
         part def V {\n\
         part a { port x; }\n\
         part b { port y; }\n\
         flow a.x to b.y;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let v = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Flows::V")
        .expect("V should be a real element");
    let flows = v.frontmatter.flow_connections.as_deref().expect("flowConnections present");
    assert_eq!(flows.len(), 1, "{flows:#?}");
    let f = &flows[0];
    assert_eq!(f.get("from").and_then(|v| v.as_str()), Some("SysML2Legacy::Flows::V::a::x"));
    assert_eq!(f.get("to").and_then(|v| v.as_str()), Some("SysML2Legacy::Flows::V::b::y"));
    assert_eq!(f.get("kind").and_then(|v| v.as_str()), Some("streaming"));
    assert!(f.get("name").is_none(), "{f:#?}");

    // Anonymous: no separate Flow element synthesized -- only the part
    // hierarchy itself (V, its ports a/b and their nested x/y ports).
    assert!(
        !elements.iter().any(|e| e.frontmatter.element_type == Some(ElementType::Flow)),
        "{:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
}

#[test]
fn a_named_flow_nested_in_a_part_usage_produces_both_an_element_and_an_entry() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flows.sysml",
        "package Flows {\n\
         part def Housing {\n\
         part a;\n\
         part b;\n\
         }\n\
         part housing : Housing {\n\
         part a;\n\
         part b;\n\
         message evt : Fuel from a to b;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    // (a) its own standalone Flow element
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Flows::housing::evt")
        .expect("evt should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::Flow));
    assert_eq!(el.frontmatter.item_type.as_deref(), Some("Fuel"));

    // (b) also a flowConnections: entry on the owning part usage
    let housing = elements.iter().find(|e| e.qualified_name == "SysML2Legacy::Flows::housing").unwrap();
    let flows = housing.frontmatter.flow_connections.as_deref().expect("flowConnections present");
    assert_eq!(flows.len(), 1, "{flows:#?}");
    let f = &flows[0];
    assert_eq!(f.get("name").and_then(|v| v.as_str()), Some("evt"));
    assert_eq!(f.get("kind").and_then(|v| v.as_str()), Some("message"));
    assert_eq!(f.get("item").and_then(|v| v.as_str()), Some("Fuel"));
}

#[test]
fn succession_flow_kind_lifts_as_succession() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flows.sysml",
        "package Flows {\n\
         action def A {\n\
         part focus;\n\
         part shoot;\n\
         succession flow focus.image to shoot.image;\n\
         }\n\
         }\n",
    );

    // Note: this flow is nested inside an *action* body, which
    // REQ-TRS-SYSMLV2-019 already, separately, excludes FlowUsage from --
    // confirm it stays invisible here too (no flowConnections: lift, since
    // that's PartDef/Part-only, and no crash).
    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "{:#?}", result.findings);
}

#[test]
fn a_genuinely_two_segment_truncated_flow_endpoint_raises_w542() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flows.sysml",
        "package Flows {\n\
         part def Remote;\n\
         part def V {\n\
         part a : Remote;\n\
         part b;\n\
         flow a.notARedeclaredFeature to b;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    assert!(codes(&result.findings).contains(&"W542"), "{:#?}", result.findings);
}

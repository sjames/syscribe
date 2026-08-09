//! Integration tests for lifting a named `connection name : Type connect a
//! to b (, c)*;` usage's endpoints onto the *owning* `part def`/`part`'s
//! `connections:` field (`REQ-TRS-SYSMLV2-010`).
//!
//! Unlike `sysmlv2_doc.rs`/`sysmlv2_metadata.rs`, correctness here isn't just
//! "did the right string land in the right field" — the whole point is a
//! *resolvable* graph edge, so most of these tests drive `graph::build_graph`
//! (via `connectivity`'s own machinery) rather than only inspecting
//! `frontmatter.connections` directly. See `ADR-SYS-SYSMLV2-001`'s addendum
//! for why endpoints are qualified to `<owning qname>::<head>`, not a
//! literal or fully-qualified chain — both were tried and both silently
//! produced zero edges before this design was settled on.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::graph::{build_graph, EdgeKind};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-connections-test-{}-{}",
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

#[test]
fn a_binary_connect_lifts_onto_the_owning_part_and_produces_a_real_edge() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def Ecu {\n\
         port p1 : SomePort;\n\
         }\n\
         part def Holder {\n\
         part a : Ecu;\n\
         part b : Ecu;\n\
         connection c : SomeConnDef connect a.p1 to b.p1;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let holder = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::Holder")
        .unwrap();
    let conns = holder.frontmatter.connections.as_ref().expect("Holder should carry connections:");
    assert_eq!(conns.len(), 1);
    let m = conns[0].as_mapping().unwrap();
    assert_eq!(
        m.get("from").and_then(|v| v.as_str()),
        Some("SysML2Legacy::CarOS::Holder::a")
    );
    assert_eq!(
        m.get("to").and_then(|v| v.as_str()),
        Some("SysML2Legacy::CarOS::Holder::b")
    );
    assert_eq!(m.get("typedBy").and_then(|v| v.as_str()), Some("SomeConnDef"));

    // The whole point: this must be a real, resolvable graph edge, not just
    // a copied string nobody reads.
    let (graph, idx) = build_graph(&elements);
    let a_idx = *idx.get("SysML2Legacy::CarOS::Holder::a").expect("a node should exist");
    let b_idx = *idx.get("SysML2Legacy::CarOS::Holder::b").expect("b node should exist");
    let has_edge = graph
        .edges_connecting(a_idx, b_idx)
        .any(|e| *e.weight() == EdgeKind::Connection);
    assert!(has_edge, "expected a real Connection edge between Holder::a and Holder::b");
}

#[test]
fn an_nary_connect_lifts_to_the_ends_shape_and_every_end_resolves() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def Ecu;\n\
         part def Holder {\n\
         part a : Ecu;\n\
         part b : Ecu;\n\
         part c : Ecu;\n\
         connection bus : SomeConnDef connect (a.p1, b.p1, c.p1);\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let holder = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::Holder")
        .unwrap();
    let conns = holder.frontmatter.connections.as_ref().unwrap();
    assert_eq!(conns.len(), 1);
    let m = conns[0].as_mapping().unwrap();
    let ends = m.get("ends").and_then(|v| v.as_sequence()).expect("ends: sequence");
    let binds: Vec<&str> = ends
        .iter()
        .map(|e| e.as_mapping().unwrap().get("binds").and_then(|v| v.as_str()).unwrap())
        .collect();
    assert_eq!(
        binds,
        vec![
            "SysML2Legacy::CarOS::Holder::a",
            "SysML2Legacy::CarOS::Holder::b",
            "SysML2Legacy::CarOS::Holder::c",
        ]
    );

    // Every end resolves to a real edge from the first (a) to each other.
    let (graph, idx) = build_graph(&elements);
    let get = |name: &str| *idx.get(name).unwrap_or_else(|| panic!("missing node {name}"));
    let a = get("SysML2Legacy::CarOS::Holder::a");
    let b = get("SysML2Legacy::CarOS::Holder::b");
    let c = get("SysML2Legacy::CarOS::Holder::c");
    assert!(graph.edges_connecting(a, b).any(|e| *e.weight() == EdgeKind::Connection));
    assert!(graph.edges_connecting(a, c).any(|e| *e.weight() == EdgeKind::Connection));
}

#[test]
fn a_connection_usage_with_no_connect_clause_contributes_no_entry_no_regression() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def Holder {\n\
         connection c : SomeConnDef;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let holder = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::Holder")
        .unwrap();
    assert_eq!(holder.frontmatter.connections, None);

    // The standalone Connection element is still synthesized unchanged
    // (REQ-TRS-SYSMLV2-007's existing mapping, untouched by this feature).
    let conn = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::Holder::c")
        .unwrap();
    assert_eq!(conn.frontmatter.element_type, Some(syscribe_model::element::ElementType::Connection));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn a_connect_on_a_part_usage_also_lifts() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def Ecu;\n\
         part def HolderDef;\n\
         part holder : HolderDef {\n\
         part a : Ecu;\n\
         part b : Ecu;\n\
         connection c : SomeConnDef connect a.p1 to b.p1;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let holder = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::holder")
        .unwrap();
    let conns = holder.frontmatter.connections.as_ref().expect("holder usage should carry connections:");
    let m = conns[0].as_mapping().unwrap();
    assert_eq!(
        m.get("from").and_then(|v| v.as_str()),
        Some("SysML2Legacy::CarOS::holder::a")
    );
}

#[test]
fn a_trailing_segment_past_the_head_is_discarded_not_a_resolution_attempt() {
    // Deliberate granularity choice, not best-effort: even if the "p1"
    // suffix were somehow separately redeclared, connections: only ever
    // carries the head. This test pins that the *entry itself* always
    // reads "Holder::a"/"Holder::b", regardless of what follows the dot.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def Ecu {\n\
         port p1 : SomePort;\n\
         port p2 : SomePort;\n\
         }\n\
         part def Holder {\n\
         part a : Ecu;\n\
         part b : Ecu;\n\
         connection c1 : SomeConnDef connect a.p1 to b.p1;\n\
         connection c2 : SomeConnDef connect a.p2 to b.p2;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let holder = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::Holder")
        .unwrap();
    let conns = holder.frontmatter.connections.as_ref().unwrap();
    assert_eq!(conns.len(), 2, "both connection usages should each contribute an entry");
    for entry in conns {
        let m = entry.as_mapping().unwrap();
        assert_eq!(m.get("from").and_then(|v| v.as_str()), Some("SysML2Legacy::CarOS::Holder::a"));
        assert_eq!(m.get("to").and_then(|v| v.as_str()), Some("SysML2Legacy::CarOS::Holder::b"));
    }
}

#[test]
fn a_bare_unchained_connect_endpoint_also_resolves() {
    // connect a to b; (no dots at all) parses as Expression::FeatureRef, not
    // FeatureChainRef -- a distinct AST shape from the dotted case, worth
    // its own coverage.
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/CarOS.sysml",
        "package CarOS {\n\
         part def Ecu;\n\
         part def Holder {\n\
         part a : Ecu;\n\
         part b : Ecu;\n\
         connection c : SomeConnDef connect a to b;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let holder = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::CarOS::Holder")
        .unwrap();
    let conns = holder.frontmatter.connections.as_ref().unwrap();
    let m = conns[0].as_mapping().unwrap();
    assert_eq!(m.get("from").and_then(|v| v.as_str()), Some("SysML2Legacy::CarOS::Holder::a"));
    assert_eq!(m.get("to").and_then(|v| v.as_str()), Some("SysML2Legacy::CarOS::Holder::b"));
}

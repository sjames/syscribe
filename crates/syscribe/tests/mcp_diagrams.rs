//! Integration tests for the diagram/doc-integrity MCP tools (Feature B).
//! Realises TC-TRS-MCP-040 (lint_docs), 041 (render_diagram), 042 (diagram_coverage),
//! 043 (generate_view).

mod common;
use common::*;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static C: AtomicU64 = AtomicU64::new(0);

fn tmp_md(body: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = C.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("syscribe-doc-{}-{}-{}", std::process::id(), nanos, n));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("doc.md");
    std::fs::write(&p, body).unwrap();
    p
}

// ---- TC-TRS-MCP-040: lint_docs ----------------------------------------------

#[test]
fn lint_docs_flags_dangling_stable_id() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let doc = tmp_md("# Notes\n\nThis cites REQ-NOPE-999 which does not exist.\n");
    let res = mcp.call_tool("lint_docs", json!({"paths": [doc.to_str().unwrap()]}));
    let findings = res.get("findings").or_else(|| res.as_array().map(|_| &res)).and_then(|f| f.as_array()).cloned()
        .or_else(|| res.as_array().cloned())
        .expect("findings array");
    assert!(
        findings.iter().any(|f| f.get("code").and_then(|c| c.as_str()) == Some("W099")
            && serde_json::to_string(f).unwrap().contains("REQ-NOPE-999")),
        "a W099 finding cites REQ-NOPE-999; got {findings:?}"
    );
}

#[test]
fn lint_docs_does_not_flag_resolvable_reference() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let doc = tmp_md("# Notes\n\nThis cites REQ-FX-001 which exists.\n");
    let res = mcp.call_tool("lint_docs", json!({"paths": [doc.to_str().unwrap()]}));
    let s = serde_json::to_string(&res).unwrap();
    assert!(!s.contains("REQ-FX-001"), "a resolvable reference is not flagged; got {s}");
}

// ---- TC-TRS-MCP-041: render_diagram (source only) ---------------------------

#[test]
fn render_diagram_returns_plantuml_source_and_findings() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("render_diagram", json!({"ref": "Diagrams::FxBlock", "format": "plantuml"}));
    let source = res.get("source").and_then(|s| s.as_str()).expect("source string");
    assert!(source.contains("@startuml"), "PlantUML source returned; got {source}");
    assert!(res.get("findings").and_then(|f| f.as_array()).is_some(), "structural findings array present");
    // Source-only: no rendered image artifact.
    assert!(res.get("svg").is_none() && res.get("png").is_none(), "no rendered image is returned");
}

#[test]
fn render_diagram_generates_mermaid_from_the_ir_on_request() {
    // REQ-TRS-VIS-009: `format: mermaid` on a manifest/derived diagram is the
    // generated classDiagram/flowchart, with a `%% ref:` per node.
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("render_diagram", json!({"ref": "Diagrams::FxBlock", "format": "mermaid"}));
    assert_eq!(res.get("format").and_then(|f| f.as_str()), Some("mermaid"));
    let source = res.get("source").and_then(|s| s.as_str()).expect("source string");
    assert!(source.starts_with("classDiagram\n"), "generated Mermaid; got {source}");
    assert!(source.contains("%% ref: Parts::Base") && source.contains("%% ref: Parts::Derived"), "{source}");
    assert!(source.contains("s_base <|-- s_derived"), "{source}");
    assert!(res.get("findings").and_then(|f| f.as_array()).is_some());
}

#[test]
fn render_diagram_svg_draws_pinned_and_unpinned_diagrams_alike() {
    // REQ-TRS-VIS-010/016: `format: svg` draws a fully pinned diagram from its
    // pins and an unpinned one through the embedded ELK; only a diagram with
    // nothing to draw is a tool error, with the CLI's exact message.
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("render_diagram", json!({"ref": "Diagrams::FxPinned", "format": "svg"}));
    let source = res.get("source").and_then(|s| s.as_str()).expect("source string");
    assert!(source.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:sysml=\"urn:syscribe:1.0\""), "{source}");
    assert!(source.contains("sysml:ref=\"Parts::Base\""));
    let res = mcp.call_tool("render_diagram", json!({"ref": "Diagrams::FxBlock", "format": "svg"}));
    let source = res.get("source").and_then(|s| s.as_str()).expect("source string");
    assert!(source.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""), "unpinned diagram laid out: {source}");
    assert!(source.contains("sysml:source=\"s-derived\" sysml:target=\"s-base\""), "{source}");
    let raw = mcp.call_tool_raw("render_diagram", json!({"ref": "Diagrams::FxEmpty", "format": "svg"}));
    assert_eq!(raw.get("isError").and_then(|e| e.as_bool()), Some(true), "a tool error; got {raw}");
    let text = serde_json::to_string(&raw).unwrap();
    assert!(text.contains("'Diagrams::FxEmpty' has no shapes to draw"), "{text}");
}

#[test]
fn render_diagram_returns_mermaid_source_for_mermaid() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("render_diagram", json!({"ref": "Diagrams::FxMermaid"}));
    let source = res.get("source").and_then(|s| s.as_str()).unwrap_or("");
    assert!(source.contains("graph TD") || source.contains("Derived"), "Mermaid source returned; got {source}");
}

// ---- TC-TRS-MCP-042: diagram_coverage ---------------------------------------

#[test]
fn diagram_coverage_reports_uncovered_and_excludes_referenced() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("diagram_coverage", json!({}));
    let uncovered: Vec<String> = res
        .get("uncoveredElements").or_else(|| res.get("uncovered"))
        .and_then(|u| u.as_array())
        .map(|a| a.iter().filter_map(|x| {
            x.as_str().map(String::from).or_else(|| x.get("qname").and_then(|q| q.as_str()).map(String::from))
        }).collect())
        .unwrap_or_default();
    assert!(uncovered.iter().any(|q| q.contains("REQ-FX-001")), "REQ-FX-001 is uncovered by any diagram; got {uncovered:?}");
    // Parts::Base is referenced by the FxBlock diagram (shape s-base) -> not uncovered.
    assert!(!uncovered.iter().any(|q| q.ends_with("Parts::Base")), "Parts::Base is diagram-referenced, not uncovered");
}

// ---- TC-TRS-MCP-043: generate_view ------------------------------------------

#[test]
fn generate_view_traceability_is_clean_mermaid() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("generate_view", json!({"kind": "traceability"}));
    let source = res.get("source").and_then(|s| s.as_str()).expect("source");
    assert!(source.contains("REQ-FX-001"), "traceability view references the requirement");
    assert!(source.contains("%% ref:"), "view embeds %% ref: annotations for round-tripping; got {source}");
}

#[test]
fn generate_view_containment_returns_source() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("generate_view", json!({"kind": "containment"}));
    let source = res.get("source").and_then(|s| s.as_str()).expect("source");
    assert!(!source.is_empty(), "containment view returns non-empty Mermaid source");
}

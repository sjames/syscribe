//! Integration tests for `syscribe export-html` (REQ-TRS-HTML-001..007).
//! Black-box: runs the binary against the checked-in fixture model and inspects
//! the generated static site. Written before the implementation (RED).
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model")
}

fn tmp_out() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("syscribe-html-{}-{}-{}", std::process::id(), nanos, n))
}

/// Run `syscribe export-html -m <fixture> --out <out> [extra...]`; returns (out, exit_code).
fn run_export(extra: &[&str]) -> (PathBuf, i32) {
    let out = tmp_out();
    let status = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(fixture_model())
        .arg("export-html")
        .arg("--out").arg(&out)
        .args(extra)
        .status()
        .expect("spawn export-html");
    (out, status.code().unwrap_or(-1))
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// Element page path for a qualified name (scheme: "::" and "/" -> "__").
fn elem_page(out: &Path, qname: &str) -> PathBuf {
    let sanitized = qname.replace("::", "__").replace('/', "__");
    out.join("elements").join(format!("{sanitized}.html"))
}

/// Recursively collect generated files with the given extension.
fn files_with_ext(root: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(root).unwrap() {
        let e = e.unwrap();
        let p = e.path();
        if e.file_type().unwrap().is_dir() {
            files_with_ext(&p, ext, out);
        } else if p.extension().is_some_and(|x| x == ext) {
            out.push(p);
        }
    }
}

// ---- TC-TRS-HTML-001 --------------------------------------------------------

#[test]
fn site_is_generated() {
    let (out, code) = run_export(&[]);
    assert_eq!(code, 0, "export-html exits 0");
    assert!(out.join("index.html").exists(), "index.html written");
    assert!(elem_page(&out, "Requirements::REQ-FX-001").exists(), "an element page is written");
    assert!(out.join("elements").is_dir(), "elements/ directory exists");
}

// ---- TC-TRS-HTML-002 --------------------------------------------------------

#[test]
fn element_page_has_identity_and_rendered_doc() {
    let (out, _) = run_export(&[]);
    let page = read(&elem_page(&out, "Requirements::REQ-FX-001"));
    assert!(page.contains("REQ-FX-001"), "page shows the stable id");
    assert!(page.contains("Fixture requirement"), "page shows the name");
    // doc body: "...expose a stable element for the MCP read tools to retrieve and trace."
    assert!(page.contains("stable element"), "page shows rendered documentation");
}

// ---- TC-TRS-HTML-003 --------------------------------------------------------

#[test]
fn cross_reference_becomes_a_link() {
    let (out, _) = run_export(&[]);
    // Parts::Derived has `supertype: Parts::Base`.
    let page = read(&elem_page(&out, "Parts::Derived"));
    assert!(page.contains("Parts__Base.html"), "supertype is a link to the Base page; got no link");
}

#[test]
fn pages_carry_navigation() {
    let (out, _) = run_export(&[]);
    let page = read(&elem_page(&out, "Parts::Derived"));
    assert!(page.contains("sitenav"), "page carries the navigation container");
}

// ---- TC-TRS-HTML-004 --------------------------------------------------------

#[test]
fn svg_diagram_is_inlined() {
    let (out, _) = run_export(&[]);
    let page = read(&elem_page(&out, "Diagrams::FxBlock"));
    assert!(page.contains("<svg"), "BDD diagram is inlined as SVG");
}

#[test]
fn mermaid_diagram_is_offline() {
    let (out, _) = run_export(&[]);
    let page = read(&elem_page(&out, "Diagrams::FxMermaid"));
    assert!(page.contains("class=\"mermaid\""), "mermaid block present");
    assert!(page.contains("mermaid.min.js"), "page references the bundled mermaid script");
    assert!(out.join("mermaid.min.js").exists(), "mermaid.min.js bundled into the site");
}

// ---- TC-TRS-HTML-005 --------------------------------------------------------

#[test]
fn report_pages_exist() {
    let (out, _) = run_export(&[]);
    for r in ["validation.html", "coverage.html", "traceability.html"] {
        assert!(out.join("reports").join(r).exists(), "reports/{r} exists");
    }
    assert!(read(&out.join("reports/coverage.html")).to_lowercase().contains("coverage"), "coverage report has content");
}

// ---- TC-TRS-HTML-006 --------------------------------------------------------

#[test]
fn search_index_and_script_present() {
    let (out, _) = run_export(&[]);
    assert!(out.join("search.js").exists(), "search.js bundled");
    let idx = read(&out.join("search-index.json"));
    let v: serde_json::Value = serde_json::from_str(&idx).expect("search-index.json is valid JSON");
    let arr = v.as_array().or_else(|| v.get("elements").and_then(|e| e.as_array())).expect("index is an array");
    let found = arr.iter().any(|e| {
        e.get("qname").and_then(|q| q.as_str()) == Some("Requirements::REQ-FX-001")
            && e.get("url").is_some()
            && e.get("id").is_some()
    });
    assert!(found, "search index contains REQ-FX-001 with id and url");
}

// ---- TC-TRS-HTML-007 --------------------------------------------------------

#[test]
fn default_stylesheet_is_written_and_linked() {
    let (out, _) = run_export(&[]);
    assert!(out.join("style.css").exists(), "default style.css written");
    let page = read(&elem_page(&out, "Requirements::REQ-FX-001"));
    assert!(page.contains("style.css"), "page links style.css");
}

#[test]
fn custom_css_replaces_default() {
    let css = tmp_out();
    std::fs::create_dir_all(&css).unwrap();
    let css_file = css.join("custom.css");
    std::fs::write(&css_file, "/* CUSTOM_MARKER_XYZ */\nbody { color: red; }\n").unwrap();
    let (out, code) = run_export(&["--css", css_file.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(read(&out.join("style.css")).contains("CUSTOM_MARKER_XYZ"), "custom CSS becomes style.css");
}

#[test]
fn output_has_no_network_resource_loads() {
    let (out, _) = run_export(&[]);
    // Offline guarantee: no generated HTML/CSS loads a resource over the network.
    // (Inline SVG legitimately contains xmlns="http://www.w3.org/..."; that is not a load.)
    let mut files = Vec::new();
    files_with_ext(&out, "html", &mut files);
    files_with_ext(&out, "css", &mut files);
    let needles = ["src=\"http", "src='http", "href=\"http", "href='http", "url(http", "url(\"http", "url('http"];
    for f in &files {
        let content = read(f);
        for n in &needles {
            assert!(!content.contains(n), "{} contains a network resource load ({n})", f.display());
        }
    }
}

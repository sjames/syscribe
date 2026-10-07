//! `syscribe diagram export` (`REQ-TRS-VIS-009`, `REQ-TRS-VIS-010`).
//! Black-box: runs the binary against the checked-in fixture model
//! (`tests/fixtures/model`), whose `Diagrams::FxBlock` is an unpinned manifest
//! BDD (laid out by the embedded ELK, `REQ-TRS-VIS-016`), `Diagrams::FxPinned`
//! a fully pinned one and `Diagrams::FxEmpty` one with nothing to draw.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model")
}

fn tmp_dir() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("syscribe-diagram-export-{}-{}-{}", std::process::id(), nanos, n))
}

/// Run `syscribe -m <fixture> diagram <args…>`.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(fixture_model())
        .arg("diagram")
        .args(args)
        .output()
        .expect("spawn syscribe")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn unknown_element_is_an_error_with_nothing_on_stdout() {
    let o = run(&["export", "Nope::Missing", "--format", "mermaid"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(stdout(&o), "", "nothing on stdout");
    assert!(stderr(&o).contains("error: element 'Nope::Missing' not found"), "{}", stderr(&o));
}

#[test]
fn a_non_diagram_element_is_refused() {
    let o = run(&["export", "Parts::Base", "--format", "mermaid"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(stdout(&o), "");
    assert!(stderr(&o).contains("error: 'Parts::Base' is not a Diagram"), "{}", stderr(&o));
}

#[test]
fn mermaid_goes_to_stdout_with_a_ref_per_node() {
    let o = run(&["export", "Diagrams::FxBlock", "--format", "mermaid"]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let m = stdout(&o);
    assert!(m.starts_with("classDiagram\n"), "{m}");
    assert!(m.contains("%% ref: Parts::Base\n") && m.contains("%% ref: Parts::Derived\n"), "{m}");
    assert!(m.contains("<<part def>>"), "{m}");
    assert!(m.contains("s_base <|-- s_derived"), "inheritance: {m}");
    assert!(!m.contains("click "), "no [links] in the fixture, no click lines");
}

#[test]
fn plantuml_is_the_default_format() {
    let o = run(&["export", "Diagrams::FxBlock"]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let p = stdout(&o);
    assert!(p.starts_with("@startuml"), "{p}");
    assert!(p.contains("@enduml"));
}

#[test]
fn out_writes_the_file_and_creates_parents() {
    let dir = tmp_dir();
    let file = dir.join("nested/deeper/FxBlock.mmd");
    let o = run(&["export", "Diagrams::FxBlock", "--format", "mermaid", "--out", file.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    assert_eq!(stdout(&o), "", "with --out nothing goes to stdout");
    let written = std::fs::read_to_string(&file).expect("file written");
    assert!(written.starts_with("classDiagram\n"), "{written}");
}

#[test]
fn svg_lays_out_an_unpinned_diagram_with_the_embedded_elk() {
    // REQ-TRS-VIS-016: no pins, no browser — the executable lays it out.
    let o = run(&["export", "Diagrams::FxBlock", "--format", "svg"]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let s = stdout(&o);
    assert!(s.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:sysml=\"urn:syscribe:1.0\""), "{s}");
    assert!(s.contains("<g id=\"s-base\" class=\"block PartDef\" sysml:ref=\"Parts::Base\">"), "{s}");
    assert!(s.contains("<g id=\"s-derived\" class=\"block PartDef\" sysml:ref=\"Parts::Derived\">"), "{s}");
    assert!(s.contains("class=\"edge inheritance\" sysml:source=\"s-derived\" sysml:target=\"s-base\""), "{s}");
    assert!(s.contains("marker-end=\"url(#arrow-inherit)\""), "{s}");
    // Only a diagram with nothing to draw is refused.
    let o = run(&["export", "Diagrams::FxEmpty", "--format", "svg"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(stdout(&o), "");
    assert!(stderr(&o).contains("error: 'Diagrams::FxEmpty' has no shapes to draw"), "{}", stderr(&o));
}

#[test]
fn svg_draws_a_fully_pinned_diagram_with_sysml_attributes() {
    let o = run(&["export", "Diagrams::FxPinned", "--format", "svg"]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let s = stdout(&o);
    assert!(s.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:sysml=\"urn:syscribe:1.0\""), "{s}");
    assert!(s.contains("<g id=\"s-base\" class=\"block PartDef\" sysml:ref=\"Parts::Base\">"), "{s}");
    assert!(s.contains("<g id=\"s-derived\" class=\"block PartDef\" sysml:ref=\"Parts::Derived\">"), "{s}");
    assert!(s.contains("class=\"edge inheritance\" sysml:source=\"s-derived\" sysml:target=\"s-base\""), "{s}");
    assert!(s.contains("marker-end=\"url(#arrow-inherit)\""), "{s}");
    assert!(!s.contains("<a "), "no [links] configured, no hyperlink wrapper");
}

#[test]
fn bad_format_is_a_usage_error_naming_the_valid_values() {
    let o = run(&["export", "Diagrams::FxBlock", "--format", "png"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(stdout(&o), "");
    let e = stderr(&o);
    assert!(e.contains("invalid value 'png' for --format"), "{e}");
    assert!(e.contains("plantuml, mermaid, svg"), "{e}");
}

#[test]
fn an_unknown_option_is_rejected_before_the_model_loads() {
    let o = run(&["export", "Diagrams::FxBlock", "--bogus"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("unknown option '--bogus'"), "{}", stderr(&o));
}

#[test]
fn only_export_is_a_diagram_subcommand() {
    for sub in ["list", "render", "compose"] {
        let o = run(&[sub]);
        assert_eq!(o.status.code(), Some(1), "diagram {sub}");
        assert_eq!(stdout(&o), "");
        assert!(stderr(&o).contains(&format!("unrecognized subcommand '{sub}'")), "{}", stderr(&o));
    }
    let o = run(&[]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("Usage: syscribe -m <model> diagram export"), "{}", stderr(&o));
}

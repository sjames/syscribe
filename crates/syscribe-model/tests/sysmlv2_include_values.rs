//! `REQ-TRS-SYSMLV2-053` (parser version guard), `-054` (use-case `include`), `-055` (attribute
//! literal values and units) and `-059` (unresolved `satisfy`/`include` accounting).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::RawElement;
use syscribe_model::sysmlv2::export::export_sysml;
use syscribe_model::sysmlv2::report::{submodels, submodels_json, PARSER_AST_VERSION, PARSER_VERSION};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-incl-{}-{}",
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
    els.iter().find(|e| e.qualified_name == q).unwrap_or_else(|| panic!("missing {q}"))
}

fn w543(els: &[RawElement]) -> Vec<String> {
    validate(els).findings.iter().filter(|f| f.code == "W543").map(|f| f.message.clone()).collect()
}

// ── REQ-TRS-SYSMLV2-053 ─────────────────────────────────────────────────────

#[test]
fn reported_parser_version_equals_the_cargo_pin() {
    let manifest = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let pin = manifest
        .lines()
        .find_map(|l| l.trim().strip_prefix("sysml-v2-parser"))
        .and_then(|rest| rest.split('"').nth(1))
        .expect("sysml-v2-parser pin in Cargo.toml");
    assert_eq!(
        pin.trim_start_matches(['=', '^', '~']),
        PARSER_VERSION,
        "update `report::PARSER_VERSION` together with the pin in Cargo.toml"
    );
}

#[test]
fn report_json_carries_the_parser_block() {
    let els = load("package P { part def X; }\n");
    let v = submodels_json(&els);
    assert_eq!(v["parser"]["name"], "sysml-v2-parser");
    assert_eq!(v["parser"]["version"], PARSER_VERSION);
    assert_eq!(v["parser"]["astVersion"], PARSER_AST_VERSION);
}

// ── REQ-TRS-SYSMLV2-054 ─────────────────────────────────────────────────────

#[test]
fn include_resolves_to_a_use_case_in_scope_and_is_stored_in_includes() {
    let els = load("package P {\n use case def Pay;\n use case def Ship;\n use case def Checkout { include Pay; then include Ship; }\n}\n");
    let c = find(&els, "S::P::Checkout");
    assert_eq!(c.frontmatter.includes.as_deref(), Some(&["S::P::Pay".to_string(), "S::P::Ship".to_string()][..]));
    assert!(w543(&els).is_empty(), "{:?}", w543(&els));
}

#[test]
fn include_on_a_use_case_usage_resolves_too() {
    let els = load("package P {\n use case def Pay;\n use case order : Pay { include Pay; }\n}\n");
    assert_eq!(find(&els, "S::P::order").frontmatter.includes.as_deref(), Some(&["S::P::Pay".to_string()][..]));
}

#[test]
fn unresolved_include_is_dropped_and_counted_as_unmapped() {
    let els = load("package P {\n use case def Pay;\n part def Pay2;\n use case def Checkout { include Pay; include Nowhere; include Pay2; include Checkout; }\n}\n");
    let c = find(&els, "S::P::Checkout");
    assert_eq!(c.frontmatter.includes.as_deref(), Some(&["S::P::Pay".to_string()][..]));
    let msgs = w543(&els);
    assert_eq!(msgs.len(), 1, "{msgs:?}");
    assert!(msgs[0].contains("include x3"), "{msgs:?}");
    let sub = &submodels(&els)[0];
    assert_eq!(sub.unmapped.get("include"), Some(&3));
}

// ── REQ-TRS-SYSMLV2-055 ─────────────────────────────────────────────────────

#[test]
fn attribute_literal_values_and_units_map_onto_the_attribute() {
    let els = load(
        "package P {\n part def M {\n attribute mass : Real = 12.5 [kg];\n attribute count : Integer = 3;\n attribute label : String = \"hi\";\n attribute on : Boolean = true;\n attribute derived : Real = mass + 1;\n attribute plain : Real;\n }\n}\n",
    );
    let num = |q: &str| find(&els, q).frontmatter.value.as_ref().and_then(|v| v.as_f64());
    assert_eq!(num("S::P::M::mass"), Some(12.5));
    assert_eq!(find(&els, "S::P::M::mass").frontmatter.unit.as_deref(), Some("kg"));
    assert_eq!(find(&els, "S::P::M::count").frontmatter.value.as_ref().and_then(|v| v.as_i64()), Some(3));
    assert!(find(&els, "S::P::M::count").frontmatter.unit.is_none());
    assert_eq!(find(&els, "S::P::M::label").frontmatter.value.as_ref().and_then(|v| v.as_str()), Some("hi"));
    assert_eq!(find(&els, "S::P::M::on").frontmatter.value.as_ref().and_then(|v| v.as_bool()), Some(true));
    // A non-literal expression and a value-less attribute map no value.
    assert!(find(&els, "S::P::M::derived").frontmatter.value.is_none());
    assert!(find(&els, "S::P::M::plain").frontmatter.value.is_none());
    // `unit:` is a recognised field: no W047 for it.
    let findings = validate(&els).findings;
    assert!(!findings.iter().any(|f| f.code == "W047" && f.message.contains("unit")), "{findings:?}");
}

#[test]
fn attribute_value_and_unit_survive_export_and_re_ingestion() {
    let els = load("package P { part def M { attribute mass : Real = 12.5 [kg]; attribute n : Integer = 3; } }\n");
    let text = export_sysml(&els, None).unwrap().text;
    assert!(text.contains("attribute mass : Real = 12.5 [kg];"), "{text}");
    assert!(text.contains("attribute n : Integer = 3;"), "{text}");
    let back = load(&text);
    let m = find(&back, "S::S::P::M::mass");
    assert_eq!(m.frontmatter.value.as_ref().and_then(|v| v.as_f64()), Some(12.5));
    assert_eq!(m.frontmatter.unit.as_deref(), Some("kg"));
}

// ── REQ-TRS-SYSMLV2-059 ─────────────────────────────────────────────────────

#[test]
fn report_counts_unresolved_package_level_satisfy_like_w543() {
    let els = load("package P {\n part def X;\n satisfy 'REQ-1' by Missing;\n satisfy 'REQ-2' by X;\n}\n");
    let sub = &submodels(&els)[0];
    assert_eq!(sub.unmapped.get("satisfy"), Some(&1), "{:?}", sub.unmapped);
    assert_eq!(sub.unmapped_total(), 1);
    let msgs = w543(&els);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].contains("satisfy x1"), "{msgs:?}");
    // The report's per-file count equals the advisory's.
    assert_eq!(sub.files[0].unmapped.get("satisfy"), Some(&1));
    assert!(sub.files[0].parsed);
}

#[test]
fn report_marks_unparsable_files_not_parsed() {
    let els = load("package P { part def ;;; broken {{{");
    let sub = &submodels(&els)[0];
    assert_eq!(sub.files_parsed(), 0);
    assert!(!sub.files[0].parsed);
}

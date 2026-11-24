//! GH #531 — table views for documents: `REQ-TRS-VIEWTAB-001` (the declaration: columns, rows, where,
//! sort, nest, marks), `-002` (`view render --format latex|markdown|csv|html`), `-003` (`validate`:
//! `E507` malformed, `W504` unknown field).

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct M {
    root: PathBuf,
}

const OPEN_QUESTIONS: &str = "expose: [{target: \"Planning::**\", filter: \"type == PlanningItem and tag == external-input\"}]\nrendering: asTableView\nrows: questions\nwhere: \"status == open\"\ncolumns:\n  - {field: ref, heading: Question}\n  - {field: text, heading: Text, width: 6cm}\n  - {field: recommended, heading: Recommendation, empty: \"—\"}\n  - {tagPrefix: phase-, heading: Phase, align: right}\nsort: [ref]\ncaption: \"Open questions\"\n";
const ITEMS: &str = "expose: [\"Planning::*\"]\ncolumns: [ref, name, status, openQuestions]\nnest: parent\nmarks:\n  - {when: \"tag == placeholder\", symbol: \"†\", note: \"Placeholder item\"}\ncaption: Planning items\n";
const PARAMS: &str = "expose: [\"Features::*\"]\nrows: parameters\ncolumns:\n  - {field: key, heading: Parameter}\n  - {field: quantity, heading: Value, align: right}\n  - range\nsort: [-name]\n";

impl M {
    fn new() -> M {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-viewtab-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
        let m = M { root };
        m.w("_index.md", "---\ntype: Package\nname: Root\n---\n");
        for p in ["Views", "Planning", "Features", "Configs"] {
            m.w(&format!("{p}/_index.md"), &format!("---\ntype: Package\nname: {p}\n---\n"));
        }
        m.w("Features/Limits.md", "---\ntype: FeatureDef\nid: FEAT-LIMITS\nname: Limits\nmandatory: true\nparameters:\n  - { name: maxTemp, type: ScalarValues::Integer, default: 120, unit: degC, range: \"0..200\" }\n  - { name: onTimeMs, type: ScalarValues::Integer, unit: ms, derivedFrom: \"Features::Limits.maxTemp * 2\" }\n---\n");
        m.w("Configs/CONF-HTR-001.md", "---\ntype: Configuration\nid: CONF-HTR-001\nname: Hot\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Limits: true\nparameterBindings:\n  Features::Limits.maxTemp: 130\n---\n");
        m.w("Planning/PI-HTR-001.md", "---\nid: PI-HTR-001\ntype: PlanningItem\nname: \"Confirm the limit & sensor_type\"\nstatus: in_progress\nitemType: feature\ntags: [external-input, phase-2]\nachieves: [REQ-HTR-001]\nquestions:\n  - id: Q1\n    text: Which sensor?\n    recommended: PT100\n    status: open\n  - id: Q2\n    text: Sample rate\n    status: answered\n    answer: \"10 Hz\"\n  - id: Q10\n    text: \"Cost, per unit\"\n    status: open\n---\n");
        m.w("Planning/PI-HTR-002.md", "---\nid: PI-HTR-002\ntype: PlanningItem\nname: \"Order sensors\"\nstatus: todo\nitemType: task\nparent: PI-HTR-001\ntags: [placeholder]\nquestions:\n  - id: Q1\n    text: Supplier?\n    status: open\n---\n");
        m.w("Planning/PI-HTR-003.md", "---\nid: PI-HTR-003\ntype: PlanningItem\nname: \"Another top item\"\nstatus: todo\nitemType: feature\nachieves: [REQ-HTR-001]\n---\n");
        m.w("REQ-HTR-001.md", "---\nid: REQ-HTR-001\ntype: Requirement\nname: \"Temperature limit\"\nstatus: draft\nreqDomain: system\nreqClass: system\n---\n\nThe heater shall hold its limit.\n");
        m.view("OpenQuestions", OPEN_QUESTIONS);
        m.view("Items", ITEMS);
        m.view("Params", PARAMS);
        m.view("Plain", "expose: [\"Planning::*\"]\nrendering: asTableView\n");
        m
    }
    fn view(&self, name: &str, body: &str) {
        self.w(&format!("Views/{name}.md"), &format!("---\ntype: View\nname: {name}\n{body}---\n"));
    }
    fn w(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&self.root).args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned(), o.status.code().unwrap_or(-1))
    }
    fn render(&self, view: &str, format: &str) -> String {
        let (out, err, code) = self.run(&["view", "render", &format!("Views::{view}"), "--format", format]);
        assert_eq!(code, 0, "{out}{err}");
        out
    }
}

// ── REQ-TRS-VIEWTAB-001: the declaration ─────────────────────────────────────

#[test]
fn rows_from_a_list_are_filtered_sorted_and_shown_in_the_declared_columns() {
    let m = M::new();
    let out = m.render("OpenQuestions", "csv");
    // One row per open question of the item tagged external-input; Q2 is answered, PI-HTR-002 is
    // not tagged; Q1 sorts before Q10; an empty recommendation shows the placeholder.
    assert_eq!(out, "Question,Text,Recommendation,Phase\r\nPI-HTR-001#Q1,Which sensor?,PT100,2\r\nPI-HTR-001#Q10,\"Cost, per unit\",—,2\r\n");
}

#[test]
fn a_view_with_columns_prints_as_a_table_by_default() {
    let m = M::new();
    let (out, _, code) = m.run(&["view", "render", "Views::Items"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("| Ref | Name | Status | Open questions |\n|---|---|---|---|\n"), "{out}");
    // A view without columns keeps its table.
    let (out, _, _) = m.run(&["view", "render", "Views::Plain"]);
    assert!(out.starts_with("| Element | Type | Name |\n|---|---|---|\n| Planning::PI-HTR-001 | PlanningItem |"), "{out}");
}

#[test]
fn nest_puts_children_under_their_parent_and_marks_flag_rows() {
    let m = M::new();
    let out = m.render("Items", "markdown");
    let rows: Vec<&str> = out.lines().filter(|l| l.starts_with("| ") || l.starts_with("| &nbsp;")).skip(1).collect();
    assert_eq!(rows.len(), 3, "{out}");
    assert!(rows[0].starts_with("| PI-HTR-001 |") && rows[0].ends_with("| 2 |"), "{out}");
    assert!(rows[1].starts_with("| &nbsp;&nbsp;PI-HTR-002† | Order sensors | todo | 1 |"), "{out}");
    assert!(rows[2].starts_with("| PI-HTR-003 |"), "{out}");
    assert!(out.starts_with("**Planning items**\n") && out.contains("\n† Placeholder item\n"), "{out}");
}

#[test]
fn parameter_rows_carry_effective_values_per_configuration() {
    let m = M::new();
    assert_eq!(m.render("Params", "csv"), "Parameter,Value,Range\r\nFeatures::Limits.onTimeMs,240 ms,\r\nFeatures::Limits.maxTemp,120 degC,0..200\r\n");
    let (out, err, code) = m.run(&["view", "render", "Views::Params", "--format", "csv", "--config", "CONF-HTR-001"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("Features::Limits.onTimeMs,260 ms,") && out.contains("Features::Limits.maxTemp,130 degC,0..200"), "{out}");
}

#[test]
fn an_expose_filter_compares_fields() {
    let m = M::new();
    m.view("Todo", "expose: [{target: \"Planning::*\", filter: \"status == todo and not tag == placeholder\"}]\ncolumns: [ref]\n");
    assert_eq!(m.render("Todo", "csv"), "Ref\r\nPI-HTR-003\r\n");
    m.view("ByCond", "expose: [\"Planning::*\"]\nfilterCondition: \"status != todo\"\ncolumns: [ref]\n");
    assert_eq!(m.render("ByCond", "csv"), "Ref\r\nPI-HTR-001\r\n");
}

#[test]
fn element_fields_computed_fields_and_element_prefix_resolve() {
    let m = M::new();
    m.w("TC-HTR-001.md", "---\nid: TC-HTR-001\ntype: TestCase\ntestLevel: L4\nstatus: active\nname: T\nverifies: [REQ-HTR-001]\n---\n\n```gherkin\nFeature: F\n  Scenario: S\n    Given x\n```\n");
    m.view("Reqs", "expose: [REQ-HTR-001]\ncolumns: [ref, type, summary, verifiedBy, reqDomain]\n");
    assert_eq!(m.render("Reqs", "csv"), "Ref,Type,Summary,Verified by,Req domain\r\nREQ-HTR-001,Requirement,The heater shall hold its limit.,TC-HTR-001,system\r\n");
    m.view("Q", "expose: [\"Planning::*\"]\nrows: questions\ncolumns: [ref, element.name, status]\nwhere: \"element.status == todo\"\n");
    assert_eq!(m.render("Q", "csv"), "Ref,Name,Status\r\nPI-HTR-002#Q1,Order sensors,open\r\n");
}

// ── REQ-TRS-VIEWTAB-002: document formats ────────────────────────────────────

#[test]
fn latex_is_a_longtable_with_caption_label_and_escaping() {
    let m = M::new();
    let out = m.render("OpenQuestions", "latex");
    assert!(out.starts_with("\\begin{longtable}{lp{6cm}lr}\n\\caption{Open questions}\\label{tab:open-questions}\\\\\n\\toprule\nQuestion & Text & Recommendation & Phase \\\\\n\\midrule\n\\endfirsthead\n"), "{out}");
    assert!(out.contains("\\endhead\n") && out.contains("\\bottomrule\n\\endlastfoot\n"), "{out}");
    assert!(out.contains("PI-HTR-001\\#Q1 & Which sensor? & PT100 & 2 \\\\\n"), "{out}");
    assert!(out.ends_with("\\end{longtable}\n"));
    let items = m.render("Items", "latex");
    assert!(items.contains("PI-HTR-001 & Confirm the limit \\& sensor\\_type & in\\_progress & 2 \\\\\n"), "{items}");
    assert!(items.contains("\\hspace{1em}PI-HTR-002\\textsuperscript{†} & Order sensors"), "{items}");
    assert!(items.contains("\\multicolumn{4}{l}{\\footnotesize \\textsuperscript{†} Placeholder item} \\\\\n\\endlastfoot"), "{items}");
    // A quantity is set the LaTeX way.
    assert!(m.render("Params", "latex").contains("Features::Limits.maxTemp & 120\\,\\textdegree{}C & 0..200 \\\\"));
}

#[test]
fn markdown_html_and_json_carry_the_same_table() {
    let m = M::new();
    let md = m.render("OpenQuestions", "markdown");
    assert_eq!(md, "**Open questions**\n\n| Question | Text | Recommendation | Phase |\n|---|---|---|---:|\n| PI-HTR-001#Q1 | Which sensor? | PT100 | 2 |\n| PI-HTR-001#Q10 | Cost, per unit | — | 2 |\n");
    let html = m.render("OpenQuestions", "html");
    assert!(html.starts_with("<table id=\"tab:open-questions\">\n<caption>Open questions</caption>\n<thead><tr><th>Question</th>"), "{html}");
    assert!(html.contains("<tr><td>PI-HTR-001#Q1</td><td>Which sensor?</td><td>PT100</td><td style=\"text-align:right\">2</td></tr>"), "{html}");
    assert!(m.render("Items", "html").contains("<td>Confirm the limit &amp; sensor_type</td>"));
    let v: serde_json::Value = serde_json::from_str(&m.render("Params", "json")).unwrap();
    assert_eq!(v["view"], "Views::Params");
    assert_eq!(v["columns"][1], serde_json::json!({"field": "quantity", "heading": "Value"}));
    assert_eq!(v["rows"][1]["cells"][1], serde_json::json!({"value": "120", "unit": "degC"}));
}

#[test]
fn a_view_without_columns_prints_default_columns_in_a_document_format() {
    let m = M::new();
    let out = m.render("Plain", "csv");
    assert!(out.starts_with("Element,Type,Name\r\nPlanning::PI-HTR-001,PlanningItem,Confirm the limit & sensor_type\r\n"), "{out}");
    let json = m.render("Plain", "json");
    assert!(json.contains("\"elements\""), "a view without columns keeps its JSON shape: {json}");
}

#[test]
fn out_writes_a_file_and_an_unknown_format_is_a_usage_error() {
    let m = M::new();
    let dest = m.root.join("../build/q.tex");
    let (out, err, code) = m.run(&["view", "render", "Views::OpenQuestions", "--format", "latex", "--out", dest.to_str().unwrap()]);
    assert_eq!((out.as_str(), code), ("", 0), "{err}");
    assert!(std::fs::read_to_string(&dest).unwrap().starts_with("\\begin{longtable}"));
    let (_, err, code) = m.run(&["view", "render", "Views::OpenQuestions", "--format", "pdf"]);
    assert!(code == 2 && err.contains("latex, markdown, csv, html"), "{err}");
}

// ── REQ-TRS-VIEWTAB-003: validation ──────────────────────────────────────────

#[test]
fn a_well_formed_table_view_validates_without_w047_e507_or_w504() {
    let m = M::new();
    let (out, _, _) = m.run(&["validate"]);
    for code in ["W047", "E507", "W504"] {
        assert!(!out.lines().any(|l| l.contains(code) && l.contains("Views/")), "{code}: {out}");
    }
    // The keys are schema fields on a View only.
    m.w("Stray.md", "---\ntype: PartDef\nname: Stray\ncolumns: [a]\n---\n\nx\n");
    let (out, _, _) = m.run(&["validate"]);
    assert!(out.lines().any(|l| l.contains("W047") && l.contains("Stray.md") && l.contains("columns")), "{out}");
}

#[test]
fn a_column_naming_an_unknown_field_is_w504() {
    let m = M::new();
    m.view("Bad", "expose: [\"Planning::*\"]\ncolumns: [ref, statsu, custom.anything]\nsort: [nope]\nwhere: \"wat == 1\"\n");
    m.view("BadRows", "expose: [\"Planning::*\"]\nrows: questions\ncolumns: [ref, textt, element.name]\n");
    m.view("NoList", "expose: [\"Planning::*\"]\nrows: quesions\ncolumns: [ref]\n");
    let (out, _, _) = m.run(&["validate"]);
    let w: Vec<&str> = out.lines().filter(|l| l.contains("W504")).collect();
    let has = |file: &str, what: &str| w.iter().any(|l| l.contains(file) && l.contains(what));
    assert!(has("Bad.md", "columns[1] names unknown field `statsu`"), "{out}");
    assert!(has("Bad.md", "`sort:` names unknown field `nope`") && has("Bad.md", "`where:` names unknown field `wat`"), "{out}");
    assert!(!w.iter().any(|l| l.contains("custom.anything") || l.contains("`ref`")), "{out}");
    assert!(has("BadRows.md", "columns[1] names unknown field `textt`") && !w.iter().any(|l| l.contains("element.name")), "{out}");
    assert!(has("NoList.md", "`rows: quesions` names a list that none"), "{out}");
    let (explain, _, code) = m.run(&["explain", "W504"]);
    assert!(code == 0 && explain.contains("W504"), "{explain}");
}

#[test]
fn a_malformed_declaration_is_e507_and_render_refuses_it() {
    let m = M::new();
    m.view("Broken", "expose: [\"Planning::*\"]\ncolumns: [{field: ref, colour: red}]\nsort: 3\n");
    m.view("NoCols", "expose: [\"Planning::*\"]\ncaption: Nothing\n");
    let (out, _, code) = m.run(&["validate"]);
    assert_ne!(code, 0);
    assert!(out.lines().any(|l| l.contains("E507") && l.contains("Broken.md") && l.contains("no key `colour`")), "{out}");
    assert!(out.lines().any(|l| l.contains("E507") && l.contains("Broken.md") && l.contains("`sort:` must be")), "{out}");
    assert!(out.lines().any(|l| l.contains("E507") && l.contains("NoCols.md") && l.contains("needs `columns:`")), "{out}");
    let (_, err, code) = m.run(&["view", "render", "Views::Broken", "--format", "latex"]);
    assert!(code == 1 && err.contains("E507"), "{err}");
    let (explain, _, code) = m.run(&["explain", "E507"]);
    assert!(code == 0 && explain.contains("E507"), "{explain}");
}

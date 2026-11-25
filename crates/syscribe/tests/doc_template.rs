//! GH #532 — documents from templates: `REQ-TRS-DOCGEN-001` (`render --template`: the engine, its
//! helpers and strict mode), `-002` (the `Document` element and its validation), `-003` (`doc build`
//! and `doc build --check`).

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct M {
    dir: PathBuf,
}

const DOC: &str = "---\ntype: Document\nname: Report\ntemplate: model:../doc/report.tex.tmpl\nrendered: model:../doc/build/report.tex\nartefacts: [model:../doc/build/report.pdf]\nversion: \"0.2\"\naudience: customer\n---\n\nThe report.\n";

const TEMPLATE: &str = "<# a comment #>\n\\title{<< document.name >> v<< document.version >>}\nMax << value(\"Limits.maxTemp\") >>, bare << value(\"Limits.maxTemp\", bare=true) >>, settle << value(\"Rig.settle\") >>.\n<< view(\"Views::Params\") >>\n<% for r in elements(type=\"Requirement\", where=\"status != draft\") %>\n\\subsection{<< r.id >>: << r.name >>}\n<< r | body >>\n<% endfor %>\n";

impl M {
    fn new() -> M {
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!("syscribe-docgen-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let m = M { dir };
        m.w("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
        for p in ["Features", "Configs", "Views", "Docs"] {
            m.w(&format!("model/{p}/_index.md"), &format!("---\ntype: Package\nname: {p}\n---\n"));
        }
        m.w("model/Features/Limits.md", "---\ntype: FeatureDef\nid: FEAT-LIMITS\nname: Limits\nmandatory: true\nparameters:\n  - { name: maxTemp, type: ScalarValues::Integer, default: 120, unit: degC }\n---\n");
        m.w("model/Features/Extra.md", "---\ntype: FeatureDef\nid: FEAT-EXTRA\nname: Extra\ngroupKind: optional\n---\n");
        m.w("model/Configs/CONF-HTR-001.md", "---\ntype: Configuration\nid: CONF-HTR-001\nname: Hot\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Limits: true\n  Features::Extra: true\nparameterBindings:\n  Features::Limits.maxTemp: 130\n---\n");
        m.w("model/Rig.md", "---\ntype: PartDef\nname: Rig\ndomain: system\nsatisfies: [REQ-HTR-001]\nfeatures:\n  - name: settle\n    value: 150\n    unit: ms\n---\n\nThe rig.\n");
        m.w("model/REQ-HTR-001.md", "---\nid: REQ-HTR-001\ntype: Requirement\nname: \"Limit & margin\"\nstatus: approved\nreqDomain: system\nreqClass: system\n---\n\nThe heater shall settle within {{Rig.settle}}, with **no** overshoot:\n\n- at `cold`\n- at 100% load\n");
        m.w("model/REQ-HTR-002.md", "---\nid: REQ-HTR-002\ntype: Requirement\nname: \"Draft one\"\nstatus: draft\nreqDomain: system\nreqClass: system\n---\n\nNot yet.\n");
        m.w("model/Views/Params.md", "---\ntype: View\nname: Params\nexpose: [\"Features::*\"]\nrows: parameters\ncolumns:\n  - {field: key, heading: Parameter}\n  - {field: quantity, heading: Value}\ncaption: Parameters\n---\n");
        m.w("model/Docs/Report.md", DOC);
        m.w("doc/report.tex.tmpl", TEMPLATE);
        m
    }
    fn w(&self, rel: &str, c: &str) {
        let p = self.dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).current_dir(&self.dir).args(["-m", "model"]).args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned(), o.status.code().unwrap_or(-1))
    }
    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.dir.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
    }
    fn exists(&self, rel: &str) -> bool {
        self.dir.join(rel).exists()
    }
    /// Render a template given as text, as a file of the given name.
    fn tmpl(&self, name: &str, text: &str, extra: &[&str]) -> (String, String, i32) {
        self.w(&format!("doc/{name}"), text);
        let path = format!("doc/{name}");
        let mut a = vec!["render", "--template", path.as_str()];
        a.extend(extra);
        self.run(&a)
    }
}

// ── REQ-TRS-DOCGEN-001: the engine ───────────────────────────────────────────

#[test]
fn a_latex_template_gets_values_a_table_and_element_text() {
    let m = M::new();
    let (out, err, code) = m.tmpl("t.tex.tmpl", "Max << value(\"Limits.maxTemp\") >>, bare << value(\"Limits.maxTemp\", bare=true) >>, settle << value(\"Rig.settle\") >>.\n<< view(\"Views::Params\") >>\n<% for r in elements(type=\"Requirement\", where=\"status != draft\") %>\n\\subsection{<< r.id >>: << r.name >>}\n<< r | body >>\n<% endfor %>\n", &[]);
    assert_eq!(code, 0, "{err}");
    assert!(out.starts_with("Max 120\\,\\textdegree{}C, bare 120, settle 150\\,ms.\n\\begin{longtable}{ll}\n\\caption{Parameters}"), "{out}");
    assert!(out.contains("Features::Limits.maxTemp & 120\\,\\textdegree{}C \\\\"), "{out}");
    // Printed text is escaped for LaTeX; the body is converted from Markdown with its value reference rendered.
    assert!(out.contains("\\subsection{REQ-HTR-001: Limit \\& margin}\n"), "{out}");
    assert!(out.contains("The heater shall settle within 150 ms, with \\textbf{no} overshoot:"), "{out}");
    assert!(out.contains("\\begin{itemize}\n\\item at \\texttt{cold}\n\\item at 100\\% load\n\\end{itemize}"), "{out}");
    assert!(!out.contains("REQ-HTR-002"), "the draft requirement is filtered out: {out}");
}

#[test]
fn a_configuration_gives_its_values_and_selected_features() {
    let m = M::new();
    let t = "<< value(\"Limits.maxTemp\") >> << meta.config >> <% if selected(\"FEAT-EXTRA\") %>extra<% endif %> << parameters | length >>\n";
    let (out, err, code) = m.tmpl("c.tex.tmpl", t, &["--config", "CONF-HTR-001"]);
    assert_eq!((out.as_str(), code), ("130\\,\\textdegree{}C CONF-HTR-001 extra 1\n", 0), "{err}");
    let (_, err, code) = m.tmpl("c.tex.tmpl", t, &[]);
    assert!(code == 1 && err.contains("selected(\"FEAT-EXTRA\") needs a Configuration"), "{err}");
}

#[test]
fn other_targets_use_jinja_delimiters_and_their_own_escaping() {
    let m = M::new();
    let t = "{# c #}{{ value(\"Rig.settle\") }} {{ element(\"REQ-HTR-001\").name }} {{ \"a<b\" | safe }}\n{{ view(\"Views::Params\") }}";
    let (out, err, code) = m.tmpl("p.html.j2", t, &[]);
    assert_eq!(code, 0, "{err}");
    assert!(out.starts_with("150&#8239;ms Limit &amp; margin a<b\n<table id=\"tab:params\">"), "{out}");
    let (out, _, _) = m.tmpl("p.md.tmpl", t, &[]);
    assert!(out.starts_with("150 ms Limit & margin a<b\n**Parameters**\n\n| Parameter | Value |"), "{out}");
    // The element's body, per target.
    let (out, _, _) = m.tmpl("b.html.tmpl", "{{ element(\"REQ-HTR-001\") | body }}", &[]);
    assert!(out.contains("<strong>no</strong>") && out.contains("<li>at <code>cold</code></li>"), "{out}");
    // --escape overrides the file name; a view can be asked for in another format, or as data.
    let (out, _, _) = m.tmpl("p.md.tmpl", "{{ value(\"Rig.settle\") }} {% for r in view(\"Views::Params\", format=\"data\").rows %}{{ r.cells[0] }}={{ r.cells[1] | quantity }}{% endfor %}", &["--escape", "latex"]);
    assert_eq!(out, "150\\,ms Features::Limits.maxTemp=120\\,\\textdegree{}C");
}

#[test]
fn an_unknown_name_stops_the_render_with_the_template_line() {
    let m = M::new();
    for (text, want) in [
        ("ok\n<< value(\"Rig.setle\") >>\n", "Rig has no attribute `setle`"),
        ("ok\n\n<< element(\"REQ-GONE-001\").name >>\n", "element `REQ-GONE-001` does not resolve"),
        ("<< view(\"Views::Nope\") >>\n", "element `Views::Nope` does not resolve"),
        ("<< view(\"Rig\") >>\n", "is not a View"),
        ("<< nothing >>\n", "undefined"),
        ("<< element(\"REQ-HTR-001\").nope >>\n", "undefined"),
        ("<% for r in elements(type=\"Requirment\") %><% endfor %>\n", "no such element type"),
        ("<< value(\"Rig.settle\", bear=true) >>\n", "has no argument `bear`"),
    ] {
        let (out, err, code) = m.tmpl("e.tex.tmpl", text, &["--out", "build/e.tex"]);
        assert_eq!(code, 1, "{text}: {out}{err}");
        let line = text.lines().position(|l| l.contains("<<") || l.contains("<%")).unwrap() + 1;
        assert!(err.contains(&format!("doc/e.tex.tmpl:{line}: ")) && err.contains(want), "{text}: {err}");
        assert!(!m.exists("build/e.tex"), "nothing is written: {text}");
    }
}

#[test]
fn out_writes_the_file_and_a_diagram_is_exported_beside_it() {
    let m = M::new();
    m.w("model/Ctx.md", "---\ntype: Diagram\nname: Ctx\ndiagramKind: BDD\nsubject: Rig\n---\n\nDerived.\n");
    let (out, err, code) = m.tmpl("d.tex.tmpl", "\\input{<< diagram(\"Ctx\", format=\"mermaid\") >>}\n", &["--out", "build/d.tex"]);
    assert_eq!((out.as_str(), code), ("", 0), "{err}");
    assert_eq!(m.read("build/d.tex"), "\\input{Ctx.mmd}\n");
    assert!(m.read("build/Ctx.mmd").contains("Rig"));
    let (_, err, code) = m.tmpl("d.tex.tmpl", "<< diagram(\"Ctx\", format=\"pdf\") >>", &[]);
    assert!(code == 1 && err.contains("one of svg, mermaid, plantuml, dot"), "{err}");
}

#[test]
fn template_delimiters_are_configurable_and_includes_read_beside_the_template() {
    let m = M::new();
    m.w("model/.syscribe.toml", "[documents.templateSyntax.typ]\nvariable = [\"<<\", \">>\"]\nblock = [\"<%\", \"%>\"]\ncomment = [\"<#\", \"#>\"]\n");
    m.w("doc/part.typ", "part << value(\"Rig.settle\") >>");
    let (out, err, code) = m.tmpl("x.typ.tmpl", "{{ kept }} << value(\"Rig.settle\") >> <% include \"part.typ\" %>\n", &[]);
    assert_eq!((out.as_str(), code), ("{{ kept }} 150~ms part 150~ms", 0), "{err}");
}

// ── REQ-TRS-DOCGEN-002: the Document element ─────────────────────────────────

#[test]
fn a_document_element_validates_and_is_listed() {
    let m = M::new();
    let (out, _, _) = m.run(&["validate"]);
    assert!(!out.lines().any(|l| l.contains("Docs/Report.md") && (l.contains("W047") || l.contains("E508") || l.contains("W505"))), "{out}");
    let (out, _, code) = m.run(&["doc", "list"]);
    assert!(code == 0 && out.contains("| Docs::Report | 0.2 | model:../doc/report.tex.tmpl | doc/build/report.tex | model:../doc/build/report.pdf |"), "{out}");
    let (json, _, _) = m.run(&["doc", "list", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["documents"][0]["audience"], "customer");
    let (t, _, code) = m.run(&["template", "Document"]);
    assert!(code == 0 && t.contains("type: Document") && t.contains("template:"), "{t}");
    let (h, _, code) = m.run(&["help", "Document"]);
    assert!(code == 0 && h.contains("Document"), "{h}");
}

#[test]
fn a_malformed_document_is_e508_and_a_missing_template_w505() {
    let m = M::new();
    m.w("model/Docs/NoTemplate.md", "---\ntype: Document\nname: NoTemplate\nartefacts: 3\nconfig: CONF-NOPE-001\n---\n\nx\n");
    m.w("model/Docs/Missing.md", "---\ntype: Document\nname: Missing\ntemplate: model:../doc/nope.tmpl\n---\n\nx\n");
    m.w("model/Docs/Cfg.md", "---\ntype: Document\nname: Cfg\ntemplate: model:../doc/report.tex.tmpl\nconfig: CONF-NOPE-001\n---\n\nx\n");
    m.w("model/Stray.md", "---\ntype: PartDef\nname: Stray\ntemplate: x\n---\n\nx\n");
    let (out, _, code) = m.run(&["validate"]);
    assert_ne!(code, 0);
    let has = |file: &str, code: &str, what: &str| out.lines().any(|l| l.contains(file) && l.contains(code) && l.contains(what));
    assert!(has("NoTemplate.md", "E508", "needs `template:`") && has("NoTemplate.md", "E508", "`artefacts:` must be"), "{out}");
    assert!(has("Missing.md", "W505", "does not exist"), "{out}");
    assert!(has("Cfg.md", "E508", "does not resolve to a Configuration"), "{out}");
    assert!(has("Stray.md", "W047", "template"), "{out}");
    for c in ["E508", "W505"] {
        let (e, _, code) = m.run(&["explain", c]);
        assert!(code == 0 && e.contains(c), "{e}");
    }
}

// ── REQ-TRS-DOCGEN-003: doc build ────────────────────────────────────────────

#[test]
fn doc_build_renders_the_document_and_check_passes_until_the_model_changes() {
    let m = M::new();
    let (out, err, code) = m.run(&["doc", "build", "Report"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("Docs::Report: wrote doc/build/report.tex"), "{out}");
    let tex = m.read("doc/build/report.tex");
    assert!(tex.starts_with("\\title{Report v0.2}\nMax 120\\,\\textdegree{}C, bare 120, settle 150\\,ms.\n\\begin{longtable}"), "{tex}");
    let manifest: serde_json::Value = serde_json::from_str(&m.read("doc/build/report.tex.manifest.json")).unwrap();
    assert_eq!(manifest["document"], "Docs::Report");
    assert!(manifest["calls"].as_array().unwrap().iter().any(|c| c["fn"] == "value" && c["arg"] == "Rig.settle"), "{manifest}");

    // The artefact is not there yet: that alone is reported.
    let (out, _, code) = m.run(&["doc", "build", "Report", "--check"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("Docs::Report: out of date") && out.contains("the artefact doc/build/report.pdf is missing") && out.matches("\n  - ").count() == 1, "{out}");
    m.w("doc/build/report.pdf", "pdf");
    let (out, _, code) = m.run(&["doc", "build", "--all", "--check"]);
    assert_eq!((out.as_str(), code), ("Docs::Report: up to date\n", 0));

    // A value the template used changes.
    m.w("model/Rig.md", "---\ntype: PartDef\nname: Rig\ndomain: system\nsatisfies: [REQ-HTR-001]\nfeatures:\n  - name: settle\n    value: 200\n    unit: ms\n---\n\nThe rig.\n");
    let (out, _, code) = m.run(&["doc", "build", "Report", "--check"]);
    assert_eq!(code, 2);
    assert!(out.contains("value(\"Rig.settle\") is now 200\\,ms"), "{out}");
    assert!(out.contains("elements(type=\"Requirement\", where=\"status != draft\") changed"), "the requirement body renders the value: {out}");
    assert!(!out.contains("Limits.maxTemp"), "an unchanged value is not reported: {out}");
    // Rebuilding brings it up to date again (the artefact is then the older file).
    let (_, _, code) = m.run(&["doc", "build", "Report"]);
    assert_eq!(code, 0);
    assert!(m.read("doc/build/report.tex").contains("settle 200\\,ms"));
}

#[test]
fn check_reports_a_changed_template_a_hand_edit_a_missing_build_and_a_renamed_element() {
    let m = M::new();
    let (out, _, code) = m.run(&["doc", "build", "Report", "--check"]);
    assert!(code == 2 && out.contains("never built"), "{out}");
    m.run(&["doc", "build", "Report"]);
    m.w("doc/build/report.pdf", "pdf");
    m.w("doc/build/report.tex", "edited by hand\n");
    m.w("doc/report.tex.tmpl", &format!("{TEMPLATE}more\n"));
    std::fs::remove_file(m.dir.join("model/Views/Params.md")).unwrap();
    let (out, _, code) = m.run(&["doc", "build", "Report", "--check"]);
    assert_eq!(code, 2);
    assert!(out.contains("was changed after it was built") && out.contains("the template doc/report.tex.tmpl changed"), "{out}");
    assert!(out.contains("view(\"Views::Params\") no longer resolves"), "{out}");
    // Building now fails on the template line that names the removed view, and writes nothing new.
    let (_, err, code) = m.run(&["doc", "build", "Report"]);
    assert!(code == 1 && err.contains("doc/report.tex.tmpl:4: ") && err.contains("Views::Params"), "{err}");
    assert_eq!(m.read("doc/build/report.tex"), "edited by hand\n");
}

#[test]
fn a_document_for_a_configuration_builds_that_variant() {
    let m = M::new();
    m.w("model/Docs/Report.md", &DOC.replace("audience: customer\n", "audience: customer\nconfig: CONF-HTR-001\n"));
    let (_, err, code) = m.run(&["doc", "build", "Report"]);
    assert_eq!(code, 0, "{err}");
    assert!(m.read("doc/build/report.tex").contains("Max 130\\,\\textdegree{}C"));
    let (out, _, _) = m.run(&["doc", "build", "Report", "--check"]);
    assert!(!out.contains("built for"), "{out}");
    // Another Document type, an unknown name and a missing argument are errors.
    let (_, err, code) = m.run(&["doc", "build", "Rig"]);
    assert!(code == 1 && err.contains("is a PartDef, not a Document"), "{err}");
    let (_, err, code) = m.run(&["doc", "build", "Nope"]);
    assert!(code == 1 && err.contains("does not resolve"), "{err}");
    assert_eq!(m.run(&["doc", "build"]).2, 1);
    assert_eq!(m.run(&["doc"]).2, 1);
}

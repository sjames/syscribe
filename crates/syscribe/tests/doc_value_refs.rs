//! GH #530 — value references in external, non-Markdown documents:
//! `REQ-TRS-DOCREF-001` (`render <file>`), `-002` (`lint-docs` on other text files), `-003` (`refs` lists
//! the uses in the documents `[documents]` names).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct M {
    dir: PathBuf,
}

const TOML: &str = "[documents]\npaths = [\"../doc\"]\n\n[documents.delimiters]\ntex = [\"<<\", \">>\"]\n\n[references]\ncopied_values = true\n";

impl M {
    fn new(toml: &str) -> M {
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!("syscribe-docref-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let m = M { dir };
        m.w("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
        m.w("model/.syscribe.toml", toml);
        m.w("model/Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
        m.w("model/Features/Limits.md", "---\ntype: FeatureDef\nid: FEAT-LIMITS\nname: Limits\nmandatory: true\nparameters:\n  - { name: maxTemp, type: ScalarValues::Integer, default: 120, unit: degC, range: \"0..200\" }\n  - { name: serial, type: ScalarValues::String }\n---\n");
        m.w("model/Features/Heater.md", "---\ntype: FeatureDef\nid: FEAT-HEATER\nname: Heater\nmandatory: true\nparameters:\n  - { name: onTimeMs, type: ScalarValues::Integer, unit: ms, derivedFrom: \"Features::Limits.maxTemp * 2\" }\n---\n");
        m.w("model/Configs/CONF-HTR-002.md", "---\ntype: Configuration\nid: CONF-HTR-002\nname: Variant 2\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Limits: true\n  Features::Heater: true\nparameterBindings:\n  Features::Limits.maxTemp: 130\n---\n");
        m.w("model/Rig.md", "---\ntype: PartDef\nname: Rig\ndomain: system\nfeatures:\n  - name: settle\n    value: 150\n    unit: ms\n  - name: label\n    value: \"R&D_rig\"\n  - name: load\n    value: 50\n    unit: \"%\"\n---\n\nThe rig.\n");
        m.w("model/REQ-HTR-001.md", "---\nid: REQ-HTR-001\ntype: Requirement\nname: \"Temperature limit\"\nstatus: draft\nreqDomain: system\nreqClass: system\n---\n\nThe heater shall settle within {{Rig.settle}}.\n\n1. {#limit} The limit holds.\n");
        m
    }
    fn w(&self, rel: &str, c: &str) -> PathBuf {
        let p = self.dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, c).unwrap();
        p
    }
    /// Run from the fixture directory, so document paths are short and stable.
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).current_dir(&self.dir).args(["-m", "model"]).args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned(), o.status.code().unwrap_or(-1))
    }
    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.dir.join(rel)).unwrap()
    }
    fn exists(&self, rel: &str) -> bool {
        Path::new(&self.dir.join(rel)).exists()
    }
}

// ── REQ-TRS-DOCREF-001: render ───────────────────────────────────────────────

#[test]
fn a_tex_file_renders_with_values_units_and_latex_escaping() {
    let m = M::new(TOML);
    m.w("doc/ch.tex", "\\section{Limits}\nMax <<Limits.maxTemp>>, settle <<Rig.settle>>, load <<Rig.load>>, rig <<Rig.label>>.\nOn time << Features::Heater.onTimeMs >>; bare <<FEAT-LIMITS.maxTemp|value>>. Braces {{stay}} as written.\n");
    let (out, err, code) = m.run(&["render", "doc/ch.tex", "--out", "build/ch.tex"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.is_empty(), "with --out nothing goes to stdout: {out}");
    let t = m.read("build/ch.tex");
    assert_eq!(
        t,
        "\\section{Limits}\nMax 120\\,\\textdegree{}C, settle 150\\,ms, load 50\\%, rig R\\&D\\_rig.\nOn time 240\\,ms; bare 120. Braces {{stay}} as written.\n"
    );
}

#[test]
fn without_out_the_result_goes_to_stdout() {
    let m = M::new(TOML);
    m.w("doc/ch.tex", "T <<Rig.settle>>\n");
    let (out, _, code) = m.run(&["render", "doc/ch.tex"]);
    assert_eq!((out.as_str(), code), ("T 150\\,ms\n", 0));
}

#[test]
fn a_configuration_gives_its_own_values() {
    let m = M::new(TOML);
    m.w("doc/ch.tex", "Max <<Limits.maxTemp>>, on <<Heater.onTimeMs>>.\n");
    let (out, err, code) = m.run(&["render", "doc/ch.tex", "--config", "CONF-HTR-002"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out, "Max 130\\,\\textdegree{}C, on 260\\,ms.\n");
    let (_, err, code) = m.run(&["render", "doc/ch.tex", "--config", "CONF-NOPE-001"]);
    assert_ne!(code, 0, "{err}");
}

#[test]
fn a_misspelt_reference_fails_with_file_and_line_and_writes_nothing() {
    let m = M::new(TOML);
    m.w("doc/ch.tex", "ok <<Rig.settle>>\nbad <<Rig.setle>>\nworse <<Nope.x>> and <<Limits.nope>>\n");
    let (out, err, code) = m.run(&["render", "doc/ch.tex", "--out", "build/ch.tex"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(err.contains("doc/ch.tex:2: E331: unresolvable value reference `<<Rig.setle>>`") && err.contains("settle"), "{err}");
    assert!(err.contains("doc/ch.tex:3: E331") && err.contains("`<<Nope.x>>`") && err.contains("`<<Limits.nope>>`"), "{err}");
    assert!(!m.exists("build/ch.tex"), "nothing is written when a reference does not resolve");
    // A parameter with no value needs a Configuration: also a failure, and it says so.
    m.w("doc/s.tex", "<<Limits.serial>>\n");
    let (_, err, code) = m.run(&["render", "doc/s.tex"]);
    assert!(code == 1 && err.contains("--config"), "{err}");
}

#[test]
fn escaping_follows_the_file_type_and_the_flag_overrides_it() {
    let m = M::new("");
    for (file, want) in [
        ("d.typ", "150~ms 120~°C R&D\\_rig"),
        ("d.html", "150&#8239;ms 120&#8239;&deg;C R&amp;D_rig"),
        ("d.md", "150 ms 120 degC R&D\\_rig"),
        ("d.adoc", "150 ms 120 degC R&D_rig"),
    ] {
        m.w(&format!("doc/{file}"), "{{Rig.settle}} {{Limits.maxTemp}} {{Rig.label}}");
        let (out, err, code) = m.run(&["render", &format!("doc/{file}")]);
        assert_eq!((out.as_str(), code), (want, 0), "{file}: {err}");
    }
    let (out, _, _) = m.run(&["render", "doc/d.adoc", "--escape", "latex"]);
    assert_eq!(out, "150\\,ms 120\\,\\textdegree{}C R\\&D\\_rig");
    let (out, _, _) = m.run(&["render", "doc/d.html", "--escape", "none"]);
    assert_eq!(out, "150 ms 120 degC R&D_rig");
    let (_, err, code) = m.run(&["render", "doc/d.html", "--escape", "pdf"]);
    assert!(code == 1 && err.contains("latex, typst, html, markdown, none"), "{err}");
    // Markdown: a reference inside code stays literal.
    m.w("doc/c.md", "`{{Rig.settle}}` is {{Rig.settle}}\n");
    assert_eq!(m.run(&["render", "doc/c.md"]).0, "`{{Rig.settle}}` is 150 ms\n");
}

#[test]
fn delimiters_come_from_the_flags_or_the_documents_table() {
    let m = M::new("");
    m.w("doc/ch.tex", "a {{Rig.settle}} b @@Rig.settle@@ c <<Rig.settle>>\n");
    // No table: the default delimiters.
    assert_eq!(m.run(&["render", "doc/ch.tex"]).0, "a 150\\,ms b @@Rig.settle@@ c <<Rig.settle>>\n");
    let (out, err, code) = m.run(&["render", "doc/ch.tex", "--open", "@@", "--close", "@@"]);
    assert_eq!((out.as_str(), code), ("a {{Rig.settle}} b 150\\,ms c <<Rig.settle>>\n", 0), "{err}");
    let (_, err, code) = m.run(&["render", "doc/ch.tex", "--open", "@@"]);
    assert!(code == 1 && err.contains("--open and --close"), "{err}");
    let (_, err, code) = m.run(&["render", "doc/ch.tex", "--open", "ab", "--close", "cd"]);
    assert!(code == 1 && err.contains("punctuation"), "{err}");
    // The table selects them by file type.
    let m = M::new(TOML);
    m.w("doc/ch.tex", "a {{Rig.settle}} c <<Rig.settle>>\n");
    assert_eq!(m.run(&["render", "doc/ch.tex"]).0, "a {{Rig.settle}} c 150\\,ms\n");
}

#[test]
fn a_malformed_documents_table_is_a_warning_and_the_entry_is_ignored() {
    let m = M::new("[documents]\npaths = \"doc\"\nbogus = 1\n[documents.delimiters]\ntex = [\"<<\"]\n");
    m.w("doc/ch.tex", "{{Rig.settle}}\n");
    let (out, err, code) = m.run(&["render", "doc/ch.tex"]);
    assert_eq!((out.as_str(), code), ("150\\,ms\n", 0));
    assert!(err.contains("warning: .syscribe.toml: [documents] paths") && err.contains("no key `bogus`") && err.contains("[documents.delimiters] tex"), "{err}");
}

#[test]
fn rendering_a_diagram_element_is_unchanged() {
    let m = M::new(TOML);
    m.w("model/D.md", "---\ntype: Diagram\nname: D\ndiagramKind: Mermaid\n---\n\n```mermaid\ngraph TD\n  A[\"a\"]\n```\n");
    let (out, err, code) = m.run(&["render", "model/D.md"]);
    assert!(code == 0 && out.contains("graph TD"), "{out}{err}");
}

// ── REQ-TRS-DOCREF-002: lint-docs ────────────────────────────────────────────

#[test]
fn lint_docs_checks_a_tex_file() {
    let m = M::new(TOML);
    m.w("doc/ch.tex", "See REQ-HTR-001 and REQ-GONE-001.\nItem REQ-HTR-001#limit and REQ-HTR-001#nope.\nSettle <<Rig.settle>>, again 150, typo <<Rig.setle>>, open <<Limits.serial>>.\n");
    let (out, err, code) = m.run(&["lint-docs", "doc"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(out.contains("doc/ch.tex:1: W099") && out.contains("REQ-GONE-001"), "{out}");
    assert!(!out.contains("'REQ-HTR-001'"), "a resolving id is clean: {out}");
    assert!(out.contains("doc/ch.tex:2: W092") && out.contains("REQ-HTR-001#nope"), "{out}");
    assert!(out.contains("doc/ch.tex:3: W098") && out.contains("`<<Rig.setle>>`"), "{out}");
    assert!(!out.contains("Limits.serial"), "a parameter that only lacks a Configuration is not broken: {out}");
    assert!(out.contains("doc/ch.tex:3: W095") && out.contains("write `<<Rig.settle>>`"), "{out}");
    let (json, _, _) = m.run(&["lint-docs", "doc", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.as_array().unwrap().iter().any(|f| f["code"] == "W098" && f["line"] == 3), "{json}");
}

#[test]
fn the_file_types_read_in_a_directory_are_selectable() {
    // Default: tex, typ, adoc, html, txt.
    let m = M::new("");
    for f in ["a.tex", "a.typ", "a.adoc", "a.html", "a.txt", "a.rst", "a.log"] {
        m.w(&format!("doc/{f}"), "REQ-GONE-001\n");
    }
    let (out, _, _) = m.run(&["lint-docs", "doc"]);
    for f in ["a.tex", "a.typ", "a.adoc", "a.html", "a.txt"] {
        assert!(out.contains(&format!("doc/{f}:1: W099")), "{f}: {out}");
    }
    assert!(!out.contains("a.rst") && !out.contains("a.log"), "{out}");
    // A file named on the command line is read when it is a known text type.
    assert!(m.run(&["lint-docs", "doc/a.rst"]).0.contains("doc/a.rst:1: W099"));
    // `extensions` replaces the default list.
    let m = M::new("[documents]\nextensions = [\"rst\"]\n");
    for f in ["a.tex", "a.rst"] {
        m.w(&format!("doc/{f}"), "REQ-GONE-001\n");
    }
    let (out, _, _) = m.run(&["lint-docs", "doc"]);
    assert!(out.contains("doc/a.rst:1: W099") && !out.contains("a.tex"), "{out}");
}

#[test]
fn a_clean_tex_file_passes_and_w098_can_be_denied() {
    let m = M::new(TOML);
    m.w("doc/ch.tex", "See REQ-HTR-001; settle <<Rig.settle>>.\n");
    let (out, _, code) = m.run(&["lint-docs", "doc"]);
    assert_eq!((out.as_str(), code), ("", 0));
    let (_, _, code) = m.run(&["lint-docs", "doc", "--deny", "W098"]);
    assert_eq!(code, 0);
    let (explain, _, code) = m.run(&["explain", "W098"]);
    assert!(code == 0 && explain.contains("W098"), "{explain}");
}

// ── REQ-TRS-DOCREF-003: refs ─────────────────────────────────────────────────

#[test]
fn refs_lists_the_documents_that_use_an_attribute() {
    let m = M::new(TOML);
    m.w("doc/ch.tex", "a\nSettle <<Rig.settle>> and <<Rig.load>>.\n");
    m.w("doc/sub/notes.md", "Settle {{Rig.settle}}.\n");
    m.w("doc/skip.log", "<<Rig.settle>>\n");
    let (out, err, code) = m.run(&["refs", "Rig.settle"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("## Uses in documents"), "{out}");
    assert!(out.contains("| doc/ch.tex | 2 | `<<Rig.settle>>` |"), "{out}");
    assert!(out.contains("| doc/sub/notes.md | 1 | `{{Rig.settle}}` |"), "{out}");
    assert!(!out.contains("skip.log") && !out.contains("Rig.load"), "{out}");
    let (json, _, _) = m.run(&["refs", "Rig.settle", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let uses = v["documentUses"].as_array().unwrap();
    assert_eq!(uses.len(), 2, "{json}");
    assert_eq!(uses[0], serde_json::json!({"file": "doc/ch.tex", "line": 2, "raw": "<<Rig.settle>>"}));
}

#[test]
fn refs_lists_the_documents_that_use_a_feature_parameter() {
    let m = M::new(TOML);
    m.w("doc/ch.tex", "Max <<Limits.maxTemp>>, <<FEAT-LIMITS.maxTemp|value>>, <<Heater.onTimeMs>>.\n");
    let (out, err, code) = m.run(&["refs", "Limits.maxTemp"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("## Uses in documents (FEAT-LIMITS.maxTemp)"), "{out}");
    assert!(out.contains("| doc/ch.tex | 1 | `<<Limits.maxTemp>>` |") && out.contains("`<<FEAT-LIMITS.maxTemp|value>>`"), "{out}");
    assert!(!out.contains("onTimeMs"), "{out}");
}

#[test]
fn without_a_documents_table_refs_reads_as_before() {
    let m = M::new("");
    m.w("doc/ch.tex", "<<Rig.settle>> {{Rig.settle}}\n");
    let (out, _, code) = m.run(&["refs", "Rig.settle"]);
    assert!(code == 0 && !out.contains("Uses in documents"), "{out}");
}

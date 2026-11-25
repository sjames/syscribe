//! Value references in external documents (GH #530, `REQ-TRS-DOCREF-001..003`).
//!
//! A document outside the model — LaTeX, Typst, AsciiDoc, HTML, plain text — states a design value
//! with the model's own reference syntax, `{{Elem.attr}}`, or with other delimiters where braces
//! clash with the document language (`<<Elem.attr>>`). `render` replaces each reference with the
//! current value and unit, escaped for the target; `lint-docs` and `refs` read the same references.
//!
//! Unlike an element body (`value_refs`), every delimited `X.y` span in a document is a reference:
//! there is no template text to leave alone, so one that does not resolve is an error. A reference
//! to a feature parameter is resolved too — in the chosen `Configuration`, else to its fixed,
//! derived or default value.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::value_refs as vr;

/// The delimiters around a reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delims {
    pub open: String,
    pub close: String,
}

impl Default for Delims {
    fn default() -> Self {
        Delims { open: "{{".into(), close: "}}".into() }
    }
}

impl Delims {
    pub fn new(open: &str, close: &str) -> Result<Self, String> {
        if open.trim().is_empty() || close.trim().is_empty() {
            return Err("a delimiter must not be empty".into());
        }
        if open.chars().chain(close.chars()).any(|c| c.is_alphanumeric() || c.is_whitespace() || c == '.' || c == '_' || c == '|') {
            return Err(format!("delimiters `{open}` `{close}` must be punctuation only (no letters, digits, spaces, `.`, `_` or `|`)"));
        }
        Ok(Delims { open: open.to_string(), close: close.to_string() })
    }

    fn regex(&self) -> Regex {
        Regex::new(&format!(
            r"{}\s*([A-Za-z_][A-Za-z0-9_\-:]*)\.([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\s*(?:\|\s*(unit|value)\s*)?{}",
            regex::escape(&self.open),
            regex::escape(&self.close)
        ))
        .expect("document reference regex")
    }
}

/// How a rendered value is written for the document language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Escape {
    Latex,
    Typst,
    Html,
    Markdown,
    None,
}

impl Escape {
    pub const NAMES: &'static [&'static str] = &["latex", "typst", "html", "markdown", "none"];

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "latex" | "tex" => Escape::Latex,
            "typst" | "typ" => Escape::Typst,
            "html" => Escape::Html,
            "markdown" | "md" => Escape::Markdown,
            "none" | "text" => Escape::None,
            _ => return None,
        })
    }

    /// The escaping a file's extension implies (`none` for one that names no known language).
    pub fn for_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
            Some("tex" | "sty" | "cls" | "ltx") => Escape::Latex,
            Some("typ") => Escape::Typst,
            Some("html" | "htm" | "xhtml") => Escape::Html,
            Some("md" | "markdown") => Escape::Markdown,
            _ => Escape::None,
        }
    }

    /// `s` with the characters the language reads as markup escaped.
    pub fn text(self, s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            match (self, c) {
                (Escape::Latex, '\\') => out.push_str("\\textbackslash{}"),
                (Escape::Latex, '~') => out.push_str("\\textasciitilde{}"),
                (Escape::Latex, '^') => out.push_str("\\textasciicircum{}"),
                (Escape::Latex, '&' | '%' | '$' | '#' | '_' | '{' | '}') => {
                    out.push('\\');
                    out.push(c);
                }
                (Escape::Typst, '\\' | '#' | '$' | '*' | '_' | '@' | '<' | '>' | '[' | ']' | '`' | '~') => {
                    out.push('\\');
                    out.push(c);
                }
                (Escape::Html, '&') => out.push_str("&amp;"),
                (Escape::Html, '<') => out.push_str("&lt;"),
                (Escape::Html, '>') => out.push_str("&gt;"),
                (Escape::Html, '"') => out.push_str("&quot;"),
                (Escape::Markdown, '\\' | '*' | '_' | '`' | '[' | ']' | '<' | '>' | '|') => {
                    out.push('\\');
                    out.push(c);
                }
                _ => out.push(c),
            }
        }
        out
    }

    /// A unit as the language writes it, and whether it is set tight against the number (a degree
    /// or percent sign) rather than after a thin space.
    fn unit(self, unit: &str) -> (String, bool) {
        let sym = |latex: &str, typst: &str, html: &str| match self {
            Escape::Latex => Some(latex.to_string()),
            Escape::Typst => Some(typst.to_string()),
            Escape::Html => Some(html.to_string()),
            Escape::Markdown | Escape::None => None,
        };
        let special = match unit {
            "degC" | "°C" => sym("\\textdegree{}C", "°C", "&deg;C").map(|s| (s, false)),
            "degF" | "°F" => sym("\\textdegree{}F", "°F", "&deg;F").map(|s| (s, false)),
            "deg" | "°" => sym("\\textdegree{}", "°", "&deg;").map(|s| (s, true)),
            "%" | "percent" => sym("\\%", "%", "%").map(|s| (s, true)),
            "Ohm" | "ohm" | "Ω" => sym("\\textohm{}", "Ω", "&Omega;").map(|s| (s, false)),
            "um" | "µm" => sym("\\textmu{}m", "µm", "&micro;m").map(|s| (s, false)),
            "us" | "µs" => sym("\\textmu{}s", "µs", "&micro;s").map(|s| (s, false)),
            _ => None,
        };
        special.unwrap_or_else(|| (self.text(unit), false))
    }

    /// `value` and `unit` as one quantity: `150\,ms`, `120\,\textdegree{}C`, `150 ms`.
    pub fn quantity(self, value: &str, unit: Option<&str>) -> String {
        let v = self.text(value);
        let Some(u) = unit.map(|u| u.strip_prefix("SI::").unwrap_or(u)).filter(|u| !u.is_empty()) else { return v };
        let (u, tight) = self.unit(u);
        if tight {
            return format!("{v}{u}");
        }
        let space = match self {
            Escape::Latex => "\\,",
            Escape::Typst => "~",
            Escape::Html => "&#8239;",
            Escape::Markdown | Escape::None => " ",
        };
        format!("{v}{space}{u}")
    }
}

/// One reference in a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocRef {
    /// As written, delimiters included.
    pub raw: String,
    pub elem: String,
    pub path: String,
    /// `|value`: the bare value, without its unit.
    pub bare: bool,
    /// 0-based line.
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

/// Every reference of `text`. With `skip_code` (Markdown) the ones inside code are left alone.
pub fn scan(text: &str, delims: &Delims, skip_code: bool) -> Vec<DocRef> {
    if !text.contains(&delims.open) {
        return Vec::new();
    }
    let code = if skip_code { vr::code_ranges(text) } else { Vec::new() };
    delims
        .regex()
        .captures_iter(text)
        .filter_map(|c| {
            let m = c.get(0)?;
            if code.iter().any(|&(a, b)| m.start() >= a && m.start() < b) {
                return None;
            }
            Some(DocRef {
                raw: m.as_str().to_string(),
                elem: c[1].to_string(),
                path: c[2].to_string(),
                bare: c.get(3).is_some_and(|s| s.as_str() == "value"),
                line: text[..m.start()].bytes().filter(|b| *b == b'\n').count(),
                start: m.start(),
                end: m.end(),
            })
        })
        .collect()
}

/// What a reference resolves to.
#[derive(Debug, Clone, PartialEq)]
pub struct Quantity {
    pub value: String,
    pub unit: Option<String>,
    /// The element holding the value (stable id, else qualified name).
    pub target: String,
    /// The attribute or parameter path on it.
    pub attr: String,
}

/// The model a document is rendered against.
pub struct Scope<'a> {
    /// The elements value references resolve in (the projection under `--config`).
    pub view: &'a [RawElement],
    /// The whole model, where feature parameters are declared.
    pub full: &'a [RawElement],
    /// The chosen Configuration, an element of `full`.
    pub config: Option<&'a RawElement>,
}

/// Resolve one reference: an attribute of an element, or a parameter of a feature.
pub fn resolve(scope: &Scope<'_>, view_resolver: &Resolver, full_resolver: &Resolver, children: &vr::ChildIndex<'_>, r: &DocRef) -> Result<Quantity, String> {
    // A feature parameter: by `FEAT-*` id, qualified name or unique basic name.
    let feature = match vr::resolve_element(scope.full, full_resolver, &r.elem) {
        vr::ElemMatch::Found(t) if t.frontmatter.element_type == Some(ElementType::FeatureDef) => Some(t),
        _ => None,
    };
    if let Some(fd) = feature {
        let decl = fd.frontmatter.parameters.iter().flatten().find_map(|p| match p {
            serde_yaml::Value::Mapping(m) if m.get("name").and_then(|v| v.as_str()) == Some(r.path.as_str()) => Some(m),
            _ => None,
        });
        let Some(decl) = decl else {
            let names: Vec<&str> = fd.frontmatter.parameters.iter().flatten().filter_map(|p| p.get("name").and_then(|v| v.as_str())).collect();
            return Err(format!("feature {} declares no parameter `{}` — its parameters: {}", vr::label(fd), r.path, if names.is_empty() { "none".to_string() } else { names.join(", ") }));
        };
        let Some(value) = crate::placeholders::param_text(scope.full, full_resolver, scope.config, &fd.qualified_name, &r.path) else {
            return Err(format!(
                "parameter {}.{} has no value{} (no binding, fixed value, derived value or default)",
                vr::label(fd),
                r.path,
                match scope.config {
                    Some(c) => format!(" in {}", vr::label(c)),
                    None => " — choose a Configuration with --config".to_string(),
                }
            ));
        };
        let unit = decl.get("unit").and_then(|v| v.as_str()).map(str::to_string);
        // A derived quantity already carries its unit in the text.
        let unit = unit.filter(|_| vr::numeric_value(&value).is_some() && value.trim().parse::<f64>().is_ok());
        return Ok(Quantity { value, unit, target: vr::label(fd), attr: r.path.clone() });
    }
    let as_ref = vr::ValueRef { raw: r.raw.clone(), elem: r.elem.clone(), path: r.path.clone(), line: r.line, start: r.start, end: r.end };
    match vr::resolve(scope.view, view_resolver, children, &as_ref) {
        vr::Resolution::Value { elem, attr } => Ok(Quantity { value: attr.value.clone().unwrap_or_default(), unit: attr.unit.clone(), target: vr::label(elem), attr: attr.path }),
        vr::Resolution::Error(m) => Err(m),
        vr::Resolution::NotOurs => Err(format!(
            "element `{}` does not resolve to any element (by stable id, qualified name or basic name) — check the name with `check-ref`, or write the full qualified name",
            r.elem
        )),
    }
}

/// A reference that does not resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unresolved {
    /// 1-based line.
    pub line: usize,
    pub raw: String,
    pub why: String,
}

/// Each reference of `text` with what it resolves to.
pub fn resolve_all(scope: &Scope<'_>, text: &str, delims: &Delims, skip_code: bool) -> Vec<(DocRef, Result<Quantity, String>)> {
    let refs = scan(text, delims, skip_code);
    if refs.is_empty() {
        return Vec::new();
    }
    let view_resolver = Resolver::new(scope.view);
    let full_resolver = Resolver::new(scope.full);
    let children = vr::ChildIndex::new(scope.view);
    refs.into_iter()
        .map(|r| {
            let q = resolve(scope, &view_resolver, &full_resolver, &children, &r);
            (r, q)
        })
        .collect()
}

/// `text` with every reference replaced by its value and unit, written for `escape`. `Err` lists
/// the references that do not resolve; nothing is rendered then.
pub fn render(scope: &Scope<'_>, text: &str, delims: &Delims, escape: Escape, skip_code: bool) -> Result<String, Vec<Unresolved>> {
    let resolved = resolve_all(scope, text, delims, skip_code);
    let bad: Vec<Unresolved> = resolved
        .iter()
        .filter_map(|(r, q)| q.as_ref().err().map(|why| Unresolved { line: r.line + 1, raw: r.raw.clone(), why: why.clone() }))
        .collect();
    if !bad.is_empty() {
        return Err(bad);
    }
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (r, q) in &resolved {
        let Ok(q) = q else { continue };
        out.push_str(&text[last..r.start]);
        out.push_str(&escape.quantity(&q.value, if r.bare { None } else { q.unit.as_deref() }));
        last = r.end;
    }
    out.push_str(&text[last..]);
    Ok(out)
}

// ── `[documents]` ────────────────────────────────────────────────────────────

/// The file types `lint-docs` and `refs` read beside Markdown when `[documents]` names none.
pub const DEFAULT_EXTENSIONS: &[&str] = &["tex", "typ", "adoc", "html", "txt"];

/// `[documents]` of `.syscribe.toml`: the external documents that use the model's values.
#[derive(Debug, Clone, Default)]
pub struct DocumentsConfig {
    /// Files and directories (resolved against the model root) that `refs` scans.
    pub paths: Vec<PathBuf>,
    /// File extensions read beside `md`; `None` = [`DEFAULT_EXTENSIONS`].
    pub extensions: Option<Vec<String>>,
    /// Extension → the delimiters its references use.
    pub delimiters: BTreeMap<String, Delims>,
    /// Extension of the rendered file → the template delimiters (`[documents.templateSyntax.<ext>]`).
    pub template_syntax: BTreeMap<String, TemplateSyntax>,
    /// Malformed entries (printed as warnings by the commands that read the table; the entry is ignored).
    pub problems: Vec<String>,
}

/// The delimiters of a template (GH #532): `variable` around an expression, `block` around a
/// statement, `comment` around a comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSyntax {
    pub variable: (String, String),
    pub block: (String, String),
    pub comment: (String, String),
}

impl TemplateSyntax {
    /// Jinja's own: `{{ }}`, `{% %}`, `{# #}`.
    pub fn jinja() -> Self {
        let p = |a: &str, b: &str| (a.to_string(), b.to_string());
        TemplateSyntax { variable: p("{{", "}}"), block: p("{%", "%}"), comment: p("{#", "#}") }
    }

    /// For LaTeX, where braces and `%` belong to the document: `<< >>`, `<% %>`, `<# #>`.
    pub fn latex() -> Self {
        let p = |a: &str, b: &str| (a.to_string(), b.to_string());
        TemplateSyntax { variable: p("<<", ">>"), block: p("<%", "%>"), comment: p("<#", "#>") }
    }
}

impl DocumentsConfig {
    /// The template delimiters for a rendered file: its `[documents.templateSyntax.<ext>]` entry, else
    /// the LaTeX set for a LaTeX file and Jinja's own for anything else.
    pub fn template_syntax_for(&self, rendered: &Path) -> TemplateSyntax {
        let ext = rendered.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).unwrap_or_default();
        self.template_syntax.get(&ext).cloned().unwrap_or_else(|| match Escape::for_path(rendered) {
            Escape::Latex => TemplateSyntax::latex(),
            _ => TemplateSyntax::jinja(),
        })
    }

    pub fn load(model_root: &Path) -> Self {
        let mut cfg = DocumentsConfig::default();
        let Some(root) = std::fs::read_to_string(model_root.join(".syscribe.toml")).ok().and_then(|t| toml::from_str::<toml::Value>(&t).ok()) else {
            return cfg;
        };
        let Some(tbl) = root.get("documents") else { return cfg };
        let Some(tbl) = tbl.as_table() else {
            cfg.problems.push("[documents] must be a table".into());
            return cfg;
        };
        let strings = |k: &str, v: &toml::Value, problems: &mut Vec<String>| -> Option<Vec<String>> {
            match v.as_array() {
                Some(a) if a.iter().all(|x| x.as_str().is_some_and(|s| !s.trim().is_empty())) => Some(a.iter().filter_map(|x| x.as_str()).map(|s| s.trim().to_string()).collect()),
                _ => {
                    problems.push(format!("[documents] {k} must be a list of non-empty strings, got {v}; the entry is ignored"));
                    None
                }
            }
        };
        for (k, v) in tbl {
            match k.as_str() {
                "paths" => {
                    if let Some(list) = strings(k, v, &mut cfg.problems) {
                        cfg.paths = list.iter().map(|p| clean(&model_root.join(p))).collect();
                    }
                }
                "extensions" => {
                    if let Some(list) = strings(k, v, &mut cfg.problems) {
                        cfg.extensions = Some(list.iter().map(|e| e.trim_start_matches('.').to_ascii_lowercase()).collect());
                    }
                }
                "delimiters" => match v.as_table() {
                    Some(t) => {
                        for (ext, pair) in t {
                            let parsed = match pair.as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>()) {
                                Some(p) if p.len() == 2 && pair.as_array().is_some_and(|a| a.len() == 2) => Delims::new(p[0], p[1]),
                                _ => Err(format!("expected [\"<open>\", \"<close>\"], got {pair}")),
                            };
                            match parsed {
                                Ok(d) => {
                                    cfg.delimiters.insert(ext.trim_start_matches('.').to_ascii_lowercase(), d);
                                }
                                Err(m) => cfg.problems.push(format!("[documents.delimiters] {ext}: {m}; the default `{{{{ }}}}` is used")),
                            }
                        }
                    }
                    None => cfg.problems.push(format!("[documents] delimiters must be a table of <extension> = [\"<open>\", \"<close>\"], got {v}")),
                },
                "templateSyntax" | "template_syntax" => match v.as_table() {
                    Some(t) => {
                        for (ext, entry) in t {
                            let pair = |k: &str| -> Option<(String, String)> {
                                let a = entry.get(k)?.as_array()?;
                                match (a.first()?.as_str(), a.get(1)?.as_str()) {
                                    (Some(o), Some(c)) if a.len() == 2 && !o.is_empty() && !c.is_empty() => Some((o.to_string(), c.to_string())),
                                    _ => None,
                                }
                            };
                            match (pair("variable"), pair("block"), pair("comment")) {
                                (Some(variable), Some(block), Some(comment)) => {
                                    cfg.template_syntax.insert(ext.trim_start_matches('.').to_ascii_lowercase(), TemplateSyntax { variable, block, comment });
                                }
                                _ => cfg.problems.push(format!("[documents.templateSyntax.{ext}] needs variable, block and comment, each [\"<open>\", \"<close>\"]; the default delimiters are used")),
                            }
                        }
                    }
                    None => cfg.problems.push(format!("[documents] templateSyntax must be a table of <extension> = {{variable, block, comment}}, got {v}")),
                },
                other => cfg.problems.push(format!("[documents] has no key `{other}` (keys: paths, extensions, delimiters, templateSyntax); it is ignored")),
            }
        }
        cfg
    }

    /// Whether a file with this extension is a document `lint-docs`/`refs` read (Markdown always is).
    pub fn reads(&self, ext: &str) -> bool {
        let ext = ext.to_ascii_lowercase();
        ext == "md" || self.delimiters.contains_key(&ext) || match &self.extensions {
            Some(list) => list.contains(&ext),
            None => DEFAULT_EXTENSIONS.contains(&ext.as_str()),
        }
    }

    /// The delimiters of a file, by its extension.
    pub fn delims_for(&self, path: &Path) -> Delims {
        path.extension().and_then(|e| e.to_str()).and_then(|e| self.delimiters.get(&e.to_ascii_lowercase())).cloned().unwrap_or_default()
    }

    /// Every document under `paths`, sorted.
    pub fn files(&self) -> Vec<PathBuf> {
        fn walk(p: &Path, cfg: &DocumentsConfig, out: &mut Vec<PathBuf>, top: bool) {
            if p.is_dir() {
                let Ok(rd) = std::fs::read_dir(p) else { return };
                let mut kids: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
                kids.sort();
                for k in kids {
                    if k.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.')) {
                        continue;
                    }
                    walk(&k, cfg, out, false);
                }
            } else if p.is_file() && (top || p.extension().and_then(|e| e.to_str()).is_some_and(|e| cfg.reads(e))) {
                out.push(p.to_path_buf());
            }
        }
        let mut out = Vec::new();
        for p in &self.paths {
            walk(p, self, &mut out, true);
        }
        out.dedup();
        out
    }
}

/// `path` with `.` and `<dir>/..` components removed (lexically; nothing is read from disk).
pub fn clean(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir if matches!(out.components().next_back(), Some(Component::Normal(_))) => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    if out.as_os_str().is_empty() {
        out.push(".");
    }
    out
}

/// Whether `path` is Markdown (code spans and fences are then left alone).
pub fn is_markdown(path: &Path) -> bool {
    matches!(path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref(), Some("md" | "markdown"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantities_are_written_for_the_target() {
        assert_eq!(Escape::Latex.quantity("150", Some("ms")), "150\\,ms");
        assert_eq!(Escape::Latex.quantity("120", Some("degC")), "120\\,\\textdegree{}C");
        assert_eq!(Escape::Latex.quantity("50", Some("%")), "50\\%");
        assert_eq!(Escape::Latex.quantity("a_b & c", None), "a\\_b \\& c");
        assert_eq!(Escape::Typst.quantity("150", Some("ms")), "150~ms");
        assert_eq!(Escape::Html.quantity("120", Some("degC")), "120&#8239;&deg;C");
        assert_eq!(Escape::Html.quantity("a<b", None), "a&lt;b");
        assert_eq!(Escape::Markdown.quantity("150", Some("SI::ms")), "150 ms");
        assert_eq!(Escape::None.quantity("120", Some("degC")), "120 degC");
    }

    #[test]
    fn references_are_found_with_any_delimiters() {
        let d = Delims::new("<<", ">>").unwrap();
        let r = scan("a <<Limits.maxTemp>> b\n<< Rig.snapshot.header | value >> {{No.pe}}", &d, false);
        assert_eq!(r.len(), 2);
        assert_eq!((r[0].elem.as_str(), r[0].path.as_str(), r[0].bare, r[0].line), ("Limits", "maxTemp", false, 0));
        assert_eq!((r[1].path.as_str(), r[1].bare, r[1].line), ("snapshot.header", true, 1));
        assert!(Delims::new("a", "b").is_err() && Delims::new("", ">>").is_err());
        // Markdown: a reference inside code is literal.
        assert_eq!(scan("`{{A.b}}` {{C.d}}", &Delims::default(), true).len(), 1);
    }

    #[test]
    fn escape_follows_the_extension() {
        assert_eq!(Escape::for_path(Path::new("a/ch.tex")), Escape::Latex);
        assert_eq!(Escape::for_path(Path::new("ch.typ")), Escape::Typst);
        assert_eq!(Escape::for_path(Path::new("ch.adoc")), Escape::None);
        assert_eq!(Escape::parse("latex"), Some(Escape::Latex));
        assert_eq!(Escape::parse("pdf"), None);
    }
}

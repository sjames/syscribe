//! Documents from templates (GH #532, `REQ-TRS-DOCGEN-001..003`).
//!
//! `render --template <file>` renders a text template with the model as its data; `doc build
//! <Document>` renders the template a `Document` element names and records what it used, and
//! `doc build --check` says when the rendered file no longer matches the model. The engine is
//! minijinja (Jinja syntax), strict: an unknown element, attribute, view or variable stops the
//! render with the template file and line.
//!
//! The template says where a value, a table or a diagram goes; a `View` says what is in a table
//! (`view_table`) and the model what a value is. A PDF is made from the rendered text by the tool
//! that owns that step (LaTeX, Typst, pandoc), not here.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use minijinja::value::Kwargs;
use minijinja::{Environment, Error, ErrorKind, UndefinedBehavior, Value};
use serde_json::Value as J;

use syscribe_model::config::{SourceLocation, ValidateConfig};
use syscribe_model::doc_render::{self as dr, DocumentsConfig, Escape, TemplateSyntax};
use syscribe_model::document;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;
use syscribe_model::view_table as vt;

/// One helper call a template made, with the hash of what it returned — the unit `--check` compares.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Call {
    #[serde(rename = "fn")]
    pub func: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arg: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub kwargs: serde_json::Map<String, J>,
    pub hash: String,
}

impl Call {
    fn show(&self) -> String {
        let mut parts: Vec<String> = self.arg.iter().map(|a| format!("{a:?}")).collect();
        parts.extend(self.kwargs.iter().map(|(k, v)| format!("{k}={v}")));
        format!("{}({})", self.func, parts.join(", "))
    }
}

/// What a helper returns: text ready for the document, or data for the template to walk.
enum Ret {
    Text(String),
    Data(J),
    /// Text, and a file to write beside the rendered document (a diagram).
    File { path: String, content: String },
}

/// The model a template is rendered against. Owned, so the engine's functions can share it.
pub struct Model {
    pub full: Vec<RawElement>,
    /// The projection under `--config` (the whole model without one).
    pub view: Vec<RawElement>,
    resolver: Resolver,
    /// Index of the chosen Configuration in `full`.
    config: Option<usize>,
    vcfg: ValidateConfig,
    model_root: PathBuf,
    /// The resolved export of `view`.
    export: J,
    by_qname: std::collections::HashMap<String, usize>,
    target: Escape,
    /// Where files a helper writes (diagrams) go; `None` = nothing is written.
    out_dir: Option<PathBuf>,
    calls: Mutex<Vec<Call>>,
}

impl Model {
    pub fn new(full: &[RawElement], view: Vec<RawElement>, vcfg: &ValidateConfig, model_root: &Path, config: Option<&str>, target: Escape, out_dir: Option<PathBuf>) -> Result<Self, String> {
        let cfg_idx = match config {
            None => None,
            Some(c) => Some(crate::residual::find_configuration(full, c).ok_or_else(|| format!("Configuration not found: {c}"))?),
        };
        let export = crate::export::resolved_document(&view, vcfg, &crate::export::Resolved { model_root, full, config });
        let by_qname = view.iter().enumerate().map(|(i, e)| (e.qualified_name.clone(), i)).collect();
        let resolver = Resolver::new(&view);
        Ok(Model { full: full.to_vec(), view, resolver, config: cfg_idx, vcfg: vcfg.clone(), model_root: model_root.to_path_buf(), export, by_qname, target, out_dir, calls: Mutex::new(Vec::new()) })
    }

    fn element_json(&self, e: &RawElement) -> J {
        self.by_qname.get(&e.qualified_name).and_then(|i| self.export["elements"].get(*i)).cloned().unwrap_or(J::Null)
    }

    fn find(&self, r: &str) -> Result<&RawElement, String> {
        self.resolver
            .resolve_ref(&self.view, r)
            .ok_or_else(|| format!("element `{r}` does not resolve{} — check the name with `check-ref`", if self.config.is_some() { " in this Configuration" } else { "" }))
    }

    /// Evaluate one helper. The same function serves the render and `--check`.
    fn eval(&self, func: &str, arg: Option<&str>, kw: &serde_json::Map<String, J>) -> Result<Ret, String> {
        let need = || arg.ok_or_else(|| format!("{func}() needs a name"));
        let kw_str = |k: &str| kw.get(k).and_then(|v| v.as_str());
        let known = |allowed: &[&str]| -> Result<(), String> {
            match kw.keys().find(|k| !allowed.contains(&k.as_str())) {
                Some(k) => Err(format!("{func}() has no argument `{k}`{}", if allowed.is_empty() { String::new() } else { format!(" (arguments: {})", allowed.join(", ")) })),
                None => Ok(()),
            }
        };
        match func {
            // value("Limits.maxTemp") — the quantity, written for the target.
            "value" => {
                known(&["bare"])?;
                let key = need()?;
                let (elem, path) = key.split_once('.').ok_or_else(|| format!("value(\"{key}\") is not `<Elem>.<attr>`"))?;
                let r = dr::DocRef { raw: key.to_string(), elem: elem.to_string(), path: path.to_string(), bare: false, line: 0, start: 0, end: 0 };
                let scope = dr::Scope { view: &self.view, full: &self.full, config: self.config.map(|i| &self.full[i]) };
                let children = syscribe_model::value_refs::ChildIndex::new(&self.view);
                let q = dr::resolve(&scope, &self.resolver, &Resolver::new(&self.full), &children, &r).map_err(|m| format!("value(\"{key}\"): {m}"))?;
                let bare = kw.get("bare").and_then(|v| v.as_bool()).unwrap_or(false);
                Ok(Ret::Text(self.target.quantity(&q.value, if bare { None } else { q.unit.as_deref() })))
            }
            "element" => {
                known(&[])?;
                Ok(Ret::Data(self.element_json(self.find(need()?)?)))
            }
            // elements(type="Requirement", where="status != draft") — in qualified-name order.
            "elements" => {
                known(&["type", "where"])?;
                if let Some(w) = kw_str("where") {
                    vt::check_expr(w).map_err(|m| format!("elements(where=…): {m}"))?;
                }
                let ty = kw_str("type");
                if let Some(t) = ty {
                    if !ElementType::ALL.iter().any(|x| x.name() == t) {
                        return Err(format!("elements(type=\"{t}\"): no such element type"));
                    }
                }
                let mut hits: Vec<&RawElement> = self
                    .view
                    .iter()
                    .filter(|e| !e.file_path.is_empty())
                    .filter(|e| ty.is_none_or(|t| e.frontmatter.element_type.as_ref().is_some_and(|x| x.name() == t)))
                    .filter(|e| kw_str("where").is_none_or(|w| vt::element_holds(e, w)))
                    .collect();
                hits.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
                Ok(Ret::Data(J::Array(hits.into_iter().map(|e| self.element_json(e)).collect())))
            }
            // view("Views::OpenQuestions") — the table, in the target's form; format="data" for the rows.
            "view" => {
                known(&["format"])?;
                let name = need()?;
                let v = self.find(name)?;
                if !vt::is_view(v) {
                    return Err(format!("view(\"{name}\"): {} is not a View", v.qualified_name));
                }
                let table = vt::parse(&v.frontmatter.extra).map_err(|e| format!("view(\"{name}\"): malformed table view: {}", e.join("; ")))?.unwrap_or_else(vt::default_table);
                let ex = syscribe_model::view_exposure::exposure(&self.view, &self.resolver, v);
                if let Some(u) = ex.unresolved.first() {
                    return Err(format!("view(\"{name}\"): expose target '{u}' does not resolve"));
                }
                let built = vt::build(&vt::Ctx { elements: &self.view, full: &self.full, config: self.config.map(|i| &self.full[i]) }, v, &ex.items, &table);
                let format = match kw_str("format") {
                    Some("data") => return Ok(Ret::Data(built.json())),
                    Some(f) => vt::Format::parse(f).ok_or_else(|| format!("view(\"{name}\", format=\"{f}\"): one of latex, markdown, csv, html, data"))?,
                    None => match self.target {
                        Escape::Latex => vt::Format::Latex,
                        Escape::Html => vt::Format::Html,
                        _ => vt::Format::Markdown,
                    },
                };
                Ok(Ret::Text(built.write(format)))
            }
            // diagram("Context") — exports the diagram beside the document and gives its file name.
            "diagram" => {
                known(&["format"])?;
                let name = need()?;
                let d = self.find(name)?;
                let format = kw_str("format").unwrap_or("svg");
                let ext = match format {
                    "svg" => "svg",
                    "mermaid" => "mmd",
                    "plantuml" => "puml",
                    "dot" => "dot",
                    other => return Err(format!("diagram(\"{name}\", format=\"{other}\"): one of svg, mermaid, plantuml, dot (make a PDF from the SVG with the document tool chain)")),
                };
                let content = crate::diagram_export::export_diagram(d, &self.view, &self.resolver, &self.model_root, &self.vcfg, format).map_err(|m| format!("diagram(\"{name}\"): {m}"))?;
                let leaf = d.qualified_name.rsplit("::").next().unwrap_or(&d.qualified_name);
                Ok(Ret::File { path: format!("{leaf}.{ext}"), content })
            }
            // selected("FEAT-ABS") — whether the Configuration selects the feature.
            "selected" => {
                known(&[])?;
                let name = need()?;
                let idx = self.config.ok_or_else(|| format!("selected(\"{name}\") needs a Configuration (--config, or `config:` on the Document)"))?;
                let fd = Resolver::new(&self.full)
                    .resolve_ref(&self.full, name)
                    .filter(|f| f.frontmatter.element_type == Some(ElementType::FeatureDef))
                    .ok_or_else(|| format!("selected(\"{name}\"): not a FeatureDef"))?;
                let prov = syscribe_model::provenance::provenance(&self.full, idx);
                Ok(Ret::Data(J::Bool(prov.selections.iter().any(|s| s.feature == fd.qualified_name && s.selected))))
            }
            other => Err(format!("no helper `{other}`")),
        }
    }

    /// Evaluate, record the call, and write the file a helper produced.
    fn call(&self, func: &str, arg: Option<&str>, kw: serde_json::Map<String, J>) -> Result<Value, Error> {
        let ret = self.eval(func, arg, &kw).map_err(|m| Error::new(ErrorKind::InvalidOperation, m))?;
        let hash = ret_hash(&ret);
        let call = Call { func: func.to_string(), arg: arg.map(str::to_string), kwargs: kw, hash };
        let mut calls = self.calls.lock().unwrap_or_else(|p| p.into_inner());
        if !calls.iter().any(|c| c.func == call.func && c.arg == call.arg && c.kwargs == call.kwargs) {
            calls.push(call);
        }
        drop(calls);
        Ok(match ret {
            Ret::Text(t) => Value::from_safe_string(t),
            Ret::Data(d) => Value::from(minijinja::value::Serde(&d)),
            Ret::File { path, content } => {
                if let Some(dir) = &self.out_dir {
                    std::fs::create_dir_all(dir).and_then(|_| std::fs::write(dir.join(&path), content)).map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("cannot write {}: {e}", dir.join(&path).display())))?;
                }
                Value::from_safe_string(path)
            }
        })
    }
}

fn ret_hash(r: &Ret) -> String {
    let text = match r {
        Ret::Text(t) => t.clone(),
        Ret::Data(d) => d.to_string(),
        Ret::File { content, .. } => content.clone(),
    };
    syscribe_model::fsio::content_hash(text.as_bytes())
}

fn kwargs_json(kw: &Kwargs) -> Result<serde_json::Map<String, J>, Error> {
    let mut out = serde_json::Map::new();
    for k in kw.args() {
        let v: Value = kw.get(k)?;
        out.insert(k.to_string(), serde_json::to_value(&v).map_err(|e| Error::new(ErrorKind::InvalidOperation, e.to_string()))?);
    }
    Ok(out)
}

// ── Markdown bodies ──────────────────────────────────────────────────────────

/// An element's Markdown body in the target's language (the `body` filter).
fn body_to(target: Escape, markdown: &str) -> String {
    use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
    let md = syscribe_model::req_items::strip_anchors(markdown);
    let md = md.trim();
    match target {
        Escape::Markdown | Escape::None => return md.to_string(),
        Escape::Html => {
            let mut out = String::new();
            pulldown_cmark::html::push_html(&mut out, Parser::new_ext(md, Options::ENABLE_TABLES));
            return out.trim_end().to_string();
        }
        Escape::Latex | Escape::Typst => {}
    }
    let latex = target == Escape::Latex;
    let mut out = String::new();
    let mut verbatim = false;
    let mut lists: Vec<bool> = Vec::new(); // ordered?
    let mut link: Vec<String> = Vec::new();
    for ev in Parser::new(md) {
        match ev {
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => out.push_str(if lists.is_empty() { "\n\n" } else { "\n" }),
            Event::Start(Tag::Heading { level, .. }) => out.push_str(match (latex, level) {
                (true, HeadingLevel::H1) => "\\subsection*{",
                (true, HeadingLevel::H2) => "\\subsubsection*{",
                (true, _) => "\\paragraph{",
                (false, HeadingLevel::H1) => "== ",
                (false, HeadingLevel::H2) => "=== ",
                (false, _) => "==== ",
            }),
            Event::End(TagEnd::Heading(_)) => out.push_str(if latex { "}\n\n" } else { "\n\n" }),
            Event::Start(Tag::Emphasis) => out.push_str(if latex { "\\emph{" } else { "_" }),
            Event::End(TagEnd::Emphasis) => out.push_str(if latex { "}" } else { "_" }),
            Event::Start(Tag::Strong) => out.push_str(if latex { "\\textbf{" } else { "*" }),
            Event::End(TagEnd::Strong) => out.push_str(if latex { "}" } else { "*" }),
            Event::Start(Tag::List(first)) => {
                lists.push(first.is_some());
                if latex {
                    out.push_str(if first.is_some() { "\\begin{enumerate}\n" } else { "\\begin{itemize}\n" });
                }
            }
            Event::End(TagEnd::List(ordered)) => {
                lists.pop();
                if latex {
                    out.push_str(if ordered { "\\end{enumerate}\n" } else { "\\end{itemize}\n" });
                }
                if lists.is_empty() {
                    out.push('\n');
                }
            }
            Event::Start(Tag::Item) => {
                let ordered = lists.last().copied().unwrap_or(false);
                if latex {
                    out.push_str("\\item ");
                } else {
                    out.push_str(&"  ".repeat(lists.len().saturating_sub(1)));
                    out.push_str(if ordered { "+ " } else { "- " });
                }
            }
            Event::End(TagEnd::Item) => {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                verbatim = true;
                if latex {
                    out.push_str("\\begin{verbatim}\n");
                } else {
                    out.push_str(&format!("```{}\n", match kind {
                        CodeBlockKind::Fenced(l) => l.to_string(),
                        CodeBlockKind::Indented => String::new(),
                    }));
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                verbatim = false;
                out.push_str(if latex { "\\end{verbatim}\n\n" } else { "```\n\n" });
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                link.push(dest_url.to_string());
                out.push_str(&if latex { format!("\\href{{{dest_url}}}{{") } else { format!("#link(\"{dest_url}\")[") });
            }
            Event::End(TagEnd::Link) => {
                link.pop();
                out.push_str(if latex { "}" } else { "]" });
            }
            Event::Text(t) if verbatim => out.push_str(&t),
            Event::Text(t) => out.push_str(&target.text(&t)),
            Event::Code(c) => out.push_str(&if latex { format!("\\texttt{{{}}}", target.text(&c)) } else { format!("`{c}`") }),
            Event::SoftBreak => out.push('\n'),
            Event::HardBreak => out.push_str(if latex { "\\\\\n" } else { " \\\n" }),
            Event::Rule => out.push_str(if latex { "\\medskip\\hrule\\medskip\n\n" } else { "#line(length: 100%)\n\n" }),
            _ => {}
        }
    }
    out.trim_end().to_string()
}

// ── the engine ───────────────────────────────────────────────────────────────

/// A failed render: the message, with the template file and line when the engine knows them.
pub struct RenderError {
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.line {
            Some(l) => write!(f, "{}:{l}: {}", self.file, self.message),
            None => write!(f, "{}: {}", self.file, self.message),
        }
    }
}

/// Render the template at `template` against `model`. `extra` is added to the template's variables
/// (`document`, for a `Document`). Returns the text and the helper calls it made.
pub fn render(model: Arc<Model>, template: &Path, syntax: &TemplateSyntax, extra: &[(&str, J)]) -> Result<(String, Vec<Call>), RenderError> {
    let file = template.display().to_string();
    let fail = |line: Option<usize>, message: String| RenderError { file: file.clone(), line, message };
    let source = std::fs::read_to_string(template).map_err(|e| fail(None, format!("cannot read the template: {e}")))?;
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    let cfg = minijinja::syntax::SyntaxConfig::builder()
        .variable_delimiters(syntax.variable.0.clone(), syntax.variable.1.clone())
        .block_delimiters(syntax.block.0.clone(), syntax.block.1.clone())
        .comment_delimiters(syntax.comment.0.clone(), syntax.comment.1.clone())
        .keep_trailing_newline(true)
        .trim_blocks(true)
        .lstrip_blocks(true)
        .build()
        .map_err(|e| fail(None, format!("the template delimiters are not usable: {e}")))?;
    env.set_syntax(cfg);
    // `include`/`import` read beside the template.
    if let Some(dir) = template.parent() {
        let dir = if dir.as_os_str().is_empty() { PathBuf::from(".") } else { dir.to_path_buf() };
        env.set_loader(move |name| {
            let p = dir.join(name);
            if name.split(['/', '\\']).any(|c| c == "..") || !p.is_file() {
                return Ok(None);
            }
            std::fs::read_to_string(&p).map(Some).map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("cannot read {}: {e}", p.display())))
        });
    }
    // Every printed expression is escaped for the target unless a helper or filter made it safe.
    let target = model.target;
    env.set_formatter(move |out, _state, value| {
        if value.is_undefined() {
            return Err(Error::new(ErrorKind::UndefinedError, "the value printed here is undefined"));
        }
        if value.is_none() {
            return Ok(());
        }
        let text = if value.is_safe() { value.to_string() } else { target.text(&value.to_string()) };
        out.write_str(&text).map_err(|_| Error::new(ErrorKind::WriteFailure, "formatting failed"))
    });
    for name in ["value", "element", "view", "diagram", "selected"] {
        let m = model.clone();
        env.add_function(name, move |arg: String, kw: Kwargs| -> Result<Value, Error> {
            let json = kwargs_json(&kw)?;
            kw.assert_all_used()?;
            m.call(name, Some(&arg), json)
        });
    }
    let m = model.clone();
    env.add_function("elements", move |kw: Kwargs| -> Result<Value, Error> {
        let json = kwargs_json(&kw)?;
        kw.assert_all_used()?;
        m.call("elements", None, json)
    });
    for (name, esc) in [("latex", Escape::Latex), ("typst", Escape::Typst), ("html", Escape::Html), ("md", Escape::Markdown)] {
        env.add_filter(name, move |v: Value| Value::from_safe_string(esc.text(&v.to_string())));
    }
    // `element | body`, or a Markdown string: the body in the target's language.
    env.add_filter("body", move |v: Value| -> Result<Value, Error> {
        let md = match v.get_attr("body").ok().filter(|b| !b.is_undefined()) {
            Some(b) => b.get_attr("rendered").ok().filter(|r| !r.is_undefined()).map(|r| r.to_string()).unwrap_or_default(),
            None if v.as_str().is_some() => v.to_string(),
            None => return Err(Error::new(ErrorKind::InvalidOperation, "`body` takes an element (from element() or elements()) or a Markdown string")),
        };
        Ok(Value::from_safe_string(body_to(target, &md)))
    });
    // `q | quantity`: a `{value, unit}` pair (a parameter row, a table cell) written for the target.
    env.add_filter("quantity", move |v: Value| -> Result<Value, Error> {
        let val = v.get_attr("value").ok().filter(|x| !x.is_undefined() && !x.is_none()).ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "`quantity` takes a map with `value` (and `unit`)"))?;
        let unit = v.get_attr("unit").ok().filter(|x| !x.is_undefined() && !x.is_none()).map(|u| u.to_string());
        Ok(Value::from_safe_string(target.quantity(&val.to_string(), unit.as_deref())))
    });
    let mut ctx = serde_json::Map::new();
    ctx.insert("meta".into(), model.export["meta"].clone());
    ctx.insert("parameters".into(), model.export.get("effectiveParameters").cloned().unwrap_or_else(|| J::Array(Vec::new())));
    for (k, v) in extra {
        ctx.insert((*k).to_string(), v.clone());
    }
    let to_err = |e: Error| {
        let mut message = e.detail().map(str::to_string).unwrap_or_else(|| e.kind().to_string());
        if e.detail().is_some() && !matches!(e.kind(), ErrorKind::InvalidOperation) {
            message = format!("{}: {message}", e.kind());
        }
        // An error inside an included file names that file.
        let in_file = e.name().filter(|n| *n != file.as_str()).map(|n| format!(" (in {n})")).unwrap_or_default();
        fail(e.line(), format!("{message}{in_file}"))
    };
    let tmpl = env.template_from_named_str(&file, &source).map_err(to_err)?;
    let text = tmpl.render(Value::from(minijinja::value::Serde(&ctx))).map_err(to_err)?;
    let calls = model.calls.lock().unwrap_or_else(|p| p.into_inner()).clone();
    Ok((text, calls))
}

/// The escaping a template's result needs: that of the rendered file's name.
fn target_of(rendered: &Path, flag: Option<&str>) -> Result<Escape, String> {
    match flag {
        None => Ok(Escape::for_path(rendered)),
        Some(s) => Escape::parse(s).ok_or_else(|| format!("unknown --escape `{s}`; one of: {}", Escape::NAMES.join(", "))),
    }
}

fn write_file(dest: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = dest.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    std::fs::write(dest, text).map_err(|e| format!("cannot write {}: {e}", dest.display()))
}

pub const RENDER_USAGE: &str = "Usage: syscribe -m <root> render --template <file> [--out <file>] [--config <C>] [--escape latex|typst|html|markdown|none]";

/// `render --template <file> [--out <file>] [--config <C>] [--escape <t>]`. `view` is the model under `--config`.
pub fn cmd_render_template(full: &[RawElement], view: Vec<RawElement>, vcfg: &ValidateConfig, model_root: &Path, args: &[String]) -> i32 {
    let value = |flag: &str| args.windows(2).find(|w| w[0] == flag).map(|w| w[1].as_str());
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--template" | "--out" | "--config" | "--escape" if i + 1 < args.len() => i += 2,
            other => {
                eprintln!("render: unexpected `{other}`\n{RENDER_USAGE}");
                return 1;
            }
        }
    }
    let Some(template) = value("--template").map(Path::new) else {
        eprintln!("{RENDER_USAGE}");
        return 1;
    };
    let out = value("--out").map(PathBuf::from);
    // The result's language: the --out name, else the template name without its template suffix.
    let named = out.clone().unwrap_or_else(|| PathBuf::from(document::rendered_name(&template.display().to_string())));
    let target = match target_of(&named, value("--escape")) {
        Ok(t) => t,
        Err(m) => {
            eprintln!("render: {m}");
            return 1;
        }
    };
    let cfg = DocumentsConfig::load(model_root);
    crate::docrender::warn_config(&cfg);
    let out_dir = out.as_ref().map(|o| o.parent().filter(|p| !p.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from(".")));
    let model = match Model::new(full, view, vcfg, model_root, value("--config"), target, out_dir) {
        Ok(m) => Arc::new(m),
        Err(m) => {
            eprintln!("render: {m}");
            return 1;
        }
    };
    match render(model, template, &cfg.template_syntax_for(&named), &[]) {
        Err(e) => {
            eprintln!("{e}");
            eprintln!("render: the template did not render; nothing written");
            1
        }
        Ok((text, _)) => match out {
            None => {
                print!("{text}");
                0
            }
            Some(dest) => match write_file(&dest, &text) {
                Ok(()) => 0,
                Err(m) => {
                    eprintln!("render: {m}");
                    1
                }
            },
        },
    }
}

// ── `doc` ────────────────────────────────────────────────────────────────────

pub const DOC_USAGE: &str = "Usage: syscribe -m <root> doc list [--json]\n       syscribe -m <root> doc build <Document> | --all [--check] [--config <C>]";

/// What `doc build` records beside the rendered file, and `--check` compares against.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    document: String,
    template: String,
    template_hash: String,
    #[serde(default)]
    config: Option<String>,
    syscribe_version: String,
    rendered_hash: String,
    calls: Vec<Call>,
}

struct Paths {
    template: PathBuf,
    rendered: PathBuf,
    outputs: Vec<PathBuf>,
}

fn local(vcfg: &ValidateConfig, s: &str, what: &str) -> Result<PathBuf, String> {
    match vcfg.classify_source(s) {
        SourceLocation::Local(p) => Ok(dr::clean(&p)),
        SourceLocation::Remote(r) => Err(format!("{what} `{r}` is not a local path")),
    }
}

fn paths(vcfg: &ValidateConfig, d: &document::Decl) -> Result<Paths, String> {
    let template = local(vcfg, &d.template, "template")?;
    let rendered = match &d.rendered {
        Some(r) => local(vcfg, r, "rendered")?,
        None => {
            let name = document::rendered_name(&template.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
            template.parent().map(|p| p.join(&name)).unwrap_or_else(|| PathBuf::from(&name))
        }
    };
    let outputs = d.artefacts.iter().map(|o| local(vcfg, o, "artefact")).collect::<Result<_, _>>()?;
    Ok(Paths { template, rendered, outputs })
}

fn manifest_path(rendered: &Path) -> PathBuf {
    let mut name = rendered.file_name().map(|n| n.to_os_string()).unwrap_or_default();
    name.push(".manifest.json");
    rendered.with_file_name(name)
}

fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn document_json(e: &RawElement, d: &document::Decl) -> J {
    serde_json::json!({
        "name": e.frontmatter.name.clone().unwrap_or_else(|| e.qualified_name.clone()),
        "qname": e.qualified_name,
        "version": d.version,
        "audience": d.audience,
        "config": d.config,
        "status": e.frontmatter.status,
    })
}

/// Build or check one Document. `Ok(drift)` lists what `--check` found (empty = up to date; always
/// empty for a build). `Err` is a failure to render.
fn build_one(full: &[RawElement], vcfg: &ValidateConfig, model_root: &Path, doc: &RawElement, config_flag: Option<&str>, check: bool) -> Result<Vec<String>, String> {
    let decl = document::of(doc).map_err(|e| format!("malformed Document (E508): {}", e.join("; ")))?;
    let p = paths(vcfg, &decl)?;
    let config = config_flag.or(decl.config.as_deref());
    let view = match config {
        None => full.to_vec(),
        Some(c) => match syscribe_model::projection::resolve_config_flag(full, c) {
            syscribe_model::projection::SelectionOutcome::Resolved(sel) => syscribe_model::projection::project(full, &sel),
            syscribe_model::projection::SelectionOutcome::Dormant => full.to_vec(),
            syscribe_model::projection::SelectionOutcome::Error(m) => return Err(m),
        },
    };
    let cfg = DocumentsConfig::load(model_root);
    crate::docrender::warn_config(&cfg);
    let target = Escape::for_path(&p.rendered);
    let syntax = cfg.template_syntax_for(&p.rendered);
    let template_hash = std::fs::read(&p.template).map(|b| syscribe_model::fsio::content_hash(&b)).map_err(|e| format!("cannot read the template {}: {e}", p.template.display()))?;
    let mpath = manifest_path(&p.rendered);

    if check {
        let mut drift = Vec::new();
        let Some(manifest) = std::fs::read_to_string(&mpath).ok().and_then(|t| serde_json::from_str::<Manifest>(&t).ok()) else {
            return Ok(vec![format!("never built: no manifest at {} (run `doc build {}`)", mpath.display(), label(doc))]);
        };
        match std::fs::read(&p.rendered) {
            Err(_) => drift.push(format!("the rendered file {} is missing", p.rendered.display())),
            Ok(b) if syscribe_model::fsio::content_hash(&b) != manifest.rendered_hash => drift.push(format!("the rendered file {} was changed after it was built", p.rendered.display())),
            Ok(_) => {}
        }
        if manifest.template_hash != template_hash {
            drift.push(format!("the template {} changed", p.template.display()));
        }
        if manifest.config.as_deref() != config {
            drift.push(format!("built for {}, now {}", manifest.config.as_deref().unwrap_or("no Configuration"), config.unwrap_or("no Configuration")));
        }
        // Re-evaluate what the template asked the model for; nothing is written.
        let model = Model::new(full, view, vcfg, model_root, config, target, None)?;
        for c in &manifest.calls {
            match model.eval(&c.func, c.arg.as_deref(), &c.kwargs) {
                Err(m) => drift.push(format!("{} no longer resolves: {m}", c.show())),
                Ok(r) if ret_hash(&r) != c.hash => drift.push(match (&r, c.func.as_str()) {
                    (Ret::Text(t), "value") => format!("{} is now {t}", c.show()),
                    _ => format!("{} changed", c.show()),
                }),
                Ok(_) => {}
            }
        }
        // A final artefact (a PDF) must exist and be no older than the text it is made from.
        let mtime = |f: &Path| std::fs::metadata(f).and_then(|m| m.modified()).ok();
        for o in &p.outputs {
            match (mtime(o), mtime(&p.rendered)) {
                (None, _) => drift.push(format!("the artefact {} is missing", o.display())),
                (Some(a), Some(b)) if a < b => drift.push(format!("the artefact {} is older than {}", o.display(), p.rendered.display())),
                _ => {}
            }
        }
        return Ok(drift);
    }

    let out_dir = p.rendered.parent().filter(|d| !d.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
    let model = Arc::new(Model::new(full, view, vcfg, model_root, config, target, Some(out_dir))?);
    let (text, calls) = render(model, &p.template, &syntax, &[("document", document_json(doc, &decl))]).map_err(|e| e.to_string())?;
    write_file(&p.rendered, &text)?;
    let manifest = Manifest {
        document: label(doc),
        template: decl.template.clone(),
        template_hash,
        config: config.map(str::to_string),
        syscribe_version: env!("CARGO_PKG_VERSION").to_string(),
        rendered_hash: syscribe_model::fsio::content_hash(text.as_bytes()),
        calls,
    };
    write_file(&mpath, &format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap_or_default()))?;
    println!("{}: wrote {}", label(doc), p.rendered.display());
    Ok(Vec::new())
}

/// `args` are the tokens after `doc`. Exit: 0 ok · 1 error · 2 `--check` found a document out of date.
pub fn cmd_doc(full: &[RawElement], vcfg: &ValidateConfig, model_root: &Path, args: &[String]) -> i32 {
    let docs: Vec<&RawElement> = full.iter().filter(|e| document::is_document(e)).collect();
    match args.first().map(String::as_str) {
        Some("list") => {
            let json = args.iter().any(|a| a == "--json");
            if let Some(bad) = args[1..].iter().find(|a| *a != "--json") {
                eprintln!("doc list: unexpected `{bad}`\n{DOC_USAGE}");
                return 1;
            }
            let rows: Vec<J> = docs
                .iter()
                .map(|e| {
                    let d = document::of(e).unwrap_or_default();
                    let rendered = paths(vcfg, &d).ok().map(|p| p.rendered.display().to_string());
                    serde_json::json!({"document": label(e), "name": e.frontmatter.name, "template": d.template, "rendered": rendered, "artefacts": d.artefacts, "version": d.version, "audience": d.audience, "config": d.config})
                })
                .collect();
            if json {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({"schemaVersion": crate::export::SCHEMA_VERSION, "documents": rows})).unwrap_or_default());
            } else if rows.is_empty() {
                println!("No Document elements.");
            } else {
                println!("| Document | Version | Template | Rendered | Artefacts |\n|---|---|---|---|---|");
                for r in &rows {
                    let s = |k: &str| r[k].as_str().unwrap_or("—").to_string();
                    let outs: Vec<&str> = r["artefacts"].as_array().into_iter().flatten().filter_map(|x| x.as_str()).collect();
                    println!("| {} | {} | {} | {} | {} |", s("document"), s("version"), s("template"), s("rendered"), if outs.is_empty() { "—".to_string() } else { outs.join(", ") });
                }
            }
            0
        }
        Some("build") => {
            let rest = &args[1..];
            let check = rest.iter().any(|a| a == "--check");
            let all = rest.iter().any(|a| a == "--all");
            let config = rest.windows(2).find(|w| w[0] == "--config").map(|w| w[1].as_str());
            let mut names: Vec<&str> = Vec::new();
            let mut i = 0;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--check" | "--all" => i += 1,
                    "--config" if i + 1 < rest.len() => i += 2,
                    s if s.starts_with("--") => {
                        eprintln!("doc build: unknown option {s}\n{DOC_USAGE}");
                        return 1;
                    }
                    s => {
                        names.push(s);
                        i += 1;
                    }
                }
            }
            let resolver = Resolver::new(full);
            let targets: Vec<&RawElement> = if all {
                docs.clone()
            } else {
                let mut out = Vec::new();
                for n in &names {
                    match resolver.resolve_ref(full, n) {
                        Some(e) if document::is_document(e) => out.push(e),
                        Some(e) => {
                            eprintln!("doc build: '{n}' is a {}, not a Document", e.frontmatter.element_type.as_ref().map(|t| t.name()).unwrap_or("?"));
                            return 1;
                        }
                        None => {
                            eprintln!("doc build: '{n}' does not resolve to an element");
                            return 1;
                        }
                    }
                }
                out
            };
            if targets.is_empty() {
                eprintln!("{}", if all { "doc build: the model has no Document element" } else { DOC_USAGE });
                return 1;
            }
            let mut code = 0;
            for doc in targets {
                match build_one(full, vcfg, model_root, doc, config, check) {
                    Err(m) => {
                        eprintln!("{}: {m}", label(doc));
                        eprintln!("doc build: {} did not {}", label(doc), if check { "check" } else { "build; nothing written" });
                        code = 1;
                    }
                    Ok(drift) if check && drift.is_empty() => println!("{}: up to date", label(doc)),
                    Ok(drift) if check => {
                        println!("{}: out of date", label(doc));
                        for d in &drift {
                            println!("  - {d}");
                        }
                        if code == 0 {
                            code = 2;
                        }
                    }
                    Ok(_) => {}
                }
            }
            code
        }
        _ => {
            eprintln!("{DOC_USAGE}");
            1
        }
    }
}

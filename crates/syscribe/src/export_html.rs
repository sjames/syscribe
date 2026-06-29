//! `syscribe export-html` — generate a self-contained, offline, multi-file
//! static HTML site for the whole model (REQ-TRS-HTML-001..007).
//!
//! Every asset (stylesheet, Mermaid runtime, search) is bundled and referenced
//! by a relative path, so the output works directly from `file://` with no web
//! server and no network access. Search avoids `fetch()` (blocked for local
//! files) by embedding the element index inline in `index.html`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use syscribe_model::config::ValidateConfig;
use syscribe_model::element::{ElementType, RawElement, RawFrontmatter};
use syscribe_model::renderer::render_diagram;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::{self, Severity, ValidationResult};

use crate::coverage::coverage_summary;

/// Bundled offline assets (`include_str!` so the binary is self-contained).
const DEFAULT_CSS: &str = include_str!("../assets/export/style.css");
const SEARCH_JS: &str = include_str!("../assets/export/search.js");
const MERMAID_JS: &str = include_str!("../assets/export/mermaid.min.js");

/// The frontmatter (camelCase) keys whose values are cross-references to other
/// elements; their cells render as links when the target resolves.
const REF_KEYS: &[&str] = &[
    "supertype",
    "typedBy",
    "subsets",
    "redefines",
    "verifies",
    "derivedFrom",
    "satisfies",
    "allocatedFrom",
    "allocatedTo",
];

// ── small helpers ───────────────────────────────────────────────────────────

/// Map a qualified name to its element-page file stem.
fn sanitize(qname: &str) -> String {
    qname.replace("::", "__").replace('/', "__")
}

/// Escape text for safe inclusion in HTML (matches `ui.rs::html_escape`).
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The element-type label, e.g. `Requirement`, or `None` when untyped.
fn type_str(fm: &RawFrontmatter) -> Option<String> {
    serde_json::to_value(&fm.element_type)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
}

/// Render Markdown to HTML. Fenced ` ```mermaid ` blocks become
/// `<pre class="mermaid">…</pre>` (rendered client-side); everything else goes
/// through pulldown-cmark with tables/strikethrough/tasklists enabled. Mirrors
/// `syscribe-server`'s `ui.rs::markdown_to_html`.
fn markdown_to_html(md: &str) -> String {
    use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(md, opts);
    let mut events: Vec<Event<'_>> = Vec::new();
    let mut in_mermaid = false;

    for event in parser {
        match &event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
                if lang.as_ref() == "mermaid" =>
            {
                in_mermaid = true;
                events.push(Event::Html(CowStr::Borrowed("<pre class=\"mermaid\">")));
            }
            Event::End(TagEnd::CodeBlock) if in_mermaid => {
                in_mermaid = false;
                events.push(Event::Html(CowStr::Borrowed("</pre>")));
            }
            // Inside a mermaid block emit the raw source text (escaped by
            // push_html), not a syntax-highlighted code element.
            Event::Text(t) if in_mermaid => {
                events.push(Event::Html(CowStr::Boxed(esc(t).into_boxed_str())));
            }
            _ => events.push(event),
        }
    }

    let mut out = String::with_capacity(md.len() * 2);
    html::push_html(&mut out, events.into_iter());
    out
}

/// Recursively drop `null` object members (so only set frontmatter shows).
fn strip_nulls(v: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match v {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(_, val)| !val.is_null())
                .map(|(k, val)| (k, strip_nulls(val)))
                .collect(),
        ),
        Value::Array(arr) => Value::Array(arr.into_iter().map(strip_nulls).collect()),
        other => other,
    }
}

/// Render a non-reference JSON frontmatter value as escaped display text.
fn render_value(v: &serde_json::Value) -> String {
    use serde_json::Value;
    match v {
        Value::String(s) => esc(s),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => String::new(),
        Value::Array(arr) => arr.iter().map(render_value).collect::<Vec<_>>().join(", "),
        Value::Object(_) => esc(&serde_json::to_string(v).unwrap_or_default()),
    }
}

/// Collect the string cross-references from a JSON value (a string or an array
/// of strings / objects with a string member).
fn ref_strings(v: &serde_json::Value) -> Vec<String> {
    use serde_json::Value;
    match v {
        Value::String(s) => vec![s.clone()],
        Value::Array(arr) => arr.iter().flat_map(ref_strings).collect(),
        _ => Vec::new(),
    }
}

/// Render a set of cross-references as a comma-joined cell: each reference that
/// resolves becomes a link to its element page; unresolved ones stay plain text.
fn render_refs(
    refs: &[String],
    elements: &[RawElement],
    resolver: &Resolver,
    rel_root: &str,
) -> String {
    refs.iter()
        .map(|r| match resolver.resolve_ref(elements, r) {
            Some(target) => format!(
                "<a class=\"refs\" href=\"{}elements/{}.html\">{}</a>",
                rel_root,
                sanitize(&target.qualified_name),
                esc(r)
            ),
            None => format!("<span class=\"unresolved\">{}</span>", esc(r)),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

// ── navigation ───────────────────────────────────────────────────────────────

/// Build the `<nav class="sitenav">` namespace tree, shared by every page.
/// `current` marks the active element page (empty for non-element pages).
fn build_nav(elements: &[RawElement], rel_root: &str, current: &str) -> String {
    let mut qnames: Vec<&str> = elements.iter().map(|e| e.qualified_name.as_str()).collect();
    qnames.sort_unstable();

    let mut out = String::from("<nav class=\"sitenav\">\n<h2>Model</h2>\n<ul>\n");
    out.push_str(&format!(
        "<li><a href=\"{rel_root}index.html\" class=\"pkg\">Overview</a></li>\n"
    ));
    for q in qnames {
        let depth = q.matches("::").count();
        let leaf = q.rsplit("::").next().unwrap_or(q);
        let active = if q == current { " active" } else { "" };
        out.push_str(&format!(
            "<li style=\"padding-left:{}rem\"><a class=\"el{active}\" href=\"{rel_root}elements/{}.html\">{}</a></li>\n",
            depth as f32 * 0.6,
            sanitize(q),
            esc(leaf),
        ));
    }
    out.push_str("</ul>\n<h2>Reports</h2>\n<ul>\n");
    for (name, label) in [
        ("validation", "Validation"),
        ("coverage", "Coverage"),
        ("traceability", "Traceability"),
    ] {
        out.push_str(&format!(
            "<li><a href=\"{rel_root}reports/{name}.html\">{label}</a></li>\n"
        ));
    }
    out.push_str("</ul>\n</nav>\n");
    out
}

// ── page shell ───────────────────────────────────────────────────────────────

/// Wrap a body in the full HTML document, including the shared stylesheet and
/// the offline Mermaid runtime + init. `head_extra` / `tail_extra` allow the
/// index page to inject the inline search index and `search.js`.
fn page_shell(
    title: &str,
    rel_root: &str,
    nav: &str,
    body: &str,
    head_extra: &str,
    tail_extra: &str,
) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n\
<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>{title}</title>\n\
<link rel=\"stylesheet\" href=\"{rel_root}style.css\">\n{head_extra}</head>\n\
<body>\n<div class=\"layout\">\n{nav}<main class=\"content\">\n{body}\n\
<footer>Generated by syscribe export-html.</footer>\n</main>\n</div>\n\
<script src=\"{rel_root}mermaid.min.js\"></script>\n\
<script>mermaid.initialize({{startOnLoad:true}});</script>\n{tail_extra}</body>\n</html>\n",
        title = esc(title),
    )
}

// ── element pages ─────────────────────────────────────────────────────────────

fn element_page(
    elem: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    result: &ValidationResult,
    config: &ValidateConfig,
) -> String {
    let rel_root = "../";
    let fm = &elem.frontmatter;
    let ty = type_str(fm).unwrap_or_else(|| "Element".to_string());
    let name = fm.name.clone().unwrap_or_else(|| elem.qualified_name.clone());

    let mut body = String::new();
    body.push_str(&format!(
        "<h1>{} <span class=\"badge badge-{}\">{}</span></h1>\n",
        esc(&name),
        esc(&ty.to_lowercase()),
        esc(&ty),
    ));

    // Identity / status line.
    body.push_str("<div class=\"meta\"><code>");
    body.push_str(&esc(&elem.qualified_name));
    body.push_str("</code>");
    if let Some(id) = &fm.id {
        body.push_str(&format!(" · id <code>{}</code>", esc(id)));
    }
    if let Some(status) = &fm.status {
        body.push_str(&format!(" · status {}", esc(status)));
    }
    if let Some(url) = config.hosted_url_for(
        &elem.file_path,
        &elem.qualified_name,
        fm.id.as_deref().unwrap_or(""),
    ) {
        body.push_str(&format!(" · <a href=\"{}\">source</a>", esc(&url)));
    }
    body.push_str("</div>\n");

    // Frontmatter table.
    let fm_json = strip_nulls(serde_json::to_value(fm).unwrap_or(serde_json::Value::Null));
    if let serde_json::Value::Object(map) = &fm_json {
        let mut rows = String::new();
        for (key, val) in map {
            // Identity already shown in the header / meta line.
            if matches!(key.as_str(), "type" | "name" | "id" | "status") {
                continue;
            }
            let cell = if REF_KEYS.contains(&key.as_str()) {
                render_refs(&ref_strings(val), elements, resolver, rel_root)
            } else {
                render_value(val)
            };
            if cell.is_empty() {
                continue;
            }
            rows.push_str(&format!(
                "<tr><td>{}</td><td>{}</td></tr>\n",
                esc(key),
                cell
            ));
        }
        if !rows.is_empty() {
            body.push_str("<h2>Frontmatter</h2>\n<table class=\"fm-table\">\n");
            body.push_str(&rows);
            body.push_str("</table>\n");
        }
    }

    // Computed reverse indices, keyed by stable id else qname.
    let key = fm.id.as_deref().unwrap_or(elem.qualified_name.as_str());
    let mut computed = String::new();
    let mut add_computed = |label: &str, list: Option<&Vec<String>>| {
        if let Some(items) = list {
            if !items.is_empty() {
                computed.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td></tr>\n",
                    label,
                    render_refs(items, elements, resolver, rel_root)
                ));
            }
        }
    };
    add_computed("verifiedBy", result.verified_by.get(key));
    add_computed("derivedChildren", result.derived_children.get(key));
    add_computed("allocatedFrom", result.allocated_from.get(key));
    if !computed.is_empty() {
        body.push_str("<h2>Computed references</h2>\n<table class=\"fm-table\">\n");
        body.push_str(&computed);
        body.push_str("</table>\n");
    }

    // Documentation (rendered Markdown — also yields mermaid <pre> blocks).
    if !elem.doc.trim().is_empty() {
        body.push_str("<div class=\"doc\">\n");
        body.push_str(&markdown_to_html(&elem.doc));
        body.push_str("</div>\n");
    }

    // Diagrams: Mermaid is already in the rendered doc; otherwise inline the SVG.
    if matches!(fm.element_type, Some(ElementType::Diagram))
        && fm.diagram_kind.as_deref() != Some("Mermaid")
    {
        if let Some(svg) = render_diagram(elem, resolver, elements) {
            body.push_str("<div class=\"diagram\">\n");
            body.push_str(&svg);
            body.push_str("\n</div>\n");
        }
    }

    let nav = build_nav(elements, rel_root, &elem.qualified_name);
    page_shell(&name, rel_root, &nav, &body, "", "")
}

// ── reports ────────────────────────────────────────────────────────────────

fn validation_report(
    elements: &[RawElement],
    result: &ValidationResult,
    file_to_qname: &BTreeMap<&str, &str>,
) -> String {
    let rel_root = "../";
    let mut body = String::from("<h1>Validation</h1>\n");
    body.push_str(&format!(
        "<p>{} finding(s).</p>\n",
        result.findings.len()
    ));
    body.push_str("<table>\n<tr><th>Code</th><th>Severity</th><th>File</th><th>Message</th></tr>\n");
    for f in &result.findings {
        let sev = match f.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        let file_cell = match file_to_qname.get(f.file.as_str()) {
            Some(q) => format!(
                "<a href=\"{}elements/{}.html\">{}</a>",
                rel_root,
                sanitize(q),
                esc(&f.file)
            ),
            None => esc(&f.file),
        };
        body.push_str(&format!(
            "<tr><td><code>{}</code></td><td class=\"severity-{}\">{}</td><td>{}</td><td>{}</td></tr>\n",
            esc(f.code),
            sev,
            sev,
            file_cell,
            esc(&f.message),
        ));
    }
    body.push_str("</table>\n");
    let nav = build_nav(elements, rel_root, "");
    page_shell("Validation", rel_root, &nav, &body, "", "")
}

fn coverage_report(elements: &[RawElement], result: &ValidationResult) -> String {
    let rel_root = "../";
    let summary = coverage_summary(elements, result);
    let mut body = String::from("<h1>Coverage</h1>\n");
    body.push_str(&format!(
        "<p>Verification coverage: <strong>{}</strong> verified requirement(s), \
{} unverified leaf gap(s), {} parent(s) missing an integration test.</p>\n",
        summary.verified_count,
        summary.unverified_leaves.len(),
        summary.parents_missing_integration.len(),
    ));

    let link = |qname: &str, label: &str| -> String {
        format!(
            "<a href=\"{}elements/{}.html\">{}</a>",
            rel_root,
            sanitize(qname),
            esc(label)
        )
    };

    body.push_str("<h2>Unverified leaf requirements</h2>\n");
    if summary.unverified_leaves.is_empty() {
        body.push_str("<p>None.</p>\n");
    } else {
        body.push_str("<table>\n<tr><th>Id</th><th>Name</th></tr>\n");
        for e in &summary.unverified_leaves {
            body.push_str(&format!(
                "<tr><td>{}</td><td>{}</td></tr>\n",
                link(&e.qname, e.id.as_deref().unwrap_or(&e.qname)),
                esc(e.name.as_deref().unwrap_or("")),
            ));
        }
        body.push_str("</table>\n");
    }

    body.push_str("<h2>Parents missing an integration test</h2>\n");
    if summary.parents_missing_integration.is_empty() {
        body.push_str("<p>None.</p>\n");
    } else {
        body.push_str("<table>\n<tr><th>Id</th><th>Name</th><th>Children</th></tr>\n");
        for e in &summary.parents_missing_integration {
            body.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                link(&e.qname, e.id.as_deref().unwrap_or(&e.qname)),
                esc(e.name.as_deref().unwrap_or("")),
                e.child_count.unwrap_or(0),
            ));
        }
        body.push_str("</table>\n");
    }

    let nav = build_nav(elements, rel_root, "");
    page_shell("Coverage", rel_root, &nav, &body, "", "")
}

fn traceability_report(
    elements: &[RawElement],
    resolver: &Resolver,
    result: &ValidationResult,
) -> String {
    let rel_root = "../";
    let mut body = String::from("<h1>Traceability</h1>\n");
    body.push_str(
        "<table>\n<tr><th>Requirement</th><th>Status</th><th>Derived from</th>\
<th>Verified by</th></tr>\n",
    );
    for e in elements {
        if !Resolver::is_native_requirement(e) {
            continue;
        }
        let fm = &e.frontmatter;
        let id = fm.id.as_deref().unwrap_or("");
        let req_cell = format!(
            "<a href=\"{}elements/{}.html\">{}</a>",
            rel_root,
            sanitize(&e.qualified_name),
            esc(if id.is_empty() { e.qualified_name.as_str() } else { id }),
        );
        let derived = fm
            .derived_from
            .as_ref()
            .map(|d| render_refs(d, elements, resolver, rel_root))
            .unwrap_or_default();
        let verified = result
            .verified_by
            .get(id)
            .map(|v| render_refs(v, elements, resolver, rel_root))
            .unwrap_or_default();
        body.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            req_cell,
            esc(fm.status.as_deref().unwrap_or("")),
            derived,
            verified,
        ));
    }
    body.push_str("</table>\n");
    let nav = build_nav(elements, rel_root, "");
    page_shell("Traceability", rel_root, &nav, &body, "", "")
}

// ── search index + index page ────────────────────────────────────────────────

/// Build the search-index JSON array (`{qname,id,type,name,url}` per element).
/// `url` is relative to the site root (where index.html lives).
fn search_index(elements: &[RawElement]) -> Vec<serde_json::Value> {
    elements
        .iter()
        .map(|e| {
            let fm = &e.frontmatter;
            serde_json::json!({
                "qname": e.qualified_name,
                "id": fm.id,
                "type": type_str(fm),
                "name": fm.name,
                "url": format!("elements/{}.html", sanitize(&e.qualified_name)),
            })
        })
        .collect()
}

fn index_page(elements: &[RawElement], index_json: &str) -> String {
    let rel_root = "";
    let mut body = String::from("<h1>Model</h1>\n");
    body.push_str(&format!(
        "<p>{} element(s). Use the search box or the navigation tree.</p>\n",
        elements.len()
    ));
    body.push_str("<input id=\"search-box\" class=\"search-box\" type=\"text\" placeholder=\"Search elements…\">\n");
    body.push_str("<ul id=\"search-results\" class=\"search-results\"></ul>\n");

    // Embed the index inline so search works from file:// (no fetch()).
    let head_extra = format!("<script>window.SEARCH_INDEX={index_json};</script>\n");
    let tail_extra = "<script src=\"search.js\"></script>\n".to_string();
    let nav = build_nav(elements, rel_root, "");
    page_shell("Model", rel_root, &nav, &body, &head_extra, &tail_extra)
}

// ── command entry point ──────────────────────────────────────────────────────

/// Parse `--out <dir>` (default `html`) and `--css <file>` from the args.
fn parse_args(args: &[String]) -> (PathBuf, Option<PathBuf>) {
    let mut out = PathBuf::from("html");
    let mut css = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                if let Some(v) = args.get(i + 1) {
                    out = PathBuf::from(v);
                    i += 1;
                }
            }
            "--css" => {
                if let Some(v) = args.get(i + 1) {
                    css = Some(PathBuf::from(v));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    (out, css)
}

/// `export-html` subcommand entry point. Writes the full static site and prints
/// the output directory and page count. Exits the process non-zero on an I/O
/// failure so a broken export does not look successful in CI.
pub fn cmd_export_html(
    elements: &[RawElement],
    resolver: &Resolver,
    config: &ValidateConfig,
    args: &[String],
) {
    let (out_dir, css) = parse_args(args);
    let result = validator::validate_with_config(elements, config);

    if let Err(e) = write_site(elements, resolver, config, &result, &out_dir, css.as_deref()) {
        eprintln!("Error: export-html failed: {e}");
        std::process::exit(1);
    }

    // index + one per element + three reports.
    let pages = 1 + elements.len() + 3;
    println!("Wrote {} pages to {}", pages, out_dir.display());
}

#[allow(clippy::too_many_arguments)]
fn write_site(
    elements: &[RawElement],
    resolver: &Resolver,
    config: &ValidateConfig,
    result: &ValidationResult,
    out_dir: &Path,
    css: Option<&Path>,
) -> std::io::Result<()> {
    let elements_dir = out_dir.join("elements");
    let reports_dir = out_dir.join("reports");
    std::fs::create_dir_all(&elements_dir)?;
    std::fs::create_dir_all(&reports_dir)?;

    // Assets.
    match css {
        Some(path) => std::fs::write(out_dir.join("style.css"), std::fs::read(path)?)?,
        None => std::fs::write(out_dir.join("style.css"), DEFAULT_CSS)?,
    }
    std::fs::write(out_dir.join("mermaid.min.js"), MERMAID_JS)?;
    std::fs::write(out_dir.join("search.js"), SEARCH_JS)?;

    // Search index (standalone JSON + inline-embedded in index.html).
    let index = search_index(elements);
    let index_json = serde_json::to_string(&index).unwrap_or_else(|_| "[]".to_string());
    std::fs::write(out_dir.join("search-index.json"), &index_json)?;

    // Index page.
    std::fs::write(out_dir.join("index.html"), index_page(elements, &index_json))?;

    // Element pages.
    for elem in elements {
        let file = elements_dir.join(format!("{}.html", sanitize(&elem.qualified_name)));
        std::fs::write(file, element_page(elem, elements, resolver, result, config))?;
    }

    // Reports.
    let file_to_qname: BTreeMap<&str, &str> = elements
        .iter()
        .map(|e| (e.file_path.as_str(), e.qualified_name.as_str()))
        .collect();
    std::fs::write(
        reports_dir.join("validation.html"),
        validation_report(elements, result, &file_to_qname),
    )?;
    std::fs::write(
        reports_dir.join("coverage.html"),
        coverage_report(elements, result),
    )?;
    std::fs::write(
        reports_dir.join("traceability.html"),
        traceability_report(elements, resolver, result),
    )?;

    Ok(())
}

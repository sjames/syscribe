use axum::{
    extract::{Path, Query, State},
    response::Html,
};
use askama::Template;
use serde::Deserialize;
use syscribe_model::renderer::render_diagram;
use crate::state::SharedState;

/// Extract the content of the first fenced code block with the given language tag.
/// Looks for ` ```{lang} ` … ` ``` ` and returns the interior (without the fence lines).
fn extract_fenced_block(doc: &str, lang: &str) -> Option<String> {
    let open_tag = format!("```{}", lang);
    let start = doc.find(&open_tag)?;
    // Advance past the opening fence line (to the newline after the tag).
    let after_open = &doc[start + open_tag.len()..];
    // Skip optional trailing characters on the fence line (e.g. a space or alias).
    let newline = after_open.find('\n')?;
    let content_start = &after_open[newline + 1..];
    // Find the closing ``` on its own line.
    let end = content_start.find("\n```")?;
    Some(content_start[..end].to_string())
}

/// Escape characters that are special in HTML so mermaid source is safe inside <pre>.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Render Markdown to HTML.  Fenced ` ```mermaid ` blocks are emitted as
/// `<pre class="mermaid">…</pre>` so that Mermaid.js can render them
/// client-side.  All other content goes through pulldown-cmark's standard
/// HTML renderer with tables and strikethrough enabled.
fn markdown_to_html(md: &str) -> String {
    use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, html};

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
                events.push(Event::Html(CowStr::Borrowed(
                    "<pre class=\"mermaid\">",
                )));
            }
            Event::End(TagEnd::CodeBlock) if in_mermaid => {
                in_mermaid = false;
                events.push(Event::Html(CowStr::Borrowed("</pre>")));
            }
            _ => {
                events.push(event);
            }
        }
    }

    let mut out = String::with_capacity(md.len() * 2);
    html::push_html(&mut out, events.into_iter());
    out
}

#[derive(Debug)]
pub struct TreeNode {
    pub qualified_name: String,
    pub display_name: String,
    pub element_type: String,
    pub is_package: bool,
    pub is_diagram: bool,
    pub url_path: String,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}

#[derive(Template)]
#[template(path = "tree_items.html")]
pub struct TreeItemsTemplate {
    pub items: Vec<TreeNode>,
}

#[derive(Template)]
#[template(path = "element_detail.html")]
pub struct ElementDetailTemplate {
    pub name: String,
    pub element_type: String,
    pub qualified_name: String,
    pub elem_id: String,
    pub doc_html: String,  // rendered HTML, emitted with |safe
    pub doc_raw: String,   // raw markdown, used in the edit textarea
    pub badge_class: String,
    /// Custom fields (GH #39) — read-only key/value pairs for the detail panel.
    /// Each value is pre-rendered (scalars inline, lists comma-joined). Empty when
    /// the element declares no `custom_fields`. Not editable via the PUT editor.
    pub custom_fields: Vec<(String, String)>,
}

/// Render a single YAML scalar (string/number/bool/null) as plain text.
fn yaml_scalar_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::Null => "null".to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => s.clone(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
    }
}

/// Render a custom-field value for the detail panel: scalars inline, lists joined.
fn custom_field_display(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::Sequence(items) => items
            .iter()
            .map(yaml_scalar_string)
            .collect::<Vec<_>>()
            .join(", "),
        other => yaml_scalar_string(other),
    }
}

#[derive(Template)]
#[template(path = "canvas.html")]
pub struct CanvasTemplate {
    pub title: String,
}

#[derive(Deserialize)]
pub struct TreeQuery {
    pub parent: Option<String>,
}

fn badge_class_for(element_type: &str) -> String {
    if element_type.ends_with("Def") {
        "badge-def".to_string()
    } else if element_type == "Requirement"
        || element_type.starts_with("Requirement")
    {
        "badge-req".to_string()
    } else if element_type == "Package"
        || element_type == "LibraryPackage"
        || element_type == "Namespace"
    {
        "badge-pkg".to_string()
    } else {
        "badge-act".to_string()
    }
}

pub async fn index() -> Html<String> {
    let tmpl = IndexTemplate {};
    Html(tmpl.render().unwrap_or_default())
}

pub async fn canvas() -> Html<String> {
    let tmpl = CanvasTemplate {
        title: "Model Graph Canvas".to_string(),
    };
    Html(tmpl.render().unwrap_or_default())
}

pub async fn tree_items(
    State(state): State<SharedState>,
    Query(query): Query<TreeQuery>,
) -> Html<String> {
    let store = state.read().await;
    let items: Vec<TreeNode> = store
        .elements
        .iter()
        .filter(|e| {
            let qn = &e.qualified_name;
            if let Some(ref parent) = query.parent {
                // Children: qualified names that start with parent:: and have
                // exactly one more segment.
                let prefix = format!("{}::", parent);
                if qn.starts_with(&prefix) {
                    let rest = &qn[prefix.len()..];
                    !rest.contains("::")
                } else {
                    false
                }
            } else {
                // Top level: no "::" in qualified name
                !qn.contains("::")
            }
        })
        .map(|e| {
            let et = e
                .frontmatter
                .element_type
                .as_ref()
                .map(|t| format!("{:?}", t))
                .unwrap_or_default();
            let is_package = matches!(
                et.as_str(),
                "Package" | "LibraryPackage" | "Namespace"
            );
            let is_diagram = et == "Diagram";
            let url_path = e.qualified_name.replace("::", "/");
            TreeNode {
                display_name: e
                    .frontmatter
                    .name
                    .clone()
                    .unwrap_or_else(|| e.qualified_name.clone()),
                qualified_name: e.qualified_name.clone(),
                element_type: et,
                is_package,
                is_diagram,
                url_path,
            }
        })
        .collect();

    let tmpl = TreeItemsTemplate { items };
    Html(tmpl.render().unwrap_or_default())
}

pub async fn element_detail(
    State(state): State<SharedState>,
    Path(qname): Path<String>,
) -> Html<String> {
    let store = state.read().await;
    let qname_norm = qname.replace('/', "::");
    match store.elements.iter().find(|e| e.qualified_name == qname_norm) {
        None => Html(r#"<p class="detail-empty">Element not found.</p>"#.to_string()),
        Some(e) => {
            let element_type = e
                .frontmatter
                .element_type
                .as_ref()
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|| "Unknown".to_string());
            let badge_class = badge_class_for(&element_type);
            let tmpl = ElementDetailTemplate {
                name: e
                    .frontmatter
                    .name
                    .clone()
                    .unwrap_or_else(|| e.qualified_name.clone()),
                element_type,
                qualified_name: e.qualified_name.clone(),
                elem_id: e.frontmatter.id.clone().unwrap_or_default(),
                doc_html: markdown_to_html(e.doc.trim()),
                doc_raw: e.doc.trim().to_string(),
                badge_class,
                custom_fields: e
                    .frontmatter
                    .custom_fields
                    .iter()
                    .map(|(k, v)| (k.clone(), custom_field_display(v)))
                    .collect(),
            };
            Html(tmpl.render().unwrap_or_default())
        }
    }
}

pub async fn diagram(
    State(state): State<SharedState>,
    Path(qname): Path<String>,
) -> Html<String> {
    let store = state.read().await;
    let qname_norm = qname.replace('/', "::");
    let element = match store.elements.iter().find(|e| e.qualified_name == qname_norm) {
        None => return Html(r#"<p class="diagram-empty">Diagram not found.</p>"#.to_string()),
        Some(e) => e,
    };

    let is_diagram = element
        .frontmatter
        .element_type
        .as_ref()
        .map(|t| format!("{:?}", t) == "Diagram")
        .unwrap_or(false);
    if !is_diagram {
        return Html(
            r#"<p class="diagram-empty">No diagram for this element.</p>"#.to_string(),
        );
    }

    let kind = element.frontmatter.diagram_kind.as_deref().unwrap_or("SVG");
    match kind {
        "Mermaid" => {
            let src = extract_fenced_block(&element.doc, "mermaid").unwrap_or_default();
            Html(format!(
                r#"<div class="diagram-mermaid-wrapper"><pre class="mermaid">{}</pre></div>"#,
                html_escape(&src)
            ))
        }
        _ => {
            // SVG / BDD / IBD / StateMachine — existing path
            match render_diagram(element, &store.resolver, &store.elements) {
                Some(svg) => Html(format!(
                    r#"<div class="diagram-svg-wrapper">{}</div>"#,
                    svg
                )),
                None => Html(
                    r#"<p class="diagram-empty">No layout defined for this diagram.</p>"#
                        .to_string(),
                ),
            }
        }
    }
}


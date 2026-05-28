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
    pub doc: String,
    pub badge_class: String,
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
                doc: e.doc.trim().to_string(),
                badge_class,
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


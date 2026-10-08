use axum::{
    extract::{Path, Query, State},
    response::Html,
};
use askama::Template;
use serde::Deserialize;
use syscribe_model::vis::DEFAULT_DIAGRAM_KIND;
use syscribe_model::frontmatter::split_frontmatter;
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
    /// `diagramKind` frontmatter (e.g. `Mermaid`, `BDD`, `Requirement`, …),
    /// defaulted to `"SVG"` like `routes::ui::diagram`'s own fallback. Lets
    /// the client (`openDiagram` in `base.html`) decide, without an extra
    /// round trip, whether to mount the sprotty editor (any non-`Mermaid`
    /// kind, `REQ-TRS-DE-004`) or keep using the legacy Mermaid HTML path.
    pub diagram_kind: String,
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
    /// REQ-TRS-LINK-005 — the element's resolved hosted source URL, or `None`.
    /// `Some` only when `[links]` is configured and the element is file-backed;
    /// drives the conditional "view source" icon in the detail panel header.
    pub source_url: Option<String>,
    /// Phase 8 (`ADR-SYS-DE-001` follow-on) — the element's on-disk
    /// frontmatter, minus `name` (edited via its own field), rendered as YAML
    /// text for the edit-mode `#edit-extra` textarea. Empty when the file has
    /// no frontmatter or couldn't be read.
    pub extra_yaml: String,
    /// REQ-TRS-PKG-001 (GH #120) — whether to render the generated "Members"
    /// list: always for a package, else only when the element owns children.
    pub show_members: bool,
    /// The element's direct members (`syscribe_model::members::direct_members`),
    /// sorted by qualified name — generated from the directory tree, never from
    /// `_index.md` prose. Each row links to that member's detail panel.
    pub members: Vec<MemberRow>,
}

/// One row of the detail panel's generated member list (REQ-TRS-PKG-001).
pub struct MemberRow {
    pub qualified_name: String,
    /// Stable id, else qualified name.
    pub label: String,
    pub element_type: String,
    pub name: String,
    pub status: String,
}

/// Serialize an element's on-disk frontmatter minus the `name` key as YAML
/// text, for the edit-mode `#edit-extra` textarea. Reads the file directly
/// (rather than re-serializing the typed `RawFrontmatter`, which would round-
/// trip every unset field as an explicit `null`) so the textarea shows
/// exactly what's on disk today.
fn extra_frontmatter_yaml(file_path: &str) -> String {
    let Ok(content) = std::fs::read_to_string(file_path) else {
        return String::new();
    };
    let (fm_opt, _) = split_frontmatter(&content);
    let mut yaml_val: serde_yaml::Value = match fm_opt {
        Some(s) => serde_yaml::from_str(s)
            .unwrap_or_else(|_| serde_yaml::Value::Mapping(serde_yaml::Mapping::new())),
        None => serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
    };
    if let serde_yaml::Value::Mapping(ref mut map) = yaml_val {
        map.remove("name");
    }
    serde_yaml::to_string(&yaml_val).unwrap_or_default()
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

pub async fn tree_items(
    State(state): State<SharedState>,
    Query(query): Query<TreeQuery>,
) -> Html<String> {
    let store = state.read().await;
    let mut ordered: Vec<&syscribe_model::element::RawElement> = store.elements.iter().collect();
    // REQ-TRS-ORDER-001 — present siblings by displayOrder (ascending, unset last),
    // tie-broken by qualified name; the walker's file order is otherwise arbitrary.
    ordered.sort_by(|a, b| {
        a.frontmatter
            .display_order_key()
            .total_cmp(&b.frontmatter.display_order_key())
            .then_with(|| a.qualified_name.cmp(&b.qualified_name))
    });
    let items: Vec<TreeNode> = ordered
        .into_iter()
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
            let diagram_kind = e
                .frontmatter
                .diagram_kind
                .clone()
                .unwrap_or_else(|| DEFAULT_DIAGRAM_KIND.to_string());
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
                diagram_kind,
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
            // REQ-TRS-LINK-005 — resolve the hosted source URL for the
            // "view source" icon. `None` when `[links]` is unconfigured or the
            // element is not file-backed, in which case no icon is rendered.
            let source_url = if e.file_path.is_empty() {
                None
            } else {
                store.config.hosted_url_for(
                    &e.file_path,
                    &e.qualified_name,
                    e.frontmatter.id.as_deref().unwrap_or(""),
                )
            };
            // REQ-TRS-PKG-001 — generated member list (same helper as `show`).
            let members: Vec<MemberRow> = syscribe_model::members::direct_members(&store.elements, &e.qualified_name)
                .into_iter()
                .map(|m| MemberRow {
                    qualified_name: m.qualified_name.clone(),
                    label: syscribe_model::members::member_label(m).to_string(),
                    element_type: m
                        .frontmatter
                        .element_type
                        .as_ref()
                        .map(|t| format!("{:?}", t))
                        .unwrap_or_else(|| "Unknown".to_string()),
                    name: m.frontmatter.name.clone().unwrap_or_else(|| "—".to_string()),
                    status: m.frontmatter.status.clone().unwrap_or_else(|| "—".to_string()),
                })
                .collect();
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
                source_url,
                extra_yaml: extra_frontmatter_yaml(&e.file_path),
                show_members: syscribe_model::members::is_package(e) || !members.is_empty(),
                members,
            };
            Html(tmpl.render().unwrap_or_default())
        }
    }
}

/// One inline feature, as shown on an element card (`REQ-TRS-VIS-026`).
pub struct FeatureCard {
    pub name: String,
    pub owner: String,
    /// Declared properties (`type`, `typedBy`, `direction`, `multiplicity`, `unit`), as written.
    pub props: Vec<(String, String)>,
    /// When the documentation shown is the feature's type's rather than its owner's.
    pub doc_from: Option<String>,
}

#[derive(Template)]
#[template(path = "element_card.html")]
pub struct ElementCardTemplate {
    pub found: bool,
    /// What was asked for, shown when it names nothing.
    pub reference: String,
    pub name: String,
    pub element_type: String,
    pub badge_class: String,
    pub qualified_name: String,
    pub elem_id: String,
    pub status: String,
    pub doc_html: String,
    pub feature: Option<FeatureCard>,
}

/// The inline feature `name` declared in `owner`'s `features:`, as `(properties, typedBy)`.
fn declared_feature(owner: &syscribe_model::element::RawElement, name: &str) -> Option<(Vec<(String, String)>, Option<String>)> {
    let list = owner.frontmatter.features.as_ref()?;
    for v in list {
        let serde_yaml::Value::Mapping(m) = v else { continue };
        let get = |k: &str| m.get(serde_yaml::Value::String(k.into())).and_then(|x| match x {
            serde_yaml::Value::String(s) => Some(s.clone()),
            serde_yaml::Value::Number(n) => Some(n.to_string()),
            serde_yaml::Value::Bool(b) => Some(b.to_string()),
            _ => None,
        });
        if get("name").as_deref() != Some(name) {
            continue;
        }
        let props: Vec<(String, String)> = ["type", "typedBy", "direction", "multiplicity", "unit"]
            .iter()
            .filter_map(|k| get(k).map(|v| (k.to_string(), v)))
            .collect();
        let typed_by = get("typedBy");
        return Some((props, typed_by));
    }
    None
}

/// `GET /ui/element-card/{*qname}` — the read-only card the diagram editor's side
/// panel shows for a clicked shape or edge (`REQ-TRS-VIS-026`): identity and the
/// element's Markdown body rendered. A reference that names an inline feature
/// (a port, part usage or attribute) resolves to its owner and shows the
/// feature's declared properties; one that names nothing says so.
pub async fn element_card(
    State(state): State<SharedState>,
    Path(qname): Path<String>,
) -> Html<String> {
    let store = state.read().await;
    let reference = qname.replace('/', "::");
    let find = |q: &str| store.elements.iter().find(|e| e.qualified_name == q);

    let card = |e: &syscribe_model::element::RawElement, feature: Option<FeatureCard>, doc_of: &syscribe_model::element::RawElement| {
        let element_type = e.frontmatter.element_type.as_ref().map(|t| format!("{:?}", t)).unwrap_or_else(|| "Unknown".to_string());
        ElementCardTemplate {
            found: true,
            reference: reference.clone(),
            name: e.frontmatter.name.clone().unwrap_or_else(|| e.qualified_name.rsplit("::").next().unwrap_or(&e.qualified_name).to_string()),
            badge_class: badge_class_for(&element_type),
            element_type,
            qualified_name: e.qualified_name.clone(),
            elem_id: e.frontmatter.id.clone().unwrap_or_default(),
            status: e.frontmatter.status.clone().unwrap_or_default(),
            doc_html: markdown_to_html(doc_of.doc.trim()),
            feature,
        }
    };

    if let Some(e) = find(&reference) {
        return Html(card(e, None, e).render().unwrap_or_default());
    }

    // An inline feature: walk up to the nearest owning element, then down its
    // `features:` (following `typedBy:` for a nested path such as `battery::powerOut`).
    let segs: Vec<&str> = reference.split("::").collect();
    for split in (1..segs.len()).rev() {
        let owner_q = segs[..split].join("::");
        let Some(owner) = find(&owner_q) else { continue };
        let mut declaring = owner;
        let mut found: Option<(String, Vec<(String, String)>, Option<String>, String)> = None;
        for (i, seg) in segs[split..].iter().enumerate() {
            let Some((props, typed_by)) = declared_feature(declaring, seg) else {
                found = None;
                break;
            };
            found = Some((seg.to_string(), props, typed_by.clone(), declaring.qualified_name.clone()));
            if i + 1 < segs[split..].len() {
                match typed_by.as_deref().and_then(|t| store.resolver.resolve_ref(&store.elements, t)) {
                    Some(next) => declaring = next,
                    None => {
                        found = None;
                        break;
                    }
                }
            }
        }
        if let Some((fname, props, typed_by, declared_in)) = found {
            let typed = typed_by.as_deref().and_then(|t| store.resolver.resolve_ref(&store.elements, t));
            let (doc_of, doc_from) = match typed {
                Some(t) => (t, Some(t.qualified_name.clone())),
                None => (store.elements.iter().find(|e| e.qualified_name == declared_in).unwrap_or(owner), None),
            };
            let feature = FeatureCard { name: fname, owner: declared_in, props, doc_from };
            return Html(card(doc_of, Some(feature), doc_of).render().unwrap_or_default());
        }
    }

    let tmpl = ElementCardTemplate {
        found: false,
        reference: reference.clone(),
        name: reference.rsplit("::").next().unwrap_or(&reference).to_string(),
        element_type: String::new(),
        badge_class: String::new(),
        qualified_name: String::new(),
        elem_id: String::new(),
        status: String::new(),
        doc_html: String::new(),
        feature: None,
    };
    Html(tmpl.render().unwrap_or_default())
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

    let kind = element.frontmatter.diagram_kind.as_deref().unwrap_or(DEFAULT_DIAGRAM_KIND);
    match kind {
        "Mermaid" => {
            let src = extract_fenced_block(&element.doc, "mermaid").unwrap_or_default();
            Html(format!(
                r#"<div class="diagram-mermaid-wrapper"><pre class="mermaid">{}</pre></div>"#,
                html_escape(&src)
            ))
        }
        _ => {
            // SVG / BDD / IBD / StateMachine — not served over this HTML
            // route: every non-Mermaid `diagram_kind` is a Diagram IR
            // (`syscribe_model::vis`) that the sprotty editor fetches from
            // `/api/diagrams/model/{*qname}` and lays out in the browser
            // (`REQ-TRS-VIS-001`); there is no server-side SVG renderer any
            // more (`REQ-TRS-VIS-013`). A direct hit here (e.g. a stale
            // bookmark) gets a plain fallback rather than a panic.
            Html(
                r#"<p class="diagram-empty">This diagram type isn't served here — open it as a diagram tab.</p>"#
                    .to_string(),
            )
        }
    }
}


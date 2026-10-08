//! `GET /ui/element-card/{*qname}` (`REQ-TRS-VIS-026`): the read-only card the
//! diagram editor's side panel shows for a clicked shape or edge — identity
//! and the element's Markdown body rendered, a feature resolved to its owner
//! (or its type), and a reference that names nothing said so, never an error.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use syscribe_model::config::ValidateConfig;
use syscribe_model::walker::walk_model;
use syscribe_server::build_router;
use syscribe_server::state::new_state;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// A small software model: a `Controller` with a port feature and a part feature
/// typed by `Engine`, a documented `Engine` (table, code, a Mermaid block) with
/// a port, and a requirement with an id and a status.
fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-card-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
    write(
        &root,
        "Sys/Controller.md",
        "---\ntype: PartDef\nname: Controller\nfeatures:\n  - name: cmdIn\n    type: Port\n    typedBy: Sys::CmdPort\n    direction: in\n  - name: engine\n    typedBy: Sys::Engine\n    multiplicity: \"2\"\n  - name: limit\n    typedBy: ScalarValues::Real\n    unit: SI::kg\n---\n\nThe **controller** decides.\n",
    );
    write(
        &root,
        "Sys/Engine.md",
        "---\ntype: PartDef\nname: Engine\nfeatures:\n  - name: powerOut\n    type: Port\n    typedBy: Sys::CmdPort\n    direction: out\n---\n\n# Engine\n\nProduces thrust.\n\n| Mode | Limit |\n|---|---|\n| idle | 10 |\n\n```rust\nfn thrust() {}\n```\n\n```mermaid\nflowchart LR\n  A --> B\n```\n",
    );
    write(&root, "Sys/CmdPort.md", "---\ntype: PortDef\nname: CmdPort\n---\n\nCommand port definition.\n");
    write(
        &root,
        "Reqs/REQ-CARD-001.md",
        "---\ntype: Requirement\nid: REQ-CARD-001\nname: Thrust limit\nstatus: approved\nreqDomain: software\n---\n\nThe engine shall not exceed 10 N.\n",
    );
    root
}

async fn get(root: &Path, uri: &str) -> (StatusCode, String) {
    let elements = walk_model(root).unwrap();
    let config = ValidateConfig::with_model_root(root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, root.to_path_buf());
    let app = build_router(shared, reload_tx);
    let resp = app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap()).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

#[tokio::test]
async fn an_element_card_shows_identity_and_the_rendered_markdown_body() {
    let root = model();
    let (status, html) = get(&root, "/ui/element-card/Sys/Engine").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Engine"), "{html}");
    assert!(html.contains("PartDef"), "type badge: {html}");
    assert!(html.contains("Sys::Engine"), "qualified name: {html}");
    assert!(html.contains("<h1>Engine</h1>"), "heading rendered: {html}");
    assert!(html.contains("<table>") && html.contains("<td>idle</td>"), "table rendered: {html}");
    assert!(html.contains("<pre><code class=\"language-rust\">"), "code block rendered: {html}");
    assert!(html.contains("<pre class=\"mermaid\">"), "mermaid block left for the client to draw: {html}");
    assert!(html.contains("hx-get=\"/ui/detail/Sys::Engine\""), "a way to the full detail dialog: {html}");
    assert!(!html.contains("**"), "no raw markdown markers left");
}

#[tokio::test]
async fn a_requirement_card_shows_its_id_and_status() {
    let root = model();
    let (_, html) = get(&root, "/ui/element-card/Reqs::REQ-CARD-001").await;
    assert!(html.contains("REQ-CARD-001"), "{html}");
    assert!(html.contains("approved"), "{html}");
    assert!(html.contains("Thrust limit"), "{html}");
    assert!(html.contains("shall not exceed 10 N"), "{html}");
}

#[tokio::test]
async fn a_port_feature_resolves_to_its_owner_and_shows_its_declared_properties() {
    let root = model();
    // `cmdIn` is declared on Controller and typed by CmdPort: the card names the
    // feature and its owner, lists its properties, and documents it with its type.
    let (_, html) = get(&root, "/ui/element-card/Sys::Controller::cmdIn").await;
    assert!(html.contains("Feature <code>cmdIn</code> of <code>Sys::Controller</code>"), "{html}");
    assert!(html.contains("<th>direction</th><td>in</td>"), "{html}");
    assert!(html.contains("<th>typedBy</th><td>Sys::CmdPort</td>"), "{html}");
    assert!(html.contains("Command port definition."), "documentation of the port's type: {html}");
    assert!(html.contains("type <code>Sys::CmdPort</code>") || html.contains("its type <code>Sys::CmdPort</code>"), "says whose documentation it is: {html}");
}

#[tokio::test]
async fn a_feature_without_a_resolvable_type_shows_its_owners_body() {
    let root = model();
    let (_, html) = get(&root, "/ui/element-card/Sys::Controller::limit").await;
    assert!(html.contains("Feature <code>limit</code> of <code>Sys::Controller</code>"), "{html}");
    assert!(html.contains("<th>unit</th><td>SI::kg</td>"), "{html}");
    assert!(html.contains("<strong>controller</strong> decides"), "the owner's body: {html}");
    assert!(!html.contains("Documentation below is that of its type"), "{html}");
}

#[tokio::test]
async fn a_nested_feature_path_follows_typed_by_to_the_declaring_element() {
    let root = model();
    // `engine::powerOut`: `engine` is a part feature typed by Engine, which declares `powerOut`.
    let (_, html) = get(&root, "/ui/element-card/Sys::Controller::engine::powerOut").await;
    assert!(html.contains("Feature <code>powerOut</code> of <code>Sys::Engine</code>"), "{html}");
    assert!(html.contains("<th>direction</th><td>out</td>"), "{html}");
}

#[tokio::test]
async fn a_reference_that_names_nothing_says_so_and_is_escaped() {
    let root = model();
    let (status, html) = get(&root, "/ui/element-card/Nowhere::%3Cb%3Ex%3C/b%3E").await;
    assert_eq!(status, StatusCode::OK, "never an error page");
    assert!(html.contains("is not a model element"), "{html}");
    assert!(!html.contains("<b>"), "the reference text is escaped, not injected: {html}");
    assert!(html.contains("&lt;b&gt;x&lt;"), "the angle brackets are entity-escaped: {html}");
}

#[tokio::test]
async fn an_unknown_feature_of_a_known_element_is_not_a_model_element() {
    let root = model();
    let (_, html) = get(&root, "/ui/element-card/Sys::Controller::nope").await;
    assert!(html.contains("is not a model element"), "{html}");
}

#[tokio::test]
async fn raw_html_and_script_links_in_a_body_are_not_rendered_live() {
    let r = model();
    write(
        &r,
        "Sys/Hostile.md",
        "---\ntype: PartDef\nname: Hostile\n---\n<img src=x onerror=alert(1)>\n\n<script>alert(2)</script>\n\n[click](javascript:alert(3)) ![i](data:text/html;base64,AAAA) <b hx-get=\"/x\">z</b>\n",
    );
    let (status, html) = get(&r, "/ui/element-card/Sys/Hostile").await;
    assert_eq!(status, StatusCode::OK, "{html}");
    for live in ["<img src=x", "<script>", "href=\"javascript:", "src=\"data:", "<b hx-get"] {
        assert!(!html.contains(live), "{live} must not survive as live markup:\n{html}");
    }
    assert!(html.contains("&lt;script&gt;"), "raw HTML is shown as text:\n{html}");
}

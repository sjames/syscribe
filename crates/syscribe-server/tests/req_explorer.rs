//! TC-TRS-REQEXPL-001 / GH #269 phase 2.

use std::path::Path;
use std::process::Command;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use syscribe_model::config::ValidateConfig;
use syscribe_model::walker::walk_model;
use syscribe_server::build_router;
use syscribe_server::state::new_state;

fn app() -> axum::Router {
    let root = std::env::temp_dir().join(format!("syscribe-reqexpl-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    let elements = walk_model(&root).unwrap();
    let (shared, tx) = new_state(elements, String::new(), ValidateConfig::with_model_root(&root), root);
    build_router(shared, tx)
}

async fn get(app: &axum::Router, uri: &str) -> (StatusCode, String, String) {
    let resp = app.clone().oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap()).await.unwrap();
    let status = resp.status();
    let ct = resp.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let body = String::from_utf8_lossy(&resp.into_body().collect().await.unwrap().to_bytes()).into_owned();
    (status, ct, body)
}

#[tokio::test]
async fn the_page_is_served_and_every_page_links_to_it() {
    let a = app();
    let (s, _, html) = get(&a, "/requirements").await;
    assert_eq!(s, StatusCode::OK);
    assert!(html.contains("id=\"req-explorer\"") && html.contains("/static/js/requirements-explorer.js"), "{html}");
    assert!(html.contains("id=\"req-graph\"") && html.contains("id=\"req-detail\""), "{html}");
    for page in ["/heat", "/features", "/requirements"] {
        assert!(get(&a, page).await.2.contains("href=\"/requirements\""), "{page} must link to the explorer");
    }
}

#[tokio::test]
async fn seeding_is_escaped_and_depth_falls_back() {
    let a = app();
    let (_, _, html) = get(&a, "/requirements?focus=%3Cscript%3Ealert(1)%3C/script%3E&depth=2&config=%22%3E%3Cb%3E").await;
    assert!(!html.contains("<script>alert(1)"), "focus must be escaped: {html}");
    assert!(html.contains("data-focus=\"&lt;script&gt;") || html.contains("data-focus=\"&#60;script&#62;"), "{html}");
    assert!(html.contains("data-depth=\"2\""));
    assert!(!html.contains("\"><b>"), "config must be escaped");
    for bad in ["depth=abc", "depth=99", "depth=-1"] {
        let (_, _, h) = get(&a, &format!("/requirements?{bad}")).await;
        let d = if bad == "depth=99" { "6" } else { "1" };
        assert!(h.contains(&format!("data-depth=\"{d}\"")), "{bad}: {h}");
    }
}

#[tokio::test]
async fn the_client_script_is_served() {
    let a = app();
    let (s, ct, js) = get(&a, "/static/js/requirements-explorer.js").await;
    assert_eq!(s, StatusCode::OK);
    assert!(ct.contains("javascript"), "{ct}");
    assert!(js.contains("/api/req-graph") && js.contains("layoutGraph"), "{js}");
}

#[test]
fn the_layout_function_layers_nodes_by_hop_without_overlap() {
    if Command::new("node").arg("--version").output().map(|o| !o.status.success()).unwrap_or(true) {
        eprintln!("node not available — skipping the layout test");
        return;
    }
    let js = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/js/requirements-explorer.js");
    let script = format!(
        r#"const m = require({js:?});
const g = {{root:'A', nodes:[{{id:'A'}},{{id:'B'}},{{id:'C'}},{{id:'D'}},{{id:'E'}}],
  edges:[{{from:'B',to:'A',kind:'derivedFrom'}},{{from:'C',to:'A',kind:'derivedFrom'}},{{from:'D',to:'B',kind:'verifies'}},{{from:'E',to:'D',kind:'x'}}]}};
const l = m.layoutGraph(g);
const by = Object.fromEntries(l.nodes.map(n => [n.id, n]));
const out = {{ col: Object.fromEntries(l.nodes.map(n => [n.id, n.col])), overlap: false, w: l.width, h: l.height }};
for (let i = 0; i < l.nodes.length; i++) for (let j = i+1; j < l.nodes.length; j++) {{
  const a = l.nodes[i], b = l.nodes[j];
  if (a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h) out.overlap = true;
}}
console.log(JSON.stringify(out));"#
    );
    let o = Command::new("node").arg("-e").arg(&script).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["col"]["A"], 0);
    assert_eq!((v["col"]["B"].as_i64(), v["col"]["C"].as_i64()), (Some(1), Some(1)));
    assert_eq!((v["col"]["D"].as_i64(), v["col"]["E"].as_i64()), (Some(2), Some(3)));
    assert_eq!(v["overlap"], false, "{v}");
    assert!(v["w"].as_f64().unwrap() > 0.0 && v["h"].as_f64().unwrap() > 0.0);
}

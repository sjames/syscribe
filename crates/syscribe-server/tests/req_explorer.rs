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
    app_with(&[])
}

fn app_with(files: &[(&str, &str)]) -> axum::Router {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-reqexpl-{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (name, body) in files {
        std::fs::write(root.join(name), body).unwrap();
    }
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

fn req(id: &str, name: &str) -> (String, String) {
    (format!("{id}.md"), format!("---\ntype: Requirement\nid: {id}\nname: \"{name}\"\nstatus: draft\nreqDomain: software\nreqClass: system\n---\nbody\n"))
}

fn search_app() -> axum::Router {
    let owned = [req("REQ-SRCH-001", "Brake control"), req("REQ-SRCH-0010", "Other"), req("REQ-ZZ-001", "see req-srch-001 mention"), req("REQ-ZZ-002", "Unrelated")];
    let files: Vec<(&str, &str)> = owned.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    app_with(&files)
}

#[tokio::test]
async fn search_ranks_exact_id_prefix_then_name_and_truncates() {
    let a = search_app();
    let (s, _, body) = get(&a, "/api/req-graph/search?q=req-srch-001").await;
    assert_eq!(s, StatusCode::OK, "{body}");
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let ids: Vec<&str> = v["results"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["REQ-SRCH-001", "REQ-SRCH-0010", "REQ-ZZ-001"], "exact id, id prefix, then name match: {body}");
    assert_eq!(v["results"][0]["type"], "Requirement");
    assert_eq!(v["truncated"], false);
    let (_, _, body) = get(&a, "/api/req-graph/search?q=req-&limit=2").await;
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["results"].as_array().unwrap().len(), 2);
    assert_eq!(v["truncated"], true);
    let (_, _, body) = get(&a, "/api/req-graph/search?q=brake").await;
    assert!(body.contains("REQ-SRCH-001") && !body.contains("REQ-ZZ-002"), "{body}");
}

#[tokio::test]
async fn search_rejects_bad_input() {
    let a = search_app();
    for uri in ["/api/req-graph/search", "/api/req-graph/search?q=", "/api/req-graph/search?q=%20", "/api/req-graph/search?q=x&limit=abc", "/api/req-graph/search?q=x&config=CONF-NOPE"] {
        let (s, _, body) = get(&a, uri).await;
        assert_eq!(s, StatusCode::BAD_REQUEST, "{uri}: {body}");
    }
}

#[test]
fn the_filter_hides_non_matching_nodes_and_edges_but_never_the_root() {
    if Command::new("node").arg("--version").output().map(|o| !o.status.success()).unwrap_or(true) {
        eprintln!("node not available — skipping the filter test");
        return;
    }
    let js = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/js/requirements-explorer.js");
    let script = format!(
        r#"const m = require({js:?});
const g = {{root:'A', nodes:[
  {{id:'A',type:'Requirement',verification:'unverified',asil:'B'}},
  {{id:'B',type:'Requirement',verification:'verified',asil:'D'}},
  {{id:'C',type:'TestCase',verification:'na'}},
  {{id:'D',type:'Block',verification:'na'}}],
  edges:[{{from:'B',to:'A',kind:'derivedFrom'}},{{from:'C',to:'B',kind:'verifies'}},{{from:'D',to:'A',kind:'satisfies'}}]}};
const run = f => {{ const r = m.filterGraph(g, f); return {{n: r.nodes.map(n=>n.id), e: r.edges.length, hidden: r.hidden}}; }};
console.log(JSON.stringify({{
  none: run({{}}),
  type: run({{types:['Requirement']}}),
  ver: run({{verification:['verified']}}),
  asil: run({{asil:['D']}}),
  all: run({{types:['Block'], verification:['verified']}}),
}}));"#
    );
    let o = Command::new("node").arg("-e").arg(&script).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["none"]["hidden"], 0);
    assert_eq!(v["none"]["e"], 3);
    assert_eq!(v["type"]["n"], serde_json::json!(["A", "B"]));
    assert_eq!(v["type"]["e"], 1);
    assert_eq!(v["type"]["hidden"], 2);
    assert_eq!(v["ver"]["n"], serde_json::json!(["A", "B"]));
    assert_eq!(v["asil"]["n"], serde_json::json!(["A", "B"]));
    assert_eq!(v["all"]["n"], serde_json::json!(["A"]), "root stays even when nothing matches");
    assert_eq!(v["all"]["e"], 0);
}

#[tokio::test]
async fn the_page_has_search_and_filter_controls() {
    let a = app();
    let (_, _, html) = get(&a, "/requirements").await;
    assert!(html.contains("id=\"req-search\"") && html.contains("id=\"req-filters\""), "{html}");
}

#[tokio::test]
async fn safety_and_security_links_are_edges() {
    let a = "---\ntype: Requirement\nid: REQ-LNK-001\nname: A\nstatus: draft\nreqDomain: software\nreqClass: system\nhazardRef: [REQ-LNK-002]\nhazardousEvents: [REQ-LNK-002]\nthreatRef: REQ-LNK-002\nthreatScenarios: [REQ-LNK-002]\nmitigatedBy: [REQ-LNK-002]\nderivedFromCybersecurityGoal: [REQ-LNK-002]\nrelatedSafetyGoal: REQ-LNK-002\nconfirms: [REQ-LNK-002]\nimplementedBy: [REQ-LNK-002]\n---\n";
    let b = "---\ntype: Requirement\nid: REQ-LNK-002\nname: B\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n";
    let app = app_with(&[("a.md", a), ("b.md", b)]);
    let (s, _, body) = get(&app, "/api/req-graph?root=REQ-LNK-001&depth=1").await;
    assert_eq!(s, StatusCode::OK, "{body}");
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let mut kinds: Vec<&str> = v["edges"].as_array().unwrap().iter().map(|e| e["kind"].as_str().unwrap()).collect();
    kinds.sort();
    let mut want = vec!["confirms", "derivedFromCybersecurityGoal", "hazardRef", "hazardousEvents", "implementedBy", "mitigatedBy", "relatedSafetyGoal", "threatRef", "threatScenarios"];
    want.sort();
    assert_eq!(kinds, want, "{body}");
    let (s, _, _) = get(&app, "/api/req-graph?root=REQ-LNK-001&edges=mitigatedBy,hazardRef").await;
    assert_eq!(s, StatusCode::OK, "new kinds are filterable");
}

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
    assert!(body.contains("REQ-SRCH-001"), "{body}");
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
  {{id:'B',type:'Requirement',verification:'verified',asil:'B(d)'}},
  {{id:'C',type:'TestCase',verification:'na'}},
  {{id:'D',type:'Block',verification:'na'}}],
  edges:[{{from:'B',to:'A',kind:'derivedFrom'}},{{from:'C',to:'B',kind:'verifies'}},{{from:'D',to:'A',kind:'satisfies'}}]}};
const run = f => {{ const r = m.filterGraph(g, f); return {{n: r.nodes.map(n=>n.id), e: r.edges.length, hidden: r.hidden}}; }};
console.log(JSON.stringify({{
  none: run({{}}),
  type: run({{types:['Requirement']}}),
  ver: run({{verification:['verified']}}),
  asil: run({{asil:['D']}}),
  asilb: run({{asil:['B']}}),
  asild: run({{asil:['D']}}),
  orphan: (() => {{ const g2 = {{root:'A', nodes:[{{id:'A',type:'Requirement'}},{{id:'C',type:'TestCase'}},{{id:'D',type:'Requirement'}}], edges:[{{from:'A',to:'C',kind:'x'}},{{from:'C',to:'D',kind:'x'}}]}}; return m.filterGraph(g2, {{types:['Requirement']}}).nodes.map(n=>n.id); }})(),
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
    assert_eq!(v["asil"]["n"], serde_json::json!(["A"]));
    assert_eq!(v["asilb"]["n"], serde_json::json!(["A", "B"]), "B(d) counts as B");
    assert_eq!(v["asild"]["n"], serde_json::json!(["A"]), "decomposed B(d) is rated B, not D");
    assert_eq!(v["orphan"], serde_json::json!(["A"]), "nodes behind a hidden node are not left floating");
    assert_eq!(v["all"]["n"], serde_json::json!(["A"]), "root stays even when nothing matches");
    assert_eq!(v["all"]["e"], 0);
}

#[tokio::test]
async fn the_page_has_search_and_filter_controls() {
    let a = app();
    let (_, _, html) = get(&a, "/requirements").await;
    assert!(html.contains("id=\"req-search\"") && html.contains("id=\"req-filters\"") && html.contains("id=\"req-trace\""), "{html}");
}

#[tokio::test]
async fn safety_and_security_links_are_edges() {
    let a = "---\ntype: Requirement\nid: REQ-LNK-001\nname: A\nstatus: draft\nreqDomain: software\nreqClass: system\nhazardRef: [REQ-LNK-002]\nhazardousEvents: [REQ-LNK-002]\nthreatRef: REQ-LNK-002\nthreatScenarios: [REQ-LNK-002]\nmitigatedBy: [REQ-LNK-002]\nderivedFromCybersecurityGoal: [REQ-LNK-002]\nrelatedSafetyGoal: REQ-LNK-002\nconfirms: [REQ-LNK-002]\nimplementedBy: [REQ-LNK-002]\ndamageScenarios: [REQ-LNK-002]\nassets: [REQ-LNK-002]\nimplementsGoals: [REQ-LNK-002]\naffectedElements: [REQ-LNK-002]\nsupports: [REQ-LNK-002]\ntopEvent: REQ-LNK-002\nftaRef: REQ-LNK-002\nassetOwner: REQ-LNK-002\n---\n";
    let b = "---\ntype: Requirement\nid: REQ-LNK-002\nname: B\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n";
    let app = app_with(&[("a.md", a), ("b.md", b)]);
    let (s, _, body) = get(&app, "/api/req-graph?root=REQ-LNK-001&depth=1").await;
    assert_eq!(s, StatusCode::OK, "{body}");
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let mut kinds: Vec<&str> = v["edges"].as_array().unwrap().iter().map(|e| e["kind"].as_str().unwrap()).collect();
    kinds.sort();
    let mut want = vec!["affectedElements", "assetOwner", "assets", "damageScenarios", "ftaRef", "implementsGoals", "supports", "topEvent", "confirms", "derivedFromCybersecurityGoal", "hazardRef", "hazardousEvents", "implementedBy", "mitigatedBy", "relatedSafetyGoal", "threatRef", "threatScenarios"];
    want.sort();
    assert_eq!(kinds, want, "{body}");
    let (s, _, _) = get(&app, "/api/req-graph?root=REQ-LNK-001&edges=mitigatedBy,hazardRef").await;
    assert_eq!(s, StatusCode::OK, "new kinds are filterable");
}

#[test]
fn trace_returns_shortest_paths_to_a_category_and_jump_links_are_fixed() {
    if Command::new("node").arg("--version").output().map(|o| !o.status.success()).unwrap_or(true) {
        panic!("node is required for the explorer client tests");
    }
    let js = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/js/requirements-explorer.js");
    let script = format!(
        r#"const m = require({js:?});
// A -> B -> T (short, edge directions mixed); A -> C -> D -> T (long); X is unrelated.
const g = {{root:'A', nodes:[{{id:'A',type:'Requirement'}},{{id:'B',type:'Requirement'}},{{id:'C',type:'Requirement'}},{{id:'D',type:'Requirement'}},
  {{id:'T',type:'TestCase'}},{{id:'X',type:'Requirement'}}],
  edges:[{{from:'B',to:'A',kind:'derivedFrom'}},{{from:'B',to:'T',kind:'x'}},{{from:'C',to:'A',kind:'x'}},{{from:'D',to:'C',kind:'x'}},{{from:'D',to:'T',kind:'x'}}]}};
const t = m.tracePath(g, 'A', 'tests');
const none = m.tracePath(g, 'A', 'security');
const fromT = m.tracePath(g, 'T', 'tests');
console.log(JSON.stringify({{
  nodes: t.nodes.slice().sort(), edges: t.edges.length, targets: t.targets,
  none: [none.nodes.length, none.edges.length, none.targets.length],
  fromT: fromT.targets,
  unknown: m.tracePath(g, 'A', 'bogus').nodes.length,
  cat: [m.categoryOf('FaultTreeGate'), m.categoryOf('Zone'), m.categoryOf('Configuration'), m.categoryOf('Requirement') || null],
  jumps: ['FeatureDef','Configuration','FeatureModel','PlanningItem','Requirement'].map(ty => m.jumpLinks({{type: ty, id: 'a"b'}}).map(l => l.href)),
}}));"#
    );
    let o = Command::new("node").arg("-e").arg(&script).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["nodes"], serde_json::json!(["A", "B", "T"]), "{v}");
    assert_eq!(v["edges"], 2);
    assert_eq!(v["targets"], serde_json::json!(["T"]));
    assert_eq!(v["none"], serde_json::json!([0, 0, 0]));
    assert_eq!(v["fromT"], serde_json::json!([]), "the start is never a target");
    assert_eq!(v["unknown"], 0);
    assert_eq!(v["cat"], serde_json::json!(["safety", "security", "features", null]));
    assert_eq!(v["jumps"], serde_json::json!([["/features"], ["/features"], ["/features"], ["/planning"], []]));
}

#[test]
fn trace_handles_parallel_edges_multiple_targets_and_known_type_names() {
    if Command::new("node").arg("--version").output().map(|o| !o.status.success()).unwrap_or(true) {
        panic!("node is required for the explorer client tests");
    }
    let js = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/js/requirements-explorer.js");
    let script = format!(
        r#"const m = require({js:?});
// Two parallel A-T1 edges, a second target T2 behind B, and a filter that shifts edge indices.
const g = {{root:'A', nodes:[{{id:'A',type:'Requirement'}},{{id:'B',type:'Requirement'}},{{id:'T1',type:'TestCase'}},{{id:'T2',type:'TestCase'}},{{id:'H',type:'Block'}}],
  edges:[{{from:'A',to:'H',kind:'x'}},{{from:'T1',to:'A',kind:'verifies'}},{{from:'T1',to:'A',kind:'covers'}},{{from:'B',to:'A',kind:'x'}},{{from:'T2',to:'B',kind:'x'}}]}};
const t = m.tracePath(g, 'A', 'tests');
const f = m.filterGraph(g, {{types:['Requirement','TestCase']}});
const tf = m.tracePath(f, 'A', 'tests');
console.log(JSON.stringify({{ t, tf, tfEdgeKinds: tf.edges.map(i => f.edges[i].kind), names: [].concat(...Object.values(m.CATEGORIES)) }}));"#
    );
    let o = Command::new("node").arg("-e").arg(&script).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["t"]["targets"], serde_json::json!(["T1", "T2"]));
    assert_eq!(v["t"]["edges"], serde_json::json!([1, 2, 3, 4]), "both parallel edges and the route to T2: {v}");
    assert_eq!(v["t"]["nodes"], serde_json::json!(["A", "B", "T1", "T2"]));
    assert_eq!(v["tf"]["edges"].as_array().unwrap().len(), 4, "indices are relative to the filtered edge list");
    assert_eq!(v["tfEdgeKinds"].as_array().unwrap().len(), 4);
    let known: std::collections::HashSet<&str> = syscribe_model::element::ElementType::ALL.iter().map(|t| t.name()).collect();
    for n in v["names"].as_array().unwrap() {
        assert!(known.contains(n.as_str().unwrap()), "{n} is not an ElementType name");
    }
}

#[test]
fn table_rows_csv_and_file_names_are_pure_and_safe() {
    if Command::new("node").arg("--version").output().map(|o| !o.status.success()).unwrap_or(true) {
        panic!("node is required for the explorer client tests");
    }
    let js = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/js/requirements-explorer.js");
    let script = format!(
        r#"const m = require({js:?});
const g = {{root:'A', nodes:[{{id:'A',type:'Requirement',name:'Root, "quoted"',status:'draft',asil:'B',verification:'verified'}},
  {{id:'B',type:'TestCase',name:'=HYPERLINK("x")',status:'active',verification:'na'}},
  {{id:'C',type:'Requirement',name:'line1\nline2'}}],
  edges:[{{from:'B',to:'A',kind:'verifies'}},{{from:'C',to:'A',kind:'derivedFrom'}}]}};
const t = m.tableRows(g);
console.log(JSON.stringify({{
  hops: t.elements.map(r => [r.id, r.hop]),
  rel: t.relations.map(r => [r.from, r.to, r.kind]),
  csv: m.toCsv(g),
  names: [m.exportName('A', 'csv'), m.exportName('../a b/c:d', 'json'), m.exportName('', 'svg')],
  cell: [m.csvCell('+1'), m.csvCell('-1'), m.csvCell('@x'), m.csvCell('\tx'), m.csvCell('plain'), m.csvCell(undefined), m.csvCell('a,b')],
}}));"#
    );
    let o = Command::new("node").arg("-e").arg(&script).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["hops"], serde_json::json!([["A", 0], ["B", 1], ["C", 1]]));
    assert_eq!(v["rel"], serde_json::json!([["B", "A", "verifies"], ["C", "A", "derivedFrom"]]));
    let csv = v["csv"].as_str().unwrap();
    assert!(csv.contains("\"Root, \"\"quoted\"\"\""), "{csv}");
    assert!(csv.contains("\"'=HYPERLINK(\"\"x\"\")\""), "formula neutralised and quoted: {csv}");
    assert!(csv.contains("\"line1\nline2\""), "{csv}");
    assert!(csv.starts_with("record,id,type,name,status,asil,verification,hop,from,to,kind\r\n"), "{csv}");
    assert!(csv.contains("relation,,,,,,,,B,A,verifies\r\n"), "relations share the one column set: {csv}");
    assert_eq!(v["names"], serde_json::json!(["req-graph-A.csv", "req-graph-.._a_b_c_d.json", "req-graph-graph.svg"]));
    assert_eq!(v["cell"], serde_json::json!(["'+1", "'-1", "'@x", "'\tx", "plain", "", "\"a,b\""]));
}

#[tokio::test]
async fn the_page_has_view_toggle_and_export_buttons() {
    let a = app();
    let (_, _, html) = get(&a, "/requirements").await;
    for id in ["req-view-table", "req-export-json", "req-export-csv", "req-export-svg", "req-table"] {
        assert!(html.contains(&format!("id=\"{id}\"")), "{id} missing: {html}");
    }
}

#[test]
fn the_matrix_model_lists_rows_columns_and_direct_edge_kinds() {
    if Command::new("node").arg("--version").output().map(|o| !o.status.success()).unwrap_or(true) {
        panic!("node is required for the explorer client tests");
    }
    let js = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/js/requirements-explorer.js");
    let script = format!(
        r#"const m = require({js:?});
const g = {{root:'R1', nodes:[{{id:'R1',type:'Requirement'}},{{id:'R2',type:'Requirement'}},{{id:'R3',type:'Requirement'}},
  {{id:'T1',type:'TestCase'}},{{id:'T2',type:'TestCase'}},{{id:'P1',type:'PartDef'}}],
  edges:[{{from:'T1',to:'R1',kind:'verifies'}},{{from:'R1',to:'T1',kind:'evidence'}},{{from:'R2',to:'P1',kind:'allocatedTo'}},{{from:'R2',to:'R1',kind:'derivedFrom'}},{{from:'R3',to:'T2'}}]}};
const t = m.matrixModel(g, 'tests');
const a = m.matrixModel(g, 'architecture');
console.log(JSON.stringify({{ rows: t.rows, cols: t.cols, cell: t.cells['R1|T1'], none: t.rows.filter(r => t.noneIn[r]), noCell: t.cells['R2|T1'] || null, reqReq: Object.keys(t.cells).filter(k => k.startsWith('R2|R1')), unnamed: t.cells['R3|T2'],
  archCols: a.cols, archCell: a.cells['R2|P1'], bad: m.matrixModel(g, 'bogus').cols }}));"#
    );
    let o = Command::new("node").arg("-e").arg(&script).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["rows"], serde_json::json!(["R1", "R2", "R3"]));
    assert_eq!(v["cols"], serde_json::json!(["T1", "T2"]), "unlinked columns are kept");
    assert_eq!(v["cell"], serde_json::json!(["evidence", "verifies"]), "both directions, sorted, deduplicated");
    assert_eq!(v["none"], serde_json::json!(["R2"]), "R3 now links to T2");
    assert_eq!(v["reqReq"], serde_json::json!([]), "requirement-to-requirement edges are not cells");
    assert_eq!(v["unnamed"], serde_json::json!(["(unnamed)"]));
    assert_eq!(v["noCell"], serde_json::Value::Null);
    assert_eq!(v["archCols"], serde_json::json!(["P1"]));
    assert_eq!(v["archCell"], serde_json::json!(["allocatedTo"]));
    assert_eq!(v["bad"], serde_json::json!([]));
}

#[tokio::test]
async fn the_page_has_matrix_controls() {
    let a = app();
    let (_, _, html) = get(&a, "/requirements").await;
    for id in ["req-view-matrix", "req-matrix-cols", "req-matrix"] {
        assert!(html.contains(&format!("id=\"{id}\"")), "{id} missing: {html}");
    }
}

#[test]
fn vmodel_columns_classify_nodes_and_the_layout_honours_them() {
    if Command::new("node").arg("--version").output().map(|o| !o.status.success()).unwrap_or(true) {
        panic!("node is required for the explorer client tests");
    }
    let js = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/js/requirements-explorer.js");
    let script = format!(
        r#"const m = require({js:?});
const nodes = [{{id:'S',type:'Requirement',reqClass:'stakeholder'}},{{id:'Y',type:'Requirement',reqClass:'system'}},{{id:'W',type:'Requirement',reqClass:'software'}},
  {{id:'N',type:'Requirement'}},{{id:'P',type:'PartDef'}},{{id:'T',type:'TestCase'}},{{id:'H',type:'HazardousEvent'}},{{id:'X'}}];
const cols = Object.fromEntries(nodes.map(n => [n.id, m.vmodelColumn(n)]));
// no stakeholder, no tests: columns 0 and 4 are empty and must not be drawn
const g = {{root:'Y', nodes:[nodes[1],nodes[2],nodes[4],nodes[6]], edges:[{{from:'W',to:'Y',kind:'x'}},{{from:'Y',to:'P',kind:'x'}}]}};
const l = m.layoutGraph(g, m.vmodelColumn, ['Stakeholder requirements','System requirements','Other requirements','Architecture','Tests','Other elements']);
const many = {{root:'S', nodes:[nodes[0],nodes[1],nodes[2],nodes[3],nodes[4],nodes[5],nodes[6],nodes[7],{{id:'T2',type:'TestCase'}},{{id:'T3',type:'TestCase'}}], edges:[{{from:'T',to:'S',kind:'verifies'}},{{from:'N',to:'W',kind:'x'}}]}};
const lm = m.layoutGraph(many, m.vmodelColumn, ['Stakeholder requirements']);
const odd = m.layoutGraph({{root:'a', nodes:[{{id:'a'}},{{id:'b'}}], edges:[]}}, n => n.id === 'a' ? undefined : NaN);
let overlap = false;
for (let i = 0; i < l.nodes.length; i++) for (let j = i+1; j < l.nodes.length; j++) {{
  const a = l.nodes[i], b = l.nodes[j];
  if (a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h) overlap = true;
}}
const used = Object.fromEntries(l.nodes.map(n => [n.id, n.col]));
console.log(JSON.stringify({{ cols, used, headers: (l.headers || []).map(h => [h.col, h.label]), overlap, manyCols: [...new Set(lm.nodes.map(n => n.col))].sort(), manyHeaders: lm.headers.map(h => h.label), manyOverlap: (() => {{ for (let i = 0; i < lm.nodes.length; i++) for (let j = i+1; j < lm.nodes.length; j++) {{ const a = lm.nodes[i], b = lm.nodes[j]; if (a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h) return true; }} return false; }})(), odd: odd.nodes.map(n => n.col), oddHeaders: odd.headers, hop: m.layoutGraph(g).headers || null }}));"#
    );
    let o = Command::new("node").arg("-e").arg(&script).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["cols"], serde_json::json!({"S": 0, "Y": 1, "W": 2, "N": 2, "P": 3, "T": 4, "H": 5, "X": 5}));
    // occupied: Y(1)->0, W(2)->1, P(3)->2, H(5)->3
    assert_eq!(v["used"], serde_json::json!({"Y": 0, "W": 1, "P": 2, "H": 3}), "{v}");
    assert_eq!(v["headers"], serde_json::json!([[0, "System requirements"], [1, "Other requirements"], [2, "Architecture"], [3, "Other elements"]]));
    assert_eq!(v["overlap"], false);
    assert_eq!(v["manyCols"], serde_json::json!([0, 1, 2, 3, 4, 5]), "all six columns occupied");
    assert_eq!(v["manyHeaders"][0], "Stakeholder requirements");
    assert_eq!(v["manyOverlap"], false, "tall columns do not overlap");
    assert_eq!(v["odd"], serde_json::json!([0, 0]), "non-finite columns count as 0");
    assert_eq!(v["oddHeaders"], serde_json::json!([{"col": 0, "label": ""}]), "no labels given, blank header");
    assert!(v["hop"].is_null(), "the default layout has no headers");
}

#[tokio::test]
async fn the_page_has_the_layout_selector() {
    let a = app();
    let (_, _, html) = get(&a, "/requirements").await;
    assert!(html.contains("id=\"req-layout\"") && html.contains("v-model"), "{html}");
}

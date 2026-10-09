//! GH #223: `GET /heat` serves the FMEA / HARA / TARA heat tables.

use std::path::Path;

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

#[tokio::test]
async fn heat_page_renders_all_three_grids() {
    let root = std::env::temp_dir().join(format!("syscribe-heat-page-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "HE-HP-001.md",
        "---\nid: HE-HP-001\ntype: HazardousEvent\nname: E\nstatus: draft\nseverity: S3\nexposure: E4\ncontrollability: C3\n---\nb\n",
    );
    let elements = walk_model(&root).unwrap();
    let config = ValidateConfig::with_model_root(&root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, root.clone());
    let app = build_router(shared, reload_tx);
    let resp = app.oneshot(Request::builder().uri("/heat").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let html = String::from_utf8_lossy(&resp.into_body().collect().await.unwrap().to_bytes()).into_owned();
    assert!(html.contains("HARA ASIL matrix") && html.contains("FMEA severity x occurrence") && html.contains("TARA risk matrix"), "{html}");
    assert!(html.contains("HE-HP-001"), "{html}");
    assert!(html.contains("href=\"/heat\""), "header link");
}

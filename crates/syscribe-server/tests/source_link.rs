//! REQ-TRS-LINK-005 — the live web UI detail panel renders a per-element
//! "view source" icon linking to the element's hosted URL, in a new tab,
//! only when `[links]` is configured and the element is file-backed.
//!
//! Driven in-process against the real router via `tower::ServiceExt::oneshot`,
//! built exactly as `main` builds it (`build_router` + `new_state`).

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use syscribe_model::config::ValidateConfig;
use syscribe_model::walker::walk_model;
use syscribe_server::build_router;
use syscribe_server::state::new_state;

/// Absolute path to a fixture model under `qual/fixtures/TC-TRS-LINK-005`.
fn fixture(rel: &str) -> PathBuf {
    // CARGO_MANIFEST_DIR = <repo>/crates/syscribe-server
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../qual/fixtures/TC-TRS-LINK-005")
        .join(rel)
}

/// Build the router over a fixture model and GET `/ui/detail/<qname>`,
/// returning the rendered HTML body.
async fn detail_html(model_rel: &str, qname: &str) -> String {
    let model_root = fixture(model_rel);
    let elements = walk_model(&model_root).expect("walk fixture model");
    let config = ValidateConfig::with_model_root(&model_root);
    let (shared, reload_tx) = new_state(elements, String::new(), config);
    let app = build_router(shared, reload_tx);

    let uri = format!("/ui/detail/{}", qname);
    let resp = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .expect("router response");
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).expect("utf8 body")
}

/// With `[links]` configured, the detail panel shows a source-link `<a>` that
/// opens the element's hosted URL in a new tab.
#[tokio::test]
async fn link_005_detail_shows_source_link_when_links_configured() {
    let html = detail_html("linked", "UAV::Avionics::FlightController").await;

    // The external link affordance: a new-tab anchor to the hosted URL.
    assert!(
        html.contains("detail-source-link"),
        "expected a source-link icon in the detail panel, got:\n{html}"
    );
    assert!(
        html.contains("target=\"_blank\""),
        "source link must open in a new tab"
    );
    assert!(
        html.contains("rel=\"noopener\""),
        "source link must carry rel=noopener"
    );
    assert!(
        html.contains(
            "href=\"https://github.com/acme/uav/blob/main/model/UAV/Avionics/FlightController.md\""
        ),
        "source link href must be the resolved hosted URL, got:\n{html}"
    );
}

/// With no `[links]` table, no element renders a source-link icon (the feature
/// is inert and the detail panel is exactly as before).
#[tokio::test]
async fn link_005_no_source_link_when_links_absent() {
    let html = detail_html("none", "UAV::Avionics::FlightController").await;

    assert!(
        !html.contains("detail-source-link"),
        "no source-link icon expected without [links], got:\n{html}"
    );
    assert!(
        !html.contains("target=\"_blank\""),
        "no new-tab link expected without [links]"
    );
}

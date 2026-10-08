//! Planning dashboard (`REQ-TRS-VIS-027`): `GET /planning`, the board fragment
//! `GET /ui/planning/board`, grouping by who is working, status columns,
//! filters, escaping and live refresh when the model changes.

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

fn pi(id: &str, name: &str, extra: &str) -> String {
    format!("---\ntype: PlanningItem\nid: {id}\nname: \"{name}\"\nachieves: [REQ-DASH-001]\n{extra}---\n")
}

fn model(users: bool) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-dash-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&root).unwrap();
    if users {
        write(&root, ".syscribe.toml", "[users]\nalice = \"Alice Archer\"\n");
    }
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Reqs/REQ-DASH-001.md", "---\ntype: Requirement\nid: REQ-DASH-001\nname: R\nstatus: draft\nreqDomain: software\n---\nR.\n");
    write(&root, "Plan/PI-DASH-001.md", &pi("PI-DASH-001", "Todo thing", "status: todo\nitemType: feature\n"));
    write(&root, "Plan/PI-DASH-002.md", &pi("PI-DASH-002", "Agent <b>work</b>", "status: in_progress\nclaimedBy: agent-7\nclaimedAt: \"2026-10-08T10:00:00Z\"\n"));
    write(&root, "Plan/PI-DASH-003.md", &pi("PI-DASH-003", "Alice work", "status: in_progress\nassignedTo: alice\n"));
    write(&root, "Plan/PI-DASH-004.md", &pi("PI-DASH-004", "Stuck", "status: blocked\nblockedBy: [PI-DASH-001]\n"));
    write(&root, "Plan/PI-DASH-005.md", &pi("PI-DASH-005", "Finished", "status: done\n"));
    write(&root, "Plan/PI-DASH-006.md", &pi("PI-DASH-006", "Nobody yet", "status: in_progress\n"));
    root
}

async fn get_at(root: &Path, uri: &str) -> (StatusCode, String) {
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
async fn the_page_is_served_and_wires_live_refresh() {
    let root = model(false);
    let (status, html) = get_at(&root, "/planning").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("id=\"planning-board\"") && html.contains("/ui/planning/board"), "{html}");
    assert!(html.contains("/static/js/planning.js"), "{html}");
    assert!(html.contains("/static/js/live-reload.js"), "reload events come from the shared client");
    assert!(html.contains("href=\"/planning\""), "header link");
}

#[tokio::test]
async fn the_board_counts_every_status_and_groups_who_is_working() {
    let root = model(true);
    let (_, html) = get_at(&root, "/ui/planning/board").await;
    assert!(html.contains("6 items"), "{html}");
    assert!(html.contains("in_progress <b>3</b>"), "{html}");
    assert!(html.contains("blocked <b>1</b>") && html.contains("done <b>1</b>") && html.contains("todo <b>1</b>"), "{html}");
    // Working now: the agent, the person by display name, and the unassigned.
    assert!(html.contains("data-who=\"agent-7\""), "{html}");
    assert!(html.contains("data-who=\"Alice Archer\""), "roster display name: {html}");
    assert!(html.contains("data-who=\"Unassigned\""), "{html}");
    assert!(html.contains("data-since=\"2026-10-08T10:00:00Z\""), "claim age source: {html}");
    // Blocked card names what it waits on; done is hidden but counted.
    assert!(html.contains("blocked by PI-DASH-001"), "{html}");
    assert!(html.contains("1 hidden"), "{html}");
    assert!(!html.contains("Finished"), "done cards hidden by default: {html}");
    // Cards open the detail dialog.
    assert!(html.contains("hx-get=\"/ui/detail/Plan::PI-DASH-001\"") || html.contains("hx-get=\"/ui/detail/Plan/PI-DASH-001\""), "{html}");
}

#[tokio::test]
async fn done_cards_appear_on_request() {
    let root = model(false);
    let (_, html) = get_at(&root, "/ui/planning/board?done=1").await;
    assert!(html.contains("Finished"), "{html}");
    assert!(!html.contains("hidden"), "{html}");
}

#[tokio::test]
async fn filtering_by_who_keeps_only_that_persons_or_agents_items() {
    let root = model(true);
    let (_, html) = get_at(&root, "/ui/planning/board?who=agent-7").await;
    assert!(html.contains("1 item<"), "{html}");
    assert!(!html.contains("Todo thing") && !html.contains("Alice work"), "{html}");
    let (_, html) = get_at(&root, "/ui/planning/board?who=Alice%20Archer").await;
    assert!(html.contains("Alice work") && !html.contains("Stuck"), "{html}");
}

#[tokio::test]
async fn user_controlled_text_is_escaped() {
    let root = model(false);
    let (_, html) = get_at(&root, "/ui/planning/board").await;
    assert!(!html.contains("<b>work</b>"), "{html}");
    assert!(html.contains("Agent &lt;b&gt;work&lt;/b&gt;"), "{html}");
}

#[tokio::test]
async fn a_model_without_planning_items_shows_an_empty_state() {
    let root = std::env::temp_dir().join(format!("syscribe-dash-empty-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    let (status, html) = get_at(&root, "/ui/planning/board").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("No planning items"), "{html}");
}

#[tokio::test]
async fn a_changed_claim_shows_on_the_next_board_fetch() {
    // Live refresh re-fetches the fragment: the server reloads the model on a
    // write, so the next fetch reflects it.
    let root = model(false);
    let (_, before) = get_at(&root, "/ui/planning/board?who=agent-9").await;
    assert!(before.contains("No planning items"), "{before}");
    write(&root, "Plan/PI-DASH-001.md", &pi("PI-DASH-001", "Todo thing", "status: in_progress\nclaimedBy: agent-9\nclaimedAt: \"2026-10-08T11:00:00Z\"\n"));
    let (_, after) = get_at(&root, "/ui/planning/board?who=agent-9").await;
    assert!(after.contains("Todo thing") && after.contains("agent-9"), "{after}");
}

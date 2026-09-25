//! Integration tests for the MCP server's file-watch auto-reload (GH #181).
//! Realises TC-TRS-MCP-049 (REQ-TRS-MCP-048).
//!
//! Every positive check polls with a generous deadline instead of relying on a
//! fixed sleep, so a slow CI machine only makes the tests slower, never red.
//! The two negative checks (`--no-watch`, self-write) necessarily wait for a
//! bounded period in which nothing must happen; that wait is several times the
//! server's debounce window.

mod common;
use common::*;
use serde_json::{json, Value};
use std::path::Path;
use std::time::{Duration, Instant};

/// Upper bound for a watch-triggered reload to become visible.
const DEADLINE: Duration = Duration::from_secs(20);
/// How long a negative check waits for something that must not happen.
const QUIET: Duration = Duration::from_millis(2500);

const REQ_TEMPLATE: &str = "---\ntype: Requirement\nid: REQ-FX-001\nname: \"NAME\"\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nThe fixture system shall expose a stable element for the MCP read tools to retrieve and trace.\n";

fn req_with_name(name: &str) -> String {
    REQ_TEMPLATE.replace("NAME", name)
}

/// Replace `path`'s content atomically (write a sibling temp file, rename over),
/// the way most editors save — so the server never observes a torn write.
fn replace_file(path: &Path, content: &str) {
    let tmp = path.with_extension("md.tmp");
    std::fs::write(&tmp, content).unwrap();
    std::fs::rename(&tmp, path).unwrap();
}

fn name_of(mcp: &mut Mcp, r: &str) -> String {
    let v = mcp.call_tool("get_element", json!({"ref": r}));
    v.get("name").and_then(|n| n.as_str()).unwrap_or_else(|| panic!("no name in {v}")).to_string()
}

/// Poll `get_element` until the element's name equals `want`, or the deadline passes.
fn wait_for_name(mcp: &mut Mcp, r: &str, want: &str) -> bool {
    let start = Instant::now();
    while start.elapsed() < DEADLINE {
        if name_of(mcp, r) == want {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

/// Logging notifications whose `data.event` is `event` and `data.source` is `watch`.
fn watch_events<'a>(mcp: &'a Mcp, event: &str) -> Vec<&'a Value> {
    mcp.notifications
        .iter()
        .filter(|n| n.get("method").and_then(|m| m.as_str()) == Some("notifications/message"))
        .filter_map(|n| n.get("params").and_then(|p| p.get("data")))
        .filter(|d| {
            d.get("event").and_then(|e| e.as_str()) == Some(event)
                && d.get("source").and_then(|s| s.as_str()) == Some("watch")
        })
        .collect()
}

// ---- external edit is picked up without `reload` ----------------------------

#[test]
fn external_edit_is_picked_up_automatically() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    assert_eq!(name_of(&mut mcp, "REQ-FX-001"), "Fixture requirement for MCP tests");

    replace_file(&model.join("Requirements/REQ-FX-001.md"), &req_with_name("Edited outside the server"));
    assert!(
        wait_for_name(&mut mcp, "REQ-FX-001", "Edited outside the server"),
        "an external edit becomes visible without calling reload"
    );
    // One more request so any notification sent with the reload has been read.
    let _ = name_of(&mut mcp, "REQ-FX-001");
    let reloads = watch_events(&mcp, "reload");
    assert!(!reloads.is_empty(), "a watch reload logging message was sent");
    assert!(reloads[0].get("count").and_then(|c| c.as_u64()).is_some(), "it carries the element count");
    assert!(
        mcp.saw_notification("notifications/resources/list_changed"),
        "a resources/list_changed notification follows the auto-reload"
    );
}

#[test]
fn external_addition_is_picked_up_automatically() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let path = model.join("Requirements/REQ-FX-900.md");
    replace_file(&path, &req_with_name("Added outside").replace("REQ-FX-001", "REQ-FX-900"));
    let start = Instant::now();
    let mut found = false;
    while start.elapsed() < DEADLINE {
        let v = mcp.call_tool_raw("get_element", json!({"ref": "REQ-FX-900"}));
        if v.get("isError").and_then(|e| e.as_bool()) != Some(true) {
            found = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(found, "a file created outside the server becomes resolvable");
}

#[test]
fn read_only_server_still_watches() {
    let model = fixture_copy();
    let mut mcp = Mcp::start_with_args(&model, &["--read-only"]);
    mcp.initialize();
    replace_file(&model.join("Requirements/REQ-FX-001.md"), &req_with_name("Seen in read-only"));
    assert!(wait_for_name(&mut mcp, "REQ-FX-001", "Seen in read-only"), "--read-only still auto-reloads");
}

// ---- --no-watch disables it --------------------------------------------------

#[test]
fn no_watch_keeps_the_loaded_model_until_reload() {
    let model = fixture_copy();
    let mut mcp = Mcp::start_with_args(&model, &["--no-watch"]);
    mcp.initialize();
    replace_file(&model.join("Requirements/REQ-FX-001.md"), &req_with_name("Not watched"));
    let start = Instant::now();
    while start.elapsed() < QUIET {
        assert_eq!(name_of(&mut mcp, "REQ-FX-001"), "Fixture requirement for MCP tests", "--no-watch does not reload");
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(watch_events(&mcp, "reload").is_empty(), "no watch reload under --no-watch");
    // The explicit reload tool still works (and proves the edit itself was valid).
    mcp.call_tool("reload", json!({}));
    assert_eq!(name_of(&mut mcp, "REQ-FX-001"), "Not watched");
}

// ---- the server's own writes are not reloaded a second time ------------------

#[test]
fn self_write_does_not_trigger_a_second_reload() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "update_element",
        json!({"ref": "REQ-FX-001", "fields": {"name": "Written by the server"}, "dry_run": false}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "commit succeeded: {res}");
    // Give the watcher ample time to see (and dismiss) the write's own events.
    let start = Instant::now();
    while start.elapsed() < QUIET {
        assert_eq!(name_of(&mut mcp, "REQ-FX-001"), "Written by the server");
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        watch_events(&mcp, "reload").is_empty(),
        "the server's own committed write caused no watch reload; saw {:?}",
        watch_events(&mcp, "reload")
    );

    // A following external edit is still picked up — and reloads exactly once.
    let p3 = model.join("Requirements/REQ-FX-003.md");
    let orig = std::fs::read_to_string(&p3).unwrap();
    replace_file(&p3, &orig.replacen("Unverified fixture requirement", "Edited after a self-write", 1));
    let seen = wait_for_name(&mut mcp, "REQ-FX-003", "Edited after a self-write");
    assert!(seen, "the external edit after a self-write is picked up");
    let _ = name_of(&mut mcp, "REQ-FX-001");
    assert_eq!(watch_events(&mcp, "reload").len(), 1, "exactly one watch reload, for the external edit");
    assert_eq!(name_of(&mut mcp, "REQ-FX-001"), "Written by the server", "the self-write survives");
}

// ---- a half-saved file keeps the current store, then recovers ----------------

#[test]
fn broken_file_keeps_old_store_then_recovers() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let path = model.join("Requirements/REQ-FX-001.md");
    // A torn save: the frontmatter's YAML does not parse.
    replace_file(&path, "---\ntype: Requirement\nid: REQ-FX-001\nname: [\"Half saved\n---\n");
    let start = Instant::now();
    while start.elapsed() < DEADLINE && watch_events(&mcp, "reload_deferred").is_empty() {
        assert_eq!(
            name_of(&mut mcp, "REQ-FX-001"),
            "Fixture requirement for MCP tests",
            "the broken file never replaces the loaded store"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(!watch_events(&mcp, "reload_deferred").is_empty(), "the deferred reload is logged");
    assert_eq!(name_of(&mut mcp, "REQ-FX-001"), "Fixture requirement for MCP tests", "old store kept");
    assert!(watch_events(&mcp, "reload").is_empty(), "no reload of the broken state");

    replace_file(&path, &req_with_name("Fixed after a torn save"));
    assert!(wait_for_name(&mut mcp, "REQ-FX-001", "Fixed after a torn save"), "recovers once the file is fixed");
}

// ---- the watcher does not keep the process alive -----------------------------

#[test]
fn server_exits_when_stdin_closes() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let _ = name_of(&mut mcp, "REQ-FX-001");
    assert!(mcp.close_stdin_and_wait(Duration::from_secs(20)), "the server exits after stdin closes");
}

#[test]
fn initialize_instructions_mention_auto_reload() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    let res = mcp.initialize();
    let text = res.get("instructions").and_then(|i| i.as_str()).unwrap_or_default();
    assert!(text.contains("reloads automatically"), "instructions describe auto-reload; got {text:?}");
}

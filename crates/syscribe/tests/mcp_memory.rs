//! `REQ-TRS-MCP-MEM-000` / `TC-TRS-MCP-MEM-000`: the MCP server keeps its memory
//! bounded and observable on a model of 12,000 elements, through load and guarded
//! writes. Resident memory is read back through the server's own `server_stats`
//! tool, which reads `/proc` and so reports on Linux only.
//!
//! The ceilings are the measured release-build figures (62 MB steady, 126 MB peak)
//! with headroom for debug builds and other allocators; the point is to
//! catch a regression that doubles memory or lets it grow with every write.

mod common;
use common::*;
use serde_json::json;

fn write_model(root: &std::path::Path, n: usize) {
    for d in ["Reqs", "Arch", "Tests"] {
        std::fs::create_dir_all(root.join(d)).unwrap();
        std::fs::write(root.join(d).join("_index.md"), format!("---\ntype: Package\nname: {d}\n---\n")).unwrap();
    }
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    let body = "This element describes behaviour in some detail. ".repeat(20);
    for i in 1..=n {
        std::fs::write(
            root.join(format!("Reqs/REQ-MEM-{i:04}.md")),
            format!("---\ntype: Requirement\nid: REQ-MEM-{i:04}\nname: Requirement {i}\nstatus: approved\nreqDomain: software\n---\n\n{body}\n"),
        )
        .unwrap();
        std::fs::write(
            root.join(format!("Arch/Part{i}.md")),
            format!("---\ntype: PartDef\nname: Part{i}\ndomain: software\nsatisfies: [REQ-MEM-{i:04}]\nfeatures:\n  - name: p\n    type: Port\n    typedBy: Arch::Part{i}\n---\n\n{body}\n"),
        )
        .unwrap();
        std::fs::write(
            root.join(format!("Tests/TC-MEM-{i:04}.md")),
            format!("---\ntype: TestCase\nid: TC-MEM-{i:04}\nname: Test {i}\nstatus: active\ntestLevel: L2\nverifies: [REQ-MEM-{i:04}]\n---\n\n{body}\n"),
        )
        .unwrap();
    }
}

#[test]
fn server_stats_reports_the_model_and_memory() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let stats = mcp.call_tool("server_stats", json!({}));
    assert!(stats["elements"].as_u64().unwrap() > 0, "{stats}");
    assert!(stats["bodyBytes"].is_u64(), "{stats}");
    assert!(stats["elementRecordBytes"].as_u64().unwrap() > 100, "{stats}");
    if cfg!(target_os = "linux") {
        let rss = stats["residentKb"].as_u64().expect("resident memory is reported on Linux");
        let peak = stats["peakResidentKb"].as_u64().unwrap();
        assert!(rss > 0 && peak >= rss, "{stats}");
    } else {
        assert!(stats["residentKb"].is_null(), "{stats}");
    }
}

#[test]
fn an_element_record_stays_small() {
    // Rarely-set frontmatter fields live in boxed tiers; a regression that moves
    // them back inline would put this near 7 KB again.
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let stats = mcp.call_tool("server_stats", json!({}));
    let bytes = stats["elementRecordBytes"].as_u64().unwrap();
    assert!(bytes < 3000, "an element record is {bytes} bytes");
}

#[cfg(target_os = "linux")]
#[test]
fn twelve_thousand_elements_stay_within_budget_and_do_not_grow_with_writes() {
    let dir = std::env::temp_dir().join(format!("syscribe-mem-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    write_model(&dir, 4000);

    let mut mcp = Mcp::start(&dir);
    mcp.initialize();
    let loaded = mcp.call_tool("server_stats", json!({}));
    assert!(loaded["elements"].as_u64().unwrap() >= 12_000, "{loaded}");
    let steady_mb = loaded["residentKb"].as_u64().unwrap() / 1024;
    assert!(steady_mb < 160, "12,000 elements use {steady_mb} MB resident (budget 160)");

    let mut after_first = 0;
    for i in 0..3 {
        let res = mcp.call_tool(
            "create_element",
            json!({"qname": format!("Arch::Added{i}"), "type": "PartDef", "dry_run": false}),
        );
        assert_eq!(res["written"], json!(true), "{res}");
        // The live model's findings are kept between writes: a later write's delta names
        // only its own file, never what an earlier write already introduced.
        let delta = serde_json::to_string(&res["validationDelta"]).unwrap();
        for earlier in 0..i {
            assert!(!delta.contains(&format!("Added{earlier}.md")), "write {i} repeats a finding of write {earlier}: {delta}");
        }
        if i == 0 {
            after_first = mcp.call_tool("server_stats", json!({}))["residentKb"].as_u64().unwrap() / 1024;
        }
    }
    // A dry run of a large model rebuilds it afterwards: it must still answer.
    let dry = mcp.call_tool("create_element", json!({"qname": "Arch::DryOnly", "type": "PartDef"}));
    assert_eq!(dry["written"], json!(false));
    let got = mcp.call_tool("get_element", json!({"ref": "Arch::Added0"}));
    assert_eq!(got["qname"], "Arch::Added0", "the model is back after a dry run: {got}");
    let end = mcp.call_tool("server_stats", json!({}));
    let end_mb = end["residentKb"].as_u64().unwrap() / 1024;
    let peak_mb = end["peakResidentKb"].as_u64().unwrap() / 1024;
    assert!(peak_mb < 190, "peak resident {peak_mb} MB through three guarded writes (budget 190): the live model must not sit beside the candidate");
    assert!(
        end_mb <= after_first + 25,
        "resident memory grew from {after_first} MB to {end_mb} MB over two more writes"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

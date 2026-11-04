//! TC-TRS-LSPPREVIEW-001 (GH #426): preview before write over the language server.

mod common;
use common::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn staged(tag: &str) -> PathBuf {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model");
    let dst = std::env::temp_dir().join(format!("syscribe-lsppre-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dst);
    copy(&src, &dst);
    dst
}
fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy(&e.path(), &p);
        } else {
            std::fs::copy(e.path(), p).unwrap();
        }
    }
}

fn uri_path(uri: &str) -> PathBuf {
    PathBuf::from(uri.strip_prefix("file://").unwrap())
}

/// Apply a WorkspaceEdit (documentChanges operations) to the file system, as a client would.
fn apply_ws_edit(edit: &Value) {
    for op in edit["documentChanges"].as_array().unwrap() {
        match op["kind"].as_str() {
            Some("create") => {
                let p = uri_path(op["uri"].as_str().unwrap());
                std::fs::create_dir_all(p.parent().unwrap()).unwrap();
                std::fs::write(p, "").unwrap();
            }
            Some("delete") => std::fs::remove_file(uri_path(op["uri"].as_str().unwrap())).unwrap(),
            _ => {
                let p = uri_path(op["textDocument"]["uri"].as_str().unwrap());
                let text = std::fs::read_to_string(&p).unwrap_or_default();
                let lines: Vec<&str> = text.split_inclusive('\n').collect();
                let mut out = text.clone();
                for e in op["edits"].as_array().unwrap() {
                    let (s, en) = (e["range"]["start"]["line"].as_u64().unwrap() as usize, e["range"]["end"]["line"].as_u64().unwrap() as usize);
                    assert_eq!((e["range"]["start"]["character"].as_u64(), e["range"]["end"]["character"].as_u64()), (Some(0), Some(0)), "line-aligned edits");
                    out = format!("{}{}{}", lines[..s.min(lines.len())].concat(), e["newText"].as_str().unwrap(), lines[en.min(lines.len())..].concat());
                }
                std::fs::write(p, out).unwrap();
            }
        }
    }
}

fn cli(m: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(m).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), o.status.code().unwrap_or(-1))
}

fn bind_params(value: Value) -> Value {
    json!({"action": "bind", "configuration": "CONF-UAV-DEPLOY-001", "target": "Features::Propulsion.motorKV", "value": value})
}

#[test]
fn a_bind_preview_reports_and_offers_an_edit_without_writing() {
    let m = staged("bind");
    let before = dir_hash(&m);
    let mut lsp = Lsp::start(&m);
    let init = lsp.initialize();
    assert!(init["capabilities"]["experimental"]["syscribe"]["lspRequests"].as_array().unwrap().iter().any(|r| r == "syscribe/preview"));
    let p = lsp.custom_request("syscribe/preview", bind_params(json!(1100)));
    assert_eq!(p["commitable"], true, "{p}");
    assert_eq!((p["report"]["written"].clone(), p["report"]["previous"]["value"].clone()), (json!(false), json!(1000)));
    assert!(p["report"]["diff"].as_str().unwrap().contains("+  Features::Propulsion.motorKV: 1100"));
    assert_eq!(dir_hash(&m), before, "the preview wrote nothing");

    // The edit it offers produces exactly what the CLI writes, and the undo restores the original.
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    let original = std::fs::read_to_string(&conf).unwrap();
    apply_ws_edit(&p["edit"]);
    let by_edit = std::fs::read_to_string(&conf).unwrap();
    let twin = staged("bind-cli");
    assert_eq!(cli(&twin, &["bind", "CONF-UAV-DEPLOY-001", "Features::Propulsion.motorKV", "1100"]).1, 0);
    assert_eq!(by_edit, std::fs::read_to_string(twin.join("Configurations/CONF-UAV-DEPLOY-001.md")).unwrap());
    apply_ws_edit(&p["undo"]);
    assert_eq!(std::fs::read_to_string(&conf).unwrap(), original);
    // Only the changed line is in the edit.
    assert_eq!(p["edit"]["documentChanges"].as_array().unwrap().len(), 1);
    lsp.shutdown();
    let _ = (std::fs::remove_dir_all(&m), std::fs::remove_dir_all(&twin));
}

#[test]
fn a_preview_that_would_add_an_error_says_so_and_is_not_commitable() {
    let m = staged("bad");
    let mut lsp = Lsp::start(&m);
    lsp.initialize();
    let p = lsp.custom_request("syscribe/preview", bind_params(json!(5000)));
    assert_eq!(p["commitable"], false);
    assert!(p["report"]["refused"].as_str().unwrap().contains("would introduce"));
    assert!(p["report"]["newErrors"].to_string().contains("E205"), "{p}");
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_scaffold_preview_creates_a_file_and_undo_deletes_it() {
    let m = staged("scaf");
    let mut lsp = Lsp::start(&m);
    lsp.initialize();
    let p = lsp.custom_request("syscribe/preview", json!({"action": "scaffold", "kind": "feature", "name": "Gimbal", "under": "Features"}));
    assert_eq!((p["commitable"].clone(), p["report"]["target"].clone()), (json!(true), json!("FEAT-GIMBAL")), "{p}");
    let kinds: Vec<Option<&str>> = p["edit"]["documentChanges"].as_array().unwrap().iter().map(|o| o["kind"].as_str()).collect();
    assert_eq!(kinds, [Some("create"), None], "a create, then the content");
    assert!(!m.join("Features/Gimbal.md").exists());
    apply_ws_edit(&p["edit"]);
    let text = std::fs::read_to_string(m.join("Features/Gimbal.md")).unwrap();
    assert!(text.starts_with("---\nid: FEAT-GIMBAL\ntype: FeatureDef\nname: Gimbal"), "{text}");
    apply_ws_edit(&p["undo"]);
    assert!(!m.join("Features/Gimbal.md").exists());
    // A parameter extends an existing file: one small edit.
    let q = lsp.custom_request("syscribe/preview", json!({"action": "scaffold", "kind": "parameter", "feature": "Features::Telemetry", "name": "burstMs", "range": "10..500", "default": "100"}));
    assert_eq!(q["commitable"], true, "{q}");
    assert_eq!(q["edit"]["documentChanges"].as_array().unwrap().len(), 1);
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn apply_commits_a_clean_change_and_returns_an_undo() {
    let m = staged("apply");
    let mut lsp = Lsp::start(&m);
    let init = lsp.initialize();
    assert!(init["capabilities"]["executeCommandProvider"]["commands"].as_array().unwrap().iter().any(|c| c == "syscribe.apply"));
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    let original = std::fs::read_to_string(&conf).unwrap();
    let r = lsp.execute_command("syscribe.apply", json!([bind_params(json!(1100))]));
    assert_eq!((r["written"].clone(), r["report"]["written"].clone()), (json!(true), json!(true)), "{r}");
    assert!(std::fs::read_to_string(&conf).unwrap().contains("motorKV: 1100"));
    assert!(r.get("edit").is_none(), "the change is already made");
    // The server's own model followed: a request now sees the new value.
    let t = lsp.custom_request("syscribe/timeline", json!({"configuration": "CONF-UAV-DEPLOY-001"}));
    assert!(t["effective"]["parameters"].to_string().contains("1100"), "{t}");
    // The undo puts the file back exactly.
    apply_ws_edit(&r["undo"]);
    assert_eq!(std::fs::read_to_string(&conf).unwrap(), original);
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_rejected_apply_leaves_the_model_untouched() {
    let m = staged("reject");
    let before = dir_hash(&m);
    let mut lsp = Lsp::start(&m);
    lsp.initialize();
    for params in [bind_params(json!(5000)), json!({"action": "scaffold", "kind": "feature", "name": "Bad-Name", "under": "Features"}), json!({"action": "bind"}), json!({"action": "teleport"})] {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| lsp.execute_command("syscribe.apply", json!([params.clone()]))));
        assert!(r.is_err(), "{params} is refused with an LSP error");
        assert_eq!(dir_hash(&m), before, "{params} wrote nothing");
    }
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| lsp.custom_request("syscribe/preview", json!({"action": "bind", "configuration": "CONF-NOPE", "target": "Features::Telemetry", "value": true}))));
    assert!(r.is_err());
    let _ = std::fs::remove_dir_all(&m);
}

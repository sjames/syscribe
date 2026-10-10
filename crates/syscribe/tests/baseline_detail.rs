//! TC-TRS-BLFIX-001 / GH #261, #262.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Repo {
    root: PathBuf,
    model: PathBuf,
}

fn git(root: &Path, args: &[&str]) {
    let o = Command::new("git")
        .current_dir(root)
        .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

fn req(body: &str) -> String {
    format!("---\ntype: Requirement\nid: REQ-BD-001\nname: r\nstatus: draft\nreqDomain: software\n---\n\n{body}\n")
}

fn repo() -> Repo {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-bld-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let model = root.join("model");
    std::fs::create_dir_all(model.join("Reqs")).unwrap();
    std::fs::write(model.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(model.join("Reqs/REQ-BD-001.md"), req("The system shall start fast.")).unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "init"]);
    Repo { root, model }
}

fn run(r: &Repo, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&r.model).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn commit(r: &Repo, msg: &str) {
    git(&r.root, &["add", "-A"]);
    git(&r.root, &["commit", "-q", "-m", msg]);
}

fn baseline(r: &Repo, id: &str, tag: &str) {
    let (o, c) = run(r, &["baseline", "create", "--tag", tag, "--id", id]);
    assert_eq!(c, 0, "{o}");
    commit(r, &format!("baseline {id}"));
}

fn edit(r: &Repo, body: &str) {
    std::fs::write(r.model.join("Reqs/REQ-BD-001.md"), req(body)).unwrap();
}

/// Two baselines around an edit (v1 -> v2).
fn two_baselines() -> Repo {
    let r = repo();
    baseline(&r, "BL-V1", "v1");
    edit(&r, "The system shall start in under one second.");
    commit(&r, "edit");
    baseline(&r, "BL-V2", "v2");
    r
}

#[test]
fn manifest_files_are_relative_to_the_git_root() {
    let r = two_baselines();
    let text = std::fs::read_to_string(r.root.join("baselines/BL-V1.manifest.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let f = v["elements"][0]["file"].as_str().unwrap();
    assert!(!f.starts_with('/'), "{f}");
    assert!(f.starts_with("model/"), "{f}");
}

#[test]
fn detail_diff_reconstructs_content_from_a_subdirectory_model_root() {
    let r = two_baselines();
    let (out, c) = run(&r, &["baseline", "diff", "BL-V1", "BL-V2", "--detail"]);
    assert_eq!(c, 0, "{out}");
    assert!(!out.contains("not retrievable"), "{out}");
    assert!(out.contains("- The system shall start fast.") && out.contains("+ The system shall start in under one second."), "{out}");
}

#[test]
fn legacy_absolute_manifest_paths_still_resolve() {
    let r = two_baselines();
    for id in ["BL-V1", "BL-V2"] {
        let p = r.root.join(format!("baselines/{id}.manifest.json"));
        let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        for e in v["elements"].as_array_mut().unwrap() {
            let rel = e["file"].as_str().unwrap().to_string();
            e["file"] = serde_json::Value::String(format!("/some/other/machine/{rel}"));
        }
        std::fs::write(&p, serde_json::to_string_pretty(&v).unwrap()).unwrap();
    }
    let (out, _) = run(&r, &["baseline", "diff", "BL-V1", "BL-V2", "--detail"]);
    assert!(!out.contains("not retrievable"), "{out}");
    assert!(out.contains("+ The system shall start in under one second."), "{out}");
}

#[test]
fn verify_skips_a_superseded_baseline() {
    let r = two_baselines();
    // BL-V1 drifted (the requirement was edited after it). Mark it superseded.
    let p = r.model.join("Baselines/BL-V1.md");
    let t = std::fs::read_to_string(&p).unwrap();
    assert!(t.contains("status: draft"), "{t}");
    std::fs::write(&p, t.replacen("status: draft", "status: superseded", 1)).unwrap();
    commit(&r, "supersede");
    let (out, c) = run(&r, &["baseline", "verify", "--all"]);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("BL-V1: skipped (superseded)"), "{out}");
    assert!(out.contains("BL-V2: OK"), "{out}");
}

#[test]
fn current_diff_and_verify_detail_list_the_changed_element() {
    let r = two_baselines();
    edit(&r, "The system shall start in under two seconds.");
    let (out, c) = run(&r, &["baseline", "diff", "BL-V2", "--current"]);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("changed (1)") && out.contains("REQ-BD-001"), "{out}");
    let (out, c) = run(&r, &["baseline", "verify", "BL-V2", "--detail"]);
    assert_eq!(c, 2, "{out}");
    assert!(out.contains("FAIL") && out.contains("REQ-BD-001"), "{out}");
}

#[test]
fn current_diff_lists_added_and_removed_elements() {
    let r = two_baselines();
    std::fs::write(
        r.model.join("Reqs/REQ-BD-002.md"),
        "---\ntype: Requirement\nid: REQ-BD-002\nname: r2\nstatus: draft\nreqDomain: software\n---\n\nThe system shall stop.\n",
    )
    .unwrap();
    std::fs::remove_file(r.model.join("Reqs/REQ-BD-001.md")).unwrap();
    let (out, _) = run(&r, &["baseline", "diff", "BL-V2", "--current"]);
    assert!(out.contains("added (1)") && out.contains("REQ-BD-002"), "{out}");
    assert!(out.contains("removed (1)") && out.contains("REQ-BD-001"), "{out}");
}

fn set_status(r: &Repo, id: &str, from: &str, to: &str) {
    let p = r.model.join(format!("Baselines/{id}.md"));
    let t = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, t.replacen(&format!("status: {from}"), &format!("status: {to}"), 1)).unwrap();
}

#[test]
fn superseded_still_reports_a_tampered_manifest() {
    // GH #262 review: only content drift is skipped for a superseded baseline.
    let r = two_baselines();
    set_status(&r, "BL-V1", "draft", "superseded");
    let mp = r.root.join("baselines/BL-V1.manifest.json");
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&mp).unwrap()).unwrap();
    v["aggregateHash"] = serde_json::Value::String("blake3:deadbeef".into());
    std::fs::write(&mp, serde_json::to_string_pretty(&v).unwrap()).unwrap();
    commit(&r, "tamper");
    let (out, c) = run(&r, &["baseline", "verify", "--all"]);
    assert_eq!(c, 2, "{out}");
    assert!(out.contains("BL-V1: FAIL") && out.contains("manifest aggregate"), "{out}");
    assert!(!out.contains("content drift") || !out.contains("BL-V1: FAIL — content drift"), "{out}");
}

#[test]
fn current_diff_reports_an_unresolvable_scope_config_instead_of_everything_removed() {
    let r = two_baselines();
    // A feature model must exist for an unknown config to be an error (otherwise the
    // variability dimension is dormant and the scope is read flat).
    std::fs::create_dir_all(r.model.join("Features")).unwrap();
    std::fs::write(
        r.model.join("Features/A.md"),
        "---\ntype: FeatureDef\nid: FEAT-BD-100\nname: A\ngroupKind: optional\n---\n\nF.\n",
    )
    .unwrap();
    let p = r.model.join("Baselines/BL-V2.md");
    let t = std::fs::read_to_string(&p).unwrap();
    // Point the frozen scope at a configuration that does not exist.
    assert!(t.contains("frozenScope: {}"), "{t}");
    let t = t.replacen("frozenScope: {}", "frozenScope:\n  config: CONF-NOPE-001", 1);
    std::fs::write(&p, t).unwrap();
    let (out, c) = run(&r, &["baseline", "diff", "BL-V2", "--current"]);
    assert_eq!(c, 1, "{out}");
    assert!(out.to_lowercase().contains("scope") && out.contains("CONF-NOPE-001"), "{out}");
    assert!(!out.contains("removed (1)"), "{out}");
}

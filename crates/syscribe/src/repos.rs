//! Multi-repository composition surface (§14, GH #62). Read-only listing plus a
//! `sync` that checks out each pinned `ref:`. Reflects the `[repos]` table of the
//! model-root `.syscribe.toml` as loaded into [`ValidateConfig::repos`].

use std::path::Path;
use std::process::Command;

use syscribe_model::config::{LoadedRepo, ValidateConfig};

/// `repos list [--json]` — configured repos with their paths, refs, and on-disk status.
pub fn cmd_list(cfg: &ValidateConfig, json: bool) {
    if json {
        let arr: Vec<serde_json::Value> = cfg
            .repos
            .iter()
            .map(|r| {
                serde_json::json!({
                    "alias": r.alias,
                    "path": r.config.path,
                    "root": r.config.root,
                    "ref": r.config.git_ref,
                    "onDisk": r.exists,
                    "modelRoot": r.model_root.display().to_string(),
                    "circular": r.circular,
                    "elements": r.qnames.len(),
                    "stableIds": r.stable_ids.len(),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({ "repos": arr })).unwrap()
        );
        return;
    }
    if cfg.repos.is_empty() {
        println!("No repositories configured ([repos] in .syscribe.toml).");
        return;
    }
    println!("| Alias | Path | Ref | On disk | Status |");
    println!("|---|---|---|---|---|");
    for r in &cfg.repos {
        println!(
            "| {} | {} | {} | {} | {} |",
            r.alias,
            r.config.path,
            r.config.git_ref.as_deref().unwrap_or("—"),
            if r.exists { "✓" } else { "✗" },
            status_label(r),
        );
    }
}

/// `repos status [--json]` — whether each pinned repo is at its configured ref.
/// Exits `2` if any pinned repo is out of sync (or its path is missing).
pub fn cmd_status(cfg: &ValidateConfig, json: bool) -> i32 {
    let mut out_of_sync = false;
    let rows: Vec<(String, &'static str, String)> = cfg
        .repos
        .iter()
        .map(|r| {
            let (state, detail) = repo_sync_state(r);
            if matches!(state, SyncState::OutOfSync | SyncState::Missing) {
                out_of_sync = true;
            }
            (r.alias.clone(), state.label(), detail)
        })
        .collect();

    if json {
        let arr: Vec<serde_json::Value> = rows
            .iter()
            .map(|(a, s, d)| serde_json::json!({ "alias": a, "status": s, "detail": d }))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({ "repos": arr, "inSync": !out_of_sync })
            )
            .unwrap()
        );
    } else if cfg.repos.is_empty() {
        println!("No repositories configured ([repos] in .syscribe.toml).");
    } else {
        println!("| Alias | Status | Detail |");
        println!("|---|---|---|");
        for (a, s, d) in &rows {
            println!("| {} | {} | {} |", a, s, d);
        }
    }
    if out_of_sync {
        2
    } else {
        0
    }
}

/// `repos sync [--all | <alias>]` — for repos with a `ref:`, run `git fetch` then
/// `git checkout <ref>` in the repo directory. Returns the process exit code.
pub fn cmd_sync(cfg: &ValidateConfig, target: Option<&str>, all: bool) -> i32 {
    let selected: Vec<&LoadedRepo> = if all {
        cfg.repos.iter().collect()
    } else if let Some(alias) = target {
        match cfg.repos.iter().find(|r| r.alias == alias) {
            Some(r) => vec![r],
            None => {
                eprintln!("Error: no repo '{}' in [repos]", alias);
                return 1;
            }
        }
    } else {
        eprintln!("Usage: repos sync [--all | <alias>]");
        return 1;
    };

    let mut failed = false;
    for r in selected {
        let Some(git_ref) = r.config.git_ref.as_deref() else {
            println!("• {} — no ref configured, skipping", r.alias);
            continue;
        };
        // The git working directory is the repo root (path), not the model root.
        let repo_dir = repo_root_dir(cfg, r);
        if !repo_dir.exists() {
            eprintln!("✗ {} — repo path '{}' does not exist", r.alias, r.config.path);
            failed = true;
            continue;
        }
        print!("• {} → {} … ", r.alias, git_ref);
        match git_checkout(&repo_dir, git_ref) {
            Ok(()) => println!("ok"),
            Err(e) => {
                println!("failed");
                eprintln!("  {}", e);
                failed = true;
            }
        }
    }
    if failed {
        1
    } else {
        0
    }
}

/// Where a repo's git working directory lives: `<model_root>/<path>` (the repo
/// root), with the model `root` subdir stripped off the loaded model_root.
fn repo_root_dir(cfg: &ValidateConfig, r: &LoadedRepo) -> std::path::PathBuf {
    let base = Path::new(&r.config.path);
    if base.is_absolute() {
        base.to_path_buf()
    } else if let Some(model_root) = &cfg.model_root {
        model_root.join(base)
    } else {
        base.to_path_buf()
    }
}

enum SyncState {
    NoRef,
    InSync,
    OutOfSync,
    Missing,
    NotGit,
}

impl SyncState {
    fn label(&self) -> &'static str {
        match self {
            SyncState::NoRef => "no ref pinned",
            SyncState::InSync => "in sync",
            SyncState::OutOfSync => "out of sync",
            SyncState::Missing => "missing",
            SyncState::NotGit => "not a git repo",
        }
    }
}

fn status_label(r: &LoadedRepo) -> String {
    if r.circular {
        return "circular import".to_string();
    }
    repo_sync_state(r).1
}

/// Determine a repo's sync state against its configured ref via
/// `git describe --tags --exact-match` / `rev-parse`.
fn repo_sync_state(r: &LoadedRepo) -> (SyncState, String) {
    if !r.exists {
        return (SyncState::Missing, "path not on disk".to_string());
    }
    let Some(want) = r.config.git_ref.as_deref() else {
        return (SyncState::NoRef, "—".to_string());
    };
    // Run git in the model root's repo (walk up to the work tree is handled by git).
    let dir = &r.model_root;
    let head = git_out(dir, &["rev-parse", "HEAD"]);
    let want_sha = git_out(dir, &["rev-parse", &format!("{want}^{{commit}}")]);
    match (head, want_sha) {
        (Some(h), Some(w)) if h.trim() == w.trim() => {
            (SyncState::InSync, format!("at {want}"))
        }
        (Some(_), Some(_)) => (SyncState::OutOfSync, format!("HEAD ≠ {want}")),
        _ => (SyncState::NotGit, "git unavailable".to_string()),
    }
}

fn git_out(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git").arg("-C").arg(dir).args(args).output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        None
    }
}

fn git_checkout(dir: &Path, git_ref: &str) -> Result<(), String> {
    let fetch = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["fetch", "--tags", "--quiet"])
        .status()
        .map_err(|e| format!("git fetch: {e}"))?;
    if !fetch.success() {
        return Err("git fetch failed".to_string());
    }
    let co = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["checkout", "--quiet", git_ref])
        .status()
        .map_err(|e| format!("git checkout: {e}"))?;
    if co.success() {
        Ok(())
    } else {
        Err(format!("git checkout {git_ref} failed"))
    }
}

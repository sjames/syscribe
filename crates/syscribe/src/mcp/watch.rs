//! File-watch auto-reload for the MCP store (REQ-TRS-MCP-048, GH #181).
//!
//! A `notify` watcher covers the model root (which holds `.syscribe.toml` and
//! the `.syscribe/results.json` sidecar) and every `[repos]` peer root. Event
//! bursts are debounced; afterwards a cheap fingerprint of the model inputs
//! (relative path + size + mtime, see [`is_input_file`]) is compared with the
//! one recorded when the current store was loaded, and the store is reloaded
//! only if they differ. The store's own writes already reload it (recording the
//! post-write fingerprint), so they never cause a second reload here.
//!
//! The fresh store is built off the lock and swapped in under the write lock.
//! A failed walk, or one that would introduce a new frontmatter parse failure
//! (a half-saved file), is deferred: the current store is kept and the next
//! event retries. Watching is best-effort — a watcher that cannot be created is
//! logged once and the server keeps serving without it.

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use rmcp::model::{LoggingLevel, LoggingMessageNotificationParam};
use rmcp::{Peer, RoleServer};
use serde_json::{json, Value};
use tokio::sync::{mpsc, RwLock};
use walkdir::WalkDir;

use syscribe_model::config::ValidateConfig;
use syscribe_model::element::{ParseIssue, RawElement};

use super::store::McpStore;

/// Quiet period that ends a burst of file events.
const DEBOUNCE: Duration = Duration::from_millis(250);

/// What the watcher covers and the fingerprint hashes, derived from a loaded store.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WatchInputs {
    /// The model root first, then every existing `[repos]` peer model root.
    pub roots: Vec<PathBuf>,
    /// `foreignFormat:`/`annotationFormat:` package directories: every file
    /// under them is an input (their content is not `.md`).
    pub opaque_dirs: Vec<PathBuf>,
}

impl WatchInputs {
    pub fn derive(model_root: &Path, elements: &[RawElement], config: &ValidateConfig) -> Self {
        let mut roots = vec![model_root.to_path_buf()];
        for repo in &config.repos {
            if repo.exists && !roots.contains(&repo.model_root) {
                roots.push(repo.model_root.clone());
            }
        }
        let mut opaque_dirs: Vec<PathBuf> = elements
            .iter()
            .filter(|e| e.frontmatter.foreign_format.is_some() || e.frontmatter.annotation_format.is_some())
            .filter_map(|e| {
                let p = Path::new(&e.file_path);
                (p.file_name()? == "_index.md").then(|| p.parent().map(Path::to_path_buf))?
            })
            .collect();
        opaque_dirs.sort();
        opaque_dirs.dedup();
        Self { roots, opaque_dirs }
    }
}

/// True for directories whose content is never a model input: VCS metadata and
/// the regenerable `.syscribe/cache/`.
fn is_ignored_dir(rel: &Path) -> bool {
    let comps: Vec<&str> = rel
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();
    comps.iter().any(|c| matches!(*c, ".git" | ".hg" | ".svn"))
        || comps.windows(2).any(|w| w[0] == ".syscribe" && w[1] == "cache")
}

/// Editor temp/swap/backup files (Vim `*.swp`/`4913`, Emacs `.#*`/`*~`, `*.tmp`).
fn is_temp_name(name: &str) -> bool {
    name.ends_with(".swp")
        || name.ends_with(".swo")
        || name.ends_with(".swx")
        || name.ends_with('~')
        || name.starts_with(".#")
        || name == "4913"
        || name.ends_with(".tmp")
}

/// True if `rel` (relative to a watched root) is ignored as an event source.
fn is_ignored_rel(rel: &Path) -> bool {
    is_ignored_dir(rel)
        || rel
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(is_temp_name)
}

/// The files the fingerprint covers: `.md`, `.sysml`, `.kerml`, `.rhai`
/// (validation scripts), `.syscribe.toml`, `.sysmlignore`,
/// `.syscribe/results.json`, and anything under an opaque package directory.
fn is_input_file(rel: &Path, abs: &Path, opaque_dirs: &[PathBuf]) -> bool {
    if is_ignored_rel(rel) {
        return false;
    }
    let ext = rel.extension().and_then(|e| e.to_str()).unwrap_or("");
    if matches!(ext, "md" | "sysml" | "kerml" | "rhai") {
        return true;
    }
    if rel == Path::new(".syscribe.toml")
        || rel == Path::new(".sysmlignore")
        || rel == Path::new(".syscribe/results.json")
    {
        return true;
    }
    opaque_dirs.iter().any(|d| abs.starts_with(d))
}

/// Hash of (root index, relative path, size, mtime) over every input file.
pub fn fingerprint(inputs: &WatchInputs) -> u64 {
    let mut entries: Vec<(usize, PathBuf, u64, u128)> = Vec::new();
    for (i, root) in inputs.roots.iter().enumerate() {
        let walker = WalkDir::new(root).follow_links(false).into_iter().filter_entry(|e| {
            let rel = e.path().strip_prefix(root).unwrap_or(e.path());
            !(e.file_type().is_dir() && is_ignored_dir(rel))
        });
        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = entry.path().strip_prefix(root).unwrap_or(entry.path());
            if !is_input_file(rel, entry.path(), &inputs.opaque_dirs) {
                continue;
            }
            let Ok(meta) = entry.metadata() else { continue };
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            entries.push((i, rel.to_path_buf(), meta.len(), mtime));
        }
    }
    entries.sort();
    let mut h = std::collections::hash_map::DefaultHasher::new();
    inputs.roots.hash(&mut h);
    entries.hash(&mut h);
    h.finish()
}

/// A file whose frontmatter fails to parse in `fresh` but parsed in `current`
/// (or is new) and looks torn — invalid YAML, or an empty / unterminated
/// frontmatter. A file that was already broken in `current` does not count,
/// so a long-standing broken file never blocks auto-reload.
fn transient_breakage(fresh: &McpStore, current: &McpStore) -> Option<String> {
    let already: HashSet<&str> = current
        .elements
        .iter()
        .filter(|e| e.parse_issue.is_some())
        .map(|e| e.file_path.as_str())
        .collect();
    fresh.elements.iter().find_map(|e| {
        if already.contains(e.file_path.as_str()) {
            return None;
        }
        let torn = match &e.parse_issue {
            Some(ParseIssue::YamlError(_)) => true,
            Some(ParseIssue::NoFrontmatter) => std::fs::read_to_string(&e.file_path)
                .map(|t| {
                    let t = t.trim_start_matches('\u{FEFF}');
                    t.trim().is_empty() || t.starts_with("---")
                })
                .unwrap_or(true),
            None => false,
        };
        torn.then(|| e.file_path.clone())
    })
}

async fn log(peer: &Peer<RoleServer>, level: LoggingLevel, data: Value) {
    #[allow(deprecated)]
    let _ = peer
        .notify_logging_message(LoggingMessageNotificationParam {
            level,
            logger: Some("syscribe".to_string()),
            data,
        })
        .await;
}

async fn warn(peer: &Peer<RoleServer>, data: Value) {
    eprintln!("syscribe mcp: {data}");
    log(peer, LoggingLevel::Warning, data).await;
}

/// A watcher whose watches are registered but whose event loop has not started.
pub struct Armed {
    watcher: notify::RecommendedWatcher,
    rx: mpsc::UnboundedReceiver<()>,
    attempted: Vec<PathBuf>,
    /// Watch-registration failures, reported once the client is connected.
    warnings: Vec<Value>,
}

/// Create the watcher and register `roots` *before* the server starts serving,
/// so no edit made after the client connects can slip past it. `Err` carries a
/// warning when no watcher could be created or nothing could be watched.
pub fn arm(roots: &[PathBuf]) -> Result<Armed, Value> {
    let (tx, rx) = mpsc::unbounded_channel::<()>();
    let filter_roots = roots.to_vec();
    let handler = move |res: notify::Result<notify::Event>| {
        let Ok(ev) = res else { return };
        let relevant = ev.paths.is_empty()
            || ev.paths.iter().any(|p| {
                let rel = filter_roots.iter().find_map(|r| p.strip_prefix(r).ok()).unwrap_or(p);
                !is_ignored_rel(rel)
            });
        if relevant {
            let _ = tx.send(());
        }
    };
    let watcher = notify::recommended_watcher(handler)
        .map_err(|e| json!({"event": "watch_unavailable", "error": e.to_string()}))?;
    let mut armed = Armed { watcher, rx, attempted: Vec::new(), warnings: Vec::new() };
    if armed.add_watches(roots) == 0 {
        return Err(armed.warnings.pop().unwrap_or_else(|| json!({"event": "watch_unavailable"})));
    }
    Ok(armed)
}

impl Armed {
    /// Watch every root not yet attempted (so a failure is reported only once
    /// per root); returns how many new watches succeeded.
    fn add_watches(&mut self, roots: &[PathBuf]) -> usize {
        use notify::{RecursiveMode, Watcher};
        let mut ok = 0;
        for root in roots {
            if self.attempted.contains(root) {
                continue;
            }
            self.attempted.push(root.clone());
            match self.watcher.watch(root, RecursiveMode::Recursive) {
                Ok(()) => ok += 1,
                Err(e) => self.warnings.push(json!({
                    "event": "watch_unavailable",
                    "path": root.display().to_string(),
                    "error": e.to_string(),
                })),
            }
        }
        ok
    }

    async fn flush_warnings(&mut self, peer: &Peer<RoleServer>) {
        for w in std::mem::take(&mut self.warnings) {
            warn(peer, w).await;
        }
    }
}

/// Run the event loop in a background task. It never keeps the process alive:
/// the task and its watcher are dropped with the runtime once stdin closes.
/// `armed` is the result of [`arm`]; an `Err` is logged once and nothing runs.
pub fn spawn(armed: Result<Armed, Value>, store: Arc<RwLock<McpStore>>, peer: Peer<RoleServer>) {
    tokio::spawn(async move {
        let mut armed = match armed {
            Ok(a) => a,
            Err(w) => return warn(&peer, w).await,
        };
        armed.flush_warnings(&peer).await;
        // Catch anything that changed between the initial walk and the watch
        // registration (no event would report it).
        let mut pending = true;
        loop {
            if !pending {
                // Wait for the first event of a burst.
                if armed.rx.recv().await.is_none() {
                    return;
                }
            }
            pending = false;
            // Debounce: wait for a quiet period.
            loop {
                match tokio::time::timeout(DEBOUNCE, armed.rx.recv()).await {
                    Ok(Some(())) => continue,
                    Ok(None) => return,
                    Err(_) => break,
                }
            }
            if let Some(new_roots) = check_and_reload(&store, &peer).await {
                armed.add_watches(&new_roots);
                armed.flush_warnings(&peer).await;
            }
        }
    });
}

/// Reload the store if its inputs changed since it was loaded. Returns the
/// store's watch roots after a successful reload (to watch any new peer root).
async fn check_and_reload(store: &Arc<RwLock<McpStore>>, peer: &Peer<RoleServer>) -> Option<Vec<PathBuf>> {
    // Bounded: each retry means another reload landed while we were loading.
    for _ in 0..5 {
        // Taking the read lock waits out any in-flight write tool (which holds
        // the write lock through its own reload), so its fingerprint is current.
        let (seen, inputs, root) = {
            let s = store.read().await;
            (s.fingerprint, s.inputs.clone(), s.model_root.clone())
        };
        let probe = inputs.clone();
        let now = tokio::task::spawn_blocking(move || fingerprint(&probe)).await.ok()?;
        if now == seen {
            return None;
        }
        let loaded = tokio::task::spawn_blocking(move || McpStore::load_with(&root, Some(&inputs))).await;
        let fresh = match loaded {
            Ok(Ok(fresh)) => fresh,
            Ok(Err(e)) => {
                warn(peer, json!({"event": "reload_deferred", "source": "watch", "reason": e.to_string()})).await;
                return None;
            }
            Err(e) => {
                warn(peer, json!({"event": "reload_deferred", "source": "watch", "reason": e.to_string()})).await;
                return None;
            }
        };
        let mut s = store.write().await;
        if s.fingerprint != seen {
            // Someone else (a write tool, `reload`) reloaded meanwhile — re-evaluate.
            continue;
        }
        if let Some(file) = transient_breakage(&fresh, &s) {
            drop(s);
            warn(
                peer,
                json!({
                    "event": "reload_deferred",
                    "source": "watch",
                    "reason": "frontmatter does not parse (half-saved file?)",
                    "file": file,
                }),
            )
            .await;
            return None;
        }
        let count = fresh.elements.len();
        let roots = fresh.inputs.roots.clone();
        *s = fresh;
        drop(s);
        log(peer, LoggingLevel::Info, json!({"event": "reload", "source": "watch", "count": count})).await;
        let _ = peer.notify_resource_list_changed().await;
        return Some(roots);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_vcs_cache_and_editor_files() {
        assert!(is_ignored_rel(Path::new(".git/index")));
        assert!(is_ignored_rel(Path::new(".syscribe/cache/summaries.json")));
        assert!(is_ignored_rel(Path::new("Req/.REQ-001.md.swp")));
        assert!(is_ignored_rel(Path::new("Req/REQ-001.md~")));
        assert!(is_ignored_rel(Path::new("Req/.#REQ-001.md")));
        assert!(is_ignored_rel(Path::new("Req/4913")));
        assert!(!is_ignored_rel(Path::new(".syscribe/results.json")));
        assert!(!is_ignored_rel(Path::new("Req/REQ-001.md")));
    }

    #[test]
    fn input_files() {
        let none: Vec<PathBuf> = Vec::new();
        let chk = |p: &str| is_input_file(Path::new(p), Path::new(p), &none);
        assert!(chk("A/B.md"));
        assert!(chk("Sub/x.sysml"));
        assert!(chk(".syscribe.toml"));
        assert!(chk(".syscribe/results.json"));
        assert!(!chk("A/notes.txt"));
        assert!(!chk(".git/HEAD"));
        let opaque = vec![PathBuf::from("/m/Fw")];
        assert!(is_input_file(Path::new("Fw/main.c"), Path::new("/m/Fw/main.c"), &opaque));
    }
}

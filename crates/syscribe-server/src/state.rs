use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use petgraph::graph::NodeIndex;
use syscribe_model::config::ValidateConfig;
use syscribe_model::element::RawElement;
use syscribe_model::graph::{build_graph, ModelGraph};
use syscribe_model::mutate::{compute_baseline, guarded_write_cached, Baseline, GuardedWriteOutcome};
use syscribe_model::resolver::Resolver;
use syscribe_model::walker::walk_model;

pub struct ModelStore {
    pub elements: Vec<RawElement>,
    pub graph: ModelGraph,
    pub node_idx: HashMap<String, NodeIndex>,
    pub resolver: Resolver,
    /// The `<defs>…</defs>` block from `_diagram-symbols.svg`, injected into
    /// every served diagram SVG so that `<use href="#sym-*">` resolves.
    pub symbol_defs: String,
    /// Validation config carrying the model root and the `[links]` table
    /// (REQ-TRS-LINK-001/005). Used to resolve a per-element hosted source URL
    /// for the detail panel's "view source" icon; `[links]`-inert by default.
    pub config: ValidateConfig,
    /// Root directory the model was walked from. Needed by the guarded-write
    /// routes (`routes::mutate`) to drive `syscribe_model::mutate::guard::
    /// guarded_write` (candidate copy + commit both happen against real
    /// filesystem roots, not just in-memory state) and by [`ModelStore::reload`]
    /// to re-walk after a successful commit.
    pub model_root: PathBuf,
    /// The same broadcast sender `new_state` hands back separately (for
    /// `main.rs`'s watcher task and the `/ws` route's `Extension<ReloadTx>`),
    /// stashed here too so [`ModelStore::commit`] can push a reload event
    /// itself right after a successful guarded write, without every mutation
    /// handler in `routes::mutate` threading its own copy of the sender
    /// through as a separate extractor.
    pub reload_tx: ReloadTx,
    /// The live model's validation findings as a guarded write compares them, kept
    /// between writes (a commit already has the new model's) and dropped by any
    /// reload from disk. See `syscribe mcp`'s store, which does the same.
    pub baseline: Option<Baseline>,
    /// Elements at or above which a guarded write drops the live model while it builds the
    /// candidate; [`RELEASE_ABOVE`] unless a test lowers it.
    pub release_above: usize,
}

/// A model of at least this many elements is dropped from memory while a guarded
/// write builds its candidate, so one model is held, not two (`REQ-TRS-MCP-MEM-000`).
pub const RELEASE_ABOVE: usize = 3000;

pub type SharedState = Arc<RwLock<ModelStore>>;

/// Channel used to broadcast "model reloaded" events to WebSocket clients.
/// The String payload is a JSON event (e.g. `{"event":"reload"}`).
pub type ReloadTx = broadcast::Sender<String>;

/// Extract the `<defs>…</defs>` block from `_diagram-symbols.svg` in the model
/// root. Returns an empty string if the file is absent or malformed.
///
/// Shared by the initial load in `main`, the file-watcher's reload, and
/// [`ModelStore::reload`] — the one place any of them needs to recompute the
/// symbol defs from disk.
pub fn load_symbol_defs(model_root: &Path) -> String {
    let path = model_root.join("_diagram-symbols.svg");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return String::new();
    };
    if let (Some(start), Some(end)) = (content.find("<defs>"), content.find("</defs>")) {
        content[start..end + 7].to_string() // 7 = len("</defs>")
    } else {
        String::new()
    }
}

pub fn new_state(
    elements: Vec<RawElement>,
    symbol_defs: String,
    config: ValidateConfig,
    model_root: PathBuf,
) -> (SharedState, ReloadTx) {
    let (graph, node_idx) = build_graph(&elements);
    let resolver = Resolver::new(&elements);
    let (tx, _) = broadcast::channel(64);
    let store = Arc::new(RwLock::new(ModelStore {
        elements,
        graph,
        node_idx,
        resolver,
        symbol_defs,
        config,
        model_root,
        reload_tx: tx.clone(),
        baseline: None,
        release_above: RELEASE_ABOVE,
    }));
    (store, tx)
}

impl ModelStore {
    /// Re-walk `self.model_root` and refresh every derived field in place:
    /// elements, graph, resolver, symbol defs, and validation config.
    ///
    /// This is the one "rebuild the store from disk" code path — both the
    /// file-watcher's reload (`main.rs::spawn_watcher`) and every guarded-write
    /// mutation route (`routes::mutate`) call this after a successful on-disk
    /// change, so there is never a second, independently-drifting copy of the
    /// rebuild logic.
    pub fn reload(&mut self) -> anyhow::Result<()> {
        let elements = walk_model(&self.model_root)?;
        // Config first: it (re)installs the `[linkTypes]` vocabulary that
        // `build_graph` reads for user-defined edges (REQ-TRS-LINKTYPE-009).
        let config = ValidateConfig::with_model_root(&self.model_root);
        let (graph, node_idx) = build_graph(&elements);
        let resolver = Resolver::new(&elements);
        let symbol_defs = load_symbol_defs(&self.model_root);
        self.elements = elements;
        self.graph = graph;
        self.node_idx = node_idx;
        self.resolver = resolver;
        self.symbol_defs = symbol_defs;
        self.config = config;
        self.baseline = None;
        Ok(())
    }

    /// Run a guarded write against this store's model root and, on a
    /// successful commit, reload the in-memory state and broadcast a reload
    /// event to every `/ws` subscriber — the single orchestration point every
    /// mutating handler in `routes::mutate` now calls, replacing what used to
    /// be a `guarded_write(...) -> if outcome.written { self.reload() }` block
    /// duplicated verbatim in all six handlers.
    ///
    /// `apply` is invoked once against a throwaway candidate copy (to compute
    /// the validation delta) and, only on a clean commit, a second time
    /// against the real model root — see `guarded_write`'s own doc comment
    /// for the full dry-run/gate semantics.
    pub fn commit<F>(
        &mut self,
        dry_run: bool,
        gate: bool,
        allow_new_errors: bool,
        apply: F,
    ) -> GuardedWriteOutcome
    where
        F: Fn(&Path) -> Result<(), String>,
    {
        self.run_guarded(dry_run, gate, allow_new_errors, apply, None::<&dyn Fn(&[RawElement]) -> ()>, |_| true).0
    }

    /// The guarded write against this store's kept baseline, with a large model's live copy
    /// dropped while the candidate is built; reloads (and keeps the candidate's findings as
    /// the next baseline) after a commit, and rebuilds the dropped model after anything else.
    fn run_guarded<F, I>(
        &mut self,
        dry_run: bool,
        gate: bool,
        allow_new_errors: bool,
        apply: F,
        inspect: Option<&dyn Fn(&[RawElement]) -> I>,
        proceed: impl Fn(&Option<I>) -> bool,
    ) -> (GuardedWriteOutcome, Option<I>)
    where
        F: Fn(&Path) -> Result<(), String>,
    {
        let baseline = match self.baseline.take() {
            Some(b) => b,
            None => compute_baseline(&self.model_root, &self.elements, &self.config),
        };
        let released = self.elements.len() >= self.release_above;
        if released {
            self.elements = Vec::new();
            let (graph, node_idx) = build_graph(&self.elements);
            self.graph = graph;
            self.node_idx = node_idx;
            self.resolver = Resolver::new(&self.elements);
        }
        let (outcome, inspected, candidate) =
            guarded_write_cached(&self.model_root, &baseline, &self.config, dry_run, gate, allow_new_errors, apply, inspect, proceed);
        if outcome.written {
            if let Err(e) = self.reload() {
                tracing::warn!("model reload after commit failed: {e}");
            } else {
                self.baseline = candidate;
                let _ = self.reload_tx.send(r#"{"event":"reload"}"#.to_string());
            }
        } else {
            if released {
                if let Err(e) = self.reload() {
                    tracing::warn!("rebuilding the model after a refused write failed: {e}");
                }
            }
            self.baseline = Some(baseline);
        }
        (outcome, inspected)
    }

    /// [`Self::commit`] that also inspects the candidate model and lets the caller
    /// veto the commit after seeing it (`REQ-TRS-FMED-004`: the feature editor
    /// previews an edit's effect on the model's validity). `proceed` returning
    /// `false` leaves the model untouched, as a dry run does.
    #[allow(clippy::too_many_arguments)]
    pub fn commit_inspect<F, I>(
        &mut self,
        dry_run: bool,
        gate: bool,
        apply: F,
        inspect: &dyn Fn(&[RawElement]) -> I,
        proceed: impl Fn(&Option<I>) -> bool,
    ) -> (GuardedWriteOutcome, Option<I>)
    where
        F: Fn(&Path) -> Result<(), String>,
    {
        self.run_guarded(dry_run, gate, false, apply, Some(inspect), proceed)
    }
}

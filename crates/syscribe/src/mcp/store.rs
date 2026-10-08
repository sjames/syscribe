//! The in-memory model store shared by every MCP tool. Mirrors the server's
//! `ModelStore` blueprint but is owned by the `syscribe mcp` subcommand.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use petgraph::graph::NodeIndex;
use syscribe_model::config::ValidateConfig;
use syscribe_model::element::RawElement;
use syscribe_model::graph::{build_graph, ModelGraph};
use syscribe_model::resolver::Resolver;
use syscribe_model::walker::walk_model;

use super::watch::{fingerprint, WatchInputs};

/// Parsed model + derived indices, rebuilt after every successful write.
pub struct McpStore {
    pub elements: Vec<RawElement>,
    pub graph: ModelGraph,
    pub node_idx: HashMap<String, NodeIndex>,
    pub resolver: Resolver,
    pub config: ValidateConfig,
    pub model_root: PathBuf,
    /// What the file watcher covers for this store (model root, peer roots,
    /// opaque package dirs) — REQ-TRS-MCP-048.
    pub inputs: WatchInputs,
    /// Fingerprint of `inputs` taken no later than the walk that built this
    /// store: a watcher reloads only when the on-disk fingerprint differs.
    pub fingerprint: u64,
    /// The live model's validation findings as a guarded write compares them, kept
    /// between writes: computing them means validating the whole model, and a write
    /// that commits already has the new model's (its candidate's). `None` until the
    /// first write, and after any reload from disk.
    pub baseline: Option<syscribe_model::mutate::Baseline>,
}

/// A model of at least this many elements is dropped from memory while a guarded
/// write builds its candidate and while it reloads, so one model is held, not two
/// (`REQ-TRS-MCP-MEM-000`). Smaller models keep the live copy: it is cheap, and it
/// avoids a rebuild after a dry run.
pub const RELEASE_ABOVE: usize = 3000;

impl McpStore {
    /// Walk `model_root`, build the graph + resolver, and load the validation config.
    pub fn load(model_root: &Path) -> anyhow::Result<Self> {
        let initial = WatchInputs { roots: vec![model_root.to_path_buf()], opaque_dirs: Vec::new() };
        Self::load_with(model_root, Some(&initial))
    }

    /// [`Self::load`], fingerprinting the inputs `prev` (those of the store being
    /// replaced) *before* the walk, so an edit racing the walk is never masked:
    /// the recorded fingerprint can only be older than what the walk read. If
    /// the fresh store's inputs differ (a new peer repo or opaque package), they
    /// are fingerprinted again after the walk.
    pub fn load_with(model_root: &Path, prev: Option<&WatchInputs>) -> anyhow::Result<Self> {
        let pre = prev.map(|p| (p.clone(), fingerprint(p)));
        let elements = walk_model(model_root)?;
        // Load the config first: it installs the model's `[linkTypes]` vocabulary,
        // which `build_graph` reads for user-defined edges (REQ-TRS-LINKTYPE-009).
        let mut config = ValidateConfig::with_model_root(model_root);
        let (graph, node_idx) = build_graph(&elements);
        let resolver = Resolver::new(&elements);
        // Load any ingested test verdicts so coverage/evidence tools reflect them
        // (and pick up a sidecar written by ingest_results after a store rebuild).
        config.results = syscribe_model::results::ResultsData::load_sidecar(model_root);
        let inputs = WatchInputs::derive(model_root, &elements, &config);
        let fingerprint = match pre {
            Some((p, fp)) if p == inputs => fp,
            _ => fingerprint(&inputs),
        };
        Ok(Self {
            elements,
            graph,
            node_idx,
            resolver,
            config,
            model_root: model_root.to_path_buf(),
            inputs,
            fingerprint,
            baseline: None,
        })
    }

    /// Re-read the model from disk, replacing all derived state in place.
    pub fn reload(&mut self) -> anyhow::Result<()> {
        // A large model is let go first so the replacement is the only one in memory.
        // If the load then fails the store is left empty; the watcher's fingerprint
        // still differs from disk, so it tries again.
        if self.elements.len() >= RELEASE_ABOVE {
            self.release_model();
        }
        let fresh = Self::load_with(&self.model_root, Some(&self.inputs))?;
        *self = fresh;
        super::memory::release_free_memory();
        Ok(())
    }

    /// Drop the in-memory model (elements, graph, resolver), keeping the configuration
    /// and the watch state, and hand the memory back to the operating system.
    pub fn release_model(&mut self) {
        self.elements = Vec::new();
        let (graph, node_idx) = build_graph(&self.elements);
        self.graph = graph;
        self.node_idx = node_idx;
        self.resolver = Resolver::new(&self.elements);
        super::memory::release_free_memory();
    }
}

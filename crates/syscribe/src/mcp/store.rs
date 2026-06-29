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

/// Parsed model + derived indices, rebuilt after every successful write.
pub struct McpStore {
    pub elements: Vec<RawElement>,
    pub graph: ModelGraph,
    pub node_idx: HashMap<String, NodeIndex>,
    pub resolver: Resolver,
    pub config: ValidateConfig,
    pub model_root: PathBuf,
}

impl McpStore {
    /// Walk `model_root`, build the graph + resolver, and load the validation config.
    pub fn load(model_root: &Path) -> anyhow::Result<Self> {
        let elements = walk_model(model_root)?;
        let (graph, node_idx) = build_graph(&elements);
        let resolver = Resolver::new(&elements);
        let config = ValidateConfig::with_model_root(model_root);
        Ok(Self {
            elements,
            graph,
            node_idx,
            resolver,
            config,
            model_root: model_root.to_path_buf(),
        })
    }

    /// Re-read the model from disk, replacing all derived state in place.
    pub fn reload(&mut self) -> anyhow::Result<()> {
        let fresh = Self::load(&self.model_root)?;
        *self = fresh;
        Ok(())
    }
}

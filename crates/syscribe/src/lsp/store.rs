//! The in-memory model store backing the `syscribe lsp` server. Same shape as
//! `crate::mcp::store::McpStore` (ADR-SYS-LSP-001 consequence: this duplicates the
//! walk→graph→resolver→config blueprint a third time; extracting a shared
//! `ModelStore` into `syscribe-model` is worth doing once, but not a v1 prerequisite).

use std::path::{Path, PathBuf};

use syscribe_model::config::ValidateConfig;
use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::walker::walk_model;

/// Parsed model + derived indices, rebuilt in full on every reload. No graph/node
/// index here (unlike `McpStore`): v1 capabilities (diagnostics, definition,
/// references, hover, workspace/symbol) only need `elements` + `resolver` +
/// `config`. Add the containment graph back if a later capability needs
/// tree/neighbor traversal.
pub struct LspStore {
    pub elements: Vec<RawElement>,
    pub resolver: Resolver,
    pub config: ValidateConfig,
    pub model_root: PathBuf,
}

impl LspStore {
    /// Walk `model_root` and build the resolver + validation config. `model_root` is
    /// canonicalized so `RawElement::file_path` values are absolute and comparable
    /// against `file://` document URIs from the client.
    pub fn load(model_root: &Path) -> anyhow::Result<Self> {
        let model_root = model_root.canonicalize()?;
        let elements = walk_model(&model_root)?;
        let resolver = Resolver::new(&elements);
        let config = ValidateConfig::with_model_root(&model_root);
        Ok(Self { elements, resolver, config, model_root })
    }

    /// Re-read the model from disk, replacing all derived state in place. On error the
    /// previous state is left untouched (REQ-TRS-LSP-007): a failed reload must not
    /// clear a good store.
    pub fn reload(&mut self) -> anyhow::Result<()> {
        let fresh = Self::load(&self.model_root)?;
        *self = fresh;
        Ok(())
    }
}

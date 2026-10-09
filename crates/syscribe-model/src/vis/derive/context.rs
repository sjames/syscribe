//! The model-level inputs a derived safety diagram needs beyond the elements
//! (GH #223 follow-up): the `[cyber]` configuration of `.syscribe.toml` (the
//! attack-tree roll-up must score exactly as `W035` does) and the
//! `.syscribe/results.json` sidecar (a GSN test case wears its real verdict).
//!
//! Neither lives in the element list, and the derivers are called from many
//! places (server, CLI export, MCP) that all know the model root, so the root
//! is registered once at start-up / reload ([`set_model_root`]) and the inputs
//! are read from it on demand. With no root registered (the unit tests, a
//! library embedder) the defaults apply: default [`CyberConfig`], no results.

use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::cyber_config::CyberConfig;
use crate::results::ResultsData;

static ROOT: RwLock<Option<PathBuf>> = RwLock::new(None);

/// Register the model root whose `.syscribe.toml` and results sidecar the
/// derived diagrams read.
pub fn set_model_root(root: &Path) {
    if let Ok(mut g) = ROOT.write() {
        *g = Some(root.to_path_buf());
    }
}

fn root() -> Option<PathBuf> {
    ROOT.read().ok().and_then(|g| g.clone())
}

/// The model's `[cyber]` configuration (the default when no root is registered).
pub fn cyber_config() -> CyberConfig {
    root().map(|r| CyberConfig::load(&r)).unwrap_or_default()
}

/// The results sidecar of the registered model root, if there is one.
pub fn results() -> Option<ResultsData> {
    root().and_then(|r| ResultsData::load_sidecar(&r))
}

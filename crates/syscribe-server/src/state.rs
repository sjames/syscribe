use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use petgraph::graph::NodeIndex;
use syscribe_model::element::RawElement;
use syscribe_model::graph::{build_graph, ModelGraph};
use syscribe_model::resolver::Resolver;

pub struct ModelStore {
    pub elements: Vec<RawElement>,
    pub graph: ModelGraph,
    pub node_idx: HashMap<String, NodeIndex>,
    pub resolver: Resolver,
    /// The `<defs>…</defs>` block from `_diagram-symbols.svg`, injected into
    /// every served diagram SVG so that `<use href="#sym-*">` resolves.
    pub symbol_defs: String,
}

pub type SharedState = Arc<RwLock<ModelStore>>;

/// Channel used to broadcast "model reloaded" events to WebSocket clients.
/// The String payload is a JSON event (e.g. `{"event":"reload"}`).
pub type ReloadTx = broadcast::Sender<String>;

pub fn new_state(elements: Vec<RawElement>, symbol_defs: String) -> (SharedState, ReloadTx) {
    let (graph, node_idx) = build_graph(&elements);
    let resolver = Resolver::new(&elements);
    let store = Arc::new(RwLock::new(ModelStore {
        elements,
        graph,
        node_idx,
        resolver,
        symbol_defs,
    }));
    let (tx, _) = broadcast::channel(64);
    (store, tx)
}

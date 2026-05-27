#![deny(warnings)]

use anyhow::Result;
use axum::{routing::{get, patch}, Router};
use clap::Parser;
use std::path::PathBuf;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(about = "Syscribe model browser")]
struct Cli {
    /// Path to the model root directory
    #[arg(default_value = "model")]
    model: PathBuf,

    /// Address to listen on
    #[arg(long, default_value = "0.0.0.0:3000")]
    bind: String,
}

mod routes;
mod state;

use routes::api_graph::{get_children, get_connections};
use routes::elements::{get_element, list_elements};
use routes::ui::{diagram, element_detail, index, tree_items};
use routes::validation::get_validation;
use routes::write::{patch_layout, put_element};
use routes::ws::ws_handler;
use state::{new_state, ReloadTx, SharedState};
use syscribe_model::walker::walk_model;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("sysml=debug".parse()?))
        .init();

    let cli = Cli::parse();
    let model_root = cli.model;

    info!("Loading model from {:?}", model_root);
    let elements = walk_model(&model_root)?;
    info!("Loaded {} elements", elements.len());

    let symbol_defs = load_symbol_defs(&model_root);
    let (shared, reload_tx) = new_state(elements, symbol_defs);

    spawn_watcher(model_root.clone(), shared.clone(), reload_tx.clone());

    let app = Router::new()
        .route("/", get(index))
        .route("/ui/tree", get(tree_items))
        .route("/ui/detail/{*qname}", get(element_detail))
        .route("/ui/diagram/{*qname}", get(diagram))
        .route("/api/elements", get(list_elements))
        .route("/api/elements/{*qname}", get(get_element).put(put_element))
        .route("/api/children", get(get_children))
        .route("/api/connections", get(get_connections))
        .route("/api/diagrams/layout/{*qname}", patch(patch_layout))
        .route("/api/validation", get(get_validation))
        .route("/ws", get(ws_handler))
        .layer(axum::Extension(reload_tx))
        .layer(CorsLayer::permissive())
        .with_state(shared);

    info!("Listening on http://{}", cli.bind);
    let listener = tokio::net::TcpListener::bind(&cli.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Extract the `<defs>…</defs>` block from `_diagram-symbols.svg` in the
/// model root. Returns an empty string if the file is absent or malformed.
fn load_symbol_defs(model_root: &std::path::Path) -> String {
    let path = model_root.join("_diagram-symbols.svg");
    let Ok(content) = std::fs::read_to_string(&path) else {
        tracing::warn!("_diagram-symbols.svg not found at {:?}", path);
        return String::new();
    };
    // Extract the <defs>…</defs> block
    if let (Some(start), Some(end)) = (content.find("<defs>"), content.find("</defs>")) {
        content[start..end + 7].to_string() // 7 = len("</defs>")
    } else {
        tracing::warn!("No <defs> block found in _diagram-symbols.svg");
        String::new()
    }
}

fn spawn_watcher(model_root: PathBuf, state: SharedState, reload_tx: ReloadTx) {
    tokio::task::spawn_blocking(move || {
        use notify::{RecursiveMode, Watcher};
        use std::sync::mpsc;

        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if res.is_ok() {
                let _ = tx.send(());
            }
        })
        .expect("watcher creation failed");

        watcher
            .watch(&model_root, RecursiveMode::Recursive)
            .expect("watch failed");

        loop {
            // Block until at least one event arrives
            if rx.recv().is_ok() {
                // Debounce: wait 500 ms and drain any further events
                std::thread::sleep(std::time::Duration::from_millis(500));
                while rx.try_recv().is_ok() {}

                // Reload model from disk
                match walk_model(&model_root) {
                    Ok(elements) => {
                        let (graph, node_idx) =
                            syscribe_model::graph::build_graph(&elements);
                        let resolver =
                            syscribe_model::resolver::Resolver::new(&elements);
                        let symbol_defs = load_symbol_defs(&model_root);

                        // Write into the shared store from inside the blocking thread
                        let rt = tokio::runtime::Handle::current();
                        rt.block_on(async {
                            let mut store = state.write().await;
                            store.elements = elements;
                            store.graph = graph;
                            store.node_idx = node_idx;
                            store.resolver = resolver;
                            store.symbol_defs = symbol_defs;
                        });

                        let _ = reload_tx.send(r#"{"event":"reload"}"#.to_string());
                        tracing::info!("Model reloaded");
                    }
                    Err(e) => tracing::warn!("Reload failed: {}", e),
                }
            }
        }
    });
}

//! WASM foreign-format plugin CLI surface (ADR-SYS-PLUGIN-001, REQ-TRS-PLUGIN-*).
//! Currently just the `run --dry-run` debug loop; `[plugins]` execution itself
//! happens automatically inside `walk_model` for every other command.

use std::path::Path;

use syscribe_model::element::RawElement;
use syscribe_model::plugins;

/// `plugins run <alias> --dry-run` — invoke one configured plugin and print its
/// raw envelope JSON, without merging it into the graph. The fastest debug loop
/// for a plugin author: no need to re-run `validate` to see what came back.
pub fn cmd_run(model_root: &Path, elements: &[RawElement], alias: &str) -> i32 {
    match plugins::dry_run(alias, model_root, elements) {
        Ok(raw) => {
            match serde_json::from_str::<serde_json::Value>(&raw) {
                Ok(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap_or(raw)),
                // Print verbatim even if it isn't valid JSON — seeing exactly
                // what the plugin returned is the point of a dry run.
                Err(_) => println!("{raw}"),
            }
            0
        }
        Err(msg) => {
            eprintln!("plugin '{alias}': {msg}");
            1
        }
    }
}

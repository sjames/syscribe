//! stdio-subprocess plugin invocation (`ADR-SYS-PLUGIN-002`). `run` invokes
//! one configured plugin live and prints its raw envelope JSON — no merge,
//! no validation. Fast feedback loop for a plugin author, including seeing
//! exactly what came back when it doesn't parse.

use std::path::Path;

use syscribe_model::config::ValidateConfig;
use syscribe_model::plugins::{build_request, find_alias_package, runtime};

/// `plugins run <alias> --dry-run` — currently `--dry-run` is the only
/// supported mode; it's required rather than implied so the call site is
/// unambiguous about what it does (invoke live, print raw, no merge).
pub fn cmd_run(cfg: &ValidateConfig, model_root: &Path, alias: &str) -> i32 {
    let Some(entry) = cfg.plugins.get(alias) else {
        eprintln!("Error: no [plugins.{alias}] entry in .syscribe.toml");
        return 1;
    };
    let Some(pkg) = find_alias_package(model_root, alias) else {
        eprintln!("Error: no package in the model declares foreignFormat: {alias}");
        return 1;
    };

    let req = build_request(&pkg, model_root);

    match runtime::invoke_raw(entry, &req, model_root) {
        Ok(raw_json) => {
            println!("{raw_json}");
            0
        }
        Err(e) => {
            eprintln!("plugin '{alias}' failed: {e}");
            1
        }
    }
}

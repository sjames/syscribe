//! `[plugins.*]` table of `.syscribe.toml` (`ADR-SYS-PLUGIN-002`).
//!
//! Parsing this table (and the `E550`/`E551` config-shape diagnostics that use
//! it) needs nothing beyond `toml`/`serde` — the actual subprocess-spawning
//! machinery lives in [`super::runtime`], kept separate so a caller that only
//! wants to inspect configuration never pulls in process-execution code.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

/// One entry in the `[plugins]` table of `.syscribe.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginEntry {
    /// The executable to spawn — a bare name resolved against `PATH`, or a
    /// path resolved relative to the model root (absolute paths used verbatim).
    pub command: String,
    /// Extra arguments passed to `command`, in order.
    #[serde(default)]
    pub args: Vec<String>,
    /// Wall-clock execution budget before the plugin process is killed (`W550`).
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_timeout_ms() -> u64 {
    10_000
}

/// View of `.syscribe.toml` carrying only the `[plugins.*]` table.
#[derive(Debug, Default, Deserialize)]
struct PluginsRootToml {
    #[serde(default)]
    plugins: BTreeMap<String, PluginEntry>,
}

/// Load the `[plugins]` table from `<model_root>/.syscribe.toml`.
///
/// Empty when the file is absent, unparseable, or has no `[plugins]` table —
/// mirrors [`crate::config`]'s `load_repos`/`load_profiles` posture: a
/// malformed `.syscribe.toml` degrades this feature to inert rather than
/// panicking the whole load.
pub fn load_plugins(model_root: &Path) -> BTreeMap<String, PluginEntry> {
    let text = match std::fs::read_to_string(model_root.join(".syscribe.toml")) {
        Ok(t) => t,
        Err(_) => return BTreeMap::new(),
    };
    toml::from_str::<PluginsRootToml>(&text)
        .map(|c| c.plugins)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A unique throwaway directory under the OS temp dir (mirrors
    /// `crate::config`'s own test helper).
    fn tempdir() -> std::path::PathBuf {
        static N: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "syscribe-plugins-cfg-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn no_syscribe_toml_yields_empty() {
        let dir = tempdir();
        assert!(load_plugins(&dir).is_empty());
    }

    #[test]
    fn malformed_toml_degrades_to_empty_not_a_panic() {
        let dir = tempdir();
        std::fs::write(dir.join(".syscribe.toml"), "[plugins.toydsl\ncommand = oops").unwrap();
        assert!(load_plugins(&dir).is_empty());
    }

    #[test]
    fn no_plugins_table_yields_empty() {
        let dir = tempdir();
        std::fs::write(dir.join(".syscribe.toml"), "[repos]\n").unwrap();
        assert!(load_plugins(&dir).is_empty());
    }

    #[test]
    fn command_required_args_and_timeout_default() {
        let dir = tempdir();
        std::fs::write(
            dir.join(".syscribe.toml"),
            "[plugins.toydsl]\ncommand = \"python3\"\n",
        )
        .unwrap();
        let plugins = load_plugins(&dir);
        let entry = &plugins["toydsl"];
        assert_eq!(entry.command, "python3");
        assert!(entry.args.is_empty());
        assert_eq!(entry.timeout_ms, 10_000);
    }

    #[test]
    fn explicit_args_and_timeout_win_over_defaults() {
        let dir = tempdir();
        std::fs::write(
            dir.join(".syscribe.toml"),
            "[plugins.toydsl]\ncommand = \"python3\"\nargs = [\"plugin.py\"]\ntimeout_ms = 3000\n",
        )
        .unwrap();
        let plugins = load_plugins(&dir);
        let entry = &plugins["toydsl"];
        assert_eq!(entry.args, vec!["plugin.py".to_string()]);
        assert_eq!(entry.timeout_ms, 3000);
    }

    #[test]
    fn multiple_entries_all_parse() {
        let dir = tempdir();
        std::fs::write(
            dir.join(".syscribe.toml"),
            "[plugins.a]\ncommand = \"a.sh\"\n\n[plugins.b]\ncommand = \"b.sh\"\n",
        )
        .unwrap();
        let plugins = load_plugins(&dir);
        assert_eq!(plugins.len(), 2);
    }
}

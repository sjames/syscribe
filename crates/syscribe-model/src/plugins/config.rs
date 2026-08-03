//! `[plugins.*]` table of `.syscribe.toml` (ADR-SYS-PLUGIN-001, REQ-TRS-PLUGIN-001).
//!
//! Deliberately no `wasmtime`/`extism` dependency here — parsing this table (and
//! the `E530`/`E532` config-shape diagnostics that use it) works the same whether
//! or not the crate was built with the `wasm-plugins` feature.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// One entry in the `[plugins]` table of `.syscribe.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginEntry {
    /// Path to the compiled `.wasm` module, relative to this model's `.syscribe.toml`.
    pub wasm: String,
    /// Wall-clock execution budget before the plugin is interrupted (`W530`).
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// Memory ceiling for the plugin instance.
    #[serde(default = "default_memory_max_bytes")]
    pub memory_max_bytes: u64,
}

fn default_timeout_ms() -> u64 {
    5000
}

fn default_memory_max_bytes() -> u64 {
    64 * 1024 * 1024
}

impl PluginEntry {
    /// Resolve [`Self::wasm`] against `model_root` (relative paths only — the
    /// convention every other `.syscribe.toml`-relative path in this crate follows).
    pub fn wasm_path(&self, model_root: &Path) -> PathBuf {
        let p = PathBuf::from(&self.wasm);
        if p.is_absolute() {
            p
        } else {
            model_root.join(p)
        }
    }
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
/// mirrors [`crate::config::load_repos`]'s posture (a malformed `.syscribe.toml`
/// degrades a composition feature to inert rather than panicking the whole load).
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

    /// A unique throwaway directory under the OS temp dir (mirrors
    /// `crate::config`'s own test helper).
    fn tempdir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
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
        std::fs::write(dir.join(".syscribe.toml"), "[plugins.sysmlv2\nwasm = oops").unwrap();
        assert!(load_plugins(&dir).is_empty());
    }

    #[test]
    fn no_plugins_table_yields_empty() {
        let dir = tempdir();
        std::fs::write(dir.join(".syscribe.toml"), "[repos]\n").unwrap();
        assert!(load_plugins(&dir).is_empty());
    }

    #[test]
    fn multiple_entries_all_parse_with_defaults() {
        let dir = tempdir();
        std::fs::write(
            dir.join(".syscribe.toml"),
            "[plugins.sysmlv2]\nwasm = \"a.wasm\"\n\n[plugins.other]\nwasm = \"b.wasm\"\ntimeout_ms = 9000\nmemory_max_bytes = 1048576\n",
        )
        .unwrap();
        let plugins = load_plugins(&dir);
        assert_eq!(plugins.len(), 2);

        let sysmlv2 = &plugins["sysmlv2"];
        assert_eq!(sysmlv2.wasm, "a.wasm");
        assert_eq!(sysmlv2.timeout_ms, 5000, "default timeout_ms should apply when unset");
        assert_eq!(sysmlv2.memory_max_bytes, 64 * 1024 * 1024, "default memory_max_bytes should apply when unset");

        let other = &plugins["other"];
        assert_eq!(other.wasm, "b.wasm");
        assert_eq!(other.timeout_ms, 9000, "explicit timeout_ms should win over the default");
        assert_eq!(other.memory_max_bytes, 1048576, "explicit memory_max_bytes should win over the default");
    }

    #[test]
    fn wasm_path_relative_resolves_against_model_root() {
        let entry = PluginEntry {
            wasm: "sub/plugin.wasm".to_string(),
            timeout_ms: default_timeout_ms(),
            memory_max_bytes: default_memory_max_bytes(),
        };
        assert_eq!(
            entry.wasm_path(Path::new("/models/uav")),
            PathBuf::from("/models/uav/sub/plugin.wasm")
        );
    }

    #[test]
    fn wasm_path_absolute_is_used_verbatim() {
        let entry = PluginEntry {
            wasm: "/opt/syscribe/plugins/plugin.wasm".to_string(),
            timeout_ms: default_timeout_ms(),
            memory_max_bytes: default_memory_max_bytes(),
        };
        assert_eq!(
            entry.wasm_path(Path::new("/models/uav")),
            PathBuf::from("/opt/syscribe/plugins/plugin.wasm")
        );
    }
}

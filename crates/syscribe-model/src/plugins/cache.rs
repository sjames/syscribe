//! Content-hash-keyed, on-disk cache of a plugin's raw envelope output
//! (ADR-SYS-PLUGIN-001, REQ-TRS-PLUGIN-*). Mirrors `syscribe`'s
//! `summarize.rs` cache convention exactly: `.syscribe/cache/<name>.json`,
//! `{ version, entries: { key -> { hash, value } } }`, dirty-flag-gated save.
//! Only compiled with `wasm-plugins` — nothing else in the crate reads or
//! writes this file.
//!
//! Only a *successful* plugin execution is ever cached — an execution
//! failure (trap, timeout, load error) always retries, since it may be
//! transient (system load, a fluke timeout) rather than a deterministic
//! property of the (wasm, content) pair. A successful run that happens to
//! return syntactically-invalid JSON *is* cached: that's still a
//! deterministic function of the same inputs, so re-invoking would reproduce
//! the identical malformed output — caching it isn't "stale", it's a
//! reproducible answer, and it saves the caller from a JIT re-compile just to
//! get the same content back.

use std::path::Path;

use serde_json::{json, Value};

/// Bump when the *shape* of what's cached changes (not the plugin's own
/// output — that's covered by the content hash) so a cache written by an
/// older syscribe-model doesn't get misread by a newer one.
const CACHE_VERSION: u32 = 1;

pub struct Cache {
    map: serde_json::Map<String, Value>,
    dirty: bool,
}

impl Cache {
    pub fn load(path: &Path) -> Cache {
        let doc = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok());
        // Invalidate on a version mismatch (a shape change the content hash
        // alone can't see), same posture as summarize.rs's cache.
        let map = doc
            .filter(|v| v.get("version").and_then(|n| n.as_u64()) == Some(CACHE_VERSION as u64))
            .and_then(|v| v.get("entries").and_then(|e| e.as_object().cloned()))
            .unwrap_or_default();
        Cache { map, dirty: false }
    }

    /// The cached raw envelope JSON for `alias` if its stored content hash
    /// still matches — `None` on any mismatch or absence (a cache miss).
    pub fn get(&self, alias: &str, hash: &str) -> Option<String> {
        let e = self.map.get(alias)?;
        if e.get("hash").and_then(|h| h.as_str()) == Some(hash) {
            e.get("envelope").and_then(|v| v.as_str()).map(str::to_string)
        } else {
            None
        }
    }

    pub fn put(&mut self, alias: &str, hash: &str, envelope: &str) {
        self.map.insert(alias.to_string(), json!({ "hash": hash, "envelope": envelope }));
        self.dirty = true;
    }

    /// No-ops (doesn't touch the file at all) when nothing changed.
    pub fn save(&self, path: &Path) {
        if !self.dirty {
            return;
        }
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let doc = json!({ "version": CACHE_VERSION, "entries": Value::Object(self.map.clone()) });
        if let Ok(s) = serde_json::to_string(&doc) {
            let _ = std::fs::write(path, s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempfile(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "syscribe-plugin-cache-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ))
    }

    #[test]
    fn missing_file_loads_empty() {
        let cache = Cache::load(&tempfile("missing"));
        assert_eq!(cache.get("sysmlv2", "anyhash"), None);
    }

    #[test]
    fn put_then_get_round_trips_with_matching_hash() {
        let mut cache = Cache::load(&tempfile("roundtrip"));
        cache.put("sysmlv2", "abc123", "{\"elements\":[]}");
        assert_eq!(cache.get("sysmlv2", "abc123"), Some("{\"elements\":[]}".to_string()));
    }

    #[test]
    fn get_with_a_different_hash_is_a_miss() {
        let mut cache = Cache::load(&tempfile("hash-mismatch"));
        cache.put("sysmlv2", "abc123", "{\"elements\":[]}");
        assert_eq!(cache.get("sysmlv2", "different-hash"), None);
    }

    #[test]
    fn save_then_load_round_trips_across_files() {
        let path = tempfile("save-load");
        let mut cache = Cache::load(&path);
        cache.put("sysmlv2", "abc123", "{\"elements\":[]}");
        cache.save(&path);

        let reloaded = Cache::load(&path);
        assert_eq!(reloaded.get("sysmlv2", "abc123"), Some("{\"elements\":[]}".to_string()));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn a_version_mismatch_in_the_file_invalidates_everything() {
        let path = tempfile("version-mismatch");
        std::fs::write(
            &path,
            serde_json::to_string(&json!({
                "version": CACHE_VERSION + 1,
                "entries": { "sysmlv2": { "hash": "abc123", "envelope": "{}" } }
            }))
            .unwrap(),
        )
        .unwrap();

        let cache = Cache::load(&path);
        assert_eq!(cache.get("sysmlv2", "abc123"), None);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn save_is_a_no_op_when_nothing_changed() {
        let path = tempfile("no-op-save");
        let cache = Cache::load(&path); // never put()'d — not dirty
        cache.save(&path);
        assert!(!path.exists(), "save() must not create a file when nothing was written");
    }
}

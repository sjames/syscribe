//! WASM execution via `extism` (ADR-SYS-PLUGIN-001, REQ-TRS-PLUGIN-003).
//!
//! Only compiled with the `wasm-plugins` feature. The plugin gets **no** real
//! filesystem access (no WASI preopens) — `extism-js`'s QuickJS runtime requires
//! WASI to be enabled for its own clock (`--wasi` is mandatory for every
//! JS/TS plugin per the js-pdk toolchain), but `allowed_paths` is left empty, so
//! the guest has no preopened directories. Instead it reads its declared
//! foreign-format subtree through three custom host functions (`fs_read`,
//! `fs_list_dir`, `fs_exists`), each of which canonicalizes the requested path
//! and rejects anything that resolves outside that subtree before touching disk.

use std::path::{Path, PathBuf};

use extism::{Function, Manifest, PluginBuilder, UserData, Wasm, PTR};

use super::config::PluginEntry;

#[path = "cache.rs"]
mod cache;

/// Instruction budget as a defense-in-depth backstop alongside `timeout_ms` —
/// generous enough that no reasonable parse run hits it, but bounds a plugin
/// stuck in a tight loop even if wall-clock interruption is ever delayed.
const FUEL_LIMIT: u64 = 5_000_000_000;

/// Run `alias`'s plugin over `pkg_dir`, consulting/populating the on-disk
/// content-hash cache at `<model_root>/.syscribe/cache/plugins.json` when
/// `use_cache` is set. `syscribe plugins run --dry-run` passes `false` so a
/// plugin author debugging always sees a guaranteed-live run. See `cache.rs`
/// for what does and doesn't get cached and why.
///
/// Returns the raw JSON output, or an error describing why the plugin itself
/// failed to run — a load failure, a wasm trap, or a timeout. The caller (§
/// `plugins::mod`) turns this `Err` into a soft `W530` finding and separately
/// parses the `Ok` JSON, tagging a parse failure `W532` — the two are kept
/// distinct so "the plugin crashed" and "the plugin ran fine but returned
/// garbage" surface as different, correctly-worded findings. Neither ever
/// aborts the rest of validation.
pub fn run(
    wasm_path: &Path,
    entry: &PluginEntry,
    pkg_dir: &Path,
    model_root: &Path,
    alias: &str,
    use_cache: bool,
) -> Result<String, String> {
    if !use_cache {
        return run_uncached(wasm_path, entry, pkg_dir);
    }

    // A hashing failure (an unreadable file mid-walk, say) just falls back to
    // an uncached run rather than failing the whole plugin invocation over a
    // caching concern.
    let Ok(hash) = content_hash(wasm_path, pkg_dir) else {
        return run_uncached(wasm_path, entry, pkg_dir);
    };

    let cache_path = model_root.join(".syscribe").join("cache").join("plugins.json");
    let mut cache = cache::Cache::load(&cache_path);
    if let Some(cached) = cache.get(alias, &hash) {
        return Ok(cached);
    }

    // Only a successful execution is cached — see cache.rs's module doc for
    // why an execution failure (this `?`) must never be.
    let raw = run_uncached(wasm_path, entry, pkg_dir)?;
    cache.put(alias, &hash, &raw);
    cache.save(&cache_path);
    Ok(raw)
}

/// A deterministic content hash of everything a plugin invocation's output
/// depends on: the compiled `.wasm` module's own bytes (so rebuilding the
/// plugin busts the cache even if the model didn't change) and every file
/// under `pkg_dir` (path + bytes, sorted by relative path so the hash is
/// independent of filesystem iteration order). `blake3` — not the
/// `DefaultHasher` `summarize.rs`'s cache uses — because a collision there
/// just forces a harmless recompute of a text summary; here it would mean
/// silently serving a *different* plugin's output into the traceability
/// graph, so the stronger, still-cheap hash earns its keep.
fn content_hash(wasm_path: &Path, pkg_dir: &Path) -> Result<String, String> {
    let wasm_bytes = std::fs::read(wasm_path).map_err(|e| format!("reading '{}': {e}", wasm_path.display()))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&wasm_bytes);
    hasher.update(b"\0");

    let mut files: Vec<PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(pkg_dir).follow_links(false) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    files.sort();

    for path in files {
        let rel = path.strip_prefix(pkg_dir).unwrap_or(&path);
        let bytes = std::fs::read(&path).map_err(|e| format!("reading '{}': {e}", path.display()))?;
        // Length-prefix each field so e.g. ("ab", "c") and ("a", "bc") can't
        // hash to the same bytes.
        hasher.update(&(rel.as_os_str().len() as u64).to_le_bytes());
        hasher.update(rel.to_string_lossy().as_bytes());
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn run_uncached(wasm_path: &Path, entry: &PluginEntry, pkg_dir: &Path) -> Result<String, String> {
    let root = pkg_dir
        .canonicalize()
        .map_err(|e| format!("package directory '{}': {e}", pkg_dir.display()))?;

    let fs_read_fn = Function::new(
        "fs_read",
        [PTR],
        [PTR],
        UserData::new(root.clone()),
        fs_read,
    )
    .with_namespace(extism::EXTISM_USER_MODULE);
    let fs_list_dir_fn = Function::new(
        "fs_list_dir",
        [PTR],
        [PTR],
        UserData::new(root.clone()),
        fs_list_dir,
    )
    .with_namespace(extism::EXTISM_USER_MODULE);
    let fs_exists_fn = Function::new(
        "fs_exists",
        [PTR],
        [PTR],
        UserData::new(root.clone()),
        fs_exists,
    )
    .with_namespace(extism::EXTISM_USER_MODULE);

    let mut manifest = Manifest::new([Wasm::file(wasm_path)]);
    manifest.timeout_ms = Some(entry.timeout_ms);
    manifest.memory.max_pages = Some(pages_for(entry.memory_max_bytes));
    // No network, ever — a foreign-format parser has no legitimate reason to
    // make HTTP calls, and this is stricter than it needs to be on purpose.
    manifest.allowed_hosts = Some(vec![]);
    // No `allowed_paths` — real filesystem access happens only via the scoped
    // host functions above, never through WASI preopens.

    let mut plugin = PluginBuilder::new(manifest)
        .with_wasi(true)
        .with_fuel_limit(FUEL_LIMIT)
        .with_functions([fs_read_fn, fs_list_dir_fn, fs_exists_fn])
        .build()
        .map_err(|e| format!("failed to load plugin: {e}"))?;

    plugin
        .call("parse", "")
        .map_err(|e| format!("execution failed: {e}"))
}

fn pages_for(max_bytes: u64) -> u32 {
    const PAGE_SIZE: u64 = 64 * 1024;
    (max_bytes.div_ceil(PAGE_SIZE)).min(u32::MAX as u64) as u32
}

/// Resolve `requested` (always relative) against `root`, rejecting anything
/// that canonicalizes outside it — the sandboxing boundary the scoped host
/// functions below all share.
fn resolve_scoped(root: &Path, requested: &str) -> Result<PathBuf, extism::Error> {
    let req = Path::new(requested);
    if req.is_absolute() {
        return Err(extism::Error::msg(format!(
            "path '{requested}' must be relative"
        )));
    }
    let joined = root.join(req);
    let canon = joined
        .canonicalize()
        .map_err(|e| extism::Error::msg(format!("'{requested}': {e}")))?;
    if !canon.starts_with(root) {
        return Err(extism::Error::msg(format!(
            "path '{requested}' escapes the plugin's subtree"
        )));
    }
    Ok(canon)
}

/// `UserData<PathBuf>` -> `PathBuf`, shared by all three host functions below.
fn root_of(user_data: &extism::UserData<PathBuf>) -> Result<PathBuf, extism::Error> {
    let arc = user_data.get()?;
    let guard = arc.lock().map_err(|_| extism::Error::msg("plugin root lock poisoned"))?;
    Ok(guard.clone())
}

extism::host_fn!(fs_read (root: PathBuf; path: String) -> Vec<u8> {
    let root = root_of(&root)?;
    let resolved = resolve_scoped(&root, &path)?;
    std::fs::read(&resolved).map_err(|e| extism::Error::msg(format!("fs_read '{path}': {e}")))
});

extism::host_fn!(fs_list_dir (root: PathBuf; path: String) -> String {
    let root = root_of(&root)?;
    let dir = resolve_scoped(&root, &path)?;
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| extism::Error::msg(format!("fs_list_dir '{path}': {e}")))? {
        let entry = entry.map_err(|e| extism::Error::msg(e.to_string()))?;
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    serde_json::to_string(&names).map_err(|e| extism::Error::msg(e.to_string()))
});

extism::host_fn!(fs_exists (root: PathBuf; path: String) -> String {
    let root = root_of(&root)?;
    // Reuses the exact same escape check as fs_read/fs_list_dir (tested below) —
    // an absolute or escaping path comes back `false`, indistinguishable from a
    // plain missing file, never leaking whether the target exists outside the root.
    let exists = resolve_scoped(&root, &path).is_ok();
    Ok(exists.to_string())
});

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique throwaway directory under the OS temp dir, containing an `inside/`
    /// subtree (what the plugin is scoped to) and an `outside/` sibling (what it
    /// must never be able to reach) — mirrors the layout an escape attempt would
    /// actually be exploiting.
    struct Sandbox {
        base: PathBuf,
        inside_root: PathBuf,
    }

    impl Sandbox {
        fn new(tag: &str) -> Self {
            let base = std::env::temp_dir().join(format!(
                "syscribe-plugin-sandbox-{tag}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos()
            ));
            let inside_root = base.join("inside");
            std::fs::create_dir_all(&inside_root).unwrap();
            std::fs::create_dir_all(base.join("outside")).unwrap();
            std::fs::write(base.join("outside/secret.txt"), b"should never be readable").unwrap();
            std::fs::write(inside_root.join("allowed.txt"), b"fine").unwrap();
            Sandbox { base, inside_root }
        }

        fn root(&self) -> PathBuf {
            self.inside_root.canonicalize().unwrap()
        }
    }

    impl Drop for Sandbox {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.base).ok();
        }
    }

    #[test]
    fn relative_path_within_root_resolves() {
        let sb = Sandbox::new("within");
        let resolved = resolve_scoped(&sb.root(), "allowed.txt").expect("should resolve");
        assert_eq!(resolved, sb.root().join("allowed.txt"));
    }

    fn write_wasm_stub(dir: &Path, bytes: &[u8]) -> PathBuf {
        let p = dir.join("stub.wasm");
        std::fs::write(&p, bytes).unwrap();
        p
    }

    #[test]
    fn content_hash_is_stable_for_identical_inputs() {
        let sb = Sandbox::new("hash-stable");
        let wasm = write_wasm_stub(&sb.base, b"fake wasm bytes");
        let a = content_hash(&wasm, &sb.root()).unwrap();
        let b = content_hash(&wasm, &sb.root()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn content_hash_changes_when_a_subtree_file_changes() {
        let sb = Sandbox::new("hash-file-change");
        let wasm = write_wasm_stub(&sb.base, b"fake wasm bytes");
        let before = content_hash(&wasm, &sb.root()).unwrap();
        std::fs::write(sb.root().join("allowed.txt"), b"different content now").unwrap();
        let after = content_hash(&wasm, &sb.root()).unwrap();
        assert_ne!(before, after);
    }

    #[test]
    fn content_hash_changes_when_the_wasm_binary_changes() {
        let sb = Sandbox::new("hash-wasm-change");
        let wasm = write_wasm_stub(&sb.base, b"version one");
        let before = content_hash(&wasm, &sb.root()).unwrap();
        std::fs::write(&wasm, b"version two, rebuilt").unwrap();
        let after = content_hash(&wasm, &sb.root()).unwrap();
        assert_ne!(before, after);
    }

    #[test]
    fn content_hash_changes_when_a_new_file_is_added() {
        let sb = Sandbox::new("hash-new-file");
        let wasm = write_wasm_stub(&sb.base, b"fake wasm bytes");
        let before = content_hash(&wasm, &sb.root()).unwrap();
        std::fs::write(sb.root().join("second.txt"), b"more content").unwrap();
        let after = content_hash(&wasm, &sb.root()).unwrap();
        assert_ne!(before, after);
    }

    #[test]
    fn content_hash_is_independent_of_no_op_field_boundary_shifts() {
        // Length-prefixing must prevent ("ab","c") and ("a","bc")-style
        // concatenation collisions across the (path, bytes) pairs. Both
        // sandboxes are reduced to exactly one file each so the two cases
        // are otherwise identical — only the path/content boundary differs.
        let sb1 = Sandbox::new("hash-boundary-1");
        std::fs::remove_file(sb1.root().join("allowed.txt")).ok();
        std::fs::write(sb1.root().join("ab"), b"c").unwrap();
        let wasm1 = write_wasm_stub(&sb1.base, b"w");
        let h1 = content_hash(&wasm1, &sb1.root()).unwrap();

        let sb2 = Sandbox::new("hash-boundary-2");
        std::fs::remove_file(sb2.root().join("allowed.txt")).ok();
        std::fs::write(sb2.root().join("a"), b"bc").unwrap();
        let wasm2 = write_wasm_stub(&sb2.base, b"w");
        let h2 = content_hash(&wasm2, &sb2.root()).unwrap();

        assert_ne!(h1, h2);
    }

    #[test]
    fn dotdot_escape_is_rejected() {
        let sb = Sandbox::new("dotdot");
        let err = resolve_scoped(&sb.root(), "../outside/secret.txt")
            .expect_err("must reject a path that escapes the plugin's subtree");
        assert!(err.to_string().contains("escapes"), "unexpected error: {err}");
    }

    #[test]
    fn absolute_path_is_rejected() {
        let sb = Sandbox::new("absolute");
        let err = resolve_scoped(&sb.root(), "/etc/passwd").expect_err("must reject an absolute path");
        assert!(err.to_string().contains("must be relative"), "unexpected error: {err}");
    }

    #[test]
    fn nonexistent_relative_path_errors_cleanly_not_a_panic() {
        let sb = Sandbox::new("missing");
        let err = resolve_scoped(&sb.root(), "no-such-file.txt").expect_err("missing file is an error, not a panic");
        assert!(!err.to_string().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escaping_root_is_rejected() {
        let sb = Sandbox::new("symlink");
        let link = sb.root().join("escape-link");
        std::os::unix::fs::symlink(sb.base.join("outside/secret.txt"), &link).unwrap();
        let err = resolve_scoped(&sb.root(), "escape-link")
            .expect_err("a symlink resolving outside the root must be rejected, not followed");
        assert!(err.to_string().contains("escapes"), "unexpected error: {err}");
    }
}

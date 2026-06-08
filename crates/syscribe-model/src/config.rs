//! Validation configuration.
//!
//! [`ValidateConfig`] carries everything the validator needs beyond the raw
//! element list: the model-root directory and repo root (so `sourceFile:` paths
//! resolve correctly), language-aware function matchers (`W009`), and ingested
//! test results (`W010`).
//!
//! The bare [`crate::validator::validate`] entry point passes
//! [`ValidateConfig::default()`], preserving the historical behaviour for the
//! web server and other callers that do not need on-disk resolution.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::matchers::MatcherConfig;
use crate::remote::RemoteHook;
use crate::results::ResultsData;

/// Where a `sourceFile:` value points, after classifying its semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceLocation {
    /// A path on the local filesystem (to be checked for existence / read).
    Local(PathBuf),
    /// A remote location addressed by URI — not resolved or read locally.
    Remote(String),
}

/// Configuration threaded into [`crate::validator::validate_with_config`].
#[derive(Debug, Clone, Default)]
pub struct ValidateConfig {
    /// Model root directory (the `-m` path). Bare relative `sourceFile:` values
    /// and `model:`-prefixed values resolve against it.
    pub model_root: Option<PathBuf>,

    /// Repository root. `repo:`-prefixed `sourceFile:` values resolve against it.
    /// Detected by walking up from the model root for a `.git` entry, overridable
    /// via `repo_root` in `<model_root>/.syscribe.toml`.
    pub repo_root: Option<PathBuf>,

    /// Language-aware function-definition matchers for function-level
    /// traceability (`W009`). Defaults cover Rust, Java, C, C++, Kotlin and
    /// shell; a `[matchers]` table in `<model_root>/.syscribe.toml` overrides
    /// per-extension.
    pub matchers: MatcherConfig,

    /// Ingested test-run results (`W010`). Loaded from
    /// `<model_root>/.syscribe/results.json`.
    pub results: Option<ResultsData>,

    /// Opt-in download hook for remote `sourceFile:` URIs. `None` (the default)
    /// means remote sources are accepted but not fetched — defining a hook in
    /// `.syscribe.toml` does **not** enable it; the CLI sets this only when
    /// `validate --fetch-remote` is passed, so validation never runs configured
    /// commands implicitly.
    pub remote_hook: Option<RemoteHook>,
}

/// Minimal view of `.syscribe.toml` for the path settings (matchers are loaded
/// separately by [`MatcherConfig`]). Unknown keys/tables are ignored.
#[derive(Debug, Default, Deserialize)]
struct PathsToml {
    #[serde(default, alias = "repoRoot")]
    repo_root: Option<String>,
}

/// A named validation severity profile (issue #18 / REQ-TRS-OUT-012).
///
/// Declared as `[profiles.<name>]` in `<model_root>/.syscribe.toml`. The
/// `promote` list names warning codes to treat as gating failures (like
/// `--deny`). The optional `sil`/`status`/`tag` fields scope promotion to
/// findings whose element matches **all** of the provided fields.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Profile {
    /// Warning codes promoted to gating failures.
    #[serde(default)]
    pub promote: Vec<String>,
    /// Scope by integrity level: element's `silLevel` stringifies to this OR
    /// `asilLevel` equals it (same matching as `list --sil`).
    #[serde(default)]
    pub sil: Option<String>,
    /// Scope by exact `status:` match.
    #[serde(default)]
    pub status: Option<String>,
    /// Scope by `tags:` membership (the element's tags contain this value).
    #[serde(default)]
    pub tag: Option<String>,
}

impl Profile {
    /// True when this profile declares no scope fields (so every finding of a
    /// promoted code is promoted regardless of the element it concerns).
    pub fn is_unscoped(&self) -> bool {
        self.sil.is_none() && self.status.is_none() && self.tag.is_none()
    }
}

/// View of `.syscribe.toml` carrying only the `[profiles.*]` tables. Unknown
/// keys/tables (`[matchers]`, `[remote]`, `repo_root`) are ignored so this parses
/// alongside the existing config.
#[derive(Debug, Default, Deserialize)]
struct ProfilesToml {
    #[serde(default)]
    profiles: HashMap<String, Profile>,
}

/// Load the named severity profiles declared in `<model_root>/.syscribe.toml`.
///
/// Returns an empty map when the file is absent or cannot be parsed (the caller
/// reports "unknown profile" when a requested name is missing).
pub fn load_profiles(model_root: &Path) -> HashMap<String, Profile> {
    match std::fs::read_to_string(model_root.join(".syscribe.toml")) {
        Ok(text) => toml::from_str::<ProfilesToml>(&text)
            .map(|c| c.profiles)
            .unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

impl ValidateConfig {
    /// Construct a config rooted at `model_root`, loading matcher overrides and
    /// the repo root from `<model_root>/.syscribe.toml` (with `.git` auto-detection).
    pub fn with_model_root(model_root: impl Into<PathBuf>) -> Self {
        let root = model_root.into();
        let (matchers, _warn) = MatcherConfig::load_from_model_root(&root);
        let results = ResultsData::load_sidecar(&root);
        let repo_root = resolve_repo_root(&root);
        Self {
            model_root: Some(root),
            repo_root,
            matchers,
            results,
            // Remote fetching is opt-in (CLI `--fetch-remote`); never enabled here.
            remote_hook: None,
        }
    }

    /// Resolve a `sourceFile:` value to a local path for checking/reading.
    ///
    /// Local forms return their path. A remote URI returns a downloaded cache
    /// path **only** when a [`RemoteHook`] is enabled and the fetch succeeds;
    /// otherwise `None` (the file is treated as unverifiable external).
    pub fn resolve_source_local(&self, value: &str) -> Option<PathBuf> {
        match self.classify_source(value) {
            SourceLocation::Local(p) => Some(p),
            SourceLocation::Remote(uri) => self.remote_hook.as_ref().and_then(|h| h.fetch(&uri)),
        }
    }

    /// Classify a `sourceFile:` value into a [`SourceLocation`], applying these
    /// semantics (so a model can choose how each path is interpreted):
    ///
    /// | Form | Meaning |
    /// |---|---|
    /// | `scheme://…` (not `file`) | remote URI — not resolved locally |
    /// | `file://…` | local path from the file URI |
    /// | `repo:<path>` | relative to the repository root |
    /// | `model:<path>` | relative to the model root |
    /// | `/abs/path` | absolute path |
    /// | `path` (bare) | relative to the model root (default) |
    pub fn classify_source(&self, value: &str) -> SourceLocation {
        let v = value.trim();

        // URI with a scheme.
        if let Some(scheme_end) = v.find("://") {
            let scheme = &v[..scheme_end];
            let is_scheme = !scheme.is_empty()
                && scheme.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'));
            if is_scheme {
                if scheme.eq_ignore_ascii_case("file") {
                    return SourceLocation::Local(file_uri_to_path(&v[scheme_end + 3..]));
                }
                return SourceLocation::Remote(v.to_string());
            }
        }

        // Explicit `repo:` / `model:` prefixes.
        if let Some(rest) = v.strip_prefix("repo:") {
            let base = self
                .repo_root
                .clone()
                .or_else(|| self.model_root.clone())
                .unwrap_or_default();
            return SourceLocation::Local(base.join(rest));
        }
        if let Some(rest) = v.strip_prefix("model:") {
            let base = self.model_root.clone().unwrap_or_default();
            return SourceLocation::Local(base.join(rest));
        }

        // Absolute path.
        let p = PathBuf::from(v);
        if p.is_absolute() {
            return SourceLocation::Local(p);
        }

        // Bare relative → model root (default), or CWD when no root is set.
        match &self.model_root {
            Some(root) => SourceLocation::Local(root.join(p)),
            None => SourceLocation::Local(p),
        }
    }
}

/// Convert the part of a `file://` URI after the scheme into a local path.
/// Handles `file:///abs` (empty host) and `file://host/abs`.
fn file_uri_to_path(after_scheme: &str) -> PathBuf {
    if after_scheme.starts_with('/') {
        PathBuf::from(after_scheme)
    } else {
        // Strip an authority component up to the next '/'.
        match after_scheme.find('/') {
            Some(i) => PathBuf::from(&after_scheme[i..]),
            None => PathBuf::from(after_scheme),
        }
    }
}

/// Determine the repo root: `repo_root` in `.syscribe.toml` (resolved against the
/// model root if relative), else the nearest ancestor containing `.git`.
fn resolve_repo_root(model_root: &Path) -> Option<PathBuf> {
    if let Ok(text) = std::fs::read_to_string(model_root.join(".syscribe.toml")) {
        if let Ok(cfg) = toml::from_str::<PathsToml>(&text) {
            if let Some(rr) = cfg.repo_root {
                let p = PathBuf::from(&rr);
                return Some(if p.is_absolute() { p } else { model_root.join(p) });
            }
        }
    }
    detect_git_root(model_root)
}

/// Walk up from `start` looking for a `.git` entry; return the directory holding it.
fn detect_git_root(start: &Path) -> Option<PathBuf> {
    let mut dir = std::fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
    loop {
        if dir.join(".git").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ValidateConfig {
        ValidateConfig {
            model_root: Some(PathBuf::from("/models/uav")),
            repo_root: Some(PathBuf::from("/work/repo")),
            ..ValidateConfig::default()
        }
    }

    #[test]
    fn bare_relative_uses_model_root() {
        assert_eq!(
            cfg().classify_source("tests/foo.rs"),
            SourceLocation::Local(PathBuf::from("/models/uav/tests/foo.rs"))
        );
    }

    #[test]
    fn model_prefix() {
        assert_eq!(
            cfg().classify_source("model:tests/foo.rs"),
            SourceLocation::Local(PathBuf::from("/models/uav/tests/foo.rs"))
        );
    }

    #[test]
    fn repo_prefix_uses_repo_root() {
        assert_eq!(
            cfg().classify_source("repo:crates/foo/src/lib.rs"),
            SourceLocation::Local(PathBuf::from("/work/repo/crates/foo/src/lib.rs"))
        );
    }

    #[test]
    fn absolute_path() {
        assert_eq!(
            cfg().classify_source("/opt/src/lib.rs"),
            SourceLocation::Local(PathBuf::from("/opt/src/lib.rs"))
        );
    }

    #[test]
    fn remote_uris() {
        assert_eq!(
            cfg().classify_source("https://example.com/a/lib.rs"),
            SourceLocation::Remote("https://example.com/a/lib.rs".to_string())
        );
        assert_eq!(
            cfg().classify_source("git+ssh://git@host/repo.git#lib.rs"),
            SourceLocation::Remote("git+ssh://git@host/repo.git#lib.rs".to_string())
        );
    }

    #[test]
    fn file_uri_is_local() {
        assert_eq!(
            cfg().classify_source("file:///abs/src/lib.rs"),
            SourceLocation::Local(PathBuf::from("/abs/src/lib.rs"))
        );
        assert_eq!(
            cfg().classify_source("file://localhost/abs/src/lib.rs"),
            SourceLocation::Local(PathBuf::from("/abs/src/lib.rs"))
        );
    }
}

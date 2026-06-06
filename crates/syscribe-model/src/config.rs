//! Validation configuration.
//!
//! [`ValidateConfig`] carries everything the validator needs beyond the raw
//! element list: the model-root directory (so on-disk paths such as
//! `sourceFile:` resolve correctly), language-aware function matchers
//! (Phase 2 / `W009`), and ingested test results (Phase 5).
//!
//! The bare [`crate::validator::validate`] entry point passes
//! [`ValidateConfig::default()`], preserving the historical behaviour for the
//! web server and other callers that do not need on-disk resolution.

use std::path::PathBuf;

use crate::matchers::MatcherConfig;
use crate::results::ResultsData;

/// Configuration threaded into [`crate::validator::validate_with_config`].
#[derive(Debug, Clone, Default)]
pub struct ValidateConfig {
    /// Model root directory. When set, on-disk references (`sourceFile:`) are
    /// resolved relative to this directory, matching the spec (§11.12, `W004`).
    /// When `None`, paths are resolved relative to the current working
    /// directory (legacy behaviour, used by tests and the web server).
    pub model_root: Option<PathBuf>,

    /// Language-aware function-definition matchers for function-level
    /// traceability (`W009`). Defaults cover Rust, Java, C, C++ and Kotlin; a
    /// `[matchers]` table in `<model_root>/.syscribe.toml` overrides per-extension.
    pub matchers: MatcherConfig,

    /// Ingested test-run results (issue #4). When present, the validator emits
    /// `W010` for `active`/`verified` TestCases whose test functions failed or
    /// were absent from the run. Loaded from `<model_root>/.syscribe/results.json`.
    pub results: Option<ResultsData>,
}

impl ValidateConfig {
    /// Construct a config rooted at `model_root`, loading matcher overrides from
    /// `<model_root>/.syscribe.toml` on top of the built-in defaults.
    pub fn with_model_root(model_root: impl Into<PathBuf>) -> Self {
        let root = model_root.into();
        let (matchers, _warn) = MatcherConfig::load_from_model_root(&root);
        let results = ResultsData::load_sidecar(&root);
        Self {
            model_root: Some(root),
            matchers,
            results,
        }
    }

    /// Resolve an on-disk reference (e.g. a `sourceFile:` value) into an
    /// absolute-or-cwd-relative path, honouring [`Self::model_root`].
    ///
    /// Absolute paths are returned unchanged. Relative paths are joined onto
    /// the model root when one is configured, otherwise returned as-is so the
    /// OS resolves them against the current working directory.
    pub fn resolve_on_disk(&self, reference: &str) -> PathBuf {
        let p = PathBuf::from(reference);
        if p.is_absolute() {
            return p;
        }
        match &self.model_root {
            Some(root) => root.join(p),
            None => p,
        }
    }
}

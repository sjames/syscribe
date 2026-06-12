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

    /// Maximum number of digits allowed in a stable-ID numeric suffix (REQ-TRS-ID-005,
    /// GH #41). `None` means the default of 8. The minimum is fixed at 3. Read from
    /// `[ids] max_digits` in `<model_root>/.syscribe.toml`; use [`Self::id_digit_max`].
    pub id_max_digits: Option<usize>,

    /// REQ-TRS-MG-* — run the gated MagicGrid validation pass (actors, mg_cell,
    /// MoE, logical/physical layering). Off by default; set from a resolved
    /// `[profiles.<name>] magicgrid = true` profile by the CLI. The base-format
    /// `refines`/`E316`/`W307` checks run regardless of this flag.
    pub magicgrid: bool,

    /// REQ-TRS-LINK-001 — hosted-source link configuration from the `[links]`
    /// table of `<model_root>/.syscribe.toml`. `None` means the feature is inert
    /// (no element resolves to a URL; diagrams/reports are exactly as before).
    pub links: Option<LinkConfig>,
}

/// REQ-TRS-LINK-001 — resolved `[links]` configuration. Carries the hosted-URL
/// template/base and the `ref` substitution. Constructed only when at least one
/// of `base_url`/`url_template` is set, so its presence means the feature is on.
#[derive(Debug, Clone)]
pub struct LinkConfig {
    /// `base_url` — the 90% case. A file's URL is `base_url`/`<path>`.
    base_url: Option<String>,
    /// `url_template` — escape hatch with `{path}`/`{qname}`/`{id}`/`{ref}`.
    url_template: Option<String>,
    /// `ref` — substituted for `{ref}` in the template (empty when unset).
    git_ref: String,
}

/// The `[links]` table of `.syscribe.toml` (REQ-TRS-LINK-001). Unknown keys are
/// ignored so this parses alongside `[profiles]`/`[matchers]`/`[remote]`.
#[derive(Debug, Default, Deserialize)]
struct LinksToml {
    #[serde(default, alias = "baseUrl")]
    base_url: Option<String>,
    #[serde(default, alias = "urlTemplate")]
    url_template: Option<String>,
    #[serde(default, rename = "ref")]
    git_ref: Option<String>,
}

/// View of `.syscribe.toml` carrying only the `[links]` table.
#[derive(Debug, Default, Deserialize)]
struct LinksRootToml {
    #[serde(default)]
    links: LinksToml,
}

impl LinkConfig {
    /// Resolve `model_relative_path` (always forward-slashed, relative to the
    /// model root) to a hosted URL, applying the REQ-TRS-LINK-001 rules. `qname`
    /// and `id` feed the `{qname}`/`{id}` template placeholders (`id` may be empty).
    pub fn resolve(&self, model_relative_path: &str, qname: &str, id: &str) -> String {
        let encoded_path = encode_path(model_relative_path);
        if let Some(tpl) = &self.url_template {
            tpl.replace("{path}", &encoded_path)
                .replace("{qname}", qname)
                .replace("{id}", id)
                .replace("{ref}", &self.git_ref)
        } else if let Some(base) = &self.base_url {
            format!("{}/{}", base.trim_end_matches('/'), encoded_path)
        } else {
            // Unreachable: a LinkConfig is only built when one of these is set.
            encoded_path
        }
    }
}

/// Percent-encode each path segment (a space → `%20`) while preserving the `/`
/// separators (REQ-TRS-LINK-001).
fn encode_path(path: &str) -> String {
    path.split('/').map(encode_segment).collect::<Vec<_>>().join("/")
}

/// Percent-encode a single path segment. Keeps the unreserved set
/// (`A–Z a–z 0–9 - _ . ~`) verbatim and `%`-encodes everything else.
fn encode_segment(seg: &str) -> String {
    let mut out = String::with_capacity(seg.len());
    for b in seg.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Default stable-ID suffix digit cap and fixed minimum (REQ-TRS-ID-005).
pub const ID_SUFFIX_DEFAULT_MAX: usize = 8;
pub const ID_SUFFIX_MIN: usize = 3;

/// Minimal view of `.syscribe.toml` for the path settings (matchers are loaded
/// separately by [`MatcherConfig`]). Unknown keys/tables are ignored.
#[derive(Debug, Default, Deserialize)]
struct PathsToml {
    #[serde(default, alias = "repoRoot")]
    repo_root: Option<String>,
    #[serde(default)]
    ids: IdsToml,
}

/// The `[ids]` table of `.syscribe.toml`.
#[derive(Debug, Default, Deserialize)]
struct IdsToml {
    #[serde(default, alias = "maxDigits")]
    max_digits: Option<usize>,
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
    /// REQ-TRS-MG-* — when `true`, `validate --profile <name>` runs the gated
    /// MagicGrid validation pass (sets [`ValidateConfig::magicgrid`]).
    #[serde(default)]
    pub magicgrid: bool,
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
        let id_max_digits = resolve_id_max_digits(&root);
        let links = load_links(&root);
        Self {
            model_root: Some(root),
            repo_root,
            matchers,
            results,
            // Remote fetching is opt-in (CLI `--fetch-remote`); never enabled here.
            remote_hook: None,
            id_max_digits,
            magicgrid: false,
            links,
        }
    }

    /// REQ-TRS-LINK-001 — resolve a file-backed element's hosted source URL.
    ///
    /// `file_path` is the element's on-disk path as recorded on the `RawElement`
    /// (`-m`-relative, e.g. `UAV/Avionics/FlightController.md`). Returns `None`
    /// when `[links]` is not configured, when the path is not under the model
    /// root, or when there is no model root. A package resolves to its
    /// `_index.md` only because that is the file recorded for it; YAML-key
    /// attributes have no file and never reach this method.
    pub fn hosted_url(&self, file_path: &str) -> Option<String> {
        self.hosted_url_for(file_path, "", "")
    }

    /// Like [`Self::hosted_url`] but supplies `{qname}`/`{id}` for the template.
    pub fn hosted_url_for(&self, file_path: &str, qname: &str, id: &str) -> Option<String> {
        let links = self.links.as_ref()?;
        let rel = self.model_relative(file_path)?;
        Some(links.resolve(&rel, qname, id))
    }

    /// Express `file_path` relative to the model root, forward-slashed. The path
    /// may already be relative (the common `RawElement::file_path` form) or
    /// absolute under the model root.
    fn model_relative(&self, file_path: &str) -> Option<String> {
        let p = Path::new(file_path);
        let root = self.model_root.as_ref()?;
        // The recorded `file_path` is `<model_root-as-passed>/<rel>` (walker uses
        // `path.display()`), so the model root is its prefix verbatim. Fall back
        // to stripping just the root's final component for hand-built paths.
        let rel: PathBuf = if let Ok(stripped) = p.strip_prefix(root) {
            stripped.to_path_buf()
        } else if let Some(name) = root.file_name() {
            p.strip_prefix(name).map(|s| s.to_path_buf()).unwrap_or_else(|_| p.to_path_buf())
        } else {
            p.to_path_buf()
        };
        // Forward-slash, dropping any `.` components.
        let parts: Vec<String> = rel
            .components()
            .filter_map(|c| match c {
                std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("/"))
        }
    }

    /// The effective stable-ID suffix digit cap: the configured `[ids] max_digits`
    /// (clamped to the fixed minimum of 3), or the default of 8 when unset
    /// (REQ-TRS-ID-005).
    pub fn id_digit_max(&self) -> usize {
        self.id_max_digits.unwrap_or(ID_SUFFIX_DEFAULT_MAX).max(ID_SUFFIX_MIN)
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

/// Read `[ids] max_digits` from `<model_root>/.syscribe.toml` (REQ-TRS-ID-005).
/// `None` when unset; the caller applies the default of 8 and the minimum of 3.
fn resolve_id_max_digits(model_root: &Path) -> Option<usize> {
    let text = std::fs::read_to_string(model_root.join(".syscribe.toml")).ok()?;
    toml::from_str::<PathsToml>(&text).ok()?.ids.max_digits
}

/// Load the `[links]` table from `<model_root>/.syscribe.toml` (REQ-TRS-LINK-001).
/// Returns `Some` only when at least one of `base_url`/`url_template` is set;
/// `None` (the feature inert) otherwise, including when the file is absent.
fn load_links(model_root: &Path) -> Option<LinkConfig> {
    let text = std::fs::read_to_string(model_root.join(".syscribe.toml")).ok()?;
    let links = toml::from_str::<LinksRootToml>(&text).ok()?.links;
    if links.base_url.is_none() && links.url_template.is_none() {
        return None;
    }
    Some(LinkConfig {
        base_url: links.base_url,
        url_template: links.url_template,
        git_ref: links.git_ref.unwrap_or_default(),
    })
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

    fn links_cfg(base: Option<&str>, tpl: Option<&str>, git_ref: &str) -> ValidateConfig {
        ValidateConfig {
            model_root: Some(PathBuf::from("model")),
            links: Some(LinkConfig {
                base_url: base.map(|s| s.to_string()),
                url_template: tpl.map(|s| s.to_string()),
                git_ref: git_ref.to_string(),
            }),
            ..ValidateConfig::default()
        }
    }

    #[test]
    fn base_url_appends_relative_path() {
        let c = links_cfg(Some("https://h/x/blob/main/model"), None, "");
        assert_eq!(
            c.hosted_url("model/UAV/Avionics/FlightController.md").as_deref(),
            Some("https://h/x/blob/main/model/UAV/Avionics/FlightController.md")
        );
    }

    #[test]
    fn base_url_trims_one_trailing_slash() {
        let c = links_cfg(Some("https://h/x/blob/main/model/"), None, "");
        assert_eq!(
            c.hosted_url("model/A.md").as_deref(),
            Some("https://h/x/blob/main/model/A.md")
        );
    }

    #[test]
    fn template_substitutes_placeholders_and_encodes_path() {
        let c = links_cfg(
            None,
            Some("https://h/x/blob/{ref}/model/{path}?q={qname}&i={id}"),
            "main",
        );
        let url = c
            .hosted_url_for("model/UAV/Flight Controller.md", "UAV::FC", "REQ-1")
            .unwrap();
        assert_eq!(
            url,
            "https://h/x/blob/main/model/UAV/Flight%20Controller.md?q=UAV::FC&i=REQ-1"
        );
    }

    #[test]
    fn no_links_config_yields_no_url() {
        let c = ValidateConfig {
            model_root: Some(PathBuf::from("model")),
            ..ValidateConfig::default()
        };
        assert_eq!(c.hosted_url("model/A.md"), None);
    }

    #[test]
    fn slash_separators_preserved_segments_encoded() {
        let c = links_cfg(Some("https://h"), None, "");
        assert_eq!(
            c.hosted_url("model/Sub Dir/My File.md").as_deref(),
            Some("https://h/Sub%20Dir/My%20File.md")
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

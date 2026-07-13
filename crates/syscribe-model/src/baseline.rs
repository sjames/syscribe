//! Release-baseline engine (ADR-SYS-BASELINE-001; REQ-TRS-BL-*).
//!
//! A `Baseline` element freezes a scope of the model into a git-anchored,
//! content-hashed release. This module owns the scope resolution, the full-content
//! aggregate seal (REQ-TRS-BL-002/003), the manifest schema (REQ-TRS-BL-004), the
//! git anchoring, and the validator drift/freeze pass (REQ-TRS-BL-005). The CLI
//! command family (`crates/syscribe/src/baseline.rs`) drives it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::element::{BaselineSeal, ElementType, FrozenScope, RawElement, RawFrontmatter};
use crate::resolver::Resolver;
use crate::suspect::content_hash;

/// Manifest schema version (bump on a breaking manifest-format change).
pub const MANIFEST_SCHEMA_VERSION: &str = "1.0";

/// The serialized variant name of an element type (`"Requirement"`, `"TestCase"`, …).
/// A fieldless serde enum serializes to a JSON string of its variant name.
pub fn element_type_name(et: &ElementType) -> String {
    serde_json::to_value(et)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "Unknown".to_string())
}

/// The stable key that identifies an element in a baseline: its `id` when present,
/// else its qualified name (REQ-TRS-BL-002/006).
pub fn element_key(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

// ── Scope resolution (REQ-TRS-BL-003) ────────────────────────────────────────

/// Resolve a `FrozenScope` to the in-scope elements, sorted by stable key.
/// `Baseline` elements are always excluded (REQ-TRS-BL-002), so sealing one
/// baseline never perturbs another. Filters compose as a logical AND; an absent
/// `package` means the whole model.
pub fn resolve_scope<'a>(elements: &'a [RawElement], scope: &FrozenScope) -> Vec<&'a RawElement> {
    let mut out: Vec<&RawElement> = elements
        .iter()
        .filter(|e| {
            let fm = &e.frontmatter;
            // Never freeze Baseline elements themselves.
            if matches!(fm.element_type, Some(ElementType::Baseline)) {
                return false;
            }
            // A package/namespace node with no own element (empty qname root) is skipped.
            if e.qualified_name.is_empty() {
                return false;
            }
            if let Some(pkg) = &scope.package {
                let prefix = format!("{pkg}::");
                if e.qualified_name != *pkg && !e.qualified_name.starts_with(&prefix) {
                    return false;
                }
            }
            if let Some(types) = &scope.types {
                let ty = fm.element_type.as_ref().map(element_type_name);
                if !ty.is_some_and(|t| types.contains(&t)) {
                    return false;
                }
            }
            if let Some(statuses) = &scope.status {
                if !fm.status.as_ref().is_some_and(|s| statuses.contains(s)) {
                    return false;
                }
            }
            if let Some(tags) = &scope.tags {
                let elem_tags = fm.tags.clone().unwrap_or_default();
                if !tags.iter().any(|t| elem_tags.contains(t)) {
                    return false;
                }
            }
            true
        })
        .collect();
    out.sort_by_key(|e| element_key(e));
    out
}

/// The in-scope elements as an owned vector, projecting to the variant when
/// `scope.config` is set (REQ-TRS-BL-011). `Err` when a `config` is given but does
/// not resolve to a Configuration or feature set (and a feature model exists).
pub fn resolve_in_scope(
    elements: &[RawElement],
    scope: &FrozenScope,
) -> Result<Vec<RawElement>, String> {
    use crate::projection::{project, resolve_selection, SelectionOutcome};
    let filtered_owned = |base: &[RawElement]| -> Vec<RawElement> {
        resolve_scope(base, scope).into_iter().cloned().collect()
    };
    match &scope.config {
        Some(cfg) => match resolve_selection(elements, cfg) {
            SelectionOutcome::Resolved(sel) => Ok(filtered_owned(&project(elements, &sel))),
            SelectionOutcome::Dormant => Ok(filtered_owned(elements)), // no feature model → inert
            SelectionOutcome::Error(m) => Err(m),
        },
        None => Ok(filtered_owned(elements)),
    }
}

/// Recompute the in-scope aggregate for a scope, projecting to the variant when
/// `scope.config` is set. Borrow-friendly: no whole-model clone in the common
/// no-config path. An unresolvable/absent config falls back to the flat scope.
pub fn aggregate_for_scope(elements: &[RawElement], scope: &FrozenScope) -> (String, usize) {
    use crate::projection::{project, resolve_selection, SelectionOutcome};
    if let Some(cfg) = &scope.config {
        if let SelectionOutcome::Resolved(sel) = resolve_selection(elements, cfg) {
            let projected = project(elements, &sel);
            return aggregate(&resolve_scope(&projected, scope));
        }
    }
    aggregate(&resolve_scope(elements, scope))
}

// ── Aggregate seal (REQ-TRS-BL-002) ──────────────────────────────────────────

/// Compute the deterministic aggregate content hash over the in-scope elements and
/// their per-element full-content hashes. Independent of file/enumeration order.
/// Returns `(blake3:<hex>, element_count)`.
pub fn aggregate(in_scope: &[&RawElement]) -> (String, usize) {
    let mut entries: Vec<(String, String)> =
        in_scope.iter().map(|e| (element_key(e), content_hash(e))).collect();
    entries.sort();
    let mut hasher = blake3::Hasher::new();
    for (key, hash) in &entries {
        hasher.update(key.as_bytes());
        hasher.update(b"\x1f"); // unit separator
        hasher.update(hash.as_bytes());
        hasher.update(b"\x1e"); // record separator
    }
    (format!("blake3:{}", hasher.finalize().to_hex()), entries.len())
}

// ── Manifest (REQ-TRS-BL-004) ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestElement {
    pub id: Option<String>,
    pub qname: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub file: String,
    pub status: Option<String>,
    pub hash: String,
}

/// The at-seal-time readiness snapshot (REQ-TRS-BL-004): validation counts and the
/// in-scope element counts by type. (Requirement-coverage ratios deferred.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Readiness {
    pub errors: usize,
    pub warnings: usize,
    #[serde(rename = "elementCountsByType")]
    pub element_counts_by_type: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: String,
    pub tool_version: String,
    pub baseline: String,
    pub name: Option<String>,
    pub date: Option<String>,
    pub approver: Option<String>,
    pub git_tag: Option<String>,
    pub git_commit: Option<String>,
    pub frozen_scope: FrozenScope,
    pub aggregate_hash: String,
    pub element_count: usize,
    pub readiness: Readiness,
    pub elements: Vec<ManifestElement>,
}

impl Manifest {
    /// Build a manifest for a resolved scope.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        id: &str,
        name: Option<&str>,
        date: Option<&str>,
        approver: Option<&str>,
        git_tag: Option<&str>,
        git_commit: Option<&str>,
        scope: &FrozenScope,
        in_scope: &[&RawElement],
        errors: usize,
        warnings: usize,
    ) -> Self {
        let (aggregate_hash, element_count) = aggregate(in_scope);
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        let elements: Vec<ManifestElement> = in_scope
            .iter()
            .map(|e| {
                let type_name = e
                    .frontmatter
                    .element_type
                    .as_ref()
                    .map(element_type_name)
                    .unwrap_or_else(|| "Unknown".to_string());
                *counts.entry(type_name.clone()).or_insert(0) += 1;
                ManifestElement {
                    id: e.frontmatter.id.clone(),
                    qname: e.qualified_name.clone(),
                    type_name,
                    file: e.file_path.clone(),
                    status: e.frontmatter.status.clone(),
                    hash: content_hash(e),
                }
            })
            .collect();
        Manifest {
            schema_version: MANIFEST_SCHEMA_VERSION.to_string(),
            tool_version: format!("syscribe {}", env!("CARGO_PKG_VERSION")),
            baseline: id.to_string(),
            name: name.map(str::to_string),
            date: date.map(str::to_string),
            approver: approver.map(str::to_string),
            git_tag: git_tag.map(str::to_string),
            git_commit: git_commit.map(str::to_string),
            frozen_scope: scope.clone(),
            aggregate_hash,
            element_count,
            readiness: Readiness { errors, warnings, element_counts_by_type: counts },
            elements,
        }
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_file(path: &Path) -> Option<Manifest> {
        let text = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&text).ok()
    }
}

/// Resolve the on-disk manifest path for a baseline: `<git-root>/<seal.manifest>`,
/// falling back to interpreting `seal.manifest` relative to the model root's parent.
pub fn manifest_path(model_root: &Path, seal: &BaselineSeal) -> PathBuf {
    let rel = Path::new(&seal.manifest);
    if rel.is_absolute() {
        return rel.to_path_buf();
    }
    if let Some(root) = crate::config::detect_git_root(model_root) {
        let p = root.join(rel);
        if p.exists() {
            return p;
        }
    }
    // Fallback: relative to the directory that contains the model root.
    model_root.parent().unwrap_or(model_root).join(rel)
}

// ── Drift / freeze validation pass (REQ-TRS-BL-005) ──────────────────────────

/// A validator finding as `(code, file, message)`.
type Finding = (&'static str, String, String);

/// Scan every `Baseline` element for drift, seal tampering, and unresolved
/// supersession. Severity is status-graded; the caller maps `E*`→Error, `W*`→Warning.
pub fn scan(elements: &[RawElement], resolver: &Resolver, model_root: &Path) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();
    for e in elements {
        if !matches!(e.frontmatter.element_type, Some(ElementType::Baseline)) {
            continue;
        }
        let fm = &e.frontmatter;
        let id = element_key(e);

        // E522 — supersedes must resolve to an existing element.
        if let Some(sup) = &fm.supersedes {
            if resolver.resolve_ref(elements, sup).is_none() {
                out.push((
                    "E522",
                    e.file_path.clone(),
                    format!("baseline `{id}` supersedes `{sup}`, which resolves to no element"),
                ));
            }
        }

        let Some(seal) = &fm.seal else { continue };
        let status = fm.status.as_deref().unwrap_or("draft");

        // E521 — seal integrity: element seal must match the manifest's aggregate.
        if !seal.manifest.is_empty() {
            if let Some(m) = Manifest::from_file(&manifest_path(model_root, seal)) {
                if m.aggregate_hash != seal.aggregate_hash {
                    out.push((
                        "E521",
                        e.file_path.clone(),
                        format!(
                            "baseline `{id}` seal.aggregateHash disagrees with its manifest \
                             (seal tampered or manifest stale)"
                        ),
                    ));
                }
            }
        }

        // Drift — recompute the in-scope aggregate and compare to the seal.
        // `superseded` baselines are historical and not drift-checked.
        if status == "superseded" {
            continue;
        }
        let scope = fm.frozen_scope.clone().unwrap_or_default();
        let (current, _n) = aggregate_for_scope(elements, &scope);
        if current != seal.aggregate_hash {
            let (code, sev): (&'static str, &str) = match status {
                "released" => ("E520", "released"),
                "approved" => ("W520", "approved"),
                _ => continue, // draft (or unknown) — silent
            };
            out.push((
                code,
                e.file_path.clone(),
                format!(
                    "baseline `{id}` ({sev}) has drifted: in-scope content no longer matches its \
                     seal — re-seal via a superseding baseline, or revert the change"
                ),
            ));
        }
    }
    out
}

/// Convenience: does this frontmatter describe a `Baseline`?
pub fn is_baseline(fm: &RawFrontmatter) -> bool {
    matches!(fm.element_type, Some(ElementType::Baseline))
}

// ── Shared diff / verify (used by both the CLI and MCP, REQ-TRS-BL-006/008) ──

/// Element-level difference between two baseline manifests, keyed by stable id
/// (falling back to qualified name so it survives file moves).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineDiff {
    pub from: String,
    pub to: String,
    pub aggregate_identical: bool,
    pub added: Vec<ManifestElement>,
    pub removed: Vec<ManifestElement>,
    /// The `to`-side of each changed element (its `hash` differs from the `from` side).
    pub changed: Vec<ManifestElement>,
}

fn manifest_key(e: &ManifestElement) -> String {
    e.id.clone().unwrap_or_else(|| e.qname.clone())
}

/// Compare two manifests into added / removed / changed sets.
pub fn diff_manifests(a: &Manifest, b: &Manifest) -> BaselineDiff {
    let ma: BTreeMap<String, &ManifestElement> =
        a.elements.iter().map(|e| (manifest_key(e), e)).collect();
    let mb: BTreeMap<String, &ManifestElement> =
        b.elements.iter().map(|e| (manifest_key(e), e)).collect();
    let mut added = Vec::new();
    let mut changed = Vec::new();
    for (k, eb) in &mb {
        match ma.get(k) {
            None => added.push((*eb).clone()),
            Some(ea) if ea.hash != eb.hash => changed.push((*eb).clone()),
            Some(_) => {}
        }
    }
    let removed: Vec<ManifestElement> =
        ma.iter().filter(|(k, _)| !mb.contains_key(*k)).map(|(_, e)| (*e).clone()).collect();
    BaselineDiff {
        from: a.baseline.clone(),
        to: b.baseline.clone(),
        aggregate_identical: a.aggregate_hash == b.aggregate_hash,
        added,
        removed,
        changed,
    }
}

/// The outcome of verifying one baseline (REQ-TRS-BL-008): the content proof and
/// git tag↔commit consistency, with a boolean `passed` and any failure messages.
#[derive(Debug, Clone, Serialize)]
pub struct VerifyResult {
    pub id: String,
    pub passed: bool,
    pub messages: Vec<String>,
}

/// Verify a single `Baseline` element against the current model and git state.
pub fn verify_baseline(elements: &[RawElement], b: &RawElement, model_root: &Path) -> VerifyResult {
    let id = element_key(b);
    let fm = &b.frontmatter;
    let mut messages = Vec::new();
    let Some(seal) = &fm.seal else {
        return VerifyResult { id, passed: false, messages: vec!["no seal".to_string()] };
    };
    let scope = fm.frozen_scope.clone().unwrap_or_default();
    let (current, _n) = aggregate_for_scope(elements, &scope);
    if current != seal.aggregate_hash {
        messages.push("content drift (recomputed aggregate ≠ seal)".to_string());
    }
    if let Some(m) = Manifest::from_file(&manifest_path(model_root, seal)) {
        if m.aggregate_hash != seal.aggregate_hash {
            messages.push("manifest aggregate ≠ seal".to_string());
        }
    }
    if let (Some(root), Some(tag), Some(commit)) = (
        crate::config::detect_git_root(model_root),
        fm.git_tag.as_deref(),
        fm.git_commit.as_deref(),
    ) {
        match crate::config::git_output(&root, &["rev-parse", &format!("{tag}^{{commit}}")]) {
            Some(tc) if tc == commit => {}
            Some(_) => messages.push(format!("gitTag `{tag}` resolves to a different commit than gitCommit")),
            None => {} // tag not present yet — informational, not a failure
        }
    }
    VerifyResult { id, passed: messages.is_empty(), messages }
}

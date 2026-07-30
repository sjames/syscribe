//! Guarded-write machinery shared by element-creation, update, move, and delete
//! flows: candidate validation via a throwaway temp copy of the model, a
//! baseline/candidate diff, and the commit gate.
//!
//! The commit gate is **referential integrity**: a write is refused if it would
//! leave a cross-reference (`supertype`, `typedBy`, `verifies`, …) that no longer
//! resolves. The full validator's *warnings* are surfaced in the delta's warning
//! channels for context, but only newly-unresolved references gate a commit — so
//! e.g. creating a not-yet-fleshed-out draft requirement is allowed, while
//! pointing a `supertype:` at a non-existent element is refused.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::ValidateConfig;
use crate::element::RawElement;
use crate::resolver::{is_builtin_type, Resolver};
use crate::validator::{validate_with_config, Severity};

use super::diff::tree_unified_diff;

/// A normalised finding: `(code, model-root-relative file, message)`.
pub type Entry = (String, String, String);

/// Failure confining a write under the model root (defeats `..`/symlink traversal).
#[derive(Debug, thiserror::Error)]
pub enum WriteConfinedError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("resolved path escapes the model root")]
    Escapes,
}

/// Write `content` to `<root>/<rel>`, confirming the resolved parent stays within
/// the canonicalized model root (defeats `..`/symlink traversal).
pub fn write_confined(root: &Path, rel: &str, content: &str) -> Result<(), WriteConfinedError> {
    let target = root.join(rel);
    let canon_root = std::fs::canonicalize(root)?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
        let canon_parent = std::fs::canonicalize(parent)?;
        if !canon_parent.starts_with(&canon_root) {
            return Err(WriteConfinedError::Escapes);
        }
    }
    std::fs::write(&target, content)?;
    Ok(())
}

/// Recursively copy `src` into `dst` (creating `dst`).
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Make a throwaway copy of the model tree; returns the copy's root.
fn make_temp_copy(model_root: &Path) -> std::io::Result<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let root = std::env::temp_dir().join(format!(
        "syscribe-mcp-cand-{}-{}",
        std::process::id(),
        nanos
    ));
    copy_dir_all(model_root, &root)?;
    Ok(root)
}

/// Collect the qualified-name strings from a `supertype`/`typedBy`/`redefines`
/// field that may be a YAML string or sequence of strings.
fn yaml_strings(v: &serde_yaml::Value) -> Vec<String> {
    match v {
        serde_yaml::Value::String(s) => vec![s.clone()],
        serde_yaml::Value::Sequence(seq) => {
            seq.iter().filter_map(|x| x.as_str().map(String::from)).collect()
        }
        _ => Vec::new(),
    }
}

/// Every cross-reference string an element holds, paired with its field name.
pub fn element_ref_strings(e: &RawElement) -> Vec<(&'static str, String)> {
    let fm = &e.frontmatter;
    let mut refs: Vec<(&'static str, String)> = Vec::new();
    for (field, val) in [
        ("supertype", &fm.supertype),
        ("typedBy", &fm.typed_by),
        ("redefines", &fm.redefines),
    ] {
        if let Some(v) = val {
            for s in yaml_strings(v) {
                refs.push((field, s));
            }
        }
    }
    for (field, list) in [
        ("subsets", &fm.subsets),
        ("verifies", &fm.verifies),
        ("derivedFrom", &fm.derived_from),
        ("satisfies", &fm.satisfies),
        ("allocatedFrom", &fm.allocated_from),
        ("allocatedTo", &fm.allocated_to),
    ] {
        if let Some(l) = list {
            for s in l {
                refs.push((field, s.clone()));
            }
        }
    }
    refs
}

/// Elements (other than the target) that hold a cross-reference resolving to
/// `target_qname` — used by `delete_element`'s reference-impact guard. Returns
/// `(qname, id)` of each distinct referrer.
pub fn referrers(elements: &[RawElement], target_qname: &str) -> Vec<(String, Option<String>)> {
    let resolver = Resolver::new(elements);
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    for e in elements {
        if e.qualified_name == target_qname {
            continue;
        }
        let hits = element_ref_strings(e).into_iter().any(|(_, r)| {
            resolver
                .resolve_ref(elements, &r)
                .is_some_and(|t| t.qualified_name == target_qname)
        });
        if hits {
            out.push((e.qualified_name.clone(), e.frontmatter.id.clone()));
        }
    }
    out
}

/// Every cross-reference in the model that does not resolve, as error entries.
/// Built-in standard-library type references (`ScalarValues::Real`, …) are exempt.
pub fn ref_errors(elements: &[RawElement], root: &Path) -> Vec<Entry> {
    let resolver = Resolver::new(elements);
    let mut out = Vec::new();
    for e in elements {
        for (field, r) in element_ref_strings(e) {
            if is_builtin_type(&r) {
                continue;
            }
            if resolver.resolve_ref(elements, &r).is_none() {
                out.push((
                    "EREF".to_string(),
                    rel_file(&e.file_path, root),
                    format!("`{field}` reference '{r}' does not resolve to any model element"),
                ));
            }
        }
    }
    out
}

/// The full validator's *warning*-severity findings, normalised.
pub fn validator_warnings(elements: &[RawElement], config: &ValidateConfig, root: &Path) -> Vec<Entry> {
    validate_with_config(elements, config)
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Warning))
        .map(|f| (f.code.to_string(), rel_file(&f.file, root), f.message.clone()))
        .collect()
}

/// Normalise an absolute file path to a model-root-relative path, so findings
/// produced against a temp copy and against the real model compare equal.
fn rel_file(file: &str, root: &Path) -> String {
    let root_s = root.to_string_lossy();
    file.strip_prefix(root_s.as_ref())
        .map(|s| s.trim_start_matches(['/', '\\']).to_string())
        .unwrap_or_else(|| file.to_string())
}

/// `candidate \ baseline`, as entries.
fn added(candidate: &[Entry], baseline: &HashSet<Entry>) -> Vec<Entry> {
    candidate.iter().filter(|e| !baseline.contains(*e)).cloned().collect()
}

/// The result of a guarded write: whether it committed, the before/after
/// validation delta, a unified diff preview, and a refusal/failure reason (if any).
#[derive(Debug, Default, Clone)]
pub struct GuardedWriteOutcome {
    pub written: bool,
    pub new_errors: Vec<Entry>,
    pub resolved_errors: Vec<Entry>,
    pub new_warnings: Vec<Entry>,
    pub resolved_warnings: Vec<Entry>,
    pub diff: String,
    pub reason: Option<String>,
}

impl GuardedWriteOutcome {
    fn refused(reason: impl Into<String>) -> Self {
        Self {
            reason: Some(reason.into()),
            ..Default::default()
        }
    }
}

/// Run a guarded write. `apply` performs the edit against an arbitrary model root
/// (invoked once on a temp copy to compute the candidate, and a second time on the
/// real model only when committing a clean change).
///
/// On `dry_run` (the default) disk is never touched. On commit, when `gate` is
/// true a change that introduces a newly-unresolved cross-reference is refused
/// unless `allow_new_errors` is set (callers decide how that escape hatch is
/// sourced — e.g. an env var — this function only consumes the decision).
/// A delete-style caller passes `gate=false` because its own reference-impact
/// guard already governs safety.
///
/// This function never reloads or mutates any caller-side model cache — on a
/// successful commit the caller is responsible for re-reading the model from
/// `model_root` to refresh its own derived state.
pub fn guarded_write<F>(
    model_root: &Path,
    elements: &[RawElement],
    config: &ValidateConfig,
    dry_run: bool,
    gate: bool,
    allow_new_errors: bool,
    apply: F,
) -> GuardedWriteOutcome
where
    F: Fn(&Path) -> Result<(), String>,
{
    let base_errs = ref_errors(elements, model_root);
    let base_warns = validator_warnings(elements, config, model_root);

    let cand_root = match make_temp_copy(model_root) {
        Ok(p) => p,
        Err(e) => return GuardedWriteOutcome::refused(format!("could not stage candidate: {e}")),
    };

    // Apply the edit to the candidate copy. A failure here (invalid dest, planning
    // error, …) is a refusal — the real model is never touched.
    if let Err(e) = apply(&cand_root) {
        let _ = std::fs::remove_dir_all(&cand_root);
        return GuardedWriteOutcome::refused(e);
    }

    let (cand_errs, cand_warns) = match crate::walker::walk_model(&cand_root) {
        Ok(elems) => {
            let cfg = ValidateConfig::with_model_root(&cand_root);
            let errs = ref_errors(&elems, &cand_root);
            let warns = validator_warnings(&elems, &cfg, &cand_root);
            (errs, warns)
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&cand_root);
            return GuardedWriteOutcome::refused(format!("candidate model failed to load: {e}"));
        }
    };

    // Unified diff of the would-be change (real tree vs candidate tree).
    let diff = tree_unified_diff(model_root, &cand_root);
    let _ = std::fs::remove_dir_all(&cand_root);

    let base_err_set: HashSet<Entry> = base_errs.iter().cloned().collect();
    let cand_err_set: HashSet<Entry> = cand_errs.iter().cloned().collect();
    let base_warn_set: HashSet<Entry> = base_warns.iter().cloned().collect();
    let cand_warn_set: HashSet<Entry> = cand_warns.iter().cloned().collect();

    let new_errors = added(&cand_errs, &base_err_set);
    let resolved_errors = added(&base_errs, &cand_err_set);
    let new_warnings = added(&cand_warns, &base_warn_set);
    let resolved_warnings = added(&base_warns, &cand_warn_set);
    let new_error_count = new_errors.len();

    let mut outcome = GuardedWriteOutcome {
        written: false,
        new_errors,
        resolved_errors,
        new_warnings,
        resolved_warnings,
        diff,
        reason: None,
    };

    if dry_run {
        return outcome;
    }

    if gate && new_error_count > 0 && !allow_new_errors {
        outcome.reason = Some("refused: commit would introduce an unresolved reference".to_string());
        return outcome;
    }

    // Commit: apply to the real model.
    if let Err(e) = apply(model_root) {
        outcome.reason = Some(format!("commit failed: {e}"));
        return outcome;
    }
    outcome.written = true;
    outcome
}

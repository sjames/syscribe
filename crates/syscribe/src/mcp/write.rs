//! Guarded-write machinery shared by `create_element`, `update_element`, and
//! `move_element`: candidate validation via a throwaway temp copy of the model,
//! a baseline/candidate diff, and the commit gate.
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

use serde_json::{json, Map, Value};
use syscribe_model::config::ValidateConfig;
use syscribe_model::element::RawElement;
use syscribe_model::resolver::{is_builtin_type, Resolver};
use syscribe_model::validator::{validate_with_config, Severity};

use super::store::McpStore;
use super::util::rel_file;

/// A normalised finding: `(code, model-root-relative file, message)`.
type Entry = (String, String, String);

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

/// Every cross-reference in the model that does not resolve, as error entries.
/// Built-in standard-library type references (`ScalarValues::Real`, …) are exempt.
fn ref_errors(elements: &[RawElement], root: &Path) -> Vec<Entry> {
    let resolver = Resolver::new(elements);
    let mut out = Vec::new();
    for e in elements {
        let fm = &e.frontmatter;
        let mut refs: Vec<(&str, String)> = Vec::new();
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
        for (field, r) in refs {
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
fn validator_warnings(elements: &[RawElement], config: &ValidateConfig, root: &Path) -> Vec<Entry> {
    validate_with_config(elements, config)
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Warning))
        .map(|f| (f.code.to_string(), rel_file(&f.file, root), f.message.clone()))
        .collect()
}

fn entry_json(e: &Entry, severity: &str) -> Value {
    json!({ "code": e.0, "severity": severity, "file": e.1, "message": e.2 })
}

/// `candidate \ baseline`, as JSON entries of the given severity.
fn added(candidate: &[Entry], baseline: &HashSet<Entry>, severity: &str) -> Vec<Value> {
    candidate
        .iter()
        .filter(|e| !baseline.contains(*e))
        .map(|e| entry_json(e, severity))
        .collect()
}

fn empty_delta() -> Value {
    json!({
        "newErrors": [],
        "resolvedErrors": [],
        "newWarnings": [],
        "resolvedWarnings": [],
    })
}

/// Assemble a result object from tool-specific `extra` fields plus the standard
/// `written` / `validationDelta` (and an optional `reason`).
fn result(extra: &Map<String, Value>, written: bool, delta: Value, reason: Option<&str>) -> Value {
    let mut obj = extra.clone();
    obj.insert("written".into(), Value::Bool(written));
    obj.insert("validationDelta".into(), delta);
    if let Some(r) = reason {
        obj.insert("reason".into(), Value::String(r.to_string()));
    }
    Value::Object(obj)
}

/// A guard refusal that never touched disk and computed no delta (e.g. an invalid
/// or traversal qname caught before any candidate work).
pub fn refuse(extra: Map<String, Value>, reason: &str) -> Value {
    result(&extra, false, empty_delta(), Some(reason))
}

/// Run a guarded write. `apply` performs the edit against an arbitrary model root
/// (invoked once on a temp copy to compute the candidate, and a second time on the
/// real model only when committing a clean change).
///
/// On `dry_run` (the default) disk is never touched. On commit, a change that
/// introduces a newly-unresolved cross-reference is refused unless
/// `SYSCRIBE_MCP_ALLOW_NEW_ERRORS=1`.
pub fn guarded_write<F>(
    store: &mut McpStore,
    dry_run: bool,
    extra: Map<String, Value>,
    apply: F,
) -> Value
where
    F: Fn(&Path) -> Result<(), String>,
{
    let base_root = store.model_root.clone();
    let base_errs = ref_errors(&store.elements, &base_root);
    let base_warns = validator_warnings(&store.elements, &store.config, &base_root);

    let cand_root = match make_temp_copy(&base_root) {
        Ok(p) => p,
        Err(e) => return refuse(extra, &format!("could not stage candidate: {e}")),
    };

    // Apply the edit to the candidate copy. A failure here (invalid dest, planning
    // error, …) is a refusal — the real model is never touched.
    if let Err(e) = apply(&cand_root) {
        let _ = std::fs::remove_dir_all(&cand_root);
        return refuse(extra, &e);
    }

    let (cand_errs, cand_warns) = match syscribe_model::walker::walk_model(&cand_root) {
        Ok(elems) => {
            let cfg = ValidateConfig::with_model_root(&cand_root);
            let errs = ref_errors(&elems, &cand_root);
            let warns = validator_warnings(&elems, &cfg, &cand_root);
            (errs, warns)
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&cand_root);
            return refuse(extra, &format!("candidate model failed to load: {e}"));
        }
    };
    let _ = std::fs::remove_dir_all(&cand_root);

    let base_err_set: HashSet<Entry> = base_errs.iter().cloned().collect();
    let cand_err_set: HashSet<Entry> = cand_errs.iter().cloned().collect();
    let base_warn_set: HashSet<Entry> = base_warns.iter().cloned().collect();
    let cand_warn_set: HashSet<Entry> = cand_warns.iter().cloned().collect();

    let new_errors = added(&cand_errs, &base_err_set, "error");
    let resolved_errors = added(&base_errs, &cand_err_set, "error");
    let new_warnings = added(&cand_warns, &base_warn_set, "warning");
    let resolved_warnings = added(&base_warns, &cand_warn_set, "warning");
    let new_error_count = new_errors.len();

    let delta = json!({
        "newErrors": new_errors,
        "resolvedErrors": resolved_errors,
        "newWarnings": new_warnings,
        "resolvedWarnings": resolved_warnings,
    });

    if dry_run {
        return result(&extra, false, delta, None);
    }

    let allow_new_errors = std::env::var("SYSCRIBE_MCP_ALLOW_NEW_ERRORS")
        .map(|v| v == "1")
        .unwrap_or(false);
    if new_error_count > 0 && !allow_new_errors {
        return result(
            &extra,
            false,
            delta,
            Some("refused: commit would introduce an unresolved reference"),
        );
    }

    // Commit: apply to the real model, then rebuild the store.
    if let Err(e) = apply(&base_root) {
        return result(&extra, false, delta, Some(&format!("commit failed: {e}")));
    }
    if let Err(e) = store.reload() {
        return result(&extra, true, delta, Some(&format!("written, but reload failed: {e}")));
    }
    result(&extra, true, delta, None)
}

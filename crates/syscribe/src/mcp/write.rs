//! Thin MCP-side shim over the shared guarded-write engine in
//! `syscribe_model::mutate`: reshapes the model-layer `GuardedWriteOutcome` into
//! the MCP tool response JSON (`written` / `validationDelta` / `diff` / `reason`),
//! reloads the MCP store on a successful commit, and is the one place that reads
//! `SYSCRIBE_MCP_ALLOW_NEW_ERRORS` — the shared engine only takes an explicit
//! `allow_new_errors: bool`, so this env var stays an MCP-specific escape hatch
//! rather than something a diagram-editor (or any other) caller would silently
//! inherit.

use std::path::Path;

use serde_json::{json, Map, Value};
use syscribe_model::mutate::{guarded_write as model_guarded_write, Entry, GuardedWriteOutcome};

use super::store::McpStore;

/// Elements (other than the target) that hold a cross-reference resolving to a
/// given qualified name — used by `delete_element`'s reference-impact guard.
pub use syscribe_model::mutate::referrers;

fn entries_json(entries: &[Entry], severity: &str) -> Vec<Value> {
    entries
        .iter()
        .map(|e| json!({ "code": e.0, "severity": severity, "file": e.1, "message": e.2 }))
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

fn delta_json(outcome: &GuardedWriteOutcome) -> Value {
    json!({
        "newErrors": entries_json(&outcome.new_errors, "error"),
        "resolvedErrors": entries_json(&outcome.resolved_errors, "error"),
        "newWarnings": entries_json(&outcome.new_warnings, "warning"),
        "resolvedWarnings": entries_json(&outcome.resolved_warnings, "warning"),
    })
}

/// Assemble a result object from tool-specific `extra` fields plus the standard
/// `written` / `validationDelta` / `diff` (and an optional `reason`).
fn result(
    extra: &Map<String, Value>,
    written: bool,
    delta: Value,
    diff: &str,
    reason: Option<&str>,
) -> Value {
    let mut obj = extra.clone();
    obj.insert("written".into(), Value::Bool(written));
    obj.insert("validationDelta".into(), delta);
    obj.insert("diff".into(), Value::String(diff.to_string()));
    if let Some(r) = reason {
        obj.insert("reason".into(), Value::String(r.to_string()));
    }
    Value::Object(obj)
}

/// A guard refusal that never touched disk and computed no delta/diff (e.g. an
/// invalid or traversal qname, or a blocked delete, caught before candidate work).
pub fn refuse(extra: Map<String, Value>, reason: &str) -> Value {
    result(&extra, false, empty_delta(), "", Some(reason))
}

/// Run a guarded write against `store`'s live model. `apply` performs the edit
/// against an arbitrary model root (invoked once on a temp copy to compute the
/// candidate, and a second time on the real model only when committing a clean
/// change).
///
/// On `dry_run` (the default) disk is never touched. On commit, when `gate` is
/// true a change that introduces a newly-unresolved cross-reference is refused
/// (unless `SYSCRIBE_MCP_ALLOW_NEW_ERRORS=1`). `delete_element` passes `gate=false`
/// because its own reference-impact guard already governs safety.
pub fn guarded_write<F>(
    store: &mut McpStore,
    dry_run: bool,
    gate: bool,
    extra: Map<String, Value>,
    apply: F,
) -> Value
where
    F: Fn(&Path) -> Result<(), String>,
{
    let allow_new_errors = std::env::var("SYSCRIBE_MCP_ALLOW_NEW_ERRORS")
        .map(|v| v == "1")
        .unwrap_or(false);
    let outcome = model_guarded_write(
        &store.model_root,
        &store.elements,
        &store.config,
        dry_run,
        gate,
        allow_new_errors,
        apply,
    );
    let delta = delta_json(&outcome);
    if !outcome.written {
        return result(&extra, false, delta, &outcome.diff, outcome.reason.as_deref());
    }
    // Commit succeeded on disk; refresh the store's derived state (elements,
    // graph, resolver) from the now-updated model root.
    if let Err(e) = store.reload() {
        return result(
            &extra,
            true,
            delta,
            &outcome.diff,
            Some(&format!("written, but reload failed: {e}")),
        );
    }
    result(&extra, true, delta, &outcome.diff, None)
}

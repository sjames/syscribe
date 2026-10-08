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
use syscribe_model::mutate::{compute_baseline, guarded_write_cached, is_gating_error, Baseline, Entry, GuardedWriteOutcome};

use super::store::McpStore;

/// Elements (other than the target) that hold a cross-reference resolving to a
/// given qualified name — used by `delete_element`'s reference-impact guard.
pub use syscribe_model::mutate::referrers;

fn entries_json(entries: &[Entry], severity: &str) -> Vec<Value> {
    entries
        .iter()
        .map(|e| {
            let mut v = json!({ "code": e.0, "severity": severity, "file": e.1, "message": e.2 });
            // Errors say whether they can refuse a commit (GH #187): `true` for an unresolved
            // reference or link-type error, `false` for every other validator error.
            if severity == "error" {
                v["gating"] = Value::Bool(is_gating_error(&e.0));
            }
            v
        })
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
/// Run the model-layer guarded write against `store` using its kept baseline (computed
/// on first use), and, for a large model, with the live copy dropped while the candidate
/// is built. When nothing was committed the model is rebuilt (large) and the baseline
/// kept. Returns the outcome, whatever `inspect` produced, and, on a commit, the
/// candidate's findings.
#[allow(clippy::too_many_arguments)]
fn run_guarded<F, I>(
    store: &mut McpStore,
    dry_run: bool,
    gate: bool,
    allow_new_errors: bool,
    apply: F,
    inspect: Option<&dyn Fn(&[syscribe_model::element::RawElement]) -> I>,
    proceed: impl Fn(&Option<I>) -> bool,
) -> (GuardedWriteOutcome, Option<I>, Option<Baseline>)
where
    F: Fn(&Path) -> Result<(), String>,
{
    let baseline = match store.baseline.take() {
        Some(b) => b,
        None => compute_baseline(&store.model_root, &store.elements, &store.config),
    };
    let released = store.elements.len() >= super::store::RELEASE_ABOVE;
    if released {
        store.release_model();
    }
    let (outcome, inspected, candidate) =
        guarded_write_cached(&store.model_root, &baseline, &store.config, dry_run, gate, allow_new_errors, apply, inspect, proceed);
    if !outcome.written {
        if released {
            // Nothing changed on disk: put the model back as it was.
            if let Err(e) = store.reload() {
                tracing_warn(&format!("rebuilding the model after a refused write failed: {e}"));
            }
        }
        store.baseline = Some(baseline);
    }
    (outcome, inspected, candidate)
}

fn tracing_warn(msg: &str) {
    eprintln!("syscribe mcp: {msg}");
}

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
    let (outcome, _, candidate) = run_guarded(store, dry_run, gate, allow_new_errors, apply, None::<&dyn Fn(&[syscribe_model::element::RawElement]) -> ()>, |_| true);
    let delta = delta_json(&outcome);
    if !outcome.written {
        return result(&extra, false, delta, &outcome.diff, outcome.reason.as_deref());
    }
    // Commit succeeded on disk; refresh the store's derived state (elements,
    // graph, resolver) from the now-updated model root. The candidate's findings
    // are the new model's: keep them as the next baseline.
    let reloaded = store.reload();
    store.baseline = candidate;
    if let Err(e) = reloaded {
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

/// Apply one semantic feature-model edit (`REQ-TRS-FMED-004`) through the guarded
/// write, as `POST /api/feature-model/edit` does for the browser.
///
/// The result carries the standard `written`/`validationDelta`/`diff`, plus
/// `featureDelta` (what the edit does to the model's validity: features that become
/// dead or false-optional, a model that becomes void, configurations that become
/// invalid), `needsConfirmation` when it makes things worse and `accept_worse` was
/// not given, and, on a commit, `undo`: the operation that reverses it.
pub fn feature_edit(store: &mut McpStore, op: syscribe_model::feature_edit::EditOp, dry_run: bool, accept_worse: bool, extra: Map<String, Value>) -> Value {
    use std::sync::{Arc, Mutex};
    use syscribe_model::feature_edit::{analysis_delta, apply};
    use syscribe_model::feature_model::analysis_json;

    let allow_new_errors = std::env::var("SYSCRIBE_MCP_ALLOW_NEW_ERRORS").map(|v| v == "1").unwrap_or(false);
    let before = analysis_json(&store.elements);
    let undo: Arc<Mutex<Option<(syscribe_model::feature_edit::EditOp, Option<String>)>>> = Arc::new(Mutex::new(None));
    let sink = undo.clone();
    let apply_op = move |root: &Path| -> Result<(), String> {
        let out = apply(root, &op)?;
        *sink.lock().unwrap() = Some((out.undo, out.feature));
        Ok(())
    };
    let before_c = before.clone();
    let (outcome, after, candidate) = run_guarded(
        store,
        dry_run,
        true,
        allow_new_errors,
        apply_op,
        Some(&|elems: &[syscribe_model::element::RawElement]| analysis_json(elems)),
        |after: &Option<Value>| accept_worse || after.as_ref().map(|a| analysis_delta(&before_c, a)["worsens"] != true).unwrap_or(true),
    );
    let delta = delta_json(&outcome);
    let feature_delta = after.as_ref().map(|a| analysis_delta(&before, a));
    let worsens = feature_delta.as_ref().is_some_and(|d| d["worsens"] == true);
    let needs_confirmation = !dry_run && !accept_worse && !outcome.written && worsens && outcome.reason.is_none();
    let mut obj = extra;
    obj.insert("featureDelta".into(), feature_delta.unwrap_or(Value::Null));
    obj.insert("needsConfirmation".into(), Value::Bool(needs_confirmation));
    let (undo_op, feature) = match undo.lock().unwrap().clone() {
        Some((u, f)) => (serde_json::to_value(u).ok(), f),
        None => (None, None),
    };
    obj.insert("feature".into(), feature.map(Value::String).unwrap_or(Value::Null));
    let reason = if needs_confirmation { Some("this edit makes the feature model worse; resend with accept_worse:true to apply it".to_string()) } else { outcome.reason.clone() };
    if !outcome.written {
        return result(&obj, false, delta, &outcome.diff, reason.as_deref());
    }
    obj.insert("undo".into(), undo_op.unwrap_or(Value::Null));
    let reloaded = store.reload();
    store.baseline = candidate;
    if let Err(e) = reloaded {
        return result(&obj, true, delta, &outcome.diff, Some(&format!("written, but reload failed: {e}")));
    }
    result(&obj, true, delta, &outcome.diff, None)
}

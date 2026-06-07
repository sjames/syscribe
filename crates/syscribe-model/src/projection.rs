//! Configuration projection — the `--config` lens (REQ-TRS-PROJ-001..003).
//!
//! Projects the 150% model onto one configuration (the 100% model) by keeping
//! only the elements whose `appliesWhen` holds for that configuration's
//! selection. A projection is itself a valid model, so the lens is implemented
//! as *filter, then reuse* the existing validator/queries. Dormant (a no-op)
//! when the model has no `FeatureDef`.

use std::collections::{BTreeMap, HashSet};

use crate::config::ValidateConfig;
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::validator::{self, Finding, Severity};
use crate::variability;

/// A concrete feature selection: feature qualified name -> selected.
pub type Selection = BTreeMap<String, bool>;

fn err(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Error }
}
fn warn(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Warning }
}

fn has_feature_model(elements: &[RawElement]) -> bool {
    elements
        .iter()
        .any(|e| e.frontmatter.element_type.as_ref() == Some(&ElementType::FeatureDef))
}

/// Outcome of resolving a `--config` argument.
pub enum SelectionOutcome {
    /// No feature model present — the lens is inert (caller uses the full model).
    Dormant,
    /// A resolved concrete selection.
    Resolved(Selection),
    /// The argument did not resolve (with a message).
    Error(String),
}

/// Resolve a `--config` argument into a concrete selection. Accepts a stored
/// `Configuration` (by id or qualified name) or an ad-hoc comma-separated set of
/// `FeatureDef` qualified names (listed = selected, all others deselected).
pub fn resolve_selection(elements: &[RawElement], arg: &str) -> SelectionOutcome {
    if !has_feature_model(elements) {
        return SelectionOutcome::Dormant;
    }
    // 1. A stored Configuration (id or qualified name).
    if let Some(cfg) = elements.iter().find(|e| {
        e.frontmatter.element_type.as_ref() == Some(&ElementType::Configuration)
            && (e.frontmatter.id.as_deref() == Some(arg) || e.qualified_name == arg)
    }) {
        return SelectionOutcome::Resolved(cfg.frontmatter.feature_selections());
    }
    // 2. An ad-hoc feature set.
    let feature_defs: HashSet<&str> = elements
        .iter()
        .filter(|e| e.frontmatter.element_type.as_ref() == Some(&ElementType::FeatureDef))
        .map(|e| e.qualified_name.as_str())
        .collect();
    let tokens: Vec<&str> = arg.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if !tokens.is_empty() && tokens.iter().all(|t| feature_defs.contains(t)) {
        let mut sel = Selection::new();
        for t in tokens {
            sel.insert(t.to_string(), true);
        }
        return SelectionOutcome::Resolved(sel);
    }
    SelectionOutcome::Error(format!(
        "--config '{}' is neither a known Configuration nor a set of FeatureDef qualified names",
        arg
    ))
}

/// Is this element active under the given selection, honouring the *effective*
/// condition (its own `appliesWhen`, else the nearest ancestor package's —
/// [`REQ-TRS-VAR-006`])? An element with no effective condition is always active;
/// a malformed `appliesWhen` (an E209 elsewhere) is treated as active here.
pub fn is_active(
    elem: &RawElement,
    sel: &Selection,
    pkg: &std::collections::HashMap<String, serde_yaml::Value>,
) -> bool {
    match variability::effective_applies_when(elem, pkg) {
        None => true,
        Some((aw, _)) => match variability::applies_when_expr(&aw) {
            Ok(Some(expr)) => expr.eval(&|q: &str| sel.get(q).copied().unwrap_or(false)),
            _ => true,
        },
    }
}

/// The projected (active) element set for a selection.
pub fn project(elements: &[RawElement], sel: &Selection) -> Vec<RawElement> {
    let pkg = variability::package_conditions(elements);
    elements.iter().filter(|e| is_active(e, sel, &pkg)).cloned().collect()
}

// ── reference taxonomy ──────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum RefKind {
    Structural,
    Traceability,
}

fn vstr(v: &serde_yaml::Value) -> Vec<String> {
    match v {
        serde_yaml::Value::String(s) => vec![s.clone()],
        serde_yaml::Value::Sequence(seq) => {
            seq.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()
        }
        _ => vec![],
    }
}

/// Outbound references that participate in escaping analysis, with their kind.
/// `appliesWhen` operands are intentionally excluded (they reference FeatureDefs).
pub fn outbound_refs(elem: &RawElement) -> Vec<(RefKind, String)> {
    let fm = &elem.frontmatter;
    let mut out: Vec<(RefKind, String)> = Vec::new();
    // structural / typing
    for v in [&fm.supertype, &fm.typed_by, &fm.redefines].into_iter().flatten() {
        for s in vstr(v) {
            out.push((RefKind::Structural, s));
        }
    }
    for xs in [&fm.subsets, &fm.allocated_from, &fm.allocated_to].into_iter().flatten() {
        for s in xs {
            out.push((RefKind::Structural, s.clone()));
        }
    }
    // traceability
    for xs in [&fm.verifies, &fm.satisfies, &fm.derived_from].into_iter().flatten() {
        for s in xs {
            out.push((RefKind::Traceability, s.clone()));
        }
    }
    for s in [&fm.breakdown_adr, &fm.derived_from_safety_goal, &fm.derived_from_security_goal]
        .into_iter()
        .flatten()
    {
        out.push((RefKind::Traceability, s.clone()));
    }
    out
}

/// Escaping references: an active element references one that resolves in the
/// full model but is inactive in the selection. Structural → E226 (error),
/// traceability → W019 (warning).
pub fn escaping_refs(full: &[RawElement], sel: &Selection) -> Vec<Finding> {
    let resolver = Resolver::new(full);
    let pkg = variability::package_conditions(full);
    let mut findings = Vec::new();
    for x in full {
        if !is_active(x, sel, &pkg) {
            continue;
        }
        for (kind, target) in outbound_refs(x) {
            let Some(t) = resolver.resolve_ref(full, &target) else {
                continue; // truly dangling — a 150% concern (whole-model E102 etc.)
            };
            if is_active(t, sel, &pkg) {
                continue;
            }
            match kind {
                RefKind::Structural => findings.push(err(
                    "E226",
                    &x.file_path,
                    format!(
                        "structural reference to '{}' escapes the configuration — the target is inactive in this variant",
                        target
                    ),
                )),
                RefKind::Traceability => findings.push(warn(
                    "W019",
                    &x.file_path,
                    format!(
                        "traceability reference to '{}' escapes the configuration — the target is inactive in this variant",
                        target
                    ),
                )),
            }
        }
    }
    findings
}

/// Cross-reference-resolution codes suppressed in the lens: they are 150%-model
/// concerns (already covered by whole-model `validate`), and for targets that
/// exist-but-are-inactive the escaping-ref pass is authoritative.
const LENS_SUPPRESS: &[&str] = &["E102", "E103", "E104", "E105", "E106"];

/// Full re-validation in the lens (REQ-TRS-PROJ-002): escaping refs plus the
/// standard validator over the projected subset (minus the suppressed
/// resolution codes).
pub fn validate_projected(
    full: &[RawElement],
    config: &ValidateConfig,
    sel: &Selection,
) -> Vec<Finding> {
    let mut findings = escaping_refs(full, sel);
    let active = project(full, sel);
    let res = validator::validate_with_config(&active, config);
    findings.extend(
        res.findings
            .into_iter()
            .filter(|f| !LENS_SUPPRESS.contains(&f.code)),
    );
    findings
}

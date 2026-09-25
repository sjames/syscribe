//! Configuration projection — the `--config` lens (REQ-TRS-PROJ-001..003).
//!
//! Projects the 150% model onto one configuration (the 100% model) by keeping
//! only the elements whose `appliesWhen` holds for that configuration's
//! selection. A projection is itself a valid model, so the lens is implemented
//! as *filter, then reuse* the existing validator/queries. With no `FeatureDef`
//! the library-level [`resolve_selection`] is dormant (a no-op, as a stored
//! `Baseline` scope needs); a user-supplied `--config` flag goes through
//! [`resolve_config_flag`], which makes it a usage error unless the argument
//! names a stored `Configuration`.

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
    // 2. An ad-hoc feature set — each token is a FeatureDef qualified name or its
    //    FEAT-* stable id (REQ-TRS-ID-006); ids are normalized to the qname.
    let feature_defs: HashSet<&str> = elements
        .iter()
        .filter(|e| e.frontmatter.element_type.as_ref() == Some(&ElementType::FeatureDef))
        .map(|e| e.qualified_name.as_str())
        .collect();
    let alias = variability::feature_id_to_qname(elements);
    let tokens: Vec<&str> = arg.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    let canon: Vec<String> = tokens.iter().map(|t| variability::canon_feature_ref(t, &alias)).collect();
    if !canon.is_empty() && canon.iter().all(|t| feature_defs.contains(t.as_str())) {
        let mut sel = Selection::new();
        for t in canon {
            sel.insert(t, true);
        }
        return SelectionOutcome::Resolved(sel);
    }
    SelectionOutcome::Error(format!(
        "--config '{}' is neither a known Configuration nor a set of FeatureDef qualified names",
        arg
    ))
}

/// Resolve a user-supplied `--config` flag (CLI) or `config` argument (MCP).
///
/// Same as [`resolve_selection`], except on a model with no feature model
/// (REQ-TRS-PROJ-001): an argument naming a stored `Configuration` (id or
/// qname — e.g. a MagicGrid parametric variant, which needs no `FeatureDef`)
/// stays [`SelectionOutcome::Dormant`] (the caller uses the whole model), but
/// anything else is a usage error rather than a silent whole-model fallback that
/// would hide a mistyped or misdirected argument. A stored `Baseline`'s
/// `frozenScope.config` keeps using [`resolve_selection`] (REQ-TRS-BL-011).
pub fn resolve_config_flag(elements: &[RawElement], arg: &str) -> SelectionOutcome {
    match resolve_selection(elements, arg) {
        SelectionOutcome::Dormant => {
            let stored = elements.iter().any(|e| {
                e.frontmatter.element_type.as_ref() == Some(&ElementType::Configuration)
                    && (e.frontmatter.id.as_deref() == Some(arg) || e.qualified_name == arg)
            });
            if stored {
                SelectionOutcome::Dormant
            } else {
                SelectionOutcome::Error(format!(
                    "--config '{}': this model declares no feature model (no FeatureDef) and no Configuration named '{}'",
                    arg, arg
                ))
            }
        }
        other => other,
    }
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
    is_active_canon(elem, sel, pkg, &std::collections::HashMap::new())
}

/// As [`is_active`], but with a feature id→qname alias map so an `appliesWhen`
/// operand written as a `FEAT-*` id (REQ-TRS-ID-006) is normalized to the
/// FeatureDef's qualified name before evaluation. `sel` must already be in the
/// canonical (qname) key space — build it via [`canonical_selection`].
pub fn is_active_canon(
    elem: &RawElement,
    sel: &Selection,
    pkg: &std::collections::HashMap<String, serde_yaml::Value>,
    alias: &std::collections::HashMap<String, String>,
) -> bool {
    match variability::effective_applies_when(elem, pkg) {
        None => true,
        Some((aw, _)) => match variability::applies_when_expr(&aw) {
            Ok(Some(expr)) => {
                let expr = expr.canonicalize(&|q: &str| variability::canon_feature_ref(q, alias));
                expr.eval(&|q: &str| sel.get(q).copied().unwrap_or(false))
            }
            _ => true,
        },
    }
}

/// A `Configuration`'s selection canonicalized to qname keys, using the model's
/// feature id→qname alias map (REQ-TRS-ID-006).
pub fn canonical_selection(elements: &[RawElement], cfg: &RawElement) -> Selection {
    let alias = variability::feature_id_to_qname(elements);
    variability::canon_selection(&cfg.frontmatter.feature_selections(), &alias)
}

/// The projected (active) element set for a selection.
pub fn project(elements: &[RawElement], sel: &Selection) -> Vec<RawElement> {
    let pkg = variability::package_conditions(elements);
    let alias = variability::feature_id_to_qname(elements);
    let sel = variability::canon_selection(sel, &alias);
    elements.iter().filter(|e| is_active_canon(e, &sel, &pkg, &alias)).cloned().collect()
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
    for s in [&fm.breakdown_adr, &fm.derived_from_safety_goal, &fm.derived_from_cybersecurity_goal]
        .into_iter()
        .flatten()
    {
        out.push((RefKind::Traceability, s.clone()));
    }
    // User-defined links (REQ-TRS-LINKTYPE-002): every well-formed `links:` target
    // is a traceability reference, exactly like `satisfies:` — an inactive target
    // escapes as W019 (and the lens suppresses its E632, below).
    if let crate::link_types::LinksField::Entries(entries) = crate::link_types::parse_links(fm) {
        for entry in entries.into_iter().filter(|e| e.key_ok) {
            for t in entry.targets.unwrap_or_default() {
                out.push((RefKind::Traceability, t));
            }
        }
    }
    out
}

/// Escaping references: an active element references one that resolves in the
/// full model but is inactive in the selection. Structural → E226 (error),
/// traceability → W019 (warning).
pub fn escaping_refs(full: &[RawElement], sel: &Selection) -> Vec<Finding> {
    let resolver = Resolver::new(full);
    let pkg = variability::package_conditions(full);
    let alias = variability::feature_id_to_qname(full);
    let sel = &variability::canon_selection(sel, &alias);
    let mut findings = Vec::new();
    for x in full {
        if !is_active_canon(x, sel, &pkg, &alias) {
            continue;
        }
        for (kind, target) in outbound_refs(x) {
            let Some(t) = resolver.resolve_ref(full, &target) else {
                continue; // truly dangling — a 150% concern (whole-model E102 etc.)
            };
            if is_active_canon(t, sel, &pkg, &alias) {
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
/// exist-but-are-inactive the escaping-ref pass is authoritative. `E632` (an
/// unresolved `links:` target, REQ-TRS-LINKTYPE-002) is the user-defined-link
/// member of the same family, and `E110`–`E114` (unresolved supertype/typedBy/
/// subsets/redefines/satisfies, REQ-TRS-XREF-007) are the structural members:
/// a target pruned from the variant is E226/W019 here, never a dangling ref.
const LENS_SUPPRESS: &[&str] = &[
    "E102", "E103", "E104", "E105", "E106", "E110", "E111", "E112", "E113", "E114", "E632",
];

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

#[cfg(test)]
mod config_flag_tests {
    use super::*;

    fn elem(qname: &str, yaml: &str) -> RawElement {
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("{qname}.md"),
            frontmatter: serde_yaml::from_str(yaml).unwrap(),
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: vec![],
        }
    }

    /// REQ-TRS-PROJ-001 — with no feature model, the library resolver stays
    /// dormant but a `--config` flag naming no stored Configuration is a usage
    /// error naming the cause; one naming a stored Configuration (a MagicGrid
    /// parametric variant) stays dormant, by id or qname.
    #[test]
    fn config_flag_on_a_model_without_a_feature_model() {
        let m = vec![
            elem("REQ-A", "type: Requirement\nid: REQ-PJ-001\n"),
            elem("Variants::CONF-PJ-VAR-001", "type: Configuration\nid: CONF-PJ-VAR-001\nname: v\n"),
        ];
        assert!(matches!(resolve_selection(&m, "CONF-X"), SelectionOutcome::Dormant));
        match resolve_config_flag(&m, "CONF-X") {
            SelectionOutcome::Error(msg) => {
                assert!(msg.contains("CONF-X") && msg.contains("declares no feature model"), "{msg}")
            }
            _ => panic!("expected a usage error"),
        }
        assert!(matches!(resolve_config_flag(&m, "CONF-PJ-VAR-001"), SelectionOutcome::Dormant));
        assert!(matches!(resolve_config_flag(&m, "Variants::CONF-PJ-VAR-001"), SelectionOutcome::Dormant));
    }

    /// With a feature model present, the flag resolver behaves exactly like
    /// `resolve_selection` (resolved or the usual unresolvable error).
    #[test]
    fn config_flag_with_a_feature_model_matches_resolve_selection() {
        let m = vec![
            elem("Features::A", "type: FeatureDef\nid: FEAT-PJ-A\nname: A\n"),
            elem("CONF-PJ-001", "type: Configuration\nid: CONF-PJ-001\nname: c\nfeatures:\n  Features::A: true\n"),
        ];
        assert!(matches!(resolve_config_flag(&m, "CONF-PJ-001"), SelectionOutcome::Resolved(_)));
        assert!(matches!(resolve_config_flag(&m, "Features::A"), SelectionOutcome::Resolved(_)));
        match resolve_config_flag(&m, "CONF-NOPE") {
            SelectionOutcome::Error(msg) => assert!(!msg.contains("declares no feature model"), "{msg}"),
            _ => panic!("expected the ordinary unresolvable error"),
        }
    }
}

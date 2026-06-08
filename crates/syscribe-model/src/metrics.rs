//! Quantitative hardware safety metrics — ISO 26262-5 §8–9 (GH #29).
//!
//! Computes the Single-Point Fault Metric (**SPFM**), Latent-Fault Metric
//! (**LFM**), and Probabilistic Metric for random Hardware Failures (**PMHF**)
//! per [`SafetyGoal`](crate::element::ElementType::SafetyGoal) from the failure
//! rates and diagnostic coverages of the `FaultTreeEvent`s that contribute to
//! the goal.
//!
//! # First-order approximation
//!
//! This is a first-order, FMEDA-style roll-up driven entirely by user-supplied
//! diagnostic-coverage and failure-rate inputs. It is **not** a substitute for a
//! full FMEDA and must be independently verified before use in a hardware safety
//! case.
//!
//! # Formulas (ISO 26262-5 §8–9)
//!
//! Over a goal's contributing events that declare a `failureRate` (events with
//! no λ are skipped):
//!
//! ```text
//! Σλ      = Σ λ_i
//! λ_RF    = Σ λ_i · (1 − DC_i)              DC_i defaults to 0 when absent
//! SPFM    = 1 − λ_RF / Σλ                    (Σλ = 0 → None)
//! λ_MPFL  = Σ λ_i · DC_i · (1 − DCl_i)       summed over events that DECLARE DCl
//!                                            (an event with no DCl is excluded, so
//!                                             PMHF = λ_RF when no DCl data exists)
//! LFM     = 1 − λ_MPFL / (Σλ − λ_RF)         (computed only if ≥1 event sets DCl;
//!                                             (Σλ − λ_RF) = 0 → None)
//! PMHF    = λ_RF + λ_MPFL                     (/h)
//! ```
//!
//! # Opt-in
//!
//! A goal's metrics are computed and gated **only** if at least one of its
//! contributing events declares `diagnosticCoverage`. Goals without DC data are
//! reported as "n/a" and never gated, so models without coverage stay silent.

use crate::element::RawElement;
use crate::resolver::Resolver;

/// One contributing failure: λ (/h), optional diagnostic coverage (DC), optional
/// latent diagnostic coverage (DCl). `dc`/`dcl` of `None` default to 0 in the
/// formulas; `has_dc`/`has_dcl` track whether the value was *declared* (needed
/// for the opt-in rule and the LFM "n/a" case).
#[derive(Debug, Clone, Copy)]
pub struct Contribution {
    pub lambda: f64,
    pub dc: f64,
    pub dcl: f64,
    pub has_dc: bool,
    pub has_dcl: bool,
}

/// The computed metric bundle for a single [`SafetyGoal`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafetyMetrics {
    /// Sum of contributing failure rates (Σλ, /h).
    pub sum_lambda: f64,
    /// Residual (single-point + residual) failure rate (λ_RF, /h).
    pub lambda_rf: f64,
    /// Latent multiple-point failure rate (λ_MPFL, /h).
    pub lambda_mpfl: f64,
    /// Single-Point Fault Metric; `None` when Σλ = 0.
    pub spfm: Option<f64>,
    /// Latent-Fault Metric; `None` unless at least one event declared DCl (and
    /// the denominator is non-zero).
    pub lfm: Option<f64>,
    /// Probabilistic Metric for random Hardware Failures (/h).
    pub pmhf: f64,
}

/// Compute the metric bundle from a goal's contributing failures.
///
/// `dc` defaults to 0 when absent; `dcl` defaults to 0 when absent. `LFM` is
/// only computed when at least one contribution declared `latentDiagnosticCoverage`.
pub fn compute(contributions: &[Contribution]) -> SafetyMetrics {
    let mut sum_lambda = 0.0;
    let mut lambda_rf = 0.0;
    let mut lambda_mpfl = 0.0;
    let mut any_dcl = false;

    for c in contributions {
        sum_lambda += c.lambda;
        lambda_rf += c.lambda * (1.0 - c.dc);
        // Latent failure rate: only events that declare latentDiagnosticCoverage
        // contribute. Absent DCl means the latent split is unknown for that
        // event, so it is excluded (rather than treated as DCl = 0), keeping
        // PMHF = λ_RF when no DCl data exists at all.
        if c.has_dcl {
            any_dcl = true;
            lambda_mpfl += c.lambda * c.dc * (1.0 - c.dcl);
        }
    }

    let spfm = if sum_lambda == 0.0 {
        None
    } else {
        Some(1.0 - lambda_rf / sum_lambda)
    };

    let lfm = if !any_dcl {
        None
    } else {
        let denom = sum_lambda - lambda_rf;
        if denom == 0.0 {
            None
        } else {
            Some(1.0 - lambda_mpfl / denom)
        }
    };

    let pmhf = lambda_rf + lambda_mpfl;

    SafetyMetrics {
        sum_lambda,
        lambda_rf,
        lambda_mpfl,
        spfm,
        lfm,
        pmhf,
    }
}

/// ASIL/SIL targets for the gateable metrics. A `None` target means the metric
/// is reported but not gated at that integrity level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Targets {
    /// Minimum required SPFM (gate when actual < target).
    pub spfm_min: Option<f64>,
    /// Minimum required LFM (gate when actual < target).
    pub lfm_min: Option<f64>,
    /// Maximum allowed PMHF/PFH in /h (gate when actual ≥ target).
    pub pmhf_max: Option<f64>,
}

/// ISO 26262 ASIL targets (case-insensitive `A`/`B`/`C`/`D`, optional `ASIL `
/// prefix). Returns `None` for an unrecognised level.
pub fn asil_targets(asil: &str) -> Option<Targets> {
    let level = asil
        .trim()
        .trim_start_matches("ASIL")
        .trim_start_matches("asil")
        .trim()
        .to_ascii_uppercase();
    match level.as_str() {
        "A" => Some(Targets { spfm_min: None, lfm_min: None, pmhf_max: None }),
        "B" => Some(Targets { spfm_min: Some(0.90), lfm_min: Some(0.60), pmhf_max: Some(1e-7) }),
        "C" => Some(Targets { spfm_min: Some(0.97), lfm_min: Some(0.80), pmhf_max: Some(1e-7) }),
        "D" => Some(Targets { spfm_min: Some(0.99), lfm_min: Some(0.90), pmhf_max: Some(1e-8) }),
        _ => None,
    }
}

/// IEC 61508 SIL targets. SPFM/LFM are not gated for SIL goals (only PMHF/PFH).
/// Returns `None` for an unrecognised level.
pub fn sil_targets(sil: u8) -> Option<Targets> {
    let pmhf_max = match sil {
        2 => Some(1e-6),
        3 => Some(1e-7),
        4 => Some(1e-8),
        _ => return None,
    };
    Some(Targets { spfm_min: None, lfm_min: None, pmhf_max })
}

/// Which metrics missed their target, with actual vs target for the message.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GateResult {
    pub misses: Vec<MetricMiss>,
}

impl GateResult {
    pub fn passed(&self) -> bool {
        self.misses.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricMiss {
    pub metric: &'static str,
    pub actual: f64,
    pub target: f64,
}

/// Gate computed metrics against targets. SPFM/LFM fail when actual < target;
/// PMHF fails when actual ≥ target. A metric with no value (e.g. LFM = None) or
/// no target is not gated.
pub fn gate(metrics: &SafetyMetrics, targets: &Targets) -> GateResult {
    let mut misses = Vec::new();

    if let (Some(actual), Some(target)) = (metrics.spfm, targets.spfm_min) {
        if actual < target {
            misses.push(MetricMiss { metric: "SPFM", actual, target });
        }
    }
    if let (Some(actual), Some(target)) = (metrics.lfm, targets.lfm_min) {
        if actual < target {
            misses.push(MetricMiss { metric: "LFM", actual, target });
        }
    }
    if let Some(target) = targets.pmhf_max {
        if metrics.pmhf >= target {
            misses.push(MetricMiss { metric: "PMHF", actual: metrics.pmhf, target });
        }
    }

    GateResult { misses }
}

/// Per-goal computed view used by both the validator and the `metrics` command.
#[derive(Debug, Clone)]
pub struct GoalReport {
    /// Stable id (or qualified name when absent) of the goal.
    pub id: String,
    /// File path of the goal element (for findings).
    pub file_path: String,
    /// `asilLevel` as written, if any.
    pub asil: Option<String>,
    /// `silLevel`, if any.
    pub sil: Option<u8>,
    /// Computed metrics — `None` when the goal has no DC data (opt-out) or no
    /// contributing events with a failure rate.
    pub metrics: Option<SafetyMetrics>,
    /// Resolved targets for the goal's integrity level, if recognised.
    pub targets: Option<Targets>,
    /// Gate result — `None` when metrics were not computed.
    pub gate: Option<GateResult>,
}

/// True when `e` is a `FaultTreeEvent` whose qualified name lives under
/// `tree_prefix` (the tree's qualified name + `"::"`).
fn is_event_under(e: &RawElement, tree_prefix: &str) -> bool {
    Resolver::is_fault_tree_event(e) && e.qualified_name.starts_with(tree_prefix)
}

/// Collect the contributing failures for a SafetyGoal: every `FaultTreeEvent`
/// with a `failureRate` that lives under a `FaultTree` whose `topEvent` resolves
/// to the goal. Events without a `failureRate` are skipped.
pub fn contributions_for_goal(
    goal: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> Vec<Contribution> {
    let this_goal_keys = goal_keys(goal);

    let mut out = Vec::new();
    for tree in elements.iter().filter(|e| {
        matches!(e.frontmatter.element_type, Some(crate::element::ElementType::FaultTree))
    }) {
        let Some(ref te) = tree.frontmatter.top_event else { continue };
        let Some(target) = resolver.resolve_ref(elements, te) else { continue };
        if !Resolver::is_safety_goal(target) {
            continue;
        }
        // Does this tree's topEvent resolve to our goal?
        let resolved_keys = goal_keys(target);
        if !this_goal_keys.iter().any(|k| resolved_keys.contains(k)) {
            continue;
        }
        let prefix = format!("{}::", tree.qualified_name);
        for ev in elements.iter().filter(|e| is_event_under(e, &prefix)) {
            let Some(lambda) = ev.frontmatter.failure_rate else { continue };
            out.push(Contribution {
                lambda,
                dc: ev.frontmatter.diagnostic_coverage.unwrap_or(0.0),
                dcl: ev.frontmatter.latent_diagnostic_coverage.unwrap_or(0.0),
                has_dc: ev.frontmatter.diagnostic_coverage.is_some(),
                has_dcl: ev.frontmatter.latent_diagnostic_coverage.is_some(),
            });
        }
    }
    out
}

/// Identity keys for a goal: its qualified name and its stable id (if any).
fn goal_keys(goal: &RawElement) -> Vec<String> {
    let mut keys = vec![goal.qualified_name.clone()];
    if let Some(ref id) = goal.frontmatter.id {
        keys.push(id.clone());
    }
    keys
}

/// Build the [`GoalReport`] for one SafetyGoal, applying the opt-in rule.
pub fn report_for_goal(
    goal: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> GoalReport {
    let id = goal
        .frontmatter
        .id
        .clone()
        .unwrap_or_else(|| goal.qualified_name.clone());
    let asil = goal.frontmatter.asil_level.clone();
    let sil = goal.frontmatter.sil_level;

    let targets = match (asil.as_deref(), sil) {
        (Some(a), _) => asil_targets(a),
        (None, Some(s)) => sil_targets(s),
        _ => None,
    };

    let contributions = contributions_for_goal(goal, elements, resolver);

    // Opt-in: compute only if at least one contributing event declares DC.
    let has_dc = contributions.iter().any(|c| c.has_dc);
    if !has_dc || contributions.is_empty() {
        return GoalReport {
            id,
            file_path: goal.file_path.clone(),
            asil,
            sil,
            metrics: None,
            targets,
            gate: None,
        };
    }

    let metrics = compute(&contributions);
    let gate_result = targets.as_ref().map(|t| gate(&metrics, t));

    GoalReport {
        id,
        file_path: goal.file_path.clone(),
        asil,
        sil,
        metrics: Some(metrics),
        targets,
        gate: gate_result,
    }
}

/// Build reports for every SafetyGoal in the model.
pub fn report_all(elements: &[RawElement], resolver: &Resolver) -> Vec<GoalReport> {
    elements
        .iter()
        .filter(|e| Resolver::is_safety_goal(e))
        .map(|g| report_for_goal(g, elements, resolver))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    fn contrib(lambda: f64, dc: Option<f64>, dcl: Option<f64>) -> Contribution {
        Contribution {
            lambda,
            dc: dc.unwrap_or(0.0),
            dcl: dcl.unwrap_or(0.0),
            has_dc: dc.is_some(),
            has_dcl: dcl.is_some(),
        }
    }

    /// Worked numbers for SG-M-001 (ASIL D): FTE-A (λ=1e-7, DC=0.99),
    /// FTE-B (λ=1e-7, DC=0.90). Σλ=2e-7, λ_RF=1.1e-8, SPFM=0.945,
    /// LFM=n/a (no DCl), PMHF=1.1e-8 — and it must MISS the ASIL D gate.
    #[test]
    fn sg_m_001_worked_numbers() {
        let m = compute(&[
            contrib(1.0e-7, Some(0.99), None),
            contrib(1.0e-7, Some(0.90), None),
        ]);
        assert!((m.sum_lambda - 2.0e-7).abs() < EPS * 2.0e-7 + 1e-18);
        assert!((m.lambda_rf - 1.1e-8).abs() < 1e-18);
        assert!((m.spfm.unwrap() - 0.945).abs() < EPS, "SPFM = {:?}", m.spfm);
        assert_eq!(m.lfm, None, "LFM must be n/a with no DCl");
        assert!((m.pmhf - 1.1e-8).abs() < 1e-18, "PMHF = {}", m.pmhf);

        let targets = asil_targets("D").unwrap();
        let g = gate(&m, &targets);
        assert!(!g.passed(), "SG-M-001 must miss the ASIL D gate");
        let missed: Vec<_> = g.misses.iter().map(|x| x.metric).collect();
        assert!(missed.contains(&"SPFM"), "SPFM should miss: {:?}", missed);
        assert!(missed.contains(&"PMHF"), "PMHF should miss: {:?}", missed);
    }

    /// Worked numbers for SG-M-002 (ASIL B): FTE-C (λ=1e-8, DC=0.99).
    /// Σλ=1e-8, λ_RF=1e-10, SPFM=0.99, PMHF=1e-10 — must PASS the ASIL B gate.
    #[test]
    fn sg_m_002_worked_numbers() {
        let m = compute(&[contrib(1.0e-8, Some(0.99), None)]);
        assert!((m.sum_lambda - 1.0e-8).abs() < 1e-18);
        assert!((m.lambda_rf - 1.0e-10).abs() < 1e-20);
        assert!((m.spfm.unwrap() - 0.99).abs() < EPS, "SPFM = {:?}", m.spfm);
        assert!((m.pmhf - 1.0e-10).abs() < 1e-20, "PMHF = {}", m.pmhf);

        let targets = asil_targets("B").unwrap();
        let g = gate(&m, &targets);
        assert!(g.passed(), "SG-M-002 must pass the ASIL B gate: {:?}", g);
    }

    #[test]
    fn spfm_none_when_sum_lambda_zero() {
        let m = compute(&[]);
        assert_eq!(m.spfm, None);
        assert_eq!(m.lfm, None);
        assert_eq!(m.pmhf, 0.0);
    }

    #[test]
    fn lfm_computed_when_dcl_present() {
        // λ=1e-6, DC=0.9, DCl=0.5.
        // Σλ=1e-6, λ_RF=1e-7, λ_MPFL=1e-6·0.9·0.5=4.5e-7.
        // LFM = 1 − 4.5e-7 / (1e-6 − 1e-7) = 1 − 4.5e-7/9e-7 = 0.5.
        let m = compute(&[contrib(1.0e-6, Some(0.9), Some(0.5))]);
        assert!((m.lambda_mpfl - 4.5e-7).abs() < 1e-18);
        assert!((m.lfm.unwrap() - 0.5).abs() < EPS, "LFM = {:?}", m.lfm);
        // PMHF = λ_RF + λ_MPFL = 1e-7 + 4.5e-7 = 5.5e-7.
        assert!((m.pmhf - 5.5e-7).abs() < 1e-18, "PMHF = {}", m.pmhf);
    }

    #[test]
    fn dc_defaults_to_zero_when_absent() {
        // No DC anywhere: λ_RF = Σλ, SPFM = 0, λ_MPFL = 0.
        let m = compute(&[contrib(1.0e-6, None, None)]);
        assert!((m.lambda_rf - 1.0e-6).abs() < 1e-18);
        assert!((m.spfm.unwrap() - 0.0).abs() < EPS);
        assert_eq!(m.lambda_mpfl, 0.0);
    }

    #[test]
    fn asil_a_has_no_gates() {
        let t = asil_targets("A").unwrap();
        assert_eq!(t.spfm_min, None);
        assert_eq!(t.lfm_min, None);
        assert_eq!(t.pmhf_max, None);
    }

    #[test]
    fn asil_targets_table() {
        assert_eq!(asil_targets("B").unwrap().spfm_min, Some(0.90));
        assert_eq!(asil_targets("C").unwrap().spfm_min, Some(0.97));
        assert_eq!(asil_targets("D").unwrap().spfm_min, Some(0.99));
        assert_eq!(asil_targets("D").unwrap().pmhf_max, Some(1e-8));
        assert_eq!(asil_targets("ASIL D").unwrap().spfm_min, Some(0.99));
        assert_eq!(asil_targets("d").unwrap().lfm_min, Some(0.90));
        assert_eq!(asil_targets("X"), None);
    }

    #[test]
    fn sil_targets_gate_pmhf_only() {
        let t = sil_targets(3).unwrap();
        assert_eq!(t.spfm_min, None);
        assert_eq!(t.lfm_min, None);
        assert_eq!(t.pmhf_max, Some(1e-7));
        assert_eq!(sil_targets(4).unwrap().pmhf_max, Some(1e-8));
        assert_eq!(sil_targets(2).unwrap().pmhf_max, Some(1e-6));
        assert_eq!(sil_targets(1), None);
    }

    #[test]
    fn pmhf_gate_is_strict_less_than() {
        // PMHF exactly equal to the target must FAIL (target is `< target`).
        let m = SafetyMetrics {
            sum_lambda: 1.0,
            lambda_rf: 1e-8,
            lambda_mpfl: 0.0,
            spfm: Some(1.0),
            lfm: None,
            pmhf: 1e-8,
        };
        let t = asil_targets("D").unwrap();
        let g = gate(&m, &t);
        assert!(g.misses.iter().any(|x| x.metric == "PMHF"));
    }
}

//! Quantitative hardware safety metrics — ISO 26262-5 §8–9 (GH #29, #211, #213).
//!
//! Computes the Single-Point Fault Metric (**SPFM**), Latent-Fault Metric
//! (**LFM**), and Probabilistic Metric for random Hardware Failures (**PMHF**)
//! per [`SafetyGoal`](crate::element::ElementType::SafetyGoal) from the failure
//! rates and diagnostic coverages of the `FaultTreeEvent`s that contribute to
//! the goal.
//!
//! # First-order approximation
//!
//! This is a first-order, FMEDA-style roll-up driven by user-supplied
//! diagnostic-coverage and failure-rate inputs. It is **not** a substitute for a
//! full FMEDA and must be independently verified before use in a hardware safety
//! case.
//!
//! # Cut-set driven (GH #213)
//!
//! Which events matter, and how, comes from the minimal cut sets of the goal's
//! `FaultTree`(s) ([`crate::fta`]) — gate logic is evaluated, so `AND(a,b)` and
//! `OR(a,b)` no longer give identical metrics:
//!
//! * Only **reachable, non-`house`** events that appear in at least one cut set
//!   and declare a non-negative `failureRate` contribute.
//! * An event in an **order-1** cut set is a *single-point* event; an event only
//!   in cut sets of order ≥ 2 is a *multi-point* event.
//!
//! # Formulas
//!
//! ```text
//! Σλ      = Σ λ_i                                   over contributing events
//! λ_RF    = Σ_{single-point} λ_i · (1 − DC_i)
//! SPFM    = 1 − λ_RF / Σλ                            (Σλ = 0 → None)
//! λ_MPFL  = Σ_{single-point} λ_i · DC_i · (1 − DCl_i)
//!         + Σ_{multi-point}  λ_i · (1 − DCl_i)
//! LFM     = 1 − λ_MPFL / (Σλ − λ_RF)                 (denominator 0 → None)
//! λ_DPF   = Σ_{order-2 cut sets {a,b}} Σ_{(x,y)∈{(a,b),(b,a)}}
//!               λ_x · (1 − DCl_x) · λ_y · (1 − DC_y) · T
//! PMHF    = λ_RF + λ_DPF                              (/h)
//! ```
//!
//! `T` is the tree's `missionTime` (hours); when absent a default of
//! [`DEFAULT_EXPOSURE_HOURS`] is used and `W986` is raised. Cut sets of order
//! ≥ 3 are not included in `λ_DPF` (their contribution is of higher order).
//!
//! # Missing diagnostic data (GH #213)
//!
//! A contributing event with no `diagnosticCoverage` (or, once any event of the
//! goal declares `latentDiagnosticCoverage`, no `latentDiagnosticCoverage`) is
//! treated as coverage **0** — the conservative choice — and reported as `W985`.
//! `LFM` is `None` while no event of the goal declares DCl at all. DC/DCl
//! outside `0.0`–`1.0` (already `E846`) are clamped.
//!
//! # Opt-in
//!
//! A goal's metrics are computed and gated **only** if at least one of its
//! contributing events declares `diagnosticCoverage`. Goals without DC data are
//! reported as "n/a" and never gated, so models without coverage stay silent.

use crate::element::RawElement;
use crate::fta::{self, AnalysisOptions, EventOrigin, EventRole};
use crate::resolver::Resolver;

/// Lifetime/exposure used for the dual-point term when the fault tree declares
/// no `missionTime` (hours; ~ a typical automotive operating lifetime).
pub const DEFAULT_EXPOSURE_HOURS: f64 = 10_000.0;

/// One contributing failure: λ (/h), diagnostic coverage (DC), latent
/// diagnostic coverage (DCl). `dc`/`dcl` are 0 when absent; `has_dc`/`has_dcl`
/// track whether the value was *declared* (opt-in rule, LFM "n/a" case, `W985`).
#[derive(Debug, Clone, Copy)]
pub struct Contribution {
    pub lambda: f64,
    pub dc: f64,
    pub dcl: f64,
    pub has_dc: bool,
    pub has_dcl: bool,
    /// True for an event in an order-1 cut set; false for a multi-point event.
    pub single_point: bool,
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
    /// Dual-point failure rate over the exposure time (λ_DPF, /h).
    pub lambda_dpf: f64,
    /// Single-Point Fault Metric; `None` when Σλ = 0.
    pub spfm: Option<f64>,
    /// Latent-Fault Metric; `None` unless at least one event declared DCl (and
    /// the denominator is non-zero).
    pub lfm: Option<f64>,
    /// Probabilistic Metric for random Hardware Failures (/h).
    pub pmhf: f64,
}

/// Compute the metric bundle with no dual-point term (`λ_DPF = 0`).
pub fn compute(contributions: &[Contribution]) -> SafetyMetrics {
    compute_with(contributions, 0.0)
}

/// Compute the metric bundle from a goal's contributing failures and the
/// dual-point rate `lambda_dpf` (/h) derived from its order-2 cut sets.
pub fn compute_with(contributions: &[Contribution], lambda_dpf: f64) -> SafetyMetrics {
    let mut sum_lambda = 0.0;
    let mut lambda_rf = 0.0;
    let mut lambda_mpfl = 0.0;
    let mut any_dcl = false;

    for c in contributions {
        sum_lambda += c.lambda;
        if c.has_dcl {
            any_dcl = true;
        }
        if c.single_point {
            lambda_rf += c.lambda * (1.0 - c.dc);
            lambda_mpfl += c.lambda * c.dc * (1.0 - c.dcl);
        } else {
            lambda_mpfl += c.lambda * (1.0 - c.dcl);
        }
    }

    let spfm = if sum_lambda == 0.0 { None } else { Some(1.0 - lambda_rf / sum_lambda) };

    let lfm = if !any_dcl {
        None
    } else {
        let denom = sum_lambda - lambda_rf;
        if denom <= 0.0 {
            None
        } else {
            Some(1.0 - lambda_mpfl / denom)
        }
    };

    SafetyMetrics {
        sum_lambda,
        lambda_rf,
        lambda_mpfl,
        lambda_dpf,
        spfm,
        lfm,
        pmhf: lambda_rf + lambda_dpf,
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

/// IEC 61508 SIL targets (high-demand / continuous PFH). SPFM/LFM are not gated
/// for SIL goals (only PMHF/PFH). Returns `None` for an unrecognised level.
pub fn sil_targets(sil: u8) -> Option<Targets> {
    let pmhf_max = match sil {
        1 => Some(1e-5),
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

/// A contributing event that lacks diagnostic data (`W985`).
#[derive(Debug, Clone, PartialEq)]
pub struct MissingDiagnostic {
    pub event_id: String,
    pub file_path: String,
    /// `"diagnosticCoverage"` or `"latentDiagnosticCoverage"`.
    pub field: &'static str,
}

/// Overall verdict of one goal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Metrics computed and every gated metric meets its target.
    Pass,
    /// Metrics computed and at least one gated metric misses its target.
    Fail,
    /// Metrics computed, but the goal has no recognised ASIL/SIL target.
    NoTarget,
    /// Metrics not computed (no tree, no λ, no DC data, or analysis failed).
    NotComputed,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::Fail => "fail",
            Verdict::NoTarget => "no target",
            Verdict::NotComputed => "n/a",
        }
    }
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
    /// Gate result — `None` when metrics were not computed or no target applies.
    pub gate: Option<GateResult>,
    /// Contributing events lacking DC/DCl (defaulted to 0) — `W985`.
    pub missing: Vec<MissingDiagnostic>,
    /// True when the dual-point term needed a mission time and used
    /// [`DEFAULT_EXPOSURE_HOURS`] — `W986`.
    pub exposure_defaulted: bool,
    /// Fault trees of this goal whose analysis failed (cycle, no top node, …).
    pub analysis_errors: Vec<String>,
}

impl GoalReport {
    pub fn verdict(&self) -> Verdict {
        match (self.metrics.as_ref(), self.gate.as_ref()) {
            (Some(_), Some(g)) if g.passed() => Verdict::Pass,
            (Some(_), Some(_)) => Verdict::Fail,
            (Some(_), None) => Verdict::NoTarget,
            _ => Verdict::NotComputed,
        }
    }
}

/// Identity keys for a goal: its qualified name and its stable id (if any).
fn goal_keys(goal: &RawElement) -> Vec<String> {
    let mut keys = vec![goal.qualified_name.clone()];
    if let Some(ref id) = goal.frontmatter.id {
        keys.push(id.clone());
    }
    keys
}

fn clamp01(v: f64) -> f64 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// The contributing events of one goal, derived from cut-set analysis.
#[derive(Debug, Clone, Default)]
pub struct GoalContributions {
    pub contributions: Vec<Contribution>,
    pub missing: Vec<MissingDiagnostic>,
    /// Dual-point rate λ_DPF (/h), see the module docs.
    pub lambda_dpf: f64,
    pub exposure_defaulted: bool,
    pub analysis_errors: Vec<String>,
}

/// Collect the contributing failures for a SafetyGoal: for every `FaultTree`
/// whose `topEvent` resolves to the goal, run the cut-set analysis
/// ([`crate::fta`]) and take its reachable, non-house events that appear in a
/// cut set and declare a `failureRate`.
pub fn contributions_for_goal(
    goal: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> GoalContributions {
    let this_goal_keys = goal_keys(goal);
    let mut out = GoalContributions::default();
    let mut any_dcl = false;
    // (event id, file, has_dc, has_dcl) per contributing event, for W985.
    let mut declared: Vec<(String, String, bool, bool)> = Vec::new();

    for tree in fta::fault_trees(elements) {
        let Some(ref te) = tree.frontmatter.top_event else { continue };
        let Some(target) = resolver.resolve_ref(elements, te) else { continue };
        if !Resolver::is_safety_goal(target) {
            continue;
        }
        let resolved_keys = goal_keys(target);
        if !this_goal_keys.iter().any(|k| resolved_keys.contains(k)) {
            continue;
        }
        let analysis = match fta::analyze_fault_tree(elements, resolver, tree, &AnalysisOptions::default()) {
            Ok(a) => a,
            Err(e) => {
                let id = tree.frontmatter.id.as_deref().unwrap_or(&tree.qualified_name);
                out.analysis_errors.push(format!("{id}: {e}"));
                continue;
            }
        };
        let mut by_id = std::collections::HashMap::new();
        for ev in &analysis.events {
            if !matches!(ev.role, EventRole::SinglePoint | EventRole::DualPoint | EventRole::MultiPoint) {
                continue;
            }
            let Some(lambda) = ev.effective_failure_rate.filter(|l| l.is_finite() && *l >= 0.0) else { continue };
            let is_model = matches!(ev.origin, EventOrigin::Model);
            let c = Contribution {
                lambda,
                dc: clamp01(ev.diagnostic_coverage.unwrap_or(0.0)),
                dcl: clamp01(ev.latent_diagnostic_coverage.unwrap_or(0.0)),
                has_dc: ev.diagnostic_coverage.is_some(),
                has_dcl: ev.latent_diagnostic_coverage.is_some(),
                single_point: ev.role == EventRole::SinglePoint,
            };
            if is_model {
                declared.push((ev.id.clone(), ev.file_path.clone(), c.has_dc, c.has_dcl));
                any_dcl |= c.has_dcl;
            }
            by_id.insert(ev.id.clone(), c);
            out.contributions.push(c);
        }

        // Dual-point term from the order-2 cut sets.
        let t = match analysis.mission_time_hours {
            Some(t) => t,
            None => DEFAULT_EXPOSURE_HOURS,
        };
        let mut used_default = false;
        for cs in analysis.cut_sets.iter().filter(|c| c.order == 2) {
            let (Some(a), Some(b)) = (by_id.get(&cs.events[0]), by_id.get(&cs.events[1])) else { continue };
            used_default |= analysis.mission_time_hours.is_none();
            for (x, y) in [(a, b), (b, a)] {
                out.lambda_dpf += x.lambda * (1.0 - x.dcl) * y.lambda * (1.0 - y.dc) * t;
            }
        }
        out.exposure_defaulted |= used_default;
    }

    for (id, file, has_dc, has_dcl) in declared {
        if !has_dc {
            out.missing.push(MissingDiagnostic { event_id: id.clone(), file_path: file.clone(), field: "diagnosticCoverage" });
        }
        if any_dcl && !has_dcl {
            out.missing.push(MissingDiagnostic { event_id: id, file_path: file, field: "latentDiagnosticCoverage" });
        }
    }
    out
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

    let gc = contributions_for_goal(goal, elements, resolver);

    // Opt-in: compute only if at least one contributing event declares DC.
    let has_dc = gc.contributions.iter().any(|c| c.has_dc);
    if !has_dc || gc.contributions.is_empty() {
        return GoalReport {
            id,
            file_path: goal.file_path.clone(),
            asil,
            sil,
            metrics: None,
            targets,
            gate: None,
            missing: Vec::new(),
            exposure_defaulted: false,
            analysis_errors: gc.analysis_errors,
        };
    }

    let metrics = compute_with(&gc.contributions, gc.lambda_dpf);
    let gate_result = targets.as_ref().map(|t| gate(&metrics, t));

    GoalReport {
        id,
        file_path: goal.file_path.clone(),
        asil,
        sil,
        metrics: Some(metrics),
        targets,
        gate: gate_result,
        missing: gc.missing,
        exposure_defaulted: gc.exposure_defaulted && gc.lambda_dpf > 0.0,
        analysis_errors: gc.analysis_errors,
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
            single_point: true,
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
        // λ=1e-6, DC=0.9, DCl=0.5 (single point).
        // Σλ=1e-6, λ_RF=1e-7, λ_MPFL=1e-6·0.9·0.5=4.5e-7.
        // LFM = 1 − 4.5e-7 / (1e-6 − 1e-7) = 0.5.
        let m = compute(&[contrib(1.0e-6, Some(0.9), Some(0.5))]);
        assert!((m.lambda_mpfl - 4.5e-7).abs() < 1e-18);
        assert!((m.lfm.unwrap() - 0.5).abs() < EPS, "LFM = {:?}", m.lfm);
        // GH #213: PMHF no longer adds the latent rate (no exposure product).
        assert!((m.pmhf - 1.0e-7).abs() < 1e-18, "PMHF = {}", m.pmhf);
    }

    /// GH #213: declaring DCl on ONE event must not silently change the LFM of
    /// the others — an event without DCl counts as DCl = 0 (conservative).
    #[test]
    fn missing_dcl_is_conservative_not_optimistic() {
        let both = compute(&[
            contrib(1.0e-6, Some(0.9), Some(0.5)),
            contrib(1.0e-6, Some(0.9), None),
        ]);
        // λ_MPFL = 1e-6·0.9·0.5 + 1e-6·0.9·1.0 = 1.35e-6 ; denom = 2e-6 − 2e-7
        let want = 1.0 - 1.35e-6 / 1.8e-6;
        assert!((both.lfm.unwrap() - want).abs() < EPS, "LFM = {:?}", both.lfm);
    }

    #[test]
    fn multi_point_events_have_no_residual_rate() {
        // Event only in order>=2 cut sets: no λ_RF, all of λ(1−DCl) is latent MP.
        let mp = Contribution { single_point: false, ..contrib(1.0e-6, Some(0.9), Some(0.5)) };
        let m = compute(&[mp]);
        assert_eq!(m.lambda_rf, 0.0);
        assert_eq!(m.spfm, Some(1.0));
        assert!((m.lambda_mpfl - 5.0e-7).abs() < 1e-18);
        assert_eq!(m.pmhf, 0.0);
    }

    #[test]
    fn dual_point_term_enters_pmhf() {
        let m = compute_with(&[contrib(1.0e-6, Some(0.9), None)], 3.0e-9);
        assert!((m.pmhf - (1.0e-7 + 3.0e-9)).abs() < 1e-18);
        assert!((m.lambda_dpf - 3.0e-9).abs() < 1e-18);
    }

    #[test]
    fn dc_defaults_to_zero_when_absent() {
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
        // GH #213: SIL 1 is gated too (PFH < 1e-5 /h).
        assert_eq!(sil_targets(1).unwrap().pmhf_max, Some(1e-5));
        assert_eq!(sil_targets(0), None);
    }

    #[test]
    fn pmhf_gate_is_strict_less_than() {
        // PMHF exactly equal to the target must FAIL (target is `< target`).
        let m = SafetyMetrics {
            sum_lambda: 1.0,
            lambda_rf: 1e-8,
            lambda_mpfl: 0.0,
            lambda_dpf: 0.0,
            spfm: Some(1.0),
            lfm: None,
            pmhf: 1e-8,
        };
        let t = asil_targets("D").unwrap();
        let g = gate(&m, &t);
        assert!(g.misses.iter().any(|x| x.metric == "PMHF"));
    }

    // ── model-driven (gate logic matters) ──

    use crate::element::{ParseIssue, RawFrontmatter};

    fn el(qname: &str, yaml: &str) -> RawElement {
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("{}.md", qname.replace("::", "/")),
            frontmatter: serde_yaml::from_str::<RawFrontmatter>(yaml).expect("yaml"),
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
            about_notes: Default::default(),
        }
    }

    fn model(gate_kind: &str, extra_b: &str) -> Vec<RawElement> {
        vec![
            el("S::SG-MM-001", "type: SafetyGoal\nid: SG-MM-001\nname: g\nasilLevel: D\nstatus: draft\n"),
            el("S::FT-MM-001", "type: FaultTree\nid: FT-MM-001\nname: t\nstatus: draft\ntopEvent: SG-MM-001\nmissionTime: \"1000 h\"\n"),
            el("S::FT-MM-001::FTE-MM-001", "type: FaultTreeEvent\nid: FTE-MM-001\nname: a\neventKind: basic\nfailureRate: 1.0e-6\ndiagnosticCoverage: 0.9\n"),
            el("S::FT-MM-001::FTE-MM-002", &format!("type: FaultTreeEvent\nid: FTE-MM-002\nname: b\neventKind: basic\nfailureRate: 1.0e-6\ndiagnosticCoverage: 0.9\n{extra_b}")),
            el("S::FT-MM-001::FTG-MM-001", &format!("type: FaultTreeGate\nid: FTG-MM-001\nname: g\ngateType: {gate_kind}\ninputs:\n  - FTE-MM-001\n  - FTE-MM-002\n")),
        ]
    }

    fn report(els: &[RawElement]) -> GoalReport {
        let r = Resolver::new(els);
        report_for_goal(&els[0], els, &r)
    }

    #[test]
    fn and_and_or_give_different_metrics() {
        let or = report(&model("OR", ""));
        let and = report(&model("AND", ""));
        let (mo, ma) = (or.metrics.unwrap(), and.metrics.unwrap());
        assert!((mo.spfm.unwrap() - 0.9).abs() < EPS, "OR: both single point");
        assert_eq!(ma.spfm, Some(1.0), "AND: no single point");
        assert_eq!(ma.lambda_rf, 0.0);
        // λ_DPF = 2 · (1e-6 · 1.0) · (1e-6 · 0.1) · 1000 = 2e-10
        assert!((ma.lambda_dpf - 2.0e-10).abs() < 1e-20, "λ_DPF = {}", ma.lambda_dpf);
        assert!((ma.pmhf - 2.0e-10).abs() < 1e-20);
        assert!(mo.pmhf > ma.pmhf);
    }

    #[test]
    fn house_and_unreachable_events_are_skipped() {
        let mut els = model("OR", "");
        els.push(el("S::FT-MM-001::FTE-MM-003", "type: FaultTreeEvent\nid: FTE-MM-003\nname: c\neventKind: basic\nfailureRate: 5.0e-6\ndiagnosticCoverage: 0.0\n"));
        els.push(el("S::FT-MM-001::FTE-MM-004", "type: FaultTreeEvent\nid: FTE-MM-004\nname: h\neventKind: house\nfailureRate: 5.0e-6\n"));
        let m = report(&els).metrics.unwrap();
        assert!((m.sum_lambda - 2.0e-6).abs() < 1e-15, "Σλ = {}", m.sum_lambda);
    }

    #[test]
    fn negative_lambda_is_ignored() {
        let els = model("OR", "").into_iter().chain([el(
            "S::FT-MM-001::FTE-MM-003",
            "type: FaultTreeEvent\nid: FTE-MM-003\nname: c\neventKind: basic\nfailureRate: -1.0e-6\ndiagnosticCoverage: 0.5\n",
        )]).collect::<Vec<_>>();
        let m = report(&els).metrics.unwrap();
        assert!((m.sum_lambda - 2.0e-6).abs() < 1e-15);
    }

    #[test]
    fn missing_dc_and_dcl_are_reported() {
        let els = model("OR", "latentDiagnosticCoverage: 0.5\n");
        let r = report(&els);
        // FTE-MM-001 lacks DCl (another event declares it); both declare DC.
        assert!(r.missing.iter().any(|m| m.event_id == "FTE-MM-001" && m.field == "latentDiagnosticCoverage"));
        assert!(!r.missing.iter().any(|m| m.field == "diagnosticCoverage"));
    }

    #[test]
    fn out_of_range_dcl_is_clamped_not_lfm_2() {
        let els = model("OR", "latentDiagnosticCoverage: 7.0\n");
        let m = report(&els).metrics.unwrap();
        assert!(m.lfm.unwrap() <= 1.0, "LFM = {:?}", m.lfm);
    }

    #[test]
    fn verdict_distinguishes_no_target_and_fail() {
        let mut els = model("OR", "");
        assert_eq!(report(&els).verdict(), Verdict::Fail);
        els[0] = el("S::SG-MM-001", "type: SafetyGoal\nid: SG-MM-001\nname: g\nstatus: draft\n");
        assert_eq!(report(&els).verdict(), Verdict::NoTarget);
        els[0] = el("S::SG-MM-001", "type: SafetyGoal\nid: SG-MM-001\nname: g\nsilLevel: 1\nstatus: draft\n");
        assert_eq!(report(&els).verdict(), Verdict::Pass);
    }
}

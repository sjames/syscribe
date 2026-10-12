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
//! The first-order SPFM/LFM/PMHF roll-up is computed only if at least one contributing event declares
//! `diagnosticCoverage`, and only then gates `W033`; models without coverage stay silent in `validate`.
//!
//! # Rate-only and demand-mode figures (GH #343)
//!
//! A goal with a quantified tree and no coverage data still gets a figure, labelled as rate only: the exact
//! (BDD) top-event probability over the tree's `missionTime`, divided by it (PFH, /h). A goal whose `target:` says
//! `mode: demand` is judged on PFDavg over its `proofTestInterval` TI instead:
//!
//! ```text
//! PFDavg = Σ_{minimal cut sets} Π_{i} λ_DU,i · TI^k / (k + 1) · Π_{j} p_j      λ_DU = λ · (1 − DC)
//! ```
//!
//! (k events with a rate, the events j with only a `probability`). The limit comes from the goal's own
//! `target.value`, else `[reliability.targets]` in `.syscribe.toml`, else the built-in table (labelled as a
//! default). A figure whose contributing events (or top event) fail with a probability above the plausibility bound
//! over the mission time is "not credible": it is no longer a rate the target can judge. A rate-only figure gates
//! `W033` only when the goal declares its own `target:`.

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
    /// Maximum allowed PFDavg in demand mode (gate when actual ≥ target; GH #343).
    pub pfd_max: Option<f64>,
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
        "A" => Some(Targets { spfm_min: None, lfm_min: None, pmhf_max: None, pfd_max: None }),
        "B" => Some(Targets { spfm_min: Some(0.90), lfm_min: Some(0.60), pmhf_max: Some(1e-7), pfd_max: None }),
        "C" => Some(Targets { spfm_min: Some(0.97), lfm_min: Some(0.80), pmhf_max: Some(1e-7), pfd_max: None }),
        "D" => Some(Targets { spfm_min: Some(0.99), lfm_min: Some(0.90), pmhf_max: Some(1e-8), pfd_max: None }),
        _ => None,
    }
}

/// IEC 61508 SIL targets: PFH for high-demand / continuous mode, PFDavg for low-demand mode (the upper bound of each
/// band, GH #343). SPFM/LFM are not gated for SIL goals. Returns `None` for an unrecognised level.
pub fn sil_targets(sil: u8) -> Option<Targets> {
    let (pmhf_max, pfd_max) = match sil {
        1 => (1e-5, 1e-1),
        2 => (1e-6, 1e-2),
        3 => (1e-7, 1e-3),
        4 => (1e-8, 1e-4),
        _ => return None,
    };
    Some(Targets { spfm_min: None, lfm_min: None, pmhf_max: Some(pmhf_max), pfd_max: Some(pfd_max) })
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
    /// A rate-only or demand figure was computed, but a contributing event's probability over the mission time (or
    /// the top event's) is above the plausibility bound, so the figure is not a rate the target can judge (GH #343).
    NotCredible,
    /// Metrics computed, but the goal has no recognised ASIL/SIL target.
    NoTarget,
    /// Metrics not computed (no tree, no λ, or analysis failed).
    NotComputed,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::Fail => "fail",
            Verdict::NotCredible => "not credible",
            Verdict::NoTarget => "no target",
            Verdict::NotComputed => "n/a",
        }
    }
}

/// The operating mode a goal's target is stated in (GH #343).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Continuous / high-demand: a dangerous-failure frequency (PMHF, PFH) in /h.
    #[default]
    Continuous,
    /// Low-demand: the average probability of failure on demand (PFDavg) over a proof-test interval.
    Demand,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Mode::Continuous => "continuous",
            Mode::Demand => "demand",
        }
    }
}

/// A goal's own `target:` (GH #343): `{mode: continuous|demand, value, unit: /h|probability, proofTestInterval}`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GoalTarget {
    pub mode: Mode,
    /// Overrides the table's PMHF/PFH (continuous) or PFDavg (demand) limit.
    pub value: Option<f64>,
    /// The proof-test interval in hours (demand mode).
    pub proof_test_hours: Option<f64>,
}

/// Parse a `SafetyGoal.target:` mapping; every defect is one message (`E985`).
pub fn parse_goal_target(v: &serde_yaml::Value) -> Result<GoalTarget, Vec<String>> {
    let Some(map) = v.as_mapping() else {
        return Err(vec!["`target:` must be a mapping with `mode`, `value`, `unit` and (demand mode) `proofTestInterval`".into()]);
    };
    let mut errs = Vec::new();
    let mut t = GoalTarget::default();
    let get = |k: &str| map.get(serde_yaml::Value::String(k.into()));
    for k in map.keys() {
        let k = k.as_str().unwrap_or("");
        if !matches!(k, "mode" | "value" | "unit" | "proofTestInterval") {
            errs.push(format!("`target.{k}` is not a target field (mode, value, unit, proofTestInterval)"));
        }
    }
    match get("mode").map(|m| m.as_str()) {
        None | Some(Some("continuous")) => {}
        Some(Some("demand")) => t.mode = Mode::Demand,
        _ => errs.push("`target.mode` must be `continuous` or `demand`".into()),
    }
    match get("value") {
        None => {}
        Some(v) => match v.as_f64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())) {
            Some(x) if x.is_finite() && x > 0.0 && (t.mode == Mode::Continuous || x <= 1.0) => t.value = Some(x),
            _ => errs.push(format!("`target.value` must be a positive number{}", if t.mode == Mode::Demand { " no larger than 1 (a probability)" } else { " (a frequency in /h)" })),
        },
    }
    match (get("unit").map(|u| u.as_str()), t.mode) {
        (None, _) | (Some(Some("/h")), Mode::Continuous) | (Some(Some("probability")), Mode::Demand) => {}
        (Some(Some(u)), m) => errs.push(format!("`target.unit` '{u}' does not fit {} mode, which needs `{}`", m.label(), if m == Mode::Continuous { "/h" } else { "probability" })),
        (Some(None), _) => errs.push("`target.unit` must be `/h` or `probability`".into()),
    }
    match get("proofTestInterval") {
        None if t.mode == Mode::Demand => errs.push("demand mode needs `target.proofTestInterval` (a duration such as `8760 h` or `1 year`)".into()),
        None => {}
        Some(v) => match v.as_str().and_then(crate::fta::parse_mission_time).or_else(|| v.as_f64().filter(|x| x.is_finite() && *x > 0.0)) {
            Some(h) => t.proof_test_hours = Some(h),
            None => errs.push("`target.proofTestInterval` must be a positive duration such as `8760 h` or `1 year`".into()),
        },
    }
    if errs.is_empty() {
        Ok(t)
    } else {
        Err(errs)
    }
}

/// Where a goal's target comes from — the values are the user's compliance decision, so a report always says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetSource {
    /// The goal's own `target:`.
    Goal,
    /// `[reliability.targets]` in `.syscribe.toml`.
    Config,
    /// The built-in ISO 26262 / IEC 61508 table.
    Default,
}

impl TargetSource {
    pub fn label(self) -> &'static str {
        match self {
            TargetSource::Goal => "goal target",
            TargetSource::Config => ".syscribe.toml",
            TargetSource::Default => "built-in default",
        }
    }
}

/// Default plausibility bound: no single event should fail with a probability above this over the mission time.
pub const DEFAULT_PLAUSIBLE_PROBABILITY: f64 = 0.1;

/// The target table (`[reliability.targets]` over the built-ins) and the plausibility bound (GH #343).
#[derive(Debug, Clone, PartialEq)]
pub struct ReliabilityConfig {
    /// ASIL letter → targets, and the limits `.syscribe.toml` set (`spfm`, `lfm`, `pmhf`, `pfd`).
    pub asil: std::collections::BTreeMap<String, (Targets, Vec<&'static str>)>,
    /// SIL → targets, and the limits `.syscribe.toml` set.
    pub sil: std::collections::BTreeMap<u8, (Targets, Vec<&'static str>)>,
    /// The largest credible probability of one event over the mission time.
    pub plausible_probability: f64,
    pub plausibility_configured: bool,
}

impl Default for ReliabilityConfig {
    fn default() -> Self {
        ReliabilityConfig {
            asil: ["A", "B", "C", "D"].iter().map(|a| (a.to_string(), (asil_targets(a).expect("built-in"), Vec::new()))).collect(),
            sil: (1..=4).map(|s| (s, (sil_targets(s).expect("built-in"), Vec::new()))).collect(),
            plausible_probability: DEFAULT_PLAUSIBLE_PROBABILITY,
            plausibility_configured: false,
        }
    }
}

impl ReliabilityConfig {
    /// Load `[reliability]` from `<model_root>/.syscribe.toml` over the built-ins. The defects (`W989`) come back
    /// alongside; a defective entry is ignored whole.
    pub fn load(model_root: &std::path::Path) -> (ReliabilityConfig, Vec<String>) {
        let mut cfg = ReliabilityConfig::default();
        let Ok(text) = std::fs::read_to_string(model_root.join(".syscribe.toml")) else { return (cfg, Vec::new()) };
        let Ok(root) = toml::from_str::<toml::Value>(&text) else { return (cfg, Vec::new()) };
        let Some(rel) = root.get("reliability") else { return (cfg, Vec::new()) };
        let mut errs = Vec::new();
        let Some(rel) = rel.as_table() else { return (cfg, vec!["`[reliability]` must be a table".into()]) };
        let num = |v: &toml::Value| v.as_float().or_else(|| v.as_integer().map(|i| i as f64));
        for (k, v) in rel {
            match k.as_str() {
                "plausibleProbability" | "plausible_probability" => match num(v) {
                    Some(x) if x > 0.0 && x <= 1.0 => {
                        cfg.plausible_probability = x;
                        cfg.plausibility_configured = true;
                    }
                    _ => errs.push(format!("`reliability.{k}` must be a probability in (0, 1]")),
                },
                "targets" => {
                    let Some(t) = v.as_table() else {
                        errs.push("`reliability.targets` must be a table of `asil.<A-D>` / `sil.<1-4>` tables".into());
                        continue;
                    };
                    for (scheme, levels) in t {
                        let Some(levels) = levels.as_table().filter(|_| matches!(scheme.as_str(), "asil" | "sil")) else {
                            errs.push(format!("`reliability.targets.{scheme}` must be the `asil` or `sil` table"));
                            continue;
                        };
                        for (level, vals) in levels {
                            let base = if scheme == "asil" {
                                cfg.asil.get(&level.to_ascii_uppercase()).map(|x| x.0)
                            } else {
                                level.parse::<u8>().ok().and_then(|l| cfg.sil.get(&l)).map(|x| x.0)
                            };
                            let mut set: Vec<&'static str> = Vec::new();
                            let (Some(mut tg), Some(vals)) = (base, vals.as_table()) else {
                                errs.push(format!("`reliability.targets.{scheme}.{level}`: not a level of {} with a table of limits", if scheme == "asil" { "ASIL A–D" } else { "SIL 1–4" }));
                                continue;
                            };
                            let mut bad = false;
                            for (m, x) in vals {
                                let Some(x) = num(x).filter(|x| x.is_finite() && *x > 0.0) else {
                                    errs.push(format!("`reliability.targets.{scheme}.{level}.{m}` must be a positive number"));
                                    bad = true;
                                    continue;
                                };
                                match m.as_str() {
                                    "spfm" if x <= 1.0 => {
                                        tg.spfm_min = Some(x);
                                        set.push("spfm");
                                    }
                                    "lfm" if x <= 1.0 => {
                                        tg.lfm_min = Some(x);
                                        set.push("lfm");
                                    }
                                    "pmhf" | "pfh" => {
                                        tg.pmhf_max = Some(x);
                                        set.push("pmhf");
                                    }
                                    "pfd" if x <= 1.0 => {
                                        tg.pfd_max = Some(x);
                                        set.push("pfd");
                                    }
                                    _ => {
                                        errs.push(format!("`reliability.targets.{scheme}.{level}.{m}`: not a limit (spfm, lfm ≤ 1; pmhf or pfh in /h; pfd ≤ 1)"));
                                        bad = true;
                                    }
                                }
                            }
                            if !bad {
                                if scheme == "asil" {
                                    cfg.asil.insert(level.to_ascii_uppercase(), (tg, set));
                                } else if let Ok(l) = level.parse::<u8>() {
                                    cfg.sil.insert(l, (tg, set));
                                }
                            }
                        }
                    }
                }
                other => errs.push(format!("`reliability.{other}` is not a key (plausibleProbability, targets)")),
            }
        }
        (cfg, errs)
    }

    /// The targets of a goal's integrity level, and where they come from.
    /// `mode` picks the limit the source is about: `pmhf` (continuous) or `pfd` (demand).
    pub fn targets_for(&self, asil: Option<&str>, sil: Option<u8>, mode: Mode) -> Option<(Targets, TargetSource)> {
        let hit = match (asil, sil) {
            (Some(a), _) => {
                let l = a.trim().trim_start_matches("ASIL").trim_start_matches("asil").trim().to_ascii_uppercase();
                self.asil.get(&l)
            }
            (None, Some(s)) => self.sil.get(&s),
            _ => None,
        }?;
        let key = match mode {
            Mode::Continuous => "pmhf",
            Mode::Demand => "pfd",
        };
        Some((hit.0, if hit.1.contains(&key) { TargetSource::Config } else { TargetSource::Default }))
    }
}

/// How a goal's figure was computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
    /// The first-order FMEDA roll-up from diagnostic coverage (PMHF = λ_RF + λ_DPF), as before GH #343.
    FirstOrder,
    /// Rate only (no diagnostic coverage): the exact top-event probability over the mission time, divided by it.
    TopEvent,
    /// Demand mode: Σ over the minimal cut sets of Π λ_DU,i · TI^k / (k + 1) (· the probability of events with no λ).
    CutSetPfd,
}

impl Basis {
    pub fn label(self) -> &'static str {
        match self {
            Basis::FirstOrder => "first-order (diagnostic coverage)",
            Basis::TopEvent => "rate only (top event / mission time)",
            Basis::CutSetPfd => "cut sets (PFDavg)",
        }
    }
}

/// The figure a goal is judged on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Figure {
    /// `PMHF` (ASIL, continuous), `PFH` (SIL or rate-only, continuous) or `PFDavg` (demand).
    pub metric: &'static str,
    pub value: f64,
    pub basis: Basis,
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
    /// Computed metrics — `None` when the goal has no DC data or no contributing events with a failure rate.
    pub metrics: Option<SafetyMetrics>,
    /// Resolved targets (the integrity level's, with the goal's own `target.value` applied).
    pub targets: Option<Targets>,
    /// Where the gating limit comes from.
    pub target_source: Option<TargetSource>,
    /// Gate result — `None` when nothing was computed or no target applies.
    pub gate: Option<GateResult>,
    /// Contributing events lacking DC/DCl (defaulted to 0) — `W985`.
    pub missing: Vec<MissingDiagnostic>,
    /// True when the dual-point term needed a mission time and used
    /// [`DEFAULT_EXPOSURE_HOURS`] — `W986`.
    pub exposure_defaulted: bool,
    /// Fault trees of this goal whose analysis failed (cycle, no top node, …).
    pub analysis_errors: Vec<String>,
    /// A contributing event holds `failureRate` / `diagnosticCoverage` / `latentDiagnosticCoverage` as
    /// a whole-value placeholder (GH #268): the figures exist per Configuration, so the base model
    /// computes none and reports no miss.
    pub pending_placeholders: bool,
    /// The goal's mode (GH #343).
    pub mode: Mode,
    /// The goal declares its own `target:` (an explicit opt-in to `W033` for a rate-only figure).
    pub declares_target: bool,
    /// The figure the goal is judged on: PMHF/PFH/PFDavg (GH #343).
    pub figure: Option<Figure>,
    /// Exact top-event probability over the mission time, summed over the goal's trees.
    pub top_probability: Option<f64>,
    /// The goal's trees' mission time (hours), the largest when they differ.
    pub mission_hours: Option<f64>,
    /// Contributing events whose rate is not `verified` or `secondary` (`rateStatus`), so the verdict rests on
    /// unverified rates.
    pub unverified_rates: Vec<String>,
    /// Why the figure is not credible: events (or the top event) whose probability over the mission time is above
    /// the plausibility bound, with that probability.
    pub implausible: Vec<(String, f64)>,
    /// The contributing event with the largest Birnbaum importance (how much the top event moves with it), and that
    /// importance.
    pub dominant: Option<(String, f64)>,
    /// The contributing event with the largest Fussell-Vesely importance (the share of the top event that involves
    /// it), and that importance: the rate most worth checking first.
    pub largest_contributor: Option<(String, f64)>,
    /// The goal's `target:` is malformed (`E985`), so it was ignored and the defaults judge the goal.
    pub target_ignored: bool,
}

impl GoalReport {
    pub fn verdict(&self) -> Verdict {
        if self.figure.is_some_and(|f| f.basis != Basis::FirstOrder) && !self.implausible.is_empty() {
            return Verdict::NotCredible;
        }
        match ((self.metrics.is_some() || self.figure.is_some()), self.gate.as_ref()) {
            (true, Some(g)) if g.passed() => Verdict::Pass,
            (true, Some(_)) => Verdict::Fail,
            (true, None) => Verdict::NoTarget,
            _ => Verdict::NotComputed,
        }
    }

    /// Whether a `fail` gates (`W033`, `metrics` exit 2): a first-order figure always does; a rate-only or PFDavg
    /// figure only when the project opted in, by the goal's own `target:` or a `.syscribe.toml` limit, so existing
    /// models without coverage data do not start failing.
    pub fn gates(&self) -> bool {
        self.figure.is_some_and(|f| f.basis == Basis::FirstOrder) || self.declares_target || self.target_source == Some(TargetSource::Config)
    }

    /// The figure is rate-only (no diagnostic coverage), so the verdict says so.
    pub fn rate_only(&self) -> bool {
        self.figure.is_some_and(|f| f.basis == Basis::TopEvent)
    }

    /// The limit the figure is gated against.
    pub fn limit(&self) -> Option<f64> {
        let t = self.targets?;
        match self.mode {
            Mode::Continuous => t.pmhf_max,
            Mode::Demand => t.pfd_max,
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
    /// See [`GoalReport::pending_placeholders`].
    pub pending_placeholders: bool,
    /// The analyses of the goal's trees (GH #343: the exact top event, the cut sets for PFDavg).
    pub analyses: Vec<fta::FaultTreeAnalysis>,
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
            if matches!(ev.origin, EventOrigin::Model)
                && resolver.get_by_id(elements, &ev.id).is_some_and(|e| {
                    ["failureRate", "diagnosticCoverage", "latentDiagnosticCoverage"].iter().any(|k| e.frontmatter.placeholder_fields.contains_key(*k))
                })
            {
                out.pending_placeholders = true;
            }
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
        out.analyses.push(analysis);
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

/// Build the [`GoalReport`] for one SafetyGoal against the built-in targets.
pub fn report_for_goal(goal: &RawElement, elements: &[RawElement], resolver: &Resolver) -> GoalReport {
    report_for_goal_with(goal, elements, resolver, &ReliabilityConfig::default())
}

/// The `rateStatus` that stands behind an event's λ: the event's own, or that of the FMEDA row that supplies it.
pub fn rate_status_of<'a>(elements: &'a [RawElement], resolver: &Resolver, event_id: &str) -> Option<&'a str> {
    let ev = resolver.get_by_id(elements, event_id)?;
    if ev.frontmatter.failure_rate.is_some() {
        return ev.frontmatter.rate_status.as_deref();
    }
    elements
        .iter()
        .filter(|r| matches!(r.frontmatter.element_type, Some(crate::element::ElementType::FMEAEntry)) && r.frontmatter.failure_rate.is_some())
        .find(|r| r.frontmatter.fta_ref.as_deref().and_then(|x| resolver.resolve_ref(elements, x)).is_some_and(|t| t.qualified_name == ev.qualified_name))
        .and_then(|r| r.frontmatter.rate_status.as_deref())
}

/// A rate is claimed when its `rateStatus` is `verified` or `secondary`.
pub fn is_claimed(status: Option<&str>) -> bool {
    matches!(status, Some("verified" | "secondary"))
}

/// PFDavg of one analysed tree over the proof-test interval `ti` (hours): Σ over the minimal cut sets of
/// Π λ_DU,i · TI^k / (k + 1), λ_DU = λ · (1 − DC), times the probability of the cut set's events that have no λ.
/// `None` when an event of a cut set has neither.
pub fn pfd_avg(analysis: &fta::FaultTreeAnalysis, ti: f64) -> Option<f64> {
    let mut sum = 0.0;
    for cs in &analysis.cut_sets {
        let (mut prod, mut k) = (1.0, 0i32);
        for id in &cs.events {
            let ev = analysis.event(id)?;
            match ev.effective_failure_rate.filter(|l| l.is_finite() && *l >= 0.0) {
                Some(l) => {
                    prod *= l * (1.0 - clamp01(ev.diagnostic_coverage.unwrap_or(0.0))) * ti;
                    k += 1;
                }
                None => prod *= ev.probability?,
            }
        }
        sum += prod / (k as f64 + 1.0);
    }
    Some(sum)
}

/// Build the [`GoalReport`] for one SafetyGoal: the first-order roll-up when the goal's events declare diagnostic
/// coverage (continuous mode, as before GH #343), else a rate-only PFH from the exact top event, or PFDavg in demand
/// mode; with the target table `cfg` and the goal's own `target:`.
pub fn report_for_goal_with(goal: &RawElement, elements: &[RawElement], resolver: &Resolver, cfg: &ReliabilityConfig) -> GoalReport {
    let id = goal.frontmatter.id.clone().unwrap_or_else(|| goal.qualified_name.clone());
    let asil = goal.frontmatter.asil_level.clone();
    let sil = goal.frontmatter.sil_level;
    let goal_target = goal.frontmatter.target.as_ref().and_then(|v| parse_goal_target(v).ok());
    let target_ignored = goal.frontmatter.target.is_some() && goal_target.is_none();
    let declares_target = goal_target.is_some();
    let goal_target = goal_target.unwrap_or_default();
    let mode = goal_target.mode;

    let (mut targets, mut target_source) = match cfg.targets_for(asil.as_deref(), sil, mode) {
        Some((t, s)) => (Some(t), Some(s)),
        None => (None, None),
    };
    if let Some(v) = goal_target.value {
        let mut t = targets.unwrap_or(Targets { spfm_min: None, lfm_min: None, pmhf_max: None, pfd_max: None });
        match mode {
            Mode::Continuous => t.pmhf_max = Some(v),
            Mode::Demand => t.pfd_max = Some(v),
        }
        targets = Some(t);
        target_source = Some(TargetSource::Goal);
    }

    let gc = contributions_for_goal(goal, elements, resolver);
    let mut r = GoalReport {
        id,
        file_path: goal.file_path.clone(),
        asil,
        sil,
        metrics: None,
        targets,
        target_source,
        gate: None,
        missing: Vec::new(),
        exposure_defaulted: false,
        analysis_errors: gc.analysis_errors.clone(),
        pending_placeholders: gc.pending_placeholders,
        mode,
        declares_target,
        figure: None,
        top_probability: None,
        mission_hours: None,
        unverified_rates: Vec::new(),
        implausible: Vec::new(),
        dominant: None,
        largest_contributor: None,
        target_ignored,
    };
    // A figure that is per Configuration cannot be judged in the base model (GH #268).
    if gc.pending_placeholders {
        return r;
    }

    // what every goal with a quantified tree carries: the exact top event, the rates' provenance, plausibility
    let contributing = |ev: &&fta::EventResult| {
        matches!(ev.role, EventRole::SinglePoint | EventRole::DualPoint | EventRole::MultiPoint) && ev.effective_failure_rate.is_some_and(|l| l.is_finite() && l >= 0.0)
    };
    let tops: Vec<Option<f64>> = gc.analyses.iter().map(|a| a.top_probability).collect();
    if !tops.is_empty() && tops.iter().all(Option::is_some) {
        r.top_probability = Some(tops.iter().flatten().sum());
    }
    r.mission_hours = gc.analyses.iter().filter_map(|a| a.mission_time_hours).fold(None, |m: Option<f64>, t| Some(m.map_or(t, |x| x.max(t))));
    for a in &gc.analyses {
        for ev in a.events.iter().filter(contributing) {
            if matches!(ev.origin, EventOrigin::Model) && !is_claimed(rate_status_of(elements, resolver, &ev.id)) && !r.unverified_rates.contains(&ev.id) {
                r.unverified_rates.push(ev.id.clone());
            }
            // on demand the figure needs λ_DU·TI small, not λ·T: judge each event over the proof-test interval
            let p = match (mode, goal_target.proof_test_hours) {
                (Mode::Demand, Some(ti)) => ev.effective_failure_rate.map(|l| l * (1.0 - clamp01(ev.diagnostic_coverage.unwrap_or(0.0))) * ti),
                _ => ev.probability,
            };
            if let Some(p) = p.filter(|p| *p > cfg.plausible_probability) {
                if !r.implausible.iter().any(|(x, _)| x == &ev.id) {
                    r.implausible.push((ev.id.clone(), p));
                }
            }
            if let Some(b) = ev.birnbaum {
                if r.dominant.as_ref().is_none_or(|(_, d)| b > *d) {
                    r.dominant = Some((ev.id.clone(), b));
                }
            }
            if let Some(fv) = ev.fussell_vesely {
                if r.largest_contributor.as_ref().is_none_or(|(_, d)| fv > *d) {
                    r.largest_contributor = Some((ev.id.clone(), fv));
                }
            }
        }
        // the mission-time top event bounds a continuous figure only; on demand the PFDavg is checked below
        if mode == Mode::Continuous {
            if let Some(p) = a.top_probability.filter(|p| *p > cfg.plausible_probability) {
                r.implausible.push((format!("top event of {}", a.tree_id), p));
            }
        }
    }
    r.implausible.sort_by(|a, b| b.1.total_cmp(&a.1));

    if gc.contributions.is_empty() {
        return r;
    }
    let has_dc = gc.contributions.iter().any(|c| c.has_dc);
    let miss = |metric: &'static str, value: f64, limit: Option<f64>| -> Option<GateResult> {
        limit.map(|t| GateResult { misses: if value >= t { vec![MetricMiss { metric, actual: value, target: t }] } else { Vec::new() } })
    };
    match mode {
        Mode::Continuous if has_dc => {
            let metrics = compute_with(&gc.contributions, gc.lambda_dpf);
            r.gate = r.targets.as_ref().map(|t| gate(&metrics, t));
            r.figure = Some(Figure { metric: if r.asil.is_some() { "PMHF" } else { "PFH" }, value: metrics.pmhf, basis: Basis::FirstOrder });
            r.metrics = Some(metrics);
            r.missing = gc.missing;
            r.exposure_defaulted = gc.exposure_defaulted && gc.lambda_dpf > 0.0;
        }
        Mode::Continuous => {
            // rate only: the average dangerous-failure frequency over each tree's mission time
            let per_tree: Option<Vec<f64>> = gc.analyses.iter().map(|a| Some(a.top_probability? / a.mission_time_hours?)).collect();
            if let Some(pfh) = per_tree.filter(|v| !v.is_empty()).map(|v| v.iter().sum::<f64>()) {
                r.figure = Some(Figure { metric: "PFH", value: pfh, basis: Basis::TopEvent });
                r.gate = miss("PFH", pfh, r.limit());
            }
        }
        Mode::Demand => {
            let pfd = goal_target.proof_test_hours.and_then(|ti| gc.analyses.iter().map(|a| pfd_avg(a, ti)).sum::<Option<f64>>());
            if let Some(pfd) = pfd.filter(|_| !gc.analyses.is_empty()) {
                r.figure = Some(Figure { metric: "PFDavg", value: pfd, basis: Basis::CutSetPfd });
                if pfd > cfg.plausible_probability {
                    r.implausible.push(("PFDavg".into(), pfd));
                }
                let mut g = miss("PFDavg", pfd, r.limit()).unwrap_or_default();
                if has_dc {
                    // SPFM/LFM still judge the diagnostic coverage; the frequency limit does not apply on demand
                    let metrics = compute_with(&gc.contributions, gc.lambda_dpf);
                    if let Some(t) = r.targets {
                        g.misses.extend(gate(&metrics, &Targets { pmhf_max: None, ..t }).misses);
                    }
                    r.metrics = Some(metrics);
                    r.missing = gc.missing;
                }
                // with no PFDavg limit (an ASIL table has none) the goal has no target on demand: never a `pass` on
                // the coverage metrics alone
                r.gate = r.limit().map(|_| g);
            }
        }
    }
    r
}

/// Build reports for every SafetyGoal in the model against the built-in targets.
pub fn report_all(elements: &[RawElement], resolver: &Resolver) -> Vec<GoalReport> {
    report_all_with(elements, resolver, &ReliabilityConfig::default())
}

/// Build reports for every SafetyGoal in the model against `cfg`.
pub fn report_all_with(elements: &[RawElement], resolver: &Resolver, cfg: &ReliabilityConfig) -> Vec<GoalReport> {
    elements.iter().filter(|e| Resolver::is_safety_goal(e)).map(|g| report_for_goal_with(g, elements, resolver, cfg)).collect()
}

/// How one event's rate moves a goal's verdict (`metrics --sensitivity`, GH #343).
#[derive(Debug, Clone, PartialEq)]
pub struct Sensitivity {
    pub event: String,
    pub birnbaum: Option<f64>,
    /// The figure with this event's λ divided by 10 and multiplied by 10.
    pub figure_down: Option<f64>,
    pub figure_up: Option<f64>,
    pub verdict_down: Verdict,
    pub verdict_up: Verdict,
}

/// For each event contributing to `goal`, the figure and verdict with its λ divided and multiplied by 10, ranked by
/// how far the figure moves.
pub fn sensitivity(goal: &RawElement, elements: &[RawElement], resolver: &Resolver, cfg: &ReliabilityConfig) -> Vec<Sensitivity> {
    let gc = contributions_for_goal(goal, elements, resolver);
    let mut ids: Vec<(String, Option<f64>)> = Vec::new();
    for a in &gc.analyses {
        for ev in &a.events {
            if matches!(ev.origin, EventOrigin::Model) && matches!(ev.role, EventRole::SinglePoint | EventRole::DualPoint | EventRole::MultiPoint) && ev.failure_rate.is_some() && !ids.iter().any(|(i, _)| i == &ev.id) {
                ids.push((ev.id.clone(), ev.birnbaum));
            }
        }
    }
    let mut out = Vec::new();
    for (id, birnbaum) in ids {
        let Some(ev) = resolver.get_by_id(elements, &id) else { continue };
        let qn = ev.qualified_name.clone();
        let own = ev.frontmatter.failure_rate.is_some();
        let scaled = |factor: f64| -> GoalReport {
            let mut copy = elements.to_vec();
            for e in copy.iter_mut() {
                let is_event = e.qualified_name == qn;
                let is_row = !own && matches!(e.frontmatter.element_type, Some(crate::element::ElementType::FMEAEntry)) && e.frontmatter.failure_rate.is_some() && e.frontmatter.fta_ref.as_deref().and_then(|x| resolver.resolve_ref(elements, x)).is_some_and(|t| t.qualified_name == qn);
                if (is_event && own) || is_row {
                    e.frontmatter.failure_rate = e.frontmatter.failure_rate.map(|l| l * factor);
                }
            }
            let res = Resolver::new(&copy);
            let g = copy.iter().find(|e| e.qualified_name == goal.qualified_name).expect("the goal is in the copy");
            report_for_goal_with(g, &copy, &res, cfg)
        };
        let (down, up) = (scaled(0.1), scaled(10.0));
        out.push(Sensitivity { event: id, birnbaum, figure_down: down.figure.map(|f| f.value), figure_up: up.figure.map(|f| f.value), verdict_down: down.verdict(), verdict_up: up.verdict() });
    }
    let spread = |s: &Sensitivity| match (s.figure_down, s.figure_up) {
        (Some(a), Some(b)) if a > 0.0 => (b / a).ln(),
        _ => 0.0,
    };
    out.sort_by(|a, b| spread(b).total_cmp(&spread(a)).then(a.event.cmp(&b.event)));
    out
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

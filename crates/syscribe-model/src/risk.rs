//! ISO/SAE 21434 cybersecurity risk determination (§15.8).
//!
//! A single, shared definition of how a `ThreatScenario`'s risk level is
//! computed from the **severity** of its linked `DamageScenario`s and the
//! threat's own **attack feasibility**. Both the validator (W031/W032) and the
//! `cyber-risk` CLI command use these functions so there is exactly ONE
//! risk-level definition in the codebase.
//!
//! Risk model (implemented exactly as specified in GH #30):
//! - severity rank: negligible=0, moderate=1, major=2, severe=3 — take the
//!   **max** `damageSeverity` over the threat's resolved `DamageScenario`s.
//! - feasibility rank: very_low=0, low=1, medium=2, high=3.
//! - if either rank is unknown → risk = **Unknown** (listed, never gated).
//! - else `score = severity + feasibility` (0..6) → level:
//!   0–1 `low`, 2–3 `medium`, 4 `high`, 5–6 `critical`.

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

/// Computed cybersecurity risk level for a `ThreatScenario`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    /// Lowercase label used in messages, tables and JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }

    /// Ordinal rank low=0 .. critical=3 (used for the CAL comparison).
    pub fn rank(self) -> u8 {
        match self {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
            RiskLevel::Critical => 3,
        }
    }
}

/// severity rank: negligible=0, moderate=1, major=2, severe=3.
pub fn severity_rank(s: &str) -> Option<u8> {
    match s {
        "negligible" => Some(0),
        "moderate" => Some(1),
        "major" => Some(2),
        "severe" => Some(3),
        _ => None,
    }
}

/// feasibility rank: very_low=0, low=1, medium=2, high=3.
pub fn feasibility_rank(f: &str) -> Option<u8> {
    match f {
        "very_low" => Some(0),
        "low" => Some(1),
        "medium" => Some(2),
        "high" => Some(3),
        _ => None,
    }
}

/// Max `damageSeverity` rank over the `DamageScenario`s named in the threat's
/// `damageScenarios`, resolved via [`Resolver`]. `None` if none resolve / none
/// carry a (valid) `damageSeverity`.
pub fn threat_severity_rank(
    threat: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> Option<u8> {
    let refs = threat.frontmatter.damage_scenarios.as_ref()?;
    let mut best: Option<u8> = None;
    for r in refs {
        if let Some(ds) = resolver.resolve_ref(elements, r) {
            if ds.frontmatter.element_type == Some(ElementType::DamageScenario) {
                if let Some(sev) = ds.frontmatter.damage_severity.as_deref().and_then(severity_rank)
                {
                    best = Some(best.map_or(sev, |b| b.max(sev)));
                }
            }
        }
    }
    best
}

/// Computed [`RiskLevel`] for a `ThreatScenario`, or `None` (= unknown, not
/// gated) when severity or feasibility cannot be determined.
pub fn threat_risk_level(
    threat: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> Option<RiskLevel> {
    let sev = threat_severity_rank(threat, elements, resolver)?;
    let feas = threat
        .frontmatter
        .attack_feasibility
        .as_deref()
        .and_then(feasibility_rank)?;
    Some(level_from_score(sev + feas))
}

/// Map a 0..6 score to a [`RiskLevel`]: 0–1 low, 2–3 medium, 4 high, 5–6 critical.
pub fn level_from_score(score: u8) -> RiskLevel {
    match score {
        0..=1 => RiskLevel::Low,
        2..=3 => RiskLevel::Medium,
        4 => RiskLevel::High,
        _ => RiskLevel::Critical,
    }
}

/// Rank of a `calLevel` string CAL1=1 .. CAL4=4 (used by W032). `None` if invalid.
pub fn cal_rank(cal: &str) -> Option<u8> {
    match cal {
        "CAL1" => Some(1),
        "CAL2" => Some(2),
        "CAL3" => Some(3),
        "CAL4" => Some(4),
        _ => None,
    }
}

/// Expected minimum CAL rank for a risk level: low→CAL1, medium→CAL2,
/// high→CAL3, critical→CAL4.
pub fn expected_cal_rank(level: RiskLevel) -> u8 {
    match level {
        RiskLevel::Low => 1,
        RiskLevel::Medium => 2,
        RiskLevel::High => 3,
        RiskLevel::Critical => 4,
    }
}

/// `CALn` label for an expected CAL rank (1..4).
pub fn cal_label(rank: u8) -> String {
    format!("CAL{}", rank)
}

//! `cyber-risk` — ISO/SAE 21434 risk determination view (GH #30).
//!
//! Lists each `ThreatScenario` with its damage severity, attack feasibility,
//! computed risk level, `riskTreatment`, whether it is addressed by a
//! `CybersecurityGoal`, and a flag: `untreated` when it would trip W031
//! (high/critical risk, no treatment, not addressed), `unknown` when the risk
//! cannot be computed, else `ok`.
//!
//! Read-only: traverses frontmatter and the [`Resolver`]; shares the single
//! risk-level definition in [`syscribe_model::risk`] with the validator.

use serde_json::json;
use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::attack_tree::feasibility_label;
use syscribe_model::risk::{
    expected_cal_for_threat, threat_feasibility_rank, threat_risk, threat_severity_rank, CyberConfig,
    RiskLevel,
};
use std::collections::HashSet;

fn disp_id(e: &RawElement) -> String {
    e.frontmatter
        .id
        .clone()
        .unwrap_or_else(|| e.qualified_name.clone())
}

struct Row {
    id: String,
    severity: String,
    feasibility: String,
    risk: String,
    treatment: String,
    addressed: bool,
    flag: &'static str,
    // Only shown when a `[cyber]` configuration is in effect (GH #222).
    vector: String,
    risk_value: Option<u8>,
    cal: Option<u8>,
}

pub fn cmd_cyber_risk(elements: &[RawElement], cfg: &CyberConfig, json_out: bool) {
    let resolver = Resolver::new(elements);

    // Threat keys (qname + id) addressed by some CybersecurityGoal.
    let mut addressed: HashSet<String> = HashSet::new();
    for csg in elements.iter().filter(|e| Resolver::is_cybersecurity_goal(e)) {
        if let Some(ref refs) = csg.frontmatter.threat_scenarios {
            for r in refs {
                if let Some(ts) = resolver.resolve_ref(elements, r) {
                    if Resolver::is_threat_scenario(ts) {
                        addressed.insert(ts.qualified_name.clone());
                        if let Some(ref id) = ts.frontmatter.id {
                            addressed.insert(id.clone());
                        }
                    }
                }
            }
        }
    }

    let mut rows: Vec<Row> = Vec::new();
    for ts in elements.iter().filter(|e| Resolver::is_threat_scenario(e)) {
        // Severity label: derived from the max over linked DamageScenarios.
        let sev_rank = threat_severity_rank(ts, elements, &resolver);
        let severity = sev_rank
            .map(severity_label)
            .unwrap_or("unknown")
            .to_string();

        let feas_str = ts.frontmatter.attack_feasibility.as_deref();
        let feasibility = match (feas_str, threat_feasibility_rank(&ts.frontmatter, cfg)) {
            (Some(f), Some(_)) => f.to_string(),
            (None, Some(r)) => feasibility_label(r).to_string(),
            _ => "unknown".to_string(),
        };

        let computed = threat_risk(ts, elements, &resolver, cfg);
        let level = computed.map(|r| r.level);
        let risk = level.map(|l| l.as_str()).unwrap_or("unknown").to_string();

        let treatment = ts
            .frontmatter
            .risk_treatment
            .clone()
            .unwrap_or_else(|| "—".to_string());

        let is_addressed = addressed.contains(&ts.qualified_name)
            || ts
                .frontmatter
                .id
                .as_ref()
                .is_some_and(|id| addressed.contains(id));

        let has_treatment = ts.frontmatter.risk_treatment.is_some();
        let flag = match level {
            None => "unknown",
            Some(l) if (l == RiskLevel::High || l == RiskLevel::Critical)
                && !has_treatment
                && !is_addressed =>
            {
                "untreated"
            }
            Some(_) => "ok",
        };

        rows.push(Row {
            id: disp_id(ts),
            severity,
            feasibility,
            risk,
            treatment,
            addressed: is_addressed,
            flag,
            vector: ts
                .frontmatter
                .attack_vector
                .clone()
                .unwrap_or_else(|| "—".to_string()),
            risk_value: computed.and_then(|r| r.value),
            cal: expected_cal_for_threat(ts, elements, &resolver, cfg),
        });
    }

    rows.sort_by(|a, b| a.id.cmp(&b.id));

    if json_out {
        emit_json(&rows, cfg);
    } else {
        emit_text(&rows, cfg);
    }
}

fn severity_label(rank: u8) -> &'static str {
    match rank {
        0 => "negligible",
        1 => "moderate",
        2 => "major",
        _ => "severe",
    }
}

fn emit_json(rows: &[Row], cfg: &CyberConfig) {
    let arr: Vec<_> = rows
        .iter()
        .map(|r| {
            let mut o = json!({
                "id": r.id,
                "severity": r.severity,
                "feasibility": r.feasibility,
                "risk": r.risk,
                "treatment": r.treatment,
                "addressed": r.addressed,
                "flag": r.flag,
            });
            if !cfg.is_default() {
                o["method"] = json!(cfg.method().as_str());
                o["attackVector"] = json!(r.vector);
                o["riskValue"] = json!(r.risk_value);
                o["expectedCal"] = json!(r.cal.map(|c| format!("CAL{}", c)));
            }
            o
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&json!(arr)).unwrap());
}

fn emit_text(rows: &[Row], cfg: &CyberConfig) {
    if rows.is_empty() {
        println!("# Cybersecurity Risk Determination (ISO/SAE 21434 §15.8)\n");
        println!("No ThreatScenario elements found.");
        return;
    }
    println!("# Cybersecurity Risk Determination (ISO/SAE 21434 §15.8)\n");
    if !cfg.is_default() {
        println!("Risk configuration: {}\n", cfg.describe());
        println!(
            "| Threat | Severity | Feasibility | Vector | Risk | Risk value | Expected CAL | Treatment | Addressed | Flag |"
        );
        println!("|---|---|---|---|---|---|---|---|---|---|");
        for r in rows {
            println!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                r.id,
                r.severity,
                r.feasibility,
                r.vector,
                r.risk,
                r.risk_value.map(|v| v.to_string()).unwrap_or_else(|| "—".to_string()),
                r.cal.map(|c| format!("CAL{}", c)).unwrap_or_else(|| "—".to_string()),
                r.treatment,
                if r.addressed { "yes" } else { "no" },
                r.flag,
            );
        }
        return;
    }
    println!(
        "| Threat | Severity | Feasibility | Risk | Treatment | Addressed | Flag |"
    );
    println!("|---|---|---|---|---|---|---|");
    for r in rows {
        println!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            r.id,
            r.severity,
            r.feasibility,
            r.risk,
            r.treatment,
            if r.addressed { "yes" } else { "no" },
            r.flag,
        );
    }
}

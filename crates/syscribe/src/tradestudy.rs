//! General-purpose `TradeStudy` surface (§15, GH #63). Read-only: computes min-max
//! normalised scores, weighted totals, and rankings; never writes to disk.

use std::collections::HashMap;
use syscribe_model::element::{ElementType, RawElement};

fn is_study(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::TradeStudy))
}

fn id_of(e: &RawElement) -> &str {
    e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name)
}

fn sval<'a>(m: &'a serde_yaml::Mapping, k: &str) -> Option<&'a serde_yaml::Value> {
    m.get(serde_yaml::Value::String(k.to_string()))
}
fn snum(v: &serde_yaml::Value) -> Option<f64> {
    match v {
        serde_yaml::Value::Number(n) => n.as_f64(),
        serde_yaml::Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

struct Criterion {
    name: String,
    weight: f64,
    maximize: bool,
}

/// Computed scoring for a study.
struct Scored {
    criteria: Vec<Criterion>,
    alternatives: Vec<String>,
    /// normalized[alt][crit] in [0,1]
    normalized: HashMap<String, HashMap<String, f64>>,
    totals: HashMap<String, f64>,
    /// alt → rank (1-based)
    ranks: HashMap<String, usize>,
}

fn compute(study: &RawElement) -> Scored {
    let fm = &study.frontmatter;
    let criteria: Vec<Criterion> = fm
        .criteria
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(|c| {
            let m = c.as_mapping()?;
            let name = sval(m, "name")?.as_str()?.to_string();
            let weight = sval(m, "weight").and_then(snum).unwrap_or(0.0);
            let maximize = sval(m, "direction").and_then(|v| v.as_str()) != Some("minimize");
            Some(Criterion { name, weight, maximize })
        })
        .collect();
    let alternatives: Vec<String> = fm
        .alternatives
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(|a| a.as_mapping().and_then(|m| sval(m, "name")).and_then(|v| v.as_str()).map(String::from))
        .collect();

    // raw[alt][crit] = score
    let mut raw: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for s in fm.scores.as_deref().unwrap_or(&[]) {
        if let Some(m) = s.as_mapping() {
            if let (Some(a), Some(c), Some(v)) = (
                sval(m, "alternative").and_then(|v| v.as_str()),
                sval(m, "criterion").and_then(|v| v.as_str()),
                sval(m, "score").and_then(snum),
            ) {
                raw.entry(a.to_string()).or_default().insert(c.to_string(), v);
            }
        }
    }

    // Min-max normalise per criterion column (direction applied; missing → 0).
    let mut normalized: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for crit in &criteria {
        let vals: Vec<f64> = alternatives
            .iter()
            .filter_map(|a| raw.get(a).and_then(|m| m.get(&crit.name)).copied())
            .collect();
        let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let span = max - min;
        for a in &alternatives {
            let n = match raw.get(a).and_then(|m| m.get(&crit.name)).copied() {
                None => 0.0,
                Some(_) if span == 0.0 || !span.is_finite() => 1.0,
                Some(v) => {
                    let up = (v - min) / span; // 1 at max, 0 at min
                    if crit.maximize { up } else { 1.0 - up }
                }
            };
            normalized.entry(a.clone()).or_default().insert(crit.name.clone(), n);
        }
    }

    // Normalised weights.
    let wsum: f64 = criteria.iter().map(|c| c.weight).sum();
    let totals: HashMap<String, f64> = alternatives
        .iter()
        .map(|a| {
            let total = criteria
                .iter()
                .map(|c| {
                    let nw = if wsum > 0.0 { c.weight / wsum } else { 0.0 };
                    normalized.get(a).and_then(|m| m.get(&c.name)).copied().unwrap_or(0.0) * nw
                })
                .sum();
            (a.clone(), total)
        })
        .collect();

    let mut ordered: Vec<(&String, f64)> = totals.iter().map(|(a, t)| (a, *t)).collect();
    ordered.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let ranks: HashMap<String, usize> = ordered.iter().enumerate().map(|(i, (a, _))| ((*a).clone(), i + 1)).collect();

    Scored { criteria, alternatives, normalized, totals, ranks }
}

pub fn cmd_trade_study(elements: &[RawElement], filter: Option<&str>, json: bool) {
    match filter {
        Some(id) => detail(elements, id, json),
        None => list(elements, json),
    }
}

fn list(elements: &[RawElement], json: bool) {
    let mut studies: Vec<&RawElement> = elements.iter().filter(|e| is_study(e)).collect();
    studies.sort_by(|a, b| id_of(a).cmp(id_of(b)));

    if json {
        let arr: Vec<serde_json::Value> = studies
            .iter()
            .map(|e| {
                serde_json::json!({
                    "id": id_of(e),
                    "name": e.frontmatter.name,
                    "status": e.frontmatter.status,
                    "alternatives": e.frontmatter.alternatives.as_ref().map(|a| a.len()).unwrap_or(0),
                    "complete": e.frontmatter.status.as_deref() == Some("complete"),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "tradeStudies": arr })).unwrap());
        return;
    }
    if studies.is_empty() {
        println!("No TradeStudy elements in the model.");
        return;
    }
    println!("| ID | Name | Status | Alternatives | Complete |");
    println!("|---|---|---|---|---|");
    for e in &studies {
        println!(
            "| {} | {} | {} | {} | {} |",
            id_of(e),
            e.frontmatter.name.as_deref().unwrap_or("—"),
            e.frontmatter.status.as_deref().unwrap_or("—"),
            e.frontmatter.alternatives.as_ref().map(|a| a.len()).unwrap_or(0),
            if e.frontmatter.status.as_deref() == Some("complete") { "✓" } else { "✗" },
        );
    }
}

fn detail(elements: &[RawElement], id: &str, json: bool) {
    let study = elements.iter().find(|e| is_study(e) && id_of(e) == id);
    let study = match study {
        Some(s) => s,
        None => {
            eprintln!("No TradeStudy with id '{}' found.", id);
            std::process::exit(1);
        }
    };
    let sc = compute(study);
    let fm = &study.frontmatter;

    if json {
        let alts: Vec<serde_json::Value> = sc
            .alternatives
            .iter()
            .map(|a| {
                let per: serde_json::Map<String, serde_json::Value> = sc
                    .criteria
                    .iter()
                    .map(|c| {
                        (c.name.clone(), serde_json::json!(sc.normalized.get(a).and_then(|m| m.get(&c.name)).copied().unwrap_or(0.0)))
                    })
                    .collect();
                serde_json::json!({
                    "name": a,
                    "normalized": per,
                    "total": sc.totals.get(a).copied().unwrap_or(0.0),
                    "rank": sc.ranks.get(a).copied().unwrap_or(0),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "id": id_of(study),
                "name": fm.name,
                "status": fm.status,
                "objective": fm.objective,
                "decision": fm.decision,
                "criteria": sc.criteria.iter().map(|c| serde_json::json!({ "name": c.name, "weight": c.weight, "direction": if c.maximize {"maximize"} else {"minimize"} })).collect::<Vec<_>>(),
                "alternatives": alts,
            }))
            .unwrap()
        );
        return;
    }

    println!("Trade Study: {} — {}", id_of(study), fm.name.as_deref().unwrap_or(""));
    println!(
        "Objective: {}   Decision ADR: {}",
        fm.objective.as_deref().unwrap_or("—"),
        fm.decision.as_deref().unwrap_or("—"),
    );
    println!();
    // Header
    let mut header = format!("| {:<14} ", "Alternative");
    let mut sep = "|---".to_string();
    for c in &sc.criteria {
        header.push_str(&format!("| {}({}) ", c.name, c.weight));
        sep.push_str("|---");
    }
    header.push_str("| Total | Rank |");
    sep.push_str("|---|---|");
    println!("{}", header);
    println!("{}", sep);
    // Rows ordered by rank
    let mut alts = sc.alternatives.clone();
    alts.sort_by_key(|a| sc.ranks.get(a).copied().unwrap_or(usize::MAX));
    for a in &alts {
        let mut row = format!("| {:<14} ", a);
        for c in &sc.criteria {
            row.push_str(&format!("| {:.3} ", sc.normalized.get(a).and_then(|m| m.get(&c.name)).copied().unwrap_or(0.0)));
        }
        row.push_str(&format!("| {:.3} | #{} |", sc.totals.get(a).copied().unwrap_or(0.0), sc.ranks.get(a).copied().unwrap_or(0)));
        println!("{}", row);
    }
}

/// True when the model contains at least one TradeStudy element.
pub fn has_trade_studies(elements: &[RawElement]) -> bool {
    elements.iter().any(is_study)
}

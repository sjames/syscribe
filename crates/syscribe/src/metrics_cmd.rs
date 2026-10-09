//! `metrics` — quantitative hardware safety metrics roll-up (GH #29).
//!
//! One row per [`SafetyGoal`](syscribe_model::element::ElementType::SafetyGoal):
//! its ASIL/SIL, computed SPFM / LFM / PMHF (ISO 26262-5 §8–9), and an overall
//! pass/fail verdict against the goal's target. Goals without a FaultTree or
//! without `diagnosticCoverage` data show `n/a`. Read-only.
//!
//! Shares the single formula definition in
//! [`syscribe_model::metrics`] with the validator's W033 gate.

use serde_json::json;
use syscribe_model::element::RawElement;
use syscribe_model::metrics::{report_all, GoalReport, Verdict};
use syscribe_model::resolver::Resolver;

fn fmt_opt(v: Option<f64>, prec: usize) -> String {
    match v {
        Some(x) => format!("{:.*}", prec, x),
        None => "n/a".to_string(),
    }
}

fn fmt_pmhf(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.3e}", x),
        None => "n/a".to_string(),
    }
}

/// Print the metrics table. Returns the process exit code: `2` when at least
/// one goal misses its ASIL/SIL target (GH #213; `2`, the `audit` gate code,
/// leaving `1` for usage errors), else `0`.
pub fn cmd_metrics(elements: &[RawElement], json_out: bool) -> i32 {
    let resolver = Resolver::new(elements);
    let mut reports = report_all(elements, &resolver);
    reports.sort_by(|a, b| a.id.cmp(&b.id));

    if json_out {
        emit_json(&reports);
    } else {
        emit_text(&reports);
    }
    let failing: Vec<&str> = reports
        .iter()
        .filter(|r| r.verdict() == Verdict::Fail)
        .map(|r| r.id.as_str())
        .collect();
    if failing.is_empty() {
        0
    } else {
        eprintln!("metrics: {} goal(s) miss their target: {}", failing.len(), failing.join(", "));
        2
    }
}

fn emit_json(reports: &[GoalReport]) {
    let arr: Vec<_> = reports
        .iter()
        .map(|r| {
            let m = r.metrics.as_ref();
            json!({
                "id": r.id,
                "asil": r.asil,
                "sil": r.sil,
                "spfm": m.and_then(|x| x.spfm),
                "lfm": m.and_then(|x| x.lfm),
                "pmhf": m.map(|x| x.pmhf),
                "lambdaDpf": m.map(|x| x.lambda_dpf),
                "pass": match r.verdict() {
                    Verdict::Pass => Some(true),
                    Verdict::Fail => Some(false),
                    _ => None,
                },
                "verdict": r.verdict().label(),
            })
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&json!(arr)).unwrap());
}

fn emit_text(reports: &[GoalReport]) {
    println!("# Quantitative HW Safety Metrics (ISO 26262-5 §8-9)\n");
    println!(
        "> First-order FMEDA approximation from user-supplied λ and diagnostic coverage. \
         Verify independently before use in a safety case.\n"
    );

    if reports.is_empty() {
        println!("No SafetyGoal elements found.");
        return;
    }

    println!("| SafetyGoal | ASIL | SIL | SPFM | LFM | PMHF (/h) | Verdict |");
    println!("|---|---|---|---|---|---|---|");
    for r in reports {
        let asil = r.asil.clone().unwrap_or_else(|| "—".to_string());
        let sil = r.sil.map(|s| s.to_string()).unwrap_or_else(|| "—".to_string());
        let (spfm, lfm, pmhf) = match r.metrics.as_ref() {
            Some(m) => (fmt_opt(m.spfm, 4), fmt_opt(m.lfm, 4), fmt_pmhf(Some(m.pmhf))),
            None => (
                "n/a".to_string(),
                "n/a".to_string(),
                "n/a".to_string(),
            ),
        };
        let verdict_str = r.verdict().label();
        println!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            r.id, asil, sil, spfm, lfm, pmhf, verdict_str
        );
    }

    let any_na = reports.iter().any(|r| r.metrics.is_none());
    if any_na {
        println!(
            "\n_Goals showing `n/a` have no FaultTree, no failure rates, or no \
             `diagnosticCoverage` data (opt-in: metrics are computed only when at \
             least one contributing FaultTreeEvent declares `diagnosticCoverage`)._"
        );
    }
    if reports.iter().any(|r| r.verdict() == Verdict::NoTarget) {
        println!("\n_`no target`: metrics are computed but the goal has no recognised `asilLevel`/`silLevel`._");
    }
    for r in reports {
        for e in &r.analysis_errors {
            println!("\n> {}: fault-tree analysis failed — {e}", r.id);
        }
    }
    println!("\n_Exit status is 2 when any goal's verdict is `fail`._");
}

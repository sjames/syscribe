//! `metrics` — quantitative safety targets per goal (GH #29, #343).
//!
//! One row per [`SafetyGoal`](syscribe_model::element::ElementType::SafetyGoal): its ASIL/SIL, mode, the figure it
//! is judged on (first-order PMHF/PFH from diagnostic coverage, a rate-only PFH from the exact top event, or PFDavg in
//! demand mode), the limit and where it comes from, and the verdict. Read-only.
//!
//! Shares the single formula definition in [`syscribe_model::metrics`] with the validator's W033 gate.

use serde_json::json;
use syscribe_model::element::RawElement;
use syscribe_model::metrics::{report_all_with, sensitivity, GoalReport, ReliabilityConfig, Verdict};
use syscribe_model::resolver::Resolver;

fn fmt_opt(v: Option<f64>, prec: usize) -> String {
    match v {
        Some(x) => format!("{:.*}", prec, x),
        None => "n/a".to_string(),
    }
}

fn fmt_e(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.3e}", x),
        None => "n/a".to_string(),
    }
}

/// Print the metrics table. Returns the process exit code: `2` when at least
/// one goal misses its target (GH #213; `2`, the `audit` gate code,
/// leaving `1` for usage errors), else `0`.
pub fn cmd_metrics(elements: &[RawElement], json_out: bool, with_sensitivity: bool, model_root: &std::path::Path) -> i32 {
    let resolver = Resolver::new(elements);
    let (cfg, defects) = ReliabilityConfig::load(model_root);
    for d in &defects {
        eprintln!("warning: .syscribe.toml: {d} — entry ignored (W989)");
    }
    let mut reports = report_all_with(elements, &resolver, &cfg);
    reports.sort_by(|a, b| a.id.cmp(&b.id));
    let sens: Vec<(String, Vec<syscribe_model::metrics::Sensitivity>)> = if with_sensitivity {
        reports
            .iter()
            .filter(|r| r.figure.is_some())
            .filter_map(|r| elements.iter().find(|e| e.file_path == r.file_path && syscribe_model::resolver::Resolver::is_safety_goal(e)).map(|g| (r.id.clone(), sensitivity(g, elements, &resolver, &cfg))))
            .collect()
    } else {
        Vec::new()
    };

    if json_out {
        emit_json(&reports, &sens, &cfg);
    } else {
        emit_text(&reports, &sens, &cfg);
    }
    let failing: Vec<&str> = reports.iter().filter(|r| r.verdict() == Verdict::Fail && r.gates()).map(|r| r.id.as_str()).collect();
    let advisory: Vec<&str> = reports.iter().filter(|r| r.verdict() == Verdict::Fail && !r.gates()).map(|r| r.id.as_str()).collect();
    if !advisory.is_empty() {
        eprintln!("metrics: {} goal(s) miss the default limit on a figure without coverage data (advisory until the goal declares `target:` or `.syscribe.toml` sets the limit): {}", advisory.len(), advisory.join(", "));
    }
    if failing.is_empty() {
        0
    } else {
        eprintln!("metrics: {} goal(s) miss their target: {}", failing.len(), failing.join(", "));
        2
    }
}

fn emit_json(reports: &[GoalReport], sens: &[(String, Vec<syscribe_model::metrics::Sensitivity>)], cfg: &ReliabilityConfig) {
    let arr: Vec<_> = reports
        .iter()
        .map(|r| {
            let m = r.metrics.as_ref();
            let mut o = json!({
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
                "mode": r.mode.label(),
                "figure": r.figure.map(|f| json!({"metric": f.metric, "value": f.value, "basis": f.basis.label()})),
                "target": r.limit().map(|v| json!({"value": v, "source": r.target_source.map(|s| s.label())})),
                "rateOnly": r.rate_only(),
                "topEventProbability": r.top_probability,
                "missionHours": r.mission_hours,
                "unverifiedRates": r.unverified_rates,
                "notCredible": r.implausible.iter().map(|(e, p)| json!({"event": e, "probability": p})).collect::<Vec<_>>(),
                "gates": r.gates(),
                "targetIgnored": r.target_ignored,
                "largestContributor": r.largest_contributor.as_ref().map(|(e, fv)| json!({"event": e, "fussellVesely": fv})),
                "highestBirnbaum": r.dominant.as_ref().map(|(e, b)| json!({"event": e, "birnbaum": b})),
            });
            if let Some((_, s)) = sens.iter().find(|(id, _)| id == &r.id) {
                o["sensitivity"] = json!(s
                    .iter()
                    .map(|x| json!({"event": x.event, "birnbaum": x.birnbaum, "figureDown": x.figure_down, "figureUp": x.figure_up, "verdictDown": x.verdict_down.label(), "verdictUp": x.verdict_up.label()}))
                    .collect::<Vec<_>>());
            }
            o
        })
        .collect();
    let _ = cfg;
    println!("{}", serde_json::to_string_pretty(&json!(arr)).unwrap());
}

fn emit_text(reports: &[GoalReport], sens: &[(String, Vec<syscribe_model::metrics::Sensitivity>)], cfg: &ReliabilityConfig) {
    println!("# Quantitative safety targets (ISO 26262-5 §8-9, IEC 61508)\n");
    println!(
        "> SPFM/LFM/PMHF are a first-order FMEDA approximation from user-supplied λ and diagnostic coverage; a goal without \
         coverage data is judged on a rate-only figure from its fault tree's exact top event. Verify independently before \
         use in a safety case.\n"
    );

    if reports.is_empty() {
        println!("No SafetyGoal elements found.");
        return;
    }

    println!("| SafetyGoal | ASIL | SIL | Mode | SPFM | LFM | Figure | Target | Basis | Verdict |");
    println!("|---|---|---|---|---|---|---|---|---|---|");
    for r in reports {
        let asil = r.asil.clone().unwrap_or_else(|| "—".to_string());
        let sil = r.sil.map(|s| s.to_string()).unwrap_or_else(|| "—".to_string());
        let (spfm, lfm) = match r.metrics.as_ref() {
            Some(m) => (fmt_opt(m.spfm, 4), fmt_opt(m.lfm, 4)),
            None => ("n/a".to_string(), "n/a".to_string()),
        };
        let unit = |metric: &str| if metric == "PFDavg" { "" } else { " /h" };
        let figure = r.figure.map(|f| format!("{} {}{}", f.metric, fmt_e(Some(f.value)), unit(f.metric))).unwrap_or_else(|| "n/a".into());
        let target = match (r.limit(), r.target_source) {
            (Some(v), Some(s)) => format!("{v:.0e} ({})", s.label()),
            _ => "—".into(),
        };
        let basis = r.figure.map(|f| f.basis.label()).unwrap_or("—");
        let mut verdict = r.verdict().label().to_string();
        if r.figure.is_some() && !r.unverified_rates.is_empty() {
            verdict.push_str(" (unverified rates)");
        }
        println!("| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |", r.id, asil, sil, r.mode.label(), spfm, lfm, figure, target, basis, verdict);
    }

    for r in reports.iter().filter(|r| r.figure.is_some()) {
        let mut notes = Vec::new();
        if let (Some(p), Some(t)) = (r.top_probability, r.mission_hours) {
            notes.push(format!("top-event probability {p:.3e} over {t:.0} h"));
        }
        if let Some((e, fv)) = &r.largest_contributor {
            notes.push(format!("largest contributor {e} (Fussell-Vesely {fv:.3})"));
        }
        if let Some((e, b)) = r.dominant.as_ref().filter(|(e, _)| r.largest_contributor.as_ref().is_none_or(|(c, _)| c != e)) {
            notes.push(format!("highest Birnbaum importance {e} ({b:.3e})"));
        }
        if r.target_ignored {
            notes.push("its `target:` is malformed (E985) and was ignored: judged against the defaults".into());
        }
        if r.mode == syscribe_model::metrics::Mode::Demand && r.limit().is_none() {
            notes.push("no PFDavg limit for its level: set `target.value` or `[reliability.targets.<scheme>.<level>] pfd`".into());
        }
        if r.verdict() == Verdict::Fail && !r.gates() {
            notes.push("advisory: a figure without coverage data gates only once the goal declares `target:` or `.syscribe.toml` sets the limit".into());
        }
        if r.verdict() == Verdict::NotCredible {
            let list: Vec<String> = r.implausible.iter().take(6).map(|(e, p)| format!("{e} {p:.2}")).collect();
            notes.push(format!(
                "not credible: {} above the plausibility bound {}{}: {}{}",
                if r.mode == syscribe_model::metrics::Mode::Demand { "λ_DU · proof-test interval (or PFDavg)" } else { "probability over the mission time" },
                cfg.plausible_probability,
                if cfg.plausibility_configured { "" } else { " (built-in default)" },
                list.join(", "),
                if r.implausible.len() > 6 { ", …" } else { "" }
            ));
        }
        if !r.unverified_rates.is_empty() {
            notes.push(format!("rates not verified (`rateStatus`): {}", r.unverified_rates.join(", ")));
        }
        if !notes.is_empty() {
            println!("\n**{}**: {}.", r.id, notes.join("; "));
        }
    }
    for (id, s) in sens {
        if s.is_empty() {
            continue;
        }
        println!("\n## Sensitivity of {id} (each event's λ ÷10 and ×10)\n");
        println!("| Event | Birnbaum | Figure ÷10 | Figure ×10 | Verdict ÷10 | Verdict ×10 |");
        println!("|---|---|---|---|---|---|");
        let base = reports.iter().find(|r| &r.id == id).map(|r| r.verdict());
        for x in s {
            let moves = base.is_some_and(|b| x.verdict_down != b || x.verdict_up != b);
            println!(
                "| {}{} | {} | {} | {} | {} | {} |",
                x.event,
                if moves { " (moves the verdict)" } else { "" },
                fmt_e(x.birnbaum),
                fmt_e(x.figure_down),
                fmt_e(x.figure_up),
                x.verdict_down.label(),
                x.verdict_up.label()
            );
        }
    }

    if reports.iter().any(|r| r.figure.is_none()) {
        println!("\n_Goals showing `n/a` have no FaultTree, no failure rates, or no `missionTime` to turn a rate into a figure._");
    }
    if reports.iter().any(|r| r.verdict() == Verdict::NoTarget) {
        println!("\n_`no target`: a figure is computed but the goal has no recognised `asilLevel`/`silLevel` and no `target:`._");
    }
    if reports.iter().any(|r| r.target_source == Some(syscribe_model::metrics::TargetSource::Default) && r.figure.is_some()) {
        println!("\n_`built-in default` targets are the ISO 26262 / IEC 61508 values; they are a compliance decision, so set your own in `[reliability.targets]` of `.syscribe.toml` or in the goal's `target:`._");
    }
    for r in reports {
        for e in &r.analysis_errors {
            println!("\n> {}: fault-tree analysis failed — {e}", r.id);
        }
    }
    println!("\n_Exit status is 2 when a goal fails on a gating figure: the first-order roll-up always gates; a rate-only or PFDavg figure only once the goal declares `target:` or `.syscribe.toml` sets the limit._");
}

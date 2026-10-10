//! `results runs` / `results diff` — retained run history (GH #258, REQ-TRS-RUNHIST-001).

use serde_json::json;
use std::path::Path;
use syscribe_model::results::{RunDiff, RunHistory, TestChange, Verdict};

const USAGE: &str = "Usage: syscribe --model <root> results runs [--json] | results failures [--json] | results diff <runA> <runB> [--json] [--fail-on-regression]";

fn v(x: Option<Verdict>) -> &'static str {
    match x {
        Some(Verdict::Pass) => "pass",
        Some(Verdict::Fail) => "fail",
        Some(Verdict::Ignored) => "ignored",
        Some(Verdict::Flaky) => "flaky",
        None => "absent",
    }
}

fn section(title: &str, items: &[TestChange], out: &mut String) {
    out.push_str(&format!("{title} ({})\n", items.len()));
    for c in items {
        out.push_str(&format!("  {}  {} -> {}\n", c.test, v(c.from), v(c.to)));
    }
}

fn render(d: &RunDiff) -> String {
    let mut s = String::new();
    section("Regressions", &d.regressions, &mut s);
    section("Fixed", &d.fixed, &mut s);
    section("Still failing", &d.still_failing, &mut s);
    section("Other changes", &d.other_changes, &mut s);
    s
}

fn change_json(c: &TestChange) -> serde_json::Value {
    json!({"test": c.test, "from": v(c.from), "to": v(c.to)})
}

/// `results failures`: the non-passing functions of the latest ingest with retained message/time.
fn failures(model_root: &Path, json_out: bool) -> i32 {
    let Some(data) = syscribe_model::results::ResultsData::load_sidecar(model_root) else {
        eprintln!("results failures: no results sidecar (run `ingest-results` first).");
        return 1;
    };
    let mut rows: Vec<(&String, Verdict)> = data
        .by_leaf
        .iter()
        .filter(|(k, v)| !k.contains("::") && **v != Verdict::Pass)
        .map(|(k, v)| (k, *v))
        .collect();
    rows.sort_by(|a, b| a.0.cmp(b.0));
    if json_out {
        let list: Vec<_> = rows
            .iter()
            .map(|(k, vd)| {
                let d = data.details.get(*k);
                json!({"function": k, "verdict": v(Some(*vd)), "message": d.and_then(|d| d.message.clone()), "time": d.and_then(|d| d.time)})
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json!({ "failures": list })).unwrap_or_default());
    } else if rows.is_empty() {
        println!("No failing, skipped or flaky functions.");
    } else {
        for (k, vd) in rows {
            let d = data.details.get(k);
            let time = d.and_then(|d| d.time).map(|t| format!("  {t}s")).unwrap_or_default();
            let msg = d.and_then(|d| d.message.as_deref()).map(|m| format!("  — {m}")).unwrap_or_default();
            println!("{k}  {}{time}{msg}", v(Some(vd)));
        }
    }
    0
}

/// Entry point for `results …`. Returns the process exit code.
pub fn cmd_results(model_root: &Path, args: &[String]) -> i32 {
    let json_out = args.iter().any(|a| a == "--json");
    let gate = args.iter().any(|a| a == "--fail-on-regression");
    if let Some(bad) = args.iter().skip(1).find(|a| a.starts_with("--") && !["--json", "--fail-on-regression"].contains(&a.as_str())) {
        eprintln!("results: unknown option '{bad}'\n{USAGE}");
        return 1;
    }
    let pos: Vec<&str> = args.iter().map(|s| s.as_str()).filter(|a| !a.starts_with("--")).collect();
    let history = match RunHistory::load(model_root) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    match pos.as_slice() {
        ["runs"] => {
            if json_out {
                let runs: Vec<_> = history
                    .runs
                    .iter()
                    .map(|r| {
                        json!({
                            "run": r.run,
                            "ingestedAtUnix": r.ingested_at_unix,
                            "functions": r.by_leaf.len(),
                            "scenarios": r.by_scenario.len(),
                            "sources": r.sources,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json!({ "runs": runs })).unwrap_or_default());
            } else if history.runs.is_empty() {
                println!("No runs retained (ingest with --run <id>).");
            } else {
                for r in &history.runs {
                    println!("{}  at {}  {} function(s), {} scenario(s)", r.run, r.ingested_at_unix, r.by_leaf.len(), r.by_scenario.len());
                }
            }
            0
        }
        ["failures"] => failures(model_root, json_out),
        ["diff", a, b] => {
            let (Some(ra), Some(rb)) = (history.get(a), history.get(b)) else {
                let missing = if history.get(a).is_none() { a } else { b };
                eprintln!("results diff: run '{missing}' is not retained (see `results runs`).");
                return 1;
            };
            let d = ra.diff(rb);
            if json_out {
                let list = |x: &[TestChange]| x.iter().map(change_json).collect::<Vec<_>>();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "from": a, "to": b,
                        "regressions": list(&d.regressions),
                        "fixed": list(&d.fixed),
                        "stillFailing": list(&d.still_failing),
                        "otherChanges": list(&d.other_changes),
                    }))
                    .unwrap_or_default()
                );
            } else {
                print!("{}", render(&d));
            }
            if gate && (!d.regressions.is_empty() || !d.vanished_failures().is_empty()) {
                1
            } else {
                0
            }
        }
        _ => {
            eprintln!("{USAGE}");
            1
        }
    }
}

//! `results runs` / `results diff` — retained run history (GH #258, REQ-TRS-RUNHIST-001).

use serde_json::json;
use std::path::Path;
use syscribe_model::results::{RunDiff, RunHistory, TestChange, Verdict};

static AS_OF: std::sync::OnceLock<syscribe_model::results::ResultsData> = std::sync::OnceLock::new();

/// Activate `--results-as-of <run>`: every later [`load_results`] returns that retained run.
pub fn set_as_of(model_root: &Path, run: &str) -> Result<(), String> {
    let history = RunHistory::load(model_root)?;
    let Some(rec) = history.get(run) else {
        let have: Vec<&str> = history.runs.iter().map(|r| r.run.as_str()).collect();
        return Err(format!(
            "--results-as-of: run '{run}' is not retained ({}). Retain a run with `ingest-results --run <id>`.",
            if have.is_empty() { "no runs retained".to_string() } else { format!("retained: {}", have.join(", ")) }
        ));
    };
    let _ = AS_OF.set(rec.to_results());
    Ok(())
}

static LENS_CONFIG: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Judge every later [`load_results`] as Configuration `config` (its id) sees the evidence: the
/// global results overlaid with that configuration's own (GH #258, REQ-TRS-CFGRES-001).
pub fn set_lens_config(config: &str) {
    let _ = LENS_CONFIG.set(config.to_string());
}

/// The id a Configuration is stored under (`ingest-results --config`): its stable id, else its
/// qualified name. `None` when `config` names no Configuration of the model.
pub fn config_key(elems: &[syscribe_model::element::RawElement], config: &str) -> Option<String> {
    elems
        .iter()
        .find(|e| {
            matches!(e.frontmatter.element_type, Some(syscribe_model::element::ElementType::Configuration))
                && (e.frontmatter.id.as_deref() == Some(config) || e.qualified_name == config)
        })
        .map(|e| e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone()))
}

/// Overlay the active `--config` lens, if any, on `data`.
pub fn lensed(data: syscribe_model::results::ResultsData) -> syscribe_model::results::ResultsData {
    match LENS_CONFIG.get() {
        Some(c) => data.for_config(c),
        None => data,
    }
}

/// The run `--results-as-of` selected, if any.
pub fn as_of() -> Option<&'static syscribe_model::results::ResultsData> {
    AS_OF.get()
}

/// The results the evidence-reading commands use: the selected retained run, else the sidecar.
pub fn load_results(model_root: &Path) -> Option<syscribe_model::results::ResultsData> {
    as_of().cloned().or_else(|| syscribe_model::results::ResultsData::load_sidecar(model_root)).map(lensed)
}

const USAGE: &str = "Usage: syscribe --model <root> results runs [--json] | results failures [--json] | results diff <runA> <runB> [--json] [--fail-on-regression] | results campaigns [--json] | results campaigns diff <subject[@config]> [<runA> <runB>] [--json]";

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
    let Some(data) = load_results(model_root) else {
        eprintln!("results failures: no results sidecar (run `ingest-results` first).");
        return 1;
    };
    // The global sections, then every configuration's own (named `<function> @ <config>`); under a
    // `--config` lens the loaded data is already that configuration's effective view.
    let mut layers: Vec<(Option<&str>, &syscribe_model::results::ResultsData)> = vec![(None, &data)];
    layers.extend(data.by_config.iter().map(|(c, d)| (Some(c.as_str()), d)));
    let mut rows: Vec<(String, Verdict, Option<&syscribe_model::results::TestDetail>)> = Vec::new();
    for (cfg, layer) in layers {
        for (k, vd) in layer.by_leaf.iter().filter(|(k, v)| !k.contains("::") && **v != Verdict::Pass) {
            let name = match cfg {
                Some(c) => format!("{k} @ {c}"),
                None => k.clone(),
            };
            rows.push((name, *vd, layer.details.get(k)));
        }
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    if json_out {
        let list: Vec<_> = rows
            .iter()
            .map(|(k, vd, d)| json!({"function": k, "verdict": v(Some(*vd)), "message": d.and_then(|d| d.message.clone()), "time": d.and_then(|d| d.time)}))
            .collect();
        println!("{}", serde_json::to_string_pretty(&json!({ "failures": list })).unwrap_or_default());
    } else if rows.is_empty() {
        println!("No failing, skipped or flaky functions.");
    } else {
        for (k, vd, d) in &rows {
            let time = d.and_then(|d| d.time).map(|t| format!("  {t}s")).unwrap_or_default();
            // One line per function: fold any line breaks of a multi-line assertion message.
            let msg = d
                .and_then(|d| d.message.as_deref())
                .map(|m| format!("  — {}", syscribe_model::results::one_line(m)))
                .unwrap_or_default();
            println!("{k}  {}{time}{msg}", v(Some(*vd)));
        }
    }
    0
}

/// Entry point for `results …`. Returns the process exit code.
pub fn cmd_results(model_root: &Path, args: &[String]) -> i32 {
    let json_out = args.iter().any(|a| a == "--json");
    let gate = args.iter().any(|a| a == "--fail-on-regression");
    if let Some(bad) = args.iter().skip(1).find(|a| a.starts_with("--") && !["--json", "--fail-on-regression", "--config"].contains(&a.as_str())) {
        eprintln!("results: unknown option '{bad}'\n{USAGE}");
        return 1;
    }
    // `--config <id>` (only `results failures` honours it, as the evidence lens set in main) takes a value.
    let mut skip_next = false;
    let pos: Vec<&str> = args
        .iter()
        .map(|s| s.as_str())
        .filter(|a| {
            if skip_next {
                skip_next = false;
                return false;
            }
            if *a == "--config" {
                skip_next = true;
                return false;
            }
            !a.starts_with("--")
        })
        .collect();
    if pos.as_slice() == ["failures"] {
        return failures(model_root, json_out);
    }
    if pos.first() == Some(&"campaigns") {
        return campaigns(model_root, &pos[1..], json_out);
    }
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
                            "configurations": r.by_config.iter().map(|(c, sec)| (c.clone(), json!({"functions": sec.by_leaf.len(), "scenarios": sec.by_scenario.len()}))).collect::<serde_json::Map<_, _>>(),
                            "sources": r.sources,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json!({ "runs": runs })).unwrap_or_default());
            } else if history.runs.is_empty() {
                println!("No runs retained (ingest with --run <id>).");
            } else {
                for r in &history.runs {
                    let cfgs = if r.by_config.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "; {}",
                            r.by_config.iter().map(|(c, s)| format!("{c}: {} function(s), {} scenario(s)", s.by_leaf.len(), s.by_scenario.len())).collect::<Vec<_>>().join("; ")
                        )
                    };
                    println!("{}  at {}  {} function(s), {} scenario(s){cfgs}", r.run, r.ingested_at_unix, r.by_leaf.len(), r.by_scenario.len());
                }
            }
            0
        }
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

/// `results campaigns` / `results campaigns diff`: the fault-injection campaigns `inject --record` kept (GH #341).
fn campaigns(model_root: &Path, pos: &[&str], json_out: bool) -> i32 {
    use syscribe_model::residual_risk::{self, CampaignRecord};
    let all = match residual_risk::load(model_root) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: the recorded campaigns cannot be read: {e}");
            return 1;
        }
    };
    let label = |r: &CampaignRecord| r.run.clone().unwrap_or_else(|| "(unnamed)".into());
    match pos {
        [] => {
            if json_out {
                let v: serde_json::Map<String, serde_json::Value> = all
                    .iter()
                    .map(|(k, l)| {
                        (k.clone(), l.iter().map(|r| json!({"run": r.run, "compositeHash": r.composite_hash, "sets": r.sets.len(), "survived": r.count("survived"), "degraded": r.sets.iter().filter(|s| s.verdict() == "survived (degraded)").count(), "defeated": r.count("defeated"), "errors": r.count("error"), "engine": r.engine, "recordedAtUnix": r.recorded_at_unix})).collect())
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json!({ "campaigns": v })).unwrap_or_default());
            } else if all.is_empty() {
                println!("No campaigns recorded (run `syscribe inject <subject> --record`).");
            } else {
                for (k, l) in &all {
                    println!("{k}");
                    for r in l {
                        let degraded = r.sets.iter().filter(|s| s.verdict() == "survived (degraded)").count();
                        let degraded = if degraded > 0 { format!(" ({degraded} only degraded)") } else { String::new() };
                        println!("  {:<14} at {}  hash {}  {} set(s): {} survived{degraded}, {} defeated, {} could not be run", label(r), r.recorded_at_unix, &r.composite_hash[..12.min(r.composite_hash.len())], r.sets.len(), r.count("survived"), r.count("defeated"), r.count("error"));
                    }
                }
            }
            0
        }
        ["diff", key, rest @ ..] => {
            let Some(list) = all.get(*key) else {
                eprintln!("results campaigns diff: no campaign '{key}' is recorded (see `results campaigns`).");
                return 1;
            };
            let find = |run: &str| list.iter().find(|r| r.run.as_deref() == Some(run) || (run == "(unnamed)" && r.run.is_none()));
            let (a, b) = match rest {
                [] if list.len() >= 2 => (&list[1], &list[0]),
                [] => {
                    eprintln!("results campaigns diff: '{key}' has only one record; name two runs or record another");
                    return 1;
                }
                [x, y] => match (find(x), find(y)) {
                    (Some(a), Some(b)) => (a, b),
                    (a, _) => {
                        eprintln!("results campaigns diff: run '{}' of '{key}' is not retained", if a.is_none() { x } else { y });
                        return 1;
                    }
                },
                _ => {
                    eprintln!("{USAGE}");
                    return 1;
                }
            };
            let d = residual_risk::diff(a, b);
            if json_out {
                println!("{}", serde_json::to_string_pretty(&json!({"from": label(a), "to": label(b), "diff": d})).unwrap_or_default());
            } else {
                println!("{key}: {} → {}", label(a), label(b));
                if d.composite_changed {
                    println!("  the composite changed ({} → {})", &a.composite_hash[..12.min(a.composite_hash.len())], &b.composite_hash[..12.min(b.composite_hash.len())]);
                }
                if d.invariants_changed {
                    println!("  the invariants changed");
                }
                for (s, x, y) in &d.changed {
                    println!("  {s}: {x} → {y}");
                }
                for (s, y) in &d.added {
                    println!("  {s}: new, {y}");
                }
                for (s, x) in &d.removed {
                    println!("  {s}: no longer run (was {x})");
                }
                if d.changed.is_empty() && d.added.is_empty() && d.removed.is_empty() {
                    println!("  no verdict changed");
                }
            }
            0
        }
        _ => {
            eprintln!("{USAGE}");
            1
        }
    }
}

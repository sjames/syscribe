//! `syscribe trace-code` — code that no design element or TestCase names, against a shrink-only
//! baseline (GH #451, `REQ-TRS-TRACECODE-001..004`). The analysis lives in
//! [`syscribe_model::trace_code`]; this module is the command, its JSON and the `[repos]` walk.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use syscribe_model::config::ValidateConfig;
use syscribe_model::element::RawElement;
use syscribe_model::trace_code::{self, Report, TraceCodeConfig};

pub const USAGE: &str = "Usage: syscribe -m <root> trace-code [--root <repo>] [--json] [--all] [--write-baseline]";

/// How to switch the command on, for the "no `[traceCode]` table" error.
pub const HOW_TO_CONFIGURE: &str = "add a [traceCode] table to <model_root>/.syscribe.toml, e.g.\n\n\
    [traceCode]\n\
    roots = [\"crates\", \"scripts\"]\n\
    units = [\n\
    \x20 { kind = \"cargo-crate\", under = \"crates\" },\n\
    \x20 { kind = \"file\", under = \"scripts\", extensions = [\"sh\", \"py\"] },\n\
    ]\n\n\
    then record today's backlog with `syscribe -m <root> trace-code --write-baseline` (see `syscribe help trace-code`)";

/// One repository's outcome.
pub struct RepoOutcome {
    pub alias: Option<String>,
    pub model_root: PathBuf,
    /// `Ok(report)`, or why this repository was not checked (`Err(note)`; `fatal` says whether that
    /// is a configuration error).
    pub result: Result<Report, (String, bool)>,
}

/// Analyse the local model's repository. `Err` is a configuration error (exit 1).
pub fn analyse_local(model_root: &Path, elements: &[RawElement], config: &ValidateConfig, root: Option<&Path>) -> Result<Report, String> {
    let Some(cfg) = TraceCodeConfig::load(model_root) else {
        return Err(format!("trace-code: no [traceCode] table in {} — {HOW_TO_CONFIGURE}", model_root.join(".syscribe.toml").display()));
    };
    let repo_root = trace_code::repo_root_for(model_root, config, root);
    run_one(model_root, elements, config, &cfg, &repo_root)
}

fn run_one(model_root: &Path, elements: &[RawElement], config: &ValidateConfig, cfg: &TraceCodeConfig, repo_root: &Path) -> Result<Report, String> {
    let mut problems = cfg.problems.clone();
    let (report, root_problems) = trace_code::analyse(model_root, elements, config, cfg, repo_root).map_err(|e| format!("trace-code: {e}"))?;
    problems.extend(root_problems);
    if !problems.is_empty() {
        return Err(format!(
            "trace-code: [traceCode] in {} has problems (W328):\n  {}",
            model_root.join(".syscribe.toml").display(),
            problems.join("\n  ")
        ));
    }
    Ok(report)
}

/// Each `[repos]` peer, checked against its own elements when it has its own `[traceCode]` table.
pub fn analyse_peers(config: &ValidateConfig) -> Vec<RepoOutcome> {
    let mut out = Vec::new();
    for repo in &config.repos {
        let alias = Some(repo.alias.clone());
        let model_root = repo.model_root.clone();
        let result = if !repo.exists {
            Err((format!("not on disk at {} (see `syscribe repos status`); not checked", model_root.display()), false))
        } else {
            match TraceCodeConfig::load(&model_root) {
                None => Err(("no [traceCode] table in its .syscribe.toml; not checked".to_string(), false)),
                Some(cfg) => match syscribe_model::walker::walk_model(&model_root) {
                    Err(e) => Err((format!("cannot load its model: {e}"), true)),
                    Ok(elems) => {
                        let pcfg = ValidateConfig::paths_only(&model_root);
                        let repo_root = trace_code::repo_root_for(&model_root, &pcfg, None);
                        run_one(&model_root, &elems, &pcfg, &cfg, &repo_root).map_err(|e| (e, true))
                    }
                },
            }
        };
        out.push(RepoOutcome { alias, model_root, result });
    }
    out
}

fn report_json(r: &Report, all: bool) -> serde_json::Map<String, Value> {
    let mut m = serde_json::Map::new();
    m.insert("repoRoot".into(), json!(r.repo_root.display().to_string()));
    m.insert("baseline".into(), json!(r.baseline_path.display().to_string()));
    m.insert(
        "untraced".into(),
        Value::Array(r.untraced.iter().map(|u| json!({"path": u.path, "unit": u.kind.as_str(), "inBaseline": u.in_baseline})).collect()),
    );
    m.insert("stale".into(), Value::Array(r.stale.iter().map(|s| json!({"path": s.path, "reason": s.reason, "why": s.why})).collect()));
    if all {
        m.insert(
            "traced".into(),
            Value::Array(
                r.traced
                    .iter()
                    .map(|t| {
                        json!({
                            "path": t.path,
                            "unit": t.kind.as_str(),
                            "namedBy": t.named_by.iter().map(|(e, v)| json!({"element": e, "via": v})).collect::<Vec<_>>(),
                        })
                    })
                    .collect(),
            ),
        );
    }
    m
}

/// The JSON document (`trace-code --json`, MCP `trace_code`).
pub fn to_json(local: &Report, peers: &[RepoOutcome], all: bool) -> Value {
    let mut m = serde_json::Map::new();
    m.insert("schemaVersion".into(), json!("1.0"));
    m.extend(report_json(local, all));
    if !peers.is_empty() {
        let list: Vec<Value> = peers
            .iter()
            .map(|p| {
                let mut o = serde_json::Map::new();
                o.insert("alias".into(), json!(p.alias));
                o.insert("modelRoot".into(), json!(p.model_root.display().to_string()));
                match &p.result {
                    Ok(r) => {
                        o.insert("checked".into(), json!(true));
                        o.extend(report_json(r, all));
                    }
                    Err((note, _)) => {
                        o.insert("checked".into(), json!(false));
                        o.insert("note".into(), json!(note));
                    }
                }
                Value::Object(o)
            })
            .collect();
        m.insert("peers".into(), Value::Array(list));
    }
    Value::Object(m)
}

fn print_report(title: &str, r: &Report, all: bool) {
    println!("# trace-code: {title}\n");
    println!("Repository: {}", r.repo_root.display());
    println!("Baseline:   {}\n", r.baseline_path.display());
    let new = r.new_untraced().count();
    if r.untraced.is_empty() {
        println!("Every code unit is named by an implementedBy: or sourceFile:.");
    } else {
        println!("Untraced code units ({}, {} new):", r.untraced.len(), new);
        for u in &r.untraced {
            let tag = if u.in_baseline {
                let reason = r.baseline.iter().find(|e| e.path == u.path).map(|e| e.reason.as_str()).unwrap_or("");
                format!("baselined: {reason}")
            } else {
                "NEW".to_string()
            };
            println!("  {}  [{}]  {}", u.path, u.kind.as_str(), tag);
        }
    }
    if !r.stale.is_empty() {
        println!("\nStale baseline entries ({}) — the baseline only shrinks:", r.stale.len());
        for s in &r.stale {
            let why = match s.why {
                "traced" => "now traced",
                "missing" => "no longer exists",
                _ => "no longer a code unit",
            };
            println!("  {}  — {} (reason was: {})", s.path, why, s.reason);
        }
    }
    if all && !r.traced.is_empty() {
        println!("\nTraced code units ({}):", r.traced.len());
        for t in &r.traced {
            // The text form names the first few; `--json` carries every one.
            const SHOWN: usize = 5;
            let mut by: Vec<String> = t.named_by.iter().take(SHOWN).map(|(e, v)| format!("{e} ({v})")).collect();
            if t.named_by.len() > SHOWN {
                by.push(format!("and {} more", t.named_by.len() - SHOWN));
            }
            println!("  {}  [{}]  ← {}", t.path, t.kind.as_str(), by.join(", "));
        }
    }
    println!();
}

/// `args` are the tokens after `trace-code`. Returns the exit code: 0 clean, 1 usage/config error, 2
/// a new untraced unit or a stale baseline entry.
pub fn cmd_trace_code(model_root: &Path, elements: &[RawElement], config: &ValidateConfig, args: &[String]) -> i32 {
    let json_out = args.iter().any(|a| a == "--json");
    let all = args.iter().any(|a| a == "--all");
    let write = args.iter().any(|a| a == "--write-baseline");
    let root = args.windows(2).find(|w| w[0] == "--root").map(|w| PathBuf::from(&w[1]));
    if let Some(stray) = args.iter().enumerate().find(|(i, a)| !a.starts_with("--") && (*i == 0 || args[i - 1] != "--root")).map(|(_, a)| a) {
        eprintln!("trace-code: unexpected argument '{stray}'\n{USAGE}");
        return 1;
    }
    if let Some(r) = &root {
        if !r.is_dir() {
            eprintln!("trace-code: --root {} is not a directory\n{USAGE}", r.display());
            return 1;
        }
    }
    let local = match analyse_local(model_root, elements, config, root.as_deref()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };

    if write {
        let entries = local.refreshed_baseline();
        let dropped = local.stale.len();
        let added = local.new_untraced().count();
        if let Err(e) = syscribe_model::fsio::write(&local.baseline_path, trace_code::render_baseline(&entries)) {
            eprintln!("trace-code: cannot write {}: {e}", local.baseline_path.display());
            return 1;
        }
        let todo = entries.iter().filter(|e| e.reason.trim() == "TODO" || e.reason.trim().is_empty()).count();
        if json_out {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"schemaVersion": "1.0", "baseline": local.baseline_path.display().to_string(), "entries": entries.len(), "added": added, "dropped": dropped, "todo": todo, "written": !syscribe_model::fsio::is_dry_run()})).unwrap()
            );
        } else {
            println!(
                "{} {} — {} entr{} ({} added, {} stale dropped).",
                syscribe_model::fsio::wrote(),
                local.baseline_path.display(),
                entries.len(),
                if entries.len() == 1 { "y" } else { "ies" },
                added,
                dropped
            );
            if todo > 0 {
                println!("{todo} entr{} TODO: replace each with the real reason the code has no design element or test yet.", if todo == 1 { "y says" } else { "ies say" });
            }
        }
        return 0;
    }

    let peers = analyse_peers(config);
    let fatal = peers.iter().any(|p| matches!(p.result, Err((_, true))));
    let dirty = !local.clean() || peers.iter().any(|p| p.result.as_ref().is_ok_and(|r| !r.clean()));
    let code = if fatal {
        1
    } else if dirty {
        2
    } else {
        0
    };

    if json_out {
        println!("{}", serde_json::to_string_pretty(&to_json(&local, &peers, all)).unwrap());
        for p in &peers {
            if let Err((e, true)) = &p.result {
                eprintln!("trace-code: peer {}: {e}", p.alias.as_deref().unwrap_or("?"));
            }
        }
        return code;
    }
    print_report("this repository", &local, all);
    for p in &peers {
        let alias = p.alias.as_deref().unwrap_or("?");
        match &p.result {
            Ok(r) => print_report(&format!("[repos] peer `{alias}` (against its own elements)"), r, all),
            Err((note, fatal)) => {
                if *fatal {
                    eprintln!("trace-code: peer {alias}: {note}");
                } else {
                    println!("# trace-code: [repos] peer `{alias}` — {note}\n");
                }
            }
        }
    }
    if code == 2 {
        let new: usize = std::iter::once(&local).chain(peers.iter().filter_map(|p| p.result.as_ref().ok())).map(|r| r.new_untraced().count()).sum();
        let stale: usize = std::iter::once(&local).chain(peers.iter().filter_map(|p| p.result.as_ref().ok())).map(|r| r.stale.len()).sum();
        println!(
            "FAIL: {new} new untraced unit(s), {stale} stale baseline entr{}. For each NEW unit add the design element (implementedBy:) or TestCase (sourceFile:) that names it, or record it with a reason; drop stale entries — both with `syscribe -m {} trace-code --write-baseline` (peers: run it in the peer's own model).",
            if stale == 1 { "y" } else { "ies" },
            model_root.display()
        );
    } else if code == 0 {
        println!("OK: no new untraced code and no stale baseline entries.");
    }
    code
}

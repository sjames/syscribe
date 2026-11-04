//! `health` — one cheap call summarising the project for a status-bar item (GH #428, `REQ-TRS-HEALTH-001`).
//!
//! Error and warning counts, orphan and unverified requirements, suspect links and stale results, and the number of
//! Configurations whose variant fails validation, each with the command that lists the detail so a status bar can
//! click through. One validation run feeds every count; the time it took is reported (`elapsedMs`).
//!
//! Parameter values files (`params export`) are not registered in the model, so the number of stale ones cannot be
//! counted here; `params check <FILE> --config <C>` checks one.

use std::collections::BTreeMap;
use std::time::Instant;

use serde_json::{json, Value};
use syscribe_model::config::ValidateConfig;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::results::ResultsData;
use syscribe_model::validator::{validate_with_config, Severity};

pub fn compute(elements: &[RawElement], config: &ValidateConfig, results: Option<&ResultsData>) -> Value {
    let started = Instant::now();
    let findings = validate_with_config(elements, config).findings;
    let errors = findings.iter().filter(|f| matches!(f.severity, Severity::Error)).count();
    let warnings = findings.len() - errors;
    let count = |code: &str| findings.iter().filter(|f| f.code == code).count();

    // The most frequent codes, so the status bar can say what dominates.
    let mut by_code: BTreeMap<&str, usize> = BTreeMap::new();
    for f in &findings {
        *by_code.entry(f.code).or_default() += 1;
    }
    let mut top: Vec<(&str, usize)> = by_code.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let top: Vec<Value> = top.into_iter().take(5).map(|(c, n)| json!({ "code": c, "count": n })).collect();

    let rollup = crate::matrix::requirement_rollup(elements, results);
    let requirements = elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Requirement)).count();
    let unverified = rollup.values().filter(|v| **v == "unverified").count();

    // Configurations whose variant has an error.
    let configs: Vec<&RawElement> = elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration)).collect();
    let fm = syscribe_model::feature_model::check_feature_model(elements);
    let failing = configs
        .iter()
        .filter(|c| {
            let sel = syscribe_model::projection::canonical_selection(elements, c);
            let projected = syscribe_model::projection::validate_projected(elements, config, &sel).iter().any(|f| matches!(f.severity, Severity::Error));
            projected || fm.iter().any(|f| matches!(f.severity, Severity::Error) && f.file == c.file_path)
        })
        .count();

    let click = |metric: &str, n: usize, command: &str| json!({ "metric": metric, "count": n, "command": command });
    let mut click_through = Vec::new();
    if errors > 0 {
        click_through.push(click("errors", errors, "syscribe validate"));
    }
    if warnings > 0 {
        click_through.push(click("warnings", warnings, "syscribe validate"));
    }
    if unverified > 0 {
        click_through.push(click("unverifiedRequirements", unverified, "syscribe requirement-coverage"));
    }
    if count("W090") > 0 {
        click_through.push(click("suspectLinks", count("W090"), "syscribe suspect list"));
    }
    if failing > 0 {
        click_through.push(click("failingConfigurations", failing, "syscribe validate --all-configs"));
    }

    let status = if errors > 0 || failing > 0 { "error" } else if warnings > 0 || unverified > 0 { "warning" } else { "ok" };
    json!({
        "schemaVersion": "1.0",
        "status": status,
        "errors": errors,
        "warnings": warnings,
        "topCodes": top,
        "requirements": { "total": requirements, "orphan": count("W005"), "unverified": unverified },
        "staleEvidence": { "suspectLinks": count("W090"), "staleResults": count("W091") },
        "configurations": { "total": configs.len(), "failingValidation": failing },
        "clickThrough": click_through,
        "elapsedMs": started.elapsed().as_millis() as u64,
    })
}

/// [`compute`], remembered per model version: a status bar polls, and the answer only changes when the model does.
/// `version` identifies the loaded model (a reload counter, a file fingerprint); the result says if it was `cached`.
pub fn cached(cache: &std::sync::Mutex<Option<(u64, Value)>>, version: u64, compute_fresh: impl FnOnce() -> Value) -> Value {
    if let Some((v, doc)) = cache.lock().ok().and_then(|g| g.clone()) {
        if v == version {
            let mut doc = doc;
            doc["cached"] = json!(true);
            return doc;
        }
    }
    let mut doc = compute_fresh();
    doc["cached"] = json!(false);
    if let Ok(mut g) = cache.lock() {
        *g = Some((version, doc.clone()));
    }
    doc
}

pub fn cmd_health(elements: &[RawElement], config: &ValidateConfig, results: Option<&ResultsData>, args: &[String]) -> i32 {
    if let Some(bad) = args.iter().find(|a| a.starts_with('-') && *a != "--json") {
        eprintln!("health: unknown option '{bad}'");
        return 2;
    }
    let out = compute(elements, config, results);
    if args.iter().any(|a| a == "--json") {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return 0;
    }
    let r = &out["requirements"];
    println!("# Project health: {}\n", out["status"].as_str().unwrap_or(""));
    println!("- {} error(s), {} warning(s)", out["errors"], out["warnings"]);
    println!("- requirements: {} total, {} orphan, {} unverified", r["total"], r["orphan"], r["unverified"]);
    println!("- stale evidence: {} suspect link(s), {} stale result(s)", out["staleEvidence"]["suspectLinks"], out["staleEvidence"]["staleResults"]);
    println!("- configurations: {} total, {} failing validation", out["configurations"]["total"], out["configurations"]["failingValidation"]);
    for c in out["clickThrough"].as_array().into_iter().flatten() {
        println!("  → {} ({}): `{}`", c["metric"].as_str().unwrap_or(""), c["count"], c["command"].as_str().unwrap_or(""));
    }
    println!("\n({} ms)", out["elapsedMs"]);
    0
}

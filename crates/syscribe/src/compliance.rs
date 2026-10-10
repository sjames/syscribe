//! `compliance --standard <name>` — which expected work products of a standard the model has and
//! has approved (GH #239, REQ-TRS-COMPLY-001). A small built-in process-area table per standard,
//! replaceable through `[standards.<name>]` in `.syscribe.toml`.

use serde_json::json;
use std::path::Path;
use syscribe_model::element::RawElement;

/// One expected work product of a process area, with the selector that finds it in the model.
#[derive(Debug, Clone)]
struct Item {
    process: String,
    work_product: String,
    etype: String,
    req_class: Option<String>,
    tags: Vec<String>,
    test_level: Option<String>,
}

const APPROVED: &[&str] = &["approved", "implemented", "verified", "active", "done", "completed", "accepted", "closed"];

fn it(process: &str, wp: &str, etype: &str, req_class: Option<&str>, test_level: Option<&str>) -> Item {
    Item {
        process: process.into(),
        work_product: wp.into(),
        etype: etype.into(),
        req_class: req_class.map(String::from),
        tags: Vec::new(),
        test_level: test_level.map(String::from),
    }
}

fn builtin(standard: &str) -> Option<Vec<Item>> {
    Some(match standard {
        "aspice" => vec![
            it("SYS.2", "System requirements", "Requirement", Some("system"), None),
            it("SYS.3", "System architecture", "PartDef", None, None),
            it("SWE.1", "Software requirements", "Requirement", Some("software"), None),
            it("SWE.2", "Software architecture decisions", "ADR", None, None),
            it("SWE.4", "Unit verification", "TestCase", None, Some("L1")),
            it("SWE.5", "Integration verification", "TestCase", None, Some("L3")),
            it("SYS.5", "System qualification verification", "TestCase", None, Some("L4")),
            it("SUP.4", "Joint review records", "ReviewRecord", None, None),
            it("SUP.8", "Configuration baselines", "Baseline", None, None),
        ],
        "iso26262" => vec![
            it("Part 3", "Hazard analysis and risk assessment (hazardous events)", "HazardousEvent", None, None),
            it("Part 3", "Safety goals", "SafetyGoal", None, None),
            it("Part 4", "Technical safety requirements", "Requirement", Some("system"), None),
            it("Part 5", "FMEA / FMEDA", "FMEASheet", None, None),
            it("Part 5", "Fault tree analysis", "FaultTree", None, None),
            it("Part 9", "Dependent failure analysis", "DependentFailureAnalysis", None, None),
            it("Part 2", "Confirmation measures", "ConfirmationMeasure", None, None),
            it("Part 8", "Verification specification (test cases)", "TestCase", None, None),
        ],
        "iso21434" => vec![
            it("Clause 15", "Threat analysis and risk assessment sheet", "TARASheet", None, None),
            it("Clause 15", "Damage scenarios", "DamageScenario", None, None),
            it("Clause 15", "Threat scenarios", "ThreatScenario", None, None),
            it("Clause 9", "Cybersecurity goals", "CybersecurityGoal", None, None),
            it("Clause 9", "Security controls", "SecurityControl", None, None),
            it("Clause 8", "Vulnerability reports", "VulnerabilityReport", None, None),
        ],
        _ => return None,
    })
}

/// The `[standards.<name>]` items of `.syscribe.toml`: `Ok(None)` when the standard is not
/// configured, `Err` for a malformed table.
fn configured(model_root: &Path, standard: &str) -> Result<Option<Vec<Item>>, String> {
    let Ok(text) = std::fs::read_to_string(model_root.join(".syscribe.toml")) else { return Ok(None) };
    let root: toml::Value = match toml::from_str(&text) {
        Ok(r) => r,
        Err(e) => {
            return if text.contains("standards") { Err(format!(".syscribe.toml does not parse: {}", e.message())) } else { Ok(None) };
        }
    };
    let Some(std_tbl) = root.get("standards") else { return Ok(None) };
    let Some(tbl) = std_tbl.as_table().and_then(|t| t.get(standard)) else { return Ok(None) };
    let items = tbl
        .get("item")
        .and_then(|i| i.as_array())
        .ok_or_else(|| format!("[standards.{standard}] needs `[[standards.{standard}.item]]` entries"))?;
    let mut out = Vec::new();
    for (n, i) in items.iter().enumerate() {
        let t = i.as_table().ok_or_else(|| format!("[[standards.{standard}.item]] #{} must be a table", n + 1))?;
        let s = |k: &str| t.get(k).and_then(|v| v.as_str()).map(String::from);
        let (Some(process), Some(wp), Some(etype)) = (s("process"), s("workProduct"), s("type")) else {
            return Err(format!("[[standards.{standard}.item]] #{} needs `process`, `workProduct` and `type`", n + 1));
        };
        let tags = match t.get("tag") {
            None => Vec::new(),
            Some(toml::Value::String(x)) => vec![x.clone()],
            Some(toml::Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(String::from)).collect(),
            Some(_) => return Err(format!("[[standards.{standard}.item]] #{}: `tag` must be a string or a list", n + 1)),
        };
        out.push(Item { process, work_product: wp, etype, req_class: s("reqClass"), tags, test_level: s("testLevel") });
    }
    Ok(Some(out))
}

fn matches(i: &Item, e: &RawElement) -> bool {
    let fm = &e.frontmatter;
    fm.element_type.as_ref().is_some_and(|t| t.name() == i.etype)
        && i.req_class.as_deref().is_none_or(|c| fm.req_class.as_deref() == Some(c))
        && i.test_level.as_deref().is_none_or(|l| fm.test_level.as_deref() == Some(l))
        && (i.tags.is_empty() || fm.tags.iter().flatten().any(|t| i.tags.contains(t)))
}

/// Entry point. `elems` is already projected when `--config` was given. Returns the exit code.
pub fn cmd_compliance(model_root: &Path, elems: &[RawElement], standard: Option<&str>, json_out: bool, fail_on_missing: bool) -> i32 {
    let Some(standard) = standard else {
        eprintln!("Usage: syscribe --model <root> compliance --standard aspice|iso26262|iso21434 [--config <C>] [--json] [--fail-on-missing]");
        return 1;
    };
    let items = match configured(model_root, standard) {
        Err(e) => {
            eprintln!("compliance: {e}");
            return 1;
        }
        Ok(Some(i)) => i,
        Ok(None) => match builtin(standard) {
            Some(i) => i,
            None => {
                eprintln!("compliance: unknown standard '{standard}' — built in: aspice, iso26262, iso21434 (or define [standards.{standard}] in .syscribe.toml)");
                return 1;
            }
        },
    };
    let (mut complete, mut partial, mut missing) = (0, 0, 0);
    let rows: Vec<(Item, usize, usize, &'static str)> = items
        .into_iter()
        .map(|i| {
            let hits: Vec<&RawElement> = elems.iter().filter(|e| matches(&i, e)).collect();
            let approved = hits.iter().filter(|e| e.frontmatter.status.as_deref().is_some_and(|s| APPROVED.contains(&s))).count();
            let status = if hits.is_empty() {
                missing += 1;
                "missing"
            } else if approved == hits.len() {
                complete += 1;
                "complete"
            } else {
                partial += 1;
                "partial"
            };
            (i, hits.len(), approved, status)
        })
        .collect();
    if json_out {
        let items: Vec<_> = rows
            .iter()
            .map(|(i, p, a, s)| json!({"process": i.process, "workProduct": i.work_product, "type": i.etype, "present": p, "approved": a, "status": s}))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({"standard": standard, "items": items, "summary": {"complete": complete, "partial": partial, "missing": missing}})).unwrap_or_default()
        );
    } else {
        println!("Compliance — {standard}");
        println!("{:<12} {:<58} {:>7} {:>8}  status", "process", "work product", "present", "approved");
        for (i, p, a, s) in &rows {
            println!("{:<12} {:<58} {:>7} {:>8}  {s}", i.process, i.work_product, p, a);
        }
        println!("{complete} complete, {partial} partial, {missing} missing.");
    }
    i32::from(fail_on_missing && missing > 0)
}

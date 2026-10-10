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
    req_domain: Option<String>,
    tags: Vec<String>,
    test_levels: Vec<String>,
}

/// Statuses that count as approved, across the element types' status vocabularies.
const APPROVED: &[&str] = &[
    "approved", "implemented", "verified", "active", "done", "completed", "accepted", "closed", "released", "resolved", "fixed",
    "mitigated", "not_affected",
];
/// Elements in these statuses are out of use and are not counted at all.
const OUT_OF_USE: &[&str] = &["retired", "deprecated", "superseded", "rejected"];
const KNOWN_KEYS: &[&str] = &["process", "workProduct", "type", "reqClass", "reqDomain", "tag", "testLevel"];

fn it(process: &str, wp: &str, etype: &str, req_class: Option<&str>, test_levels: &[&str]) -> Item {
    Item {
        process: process.into(),
        work_product: wp.into(),
        etype: etype.into(),
        req_class: req_class.map(String::from),
        req_domain: None,
        tags: Vec::new(),
        test_levels: test_levels.iter().map(|s| s.to_string()).collect(),
    }
}

fn builtin(standard: &str) -> Option<Vec<Item>> {
    Some(match standard {
        "aspice" => vec![
            it("SYS.2", "System requirements", "Requirement", Some("system"), &[]),
            it("SYS.3", "System architecture", "PartDef", None, &[]),
            Item { req_domain: Some("software".into()), ..it("SWE.1", "Software requirements", "Requirement", None, &[]) },
            it("SWE.2", "Software architecture decisions", "ADR", None, &[]),
            it("SWE.4", "Unit verification", "TestCase", None, &["L1", "L2", "L3"]),
            it("SWE.5", "Integration verification", "TestCase", None, &["L4"]),
            it("SYS.5", "System qualification verification", "TestCase", None, &["L4", "L5"]),
            it("SUP.4", "Joint review records", "ReviewRecord", None, &[]),
            it("SUP.8", "Configuration baselines", "Baseline", None, &[]),
        ],
        "iso26262" => vec![
            it("Part 3", "Hazard analysis and risk assessment (hazardous events)", "HazardousEvent", None, &[]),
            it("Part 3", "Safety goals", "SafetyGoal", None, &[]),
            it("Part 4", "Technical safety requirements", "Requirement", Some("system"), &[]),
            it("Part 5", "FMEA / FMEDA", "FMEASheet", None, &[]),
            it("Part 5", "Fault tree analysis", "FaultTree", None, &[]),
            it("Part 9", "Dependent failure analysis", "DependentFailureAnalysis", None, &[]),
            it("Part 2", "Confirmation measures", "ConfirmationMeasure", None, &[]),
            it("Part 8", "Verification specification (test cases)", "TestCase", None, &[]),
        ],
        "iso21434" => vec![
            it("Clause 15", "Threat analysis and risk assessment sheet", "TARASheet", None, &[]),
            it("Clause 15", "Damage scenarios", "DamageScenario", None, &[]),
            it("Clause 15", "Threat scenarios", "ThreatScenario", None, &[]),
            it("Clause 9", "Cybersecurity goals", "CybersecurityGoal", None, &[]),
            it("Clause 9", "Security controls", "SecurityControl", None, &[]),
            it("Clause 8", "Vulnerability reports", "VulnerabilityReport", None, &[]),
        ],
        _ => return None,
    })
}

/// The `[standards.<name>]` items of `.syscribe.toml`: `Ok(None)` when the standard is not
/// configured, `Err` for a malformed table (unknown keys, wrong value types, an unknown element
/// type, or no items), so a typo can never silently widen a selector.
fn configured(model_root: &Path, standard: &str) -> Result<Option<Vec<Item>>, String> {
    let Ok(text) = std::fs::read_to_string(model_root.join(".syscribe.toml")) else { return Ok(None) };
    let root: toml::Value = match toml::from_str(&text) {
        Ok(r) => r,
        Err(e) => {
            return if text.contains("standards") { Err(format!(".syscribe.toml does not parse: {}", e.message())) } else { Ok(None) };
        }
    };
    let Some(std_tbl) = root.get("standards") else { return Ok(None) };
    let std_tbl = std_tbl.as_table().ok_or("`standards` must be a table of [standards.<name>] tables")?;
    let Some(tbl) = std_tbl.get(standard) else { return Ok(None) };
    let at = format!("[[standards.{standard}.item]]");
    let items = tbl
        .get("item")
        .and_then(|i| i.as_array())
        .filter(|a| !a.is_empty())
        .ok_or_else(|| format!("[standards.{standard}] needs at least one `{at}` entry"))?;
    let known_types: Vec<&str> = syscribe_model::element::ElementType::ALL.iter().map(|t| t.name()).collect();
    let mut out = Vec::new();
    for (n, i) in items.iter().enumerate() {
        let at = format!("{at} #{}", n + 1);
        let t = i.as_table().ok_or_else(|| format!("{at} must be a table"))?;
        if let Some(k) = t.keys().find(|k| !KNOWN_KEYS.contains(&k.as_str())) {
            return Err(format!("{at}: unknown key '{k}' (known: {})", KNOWN_KEYS.join(", ")));
        }
        let str_of = |k: &str| -> Result<Option<String>, String> {
            match t.get(k) {
                None => Ok(None),
                Some(toml::Value::String(x)) => Ok(Some(x.clone())),
                Some(_) => Err(format!("{at}: `{k}` must be a string")),
            }
        };
        let list_of = |k: &str| -> Result<Vec<String>, String> {
            match t.get(k) {
                None => Ok(Vec::new()),
                Some(toml::Value::String(x)) => Ok(vec![x.clone()]),
                Some(toml::Value::Array(a)) => a
                    .iter()
                    .map(|x| x.as_str().map(String::from).ok_or_else(|| format!("{at}: `{k}` entries must be strings")))
                    .collect(),
                Some(_) => Err(format!("{at}: `{k}` must be a string or a list of strings")),
            }
        };
        let (Some(process), Some(work_product), Some(etype)) = (str_of("process")?, str_of("workProduct")?, str_of("type")?) else {
            return Err(format!("{at} needs `process`, `workProduct` and `type`"));
        };
        if !known_types.contains(&etype.as_str()) {
            return Err(format!("{at}: unknown element type '{etype}'"));
        }
        out.push(Item { process, work_product, etype, req_class: str_of("reqClass")?, req_domain: str_of("reqDomain")?, tags: list_of("tag")?, test_levels: list_of("testLevel")? });
    }
    Ok(Some(out))
}

fn matches(i: &Item, e: &RawElement) -> bool {
    let fm = &e.frontmatter;
    fm.element_type.as_ref().is_some_and(|t| t.name() == i.etype)
        && !fm.status.as_deref().is_some_and(|s| OUT_OF_USE.contains(&s))
        && i.req_class.as_deref().is_none_or(|c| fm.req_class.as_deref() == Some(c))
        && i.req_domain.as_deref().is_none_or(|d| fm.req_domain.as_deref() == Some(d))
        && (i.test_levels.is_empty() || fm.test_level.as_deref().is_some_and(|l| i.test_levels.iter().any(|x| x == l)))
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
        let w0 = rows.iter().map(|r| r.0.process.chars().count()).chain([7]).max().unwrap_or(7);
        let w1 = rows.iter().map(|r| r.0.work_product.chars().count()).chain([12]).max().unwrap_or(12);
        println!("{:<w0$} {:<w1$} {:>7} {:>8}  status", "process", "work product", "present", "approved");
        for (i, p, a, s) in &rows {
            println!("{:<w0$} {:<w1$} {:>7} {:>8}  {s}", i.process, i.work_product, p, a);
        }
        println!("{complete} complete, {partial} partial, {missing} missing.");
    }
    i32::from(fail_on_missing && missing > 0)
}

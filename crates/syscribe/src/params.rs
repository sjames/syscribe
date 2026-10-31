//! `params export` / `params check` — the late-bound parameter file (`REQ-TRS-BT-023..030`,
//! `ADR-SYS-BT-001`). The model is the source of truth: `export` generates the YAML values
//! file (and optionally its JSON Schema) from a Configuration and a stage; `check` validates
//! a values file — hand-edited or produced elsewhere — against the same contract.

use syscribe_model::binding_time::BindingTime;
use syscribe_model::element::RawElement;
use syscribe_model::params::{check_file, contract, header_field, Contract};
use syscribe_model::validator::Severity;

fn value_of<'a>(rest: &'a [String], flag: &str) -> Option<&'a str> {
    rest.windows(2).find(|w| w[0] == flag).map(|w| w[1].as_str())
}

fn usage() -> i32 {
    eprintln!("Usage: syscribe -m <root> params export --config <Configuration> [--stage <stage>] [--out FILE] [--schema FILE] [--allow-open]");
    eprintln!("       syscribe -m <root> params check <FILE> --config <Configuration> [--stage <stage>] [--deny <CODES>]");
    2
}

fn parse_stage(s: &str) -> Result<BindingTime, i32> {
    BindingTime::parse(s).ok_or_else(|| {
        eprintln!("Unknown stage '{s}'; the vocabulary is: {}", syscribe_model::binding_time::active().names_hint());
        2
    })
}

fn build(elements: &[RawElement], conf: &str, stage: Option<BindingTime>) -> Result<Contract, i32> {
    let Some(idx) = crate::residual::find_configuration(elements, conf) else {
        eprintln!("Configuration not found: {conf}");
        return Err(1);
    };
    contract(elements, idx, stage).map_err(|e| {
        eprintln!("{e}");
        2
    })
}

pub fn cmd_params(elements: &[RawElement], rest: &[String]) -> i32 {
    match rest.first().map(String::as_str) {
        Some("export") => export(elements, &rest[1..]),
        Some("check") => check(elements, &rest[1..]),
        _ => usage(),
    }
}

fn export(elements: &[RawElement], rest: &[String]) -> i32 {
    let Some(conf) = value_of(rest, "--config") else { return usage() };
    let stage = match value_of(rest, "--stage").map(parse_stage).transpose() {
        Ok(s) => s,
        Err(c) => return c,
    };
    let c = match build(elements, conf, stage) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let allow_open = rest.iter().any(|a| a == "--allow-open");
    let open = c.unresolved();
    if !open.is_empty() && !allow_open {
        eprintln!("Cannot export {} at stage {}: {} required parameter(s) have no value, default or fallback:", c.configuration, c.stage.name(), open.len());
        for p in &open {
            eprintln!("  - {}.{} ({})", p.feature, p.name, p.feature_qname);
        }
        eprintln!("Bind them in the Configuration chain, give them a `default:` or `fallback:`, or pass --allow-open to emit marked placeholders.");
        return 1;
    }
    let yaml = c.to_yaml(env!("CARGO_PKG_VERSION"));
    match value_of(rest, "--out") {
        Some(path) => {
            if let Err(e) = std::fs::write(path, &yaml) {
                eprintln!("Cannot write {path}: {e}");
                return 1;
            }
        }
        None => print!("{yaml}"),
    }
    if let Some(path) = value_of(rest, "--schema") {
        let text = serde_json::to_string_pretty(&c.schema()).unwrap_or_default() + "\n";
        if let Err(e) = std::fs::write(path, text) {
            eprintln!("Cannot write {path}: {e}");
            return 1;
        }
    }
    0
}

fn check(elements: &[RawElement], rest: &[String]) -> i32 {
    let Some(file) = rest.first().filter(|a| !a.starts_with("--")) else { return usage() };
    let Some(conf) = value_of(rest, "--config") else { return usage() };
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Cannot read {file}: {e}");
            return 1;
        }
    };
    // The stage the file was generated for, unless one is named explicitly.
    let stage_name = value_of(rest, "--stage").map(str::to_string).or_else(|| header_field(&text, "stage"));
    let stage = match stage_name.as_deref().map(parse_stage).transpose() {
        Ok(s) => s,
        Err(c) => return c,
    };
    let c = match build(elements, conf, stage) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let findings = check_file(&c, &text, file);
    println!("# Params check: {file}");
    println!();
    println!("Configuration {} at stage {}", c.configuration, c.stage.name());
    println!();
    if findings.is_empty() {
        println!("The values file matches the contract.");
        return 0;
    }
    println!("| Code | Message |");
    println!("|---|---|");
    for f in &findings {
        println!("| {} | {} |", f.code, f.message);
    }
    let errors = findings.iter().filter(|f| f.severity == Severity::Error).count();
    let warnings = findings.len() - errors;
    println!();
    println!("{errors} error(s), {warnings} warning(s)");
    let denied: Vec<&str> = value_of(rest, "--deny").map(|d| d.split(',').map(str::trim).collect()).unwrap_or_default();
    if errors > 0 {
        1
    } else if findings.iter().any(|f| denied.contains(&f.code)) {
        2
    } else {
        0
    }
}

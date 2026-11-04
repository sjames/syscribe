//! `propagate` — forced / excluded / free features for a partial selection, without writing a file (GH #416,
//! `REQ-TRS-PROPAGATE-001`). The SAT engine is `feature_model::configure_selection` (what the web configurator
//! uses); this adds seeding from a Configuration, the stage filter `configure --stage` applies, and one payload
//! shared by the CLI, the MCP tool and the LSP request `syscribe/propagate`.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use syscribe_model::element::{ElementType, RawElement};

/// What to propagate.
#[derive(Debug, Default, Clone)]
pub struct Request {
    /// Seed the selection from this Configuration's own `features:` (as `configure` reads it).
    pub from: Option<String>,
    /// Explicit choices, feature (qualified name or `FEAT-*` id) -> selected. They override `from`.
    pub select: BTreeMap<String, bool>,
    /// Report only the choices still open at this binding stage; features bound outside it are `deferred`.
    pub stage: Option<String>,
}

/// An error with the exit code the CLI should use (1 = not found, 2 = usage).
pub type Fail = (i32, String);

/// `feature=true|false` (also `on|off`, `yes|no`).
pub fn parse_choice(s: &str) -> Result<(String, bool), String> {
    let (k, v) = s.rsplit_once('=').ok_or_else(|| format!("'{s}' is not <feature>=<true|false>"))?;
    let v = match v.trim().to_ascii_lowercase().as_str() {
        "true" | "on" | "yes" | "1" => true,
        "false" | "off" | "no" | "0" => false,
        o => return Err(format!("'{o}' is not true or false in '{s}'")),
    };
    if k.trim().is_empty() {
        return Err(format!("'{s}' names no feature"));
    }
    Ok((k.trim().to_string(), v))
}

/// The effective choices of `req`: the Configuration's own selection, overridden by `select`, in canonical keys.
pub fn seed(elements: &[RawElement], req: &Request) -> Result<BTreeMap<String, bool>, Fail> {
    let alias = syscribe_model::variability::feature_id_to_qname(elements);
    let mut selection: BTreeMap<String, bool> = BTreeMap::new();
    if let Some(conf) = &req.from {
        let cfg = elements
            .iter()
            .find(|e| e.frontmatter.element_type == Some(ElementType::Configuration) && (e.frontmatter.id.as_deref() == Some(conf) || e.qualified_name == *conf))
            .ok_or_else(|| (1, format!("Configuration not found: {conf}")))?;
        selection = syscribe_model::variability::canon_selection(&cfg.frontmatter.feature_selections(), &alias);
    }
    for (k, v) in &req.select {
        selection.insert(syscribe_model::variability::canon_feature_ref(k, &alias), *v);
    }
    Ok(selection)
}

pub fn compute(elements: &[RawElement], req: &Request) -> Result<Value, Fail> {
    let stage = match req.stage.as_deref() {
        None => None,
        Some(s) => Some(syscribe_model::binding_time::BindingTime::parse(s).ok_or_else(|| {
            (2, format!("Unknown stage '{s}'; the vocabulary is: {}", syscribe_model::binding_time::active().names_hint()))
        })?),
    };
    let selection = seed(elements, req)?;

    let mut out = syscribe_model::feature_model::configure_selection(elements, &selection);
    let windows = syscribe_model::staging::feature_windows(elements);
    let deferred = |name: &str| stage.is_some_and(|st| windows.get(name).is_some_and(|w| !w.contains(st)));

    let (mut forced_true, mut forced_false, mut free) = (Vec::new(), Vec::new(), Vec::new());
    if let Some(features) = out.get_mut("features").and_then(Value::as_object_mut) {
        for (name, info) in features.iter_mut() {
            let d = deferred(name);
            if d {
                info["deferred"] = json!(true);
            }
            if d {
                continue;
            }
            match info["state"].as_str() {
                Some("forcedOn") => forced_true.push(name.clone()),
                Some("forcedOff") => forced_false.push(name.clone()),
                Some("free") => free.push(name.clone()),
                _ => {}
            }
        }
    }
    out["schemaVersion"] = json!("1.0");
    out["stage"] = json!(stage.map(|s| s.name()));
    out["selection"] = json!(selection);
    out["forcedTrue"] = json!(forced_true);
    out["forcedFalse"] = json!(forced_false);
    out["free"] = json!(free);
    Ok(out)
}

const USAGE: &str = "Usage: syscribe -m <root> propagate [--from <Configuration>] [--select <feature>=<true|false>]... [--stage <stage>] [--json]";

/// Parse the tokens after `propagate`.
pub fn parse_args(args: &[String]) -> Result<(Request, bool), String> {
    let (mut req, mut json) = (Request::default(), false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |f: &str| it.next().cloned().ok_or_else(|| format!("propagate: {f} needs a value"));
        match a.as_str() {
            "--json" => json = true,
            "--from" => req.from = Some(val("--from")?),
            "--stage" => req.stage = Some(val("--stage")?),
            "--select" => {
                let (k, v) = parse_choice(&val("--select")?).map_err(|m| format!("propagate: --select {m}"))?;
                req.select.insert(k, v);
            }
            f if f.starts_with('-') => return Err(format!("propagate: unknown option '{f}'")),
            _ => return Err(USAGE.to_string()),
        }
    }
    Ok((req, json))
}

pub fn cmd_propagate(elements: &[RawElement], args: &[String]) -> i32 {
    let (req, json) = match parse_args(args) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("{m}");
            return 2;
        }
    };
    let out = match compute(elements, &req) {
        Ok(o) => o,
        Err((code, m)) => {
            eprintln!("{m}");
            return code;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return 0;
    }
    if out["hasFeatureModel"] == false {
        println!("No feature model present — nothing to propagate.");
        return 0;
    }
    if let Some(s) = out["skipped"].as_str() {
        println!("{s}");
        return 0;
    }
    if out["satisfiable"] == false {
        println!("# Conflict\n\nNo product satisfies this selection.");
        for c in out["conflict"]["choices"].as_array().into_iter().flatten() {
            println!("- {} = {}", c["feature"].as_str().unwrap_or(""), c["selected"]);
        }
        for c in out["conflict"]["constraints"].as_array().into_iter().flatten() {
            println!("- clashes with {}", c.as_str().unwrap_or(""));
        }
        return 0;
    }
    println!("# Propagation{}\n", out["stage"].as_str().map(|s| format!(" at stage {s}")).unwrap_or_default());
    let list = |title: &str, key: &str| {
        let v: Vec<&str> = out[key].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
        println!("**{title}** ({})", v.len());
        v.iter().for_each(|n| println!("- {n}"));
        println!();
    };
    list("Forced on", "forcedTrue");
    list("Forced off", "forcedFalse");
    list("Free", "free");
    let p = &out["products"];
    println!("{} product(s) remain{}.", p["count"], if p["capped"] == true { " (at least)" } else { "" });
    0
}

//! `explain-feature <feature>` — why a feature is forced, excluded or free for a selection, and what would have to
//! change to select it (GH #417, `REQ-TRS-EXPLAIN-001`). The causes come from `feature_model::explain_feature`;
//! this adds seeding from a Configuration and the parameter constraints that touch the feature, and one payload
//! shared by the CLI, the MCP tool and the LSP request `syscribe/explainFeature`.

use serde_json::{json, Value};
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::feature_model::{constraint_states, Tri};

use crate::propagate::{seed, Fail, Request};

pub fn compute(elements: &[RawElement], feature: &str, req: &Request) -> Result<Value, Fail> {
    let selection = seed(elements, req)?;
    let mut out = syscribe_model::feature_model::explain_feature(elements, &selection, feature);
    out["schemaVersion"] = json!("1.0");
    out["selection"] = json!(selection);
    // Parameter constraints that mention one of the feature's parameters, as they stand for the Configuration.
    let mut pcs = Vec::new();
    if let (Some(conf), Some(q)) = (&req.from, out["feature"].as_str().filter(|_| out["found"] == true)) {
        if let Some(idx) = elements.iter().position(|e| {
            e.frontmatter.element_type == Some(ElementType::Configuration) && (e.frontmatter.id.as_deref() == Some(conf.as_str()) || e.qualified_name == *conf)
        }) {
            let prefix = format!("{q}.");
            for c in constraint_states(elements, idx).into_iter().filter(|c| c.expression.contains(&prefix)) {
                let state = match c.state {
                    Tri::Satisfied => "satisfied",
                    Tri::Violated => "violated",
                    Tri::Undetermined => "undetermined",
                };
                pcs.push(json!({ "id": c.id, "expression": c.expression, "state": state, "severity": if c.warning { "warning" } else { "error" }, "open": c.open }));
            }
        }
    }
    out["parameterConstraints"] = json!(pcs);
    Ok(out)
}

const USAGE: &str = "Usage: syscribe -m <root> explain-feature <feature> [--from <Configuration>] [--select <feature>=<true|false>]... [--json]";

pub fn cmd_explain_feature(elements: &[RawElement], args: &[String]) -> i32 {
    let mut feature: Option<String> = None;
    let (mut req, mut json) = (Request::default(), false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |f: &str| it.next().cloned().ok_or_else(|| format!("explain-feature: {f} needs a value"));
        let r: Result<(), String> = match a.as_str() {
            "--json" => {
                json = true;
                Ok(())
            }
            "--from" => val("--from").map(|v| req.from = Some(v)),
            "--select" => val("--select").and_then(|v| crate::propagate::parse_choice(&v).map_err(|m| format!("explain-feature: --select {m}"))).map(|(k, v)| {
                req.select.insert(k, v);
            }),
            f if f.starts_with('-') => Err(format!("explain-feature: unknown option '{f}'")),
            _ if feature.is_none() => {
                feature = Some(a.clone());
                Ok(())
            }
            _ => Err(USAGE.to_string()),
        };
        if let Err(m) = r {
            eprintln!("{m}");
            return 2;
        }
    }
    let Some(feature) = feature else {
        eprintln!("{USAGE}");
        return 2;
    };
    let out = match compute(elements, &feature, &req) {
        Ok(o) => o,
        Err((code, m)) => {
            eprintln!("{m}");
            return code;
        }
    };
    if out["found"] == false {
        eprintln!("{}", out["skipped"].as_str().map(str::to_string).unwrap_or_else(|| format!("'{feature}' is not a known FeatureDef (give a qualified name or a FEAT-* id)")));
        return 1;
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return 0;
    }
    println!("# {} — {}\n", out["feature"].as_str().unwrap_or(""), out["state"].as_str().unwrap_or(""));
    let because = &out["because"];
    if because.is_object() {
        println!("## Because\n");
        for c in because["choices"].as_array().into_iter().flatten() {
            println!("- your choice: {} = {}", c["feature"].as_str().unwrap_or(""), c["selected"]);
        }
        for c in because["constraints"].as_array().into_iter().flatten() {
            println!("- {} ({})", c["label"].as_str().unwrap_or(""), c["kind"].as_str().unwrap_or(""));
        }
        println!();
    }
    let t = &out["toSelect"];
    if t["needed"] == true {
        println!("## To select it\n");
        if t["modelLevel"] == true {
            println!("It cannot be selected in any product; the model itself prevents it:");
            for c in t["constraints"].as_array().into_iter().flatten() {
                println!("- {}", c["label"].as_str().unwrap_or(""));
            }
        } else if t["options"].as_array().is_some_and(|o| o.is_empty()) {
            println!("Nothing blocks it.");
        } else {
            println!("Drop any one of these choices:");
            for o in t["options"].as_array().into_iter().flatten() {
                println!("- {} = {}", o["drop"]["feature"].as_str().unwrap_or(""), o["drop"]["selected"]);
            }
        }
        println!();
    }
    for c in out["parameterConstraints"].as_array().into_iter().flatten() {
        println!("- parameter constraint {} [{}]: {}", c["id"].as_str().unwrap_or(""), c["state"].as_str().unwrap_or(""), c["expression"].as_str().unwrap_or(""));
    }
    0
}

//! The parameters section of `diff --config A --config B` (GH #466, `REQ-TRS-DIFFPARAM-001`): for each parameter,
//! the effective value in each Configuration, where it came from, and its unit. Two variants that select the same
//! features but bind different values differ here and nowhere else. The CLI and the MCP `diff_configs` tool share it.

use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use syscribe_model::element::{ElementType, RawElement};

/// One side of a row: the effective value and its provenance (`kind`, `from`, `stage` as in `timeline`).
fn side(p: &Value) -> Value {
    json!({ "value": p["value"], "source": p["source"], "stage": p["stage"] })
}

/// `<FeatureDef qname>.<param>` -> unit, from the parameter declarations.
fn units(elements: &[RawElement]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for fd in elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef)) {
        for p in fd.frontmatter.parameters.iter().flatten() {
            let serde_yaml::Value::Mapping(m) = p else { continue };
            let get = |k: &str| m.get(serde_yaml::Value::String(k.to_string())).and_then(|v| v.as_str());
            if let (Some(name), Some(unit)) = (get("name"), get("unit")) {
                out.insert(format!("{}.{}", fd.qualified_name, name), unit.to_string());
            }
        }
    }
    out
}

/// The parameter rows of two Configurations, sorted by key. Only parameters whose effective value differs (or that
/// only one side has) unless `all`. `None` when either is not a Configuration of the model.
pub fn rows(elements: &[RawElement], a: &str, b: &str, all: bool) -> Option<Vec<Value>> {
    let params = |conf: &str| -> Option<BTreeMap<String, Value>> {
        let idx = crate::residual::find_configuration(elements, conf)?;
        let p = syscribe_model::provenance::to_json(&syscribe_model::provenance::provenance(elements, idx));
        Some(p["parameters"].as_array()?.iter().filter_map(|q| Some((q["key"].as_str()?.to_string(), q.clone()))).collect())
    };
    let (pa, pb) = (params(a)?, params(b)?);
    let unit = units(elements);
    let keys: BTreeSet<&String> = pa.keys().chain(pb.keys()).collect();
    let mut out = Vec::new();
    for k in keys {
        let (x, y) = (pa.get(k), pb.get(k));
        let differs = x.map(|q| &q["value"]) != y.map(|q| &q["value"]);
        if !all && !differs {
            continue;
        }
        out.push(json!({
            "key": k,
            "unit": unit.get(k),
            "differs": differs,
            "a": x.map(side),
            "b": y.map(side),
        }));
    }
    Some(out)
}

fn show(s: &Value, unit: Option<&str>) -> String {
    if s.is_null() {
        return "(not set)".into();
    }
    let v = match &s["value"] {
        Value::String(t) => t.clone(),
        o => o.to_string(),
    };
    let kind = s["source"]["kind"].as_str().unwrap_or("");
    let from = s["source"]["from"].as_str().map(|f| format!(" {f}")).unwrap_or_default();
    format!("{v}{} ({kind}{from})", unit.map(|u| format!(" {u}")).unwrap_or_default())
}

/// The text rendering: a section titled for the differing (or `all`) parameters.
pub fn print_text(rows: &[Value], a: &str, b: &str, all: bool) {
    println!();
    println!("{} ({}):", if all { "Parameters" } else { "Parameters that differ" }, rows.len());
    for r in rows {
        let unit = r["unit"].as_str();
        let mark = if r["differs"] == true { "~" } else { "=" };
        println!("  {mark} {}: {a} = {}; {b} = {}", r["key"].as_str().unwrap_or(""), show(&r["a"], unit), show(&r["b"], unit));
    }
}

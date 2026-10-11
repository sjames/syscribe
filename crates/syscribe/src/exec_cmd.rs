//! `eval` — evaluate an expression of the execution engine's language (GH #273,
//! REQ-TRS-EXPR-001). Model-free and deterministic.

use serde_json::json;
use syscribe_model::expr::{format_number, format_value, parse, parse_value, to_unit, unit_label, Value};

const USAGE: &str = "Usage: syscribe eval '<expression>' [--set name=value]... [--unit <unit>] [--json]";

fn fail(msg: &str) -> i32 {
    eprintln!("eval: {msg}");
    1
}

/// Entry point; `args` are the tokens after `eval`. Returns the process exit code.
pub fn cmd_eval(args: &[String]) -> i32 {
    let mut expr: Option<&str> = None;
    let mut sets: Vec<(&str, &str)> = Vec::new();
    let mut unit: Option<&str> = None;
    let mut json_out = false;
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--json" => json_out = true,
            "--set" => {
                i += 1;
                let Some(kv) = args.get(i) else { return fail(&format!("--set expects name=value\n{USAGE}")) };
                match kv.split_once('=') {
                    Some((k, v)) if !k.trim().is_empty() => sets.push((k.trim(), v)),
                    _ => return fail(&format!("--set expects name=value, got '{kv}'")),
                }
            }
            "--unit" => {
                i += 1;
                let Some(u) = args.get(i) else { return fail(&format!("--unit expects a unit\n{USAGE}")) };
                unit = Some(u.as_str());
            }
            _ if a.starts_with("--") => return fail(&format!("unknown option '{a}'\n{USAGE}")),
            _ if expr.is_none() => expr = Some(a),
            _ => return fail(&format!("unexpected argument '{a}' (quote the expression)\n{USAGE}")),
        }
        i += 1;
    }
    let Some(src) = expr else { return fail(USAGE) };

    let mut env: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for (k, v) in sets {
        match parse_value(v) {
            Some(val) => {
                env.insert(k.to_string(), val);
            }
            None => return fail(&format!("--set {k}: cannot read the value '{v}'")),
        }
    }
    let value = match parse(src).and_then(|e| e.eval(&|n: &str| env.get(n).cloned())) {
        Ok(v) => v,
        Err(e) => return fail(&e.to_string()),
    };

    // (printed number, unit label) for a number; `--unit` converts.
    let (text, unit_out, num): (String, Option<String>, Option<f64>) = match (&value, unit) {
        (Value::Num(q), Some(u)) => match to_unit(q, u) {
            Ok(x) => (format!("{} {u}", format_number(x)), Some(u.to_string()), Some(x)),
            Err(e) => return fail(&e.to_string()),
        },
        (Value::Num(q), None) => (format_value(&value), unit_label(&q.dim), Some(q.v)),
        (_, Some(_)) => return fail("--unit applies to a numeric result"),
        _ => (format_value(&value), None, None),
    };
    if json_out {
        let dimension = match &value {
            Value::Num(q) => q.dim.human(),
            _ => "n/a".to_string(),
        };
        let v = match (&value, num) {
            (Value::Num(_), Some(x)) => json!(x),
            (Value::Bool(b), _) => json!(b),
            (Value::Str(s), _) => json!(s),
            _ => json!(null),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({"expression": src, "type": value.kind(), "value": v, "unit": unit_out, "dimension": dimension})).unwrap_or_default()
        );
    } else {
        println!("{text}");
    }
    0
}

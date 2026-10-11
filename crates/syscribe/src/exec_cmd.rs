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

// ── constraints ──────────────────────────────────────────────────────────────

use std::collections::BTreeMap;

use syscribe_model::constraints::{attribute_env, check_all, config_env, summary, CheckResult, Env, Status};
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;

const CONSTRAINTS_USAGE: &str = "Usage: syscribe -m <root> constraints check [--config <CONF> | --all-configs] [--set name=value]... [--strict] [--json]";

fn usage_err(msg: &str) -> i32 {
    eprintln!("constraints: {msg}\n{CONSTRAINTS_USAGE}");
    1
}

struct Opts {
    json: bool,
    strict: bool,
    all_configs: bool,
    config: Option<String>,
    sets: Vec<(String, Value)>,
}

fn parse_opts(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts { json: false, strict: false, all_configs: false, config: None, sets: Vec::new() };
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--json" => o.json = true,
            "--strict" => o.strict = true,
            "--all-configs" => o.all_configs = true,
            "--config" => {
                i += 1;
                o.config = Some(args.get(i).ok_or("--config expects a Configuration id")?.clone());
            }
            "--set" => {
                i += 1;
                let kv = args.get(i).ok_or("--set expects name=value")?;
                let (k, v) = kv.split_once('=').filter(|(k, _)| !k.trim().is_empty()).ok_or_else(|| format!("--set expects name=value, got '{kv}'"))?;
                let val = parse_value(v).ok_or_else(|| format!("--set {k}: cannot read the value '{v}'"))?;
                o.sets.push((k.trim().to_string(), val));
            }
            _ => return Err(format!("unknown option or argument '{a}'")),
        }
        i += 1;
    }
    if o.all_configs && o.config.is_some() {
        return Err("--config and --all-configs are mutually exclusive".to_string());
    }
    Ok(o)
}

fn disp_id(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// The environment of one evaluation: attribute values, then the configuration's parameters, then `--set`.
fn build_env(view: &[RawElement], cfg: Option<&str>, sets: &[(String, Value)]) -> Env {
    let resolver = Resolver::new(view);
    let mut env = attribute_env(view);
    if let Some(c) = cfg {
        if let Some(ce) = view.iter().find(|e| e.frontmatter.element_type == Some(ElementType::Configuration) && (disp_id(e) == c || e.qualified_name == c)) {
            env.extend(config_env(view, &resolver, ce));
        }
    }
    for (k, v) in sets {
        env.insert(k.clone(), v.clone());
    }
    env
}

fn result_json(r: &CheckResult) -> serde_json::Value {
    let bindings: serde_json::Map<String, serde_json::Value> = r.bindings.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
    json!({
        "id": r.id, "kind": r.kind, "constraint": r.constraint, "expression": r.expression,
        "status": r.status.as_str(), "message": r.message, "bindings": bindings, "unresolved": r.unresolved,
    })
}

fn failed(results: &[CheckResult], strict: bool) -> bool {
    results.iter().any(|r| matches!(r.status, Status::Fail | Status::Error) || (strict && r.status == Status::Unresolved))
}

fn detail(r: &CheckResult) -> String {
    match r.status {
        Status::Pass => String::new(),
        Status::Fail => r.bindings.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(", "),
        _ => r.message.clone(),
    }
}

fn summary_line(results: &[CheckResult]) -> String {
    let s = summary(results);
    format!("{} check(s): {} pass, {} fail, {} unresolved, {} error", results.len(), s["pass"], s["fail"], s["unresolved"], s["error"])
}

/// Entry point for `constraints …`. `lens(config)` returns the model projected onto a Configuration
/// (or the whole model for `None`). Returns the process exit code.
pub fn cmd_constraints(elems: &[RawElement], args: &[String], lens: &dyn Fn(Option<&str>) -> Vec<RawElement>) -> i32 {
    match args.first().map(String::as_str) {
        Some("check") => {}
        Some(other) => return usage_err(&format!("unknown subcommand '{other}'")),
        None => return usage_err("a subcommand is required"),
    }
    let o = match parse_opts(&args[1..]) {
        Ok(o) => o,
        Err(m) => return usage_err(&m),
    };
    if o.all_configs {
        let mut cfgs: Vec<&RawElement> = elems.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration)).collect();
        cfgs.sort_by_key(|e| disp_id(e));
        let per: Vec<(String, Vec<CheckResult>)> = cfgs
            .iter()
            .map(|c| {
                let id = disp_id(c);
                let view = lens(Some(&id));
                let env = build_env(&view, Some(&id), &o.sets);
                (id, check_all(&view, &env))
            })
            .collect();
        let any_failed = per.iter().any(|(_, r)| failed(r, o.strict));
        if o.json {
            let list: Vec<_> = per
                .iter()
                .map(|(id, r)| json!({"id": id, "summary": summary(r), "results": r.iter().map(result_json).collect::<Vec<_>>()}))
                .collect();
            println!("{}", serde_json::to_string_pretty(&json!({"configurations": list})).unwrap_or_default());
        } else if per.is_empty() {
            println!("No Configuration in the model.");
        } else {
            // check × configuration table
            let mut ids: Vec<String> = per.iter().flat_map(|(_, r)| r.iter().map(|x| format!("{} [{}]", x.id, x.kind))).collect();
            ids.sort();
            ids.dedup();
            let mut cells: BTreeMap<(String, String), &str> = BTreeMap::new();
            for (cfg, rs) in &per {
                for r in rs {
                    cells.insert((format!("{} [{}]", r.id, r.kind), cfg.clone()), r.status.as_str());
                }
            }
            print!("| Check |");
            for (cfg, _) in &per {
                print!(" {cfg} |");
            }
            println!();
            print!("|---|");
            for _ in &per {
                print!("---|");
            }
            println!();
            for id in &ids {
                print!("| {id} |");
                for (cfg, _) in &per {
                    print!(" {} |", cells.get(&(id.clone(), cfg.clone())).copied().unwrap_or("-"));
                }
                println!();
            }
        }
        return if any_failed { 2 } else { 0 };
    }

    let view = lens(o.config.as_deref());
    let env = build_env(&view, o.config.as_deref(), &o.sets);
    let results = check_all(&view, &env);
    if o.json {
        let mut doc = json!({"results": results.iter().map(result_json).collect::<Vec<_>>(), "summary": summary(&results)});
        if let Some(c) = &o.config {
            doc["configuration"] = json!(c);
        }
        println!("{}", serde_json::to_string_pretty(&doc).unwrap_or_default());
    } else if results.is_empty() {
        println!("No constraint to check.");
    } else {
        println!("Constraint check — {}", summary_line(&results));
        println!();
        println!("| Check | Kind | Status | Expression | Detail |");
        println!("|---|---|---|---|---|");
        for r in &results {
            println!("| {} | {} | {} | `{}` | {} |", r.id, r.kind, r.status.as_str(), r.expression, detail(r));
        }
    }
    if failed(&results, o.strict) {
        2
    } else {
        0
    }
}

/// Entry point for `compile <StateDef> [--json]` (GH #278): lower a state machine to the IR and print it.
/// Exit 0 · 1 usage or not a state machine · 2 lowering diagnostics.
pub fn cmd_compile(elems: &[RawElement], args: &[String]) -> i32 {
    let json = args.iter().any(|a| a == "--json");
    let Some(name) = args.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("Usage: syscribe --model <root> compile <StateDef> [--json]");
        return 1;
    };
    match syscribe_model::ir::lower(elems, name) {
        Ok(m) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&m).expect("IR serialises"));
            } else {
                println!("{}  (hash {})", m.machine, m.hash);
                println!("  {} variable(s), {} event(s), {} state(s), {} transition(s)", m.variables.len(), m.events.len(), m.states.len(), m.transitions.len());
                for s in &m.states {
                    println!("  state {:>3}  {:<24} {:?}", s.id, s.path, s.kind);
                }
                for t in &m.transitions {
                    let tgt = t.target.map(|x| m.states[x].path.clone()).unwrap_or_else(|| "(internal)".into());
                    println!("  trans {:>3}  {} → {}  {}", t.id, m.states[t.source].path, tgt, serde_json::to_string(&t.trigger).unwrap_or_default());
                }
            }
            0
        }
        Err(diags) => {
            let usage = diags.iter().all(|(c, _)| *c == "E001");
            for (c, msg) in &diags {
                eprintln!("{c}: {msg}");
            }
            if usage { 1 } else { 2 }
        }
    }
}

/// Entry point for `simulate <StateDef> --do "<script>" [--strict] [--json]` (GH #279): lower the machine,
/// run a script of `send <event> [k=v ...]`, `advance <duration>` and `set <var>=<value>` commands and print
/// the trace. Exit 0 · 1 usage · 2 a lowering diagnostic or a run error.
pub fn cmd_simulate(elems: &[RawElement], args: &[String]) -> i32 {
    use syscribe_model::expr::{format_value, parse_value, Value};
    use syscribe_model::sm_run::{Options, Run};
    let json = args.iter().any(|a| a == "--json");
    let strict = args.iter().any(|a| a == "--strict");
    let script = args.windows(2).find(|w| w[0] == "--do").map(|w| w[1].clone()).unwrap_or_default();
    let skip: Vec<usize> = args.iter().enumerate().filter(|(_, a)| *a == "--do").map(|(i, _)| i + 1).collect();
    let Some(name) = args.iter().enumerate().find(|(i, a)| !a.starts_with("--") && !skip.contains(i)).map(|(_, a)| a) else {
        eprintln!("Usage: syscribe --model <root> simulate <StateDef> --do \"send <event> [k=v ...]; advance <duration>; set <var>=<value>\" [--strict] [--json]");
        return 1;
    };
    let machine = match syscribe_model::ir::lower(elems, name) {
        Ok(m) => m,
        Err(diags) => {
            for (c, msg) in &diags {
                eprintln!("{c}: {msg}");
            }
            return if diags.iter().all(|(c, _)| *c == "E001") { 1 } else { 2 };
        }
    };
    // Parse the whole script before running anything.
    enum Cmd {
        Send(String, Vec<(String, Value)>),
        Advance(f64),
        Set(String, Value),
    }
    let mut cmds = Vec::new();
    for raw in script.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let (verb, rest) = raw.split_once(char::is_whitespace).map(|(a, b)| (a, b.trim())).unwrap_or((raw, ""));
        match verb {
            "send" => {
                let mut it = rest.split_whitespace();
                let Some(ev) = it.next() else {
                    eprintln!("error: `send` needs an event name");
                    return 1;
                };
                let mut payload = Vec::new();
                for kv in it {
                    let Some((k, v)) = kv.split_once('=') else {
                        eprintln!("error: payload '{kv}' is not key=value");
                        return 1;
                    };
                    let Some(v) = parse_value(v) else {
                        eprintln!("error: payload value '{v}' is not a value");
                        return 1;
                    };
                    payload.push((k.to_string(), v));
                }
                cmds.push(Cmd::Send(ev.to_string(), payload));
            }
            "advance" => match parse_value(rest) {
                Some(Value::Num(q)) if q.dim == syscribe_model::expr::unit_scale("s").map(|x| x.1).unwrap_or(q.dim) && q.v >= 0.0 => cmds.push(Cmd::Advance(q.v)),
                _ => {
                    eprintln!("error: `advance` needs a non-negative duration, got '{rest}'");
                    return 1;
                }
            },
            "set" => {
                let Some((k, v)) = rest.split_once('=') else {
                    eprintln!("error: `set` needs <var>=<value>");
                    return 1;
                };
                let Some(v) = parse_value(v.trim()) else {
                    eprintln!("error: '{}' is not a value", v.trim());
                    return 1;
                };
                cmds.push(Cmd::Set(k.trim().to_string(), v));
            }
            other => {
                eprintln!("error: unknown command '{other}' (send, advance, set)");
                return 1;
            }
        }
    }
    let mut run = Run::new(&machine, Options { strict, ..Options::default() });
    for c in cmds {
        match c {
            Cmd::Send(e, p) => {
                let p: Vec<(&str, Value)> = p.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                run.send(&e, &p);
            }
            Cmd::Advance(s) => run.advance(s),
            Cmd::Set(k, v) => run.set(&k, v),
        }
    }
    let vars: std::collections::BTreeMap<String, String> = machine.variables.iter().filter_map(|v| run.var(&v.name).map(|x| (v.name.clone(), format_value(&x)))).collect();
    if json {
        let out = serde_json::json!({
            "machine": machine.machine,
            "machineHash": machine.hash,
            "time": run.time(),
            "active": run.active(),
            "variables": vars,
            "terminated": run.terminated(),
            "error": run.error(),
            "trace": run.trace(),
        });
        println!("{}", serde_json::to_string_pretty(&out).expect("serialises"));
    } else {
        println!("{}  (hash {})", machine.machine, machine.hash);
        for s in run.trace() {
            let what = match (&s.from, &s.to) {
                (Some(f), Some(t)) => format!("{f} → {t}"),
                (Some(f), None) => format!("{f} (internal)"),
                _ => s.note.clone().unwrap_or_default(),
            };
            println!("  {:>3}  t={:<8} {:<22} {}", s.n, syscribe_model::expr::format_number(s.time), s.cause, what);
            for c in &s.changed {
                println!("            {} : {} → {}", c.var, c.old, c.new);
            }
        }
        println!("active: {}   time: {}s", run.active().join(", "), syscribe_model::expr::format_number(run.time()));
        for (k, v) in &vars {
            println!("  {k} = {v}");
        }
        if let Some(e) = run.error() {
            println!("ERROR: {e}");
        }
    }
    if let Some(e) = run.error() {
        eprintln!("error: {e}");
        return 2;
    }
    0
}

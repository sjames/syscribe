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

const CONSTRAINTS_USAGE: &str = "Usage: syscribe -m <root> constraints check [--config <CONF> | --all-configs] [--set name=value]... [--strict] [--json]\n       syscribe -m <root> constraints calc <CalculationDef> [--config <CONF>] [--set name=value]... [--json]\n       syscribe -m <root> constraints sweep --param name=<lo>..<hi>:<step> [--param ...] [--config <CONF>] [--set name=value]... [--show '<expr>'] [--json]";

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
        Some("calc") => return cmd_calc(&args[1..], lens),
        Some("sweep") => return cmd_sweep(&args[1..], lens),
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
    // An ActionDef lowers to the action-graph IR (GH #276).
    {
        let resolver = Resolver::new(elems);
        if resolver.resolve_ref(elems, name).is_some_and(|e| matches!(e.frontmatter.element_type, Some(ElementType::ActionDef) | Some(ElementType::Action))) {
            return match syscribe_model::act_ir::lower_action(elems, name) {
                Ok(p) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&p).expect("IR serialises"));
                    } else {
                        println!("{}  (hash {})", p.root, p.hash);
                        for d in &p.defs {
                            println!("  {}: {} slot(s), {} node(s), {} edge(s), start {:?}", d.qname, d.slots.len(), d.nodes.len(), d.edges.len(), d.start);
                            for n in &d.nodes {
                                println!("    node {:>3}  {:<24} {}", n.id, n.stmt.name, serde_json::to_value(&n.stmt.kind).ok().and_then(|v| v["kind"].as_str().map(String::from)).unwrap_or_default());
                            }
                            for e in &d.edges {
                                println!("    edge {:>3}  {} → {}{}", e.id, d.nodes[e.from].stmt.name, d.nodes[e.to].stmt.name, e.guard.as_ref().map(|g| format!("  [{}]", g.src)).unwrap_or_else(|| if e.is_else { "  [else]".into() } else { String::new() }));
                            }
                        }
                    }
                    0
                }
                Err(diags) => {
                    for (c, msg) in &diags {
                        eprintln!("{c}: {msg}");
                    }
                    if diags.iter().all(|(c, _)| *c == "E001") { 1 } else { 2 }
                }
            };
        }
    }
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
fn simulate_script(elems: &[RawElement], args: &[String], model_root: &std::path::Path) -> i32 {
    use syscribe_model::expr::{format_value, parse_value, Value};
    use syscribe_model::sm_run::{Options, Run};
    let json = args.iter().any(|a| a == "--json");
    let strict = args.iter().any(|a| a == "--strict");
    let script = args.windows(2).find(|w| w[0] == "--do").map(|w| w[1].clone()).unwrap_or_default();
    let skip: Vec<usize> = args.iter().enumerate().filter(|(_, a)| *a == "--do" || *a == "--crosscheck").map(|(i, _)| i + 1).collect();
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
    let crosscheck = args.windows(2).find(|w| w[0] == "--crosscheck").map(|w| w[1].clone());
    let mut configs: Vec<Vec<String>> = vec![run.active()];
    let mut wire: Vec<serde_json::Value> = Vec::new();
    let to_json = |v: &Value| match v {
        Value::Num(q) => serde_json::json!(q.v),
        Value::Bool(b) => serde_json::json!(b),
        Value::Str(s) => serde_json::json!(s),
    };
    for c in cmds {
        match &c {
            Cmd::Send(e, p) => wire.push(serde_json::json!({"event": syscribe_model::sm_scxml::scxml_event(e), "data": p.iter().map(|(k, v)| (k.clone(), to_json(v))).collect::<serde_json::Map<_, _>>()})),
            Cmd::Advance(s) => wire.push(serde_json::json!({"advance": s})),
            Cmd::Set(k, v) => wire.push(serde_json::json!({"set": {k.clone(): to_json(v)}})),
        }
        match c {
            Cmd::Send(e, p) => {
                let p: Vec<(&str, Value)> = p.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                run.send(&e, &p);
            }
            Cmd::Advance(s) => run.advance(s),
            Cmd::Set(k, v) => run.set(&k, v),
        }
        configs.push(run.active());
    }
    if let Some(engine) = crosscheck {
        if run.error().is_none() {
            if let Err(e) = crosscheck_engine(&machine, &engine, &wire, &configs, model_root) {
                eprintln!("crosscheck: {e}");
                return 2;
            }
            println!("crosscheck: the SCXML engine agrees with syscribe on all {} configuration(s)", configs.len());
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

/// Entry point for `simulate`: a state machine with `--do` runs a script; a `TestCase` (or a machine, with
/// every scenario `TestCase` whose subject it is) runs its Gherkin scenarios (GH #279, #280).
pub fn cmd_simulate(elems: &[RawElement], args: &[String], lens: &dyn Fn(Option<&str>) -> Vec<RawElement>, model_root: &std::path::Path) -> i32 {
    use syscribe_model::element::ElementType;
    use syscribe_model::sm_scen::{extract_scenarios, subject_of, Status};
    let flag = |f: &str| args.iter().any(|a| a == f);
    let value = |f: &str| args.windows(2).find(|w| w[0] == f).map(|w| w[1].clone());
    let valued = ["--do", "--scenario", "--run", "--config", "--crosscheck", "--min-states", "--min-transitions", "--min-guards", "--min-nodes", "--min-edges"];
    let skip: Vec<usize> = args.iter().enumerate().filter(|(_, a)| valued.contains(&a.as_str())).map(|(i, _)| i + 1).collect();
    let Some(name) = args.iter().enumerate().find(|(i, a)| !a.starts_with("--") && !skip.contains(i)).map(|(_, a)| a.clone()) else {
        eprintln!("Usage: syscribe --model <root> simulate <TestCase|StateDef> [--scenario <title>] [--do \"<script>\"] [--trace] [--record [--run <id>] [--config <C>]] [--strict] [--json]");
        return 1;
    };
    let config = value("--config");
    let projected;
    let active: &[RawElement] = match &config {
        Some(c) => {
            projected = lens(Some(c));
            &projected
        }
        None => elems,
    };
    let resolver = syscribe_model::resolver::Resolver::new(active);
    let Some(target) = resolver.resolve_ref(active, &name) else {
        eprintln!("error: no element '{name}'");
        return 1;
    };
    let is_action = |e: &RawElement| matches!(e.frontmatter.element_type, Some(ElementType::ActionDef) | Some(ElementType::Action));
    let is_composite = |e: &RawElement| matches!(e.frontmatter.element_type, Some(ElementType::Part) | Some(ElementType::PartDef));
    let is_machine = |e: &RawElement| matches!(e.frontmatter.element_type, Some(ElementType::StateDef) | Some(ElementType::State)) || is_action(e) || is_composite(e);
    if (is_machine(target) && !is_composite(target)) && flag("--do") {
        return if is_action(target) { simulate_action_script(active, args) } else { simulate_script(active, args, model_root) };
    }
    // The scenario TestCases to run.
    let tcs: Vec<&RawElement> = match target.frontmatter.element_type {
        Some(ElementType::TestCase) => vec![target],
        _ if is_machine(target) => active
            .iter()
            .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::TestCase)))
            .filter(|e| subject_of(active, e).map(|s| s.qualified_name == target.qualified_name).unwrap_or(false))
            .collect(),
        _ => {
            eprintln!("error: '{name}' is neither a TestCase, a StateDef nor a composite Part");
            return 1;
        }
    };
    if flag("--crosscheck") {
        eprintln!("error: --crosscheck needs a script (`simulate <StateDef> --do \"…\" --crosscheck <engine>`); it is not available for scenarios");
        return 1;
    }
    if !flag("--coverage") && (flag("--min-states") || flag("--min-transitions") || flag("--min-guards") || flag("--min-nodes") || flag("--min-edges")) {
        eprintln!("error: --min-states, --min-transitions and --min-guards need --coverage");
        return 1;
    }
    let filter = value("--scenario");
    let strict = flag("--strict");
    let opts = syscribe_model::sm_run::Options { strict, ..Default::default() };
    struct Out {
        tc: String,
        machine: String,
        hash: String,
        v: syscribe_model::sm_scen::Verdict,
    }
    let mut outs: Vec<Out> = Vec::new();
    let mut lowered: std::collections::HashMap<String, syscribe_model::ir::Machine> = Default::default();
    let mut lowered_act: std::collections::HashMap<String, syscribe_model::act_ir::ActionProgram> = Default::default();
    let mut composites: std::collections::HashMap<String, syscribe_model::system_run::SystemModel> = Default::default();
    let mut exit = 0;
    for tc in tcs {
        let tc_id = tc.frontmatter.id.clone().unwrap_or_else(|| tc.qualified_name.clone());
        let scenarios: Vec<_> = extract_scenarios(&tc.doc)
            .into_iter()
            .filter(|sc| filter.as_ref().is_none_or(|f| &sc.title == f || sc.outline.as_ref() == Some(f)))
            .collect();
        if scenarios.is_empty() {
            continue;
        }
        let subject = match subject_of(active, tc) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {tc_id}: {e}");
                exit = 2;
                continue;
            }
        };
        if is_composite(subject) {
            if !composites.contains_key(&subject.qualified_name) {
                match syscribe_model::system_run::SystemModel::build_in(active, &subject.qualified_name, model_root) {
                    Ok(m) => {
                        composites.insert(subject.qualified_name.clone(), m);
                    }
                    Err(errs) => {
                        for e in &errs {
                            eprintln!("error: {tc_id}: {e}");
                        }
                        if errs.iter().any(|e| e.contains("no FMI support")) {
                            return 1;
                        }
                        exit = 2;
                        continue;
                    }
                }
            }
            let sm = &composites[&subject.qualified_name];
            for sc in &scenarios {
                outs.push(Out { tc: tc_id.clone(), machine: sm.spec.composite.clone(), hash: sm.hash.clone(), v: syscribe_model::system_scen::run_system_scenario(sm, sc, &opts) });
            }
            continue;
        }
        if is_action(subject) {
            if !lowered_act.contains_key(&subject.qualified_name) {
                match syscribe_model::act_ir::lower_action(active, &subject.qualified_name) {
                    Ok(p) => {
                        lowered_act.insert(subject.qualified_name.clone(), p);
                    }
                    Err(diags) => {
                        for (c, msg) in &diags {
                            eprintln!("{c}: {msg}");
                        }
                        exit = 2;
                        continue;
                    }
                }
            }
            let p = &lowered_act[&subject.qualified_name];
            let aopts = syscribe_model::act_run::ActOptions { strict, ..Default::default() };
            for sc in &scenarios {
                outs.push(Out { tc: tc_id.clone(), machine: p.root.clone(), hash: p.hash.clone(), v: syscribe_model::act_scen::run_action_scenario(p, sc, &aopts) });
            }
            continue;
        }
        if !lowered.contains_key(&subject.qualified_name) {
            match syscribe_model::ir::lower(active, &subject.qualified_name) {
                Ok(m) => {
                    lowered.insert(subject.qualified_name.clone(), m);
                }
                Err(diags) => {
                    for (c, msg) in &diags {
                        eprintln!("{c}: {msg}");
                    }
                    exit = 2;
                    continue;
                }
            }
        }
        let m = &lowered[&subject.qualified_name];
        let factory = match plant_factory(active, tc, m, model_root) {
            Ok(f) => f,
            Err((code, msg)) => {
                eprintln!("error: {tc_id}: {msg}");
                return code;
            }
        };
        for sc in &scenarios {
            let v = syscribe_model::sm_scen::run_scenario_with(m, sc, &opts, factory.as_ref().map(|f| f.as_ref()));
            outs.push(Out { tc: tc_id.clone(), machine: m.machine.clone(), hash: m.hash.clone(), v });
        }
    }
    if outs.is_empty() {
        if exit == 0 {
            eprintln!("error: no scenario to run{}", filter.map(|f| format!(" matching '{f}'")).unwrap_or_default());
            return 1;
        }
        return exit;
    }
    if outs.iter().any(|o| o.v.status != Status::Pass) {
        exit = 2;
    }
    // Coverage (GH #281): merged per machine over the scenarios that ran.
    let mut coverage_json: Vec<serde_json::Value> = Vec::new();
    let mut coverage_text = String::new();
    if flag("--coverage") {
        let pct = |f: &str| -> Result<Option<f64>, String> {
            match value(f) {
                None => Ok(None),
                Some(v) => v.parse::<f64>().map(Some).map_err(|_| format!("{f} needs a percentage, got '{v}'")),
            }
        };
        let (min_s, min_t, min_g) = match (pct("--min-states"), pct("--min-transitions"), pct("--min-guards")) {
            (Ok(a), Ok(b), Ok(c)) => (a, b, c),
            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        let mut machines: Vec<String> = outs.iter().map(|o| o.machine.clone()).collect();
        machines.dedup();
        machines.sort();
        machines.dedup();
        for mname in machines {
            let mine: Vec<&Out> = outs.iter().filter(|o| o.machine == mname).collect();
            if let Some(sm) = composites.get(&mname) {
                let reqs = verified_requirements(active, &resolver, &mine.iter().map(|o| (o.tc.as_str(), o.v.status == Status::Pass)).collect::<Vec<_>>());
                let (mut agg_s, mut agg_t, mut agg_g) = ((0usize, 0usize), (0usize, 0usize), (0usize, 0usize));
                let mut parts_json: Vec<serde_json::Value> = Vec::new();
                for (pi, inst) in sm.spec.instances.iter().enumerate() {
                    let Some(m) = sm.machines[pi].as_ref() else { continue };
                    let mut cov = syscribe_model::sm_run::Coverage::default();
                    for o in &mine {
                        if let Some(pc) = o.v.part_coverage.iter().find(|c| c.part == inst.name) {
                            cov.merge(&pc.coverage);
                        }
                    }
                    let rep = syscribe_model::sm_cov::report(m, &cov, mine.len(), config.clone(), &reqs);
                    let label = format!("{}.{} ({})", mname, inst.name, m.machine);
                    // GH #332: the gates judge the product, not the test set-up
                    for (what, got, min) in [("states", rep.states_percent(), min_s), ("transitions", rep.transitions_percent(), min_t), ("guards", rep.guards_percent(), min_g)].into_iter().filter(|_| !inst.environment) {
                        if let Some(min) = min {
                            if got + 1e-9 < min {
                                eprintln!("coverage gate: {mname}.{} {what} {:.1}% is below the required {min}%", inst.name, got);
                                exit = 2;
                            }
                        }
                    }
                    // GH #332: an environment part is not the product, so it is not in the aggregate
                    if !inst.environment {
                        agg_s = (agg_s.0 + rep.states.covered, agg_s.1 + rep.states.total);
                        agg_t = (agg_t.0 + rep.transitions.covered, agg_t.1 + rep.transitions.total);
                        agg_g = (agg_g.0 + rep.guards.covered, agg_g.1 + rep.guards.total);
                    }
                    coverage_text.push_str(&format!(
                        "coverage of {label}{} (hash {}, {} scenario(s)):\n  states        {}/{}  {:.0}%\n  transitions   {}/{}  {:.0}%\n  pairs         {}/{}\n  guard conds   {}/{}  {:.0}%\n  timers        {}/{}\n",
                        inst.role.map(|r| format!(" [{r}]")).unwrap_or_default(),
                        &rep.basis.machine_hash[..12], rep.basis.scenarios,
                        rep.states.covered, rep.states.total, rep.states_percent(),
                        rep.transitions.covered, rep.transitions.total, rep.transitions_percent(),
                        rep.pairs.covered, rep.pairs.total,
                        rep.guards.covered, rep.guards.total, rep.guards_percent(),
                        rep.timers.covered, rep.timers.total,
                    ));
                    for u in &rep.states.uncovered {
                        coverage_text.push_str(&format!("    state not entered: {u}\n"));
                    }
                    for u in &rep.transitions.uncovered {
                        coverage_text.push_str(&format!("    transition not taken: t{} {}\n", u.id, u.description));
                    }
                    for u in &rep.timers.uncovered {
                        coverage_text.push_str(&format!("    timer: {}\n", u.description));
                    }
                    for r in &rep.requirements.uncovered {
                        coverage_text.push_str(&format!("    requirement not verified by a passing scenario: {r}\n"));
                    }
                    let mut pj = serde_json::to_value(&rep).expect("serialises");
                    pj["part"] = serde_json::json!(inst.name);
                    pj["role"] = serde_json::json!(inst.role.unwrap_or("product"));
                    pj["basis"]["machine"] = serde_json::json!(label);
                    parts_json.push(pj);
                }
                let env: Vec<&str> = sm.spec.instances.iter().filter(|i| i.environment).map(|i| i.name.as_str()).collect();
                let env_note = if env.is_empty() { String::new() } else { format!(" (product parts only; not counted, as not the product: {})", env.join(", ")) };
                coverage_text.push_str(&format!("aggregate coverage of {mname} (hash {}): states {}/{}, transitions {}/{}, guard conds {}/{}{env_note}\n", &sm.hash[..12], agg_s.0, agg_s.1, agg_t.0, agg_t.1, agg_g.0, agg_g.1));
                coverage_json.push(serde_json::json!({
                    "composite": mname, "hash": sm.hash, "parts": parts_json, "environment": env,
                    "aggregate": {
                        "states": {"covered": agg_s.0, "total": agg_s.1},
                        "transitions": {"covered": agg_t.0, "total": agg_t.1},
                        "guards": {"covered": agg_g.0, "total": agg_g.1},
                    },
                }));
                continue;
            }
            let act_prog = lowered_act.values().find(|p| p.root == mname);
            let mut cov = syscribe_model::sm_run::Coverage::default();
            let mut acov = syscribe_model::act_run::ActCoverage::default();
            for o in &mine {
                cov.merge(&o.v.coverage);
                acov.merge(&o.v.act_coverage);
            }
            // Requirements verified by the scenario TestCases of this machine: verified when every scenario of
            // one verifying TestCase passed.
            let reqs = verified_requirements(active, &resolver, &mine.iter().map(|o| (o.tc.as_str(), o.v.status == Status::Pass)).collect::<Vec<_>>());
            if let Some(p) = act_prog {
                let rep = syscribe_model::act_cov::report(p, &acov, mine.len(), config.clone(), &reqs);
                let min_n = value("--min-nodes").and_then(|v| v.parse::<f64>().ok());
                let min_e = value("--min-edges").and_then(|v| v.parse::<f64>().ok());
                for (what, got, min) in [("nodes", rep.nodes_percent(), min_n), ("edges", rep.edges_percent(), min_e), ("guards", rep.guards_percent(), min_g)] {
                    if let Some(min) = min {
                        if got + 1e-9 < min {
                            eprintln!("coverage gate: {what} {:.1}% is below the required {min}%", got);
                            exit = 2;
                        }
                    }
                }
                coverage_text.push_str(&format!(
                    "coverage of {} (hash {}, {} scenario(s)):\n  nodes         {}/{}  {:.0}%\n  edges         {}/{}  {:.0}%\n  decisions     {}/{}\n  loops         {}/{}\n  guard conds   {}/{}  {:.0}%\n",
                    rep.basis.machine, &rep.basis.machine_hash[..12], rep.basis.scenarios,
                    rep.nodes.covered, rep.nodes.total, rep.nodes_percent(),
                    rep.edges.covered, rep.edges.total, rep.edges_percent(),
                    rep.decisions.covered, rep.decisions.total,
                    rep.loops.covered, rep.loops.total,
                    rep.guards.covered, rep.guards.total, rep.guards_percent(),
                ));
                for u in &rep.nodes.uncovered {
                    coverage_text.push_str(&format!("    node not fired: {u}\n"));
                }
                for u in &rep.edges.uncovered {
                    coverage_text.push_str(&format!("    succession not taken: {}\n", u.description));
                }
                for u in &rep.decisions.uncovered {
                    coverage_text.push_str(&format!("    decision branch not taken: {}\n", u.description));
                }
                for u in &rep.loops.uncovered {
                    coverage_text.push_str(&format!("    loop: {u}\n"));
                }
                for r in &rep.requirements.uncovered {
                    coverage_text.push_str(&format!("    requirement not verified by a passing scenario: {r}\n"));
                }
                coverage_json.push(serde_json::to_value(&rep).expect("serialises"));
                continue;
            }
            let m = lowered.values().find(|m| m.machine == mname).expect("lowered");
            let rep = syscribe_model::sm_cov::report(m, &cov, mine.len(), config.clone(), &reqs);
            for (what, got, min) in [("states", rep.states_percent(), min_s), ("transitions", rep.transitions_percent(), min_t), ("guards", rep.guards_percent(), min_g)] {
                if let Some(min) = min {
                    if got + 1e-9 < min {
                        eprintln!("coverage gate: {what} {:.1}% is below the required {min}%", got);
                        exit = 2;
                    }
                }
            }
            coverage_text.push_str(&format!(
                "coverage of {} (hash {}, {} scenario(s)):\n  states        {}/{}  {:.0}%\n  transitions   {}/{}  {:.0}%\n  pairs         {}/{}\n  guard conds   {}/{}  {:.0}%\n  timers        {}/{}\n",
                rep.basis.machine, &rep.basis.machine_hash[..12], rep.basis.scenarios,
                rep.states.covered, rep.states.total, rep.states_percent(),
                rep.transitions.covered, rep.transitions.total, rep.transitions_percent(),
                rep.pairs.covered, rep.pairs.total,
                rep.guards.covered, rep.guards.total, rep.guards_percent(),
                rep.timers.covered, rep.timers.total,
            ));
            for u in &rep.states.uncovered {
                coverage_text.push_str(&format!("    state not entered: {u}\n"));
            }
            for u in &rep.transitions.uncovered {
                coverage_text.push_str(&format!("    transition not taken: t{} {}\n", u.id, u.description));
            }
            for u in &rep.timers.uncovered {
                coverage_text.push_str(&format!("    timer: {}\n", u.description));
            }
            for r in &rep.requirements.uncovered {
                coverage_text.push_str(&format!("    requirement not verified by a passing scenario: {r}\n"));
            }
            coverage_json.push(serde_json::to_value(&rep).expect("serialises"));
        }
    }
    if flag("--json") {
        let verdicts: Vec<_> = outs
            .iter()
            .map(|o| {
                let mut j = serde_json::json!({
                    "testCase": o.tc, "scenario": o.v.scenario, "outline": o.v.outline, "status": o.v.status,
                    "message": o.v.message, "machine": o.machine, "machineHash": o.hash, "steps": o.v.steps,
                });
                if !o.v.plants.is_empty() {
                    j["plants"] = serde_json::Value::Array(o.v.plants.clone());
                }
                if !o.v.messages.is_empty() {
                    j["messages"] = serde_json::json!(o.v.messages);
                }
                if flag("--trace") {
                    j["trace"] = serde_json::to_value(&o.v.trace).unwrap_or_default();
                }
                j
            })
            .collect();
        let hash = outs.first().map(|o| o.hash.clone()).unwrap_or_default();
        let mut doc = serde_json::json!({ "machineHash": hash, "verdicts": verdicts });
        match coverage_json.len() {
            0 => {}
            1 => doc["coverage"] = coverage_json.remove(0),
            _ => doc["coverages"] = serde_json::Value::Array(coverage_json),
        }
        println!("{}", serde_json::to_string_pretty(&doc).expect("serialises"));
    } else {
        for o in &outs {
            let tag = match o.v.status {
                Status::Pass => "PASS ",
                Status::Fail => "FAIL ",
                Status::Error => "ERROR",
            };
            println!("{tag}  {}  {}", o.tc, o.v.scenario);
            if let Some(m) = &o.v.message {
                println!("       {m}");
            }
            if flag("--trace") {
                for t in &o.v.trace {
                    println!("       {:>3} t={:<6} {:<22} {}", t.n, syscribe_model::expr::format_number(t.time), t.cause, t.note.clone().unwrap_or_else(|| match (&t.from, &t.to) {
                        (Some(f), Some(to)) => format!("{f} → {to}"),
                        (Some(f), None) => format!("{f} (internal)"),
                        _ => String::new(),
                    }));
                }
            }
        }
        let pass = outs.iter().filter(|o| o.v.status == Status::Pass).count();
        println!("{pass}/{} scenario(s) pass", outs.len());
        print!("{coverage_text}");
    }
    if flag("--record") {
        // The current number of example rows of every outline of the TestCases that ran.
        let mut rows: std::collections::BTreeMap<(String, String), usize> = Default::default();
        for o in &outs {
            if o.v.outline.is_some() {
                if let Some(tc) = active.iter().find(|e| e.frontmatter.id.as_deref() == Some(o.tc.as_str()) || e.qualified_name == o.tc) {
                    for sc in extract_scenarios(&tc.doc) {
                        if let Some(ol) = sc.outline {
                            *rows.entry((o.tc.clone(), ol)).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        rows.iter_mut().for_each(|(_, n)| *n = (*n).max(1));
        if let Err(e) = record_scenarios(model_root, elems, &outs.iter().map(|o| (o.tc.as_str(), &o.hash, &o.v)).collect::<Vec<_>>(), value("--run").as_deref(), config.as_deref(), &rows) {
            eprintln!("error: {e}");
            return 1;
        }
    }
    exit
}

/// Write scenario verdicts as session-log evidence (kept alongside manual session-log verdicts) and the
/// traces under `.syscribe/traces/` (REQ-TRS-SMSCEN-001).
fn record_scenarios(
    model_root: &std::path::Path,
    elems: &[RawElement],
    outs: &[(&str, &String, &syscribe_model::sm_scen::Verdict)],
    run: Option<&str>,
    config: Option<&str>,
    outline_rows: &std::collections::BTreeMap<(String, String), usize>,
) -> Result<(), String> {
    use syscribe_model::results::{ResultsData, RunHistory};
    use syscribe_model::sm_scen::Status;
    let config_key: Option<String> = match config {
        None => None,
        Some(c) => Some(
            elems
                .iter()
                .find(|e| {
                    matches!(e.frontmatter.element_type, Some(syscribe_model::element::ElementType::Configuration))
                        && (e.frontmatter.id.as_deref() == Some(c) || e.qualified_name == c)
                })
                .map(|e| e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone()))
                .ok_or_else(|| format!("--config '{c}' is not a Configuration of this model; nothing was written"))?,
        ),
    };
    let result = |s: Status| if s == Status::Pass { "pass" } else { "fail" };
    // One record per scenario that ran; an outline's own title is derived below from all of its rows.
    let mut records: Vec<serde_json::Value> = Vec::new();
    for (tc, _, v) in outs {
        let steps = if v.steps.is_empty() { vec!["(no steps)".to_string()] } else { v.steps.clone() };
        records.push(serde_json::json!({"testCase": tc, "scenario": v.scenario, "steps": steps, "result": result(v.status), "timestamp": ""}));
    }
    let text = serde_json::to_string(&records).map_err(|e| e.to_string())?;
    let ran = ResultsData::parse_session_log(&text, "syscribe simulate --record")?;
    let mut data = ran.clone();
    // Keep the verdicts of scenarios this run did not cover (manual session logs, other TestCases), drop
    // example rows an outline no longer has, and derive each touched outline's own verdict from all its rows.
    if let Some(old) = ResultsData::load_sidecar(model_root) {
        let base = match &config_key {
            Some(c) => old.by_config.get(c).map(|x| x.by_scenario.clone()).unwrap_or_default(),
            None => old.by_scenario.clone(),
        };
        for (k, v) in base {
            data.by_scenario.entry(k).or_insert(v);
        }
    }
    for ((tc, outline), n) in outline_rows {
        let prefix = format!("{tc}::{outline} [row ");
        data.by_scenario.retain(|k, _| {
            k.strip_prefix(&prefix).and_then(|r| r.strip_suffix(']')).and_then(|r| r.parse::<usize>().ok()).is_none_or(|row| row <= *n)
        });
        let rows: Vec<syscribe_model::results::Verdict> = data.by_scenario.iter().filter(|(k, _)| k.starts_with(&prefix)).map(|(_, v)| *v).collect();
        use syscribe_model::results::Verdict as V;
        let worst = if rows.contains(&V::Fail) { V::Fail } else if rows.contains(&V::Flaky) { V::Flaky } else if rows.is_empty() { V::Ignored } else { V::Pass };
        data.by_scenario.insert(format!("{tc}::{outline}"), worst);
    }
    let mut history = None;
    if let Some(r) = run {
        // The run history records what this run ran, not verdicts carried over from earlier runs.
        let mut h = RunHistory::load(model_root)?;
        let mut only = ran.clone();
        for ((tc, outline), _) in outline_rows {
            let prefix = format!("{tc}::{outline} [row ");
            let rows: Vec<_> = only.by_scenario.iter().filter(|(k, _)| k.starts_with(&prefix)).map(|(_, v)| *v).collect();
            if !rows.is_empty() {
                use syscribe_model::results::Verdict as V;
                only.by_scenario.insert(format!("{tc}::{outline}"), if rows.contains(&V::Fail) { V::Fail } else { V::Pass });
            }
        }
        h.record_for(r, &only, config_key.as_deref());
        history = Some(h);
    }
    match &config_key {
        Some(c) => data.merge_config_into_sidecar(model_root, c),
        None => data.merge_into_sidecar(model_root),
    }
    .map_err(|e| format!("cannot write the results sidecar: {e}"))?;
    if let Some(h) = history {
        h.save(model_root).map_err(|e| format!("cannot write run history: {e}"))?;
    }
    let dir = model_root.join(".syscribe").join("traces");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (tc, hash, v) in outs {
        let slug: String = v.scenario.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect();
        let tag = format!("{:08x}", v.scenario.bytes().fold(0x811c9dc5u32, |h, b| (h ^ b as u32).wrapping_mul(0x01000193)));
        let file = dir.join(format!("{tc}__{slug}-{tag}.json"));
        let body = serde_json::json!({"testCase": tc, "scenario": v.scenario, "status": v.status, "message": v.message, "machineHash": hash, "steps": v.steps, "trace": v.trace});
        let mut body = body;
        if !v.plants.is_empty() {
            body["plants"] = serde_json::Value::Array(v.plants.clone());
        }
        if !v.messages.is_empty() {
            body["messages"] = serde_json::json!(v.messages);
        }
        std::fs::write(&file, serde_json::to_string_pretty(&body).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    }
    println!("Recorded {} scenario verdict(s){}.", outs.len(), run.map(|r| format!(" in run '{r}'")).unwrap_or_default());
    Ok(())
}

/// Entry point for `mutate <StateDef|TestCase> [--min-score <pct>] [--json]` (GH #282): seed faults into the
/// machine's IR and re-run its scenarios. Exit 0 · 1 usage · 2 below `--min-score`, or the scenarios do not
/// pass on the unmutated machine.
pub fn cmd_mutate(elems: &[RawElement], args: &[String], model_root: &std::path::Path) -> i32 {
    use syscribe_model::element::ElementType;
    use syscribe_model::sm_scen::{extract_scenarios, subject_of};
    let flag = |f: &str| args.iter().any(|a| a == f);
    let value = |f: &str| args.windows(2).find(|w| w[0] == f).map(|w| w[1].clone());
    let skip: Vec<usize> = args.iter().enumerate().filter(|(_, a)| *a == "--min-score" || *a == "--part").map(|(i, _)| i + 1).collect();
    let Some(name) = args.iter().enumerate().find(|(i, a)| !a.starts_with("--") && !skip.contains(i)).map(|(_, a)| a.clone()) else {
        eprintln!("Usage: syscribe --model <root> mutate <StateDef|TestCase|composite> [--part <name>] [--min-score <percent>] [--record] [--json]");
        return 1;
    };
    let min = match value("--min-score").map(|v| v.parse::<f64>()) {
        None => None,
        Some(Ok(v)) => Some(v),
        Some(Err(_)) => {
            eprintln!("error: --min-score needs a percentage");
            return 1;
        }
    };
    let resolver = syscribe_model::resolver::Resolver::new(elems);
    let Some(target) = resolver.resolve_ref(elems, &name) else {
        eprintln!("error: no element '{name}'");
        return 1;
    };
    let is_composite = |e: &RawElement| matches!(e.frontmatter.element_type, Some(ElementType::Part) | Some(ElementType::PartDef));
    let is_machine = |e: &RawElement| matches!(e.frontmatter.element_type, Some(ElementType::StateDef) | Some(ElementType::State) | Some(ElementType::ActionDef) | Some(ElementType::Action)) || is_composite(e);
    let (machine_q, tcs): (String, Vec<&RawElement>) = if matches!(target.frontmatter.element_type, Some(ElementType::TestCase)) {
        match subject_of(elems, target) {
            Ok(s) => (s.qualified_name.clone(), vec![target]),
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        }
    } else if is_machine(target) {
        let tcs = elems
            .iter()
            .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::TestCase)))
            .filter(|e| subject_of(elems, e).map(|s| s.qualified_name == target.qualified_name).unwrap_or(false))
            .collect();
        (target.qualified_name.clone(), tcs)
    } else {
        eprintln!("error: '{name}' is neither a StateDef, a composite part nor a TestCase");
        return 1;
    };
    if resolver.resolve_ref(elems, &machine_q).is_some_and(is_composite) {
        return mutate_composite(elems, &machine_q, &tcs, min, value("--part").as_deref(), flag("--json"), flag("--record"), model_root);
    }
    if value("--part").is_some() {
        eprintln!("error: --part applies to a composite subject");
        return 1;
    }
    if tcs.iter().any(|tc| syscribe_model::plant_cfg::declares_plants(&tc.frontmatter)) {
        eprintln!("error: mutation analysis of scenarios that use plants is not supported yet (each mutant would need its own co-simulation)");
        return 1;
    }
    let scenarios: Vec<_> = tcs.iter().flat_map(|tc| extract_scenarios(&tc.doc)).collect();
    if scenarios.is_empty() {
        eprintln!("error: no scenarios to run against {machine_q}");
        return 1;
    }
    let subject_is_action = resolver.resolve_ref(elems, &machine_q).is_some_and(|e| matches!(e.frontmatter.element_type, Some(ElementType::ActionDef) | Some(ElementType::Action)));
    let analysed = if subject_is_action {
        match syscribe_model::act_ir::lower_action(elems, &machine_q) {
            Ok(p) => syscribe_model::act_mut::analyse(&p, &scenarios, &syscribe_model::act_run::ActOptions::default()),
            Err(diags) => {
                for (c, msg) in &diags {
                    eprintln!("{c}: {msg}");
                }
                return 2;
            }
        }
    } else {
        match syscribe_model::ir::lower(elems, &machine_q) {
            Ok(m) => syscribe_model::sm_mut::analyse(&m, &scenarios, &syscribe_model::sm_run::Options::default()),
            Err(diags) => {
                for (c, msg) in &diags {
                    eprintln!("{c}: {msg}");
                }
                return 2;
            }
        }
    };
    let report = match analysed {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    if flag("--json") {
        println!("{}", serde_json::to_string_pretty(&report).expect("serialises"));
    } else {
        println!(
            "mutation score {:.1}%  ({} killed, {} survived, {} equivalent-suspect) over {} scenario(s) on {} (hash {})",
            report.score, report.killed, report.survived, report.equivalent_suspect, report.scenarios, report.machine, &report.machine_hash[..12]
        );
        for m in report.mutants.iter().filter(|m| m.fate == "survived") {
            println!("  SURVIVED  {}  [{} at {}]{}", m.description, m.operator, m.origin.path, m.closest.as_ref().map(|c| format!("  (closest: {c})")).unwrap_or_default());
        }
        for m in report.mutants.iter().filter(|m| m.fate == "equivalentSuspect") {
            println!("  uncovered {}  [{}]", m.description, m.operator);
        }
        if !report.uncovered.states.is_empty() {
            println!("  states never entered: {}", report.uncovered.states.join(", "));
        }
    }
    if flag("--record") {
        let rec = syscribe_model::exec_evidence::MutationRecord { subject: machine_q.clone(), machine: report.machine.clone(), machine_hash: report.machine_hash.clone(), score: report.score, killed: report.killed, survived: report.survived, recorded_at_unix: now_unix() };
        if let Err(code) = record_scores(model_root, vec![rec], flag("--json")) {
            return code;
        }
    }
    if let Some(min) = min {
        if report.score + 1e-9 < min {
            eprintln!("mutation gate: score {:.1}% is below the required {min}%", report.score);
            return 2;
        }
    }
    0
}

/// Run an external SCXML engine on the exported document and compare its configurations with ours.
/// Driver contract: the command (from `[scxml.engines.<name>] command` in `.syscribe.toml`, else the argument
/// itself) is run with `sh -c`; it reads `{"scxml": "<document>", "commands": [{"event", "data"} |
/// {"advance": seconds} | {"set": {var: value}}]}` on stdin and prints `{"configurations": [[state ids]…]}` —
/// the active states after start and after each command.
fn crosscheck_engine(machine: &syscribe_model::ir::Machine, engine: &str, wire: &[serde_json::Value], ours: &[Vec<String>], model_root: &std::path::Path) -> Result<(), String> {
    use std::io::Write;
    let command = std::fs::read_to_string(model_root.join(".syscribe.toml"))
        .ok()
        .and_then(|t| t.parse::<toml::Value>().ok())
        .and_then(|v| v.get("scxml")?.get("engines")?.get(engine)?.get("command")?.as_str().map(String::from))
        .unwrap_or_else(|| engine.to_string());
    let (doc, _) = syscribe_model::sm_scxml::export(machine);
    let input = serde_json::to_string(&serde_json::json!({"scxml": doc, "commands": wire})).map_err(|e| e.to_string())?;
    let mut child = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot start the engine '{command}': {e}"))?;
    // Write on a thread so an engine that answers before it has read everything cannot deadlock the pipes.
    let mut stdin = child.stdin.take().expect("piped");
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(input.as_bytes());
    });
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let _ = writer.join();
    if !out.status.success() {
        return Err(format!("the engine failed ({}): {}", out.status, String::from_utf8_lossy(&out.stderr).trim()));
    }
    let reply: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| format!("the engine's output is not JSON: {e}"))?;
    let theirs = reply["configurations"].as_array().ok_or("the engine's output has no `configurations` array")?;
    if theirs.len() != ours.len() {
        return Err(format!("the engine returned {} configuration(s), expected {}", theirs.len(), ours.len()));
    }
    let known: std::collections::BTreeSet<String> = machine.states.iter().map(|s| s.path.clone()).collect();
    for (i, (o, t)) in ours.iter().zip(theirs).enumerate() {
        let mut a: Vec<String> = o.clone();
        let mut b: Vec<String> = t.as_array().map(|x| x.iter().filter_map(|v| v.as_str().map(String::from)).filter(|s| known.contains(s)).collect()).unwrap_or_default();
        a.sort();
        b.sort();
        if a != b {
            let what = if i == 0 { "after start".to_string() } else { format!("after command {i}: {}", wire[i - 1]) };
            return Err(format!("the active states differ {what}: syscribe {a:?}, engine {b:?}"));
        }
    }
    Ok(())
}

/// Entry point for `export-scxml <StateDef> [--output <file>]` (GH #283). Exit 0 · 1 usage · 2 diagnostics.
pub fn cmd_export_scxml(elems: &[RawElement], args: &[String]) -> i32 {
    let out = args.windows(2).find(|w| w[0] == "--output").map(|w| w[1].clone());
    let skip: Vec<usize> = args.iter().enumerate().filter(|(_, a)| *a == "--output").map(|(i, _)| i + 1).collect();
    let Some(name) = args.iter().enumerate().find(|(i, a)| !a.starts_with("--") && !skip.contains(i)).map(|(_, a)| a) else {
        eprintln!("Usage: syscribe --model <root> export-scxml <StateDef> [--output <file>]");
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
    let (doc, warnings) = syscribe_model::sm_scxml::export(&machine);
    for w in &warnings {
        eprintln!("note: {w}");
    }
    match out {
        Some(f) => {
            if let Err(e) = std::fs::write(&f, &doc) {
                eprintln!("error: cannot write {f}: {e}");
                return 1;
            }
            println!("Wrote {f} ({} state(s), {} transition(s), {} note(s))", machine.states.len(), machine.transitions.len(), warnings.len());
        }
        None => print!("{doc}"),
    }
    0
}

// ── constraints calc / sweep (GH #274) ───────────────────────────────────────

use syscribe_model::constraints::{calc_params, coerce_param, evaluate_value};

/// `constraints calc <CalculationDef> [--config <C>] [--set name=value]... [--json]`
fn cmd_calc(args: &[String], lens: &dyn Fn(Option<&str>) -> Vec<RawElement>) -> i32 {
    let pos = args.iter().enumerate().position(|(i, a)| !a.starts_with("--") && !(i > 0 && matches!(args[i - 1].as_str(), "--set" | "--config")));
    let Some(pos) = pos else { return usage_err("calc needs a CalculationDef") };
    let name = &args[pos];
    let rest: Vec<String> = args.iter().enumerate().filter(|(i, _)| *i != pos).map(|(_, a)| a.clone()).collect();
    let o = match parse_opts(&rest) {
        Ok(o) => o,
        Err(m) => return usage_err(&m),
    };
    let view = lens(o.config.as_deref());
    let resolver = Resolver::new(&view);
    let Some(def) = resolver.resolve_ref(&view, name).filter(|d| d.frontmatter.element_type == Some(ElementType::CalculationDef)) else {
        return usage_err(&format!("'{name}' is not a CalculationDef of the model"));
    };
    if !matches!(def.frontmatter.body_language.as_deref(), None | Some("budget") | Some("syscribe")) {
        return usage_err(&format!("the body of {} is written in {}, which syscribe does not evaluate", def.qualified_name, def.frontmatter.body_language.as_deref().unwrap_or("")));
    }
    let Some(body) = def.frontmatter.body.as_deref() else { return usage_err(&format!("{} has no body", def.qualified_name)) };
    let mut env = build_env(&view, o.config.as_deref(), &o.sets);
    let mut bound: BTreeMap<String, String> = BTreeMap::new();
    for (pname, ty, unit, default) in calc_params(def) {
        let given = o.sets.iter().find(|(k, _)| *k == pname).map(|(_, v)| v.clone());
        let v = match given {
            Some(v) => Some(v),
            None => default.and_then(|d| evaluate_value(&view, &d, &syscribe_model::constraints::Env::new()).ok()),
        };
        if let Some(v) = v {
            let v = coerce_param(v, ty.as_deref(), unit.as_deref());
            bound.insert(pname.clone(), format_value(&v));
            env.insert(pname, v);
        }
    }
    match evaluate_value(&view, body, &env) {
        Ok(v) => {
            if o.json {
                let (value, unit, dim) = match &v {
                    Value::Num(q) => (json!(q.v), json!(unit_label(&q.dim)), q.dim.human()),
                    Value::Bool(b) => (json!(b), json!(null), "n/a".into()),
                    Value::Str(s) => (json!(s), json!(null), "n/a".into()),
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({"calculation": def.qualified_name, "value": value, "unit": unit, "dimension": dim, "bindings": bound})).unwrap_or_default()
                );
            } else {
                println!("{}", format_value(&v));
            }
            0
        }
        Err(e) => {
            if o.json {
                println!("{}", serde_json::to_string_pretty(&json!({"calculation": def.qualified_name, "error": e, "bindings": bound})).unwrap_or_default());
            } else {
                eprintln!("constraints calc: {e}");
            }
            2
        }
    }
}

const MAX_SWEEP_POINTS: usize = 5000;

/// One `--param` range: `lo..hi:step` (inclusive) or a list `a,b,c`.
fn parse_range(spec: &str) -> Result<(String, Vec<Value>), String> {
    let (name, range) = spec.split_once('=').filter(|(n, _)| !n.trim().is_empty()).ok_or_else(|| format!("--param expects name=range, got '{spec}'"))?;
    let name = name.trim().to_string();
    if let Some((lo, rest)) = range.split_once("..") {
        let (hi, step) = rest.split_once(':').ok_or_else(|| format!("--param {name}: a range is lo..hi:step"))?;
        let get = |t: &str, what: &str| parse_value(t.trim()).and_then(|v| if let Value::Num(q) = v { Some(q) } else { None }).ok_or_else(|| format!("--param {name}: {what} '{t}' is not a number"));
        let (lo, hi, step) = (get(lo, "the lower bound")?, get(hi, "the upper bound")?, get(step, "the step")?);
        if lo.dim != hi.dim || lo.dim != step.dim {
            return Err(format!("--param {name}: the bounds and the step must have the same dimension"));
        }
        if !(step.v > 0.0) {
            return Err(format!("--param {name}: the step must be positive"));
        }
        if hi.v < lo.v {
            return Err(format!("--param {name}: the upper bound is below the lower bound"));
        }
        let n = ((hi.v - lo.v) / step.v + 1e-9).floor() as usize + 1;
        if n > MAX_SWEEP_POINTS {
            return Err(format!("--param {name}: {n} values exceed the limit of {MAX_SWEEP_POINTS} points"));
        }
        Ok((name, (0..n).map(|i| Value::Num(syscribe_model::expr::Quantity { v: lo.v + step.v * i as f64, dim: lo.dim })).collect()))
    } else {
        let vals: Option<Vec<Value>> = range.split(',').map(|t| parse_value(t.trim())).collect();
        match vals {
            Some(v) if !v.is_empty() => Ok((name, v)),
            _ => Err(format!("--param {name}: '{range}' is neither lo..hi:step nor a comma-separated list of values")),
        }
    }
}

/// `constraints sweep --param name=<range> ... [--show '<expr>'] [--config <C>] [--set ...] [--json]`
fn cmd_sweep(args: &[String], lens: &dyn Fn(Option<&str>) -> Vec<RawElement>) -> i32 {
    let mut params: Vec<(String, Vec<Value>)> = Vec::new();
    let mut show: Option<String> = None;
    let mut rest: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--param" => {
                i += 1;
                let Some(spec) = args.get(i) else { return usage_err("--param expects name=range") };
                match parse_range(spec) {
                    Ok(p) => params.push(p),
                    Err(m) => return usage_err(&m),
                }
            }
            "--show" => {
                i += 1;
                let Some(e) = args.get(i) else { return usage_err("--show expects an expression") };
                show = Some(e.clone());
            }
            other => {
                rest.push(other.to_string());
                if matches!(other, "--set" | "--config") {
                    i += 1;
                    match args.get(i) {
                        Some(v) => rest.push(v.clone()),
                        None => return usage_err(&format!("{other} expects a value")),
                    }
                }
            }
        }
        i += 1;
    }
    if params.is_empty() {
        return usage_err("sweep needs at least one --param");
    }
    let o = match parse_opts(&rest) {
        Ok(o) => o,
        Err(m) => return usage_err(&m),
    };
    let total = params.iter().try_fold(1usize, |a, p| a.checked_mul(p.1.len()));
    match total {
        Some(t) if t <= MAX_SWEEP_POINTS => {}
        _ => return usage_err(&format!("the grid exceeds {MAX_SWEEP_POINTS} points")),
    }
    let view = lens(o.config.as_deref());
    let base_env = build_env(&view, o.config.as_deref(), &o.sets);
    // Cartesian product in --param order, the last parameter varying fastest.
    let mut points: Vec<Vec<usize>> = vec![vec![]];
    for (_, vals) in &params {
        points = points.into_iter().flat_map(|p| (0..vals.len()).map(move |k| { let mut q = p.clone(); q.push(k); q })).collect();
    }
    let alt_key = |k: &str| -> Option<String> {
        if let Some(pos) = k.rfind("::") {
            Some(format!("{}.{}", &k[..pos], &k[pos + 2..]))
        } else {
            k.rfind('.').map(|pos| format!("{}::{}", &k[..pos], &k[pos + 1..]))
        }
    };
    let mut rows: Vec<serde_json::Value> = Vec::new();
    for p in &points {
        let mut env = base_env.clone();
        let mut values: BTreeMap<String, String> = BTreeMap::new();
        for ((name, vals), k) in params.iter().zip(p) {
            let v = vals[*k].clone();
            values.insert(name.clone(), format_value(&v));
            if let Some(a) = alt_key(name) {
                env.insert(a, v.clone());
            }
            env.insert(name.clone(), v);
        }
        let results = check_all(&view, &env);
        let s = summary(&results);
        let shown = show.as_ref().map(|e| evaluate_value(&view, e, &env).map(|v| format_value(&v)).unwrap_or_else(|m| format!("({m})")));
        rows.push(json!({"values": values, "summary": {"pass": s["pass"], "fail": s["fail"], "unresolved": s["unresolved"], "error": s["error"]}, "show": shown}));
    }
    if o.json {
        println!("{}", serde_json::to_string_pretty(&json!({"params": params.iter().map(|p| p.0.clone()).collect::<Vec<_>>(), "points": rows})).unwrap_or_default());
    } else {
        let names: Vec<&str> = params.iter().map(|p| p.0.as_str()).collect();
        println!("| {} | pass | fail | unresolved | error |{}", names.join(" | "), if show.is_some() { " show |" } else { "" });
        println!("|{}---|---|---|---|{}", "---|".repeat(names.len()), if show.is_some() { "---|" } else { "" });
        for r in &rows {
            let vals: Vec<String> = names.iter().map(|n| r["values"][n].as_str().unwrap_or("").to_string()).collect();
            println!(
                "| {} | {} | {} | {} | {} |{}",
                vals.join(" | "),
                r["summary"]["pass"], r["summary"]["fail"], r["summary"]["unresolved"], r["summary"]["error"],
                r["show"].as_str().map(|s| format!(" {s} |")).unwrap_or_default()
            );
        }
    }
    0
}

/// `simulate <ActionDef> --do "<script>"`: `set <name>=<value>`, `run`, `send <event> [k=v …]`, `advance <duration>`.
fn simulate_action_script(elems: &[RawElement], args: &[String]) -> i32 {
    use syscribe_model::act_run::{ActOptions, ActRun, Status as RS};
    use syscribe_model::expr::{format_value, parse_value, Value};
    let json = args.iter().any(|a| a == "--json");
    let strict = args.iter().any(|a| a == "--strict");
    let script = args.windows(2).find(|w| w[0] == "--do").map(|w| w[1].clone()).unwrap_or_default();
    let skip: Vec<usize> = args.iter().enumerate().filter(|(_, a)| *a == "--do").map(|(i, _)| i + 1).collect();
    let Some(name) = args.iter().enumerate().find(|(i, a)| !a.starts_with("--") && !skip.contains(i)).map(|(_, a)| a) else {
        eprintln!("Usage: syscribe --model <root> simulate <ActionDef> --do \"set <name>=<value>; run; send <event> [k=v ...]; advance <duration>\" [--strict] [--json]");
        return 1;
    };
    let prog = match syscribe_model::act_ir::lower_action(elems, name) {
        Ok(p) => p,
        Err(diags) => {
            for (c, msg) in &diags {
                eprintln!("{c}: {msg}");
            }
            return if diags.iter().all(|(c, _)| *c == "E001") { 1 } else { 2 };
        }
    };
    enum Cmd {
        Set(String, Value),
        Run,
        Send(String, Vec<(String, Value)>),
        Advance(f64),
    }
    let mut cmds = Vec::new();
    for raw in script.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let (verb, rest) = raw.split_once(char::is_whitespace).map(|(a, b)| (a, b.trim())).unwrap_or((raw, ""));
        match verb {
            "run" => cmds.push(Cmd::Run),
            "set" => {
                let Some((k, v)) = rest.split_once('=') else {
                    eprintln!("error: `set` needs <name>=<value>");
                    return 1;
                };
                let Some(v) = parse_value(v.trim()) else {
                    eprintln!("error: '{}' is not a value", v.trim());
                    return 1;
                };
                cmds.push(Cmd::Set(k.trim().to_string(), v));
            }
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
                Some(Value::Num(q)) if q.v >= 0.0 && syscribe_model::expr::unit_scale("s").map(|x| x.1) == Some(q.dim) => cmds.push(Cmd::Advance(q.v)),
                _ => {
                    eprintln!("error: `advance` needs a non-negative duration, got '{rest}'");
                    return 1;
                }
            },
            other => {
                eprintln!("error: unknown command '{other}' (set, run, send, advance)");
                return 1;
            }
        }
    }
    let mut run = ActRun::new(&prog, ActOptions { strict, ..Default::default() });
    for c in cmds {
        match c {
            Cmd::Set(k, v) => {
                if let Err(e) = run.set_input(&k, v) {
                    eprintln!("error: {e}");
                    return 2;
                }
            }
            Cmd::Run => run.run(),
            Cmd::Send(e, p) => {
                let p: Vec<(&str, Value)> = p.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                run.send(&e, &p);
            }
            Cmd::Advance(s) => run.advance(s),
        }
    }
    let outputs: std::collections::BTreeMap<String, String> = run.outputs().iter().map(|(k, v)| (k.clone(), format_value(v))).collect();
    if json {
        let out = serde_json::json!({
            "action": prog.root, "machineHash": prog.hash, "status": run.status(), "time": run.time(),
            "outputs": outputs, "sent": run.sent(), "error": run.error(), "trace": run.trace(),
        });
        println!("{}", serde_json::to_string_pretty(&out).expect("serialises"));
    } else {
        println!("{}  (hash {})", prog.root, prog.hash);
        for s in run.trace() {
            println!("  {:>3}  t={:<6} {:<10} {:<20} {}", s.n, syscribe_model::expr::format_number(s.time), s.kind, s.node, s.note.clone().unwrap_or_default());
            for c in &s.changed {
                println!("            {} : {} → {}", c.var, c.old, c.new);
            }
        }
        println!("status: {:?}   time: {}s", run.status(), syscribe_model::expr::format_number(run.time()));
        for (k, v) in &outputs {
            println!("  {k} = {v}");
        }
        if let Some(e) = run.error() {
            println!("ERROR: {e}");
        }
    }
    if run.status() == RS::Error {
        eprintln!("error: {}", run.error().unwrap_or(""));
        return 2;
    }
    0
}

type PlantFactoryBox = Box<dyn Fn() -> Result<syscribe_model::plant::CoSim, Vec<String>>>;

/// The plants of a TestCase (REQ-TRS-FMI-001): `Ok(None)` when it declares none; otherwise a factory that
/// instantiates fresh plants for each scenario. `Err((exit code, message))` when they cannot be built; exit
/// `1` when this build has no FMI support.
fn plant_factory(elems: &[RawElement], tc: &RawElement, machine: &syscribe_model::ir::Machine, model_root: &std::path::Path) -> Result<Option<PlantFactoryBox>, (i32, String)> {
    use syscribe_model::config::{SourceLocation, ValidateConfig};
    use syscribe_model::plant::{backend, CoSim};
    use syscribe_model::plant_cfg::{declares_plants, parse_fmu, parse_wiring};
    if !declares_plants(&tc.frontmatter) {
        return Ok(None);
    }
    let Some(be) = backend() else {
        return Err((1, "this TestCase uses plants but this build has no FMI support (rebuild with `cargo build --features fmi`)".into()));
    };
    let fail = |code: &str| { let code = code.to_string(); move |es: Vec<String>| (2, es.iter().map(|e| format!("{code}: {e}")).collect::<Vec<_>>().join("; ")) };
    let wiring = parse_wiring(&tc.frontmatter).map_err(fail("E888"))?;
    let step = wiring.step.ok_or_else(|| (2, "`plants:` needs a `plantStep:`".to_string()))?;
    let resolver = syscribe_model::resolver::Resolver::new(elems);
    let cfg = ValidateConfig::with_model_root(model_root);
    struct Spec {
        name: String,
        file: std::path::PathBuf,
        params: Vec<(String, f64)>,
        sha256: Option<String>,
    }
    let mut specs: Vec<Spec> = Vec::new();
    for p in &wiring.plants {
        let pe = resolver.resolve_ref(elems, p).ok_or_else(|| (2, format!("plant '{p}' does not resolve")))?;
        let decl = pe.frontmatter.fmu.as_ref().ok_or_else(|| (2, format!("plant '{p}' has no `fmu:` declaration")))?;
        let decl = parse_fmu(decl).map_err(fail("E888"))?;
        let file = match cfg.classify_source(&decl.file) {
            SourceLocation::Local(f) => f,
            SourceLocation::Remote(u) => return Err((2, format!("the FMU '{u}' is remote; only local files are supported"))),
        };
        let info = be.inspect(&file).map_err(|e| (2, format!("{}: {e}", file.display())))?;
        if let Some(h) = &decl.sha256 {
            if !h.eq_ignore_ascii_case(&info.sha256) {
                return Err((2, format!("E966: {}: declared sha256 {h} but the file hashes to {} (the FMU changed)", file.display(), info.sha256)));
            }
        }
        let params = decl.raw_parameters(&info).map_err(fail("E888"))?;
        specs.push(Spec { name: pe.qualified_name.rsplit("::").next().unwrap_or(&pe.qualified_name).to_string(), file, params, sha256: decl.sha256.clone() });
    }
    let machine = machine.clone();
    let conns = wiring.connections.clone();
    let build = move || -> Result<CoSim, Vec<String>> {
        let be = backend().expect("registered");
        let mut plants = Vec::new();
        for s in &specs {
            // the file may have changed since it was checked: every instantiation is verified again
            if let Some(h) = &s.sha256 {
                let now = be.inspect(&s.file).map_err(|e| vec![format!("{}: {e}", s.name)])?;
                if !h.eq_ignore_ascii_case(&now.sha256) {
                    return Err(vec![format!("{}: declared sha256 {h} but the file hashes to {} (the FMU changed)", s.name, now.sha256)]);
                }
            }
            plants.push((s.name.clone(), be.instantiate(&s.file, &s.params).map_err(|e| vec![format!("{}: {e}", s.name)])?));
        }
        CoSim::new(plants, conns.clone(), step, &machine)
    };
    // build once now so wiring errors are reported before any scenario runs
    build().map_err(fail("E889"))?;
    Ok(Some(Box::new(build)))
}

/// Requirements verified by the scenario TestCases that ran: verified when every scenario of one verifying TestCase passed.
fn verified_requirements(active: &[RawElement], resolver: &syscribe_model::resolver::Resolver, ran: &[(&str, bool)]) -> Vec<(String, bool)> {
    let mut reqs: Vec<(String, bool)> = Vec::new();
    let mut tc_ids: Vec<&str> = ran.iter().map(|r| r.0).collect();
    tc_ids.sort();
    tc_ids.dedup();
    for tc_id in tc_ids {
        let all_pass = ran.iter().filter(|r| r.0 == tc_id).all(|r| r.1);
        let Some(tc) = active.iter().find(|e| e.frontmatter.id.as_deref() == Some(tc_id) || e.qualified_name == tc_id) else { continue };
        for r in tc.frontmatter.verifies.iter().flatten() {
            let id = resolver.resolve_ref(active, r).map(|e| e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())).unwrap_or_else(|| r.clone());
            match reqs.iter_mut().find(|x| x.0 == id) {
                Some(x) => x.1 |= all_pass,
                None => reqs.push((id, all_pass)),
            }
        }
    }
    reqs
}

/// `mutate` of a composite: every part's machine is mutated and judged by the composite scenarios (REQ-TRS-SYSMUT-001).
fn now_unix() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// `mutate --record`: keep the scores in `.syscribe/mutation.json` for `requirement-coverage`/`exec-coverage` (GH #333).
fn record_scores(model_root: &std::path::Path, records: Vec<syscribe_model::exec_evidence::MutationRecord>, quiet: bool) -> Result<(), i32> {
    let n = records.len();
    match syscribe_model::exec_evidence::record_mutation(model_root, records) {
        Ok(path) => {
            if !quiet {
                println!("recorded {n} mutation score(s) in {}", path.display());
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("error: could not record the mutation scores: {e}");
            Err(2)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn mutate_composite(elems: &[RawElement], composite: &str, tcs: &[&RawElement], min: Option<f64>, only_part: Option<&str>, json: bool, record: bool, model_root: &std::path::Path) -> i32 {
    use syscribe_model::sm_scen::extract_scenarios;
    let scenarios: Vec<_> = tcs.iter().flat_map(|tc| extract_scenarios(&tc.doc)).collect();
    if scenarios.is_empty() {
        eprintln!("error: no scenarios to run against {composite}");
        return 1;
    }
    let model = match syscribe_model::system_run::SystemModel::build_in(elems, composite, model_root) {
        Ok(m) => m,
        Err(errs) => {
            for e in &errs {
                eprintln!("error: {e}");
            }
            return if errs.iter().any(|e| e.contains("no FMI support") || e.contains("is not a composite with an executable")) { 1 } else { 2 };
        }
    };
    if let Some(p) = only_part {
        if !model.spec.instances.iter().enumerate().any(|(i, inst)| inst.name == p && model.machines[i].is_some()) {
            let names: Vec<&str> = model.spec.instances.iter().enumerate().filter(|(i, _)| model.machines[*i].is_some()).map(|(_, i)| i.name.as_str()).collect();
            eprintln!("error: '{p}' is not a part with a state machine of {composite} (parts: {})", names.join(", "));
            return 1;
        }
    }
    let report = match syscribe_model::system_mut::analyse(&model, &scenarios, &syscribe_model::sm_run::Options::default(), only_part) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&report).expect("serialises"));
    } else {
        println!(
            "mutation score {:.1}%  ({} killed, {} survived, {} equivalent-suspect) over {} scenario(s) on {} (hash {})",
            report.score, report.killed, report.survived, report.equivalent_suspect, report.scenarios, report.composite, &report.hash[..12]
        );
        for p in &report.parts {
            println!("{}: {:.1}%  ({} killed, {} survived, {} equivalent-suspect)", p.machine, p.score, p.killed, p.survived, p.equivalent_suspect);
            for m in p.mutants.iter().filter(|m| m.fate == "survived") {
                println!("  SURVIVED  {}  [{} at {}]{}", m.description, m.operator, m.origin.path, m.closest.as_ref().map(|c| format!("  (closest: {c})")).unwrap_or_default());
            }
            for m in p.mutants.iter().filter(|m| m.fate == "equivalentSuspect") {
                println!("  uncovered {}  [{}]", m.description, m.operator);
            }
            if !p.uncovered.states.is_empty() {
                println!("  states never entered: {}", p.uncovered.states.join(", "));
            }
            if p.killed + p.survived == 0 && p.equivalent_suspect > 0 {
                println!("  note: no mutant of this part was judged; the scenarios never exercise it");
            }
        }
    }
    if record {
        // one score per machine, summed over the product parts that use it (environment parts are not the product);
        // a machine none of whose mutants was judged has no score worth keeping
        let parts = report.parts.iter().filter_map(|p| {
            let i = model.spec.instance(&p.part)?;
            let m = model.machines[i].as_ref().filter(|_| !model.spec.instances[i].environment)?;
            Some((m.machine.clone(), p.machine_hash.clone(), p.killed, p.survived))
        });
        let recs = syscribe_model::exec_evidence::aggregate_parts(parts)
            .into_iter()
            .map(|(machine, machine_hash, killed, survived)| syscribe_model::exec_evidence::MutationRecord { subject: composite.to_string(), machine, machine_hash, score: syscribe_model::exec_evidence::score_of(killed, survived), killed, survived, recorded_at_unix: now_unix() })
            .collect();
        if let Err(code) = record_scores(model_root, recs, json) {
            return code;
        }
    }
    let mut exit = 0;
    if let Some(min) = min {
        for p in &report.parts {
            if p.killed + p.survived == 0 && p.equivalent_suspect > 0 {
                // nothing was judged: the scenarios never exercised the part, which a score of 100% would hide
                eprintln!("mutation gate: {} had no mutant judged ({} equivalent-suspect: the scenarios never exercise this part)", p.machine, p.equivalent_suspect);
                exit = 2;
            } else if p.score + 1e-9 < min {
                eprintln!("mutation gate: {} score {:.1}% is below the required {min}%", p.machine, p.score);
                exit = 2;
            }
        }
    }
    exit
}

/// `inject <composite|TestCase> [--ft <FT>] [--max-order N] [--fmea <sheet>] [--report-only] [--json]` (REQ-TRS-FAULT-002):
/// run the `@invariant` scenarios of a composite under every single fault and every minimal cut set. Exit 0 every set
/// survived · 1 usage · 2 a set defeated an invariant or could not be run (`--report-only` makes defeats exit 0).
pub fn cmd_inject(elems: &[RawElement], args: &[String], model_root: &std::path::Path) -> i32 {
    use syscribe_model::element::ElementType;
    use syscribe_model::fault_campaign::{self, FaultSet, NotInjectable};
    use syscribe_model::sm_scen::{extract_scenarios, subject_of};
    let flag = |f: &str| args.iter().any(|a| a == f);
    let value = |f: &str| args.windows(2).find(|w| w[0] == f).map(|w| w[1].clone());
    let skip: Vec<usize> = args.iter().enumerate().filter(|(_, a)| matches!(a.as_str(), "--ft" | "--max-order" | "--fmea")).map(|(i, _)| i + 1).collect();
    let Some(name) = args.iter().enumerate().find(|(i, a)| !a.starts_with("--") && !skip.contains(i)).map(|(_, a)| a.clone()) else {
        eprintln!("Usage: syscribe --model <root> inject <composite|TestCase> [--ft <FaultTree>] [--max-order <N>] [--fmea <FMEASheet>] [--report-only] [--json]");
        return 1;
    };
    let max_order = match value("--max-order").map(|v| v.parse::<usize>()) {
        None => 2,
        Some(Ok(n)) if n >= 1 => n,
        _ => {
            eprintln!("error: --max-order needs a positive integer");
            return 1;
        }
    };
    let resolver = syscribe_model::resolver::Resolver::new(elems);
    let Some(target) = resolver.resolve_ref(elems, &name) else {
        eprintln!("error: no element '{name}'");
        return 1;
    };
    let is_composite = |e: &RawElement| matches!(e.frontmatter.element_type, Some(ElementType::Part) | Some(ElementType::PartDef));
    let (composite, tcs): (String, Vec<&RawElement>) = if matches!(target.frontmatter.element_type, Some(ElementType::TestCase)) {
        match subject_of(elems, target) {
            Ok(s) if is_composite(s) => (s.qualified_name.clone(), vec![target]),
            Ok(s) => {
                eprintln!("error: the subject '{}' of {name} is not a composite part", s.qualified_name);
                return 1;
            }
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        }
    } else if is_composite(target) {
        let tcs = elems.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::TestCase))).filter(|e| subject_of(elems, e).map(|s| s.qualified_name == target.qualified_name).unwrap_or(false)).collect();
        (target.qualified_name.clone(), tcs)
    } else {
        eprintln!("error: '{name}' is neither a TestCase nor a composite part");
        return 1;
    };
    let invariants: Vec<_> = tcs.iter().flat_map(|tc| extract_scenarios(&tc.doc)).filter(|s| s.has_tag("invariant")).collect();
    if invariants.is_empty() {
        eprintln!("error: no scenario of {composite} is tagged @invariant; tag the scenarios that state a safety property that must hold under faults (a Gherkin tag on the line before `Scenario:`)");
        return 1;
    }
    // the campaign injects the faults itself: an invariant must not inject or clear any
    for sc in &invariants {
        if sc.steps.iter().any(|st| matches!(syscribe_model::system_scen::parse_step(st.kw, &st.text), Ok(syscribe_model::system_scen::SCmd::Fault { .. }))) {
            eprintln!("error: the invariant '{}' has `fault … is injected/cleared` steps; the campaign injects the faults itself, so an invariant states only the property", sc.title);
            return 1;
        }
    }
    let model = match syscribe_model::system_run::SystemModel::build_in(elems, &composite, model_root) {
        Ok(m) => m,
        Err(errs) => {
            for e in &errs {
                eprintln!("error: {e}");
            }
            return if errs.iter().any(|e| e.contains("no FMI support") || e.contains("is not a composite with an executable")) { 1 } else { 2 };
        }
    };
    // single faults: every fault that applies to the composite, optionally only the rows of one sheet
    let sheet_rows: Option<Vec<String>> = match value("--fmea") {
        None => None,
        Some(id) => match resolver.resolve_ref(elems, &id).filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FMEASheet))) {
            Some(sheet) => {
                let prefix = format!("{}::", sheet.qualified_name);
                Some(elems.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FMEAEntry)) && e.qualified_name.starts_with(&prefix)).filter_map(|e| e.frontmatter.id.clone()).collect())
            }
            None => {
                eprintln!("error: '{id}' is not an FMEASheet");
                return 1;
            }
        },
    };
    let mut sets: Vec<FaultSet> = Vec::new();
    let push = |sets: &mut Vec<FaultSet>, mut ids: Vec<String>, origin: String| {
        ids.sort();
        ids.dedup();
        if !sets.iter().any(|s| s.ids == ids) {
            sets.push(FaultSet { ids, origin });
        }
    };
    for f in &model.faults {
        if sheet_rows.as_ref().is_none_or(|rows| rows.contains(&f.id)) {
            push(&mut sets, vec![f.id.clone()], format!("single fault {}", f.id));
        }
    }
    let mut not_injectable: Vec<NotInjectable> = Vec::new();
    let mut cut_set_order: Option<usize> = None;
    let mut cut_sets_truncated = false;
    if let Some(ft_id) = value("--ft") {
        let Some(tree) = syscribe_model::fta::find_fault_tree(elems, &ft_id) else {
            eprintln!("error: no fault tree '{ft_id}'");
            return 1;
        };
        let opts = syscribe_model::fta::AnalysisOptions { max_order: Some(max_order), ..Default::default() };
        let analysis = match syscribe_model::fta::analyze_fault_tree(elems, &resolver, tree, &opts) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("error: fault tree {ft_id}: {e:?}");
                return 2;
            }
        };
        let tree_label = tree.frontmatter.id.clone().unwrap_or_else(|| tree.qualified_name.clone());
        cut_set_order = Some(max_order);
        cut_sets_truncated = analysis.truncated;
        for cs in analysis.cut_sets.iter().filter(|c| c.order <= max_order) {
            // a common-cause pseudo-event `CCF:<group>` stands for every member of the group failing together
            let mut ids: Vec<String> = Vec::new();
            let mut missing: Vec<String> = Vec::new();
            for e in &cs.events {
                let members: Vec<String> = match e.strip_prefix("CCF:") {
                    Some(group) => elems.iter().filter(|x| matches!(x.frontmatter.element_type, Some(ElementType::FaultTreeEvent)) && x.frontmatter.ccf_group.as_deref() == Some(group)).filter_map(|x| x.frontmatter.id.clone()).collect(),
                    None => vec![e.clone()],
                };
                for m in members {
                    if model.faults.iter().any(|f| f.id == m) {
                        ids.push(m);
                    } else {
                        missing.push(m);
                    }
                }
            }
            let origin = format!("cut set of {tree_label}, order {}", cs.order);
            if missing.is_empty() && !ids.is_empty() {
                push(&mut sets, ids, origin);
            } else {
                not_injectable.push(NotInjectable { origin, events: missing, reason: "no fault model (`injection:`) of these events applies to the composite".into() });
            }
        }
    }
    if sets.is_empty() {
        eprintln!("error: no fault set to run: no fault of {composite} has an `injection:` that applies to it{}", if value("--ft").is_some() { " (and no cut set of the fault tree has events that all have one)" } else { "" });
        return 1;
    }
    let opts = syscribe_model::sm_run::Options::default();
    let mut campaign = match fault_campaign::run(&model, &invariants, &opts, &sets) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    campaign.not_injectable = not_injectable;
    campaign.cut_set_order = cut_set_order;
    campaign.cut_sets_truncated = cut_sets_truncated;
    let rows = fault_campaign::analysis_rows(elems, &resolver, &model);
    // exercised: in a set that actually ran (not one that ended in an error)
    let ran: std::collections::HashSet<&str> = campaign.sets.iter().filter(|s| s.status != "error").flat_map(|s| s.faults.iter().map(|x| x.as_str())).collect();
    for (id, has_model) in &rows {
        if !has_model {
            campaign.fmea.without_model.push(id.clone());
        } else if ran.contains(id.as_str()) {
            campaign.fmea.exercised.push(id.clone());
        } else {
            campaign.fmea.not_exercised.push(id.clone());
        }
    }
    let (survived, defeated, errored) = (
        campaign.sets.iter().filter(|s| s.status == "survived").count(),
        campaign.sets.iter().filter(|s| s.status == "defeated").count(),
        campaign.sets.iter().filter(|s| s.status == "error").count(),
    );
    if flag("--json") {
        println!("{}", serde_json::to_string_pretty(&campaign).expect("serialises"));
    } else {
        println!("fault campaign on {} (hash {}): {} invariant scenario(s), {} fault set(s)", campaign.composite, &campaign.hash[..12], campaign.invariants, campaign.sets.len());
        if !campaign.environment.is_empty() {
            println!("  parts that are not the product (environment, external): {}; a fault set marked with their role fails only the set-up around the product", campaign.environment.join(", "));
        }
        for s in &campaign.sets {
            println!("  {:<9} {}{}   ({})", s.status.to_uppercase(), s.label, if s.environment { format!(" [{}]", s.role.unwrap_or("not the product")) } else { String::new() }, s.origin);
            for d in &s.defeats {
                println!("      defeats \"{}\": {}", d.scenario, d.message);
            }
            for d in &s.errors {
                println!("      error in \"{}\": {}", d.scenario, d.message);
            }
        }
        if let Some(n) = campaign.cut_set_order {
            println!("  cut sets of the fault tree up to order {n} were run; larger orders are not{}", if campaign.cut_sets_truncated { " (the tree has cut sets of a higher order, which were left out)" } else { "" });
        }
        for n in &campaign.not_injectable {
            println!("  not run   {} ({}): {}", n.events.join(", "), n.origin, n.reason);
        }
        println!("FMEA/FTA rows of the composite's parts: {} exercised, {} with a fault model but not exercised, {} with no usable fault model (prose only or malformed)", campaign.fmea.exercised.len(), campaign.fmea.not_exercised.len(), campaign.fmea.without_model.len());
        if !campaign.fmea.not_exercised.is_empty() {
            println!("  not exercised: {}", campaign.fmea.not_exercised.join(", "));
        }
        if !campaign.fmea.without_model.is_empty() {
            println!("  no usable fault model: {}", campaign.fmea.without_model.join(", "));
        }
        println!("summary: {survived} survived, {defeated} defeated, {errored} could not be run");
    }
    if (defeated > 0 || errored > 0) && !flag("--report-only") {
        return 2;
    }
    0
}

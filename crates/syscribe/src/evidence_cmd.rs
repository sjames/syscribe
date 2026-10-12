//! `requirement-coverage` and `exec-coverage` (GH #344, #333): the requirement view and the architecture-element view
//! of the executable evidence `syscribe_model::exec_evidence` computes. Read-only. Exit 0 · 1 usage · 2 a gate missed.

use syscribe_model::element::RawElement;
use syscribe_model::exec_evidence::{self, Evidence, ReqVerdict, Request, Thresholds};

const REQ_USAGE: &str = "Usage: syscribe -m <root> requirement-coverage [<REQ>...] [--config <C>] [--min-executable <percent>] [--require <REQ>]... [--min-states <percent>] [--min-transitions <percent>] [--min-guards <percent>] [--min-score <percent>] [--mutate] [--json]";
const EXEC_USAGE: &str = "Usage: syscribe -m <root> exec-coverage [<element>...] [--config <C>] [--min-modelled <percent>] [--require <element>]... [--min-states <percent>] [--min-transitions <percent>] [--min-guards <percent>] [--min-score <percent>] [--mutate] [--json]";

const VALUED: &[&str] = &["--config", "--min-executable", "--min-modelled", "--require", "--min-states", "--min-transitions", "--min-guards", "--min-score"];

struct Args {
    positional: Vec<String>,
    config: Option<String>,
    require: Vec<String>,
    min_gate: Option<f64>,
    thresholds: Thresholds,
    mutate: bool,
    json: bool,
}

fn percent(flag: &str, v: Option<&String>) -> Result<Option<f64>, String> {
    match v {
        None => Ok(None),
        Some(v) => v.parse::<f64>().ok().filter(|p| (0.0..=100.0).contains(p)).map(Some).ok_or_else(|| format!("{flag} needs a percentage between 0 and 100 (got '{v}')")),
    }
}

fn parse(args: &[String], gate: &str) -> Result<Args, String> {
    let mut out = Args { positional: Vec::new(), config: None, require: Vec::new(), min_gate: None, thresholds: Thresholds::default(), mutate: false, json: false };
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if VALUED.contains(&a) {
            let v = args.get(i + 1).filter(|v| !v.starts_with("--")).ok_or_else(|| format!("{a} expects a value"))?;
            match a {
                "--config" => out.config = Some(v.clone()),
                "--require" => out.require.push(v.clone()),
                "--min-states" => out.thresholds.min_states = percent(a, Some(v))?,
                "--min-transitions" => out.thresholds.min_transitions = percent(a, Some(v))?,
                "--min-guards" => out.thresholds.min_guards = percent(a, Some(v))?,
                "--min-score" => out.thresholds.min_score = percent(a, Some(v))?,
                f if f == gate => out.min_gate = percent(a, Some(v))?,
                f => return Err(format!("{f} does not apply to this command")),
            }
            i += 2;
            continue;
        }
        match a {
            "--mutate" => out.mutate = true,
            "--json" => out.json = true,
            f if f.starts_with("--") => return Err(format!("unknown option {f}")),
            _ => out.positional.push(a.to_string()),
        }
        i += 1;
    }
    Ok(out)
}

fn evidence(elems: &[RawElement], a: &Args, lens: &dyn Fn(Option<&str>) -> Vec<RawElement>, model_root: &std::path::Path) -> Result<Evidence, String> {
    if let Some(c) = &a.config {
        if !elems.iter().any(|e| matches!(e.frontmatter.element_type, Some(syscribe_model::element::ElementType::Configuration)) && e.frontmatter.id.as_deref() == Some(c.as_str())) {
            return Err(format!("--config '{c}' is not a Configuration of this model"));
        }
    }
    let view = lens(a.config.as_deref());
    let req = Request { model_root, configuration: a.config.clone(), mutate: a.mutate, thresholds: a.thresholds.clone() };
    Ok(exec_evidence::compute(&view, &req))
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        100.0
    } else {
        100.0 * n as f64 / d as f64
    }
}

fn find_req<'a>(ev: &'a Evidence, r: &str) -> Option<&'a exec_evidence::RequirementEvidence> {
    ev.requirements.iter().find(|x| x.id == r || x.qualified_name == r)
}

/// `requirement-coverage` (GH #344).
pub fn cmd_requirement_coverage(elems: &[RawElement], args: &[String], lens: &dyn Fn(Option<&str>) -> Vec<RawElement>, model_root: &std::path::Path) -> i32 {
    let a = match parse(args, "--min-executable") {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}\n{REQ_USAGE}");
            return 1;
        }
    };
    let ev = match evidence(elems, &a, lens, model_root) {
        Ok(ev) => ev,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };
    for r in a.positional.iter().chain(&a.require) {
        if find_req(&ev, r).is_none() {
            eprintln!("error: '{r}' is not a Requirement of this model");
            return 1;
        }
    }
    let shown: Vec<&exec_evidence::RequirementEvidence> = if a.positional.is_empty() { ev.requirements.iter().collect() } else { a.positional.iter().filter_map(|r| find_req(&ev, r)).collect() };
    let count = |v: ReqVerdict| shown.iter().filter(|r| r.verdict == v).count();
    let applicable = shown.len() - count(ReqVerdict::NotApplicable);
    let strong = count(ReqVerdict::Executable);
    let share = pct(strong, applicable);

    if a.json {
        let out = serde_json::json!({
            "configuration": ev.configuration,
            "summary": {
                "requirements": shown.len(),
                "executable": strong,
                "executableWeak": count(ReqVerdict::ExecutableWeak),
                "descriptiveOnly": count(ReqVerdict::DescriptiveOnly),
                "unverified": count(ReqVerdict::Unverified),
                "notApplicable": count(ReqVerdict::NotApplicable),
                "executablePercent": share,
            },
            "requirements": shown,
            "scenarios": ev.scenarios,
            "errors": ev.errors,
        });
        println!("{}", serde_json::to_string_pretty(&out).expect("serialises"));
    } else {
        if let Some(c) = &ev.configuration {
            println!("Configuration {c}");
        }
        let detail = !a.positional.is_empty();
        for r in &shown {
            println!("{}  {}  {}", r.id, r.verdict.label(), r.name);
            if !detail && r.verdict == ReqVerdict::Executable {
                continue;
            }
            for why in &r.reasons {
                println!("    {why}");
            }
            if detail {
                for s in &r.scenarios {
                    println!("    scenario  {} / {}  [{}]", s.test_case, s.scenario, s.status);
                }
                for c in &r.coverage {
                    let who = match &c.part {
                        Some(p) => format!("{}.{p} ({})", c.subject, c.machine),
                        None => c.machine.clone(),
                    };
                    println!(
                        "    coverage  {who}: states {}/{}, transitions {}/{}, guard conditions {}/{} over {} scenario(s)",
                        c.states.covered, c.states.total, c.transitions.covered, c.transitions.total, c.guards.covered, c.guards.total, c.scenarios
                    );
                }
                for m in &r.mutation {
                    println!("    mutation  {}: {:.1}% ({} killed, {} survived, {}){}", m.machine, m.score, m.killed, m.survived, m.source, if m.stale { " STALE: the machine changed since it was recorded" } else { "" });
                }
                if !r.satisfiers.is_empty() {
                    println!("    satisfied by {}", r.satisfiers.join(", "));
                }
            }
            for s in &r.claimed_not_exercised {
                println!("    claimed but not exercised: {} / {} (its subject exercises nothing that satisfies {})", s.test_case, s.scenario, r.id);
            }
            if r.verdict == ReqVerdict::DescriptiveOnly || detail {
                for d in &r.descriptive {
                    println!("    descriptive  {}{}", d.test_case, d.level.as_ref().map(|l| format!(" ({l})")).unwrap_or_default());
                }
            }
        }
        println!(
            "\n{} requirement(s): {} executable, {} executable-weak, {} descriptive-only, {} unverified, {} not applicable. Executable: {share:.1}% of {applicable} applicable.",
            shown.len(),
            strong,
            count(ReqVerdict::ExecutableWeak),
            count(ReqVerdict::DescriptiveOnly),
            count(ReqVerdict::Unverified),
            count(ReqVerdict::NotApplicable)
        );
        for e in &ev.errors {
            println!("could not run: {e}");
        }
    }
    let mut exit = 0;
    if let Some(min) = a.min_gate {
        if share + 1e-9 < min {
            eprintln!("requirement-coverage gate: {share:.1}% of the applicable requirements have executable evidence, below the required {min}%");
            exit = 2;
        }
    }
    for r in &a.require {
        let e = find_req(&ev, r).expect("checked above");
        if e.verdict != ReqVerdict::Executable {
            eprintln!("requirement-coverage gate: {} is {}, not executable", e.id, e.verdict.label());
            exit = 2;
        }
    }
    exit
}

/// `exec-coverage` (GH #333).
pub fn cmd_exec_coverage(elems: &[RawElement], args: &[String], lens: &dyn Fn(Option<&str>) -> Vec<RawElement>, model_root: &std::path::Path) -> i32 {
    let a = match parse(args, "--min-modelled") {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}\n{EXEC_USAGE}");
            return 1;
        }
    };
    let ev = match evidence(elems, &a, lens, model_root) {
        Ok(ev) => ev,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };
    let resolver = syscribe_model::resolver::Resolver::new(elems);
    let find = |n: &str| {
        let q = resolver.resolve_ref(elems, n).map(|e| e.qualified_name.clone()).unwrap_or_else(|| n.to_string());
        ev.elements.iter().find(|e| e.qualified_name == q)
    };
    for n in a.positional.iter().chain(&a.require) {
        if find(n).is_none() {
            eprintln!("error: '{n}' is not a software or hardware part of this model");
            return 1;
        }
    }
    let shown: Vec<&exec_evidence::ElementEvidence> = if a.positional.is_empty() { ev.elements.iter().collect() } else { a.positional.iter().filter_map(|n| find(n)).collect() };
    let modelled = shown.iter().filter(|e| e.modelled()).count();
    let share = pct(modelled, shown.len());
    if a.json {
        let out = serde_json::json!({
            "configuration": ev.configuration,
            "summary": { "elements": shown.len(), "modelled": modelled, "modelledPercent": share },
            "elements": shown,
            "errors": ev.errors,
        });
        println!("{}", serde_json::to_string_pretty(&out).expect("serialises"));
    } else {
        if let Some(c) = &ev.configuration {
            println!("Configuration {c}");
        }
        for e in &shown {
            let how = if e.executable_itself {
                "executable".to_string()
            } else if !e.modelled_by.is_empty() {
                format!("modelled by {}", e.modelled_by.join(", "))
            } else {
                "not modelled".to_string()
            };
            let r = &e.requirements;
            println!(
                "{} [{}]  {how}; {} scenario(s){}; requirements: {} executable{}, {} descriptive-only, {} unverified",
                e.qualified_name,
                e.domain,
                e.scenarios,
                if e.passing < e.scenarios { format!(" ({} passing)", e.passing) } else { String::new() },
                r.executable.len(),
                if r.executable_elsewhere.is_empty() { String::new() } else { format!(" (+{} through another element)", r.executable_elsewhere.len()) },
                r.descriptive_only.len(),
                r.unverified.len()
            );
            for m in &e.mutation {
                println!("    mutation  {}: {:.1}% ({}){}", m.machine, m.score, m.source, if m.stale { " STALE" } else { "" });
            }
            for c in &e.correspondence {
                println!("    {c}");
            }
            if !a.positional.is_empty() {
                for (what, l) in [("executable", &r.executable), ("executable through another element", &r.executable_elsewhere), ("descriptive-only", &r.descriptive_only), ("unverified", &r.unverified), ("not applicable", &r.not_applicable)] {
                    if !l.is_empty() {
                        println!("    {what}: {}", l.join(", "));
                    }
                }
            }
        }
        println!("\n{modelled} of {} software/hardware element(s) modelled ({share:.1}%).", shown.len());
        for e in &ev.errors {
            println!("could not run: {e}");
        }
    }
    let mut exit = 0;
    if let Some(min) = a.min_gate {
        if share + 1e-9 < min {
            eprintln!("exec-coverage gate: {share:.1}% of the software/hardware elements are modelled, below the required {min}%");
            exit = 2;
        }
    }
    for n in &a.require {
        let e = find(n).expect("checked above");
        if !e.modelled() || e.passing == 0 {
            eprintln!("exec-coverage gate: {} is {}", e.qualified_name, if e.modelled() { "modelled but no passing executable scenario exercises it" } else { "not modelled" });
            exit = 2;
        }
    }
    exit
}

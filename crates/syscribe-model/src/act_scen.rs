//! Executable Gherkin scenarios for action flows (REQ-TRS-ACTEXEC-001, GH #276): the closed step vocabulary
//! for an `ActionDef` subject and running a scenario to a verdict. Extraction and outlines are shared with
//! the state-machine scenarios (`sm_scen`).

use crate::act_ir::ActionProgram;
use crate::act_run::{ActOptions, ActRun, Status as RunStatus};
use crate::expr::{self, Value};
use crate::sm_run::{Coverage, Step as TraceStep};
use crate::sm_scen::{extract_scenarios, find_unquoted, split_top, Kw, Scenario, Status, Verdict};

#[derive(Debug, Clone, PartialEq)]
pub enum ACmd {
    Set(String, String),
    Run,
    Inject(String, Vec<(String, String)>),
    Elapse(String),
    Expect(String),
    Ran(String, bool),
    /// `completed`, `waiting`, `terminated` or `deadlocked`.
    Is(&'static str),
}

/// Parse one step against the action vocabulary; `Err` is why it is outside it (`E886`).
pub fn parse_action_step(kw: Kw, text: &str) -> Result<ACmd, String> {
    let t = text.trim();
    let lower = t.to_ascii_lowercase();
    if kw == Kw::When && matches!(lower.as_str(), "the action runs" | "the action starts" | "the action is run" | "the action is started") {
        return Ok(ACmd::Run);
    }
    for suffix in [" elapses", " elapse"] {
        if kw == Kw::When && lower.ends_with(suffix) {
            let d = t[..t.len() - suffix.len()].trim();
            return match expr::parse(d) {
                Ok(_) => Ok(ACmd::Elapse(d.to_string())),
                Err(e) => Err(format!("`{d}` is not a duration: {e}")),
            };
        }
    }
    if let Some(i) = find_unquoted(&lower, " is injected") {
        if kw != Kw::When {
            return Err("`is injected` is a When step".into());
        }
        let ev = t[..i].trim();
        if ev.is_empty() || ev.contains(char::is_whitespace) {
            return Err(format!("'{ev}' is not an event name"));
        }
        let rest = t[i + " is injected".len()..].trim();
        let mut fields = Vec::new();
        if !rest.is_empty() {
            let Some(list) = rest.strip_prefix("with ").or_else(|| rest.strip_prefix("With ")) else {
                return Err(format!("expected `with <field> = <expression>` after `is injected`, got `{rest}`"));
            };
            for f in split_top(list) {
                let Some((k, v)) = f.split_once('=') else { return Err(format!("payload `{f}` is not <field> = <expression>")) };
                if let Err(e) = expr::parse(v.trim()) {
                    return Err(format!("payload value `{}`: {e}", v.trim()));
                }
                fields.push((k.trim().to_string(), v.trim().to_string()));
            }
        }
        return Ok(ACmd::Inject(ev.to_string(), fields));
    }
    if kw == Kw::Then {
        for (phrase, what) in [
            ("the action completed", "completed"),
            ("the action is completed", "completed"),
            ("the action is waiting", "waiting"),
            ("the action waits", "waiting"),
            ("the action terminated", "terminated"),
            ("the action is terminated", "terminated"),
            ("the action is deadlocked", "deadlocked"),
            ("the action deadlocked", "deadlocked"),
        ] {
            if lower == phrase {
                return Ok(ACmd::Is(what));
            }
        }
        if let Some(n) = lower.strip_suffix(" did not run").or_else(|| lower.strip_suffix(" did not execute")) {
            let n = t[..n.len()].trim();
            if !n.is_empty() && !n.contains(char::is_whitespace) && !matches!(n.to_ascii_lowercase().as_str(), "not" | "and" | "or") {
                return Ok(ACmd::Ran(n.to_string(), false));
            }
        }
        if let Some(n) = lower.strip_suffix(" ran").or_else(|| lower.strip_suffix(" was run")) {
            let n = t[..n.len()].trim();
            if !n.is_empty() && !n.contains(char::is_whitespace) && !matches!(n.to_ascii_lowercase().as_str(), "not" | "and" | "or") {
                return Ok(ACmd::Ran(n.to_string(), true));
            }
        }
        return match expr::parse(t) {
            Ok(_) => Ok(ACmd::Expect(t.to_string())),
            Err(e) => Err(format!("`{t}` is not an expression: {e}")),
        };
    }
    if let Some(i) = find_unquoted(&lower, " becomes ") {
        if kw != Kw::When {
            return Err("`becomes` is a When step".into());
        }
        let (var, e) = (t[..i].trim(), t[i + " becomes ".len()..].trim());
        return match expr::parse(e) {
            Ok(_) if !var.is_empty() && !var.contains(char::is_whitespace) => Ok(ACmd::Set(var.to_string(), e.to_string())),
            Ok(_) => Err(format!("'{var}' is not a name")),
            Err(x) => Err(format!("`{e}`: {x}")),
        };
    }
    if kw == Kw::Given {
        if let Some(i) = find_unquoted(&lower, " is ") {
            let (var, e) = (t[..i].trim(), t[i + 4..].trim());
            if !var.is_empty() && !var.contains(char::is_whitespace) {
                return match expr::parse(e) {
                    Ok(_) => Ok(ACmd::Set(var.to_string(), e.to_string())),
                    Err(x) => Err(format!("`{e}`: {x}")),
                };
            }
        }
        return Err("a Given step is `<parameter or variable> is <expression>`".into());
    }
    Err("a When step is `the action runs`, `<event> is injected [with …]`, `<name> becomes <expression>` or `<duration> elapse[s]`".into())
}

/// `E886` findings of a `TestCase` body for an action subject.
pub fn vocabulary_findings(doc: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for sc in extract_scenarios(doc) {
        if sc.steps.is_empty() {
            let why = if sc.title.ends_with("[no examples]") { "the Scenario Outline has no Examples rows" } else { "the scenario has no steps" };
            out.push((0, format!("scenario '{}': {why}", sc.title)));
        }
        for st in &sc.steps {
            if !seen.insert((st.line, st.text.clone())) {
                continue;
            }
            if let Err(e) = parse_action_step(st.kw, &st.text) {
                out.push((st.line, format!("scenario '{}': step `{}` is outside the executable action vocabulary: {e}", sc.title, st.text)));
            }
        }
    }
    out
}

fn to_trace(r: &ActRun) -> Vec<TraceStep> {
    r.trace()
        .iter()
        .map(|s| TraceStep {
            n: s.n,
            time: s.time,
            cause: format!("{} {}", s.kind, s.node),
            transition: None,
            taken: vec![],
            from: Some(s.node.clone()),
            to: None,
            guard: None,
            exited: vec![],
            entered: vec![],
            actions: s.sent.iter().map(|e| format!("send {e}")).collect(),
            changed: s.changed.clone(),
            note: s.note.clone(),
        })
        .collect()
}

/// Run one scenario on a fresh run of `prog`. The verdict is shared with the state-machine scenarios so the
/// CLI, evidence recording and gates work unchanged.
pub fn run_action_scenario(prog: &ActionProgram, sc: &Scenario, opts: &ActOptions) -> Verdict {
    let mut run = ActRun::new(prog, opts.clone());
    let steps: Vec<String> = sc.steps.iter().map(|s| format!("{} {}", match s.kw { Kw::Given => "Given", Kw::When => "When", Kw::Then => "Then" }, s.text)).collect();
    let done = |run: &ActRun, status: Status, message: Option<String>| Verdict {
        scenario: sc.title.clone(),
        outline: sc.outline.clone(),
        status,
        message,
        steps: steps.clone(),
        trace: to_trace(run),
        coverage: Coverage::default(),
        act_coverage: run.coverage(),
        plants: vec![],
        messages: vec![],
    };
    if sc.steps.is_empty() {
        let why = if sc.title.ends_with("[no examples]") { "the Scenario Outline has no Examples rows" } else { "the scenario has no steps" };
        return done(&run, Status::Error, Some(why.to_string()));
    }
    let names = prog.stmt_names();
    let mut cov = crate::act_run::ActCoverage::default();
    let finish = |run: &ActRun, status, msg, cov: &mut crate::act_run::ActCoverage| {
        cov.merge(&run.coverage());
        let mut v = done(run, status, msg);
        v.act_coverage = cov.clone();
        v
    };
    for (i, st) in sc.steps.iter().enumerate() {
        let label = format!("step {} (`{}`)", i + 1, steps[i]);
        let cmd = match parse_action_step(st.kw, &st.text) {
            Ok(c) => c,
            Err(e) => return finish(&run, Status::Error, Some(format!("{label}: {e}")), &mut cov),
        };
        let applied: Result<(), String> = match &cmd {
            ACmd::Set(name, e) => run.eval_str(e).and_then(|v| run.set_input(name, v)),
            ACmd::Run => {
                run.run();
                Ok(())
            }
            ACmd::Inject(ev, fields) => fields
                .iter()
                .map(|(k, v)| run.eval_str(v).map(|x| (k.clone(), x)))
                .collect::<Result<Vec<_>, _>>()
                .map(|p| {
                    let p: Vec<(&str, Value)> = p.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                    run.send(ev, &p);
                }),
            ACmd::Elapse(d) => match run.eval_str(d) {
                Ok(Value::Num(q)) if expr::unit_scale("s").map(|x| x.1) == Some(q.dim) && q.v >= 0.0 => {
                    run.advance(q.v);
                    Ok(())
                }
                Ok(v) => Err(format!("`{d}` is a {}, not a non-negative duration", v.kind())),
                Err(e) => Err(e),
            },
            ACmd::Ran(n, expect) => {
                if !names.contains(n) {
                    Err(format!("the action has no node or statement named '{n}'"))
                } else if run.ran(n) == *expect {
                    Ok(())
                } else {
                    return finish(&run, Status::Fail, Some(format!("{label}: expected '{n}' {}, it {}", if *expect { "to have run" } else { "not to have run" }, if *expect { "did not" } else { "did" })), &mut cov);
                }
            }
            ACmd::Is(what) => {
                let ok = matches!((what, run.status()), (&"completed", RunStatus::Completed) | (&"waiting", RunStatus::Waiting) | (&"terminated", RunStatus::Terminated) | (&"deadlocked", RunStatus::Deadlocked));
                if ok {
                    Ok(())
                } else {
                    return finish(&run, Status::Fail, Some(format!("{label}: expected the action to be {what}, its status is {:?}", run.status())), &mut cov);
                }
            }
            ACmd::Expect(e) => match run.eval_str(e) {
                Ok(Value::Bool(true)) => Ok(()),
                Ok(Value::Bool(false)) => {
                    let outs: Vec<String> = run.outputs().iter().map(|(k, v)| format!("{k} = {}", expr::format_value(v))).collect();
                    return finish(&run, Status::Fail, Some(format!("{label}: `{e}` is false (status {:?}; outputs: {})", run.status(), outs.join(", "))), &mut cov);
                }
                Ok(v) => Err(format!("`{e}` is a {}, not a boolean", v.kind())),
                Err(x) => Err(x),
            },
        };
        if let Err(e) = applied {
            return finish(&run, Status::Error, Some(format!("{label}: {e}")), &mut cov);
        }
        if let Some(e) = run.error() {
            let e = e.to_string();
            return finish(&run, Status::Error, Some(format!("{label}: run error: {e}")), &mut cov);
        }
    }
    finish(&run, Status::Pass, None, &mut cov)
}

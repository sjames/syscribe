//! Executable Gherkin scenarios for composites (REQ-TRS-SYSCOMP-001): the closed vocabulary of a part-qualified
//! step language, and running a scenario against a [`SystemModel`] to a verdict.
//!
//! * `Given <part> is in state <State>` · `Given <part>.<variable> is <expression>`
//! * `When <part> receives <Event> [via <port>] [with <field> = <expression>, …]` · `When <part>.<variable> becomes <expression>` · `When <duration> elapses`
//! * `Then <part> is in state <State>` · `Then <expression over <part>.<variable>>` · `Then <part> sent <Event> [via <port>]` · `Then <part> did not send <Event> [via <port>]`
//! * `Given|When fault <id> is injected` · `When fault <id> is cleared` (REQ-TRS-FAULT-001: an `FMEAEntry` or `FaultTreeEvent` with an `injection:`)
//! * `Then <name> holds` · `Then within <duration>: <property>` · `Then always <property>` · `Then from now on <property>` ·
//!   `Then never <property>` (GH #342), where a property is an expression or `<name> holds` for a `Where <name> is …`

use std::collections::BTreeSet;

use crate::expr::{self, Value};
use crate::sm_run::Options;
use crate::sm_scen::{find_unquoted, split_top, Clause, GStep, Kw, Scenario, Status, Verdict};
use crate::system_run::{System, SystemModel};

#[derive(Debug, Clone, PartialEq)]
pub enum SCmd {
    InState(String, String),
    Set(String, String, String),
    Receive { part: String, event: String, via: Option<String>, fields: Vec<(String, String)> },
    Elapse(String),
    ExpectState(String, String),
    Expect(String),
    Sent { part: String, event: String, via: Option<String>, negate: bool },
    /// Inject (`true`) or clear (`false`) the fault of an FMEA row or fault-tree event.
    Fault { id: String, inject: bool },
    /// A property checked now or over the rest of the run (GH #342).
    Holds { when: Temporal, prop: PropRef },
}

/// When a property must hold (GH #342).
#[derive(Debug, Clone, PartialEq)]
pub enum Temporal {
    /// At this instant (`Then <name> holds`).
    Now,
    /// At some settled instant no later than the duration from now, and at every settled instant after that.
    Within(String),
    /// At every settled instant from now on (`always`, `from now on`).
    Always,
    /// At no settled instant from now on.
    Never,
}

/// A property: an expression, or a named one (`Where <name> is …`).
#[derive(Debug, Clone, PartialEq)]
pub enum PropRef {
    Expr(String),
    Named(String),
}

impl PropRef {
    fn parse(t: &str) -> Result<PropRef, String> {
        let t = t.trim();
        let lower = t.to_ascii_lowercase();
        if let Some(name) = lower.strip_suffix(" holds").map(|_| t[..t.len() - " holds".len()].trim()) {
            if name.is_empty() {
                return Err("`<name> holds` needs a name".into());
            }
            return Ok(PropRef::Named(name.to_string()));
        }
        expr::parse(t).map(|_| PropRef::Expr(t.to_string())).map_err(|e| format!("`{t}` is not an expression or `<name> holds`: {e}"))
    }
}

/// `within <duration>: <property>`, or `within <duration> <name> holds` (the longest prefix that is an expression is
/// the duration).
fn parse_within(rest: &str) -> Result<SCmd, String> {
    if let Some(i) = find_unquoted(rest, ":") {
        let d = rest[..i].trim();
        expr::parse(d).map_err(|e| format!("`{d}` is not a duration: {e}"))?;
        return Ok(SCmd::Holds { when: Temporal::Within(d.to_string()), prop: PropRef::parse(&rest[i + 1..])? });
    }
    let words: Vec<&str> = rest.split_whitespace().collect();
    for k in (1..words.len()).rev() {
        let d = words[..k].join(" ");
        if expr::parse(&d).is_ok() {
            if let Ok(prop @ PropRef::Named(_)) = PropRef::parse(&words[k..].join(" ")) {
                return Ok(SCmd::Holds { when: Temporal::Within(d), prop });
            }
        }
    }
    Err("`within <duration>: <property>` or `within <duration> <name> holds`".into())
}

fn word(s: &str) -> bool {
    !s.is_empty() && !s.contains(char::is_whitespace)
}

fn qualified(s: &str) -> Option<(String, String)> {
    let (p, v) = s.split_once('.')?;
    (word(p) && word(v)).then(|| (p.to_string(), v.to_string()))
}

/// Parse one step; `Err` is the reason it is outside the vocabulary (`E886`).
pub fn parse_step(kw: Kw, text: &str) -> Result<SCmd, String> {
    let t = text.trim();
    let lower = t.to_ascii_lowercase();
    if lower.starts_with("fault ") && (lower.ends_with(" is injected") || lower.ends_with(" is cleared")) {
        let rest = t["fault ".len()..].trim();
        let rl = rest.to_ascii_lowercase();
        for (suffix, inject) in [(" is injected", true), (" is cleared", false)] {
            if let Some(id) = rl.strip_suffix(suffix).map(|_| rest[..rest.len() - suffix.len()].trim()) {
                if !word(id) {
                    return Err(format!("'{id}' is not a fault id"));
                }
                return match (inject, kw) {
                    (_, Kw::Then) => Err("`fault … is injected/cleared` is a Given or When step".into()),
                    (false, Kw::Given) => Err("`fault … is cleared` is a When step".into()),
                    _ => Ok(SCmd::Fault { id: id.to_string(), inject }),
                };
            }
        }
        return Err("`fault <id> is injected` or `fault <id> is cleared`".into());
    }
    for (head, when) in [("within ", None), ("always ", Some(Temporal::Always)), ("from now on ", Some(Temporal::Always)), ("never ", Some(Temporal::Never))] {
        if lower.starts_with(head) {
            if kw != Kw::Then {
                return Err(format!("`{}` is a Then step", head.trim()));
            }
            let rest = &t[head.len()..];
            return match when {
                None => parse_within(rest),
                Some(when) => Ok(SCmd::Holds { when, prop: PropRef::parse(rest)? }),
            };
        }
    }
    if kw == Kw::Then && lower.ends_with(" holds") {
        return Ok(SCmd::Holds { when: Temporal::Now, prop: PropRef::parse(t)? });
    }
    if let Some(i) = find_unquoted(&lower, " is in state ") {
        let (part, state) = (t[..i].trim(), t[i + " is in state ".len()..].trim().trim_matches(['"', '\'']).replace('/', "."));
        if !word(part) || state.is_empty() {
            return Err("`is in state` is `<part> is in state <State>`".into());
        }
        return match kw {
            Kw::Given => Ok(SCmd::InState(part.to_string(), state)),
            Kw::Then => Ok(SCmd::ExpectState(part.to_string(), state)),
            Kw::When => Err("`is in state` is a Given or Then step".into()),
        };
    }
    for suffix in [" elapses", " elapse"] {
        if kw == Kw::When && lower.ends_with(suffix) {
            let d = t[..t.len() - suffix.len()].trim();
            return expr::parse(d).map(|_| SCmd::Elapse(d.to_string())).map_err(|e| format!("`{d}` is not a duration: {e}"));
        }
    }
    if let Some(i) = find_unquoted(&lower, " receives ") {
        if kw != Kw::When {
            return Err("`receives` is a When step".into());
        }
        let part = t[..i].trim();
        if !word(part) {
            return Err(format!("'{part}' is not a part name"));
        }
        let mut rest = t[i + " receives ".len()..].trim();
        let (event, tail) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
        if event.is_empty() {
            return Err("`receives` needs an event".into());
        }
        rest = tail.trim();
        let mut via = None;
        if let Some(r) = rest.strip_prefix("via ") {
            let (p, tail) = r.trim().split_once(char::is_whitespace).unwrap_or((r.trim(), ""));
            if p.is_empty() {
                return Err("`via` needs a port".into());
            }
            via = Some(p.to_string());
            rest = tail.trim();
        }
        let mut fields = Vec::new();
        if !rest.is_empty() {
            let Some(list) = rest.strip_prefix("with ") else { return Err(format!("expected `with <field> = <expression>`, got `{rest}`")) };
            for f in split_top(list) {
                let Some((k, v)) = f.split_once('=') else { return Err(format!("payload `{f}` is not <field> = <expression>")) };
                expr::parse(v.trim()).map_err(|e| format!("payload value `{}`: {e}", v.trim()))?;
                fields.push((k.trim().to_string(), v.trim().to_string()));
            }
        }
        return Ok(SCmd::Receive { part: part.to_string(), event: event.to_string(), via, fields });
    }
    for (sep, negate) in [(" did not send ", true), (" sent ", false)] {
        if let Some(i) = find_unquoted(&lower, sep) {
            if kw != Kw::Then {
                return Err("`sent` / `did not send` are Then steps".into());
            }
            let part = t[..i].trim();
            let mut it = t[i + sep.len()..].split_whitespace();
            let (Some(event), true) = (it.next(), word(part)) else { return Err("`<part> sent <Event> [via <port>]`".into()) };
            let via = match (it.next(), it.next(), it.next()) {
                (None, _, _) => None,
                (Some("via"), Some(p), None) => Some(p.to_string()),
                _ => return Err("expected `via <port>` after the event".into()),
            };
            return Ok(SCmd::Sent { part: part.to_string(), event: event.to_string(), via, negate });
        }
    }
    if let Some(i) = find_unquoted(&lower, " becomes ") {
        if kw != Kw::When {
            return Err("`becomes` is a When step".into());
        }
        let (var, e) = (t[..i].trim(), t[i + " becomes ".len()..].trim());
        let Some((p, v)) = qualified(var) else { return Err(format!("'{var}' is not `<part>.<variable>`")) };
        return expr::parse(e).map(|_| SCmd::Set(p, v, e.to_string())).map_err(|x| format!("`{e}`: {x}"));
    }
    match kw {
        Kw::Given => {
            if let Some(i) = find_unquoted(&lower, " is ") {
                let (var, e) = (t[..i].trim(), t[i + 4..].trim());
                if let Some((p, v)) = qualified(var) {
                    return expr::parse(e).map(|_| SCmd::Set(p, v, e.to_string())).map_err(|x| format!("`{e}`: {x}"));
                }
            }
            Err("a Given step is `<part> is in state <State>` or `<part>.<variable> is <expression>`".into())
        }
        Kw::Then => expr::parse(t).map(|_| SCmd::Expect(t.to_string())).map_err(|e| format!("`{t}` is not an expression: {e}")),
        Kw::When => Err("a When step is `<part> receives <Event> [via <port>] [with …]`, `<part>.<variable> becomes <expression>` or `<duration> elapses`".into()),
    }
}

/// `E886` findings of a composite TestCase body: `(line, message)` for each step outside the vocabulary.
pub fn vocabulary_findings(doc: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for sc in crate::sm_scen::extract_scenarios(doc) {
        if sc.steps.is_empty() {
            out.push((0, format!("scenario '{}': the scenario has no steps", sc.title)));
        }
        for (line, e) in &sc.prop_errors {
            if seen.insert((*line, String::new())) {
                out.push((*line, format!("scenario '{}': {e}", sc.title)));
            }
        }
        for st in &sc.steps {
            if seen.insert((st.line, st.text.clone())) {
                match parse_step(st.kw, &st.text) {
                    Err(e) => out.push((st.line, format!("scenario '{}': step `{}` is outside the executable vocabulary: {e}", sc.title, st.text))),
                    Ok(SCmd::Holds { prop: PropRef::Named(n), .. }) if !sc.props.iter().any(|p| p.name.eq_ignore_ascii_case(&n)) => {
                        out.push((st.line, format!("scenario '{}': step `{}` names no property: there is no `Where {n} is <expression>`", sc.title, st.text)))
                    }
                    Ok(_) => {}
                }
            }
        }
    }
    out
}

/// Run one scenario on a fresh system.
pub fn run_system_scenario(model: &SystemModel, sc: &Scenario, opts: &Options) -> Verdict {
    run_system_scenario_faulted(model, sc, opts, &[])
}

/// A property checked at every settled instant from the step that set it to the end of the run (GH #342).
struct Monitor {
    prop: PropRef,
    kind: MKind,
}

enum MKind {
    /// Reached no later than `deadline`, and kept from then on.
    Within { deadline: f64, dur: String, reached: Option<f64> },
    Always,
    Never,
}

fn eval_bool(sys: &System, src: &str, what: &str) -> Result<bool, String> {
    match sys.eval_qualified(src)? {
        Value::Bool(b) => Ok(b),
        v => Err(format!("{what} is a {}, not a boolean", v.kind())),
    }
}

/// Evaluate a property now: whether it holds, and what was checked (the clause in force for a named property). A
/// degraded clause is added to `used` only when it carried the verdict: it holds while the full property does not.
/// `degradable` is false for `never`, where a weaker property would make the check stricter: the full property
/// judges it.
fn eval_prop(sys: &System, sc: &Scenario, prop: &PropRef, degradable: bool, used: &mut BTreeSet<String>) -> Result<(bool, String), String> {
    match prop {
        PropRef::Expr(e) => {
            let what = format!("`{e}`");
            Ok((eval_bool(sys, e, &what)?, what))
        }
        PropRef::Named(n) => {
            let Some(p) = sc.props.iter().find(|p| p.name.eq_ignore_ascii_case(n)) else {
                return Err(format!("no property '{n}' (`Where {n} is <expression>`)"));
            };
            let full_what = format!("'{}' (`{}`)", p.name, p.expr);
            let clause = if degradable { Some(p.clause(&sys.active_faults())?) } else { None };
            match clause {
                Some(Clause::Degraded(x)) => {
                    let what = format!("'{}' as degraded by `{}` (`{}`)", p.name, x.label(), x.expr);
                    let holds = eval_bool(sys, &x.expr, &what)?;
                    if holds && !eval_bool(sys, &p.expr, &full_what)? {
                        used.insert(format!("'{}': {}", p.name, x.label()));
                    }
                    Ok((holds, what))
                }
                _ => Ok((eval_bool(sys, &p.expr, &full_what)?, full_what)),
            }
        }
    }
}

fn secs(t: f64) -> String {
    format!("{} s", crate::sm_run::quantise(t))
}

/// Check every monitor at the instant the system has settled at.
fn check(sys: &System, sc: &Scenario, monitors: &mut [Monitor], used: &mut BTreeSet<String>) -> Result<(), (Status, String)> {
    let now = sys.time();
    for m in monitors.iter_mut() {
        let (holds, what) = eval_prop(sys, sc, &m.prop, !matches!(m.kind, MKind::Never), used).map_err(|e| (Status::Error, e))?;
        let fail = |msg: String| Err((Status::Fail, msg));
        match &mut m.kind {
            MKind::Within { deadline, dur, reached } => match (*reached, holds) {
                (None, true) => *reached = Some(now),
                (None, false) if now >= *deadline - 1e-9 => return fail(format!("{what} is not reached within {dur} (by t = {})", secs(*deadline))),
                (Some(at), false) => return fail(format!("{what} was reached at t = {} but no longer holds at t = {}", secs(at), secs(now))),
                _ => {}
            },
            MKind::Always if !holds => return fail(format!("{what} does not hold at t = {}", secs(now))),
            MKind::Never if holds => return fail(format!("{what} holds at t = {}, and must never", secs(now))),
            _ => {}
        }
    }
    Ok(())
}

/// Advance the clock, checking the monitors at every settled instant; stops at the first violation.
fn advance_checked(sys: &mut System, sc: &Scenario, by: f64, monitors: &mut [Monitor], used: &mut BTreeSet<String>) -> Result<(), (Status, String)> {
    let mut failed = None;
    sys.advance_with(by, &mut |s| match check(s, sc, monitors, used) {
        Ok(()) => true,
        Err(e) => {
            failed = Some(e);
            false
        }
    });
    match failed {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// Run one scenario on a fresh system with the faults of `faults` (analysis element ids) injected before its first step.
pub fn run_system_scenario_faulted(model: &SystemModel, sc: &Scenario, opts: &Options, faults: &[String]) -> Verdict {
    let mut sys = System::new(model, opts.clone());
    let injected: Result<(), String> = faults.iter().try_for_each(|f| sys.inject(f).map_err(|e| format!("injecting {f}: {e}")));
    let steps: Vec<String> = sc.steps.iter().map(|s: &GStep| format!("{} {}", match s.kw { Kw::Given => "Given", Kw::When => "When", Kw::Then => "Then" }, s.text)).collect();
    let mut used: BTreeSet<String> = BTreeSet::new();
    let mut monitors: Vec<Monitor> = Vec::new();
    let done = |sys: &System, used: &BTreeSet<String>, status: Status, message: Option<String>| Verdict {
        degraded: used.iter().cloned().collect(),
        scenario: sc.title.clone(),
        outline: sc.outline.clone(),
        status,
        message,
        steps: steps.clone(),
        trace: sys
            .trace()
            .into_iter()
            .map(|(part, mut s)| {
                s.cause = format!("[{part}] {}", s.cause);
                s
            })
            .collect(),
        coverage: Default::default(),
        act_coverage: Default::default(),
        part_coverage: sys.part_coverage(),
        plants: sys.plant_provenance(),
        messages: sys.fault_events().iter().cloned().chain(sys.messages().iter().map(|m| sys.describe(m))).collect(),
    };
    if let Err(e) = injected {
        return done(&sys, &used, Status::Error, Some(e));
    }
    if let Some(e) = sys.error() {
        return done(&sys, &used, Status::Error, Some(e));
    }
    if sc.steps.is_empty() {
        return done(&sys, &used, Status::Error, Some("the scenario has no steps".into()));
    }
    for (i, st) in sc.steps.iter().enumerate() {
        let label = format!("step {} (`{}`)", i + 1, steps[i]);
        let cmd = match parse_step(st.kw, &st.text) {
            Ok(c) => c,
            Err(e) => return done(&sys, &used, Status::Error, Some(format!("{label}: {e}"))),
        };
        let eval = |sys: &System, src: &str| sys.eval_qualified(src);
        let applied: Result<(), String> = match &cmd {
            SCmd::InState(p, s) => {
                sys.force(p, s);
                Ok(())
            }
            SCmd::Set(p, v, e) => eval(&sys, e).map(|val| sys.set(p, v, val)),
            SCmd::Receive { part, event, via, fields } => fields
                .iter()
                .map(|(k, v)| eval(&sys, v).map(|x| (k.clone(), x)))
                .collect::<Result<Vec<_>, _>>()
                .map(|p| {
                    let p: Vec<(&str, Value)> = p.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                    sys.receive(part, via.as_deref(), event, &p);
                }),
            SCmd::Elapse(d) => match eval(&sys, d) {
                Ok(Value::Num(q)) if expr::unit_scale("s").map(|x| x.1) == Some(q.dim) && q.v >= 0.0 => {
                    if let Err((status, m)) = advance_checked(&mut sys, sc, q.v, &mut monitors, &mut used) {
                        return done(&sys, &used, status, Some(format!("{label}: {m}")));
                    }
                    Ok(())
                }
                Ok(v) => Err(format!("`{d}` is a {}, not a non-negative duration", v.kind())),
                Err(e) => Err(e),
            },
            SCmd::ExpectState(p, s) => match sys.holds_state(p, s) {
                Ok(true) => Ok(()),
                Ok(false) => return done(&sys, &used, Status::Fail, Some(format!("{label}: expected {p} in state {s}, it is in {}", sys.active(p).unwrap_or_default().join(", ")))),
                Err(e) => Err(e),
            },
            SCmd::Expect(e) => match eval(&sys, e) {
                Ok(Value::Bool(true)) => Ok(()),
                Ok(Value::Bool(false)) => return done(&sys, &used, Status::Fail, Some(format!("{label}: `{e}` is false"))),
                Ok(v) => Err(format!("`{e}` is a {}, not a boolean", v.kind())),
                Err(x) => Err(x),
            },
            SCmd::Fault { id, inject } => if *inject { sys.inject(id) } else { sys.clear_fault(id) },
            SCmd::Holds { when: Temporal::Now, prop } => match eval_prop(&sys, sc, prop, true, &mut used) {
                Ok((true, _)) => Ok(()),
                Ok((false, what)) => return done(&sys, &used, Status::Fail, Some(format!("{label}: {what} does not hold"))),
                Err(e) => return done(&sys, &used, Status::Error, Some(format!("{label}: {e}"))),
            },
            SCmd::Holds { when: Temporal::Within(d), prop } => match eval(&sys, d) {
                Ok(Value::Num(q)) if expr::unit_scale("s").map(|x| x.1) == Some(q.dim) && q.v >= 0.0 => {
                    monitors.push(Monitor { prop: prop.clone(), kind: MKind::Within { deadline: sys.time() + q.v, dur: d.clone(), reached: None } });
                    let checked = check(&sys, sc, &mut monitors, &mut used).and_then(|_| advance_checked(&mut sys, sc, q.v, &mut monitors, &mut used));
                    if let Err((status, m)) = checked {
                        return done(&sys, &used, status, Some(format!("{label}: {m}")));
                    }
                    Ok(())
                }
                Ok(v) => Err(format!("`{d}` is a {}, not a non-negative duration", v.kind())),
                Err(e) => Err(e),
            },
            SCmd::Holds { when, prop } => {
                monitors.push(Monitor { prop: prop.clone(), kind: if *when == Temporal::Never { MKind::Never } else { MKind::Always } });
                Ok(())
            }
            SCmd::Sent { part, event, via, negate } => match model.spec.instance(part) {
                None => Err(format!("no part '{part}'")),
                Some(pi) => {
                    let hit = sys.messages().iter().any(|m| m.from == pi && m.event == *event && via.as_ref().is_none_or(|v| m.from_port.as_deref() == Some(v.as_str())));
                    if hit == *negate {
                        let what = if *negate { "sent it" } else { "did not send it" };
                        return done(&sys, &used, Status::Fail, Some(format!("{label}: {part} {what}")));
                    }
                    Ok(())
                }
            },
        };
        if let Err(e) = applied {
            return done(&sys, &used, Status::Error, Some(format!("{label}: {e}")));
        }
        if let Some(e) = sys.error() {
            return done(&sys, &used, Status::Error, Some(format!("{label}: run error: {e}")));
        }
        // every settled instant counts, including the one a step leaves the system in
        if let Err((status, m)) = check(&sys, sc, &mut monitors, &mut used) {
            return done(&sys, &used, status, Some(format!("{label}: {m}")));
        }
    }
    done(&sys, &used, Status::Pass, None)
}

/// The fault ids named by `fault … is injected/cleared` steps of a TestCase body, with their lines (for `E937`).
pub fn fault_ids_in(doc: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for sc in crate::sm_scen::extract_scenarios(doc) {
        for st in &sc.steps {
            if let Ok(SCmd::Fault { id, .. }) = parse_step(st.kw, &st.text) {
                if seen.insert((st.line, id.clone())) {
                    out.push((st.line, id));
                }
            }
        }
        // the faults an `Except under` clause names must be faults of the composite too (GH #342)
        for x in sc.props.iter().flat_map(|p| &p.excepts) {
            for id in &x.faults {
                if seen.insert((x.line, id.clone())) {
                    out.push((x.line, id.clone()));
                }
            }
        }
    }
    out
}

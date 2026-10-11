//! Executable Gherkin scenarios for composites (REQ-TRS-SYSCOMP-001): the closed vocabulary of a part-qualified
//! step language, and running a scenario against a [`SystemModel`] to a verdict.
//!
//! * `Given <part> is in state <State>` · `Given <part>.<variable> is <expression>`
//! * `When <part> receives <Event> [via <port>] [with <field> = <expression>, …]` · `When <part>.<variable> becomes <expression>` · `When <duration> elapses`
//! * `Then <part> is in state <State>` · `Then <expression over <part>.<variable>>` · `Then <part> sent <Event> [via <port>]` · `Then <part> did not send <Event> [via <port>]`

use crate::expr::{self, Value};
use crate::sm_run::Options;
use crate::sm_scen::{find_unquoted, split_top, GStep, Kw, Scenario, Status, Verdict};
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
        for st in &sc.steps {
            if seen.insert((st.line, st.text.clone())) {
                if let Err(e) = parse_step(st.kw, &st.text) {
                    out.push((st.line, format!("scenario '{}': step `{}` is outside the executable vocabulary: {e}", sc.title, st.text)));
                }
            }
        }
    }
    out
}

/// Run one scenario on a fresh system.
pub fn run_system_scenario(model: &SystemModel, sc: &Scenario, opts: &Options) -> Verdict {
    let mut sys = System::new(model, opts.clone());
    let steps: Vec<String> = sc.steps.iter().map(|s: &GStep| format!("{} {}", match s.kw { Kw::Given => "Given", Kw::When => "When", Kw::Then => "Then" }, s.text)).collect();
    let done = |sys: &System, status: Status, message: Option<String>| Verdict {
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
        plants: sys.plant_provenance(),
        messages: sys.messages().iter().map(|m| sys.describe(m)).collect(),
    };
    if let Some(e) = sys.error() {
        return done(&sys, Status::Error, Some(e));
    }
    if sc.steps.is_empty() {
        return done(&sys, Status::Error, Some("the scenario has no steps".into()));
    }
    for (i, st) in sc.steps.iter().enumerate() {
        let label = format!("step {} (`{}`)", i + 1, steps[i]);
        let cmd = match parse_step(st.kw, &st.text) {
            Ok(c) => c,
            Err(e) => return done(&sys, Status::Error, Some(format!("{label}: {e}"))),
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
                    sys.advance(q.v);
                    Ok(())
                }
                Ok(v) => Err(format!("`{d}` is a {}, not a non-negative duration", v.kind())),
                Err(e) => Err(e),
            },
            SCmd::ExpectState(p, s) => match sys.holds_state(p, s) {
                Ok(true) => Ok(()),
                Ok(false) => return done(&sys, Status::Fail, Some(format!("{label}: expected {p} in state {s}, it is in {}", sys.active(p).unwrap_or_default().join(", ")))),
                Err(e) => Err(e),
            },
            SCmd::Expect(e) => match eval(&sys, e) {
                Ok(Value::Bool(true)) => Ok(()),
                Ok(Value::Bool(false)) => return done(&sys, Status::Fail, Some(format!("{label}: `{e}` is false"))),
                Ok(v) => Err(format!("`{e}` is a {}, not a boolean", v.kind())),
                Err(x) => Err(x),
            },
            SCmd::Sent { part, event, via, negate } => match model.spec.instance(part) {
                None => Err(format!("no part '{part}'")),
                Some(pi) => {
                    let hit = sys.messages().iter().any(|m| m.from == pi && m.event == *event && via.as_ref().is_none_or(|v| m.from_port.as_deref() == Some(v.as_str())));
                    if hit == *negate {
                        let what = if *negate { "sent it" } else { "did not send it" };
                        return done(&sys, Status::Fail, Some(format!("{label}: {part} {what}")));
                    }
                    Ok(())
                }
            },
        };
        if let Err(e) = applied {
            return done(&sys, Status::Error, Some(format!("{label}: {e}")));
        }
        if let Some(e) = sys.error() {
            return done(&sys, Status::Error, Some(format!("{label}: run error: {e}")));
        }
    }
    done(&sys, Status::Pass, None)
}

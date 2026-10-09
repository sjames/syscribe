//! Export of `ActionDef`/`Action` and `StateDef`/`State` bodies as SysML v2 text
//! (`REQ-TRS-SYSMLV2-056`..`-058`, `ADR-SYS-SYSMLV2-002` addendum).
//!
//! The writer is held to one rule: it only emits a statement that the real ingestion
//! converters (`REQ-TRS-SYSMLV2-018`/`-019`) read back as the *same* native value. Every
//! top-level entry is generated in the grammar ingestion accepts, then re-parsed through
//! [`super::ingest::probe_action_body`]/[`super::ingest::probe_state_body`]; an entry whose
//! probe differs (unknown fields, a hand-chosen name where ingestion synthesizes one, text that
//! is not valid SysML v2) becomes `//` comment lines instead of an approximation.

use std::collections::HashMap;

use serde_yaml::{Mapping, Value};

use super::export::{esc_single, sysml_ident};
use super::ingest::{canonical_expression, probe_action_body, probe_part_body, probe_state_body};
use crate::element::RawFrontmatter;

type Refs<'a> = &'a dyn Fn(&str) -> String;

fn key(k: &str) -> Value {
    Value::String(k.to_string())
}

fn get<'a>(m: &'a Mapping, k: &str) -> Option<&'a Value> {
    m.get(key(k))
}

fn text<'a>(m: &'a Mapping, k: &str) -> Option<&'a str> {
    get(m, k).and_then(Value::as_str)
}

/// A scalar field as text: hand-authored YAML may write `value: 10` (a number) where ingestion
/// always stores the expression text as a string; both mean the same source text.
fn scalar(m: &Mapping, k: &str) -> Option<String> {
    match get(m, k)? {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Every key of `m` is one of `allowed` (anything else cannot be read back).
fn only_keys(m: &Mapping, allowed: &[&str]) -> bool {
    m.keys().all(|k| k.as_str().is_some_and(|k| allowed.contains(&k)))
}

/// A dotted feature-chain name (`a.b`) with each segment quoted when needed.
fn chain(name: &str) -> String {
    super::export::qualified(name)
}

fn comment(pad: &str, what: &str, label: &str, reason: &str) -> String {
    let label = label.replace('\n', " ");
    format!("{pad}// {what} not exported ({reason}): {label}\n")
}

/// Number of `// … not exported (…): …` comment lines in generated behaviour text (the format
/// [`comment`] writes), for the export report (`REQ-TRS-SYSMLV2-072`).
pub(super) fn count_degraded(text: &str) -> usize {
    text.lines().filter(|l| l.trim_start().starts_with("// ") && l.contains(" not exported (")).count()
}

fn describe(v: &Value) -> String {
    match v.as_mapping() {
        Some(m) => {
            let name = text(m, "name").unwrap_or("?");
            let kind = text(m, "kind").map(|k| format!("{k} ")).unwrap_or_default();
            format!("{kind}{name}")
        }
        None => "non-mapping entry".to_string(),
    }
}

/// Keys whose value is expression text that ingestion re-renders (`and` becomes `&&`, a bare number
/// becomes its text, …). `REQ-TRS-SYSMLV2-068`.
const EXPRESSION_KEYS: &[&str] = &["guard", "condition", "untilCondition", "value", "sequence", "target"];

/// A plain (possibly qualified or dotted) name, as opposed to an expression (`REQ-TRS-SYSMLV2-096`).
fn is_plain_ref(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == ':')
}

/// The canonical form of a native value for the read-back comparison: expression fields in the
/// rendering ingestion produces, and an `accept:` mapping holding only `payload:` as the plain
/// string (`REQ-TRS-SYSMLV2-067`/`-068`). Applied to both sides, so only genuinely different
/// content still degrades.
fn canon(v: &Value) -> Value {
    match v {
        Value::Mapping(m) => {
            let mut out = Mapping::new();
            // `a.b := v` and `target: a, referent: b` are one statement (`REQ-TRS-SYSMLV2-079`).
            let split = (m.get(key("kind")).and_then(Value::as_str) == Some("AssignmentAction") && m.get(key("referent")).is_none())
                .then(|| m.get(key("target")).and_then(Value::as_str).and_then(super::ingest::split_feature_chain))
                .flatten();
            for (k, val) in m {
                if let Some((t, r)) = &split {
                    match k.as_str() {
                        Some("target") => {
                            out.insert(k.clone(), Value::String(t.clone()));
                            out.insert(key("referent"), Value::String(r.clone()));
                            continue;
                        }
                        _ => {}
                    }
                }
                let name = k.as_str().unwrap_or("");
                let nv = if EXPRESSION_KEYS.contains(&name) {
                    match val {
                        Value::String(_) | Value::Number(_) | Value::Bool(_) => {
                            let raw = match val {
                                Value::String(s) => s.clone(),
                                other => scalar_text(other),
                            };
                            Value::String(canonical_expression(&raw).unwrap_or(raw))
                        }
                        other => canon(other),
                    }
                } else if name == "accept" {
                    match val {
                        Value::Mapping(am) if am.len() == 1 && am.get(key("payload")).is_some_and(Value::is_string) => {
                            am.get(key("payload")).cloned().unwrap_or(Value::Null)
                        }
                        other => canon(other),
                    }
                } else {
                    canon(val)
                };
                out.insert(k.clone(), nv);
            }
            Value::Mapping(out)
        }
        Value::Sequence(seq) => Value::Sequence(seq.iter().map(canon).collect()),
        other => other.clone(),
    }
}

fn scalar_text(v: &Value) -> String {
    match v {
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

fn canon_list(list: &[Value]) -> Vec<Value> {
    list.iter().map(canon).collect()
}

/// `@SyscribeStep { k = 'v'; … }` for the extension fields of one step (`REQ-TRS-SYSMLV2-069`/`-070`).
fn step_annotation(fields: &[(&str, String)], pad: &str) -> String {
    if fields.is_empty() {
        return String::new();
    }
    let mut s = format!("{pad}@SyscribeStep {{\n");
    for (k, v) in fields {
        s.push_str(&format!("{pad}    {k} = '{}';\n", esc_single(v)));
    }
    s.push_str(&format!("{pad}}}\n"));
    s
}

// ── Actions ────────────────────────────────────────────────────────────────

/// Per-body counters for the names ingestion synthesizes (`if_1`, `while_2`, …).
#[derive(Default)]
struct Counters(HashMap<&'static str, u32>);

impl Counters {
    fn expected(&self, kind: &'static str) -> String {
        format!("{kind}_{}", self.0.get(kind).copied().unwrap_or(0) + 1)
    }
    fn bump(&mut self, kind: &'static str) {
        *self.0.entry(kind).or_insert(0) += 1;
    }
}

/// Generate a nested body (loop/if branch): text plus the entries actually kept.
fn action_list(list: &[Value], pad: &str, r: Refs) -> (String, Vec<Value>) {
    let mut counters = Counters::default();
    let mut out = String::new();
    let mut kept = Vec::new();
    for v in list {
        match action_entry(v, &mut counters, pad, r) {
            Ok((t, k)) => {
                out.push_str(&t);
                kept.push(k);
            }
            Err(reason) => out.push_str(&comment(pad, "subAction", &describe(v), &reason)),
        }
    }
    (out, kept)
}

fn nested(m: &Mapping, k: &str, pad: &str, r: Refs) -> Result<(String, Vec<Value>), String> {
    match get(m, k) {
        None => Ok((String::new(), Vec::new())),
        Some(Value::Sequence(seq)) => Ok(action_list(seq, &format!("{pad}    "), r)),
        Some(_) => Err(format!("`{k}` is not a list")),
    }
}

fn with_list(mut m: Mapping, k: &str, kept: Vec<Value>) -> Value {
    if !kept.is_empty() {
        m.insert(key(k), Value::Sequence(kept));
    }
    Value::Mapping(m)
}

/// One `subActions:` entry as statement text and the value ingestion must give back.
fn action_entry(v: &Value, c: &mut Counters, pad: &str, r: Refs) -> Result<(String, Value), String> {
    let m = v.as_mapping().ok_or("not a mapping")?;
    let name = text(m, "name").filter(|n| !n.is_empty()).ok_or("no name")?;
    let kind = text(m, "kind").ok_or("no kind")?;
    let mut base = Mapping::new();
    base.insert(key("name"), key(name));
    base.insert(key("kind"), key(kind));

    // Kinds whose name ingestion synthesizes: the bare statement when the name is exactly the
    // positional one, otherwise the named-step wrapper `action <name> { <stmt> }`
    // (REQ-TRS-SYSMLV2-060/-061), which leaves the synthesized-name counters untouched.
    // An entry with `@SyscribeStep` extras is always written as the wrapper, which owns the annotation.
    // `REQ-TRS-SYSMLV2-079`: `referent` and `until` are native statements now; only `valueKind` of an
    // assign still travels in the (deprecated) annotation.
    let has_extras = get(m, "valueKind").is_some();
    let bare_for = |c: &Counters, k: &'static str| !has_extras && name == c.expected(k);
    let inner_pad = |bare: bool| if bare { pad.to_string() } else { format!("{pad}    ") };
    let wrap = |bare: bool, stmt: String, ann: String| -> String {
        if bare {
            stmt
        } else {
            format!("{pad}action {} {{\n{stmt}{ann}{pad}}}\n", sysml_ident(name))
        }
    };

    match kind {
        "PerformAction" => {
            if !only_keys(m, &["name", "kind", "typedBy"]) {
                return Err("unsupported fields".into());
            }
            let mut s = format!("{pad}action {}", sysml_ident(name));
            if let Some(t) = text(m, "typedBy") {
                s.push_str(&format!(" : {}", r(t)));
                base.insert(key("typedBy"), key(t));
            }
            s.push_str(";\n");
            Ok((s, Value::Mapping(base)))
        }
        "AcceptAction" | "SendAction" => {
            let accept = kind == "AcceptAction";
            let allowed: &[&str] = if accept {
                &["name", "kind", "payload", "via", "trigger"]
            } else {
                &["name", "kind", "payload", "via", "to"]
            };
            if !only_keys(m, allowed) {
                return Err("unsupported fields".into());
            }
            // `REQ-TRS-SYSMLV2-078`: a payload-less accept with a time/change trigger is
            // `accept after|when|at <e>;` (or `action <n> accept ...;` for a hand-chosen name).
            if accept && get(m, "payload").is_none() {
                let tm = get(m, "trigger").and_then(Value::as_mapping).ok_or("no payload")?;
                let (kw, field) = match text(tm, "kind") {
                    Some("timeOut") => ("after", "when"),
                    Some("change") => ("when", "condition"),
                    Some("at") => ("at", "when"),
                    _ => return Err("trigger has no native syntax".into()),
                };
                if !only_keys(tm, &["kind", field]) || get(m, "via").is_some() {
                    return Err("unsupported trigger".into());
                }
                let e = scalar(tm, field).ok_or("trigger has no expression")?;
                let bare = name == c.expected("accept");
                if bare {
                    c.bump("accept");
                }
                let mut tv = Mapping::new();
                tv.insert(key("kind"), key(text(tm, "kind").unwrap_or_default()));
                tv.insert(key(field), key(&e));
                base.insert(key("trigger"), Value::Mapping(tv));
                let s = if bare {
                    format!("{pad}accept {kw} {e};\n")
                } else {
                    format!("{pad}action {} accept {kw} {e};\n", sysml_ident(name))
                };
                return Ok((s, Value::Mapping(base)));
            }
            let payload = text(m, "payload").ok_or("no payload")?;
            let mut extras: Vec<(&str, String)> = Vec::new();
            let mut tail = String::new();
            if let Some(v) = get(m, "via") {
                let via = v.as_str().ok_or("`via` is not text")?;
                tail.push_str(&format!(" via {}", chain(via)));
                base.insert(key("via"), key(via));
            }
            if let Some(v) = get(m, "to") {
                let to = v.as_str().ok_or("`to` is not text")?;
                tail.push_str(&format!(" to {}", chain(to)));
                base.insert(key("to"), key(to));
            }
            // A trigger beside a payload has no SysML syntax: the deprecated annotation carries it.
            if let Some(t) = get(m, "trigger") {
                let tm = t.as_mapping().filter(|tm| only_keys(tm, &["kind", "condition"])).ok_or("unsupported trigger")?;
                let (k, c) = (text(tm, "kind").ok_or("trigger has no kind")?, scalar(tm, "condition").ok_or("trigger has no condition")?);
                extras.push(("triggerKind", k.to_string()));
                extras.push(("triggerCondition", c.clone()));
                let mut tv = Mapping::new();
                tv.insert(key("kind"), key(k));
                tv.insert(key("condition"), key(&c));
                base.insert(key("trigger"), Value::Mapping(tv));
            }
            let kw = if accept { "accept" } else { "send" };
            let mut s = format!("{pad}{kw} ");
            // `REQ-TRS-SYSMLV2-096`: a payload that is an expression (`new Cmd()`) is written as that
            // expression, unquoted, when ingestion renders it back identically; a quoted name otherwise.
            if payload == name && !is_plain_ref(name) && canonical_expression(name).as_deref() == Some(name) {
                s.push_str(name);
            } else {
                s.push_str(&sysml_ident(name));
                if payload != name {
                    s.push_str(&format!(" : {}", r(payload)));
                }
            }
            s.push_str(&tail);
            if extras.is_empty() {
                s.push_str(";\n");
            } else {
                s.push_str(&format!(" {{\n{}{pad}}}\n", step_annotation(&extras, &format!("{pad}    "))));
            }
            base.insert(key("payload"), key(payload));
            Ok((s, Value::Mapping(base)))
        }
        "AssignmentAction" => {
            if !only_keys(m, &["name", "kind", "target", "value", "referent", "valueKind"]) {
                return Err("unsupported fields".into());
            }
            let bare = bare_for(c, "assign");
            let (t, v) = (scalar(m, "target").ok_or("no target")?, scalar(m, "value").ok_or("no value")?);
            base.insert(key("target"), key(&t));
            base.insert(key("value"), key(&v));
            let mut extras: Vec<(&str, String)> = Vec::new();
            let mut lhs = t.clone();
            if let Some(x) = get(m, "referent") {
                let x = x.as_str().ok_or("`referent` is not text")?;
                lhs = format!("{t}.{x}");
                base.insert(key("referent"), key(x));
            }
            if let Some(x) = get(m, "valueKind") {
                let x = x.as_str().ok_or("`valueKind` is not text")?;
                extras.push(("valueKind", x.to_string()));
                base.insert(key("valueKind"), key(x));
            }
            if bare {
                c.bump("assign");
            }
            let stmt = format!("{}assign {lhs} := {v};\n", inner_pad(bare));
            let ann = step_annotation(&extras, &inner_pad(bare));
            Ok((wrap(bare, stmt, ann), Value::Mapping(base)))
        }
        "TerminateAction" => {
            if !only_keys(m, &["name", "kind", "target"]) {
                return Err("unsupported fields".into());
            }
            let bare = bare_for(c, "terminate");
            if bare {
                c.bump("terminate");
            }
            let ip = inner_pad(bare);
            match text(m, "target") {
                Some(t) => {
                    base.insert(key("target"), key(t));
                    Ok((wrap(bare, format!("{ip}terminate {};\n", chain(t)), String::new()), Value::Mapping(base)))
                }
                None => Ok((wrap(bare, format!("{ip}terminate;\n"), String::new()), Value::Mapping(base))),
            }
        }
        "LoopAction" => {
            let loop_kind = text(m, "loopKind").ok_or("no loopKind")?;
            let (allowed, kind_key, head): (&[&str], &'static str, String) = match loop_kind {
                // `REQ-TRS-SYSMLV2-089`: `while c { } until d;` keeps both conditions.
                "while" => (
                    &["name", "kind", "loopKind", "condition", "untilCondition", "body"],
                    "while",
                    format!("while {}", scalar(m, "condition").ok_or("no condition")?),
                ),
                "loop" => (&["name", "kind", "loopKind", "body"], "loop", "loop".to_string()),
                // `REQ-TRS-SYSMLV2-079`: `loop { } until <c>;` (0.57); `condition` goes after the body.
                "until" => (&["name", "kind", "loopKind", "condition", "body"], "loop", "loop".to_string()),
                "for" => (
                    &["name", "kind", "loopKind", "variable", "sequence", "body"],
                    "for",
                    format!(
                        "for {} in {}",
                        sysml_ident(text(m, "variable").ok_or("no variable")?),
                        scalar(m, "sequence").ok_or("no sequence")?
                    ),
                ),
                other => return Err(format!("unknown loopKind '{other}'")),
            };
            if !only_keys(m, allowed) {
                return Err("unsupported fields".into());
            }
            let bare = bare_for(c, kind_key);
            let ip = inner_pad(bare);
            base.insert(key("loopKind"), key(loop_kind));
            for k in ["condition", "untilCondition", "variable", "sequence"] {
                if let Some(t) = scalar(m, k) {
                    base.insert(key(k), key(&t));
                }
            }
            let (body, kept) = nested(m, "body", &ip, r)?;
            if bare {
                c.bump(kind_key);
            }
            let until = match loop_kind {
                "until" => format!(" until {};", scalar(m, "condition").ok_or("no condition")?),
                "while" => scalar(m, "untilCondition").map(|u| format!(" until {u};")).unwrap_or_default(),
                _ => String::new(),
            };
            Ok((wrap(bare, format!("{ip}{head} {{\n{body}{ip}}}{until}\n"), String::new()), with_list(base, "body", kept)))
        }
        "IfAction" => {
            if !only_keys(m, &["name", "kind", "condition", "then", "else"]) {
                return Err("unsupported fields".into());
            }
            let bare = bare_for(c, "if");
            let ip = inner_pad(bare);
            let cond = scalar(m, "condition").ok_or("no condition")?;
            base.insert(key("condition"), key(&cond));
            let (then_t, then_k) = nested(m, "then", &ip, r)?;
            let mut s = format!("{ip}if {cond} {{\n{then_t}{ip}}}");
            let mut out = with_list(base, "then", then_k);
            if get(m, "else").is_some() {
                let (else_t, else_k) = nested(m, "else", &ip, r)?;
                s.push_str(&format!(" else {{\n{else_t}{ip}}}"));
                if let Value::Mapping(om) = &mut out {
                    if !else_k.is_empty() {
                        om.insert(key("else"), Value::Sequence(else_k));
                    }
                }
            }
            s.push('\n');
            if bare {
                c.bump("if");
            }
            Ok((wrap(bare, s, String::new()), out))
        }
        other => Err(format!("unsupported kind '{other}'")),
    }
}

/// `subActions:`/`controlNodes:`/`successionConnections:` of an `ActionDef`/`Action`.
pub(super) fn action_body(fm: &RawFrontmatter, is_usage: bool, pad: &str, r: Refs) -> String {
    let mut out = String::new();
    let mut counters = Counters::default();
    // Names of top-level entries written as comments: a succession must not reference them.
    let mut dropped: Vec<String> = Vec::new();
    for v in fm.sub_actions.as_deref().unwrap_or(&[]) {
        let verdict = verify_action_entry(v, is_usage, r).and_then(|()| {
            let (t, _) = action_entry(v, &mut counters, pad, r)?;
            Ok(t)
        });
        match verdict {
            Ok(t) => out.push_str(&t),
            Err(reason) => {
                dropped.extend(v.as_mapping().and_then(|m| text(m, "name")).map(str::to_string));
                out.push_str(&comment(pad, "subAction", &describe(v), &reason));
            }
        }
    }
    for v in fm.control_nodes.as_deref().unwrap_or(&[]) {
        let verdict = (|| -> Result<String, String> {
            let m = v.as_mapping().ok_or("not a mapping")?;
            if !only_keys(m, &["name", "kind", "parameters"]) {
                return Err("unsupported fields".into());
            }
            let kw = match text(m, "kind") {
                Some("ForkNode") => "fork",
                Some("JoinNode") => "join",
                Some("DecisionNode") => "decide",
                Some("MergeNode") => "merge",
                _ => return Err("unknown control node kind".into()),
            };
            let name = text(m, "name").ok_or("no name")?;
            // `REQ-TRS-SYSMLV2-090`: the node's pins as its body; a parameter with no `direction`
            // is an `in` one, which is what ingestion reads back.
            let (line, expected) = match get(m, "parameters") {
                None => (format!("{kw} {};\n", chain(name)), v.clone()),
                Some(Value::Sequence(ps)) => {
                    let mut s = format!("{kw} {} {{\n", chain(name));
                    let mut norm = Vec::new();
                    for p in ps {
                        let pm = p.as_mapping().ok_or("parameter is not a mapping")?;
                        if !only_keys(pm, &["name", "direction", "typedBy"]) {
                            return Err("unsupported parameter fields".into());
                        }
                        let pn = text(pm, "name").ok_or("parameter has no name")?;
                        let dir = match text(pm, "direction") {
                            Some("out") => "out",
                            Some("inout") => "inout",
                            None | Some("in") => "in",
                            Some(_) => return Err("unknown parameter direction".into()),
                        };
                        s.push_str(&format!("{pad}    {dir} {}", sysml_ident(pn)));
                        let mut nm = pm.clone();
                        if let Some(t) = text(pm, "typedBy") {
                            s.push_str(&format!(" : {}", r(t)));
                        }
                        s.push_str(";\n");
                        nm.insert(key("direction"), key(dir));
                        norm.push(Value::Mapping(nm));
                    }
                    s.push_str(&format!("{pad}}}\n"));
                    let mut em = m.clone();
                    em.insert(key("parameters"), Value::Sequence(norm));
                    (s, Value::Mapping(em))
                }
                Some(_) => return Err("`parameters` is not a list".into()),
            };
            let p = probe_action_body(is_usage, &line).ok_or("does not parse")?;
            if p.control_nodes == vec![expected] && p.sub_actions.is_empty() && p.successions.is_empty() {
                Ok(format!("{pad}{line}"))
            } else {
                Err("does not read back identically".into())
            }
        })();
        match verdict {
            Ok(t) => out.push_str(&t),
            Err(reason) => {
                dropped.extend(v.as_mapping().and_then(|m| text(m, "name")).map(str::to_string));
                out.push_str(&comment(pad, "controlNode", &describe(v), &reason));
            }
        }
    }
    for v in fm.succession_connections.as_deref().unwrap_or(&[]) {
        let verdict = (|| -> Result<String, String> {
            let m = v.as_mapping().ok_or("not a mapping")?;
            let line = succession_line(m, &dropped, r)?;
            let p = probe_action_body(is_usage, &line).ok_or("does not parse")?;
            if canon_list(&p.successions) == canon_list(&[v.clone()]) && p.sub_actions.is_empty() && p.control_nodes.is_empty() {
                Ok(format!("{pad}{line}"))
            } else {
                Err("does not read back identically".into())
            }
        })();
        match verdict {
            Ok(t) => out.push_str(&t),
            Err(reason) => out.push_str(&comment(pad, "successionConnection", &describe_succession(v), &reason)),
        }
    }
    out
}

/// One `successionConnections:` entry as a `first`/`succession` statement (no indentation).
/// `REQ-TRS-SYSMLV2-074`: a guarded succession is `first a if <guard> then b;`.
/// `REQ-TRS-SYSMLV2-081`: an own name and multiplicities are `succession n [m] first [x] a then [y] b;`.
/// `REQ-TRS-SYSMLV2-091`: an own type is `succession n : T first a then b;`.
fn succession_line(m: &Mapping, dropped: &[String], r: Refs) -> Result<String, String> {
    if !only_keys(m, &["name", "typedBy", "after", "before", "guard", "multiplicity", "afterMultiplicity", "beforeMultiplicity"]) {
        return Err("unsupported fields".into());
    }
    let (a, b) = (text(m, "after").ok_or("no after")?, text(m, "before").ok_or("no before")?);
    // REQ-TRS-SYSMLV2-062: never reference a step this body did not export.
    if let Some(gone) = [a, b].into_iter().find(|n| dropped.iter().any(|d| d == n)) {
        return Err(format!("endpoint '{gone}' was not exported"));
    }
    let name = text(m, "name").map(sysml_ident);
    let typed = text(m, "typedBy").map(|t| format!(": {} ", r(t)));
    let mult = |k: &str| scalar(m, k).map(|x| format!("[{x}] "));
    let (sm, am, bm) = (mult("multiplicity"), mult("afterMultiplicity"), mult("beforeMultiplicity"));
    let decl = |sm: Option<String>| format!("succession {}{}{}", name.clone().map(|n| format!("{n} ")).unwrap_or_default(), typed.clone().unwrap_or_default(), sm.unwrap_or_default());
    Ok(match scalar(m, "guard") {
        Some(_) if sm.is_some() || am.is_some() || bm.is_some() => return Err("guarded succession with multiplicities".into()),
        Some(g) => {
            let head = if name.is_some() || typed.is_some() { decl(None) } else { String::new() };
            format!("{head}first {} if {g} then {};\n", chain(a), chain(b))
        }
        None => {
            let head = if name.is_some() || typed.is_some() || sm.is_some() { decl(sm) } else { String::new() };
            format!("{head}first {}{} then {}{};\n", am.unwrap_or_default(), chain(a), bm.unwrap_or_default(), chain(b))
        }
    })
}

/// `REQ-TRS-SYSMLV2-092`: the `successionConnections:` of a `PartDef`/`Part` as structural
/// `first`/`succession` statements, each only when a `part def`/`part` body reads it back identically.
pub(super) fn part_successions(fm: &RawFrontmatter, is_usage: bool, pad: &str, r: Refs) -> String {
    let mut out = String::new();
    for v in fm.succession_connections.as_deref().unwrap_or(&[]) {
        let verdict = (|| -> Result<String, String> {
            let m = v.as_mapping().ok_or("not a mapping")?;
            let line = succession_line(m, &[], r)?;
            let p = probe_part_body(is_usage, &line).ok_or("does not parse")?;
            if canon_list(&p) == canon_list(&[v.clone()]) {
                Ok(format!("{pad}{line}"))
            } else {
                Err("does not read back identically".into())
            }
        })();
        match verdict {
            Ok(t) => out.push_str(&t),
            Err(reason) => out.push_str(&comment(pad, "successionConnection", &describe_succession(v), &reason)),
        }
    }
    out
}

/// Read-back check for one top-level entry, in isolation: it is first in its body, so a
/// synthesized name is `<kind>_1` on both sides. Nested bodies compare verbatim.
fn verify_action_entry(v: &Value, is_usage: bool, r: Refs) -> Result<(), String> {
    let (txt, kept) = action_entry(v, &mut Counters::default(), "", r)?;
    let probed = probe_action_body(is_usage, &txt).ok_or("does not parse")?;
    if canon_list(&probed.sub_actions) == canon_list(&[kept]) && probed.control_nodes.is_empty() && probed.successions.is_empty() {
        Ok(())
    } else {
        Err("does not read back identically".to_string())
    }
}

// ── States ─────────────────────────────────────────────────────────────────

/// `entry`/`do`/`exit action n;` lines for a state-like mapping or the element itself.
fn behaviour_lines(entry: Option<&Value>, doa: Option<&Value>, exit: Option<&Value>, pad: &str) -> Result<String, String> {
    let mut s = String::new();
    for (kw, v) in [("entry", entry), ("do", doa), ("exit", exit)] {
        let Some(v) = v else { continue };
        let n = v.as_str().filter(|n| !n.is_empty()).ok_or_else(|| format!("{kw}Action is not a plain action name"))?;
        s.push_str(&format!("{pad}{kw} action {};\n", super::export::qualified(n)));
    }
    Ok(s)
}

fn effect_text(v: &Value, r: Refs) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        Value::Mapping(m) => {
            if !only_keys(m, &["name", "typedBy"]) {
                return Err("unsupported effect fields".into());
            }
            let name = text(m, "name").ok_or("effect has no name")?;
            Ok(match text(m, "typedBy") {
                Some(t) => format!("action {} : {}", sysml_ident(name), r(t)),
                None => format!("action {}", sysml_ident(name)),
            })
        }
        _ => Err("unsupported effect".into()),
    }
}

fn accept_text(v: &Value, r: Refs) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        Value::Mapping(m) => {
            if !only_keys(m, &["payload", "via"]) {
                return Err("unsupported accept fields".into());
            }
            let p = text(m, "payload").ok_or("accept has no payload")?;
            Ok(match text(m, "via") {
                Some(via) => format!("{p} via {}", r(via)),
                None => p.to_string(),
            })
        }
        _ => Err("unsupported accept".into()),
    }
}

/// One `transitions:` entry as text. `explicit_source`: a top-level transition must name its
/// source; inside a substate the source is implicit and the shorthand form is used.
fn transition_text(v: &Value, nested: bool, pad: &str, r: Refs) -> Result<String, String> {
    let m = v.as_mapping().ok_or("not a mapping")?;
    if !only_keys(m, &["source", "target", "accept", "guard", "effect"]) {
        return Err("unsupported fields".into());
    }
    let target = text(m, "target").ok_or("no target")?;
    let mut s = String::new();
    match (text(m, "source"), nested) {
        (Some(src), _) => s.push_str(&format!("transition first {} ", chain(src))),
        (None, true) => {}
        (None, false) => return Err("top-level transition has no source".into()),
    }
    if let Some(a) = get(m, "accept") {
        s.push_str(&format!("accept {} ", accept_text(a, r)?));
    }
    if let Some(g) = scalar(m, "guard") {
        s.push_str(&format!("if {g} "));
    }
    if let Some(e) = get(m, "effect") {
        s.push_str(&format!("do {} ", effect_text(e, r)?));
    }
    s.push_str(&format!("then {};", chain(target)));
    Ok(format!("{pad}{s}\n"))
}

/// A substate entry: statement text (state + its sibling markers) and the value to expect.
fn substate(v: &Value, pad: &str, r: Refs) -> Result<(String, String, Value), String> {
    let m = v.as_mapping().ok_or("not a mapping")?;
    if !only_keys(
        m,
        &["name", "typedBy", "entryAction", "doAction", "exitAction", "subStates", "transitions", "isInitial", "isFinal"],
    ) {
        return Err("unsupported fields".into());
    }
    let name = text(m, "name").filter(|n| !n.is_empty()).ok_or("no name")?;
    let inner = format!("{pad}    ");
    let mut head = format!("{pad}state {}", sysml_ident(name));
    let mut kept = Mapping::new();
    kept.insert(key("name"), key(name));
    if let Some(t) = text(m, "typedBy") {
        head.push_str(&format!(" : {}", r(t)));
        kept.insert(key("typedBy"), key(t));
    }
    let mut body = behaviour_lines(get(m, "entryAction"), get(m, "doAction"), get(m, "exitAction"), &inner)?;
    for (k, kk) in [("entryAction", "entryAction"), ("doAction", "doAction"), ("exitAction", "exitAction")] {
        if let Some(a) = get(m, k) {
            kept.insert(key(kk), a.clone());
        }
    }
    let (subs, kept_subs) = match get(m, "subStates") {
        Some(Value::Sequence(seq)) => state_list(seq, &inner, r),
        None => (String::new(), Vec::new()),
        Some(_) => return Err("`subStates` is not a list".into()),
    };
    body.push_str(&subs);
    if !kept_subs.is_empty() {
        kept.insert(key("subStates"), Value::Sequence(kept_subs));
    }
    let mut kept_tr = Vec::new();
    match get(m, "transitions") {
        Some(Value::Sequence(seq)) => {
            for t in seq {
                match transition_text(t, true, &inner, r) {
                    Ok(txt) => {
                        body.push_str(&txt);
                        kept_tr.push(t.clone());
                    }
                    Err(reason) => body.push_str(&comment(&inner, "transition", &describe_transition(t), &reason)),
                }
            }
        }
        None => {}
        Some(_) => return Err("`transitions` is not a list".into()),
    }
    if !kept_tr.is_empty() {
        kept.insert(key("transitions"), Value::Sequence(kept_tr));
    }
    let mut markers = String::new();
    if get(m, "isInitial").and_then(Value::as_bool) == Some(true) {
        markers.push_str(&format!("{pad}then {};\n", chain(name)));
        kept.insert(key("isInitial"), Value::Bool(true));
    }
    if get(m, "isFinal").and_then(Value::as_bool) == Some(true) {
        markers.push_str(&format!("{pad}final {};\n", chain(name)));
        kept.insert(key("isFinal"), Value::Bool(true));
    }
    let stmt = if body.is_empty() { format!("{head};\n") } else { format!("{head} {{\n{body}{pad}}}\n") };
    Ok((stmt, markers, Value::Mapping(kept)))
}

fn describe_succession(v: &Value) -> String {
    match v.as_mapping() {
        Some(m) => format!("{} -> {}", text(m, "after").unwrap_or("?"), text(m, "before").unwrap_or("?")),
        None => "non-mapping entry".to_string(),
    }
}

fn describe_transition(v: &Value) -> String {
    match v.as_mapping() {
        Some(m) => format!("{} -> {}", text(m, "source").unwrap_or("(implicit)"), text(m, "target").unwrap_or("?")),
        None => "non-mapping entry".to_string(),
    }
}

/// A nested list of substates: statements, then all their sibling markers (kept entries only).
fn state_list(list: &[Value], pad: &str, r: Refs) -> (String, Vec<Value>) {
    let mut out = String::new();
    let mut markers = String::new();
    let mut kept = Vec::new();
    for v in list {
        match substate(v, pad, r) {
            Ok((stmt, mk, k)) => {
                out.push_str(&stmt);
                markers.push_str(&mk);
                kept.push(k);
            }
            Err(reason) => out.push_str(&comment(pad, "subState", &describe(v), &reason)),
        }
    }
    out.push_str(&markers);
    (out, kept)
}

/// `entryAction:`/`doAction:`/`exitAction:`/`subStates:`/`transitions:` of a `StateDef`/`State`.
pub(super) fn state_body(fm: &RawFrontmatter, pad: &str, r: Refs) -> String {
    let mut out = String::new();
    for (kw, v) in [("entry", &fm.entry_action), ("do", &fm.do_action), ("exit", &fm.exit_action)] {
        let Some(v) = v else { continue };
        let line = behaviour_lines(
            (kw == "entry").then_some(v),
            (kw == "do").then_some(v),
            (kw == "exit").then_some(v),
            "",
        );
        match line {
            Ok(l) => {
                let p = probe_state_body(&l);
                let got = p.as_ref().and_then(|p| match kw {
                    "entry" => p.entry.clone(),
                    "do" => p.do_action.clone(),
                    _ => p.exit.clone(),
                });
                if got.as_ref() == Some(v) {
                    out.push_str(&format!("{pad}{l}"));
                } else {
                    out.push_str(&comment(pad, &format!("{kw}Action"), &describe_scalar(v), "does not read back identically"));
                }
            }
            Err(reason) => out.push_str(&comment(pad, &format!("{kw}Action"), &describe_scalar(v), &reason)),
        }
    }
    // Substates: verify each top-level one (with its markers) in isolation.
    let mut markers = String::new();
    for v in fm.sub_states.as_deref().unwrap_or(&[]) {
        let verdict = verify_substate(v, r).and_then(|()| substate(v, pad, r).map(|(stmt, mk, _)| (stmt, mk)));
        match verdict {
            Ok((stmt, mk)) => {
                out.push_str(&stmt);
                markers.push_str(&mk);
            }
            Err(reason) => out.push_str(&comment(pad, "subState", &describe(v), &reason)),
        }
    }
    out.push_str(&markers);
    for t in fm.transitions.as_deref().unwrap_or(&[]) {
        let verdict = verify_transition(t, r).and_then(|()| transition_text(t, false, pad, r));
        match verdict {
            Ok(l) => out.push_str(&l),
            Err(reason) => out.push_str(&comment(pad, "transition", &describe_transition(t), &reason)),
        }
    }
    out
}

fn verify_substate(v: &Value, r: Refs) -> Result<(), String> {
    let (stmt, mk, kept) = substate(v, "", r)?;
    let p = probe_state_body(&format!("{stmt}{mk}")).ok_or("does not parse")?;
    if canon_list(&p.sub_states) == canon_list(&[kept]) && p.transitions.is_empty() && p.entry.is_none() && p.do_action.is_none() && p.exit.is_none() {
        Ok(())
    } else {
        Err("does not read back identically".to_string())
    }
}

fn verify_transition(t: &Value, r: Refs) -> Result<(), String> {
    let line = transition_text(t, false, "", r)?;
    let p = probe_state_body(&line).ok_or("does not parse")?;
    if canon_list(&p.transitions) == canon_list(&[t.clone()]) && p.sub_states.is_empty() {
        Ok(())
    } else {
        Err("does not read back identically".to_string())
    }
}

fn describe_scalar(v: &Value) -> String {
    v.as_str().map(str::to_string).unwrap_or_else(|| format!("{v:?}"))
}

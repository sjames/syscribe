//! The fault model of the safety analyses (REQ-TRS-FAULT-001): the `injection:` list of an `FMEAEntry` row and of a
//! `FaultTreeEvent`, parsed into typed [`FaultSpec`]s that the composite engine can inject.

use serde_yaml::{Mapping, Value as Y};

use crate::element::{ElementType, RawElement};

#[derive(Debug, Clone, PartialEq)]
pub enum FaultKind {
    /// The variable is held at the value of the expression.
    Stuck { variable: String, value: String },
    /// Values written to the variable are increased by the expression (same dimension).
    Offset { variable: String, by: String },
    /// The part fails silent: no event, no timer, no output.
    Silent,
    /// Messages through the port (optionally only this event) are lost.
    Drop { port: String, event: Option<String> },
    /// Messages through the port arrive `by` seconds later.
    Delay { port: String, event: Option<String>, by: f64 },
    /// `field` of the payload of messages through the port is replaced by the value of the expression.
    Corrupt { port: String, event: Option<String>, field: String, value: String },
    /// A plant parameter takes the value of the expression (GH #340): a changed grid impedance, a drifted gain.
    Parameter { parameter: String, value: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FaultSpec {
    /// An instance name of the composite; when absent every part whose definition is, or specialises, the element's `ref`.
    pub part: Option<String>,
    pub kind: FaultKind,
}

/// An element that carries a fault model, with its id.
#[derive(Debug, Clone)]
pub struct FaultDef {
    pub id: String,
    pub qualified_name: String,
    /// The element the failure belongs to (`ref:`), unresolved.
    pub target: Option<String>,
    pub specs: Vec<FaultSpec>,
}

fn field<'a>(m: &'a Mapping, k: &str) -> Option<&'a Y> {
    m.get(Y::String(k.to_string()))
}

fn text(v: &Y) -> Option<String> {
    match v {
        Y::String(s) => Some(s.clone()),
        Y::Number(n) => Some(n.to_string()),
        Y::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

const KEYS: &[&str] = &["kind", "part", "port", "event", "variable", "field", "value", "by", "parameter"];

fn expression(m: &Mapping, key: &str, at: &str, errs: &mut Vec<String>) -> Option<String> {
    match field(m, key).and_then(text) {
        Some(s) if crate::expr::parse(&s).is_ok() => Some(s),
        Some(s) => {
            errs.push(format!("{at}: `{key}: {s}` is not an expression"));
            None
        }
        None => {
            errs.push(format!("{at}: needs `{key}`"));
            None
        }
    }
}

fn name(m: &Mapping, key: &str, at: &str, errs: &mut Vec<String>) -> Option<String> {
    match field(m, key).and_then(text).filter(|s| !s.trim().is_empty()) {
        Some(s) => Some(s),
        None => {
            errs.push(format!("{at}: needs `{key}`"));
            None
        }
    }
}

/// Parse an `injection:` value. `Err` lists the problems (`E936`).
pub fn parse_injection(v: &Y) -> Result<Vec<FaultSpec>, Vec<String>> {
    let Y::Sequence(items) = v else { return Err(vec!["`injection:` must be a list of fault items".to_string()]) };
    let mut errs = Vec::new();
    let mut out = Vec::new();
    for (i, it) in items.iter().enumerate() {
        let at = format!("injection[{i}]");
        let Some(m) = it.as_mapping() else {
            errs.push(format!("{at}: must be a mapping with a `kind`"));
            continue;
        };
        for k in m.keys().filter_map(|k| k.as_str()) {
            if !KEYS.contains(&k) {
                errs.push(format!("{at}: unknown key `{k}` (kind, part, port, event, variable, field, value, by, parameter)"));
            }
        }
        let kind = field(m, "kind").and_then(|k| k.as_str()).unwrap_or("");
        let event = field(m, "event").and_then(text);
        let spec = match kind {
            "stuck" => {
                let (variable, value) = (name(m, "variable", &at, &mut errs), expression(m, "value", &at, &mut errs));
                variable.zip(value).map(|(variable, value)| FaultKind::Stuck { variable, value })
            }
            "offset" => {
                let (variable, by) = (name(m, "variable", &at, &mut errs), expression(m, "by", &at, &mut errs));
                variable.zip(by).map(|(variable, by)| FaultKind::Offset { variable, by })
            }
            "silent" => Some(FaultKind::Silent),
            "parameter" => {
                let (parameter, value) = (name(m, "parameter", &at, &mut errs), expression(m, "value", &at, &mut errs));
                parameter.zip(value).map(|(parameter, value)| FaultKind::Parameter { parameter, value })
            }
            "drop" => name(m, "port", &at, &mut errs).map(|port| FaultKind::Drop { port, event }),
            "delay" => {
                let port = name(m, "port", &at, &mut errs);
                let by = match field(m, "by").and_then(text) {
                    Some(s) => match crate::system::duration(&s) {
                        Some(d) => Some(d),
                        None => {
                            errs.push(format!("{at}: `by: {s}` is not a non-negative duration such as \"20 ms\""));
                            None
                        }
                    },
                    None => {
                        errs.push(format!("{at}: needs `by`"));
                        None
                    }
                };
                port.zip(by).map(|(port, by)| FaultKind::Delay { port, event, by })
            }
            "corrupt" => {
                let (port, f, value) = (name(m, "port", &at, &mut errs), name(m, "field", &at, &mut errs), expression(m, "value", &at, &mut errs));
                port.zip(f).zip(value).map(|((port, field), value)| FaultKind::Corrupt { port, event, field, value })
            }
            "" => {
                errs.push(format!("{at}: needs a `kind` (stuck, offset, silent, parameter, drop, delay, corrupt)"));
                None
            }
            other => {
                errs.push(format!("{at}: unknown kind `{other}` (stuck, offset, silent, parameter, drop, delay, corrupt)"));
                None
            }
        };
        if let Some(kind) = spec {
            out.push(FaultSpec { part: field(m, "part").and_then(text), kind });
        }
    }
    if errs.is_empty() {
        Ok(out)
    } else {
        Err(errs)
    }
}

/// Every `FMEAEntry` and `FaultTreeEvent` with a well-formed `injection:`.
pub fn fault_defs(elements: &[RawElement]) -> Vec<FaultDef> {
    elements
        .iter()
        .filter_map(|e| {
            let fm = &e.frontmatter;
            let (id, target) = match fm.element_type {
                Some(ElementType::FMEAEntry) => (fm.id.clone()?, fm.subject.clone()),
                Some(ElementType::FaultTreeEvent) => (fm.id.clone()?, fm.event_ref.clone()),
                _ => return None,
            };
            let specs = parse_injection(fm.injection.as_ref()?).ok()?;
            Some(FaultDef { id, qualified_name: e.qualified_name.clone(), target, specs })
        })
        .collect()
}

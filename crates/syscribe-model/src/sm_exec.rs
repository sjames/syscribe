//! Static checks of the executable state-machine schema (REQ-TRS-SMSCHEMA-001, GH #277):
//! `E882` triggers, `E883` variables, `E884` assignments, `E885` internal transitions,
//! `W933` guards that are not in the expression language. Pure over the frontmatter values,
//! so the validator, the IR lowering and the tests share one reading of the schema.

use std::collections::HashSet;

use crate::expr::{self, Value};
use serde_yaml::{Mapping, Value as Y};

/// One finding: `(code, message)`; the validator adds the file and severity.
pub type Finding = (&'static str, String);

fn field<'a>(m: &'a Mapping, k: &str) -> Option<&'a Y> {
    m.get(Y::String(k.to_string()))
}

/// Whether a transition makes the machine executable.
fn transition_is_executable(m: &Mapping) -> bool {
    field(m, "internal").is_some()
        || field(m, "assign").is_some()
        || matches!(field(m, "accept"), Some(Y::Mapping(a))
            if field(a, "payload").is_none() || ["after", "when", "done"].iter().any(|k| field(a, k).is_some()))
}

/// Every (state name, transition mapping) of the machine, descending into composite substates.
/// Top-level transitions carry the name from their own `source`/`from`.
fn transitions<'a>(subs: &'a [Y], top: &'a [Y], out: &mut Vec<(String, &'a Mapping)>) {
    for s in subs {
        let Some(sm) = s.as_mapping() else { continue };
        let name = field(sm, "name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if let Some(Y::Sequence(ts)) = field(sm, "transitions") {
            for t in ts.iter().filter_map(|t| t.as_mapping()) {
                out.push((name.clone(), t));
            }
        }
        if let Some(Y::Sequence(inner)) = field(sm, "subStates") {
            transitions(inner, &[], out);
        }
    }
    for t in top.iter().filter_map(|t| t.as_mapping()) {
        let src = field(t, "source").or_else(|| field(t, "from")).and_then(|v| v.as_str()).unwrap_or("");
        out.push((src.to_string(), t));
    }
}

fn state_names(subs: &[Y], prefix: &str, out: &mut HashSet<String>) {
    for s in subs {
        let Some(sm) = s.as_mapping() else { continue };
        if let Some(n) = field(sm, "name").and_then(|v| v.as_str()) {
            out.insert(n.to_string());
            let path = if prefix.is_empty() { n.to_string() } else { format!("{prefix}.{n}") };
            out.insert(path.clone());
            if let Some(Y::Sequence(inner)) = field(sm, "subStates") {
                state_names(inner, &path, out);
            }
        }
    }
}

fn is_basic_name(n: &str) -> bool {
    n.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn text(v: &Y) -> Option<String> {
    match v {
        Y::String(s) => Some(s.clone()),
        Y::Number(n) => Some(n.to_string()),
        Y::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Whether the machine declares or uses any executable feature.
pub fn is_executable(variables: Option<&[Y]>, initial_effect: Option<&Y>, subs: &[Y], top: &[Y]) -> bool {
    if variables.is_some() || initial_effect.is_some() {
        return true;
    }
    let mut ts = Vec::new();
    transitions(subs, top, &mut ts);
    ts.iter().any(|(_, t)| transition_is_executable(t))
}

/// Check one machine. `variables`, `initial_effect`, `subs` and `top` are the `StateDef`'s fields.
pub fn check(variables: Option<&[Y]>, initial_effect: Option<&Y>, subs: &[Y], top: &[Y]) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();
    if !is_executable(variables, initial_effect, subs, top) {
        return out;
    }

    // E883 — variables.
    let mut declared: HashSet<String> = HashSet::new();
    for (i, v) in variables.unwrap_or(&[]).iter().enumerate() {
        let Some(m) = v.as_mapping() else {
            out.push(("E883", format!("`variables:` entry {} is not a mapping {{name, value, typedBy?, unit?}}", i + 1)));
            continue;
        };
        let Some(name) = field(m, "name").and_then(|n| n.as_str()) else {
            out.push(("E883", format!("`variables:` entry {} has no `name:`", i + 1)));
            continue;
        };
        if !is_basic_name(name) {
            out.push(("E883", format!("variable name '{name}' is not a basic name (letters, digits, `_`; not starting with a digit)")));
        } else if !declared.insert(name.to_string()) {
            out.push(("E883", format!("variable '{name}' is declared twice")));
        }
        let unit = field(m, "unit").and_then(|u| u.as_str()).map(|u| u.trim_start_matches("SI::").to_string());
        if let Some(u) = &unit {
            if expr::unit_scale(u).is_none() {
                out.push(("E883", format!("variable '{name}': unknown unit '{u}'")));
            }
        }
        match field(m, "value").and_then(text) {
            None => out.push(("E883", format!("variable '{name}' has no initial `value:`"))),
            Some(src) => match expr::parse(&src).and_then(|e| e.eval(&|_: &str| None)) {
                Ok(Value::Num(q)) if unit.is_some() && q.dim != crate::units::Dim([0; 7]) => {
                    out.push(("E883", format!("variable '{name}': `unit:` given but the value `{src}` already has a unit")));
                }
                Ok(_) => {}
                Err(e) => out.push(("E883", format!("variable '{name}': initial value `{src}` does not evaluate: {e}"))),
            },
        }
    }

    let mut states = HashSet::new();
    state_names(subs, "", &mut states);

    let check_assign = |src: &str, where_: &str, out: &mut Vec<Finding>| match expr::parse_statements(src) {
        Err(e) => out.push(("E884", format!("{where_}: `assign:` {e}"))),
        Ok(stmts) => {
            for (n, _) in stmts {
                if !declared.contains(&n) {
                    out.push(("E884", format!("{where_}: `assign:` assigns '{n}', which is not a declared variable")));
                }
            }
        }
    };

    // initialEffect / entryAction / exitAction maps may carry `assign:`.
    if let Some(Y::Mapping(m)) = initial_effect {
        if let Some(a) = field(m, "assign").and_then(text) {
            check_assign(&a, "initialEffect", &mut out);
        }
    }

    // State action maps may carry `assign:` too.
    fn state_assigns<'a>(subs: &'a [Y], out: &mut Vec<(String, String)>) {
        for s in subs.iter().filter_map(|s| s.as_mapping()) {
            let name = field(s, "name").and_then(|v| v.as_str()).unwrap_or("");
            for k in ["entryAction", "doAction", "exitAction"] {
                if let Some(Y::Mapping(am)) = field(s, k) {
                    if let Some(a) = field(am, "assign").and_then(text) {
                        out.push((format!("state '{name}' {k}"), a));
                    }
                }
            }
            if let Some(Y::Sequence(inner)) = field(s, "subStates") {
                state_assigns(inner, out);
            }
        }
    }
    let mut sa = Vec::new();
    state_assigns(subs, &mut sa);
    for (w, a) in sa {
        check_assign(&a, &w, &mut out);
    }

    let mut ts = Vec::new();
    transitions(subs, top, &mut ts);
    for (src, t) in ts {
        let label = {
            let tgt = field(t, "target").or_else(|| field(t, "to")).and_then(|v| v.as_str()).unwrap_or("(internal)");
            format!("transition '{src}' → '{tgt}'")
        };
        let mut bound: HashSet<String> = HashSet::new(); // `as` bindings: <as>.<field> is a dotted path

        // E885 — internal.
        if let Some(i) = field(t, "internal") {
            if !matches!(i, Y::Bool(_)) {
                out.push(("E885", format!("{label}: `internal:` must be a boolean")));
            } else if *i == Y::Bool(true) && (field(t, "target").is_some() || field(t, "to").is_some()) {
                out.push(("E885", format!("{label}: an internal transition has no `target:`")));
            }
        }

        // E882 — trigger.
        let mut when_src: Option<String> = None;
        if let Some(Y::Mapping(a)) = field(t, "accept") {
            let mut kinds = 0;
            for (k, v) in a {
                let Some(k) = k.as_str() else { continue };
                match k {
                    "after" => {
                        kinds += 1;
                        let src = text(v).unwrap_or_default();
                        let want = expr::unit_scale("s").map(|(_, d)| d);
                        match expr::parse(&src).and_then(|e| e.eval(&|_: &str| None)) {
                            Ok(Value::Num(q)) if Some(q.dim) == want && q.v >= 0.0 => {}
                            Ok(_) => out.push(("E882", format!("{label}: `after: {src}` is not a non-negative duration"))),
                            Err(e) => out.push(("E882", format!("{label}: `after: {src}` does not evaluate: {e}"))),
                        }
                    }
                    "when" => {
                        kinds += 1;
                        let src = text(v).unwrap_or_default();
                        if let Err(e) = expr::parse(&src) {
                            out.push(("E882", format!("{label}: `when: {src}` does not parse: {e}")));
                        }
                        when_src = Some(src);
                    }
                    "done" => {
                        kinds += 1;
                        let n = text(v).unwrap_or_default();
                        if !states.contains(&n) {
                            out.push(("E882", format!("{label}: `done: {n}` names no region or state of the machine")));
                        }
                    }
                    "payload" | "via" | "as" => {}
                    other => out.push(("E882", format!("{label}: unknown trigger key `{other}` (use payload/via/as, after, when or done)"))),
                }
            }
            let payload = field(a, "payload").is_some();
            if kinds == 0 && !payload {
                out.push(("E882", format!("{label}: `accept:` has no trigger (payload, after, when or done)")));
            }
            if kinds > 1 || (kinds == 1 && payload) {
                out.push(("E882", format!("{label}: `accept:` has more than one trigger kind")));
            }
            if (field(a, "via").is_some() || field(a, "as").is_some()) && !payload {
                out.push(("E882", format!("{label}: `via`/`as` need a `payload`")));
            }
            if let Some(n) = field(a, "as").and_then(|v| v.as_str()) {
                bound.insert(n.to_string());
            }
        }

        if let Some(Y::Mapping(em)) = field(t, "effect") {
            if let Some(a) = field(em, "assign").and_then(text) {
                check_assign(&a, &label, &mut out);
            }
        }

        // E884 — assignments.
        if let Some(a) = field(t, "assign").and_then(text) {
            check_assign(&a, &label, &mut out);
        }

        // W933 — guard / when in the expression language, over known names.
        for (what, src) in [("guard", field(t, "guard").and_then(text)), ("when", when_src)] {
            let Some(src) = src.filter(|s| !s.trim().is_empty()) else { continue };
            match expr::parse(&src) {
                Err(_) => out.push(("W933", format!("{label}: `{what}: {src}` is not in the expression language, so the machine cannot be simulated"))),
                Ok(e) => {
                    for n in e.names() {
                        let path = n.contains('.') || n.contains("::");
                        if !path && !declared.contains(&n) {
                            out.push(("W933", format!("{label}: `{what}` uses '{n}', which is not a declared variable or an `as` binding")));
                        }
                    }
                }
            }
        }
    }
    out
}

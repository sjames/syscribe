//! Static checks of the executable action-flow schema (REQ-TRS-ACTSCHEMA-001, GH #276): `E883`/`E884`
//! (shared with state machines), `E887` wiring, `W935` expressions. Pure over the frontmatter values.

use std::collections::HashSet;

use serde_yaml::{Mapping, Value as Y};

use crate::expr::{self, Value};
use crate::sm_exec::{check_variables, field, text, Finding};

/// Every sub-action mapping reachable through `then`/`else`/`body`/`subActions`, in document order.
pub(crate) fn walk<'a>(list: &'a [Y], out: &mut Vec<&'a Mapping>) {
    for v in list {
        let Y::Mapping(m) = v else { continue };
        out.push(m);
        for key in ["then", "else", "body", "subActions"] {
            if let Some(Y::Sequence(inner)) = field(m, key) {
                walk(inner, out);
            }
        }
    }
}

fn kind_of(m: &Mapping) -> &str {
    field(m, "kind").and_then(|k| k.as_str()).unwrap_or("Action")
}

/// Whether the action declares or uses anything executable.
pub fn is_executable(variables: Option<&[Y]>, subs: &[Y]) -> bool {
    if variables.is_some() {
        return true;
    }
    let mut all = Vec::new();
    walk(subs, &mut all);
    all.iter().any(|m| field(m, "assign").is_some())
}

fn chain_root(s: &str) -> &str {
    s.split('.').next().unwrap_or(s)
}

pub fn check(params: &[Y], variables: Option<&[Y]>, subs: &[Y], controls: &[Y], successions: &[Y]) -> Vec<Finding> {
    check_with(false, params, variables, subs, controls, successions, &[], &[])
}

/// `check`, optionally treating the action as executable regardless of what it declares (a performed callee
/// is executed, so its expressions must be valid), and also checking its flow and binding ends.
#[allow(clippy::too_many_arguments)]
pub fn check_with(force: bool, params: &[Y], variables: Option<&[Y]>, subs: &[Y], controls: &[Y], successions: &[Y], flows: &[Y], bindings: &[Y]) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();
    if !force && !is_executable(variables, subs) {
        return out;
    }
    let (vf, declared) = check_variables(variables);
    out.extend(vf);
    let mut known: HashSet<String> = declared.clone();
    for p in params.iter().filter_map(|p| p.as_mapping()) {
        if let Some(n) = field(p, "name").and_then(|n| n.as_str()) {
            known.insert(n.to_string());
        }
    }
    let mut all = Vec::new();
    walk(subs, &mut all);
    let top_names: HashSet<&str> = subs.iter().chain(controls.iter()).filter_map(|v| v.as_mapping()).filter_map(|m| field(m, "name").and_then(|n| n.as_str())).collect();

    let names_ok = |src: &str, what: &str, out: &mut Vec<Finding>| {
        let Ok(e) = expr::parse(src) else { return };
        for n in e.names() {
            if n.contains('.') || n.contains("::") {
                continue;
            }
            if !known.contains(&n) {
                out.push(("W935", format!("{what}: `{src}` uses '{n}', which is not a parameter or a declared variable")));
            }
        }
    };

    // assignments, conditions, accepts, loops
    for m in &all {
        let name = field(m, "name").and_then(|n| n.as_str()).unwrap_or("?");
        let kind = kind_of(m);
        if let Some(a) = field(m, "assign").and_then(text) {
            match expr::parse_statements(&a) {
                Err(e) => out.push(("E884", format!("sub-action '{name}': `assign:` {e}"))),
                Ok(stmts) => {
                    for (target, rhs) in stmts {
                        // an assignment may write a variable or an `out`/`inout` parameter
                        if !known.contains(&target) {
                            out.push(("E884", format!("sub-action '{name}': `assign:` assigns '{target}', which is not a declared variable or parameter")));
                        }
                        names_ok(&rhs.to_source(), &format!("sub-action '{name}' assign"), &mut out);
                    }
                }
            }
        }
        if let Some(c) = field(m, "condition").and_then(text) {
            if expr::parse(&c).is_err() {
                out.push(("W935", format!("sub-action '{name}': `condition: {c}` is not in the expression language")));
            } else {
                names_ok(&c, &format!("sub-action '{name}' condition"), &mut out);
            }
        }
        if kind == "IfAction" && field(m, "condition").and_then(text).is_none() {
            out.push(("E887", format!("sub-action '{name}': an IfAction needs a `condition:`")));
        }
        if kind == "LoopAction" {
            let lk = field(m, "loopKind").and_then(|k| k.as_str()).unwrap_or("while");
            if lk == "for" {
                out.push(("E887", format!("sub-action '{name}': `loopKind: for` is not supported; use `while` or `until` with a `condition:`")));
            } else if field(m, "condition").and_then(text).is_none() {
                out.push(("E887", format!("sub-action '{name}': a {lk} loop needs a `condition:`")));
            }
        }
        if kind == "AcceptAction" {
            if let Some(a) = field(m, "after").and_then(text) {
                let want = expr::unit_scale("s").map(|x| x.1);
                match expr::parse(&a).and_then(|e| e.eval(&|_: &str| None)) {
                    Ok(Value::Num(q)) if Some(q.dim) == want && q.v >= 0.0 => {}
                    Ok(_) => out.push(("E887", format!("sub-action '{name}': `after: {a}` is not a non-negative duration"))),
                    Err(e) => out.push(("E887", format!("sub-action '{name}': `after: {a}` does not evaluate: {e}"))),
                }
            }
        }
    }

    // succession, flow and binding ends must name something
    for (i, sc) in successions.iter().filter_map(|x| x.as_mapping()).enumerate() {
        for key in ["after", "before"] {
            if let Some(v) = field(sc, key).and_then(|v| v.as_str()) {
                let root = chain_root(v);
                if root != "start" && root != "done" && !top_names.contains(root) {
                    out.push(("E887", format!("succession {i} names '{root}', which is not a sub-action or control node")));
                }
            }
        }
    }
    let end_ok = |x: &str| {
        let root = chain_root(x);
        top_names.contains(root) || known.contains(root)
    };
    for (i, f) in flows.iter().filter_map(|x| x.as_mapping()).enumerate() {
        for key in ["from", "to"] {
            if let Some(v) = field(f, key).and_then(|v| v.as_str()) {
                if !end_ok(v) {
                    out.push(("E887", format!("flowConnections[{i}] `{key}: {v}` is not a sub-action pin or a parameter/variable")));
                }
            }
        }
    }
    for (i, b) in bindings.iter().filter_map(|x| x.as_mapping()).enumerate() {
        for key in ["left", "right"] {
            if let Some(v) = field(b, key).and_then(|v| v.as_str()) {
                if !end_ok(v) {
                    out.push(("E887", format!("bindingConnections[{i}] `{key}: {v}` is not a sub-action pin or a parameter/variable")));
                }
            }
        }
    }

    // decision nodes and succession guards
    let decisions: HashSet<&str> = controls
        .iter()
        .filter_map(|c| c.as_mapping())
        .filter(|c| field(c, "kind").and_then(|k| k.as_str()) == Some("DecisionNode"))
        .filter_map(|c| field(c, "name").and_then(|n| n.as_str()))
        .collect();
    let succ: Vec<&Mapping> = successions.iter().filter_map(|s| s.as_mapping()).collect();
    for d in &decisions {
        let outgoing: Vec<&&Mapping> = succ.iter().filter(|s| field(s, "after").and_then(|a| a.as_str()).map(chain_root) == Some(*d)).collect();
        if outgoing.is_empty() {
            out.push(("E887", format!("decision '{d}' has no outgoing succession")));
            continue;
        }
        let is_else = |s: &&&Mapping| field(s, "guard").and_then(text).is_some_and(|g| g.trim().eq_ignore_ascii_case("else"));
        let elses = outgoing.iter().filter(|s| is_else(s)).count();
        if elses > 1 {
            out.push(("E887", format!("decision '{d}' has {elses} `else` successions")));
        } else if elses == 1 && !is_else(outgoing.last().unwrap()) {
            out.push(("E887", format!("decision '{d}': the `else` succession must be last")));
        }
        for s in &outgoing {
            if is_else(s) {
                continue;
            }
            match field(s, "guard").and_then(text) {
                Some(g) if expr::parse(&g).is_ok() => {}
                Some(g) => out.push(("E887", format!("decision '{d}': guard `{g}` is not in the expression language"))),
                None => {}
            }
        }
    }
    for s in &succ {
        let from = field(s, "after").and_then(|a| a.as_str()).map(chain_root).unwrap_or("");
        if decisions.contains(from) {
            if let Some(g) = field(s, "guard").and_then(text).filter(|g| !g.trim().eq_ignore_ascii_case("else")) {
                names_ok(&g, &format!("decision '{from}' guard"), &mut out);
            }
            continue;
        }
        if let Some(g) = field(s, "guard").and_then(text) {
            if expr::parse(&g).is_err() {
                out.push(("W935", format!("succession after '{from}': guard `{g}` is not in the expression language")));
            } else {
                names_ok(&g, &format!("succession after '{from}' guard"), &mut out);
            }
        }
    }
    out
}

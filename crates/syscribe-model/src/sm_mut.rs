//! Mutation analysis of state-machine scenarios (REQ-TRS-SMMUT-001, GH #282): seed one fault at a time
//! into the IR (never the model files) and re-run the scenarios; scenarios that cannot tell a mutant from the
//! original are not testing that part of the machine.

use serde::Serialize;

use crate::expr::{format_number, Expr, Value};
use crate::ir::{Action, IrExpr, Machine, NameRef, Origin, Trigger};

trait IntoTimer {
    fn into_timer(self) -> Option<f64>;
}

impl IntoTimer for Trigger {
    fn into_timer(self) -> Option<f64> {
        if let Trigger::Timer { delay_seconds } = self { Some(delay_seconds) } else { None }
    }
}
use crate::sm_run::{Coverage, Options};
use crate::sm_scen::{run_scenario, Scenario, Status};

#[derive(Debug, Clone, Copy)]
enum Elem {
    Transition(usize),
    State(usize),
}

pub struct Mutant {
    pub id: usize,
    pub operator: &'static str,
    pub description: String,
    pub origin: Origin,
    elem: Elem,
    pub machine: Machine,
}

pub(crate) fn rebuild(old: &IrExpr, ast: Expr) -> IrExpr {
    IrExpr { src: ast.to_source(), refs: old.refs.clone(), ast }
}

fn swap_rel(op: &str) -> Option<&'static str> {
    Some(match op {
        "<" => "<=",
        "<=" => "<",
        ">" => ">=",
        ">=" => ">",
        "==" => "!=",
        "!=" => "==",
        _ => return None,
    })
}

/// Every single-point relational-operator and literal mutation of an expression.
pub(crate) fn expr_mutations(e: &Expr) -> Vec<(&'static str, String, Expr)> {
    let mut out = Vec::new();
    match e {
        Expr::Bin(op, a, b) => {
            if let Some(new) = swap_rel(op) {
                out.push(("relational-operator", format!("`{op}` → `{new}` in `{}`", e.to_source()), Expr::Bin(new, a.clone(), b.clone())));
            }
            for (o, d, na) in expr_mutations(a) {
                out.push((o, d, Expr::Bin(op, Box::new(na), b.clone())));
            }
            for (o, d, nb) in expr_mutations(b) {
                out.push((o, d, Expr::Bin(op, a.clone(), Box::new(nb))));
            }
        }
        Expr::Unary(op, a) => {
            for (o, d, na) in expr_mutations(a) {
                out.push((o, d, Expr::Unary(op, Box::new(na))));
            }
        }
        Expr::If(c, a, b) => {
            for (o, d, n) in expr_mutations(c) {
                out.push((o, d, Expr::If(Box::new(n), a.clone(), b.clone())));
            }
            for (o, d, n) in expr_mutations(a) {
                out.push((o, d, Expr::If(c.clone(), Box::new(n), b.clone())));
            }
            for (o, d, n) in expr_mutations(b) {
                out.push((o, d, Expr::If(c.clone(), a.clone(), Box::new(n))));
            }
        }
        Expr::Call(f, args) => {
            for i in 0..args.len() {
                for (o, d, n) in expr_mutations(&args[i]) {
                    let mut na = args.clone();
                    na[i] = n;
                    out.push((o, d, Expr::Call(f.clone(), na)));
                }
            }
        }
        Expr::Lit(Value::Num(q)) => {
            let text = format_number(q.v);
            let decimals = text.split('.').nth(1).map(|d| d.trim_end_matches(|c: char| !c.is_ascii_digit()).len()).unwrap_or(0);
            let delta = 10f64.powi(-(decimals as i32));
            for sign in [1.0, -1.0] {
                let mut nq = *q;
                nq.v += sign * delta;
                out.push(("literal-shift", format!("literal {} → {}", format_number(q.v), format_number(nq.v)), Expr::Lit(Value::Num(nq))));
            }
        }
        _ => {}
    }
    out
}

/// Generate the mutants of a machine, deterministically, in a fixed operator order.
pub fn generate(m: &Machine) -> Vec<Mutant> {
    let mut out: Vec<Mutant> = Vec::new();
    let mut add = |operator: &'static str, description: String, origin: Origin, elem: Elem, machine: Machine| {
        let id = out.len();
        out.push(Mutant { id, operator, description, origin, elem, machine });
    };
    let describe = |t: usize| {
        let tr = &m.transitions[t];
        format!("t{t} {} → {}", m.states[tr.source].path, tr.target.map(|x| m.states[x].path.as_str()).unwrap_or("(internal)"))
    };
    for tr in &m.transitions {
        let t = tr.id;
        let origin = tr.origin.clone();
        let d = describe(t);
        // conditions: guard and `when`
        let mut conds: Vec<(bool, &IrExpr)> = Vec::new(); // (is_guard, expr)
        if let Some(g) = &tr.guard {
            conds.push((true, g));
        }
        if let Trigger::Change { cond } = &tr.trigger {
            conds.push((false, cond));
        }
        for (is_guard, e) in conds {
            let what = if is_guard { "guard" } else { "when" };
            let set = |mm: &mut Machine, n: IrExpr| {
                if is_guard {
                    mm.transitions[t].guard = Some(n);
                } else {
                    mm.transitions[t].trigger = Trigger::Change { cond: n };
                }
            };
            let mut mm = m.clone();
            set(&mut mm, rebuild(e, Expr::Unary("not", Box::new(e.ast.clone()))));
            add("negate-condition", format!("negate the {what} `{}` of {d}", e.src), origin.clone(), Elem::Transition(t), mm);
            for (op, desc, ast) in expr_mutations(&e.ast) {
                let mut mm = m.clone();
                set(&mut mm, rebuild(e, ast));
                add(op, format!("{desc} ({what} of {d})"), origin.clone(), Elem::Transition(t), mm);
            }
        }
        // retarget within the target's region
        if let (false, Some(target)) = (tr.internal, tr.target) {
            for alt in m.states.iter().filter(|s| s.parent == m.states[target].parent && s.id != target) {
                let mut mm = m.clone();
                mm.transitions[t].target = Some(alt.id);
                add("retarget", format!("retarget {d} to {}", alt.path), origin.clone(), Elem::Transition(t), mm);
            }
        }
        // remove the transition
        {
            let mut mm = m.clone();
            mm.transitions[t].trigger = Trigger::Event { name: "\u{0}removed".into(), bind: None };
            mm.transitions[t].guard = None;
            add("remove-transition", format!("remove {d}"), origin.clone(), Elem::Transition(t), mm);
        }
        // actions
        for (i, a) in tr.actions.iter().enumerate() {
            let mut mm = m.clone();
            mm.transitions[t].actions.remove(i);
            add("remove-action", format!("remove action {} of {d}", action_text(m, a)), origin.clone(), Elem::Transition(t), mm);
            if let Action::Assign { var, expr } = a {
                let mut mm = m.clone();
                mm.transitions[t].actions[i] = self_assign(m, *var, expr);
                add("assign-self", format!("assign {} to itself instead of `{}` ({d})", m.variables[*var].name, expr.src), origin.clone(), Elem::Transition(t), mm);
            }
        }
        // timers
        if let Trigger::Timer { delay_seconds } = tr.trigger.clone().into_timer().filter(|d| *d != 0.0).map(|d| Trigger::Timer { delay_seconds: d }).unwrap_or(Trigger::Eventless) {
            for (f, label) in [(0.5, "0.5"), (2.0, "2")] {
                let mut mm = m.clone();
                mm.transitions[t].trigger = Trigger::Timer { delay_seconds: delay_seconds * f };
                add("scale-after", format!("scale the delay of {d} by {label}"), origin.clone(), Elem::Transition(t), mm);
            }
        }
    }
    // swap the order of two transitions of one source with the same trigger
    for i in 0..m.transitions.len() {
        for j in i + 1..m.transitions.len() {
            let (a, b) = (&m.transitions[i], &m.transitions[j]);
            let same = a.source == b.source
                && match (&a.trigger, &b.trigger) {
                    (Trigger::Event { name: x, .. }, Trigger::Event { name: y, .. }) => x == y,
                    (Trigger::Eventless, Trigger::Eventless) => true,
                    _ => false,
                };
            if same {
                let mut mm = m.clone();
                mm.transitions.swap(i, j);
                for (k, t) in mm.transitions.iter_mut().enumerate() {
                    t.id = k;
                }
                add("swap-order", format!("swap the order of t{i} and t{j} of {}", m.states[a.source].path), a.origin.clone(), Elem::Transition(i), mm);
            }
        }
    }
    // state entry / exit actions
    for s in &m.states {
        for (which, list) in [("entry", &s.entry), ("exit", &s.exit)] {
            for (i, a) in list.iter().enumerate() {
                let mut mm = m.clone();
                if which == "entry" {
                    mm.states[s.id].entry.remove(i);
                } else {
                    mm.states[s.id].exit.remove(i);
                }
                add("remove-action", format!("remove {which} action {} of {}", action_text(m, a), s.path), s.origin.clone(), Elem::State(s.id), mm);
                if let Action::Assign { var, expr } = a {
                    let mut mm = m.clone();
                    let n = self_assign(m, *var, expr);
                    if which == "entry" {
                        mm.states[s.id].entry[i] = n;
                    } else {
                        mm.states[s.id].exit[i] = n;
                    }
                    add("assign-self", format!("assign {} to itself instead of `{}` ({which} of {})", m.variables[*var].name, expr.src, s.path), s.origin.clone(), Elem::State(s.id), mm);
                }
            }
        }
    }
    out
}

fn action_text(m: &Machine, a: &Action) -> String {
    match a {
        Action::Assign { var, expr } => format!("`{} := {}`", m.variables[*var].name, expr.src),
        Action::Behavior { target, .. } => format!("`{target}`"),
    }
}

fn self_assign(m: &Machine, var: usize, _old: &IrExpr) -> Action {
    let name = m.variables[var].name.clone();
    Action::Assign { var, expr: IrExpr { src: name.clone(), refs: vec![NameRef { name: name.clone(), kind: "var", slot: Some(var) }], ast: Expr::Name(name) } }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutantResult {
    pub id: usize,
    pub operator: &'static str,
    pub description: String,
    pub origin: Origin,
    pub fate: &'static str,
    pub killed_by: Option<String>,
    pub closest: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Uncovered {
    pub states: Vec<String>,
    pub transitions: Vec<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutReport {
    pub machine: String,
    pub machine_hash: String,
    pub scenarios: usize,
    pub score: f64,
    pub killed: usize,
    pub survived: usize,
    pub equivalent_suspect: usize,
    pub mutants: Vec<MutantResult>,
    pub uncovered: Uncovered,
}

fn trace_distance(a: &[crate::sm_run::Step], b: &[crate::sm_run::Step]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x != y).count() + a.len().abs_diff(b.len())
}

/// Run the mutation analysis. `Err` when a scenario does not pass on the unmutated machine.
pub fn analyse(m: &Machine, scenarios: &[Scenario], opts: &Options) -> Result<MutReport, String> {
    let base: Vec<_> = scenarios.iter().map(|s| run_scenario(m, s, opts)).collect();
    if let Some(v) = base.iter().find(|v| v.status != Status::Pass) {
        return Err(format!("scenario '{}' does not pass on the unmutated machine: {}", v.scenario, v.message.clone().unwrap_or_default()));
    }
    let mut cov = Coverage::default();
    base.iter().for_each(|v| cov.merge(&v.coverage));
    let covered = |e: Elem| match e {
        Elem::Transition(t) => cov.transitions.contains(&t) || cov.conds.contains_key(&(t, 0)) || cov.conds.contains_key(&(t, 1)),
        Elem::State(s) => cov.states.contains(&s),
    };
    let (mut killed, mut survived, mut equiv) = (0, 0, 0);
    let mut results = Vec::new();
    for mu in generate(m) {
        let mut killer = None;
        let mut closest: Option<(usize, String)> = None;
        for (i, sc) in scenarios.iter().enumerate() {
            let v = run_scenario(&mu.machine, sc, opts);
            if v.status != Status::Pass {
                killer = Some(sc.title.clone());
                break;
            }
            let d = trace_distance(&v.trace, &base[i].trace);
            if d > 0 && closest.as_ref().is_none_or(|c| d > c.0) {
                closest = Some((d, sc.title.clone()));
            }
        }
        let (fate, killed_by, closest) = match killer {
            Some(k) => {
                killed += 1;
                ("killed", Some(k), None)
            }
            None if closest.is_none() && !covered(mu.elem) => {
                equiv += 1;
                ("equivalentSuspect", None, None)
            }
            None => {
                survived += 1;
                ("survived", None, closest.map(|c| c.1))
            }
        };
        results.push(MutantResult { id: mu.id, operator: mu.operator, description: mu.description, origin: mu.origin, fate, killed_by, closest });
    }
    let score = if killed + survived == 0 { 100.0 } else { 100.0 * killed as f64 / (killed + survived) as f64 };
    Ok(MutReport {
        machine: m.machine.clone(),
        machine_hash: m.hash.clone(),
        scenarios: scenarios.len(),
        score,
        killed,
        survived,
        equivalent_suspect: equiv,
        mutants: results,
        uncovered: Uncovered {
            states: m.states.iter().filter(|s| !cov.states.contains(&s.id)).map(|s| s.path.clone()).collect(),
            transitions: m.transitions.iter().filter(|t| !cov.transitions.contains(&t.id)).map(|t| t.id).collect(),
        },
    })
}

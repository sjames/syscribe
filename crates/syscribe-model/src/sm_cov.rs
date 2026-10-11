//! Coverage report of state-machine scenarios (REQ-TRS-SMCOV-001, GH #281): states, transitions,
//! transition pairs, guard / `when` conditions with an MC/DC demonstration, timers and requirements,
//! computed from the [`Coverage`] the interpreter recorded and the structure of the [`Machine`].

use serde::Serialize;

use crate::ir::{Machine, Trigger};
use crate::sm_run::Coverage;

#[derive(Debug, Serialize)]
pub struct Ratio {
    pub covered: usize,
    pub total: usize,
}

impl Ratio {
    pub fn percent(&self) -> f64 {
        if self.total == 0 { 100.0 } else { 100.0 * self.covered as f64 / self.total as f64 }
    }
}

#[derive(Debug, Serialize)]
pub struct StateCov {
    pub covered: usize,
    pub total: usize,
    pub uncovered: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Uncovered {
    pub id: usize,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct TransCov {
    pub covered: usize,
    pub total: usize,
    pub uncovered: Vec<Uncovered>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Atom {
    pub text: String,
    pub seen_true: bool,
    pub seen_false: bool,
}

#[derive(Debug, Serialize)]
pub struct Mcdc {
    pub atom: usize,
    pub independent: bool,
}

#[derive(Debug, Serialize)]
pub struct ExprCov {
    pub transition: usize,
    pub kind: &'static str,
    pub expr: String,
    pub atoms: Vec<Atom>,
    pub mcdc: Vec<Mcdc>,
}

#[derive(Debug, Serialize)]
pub struct GuardCov {
    pub covered: usize,
    pub total: usize,
    pub expressions: Vec<ExprCov>,
}

#[derive(Debug, Serialize)]
pub struct TimerCov {
    pub covered: usize,
    pub total: usize,
    pub uncovered: Vec<Uncovered>,
}

#[derive(Debug, Serialize)]
pub struct ReqCov {
    pub covered: Vec<String>,
    pub uncovered: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Basis {
    pub machine: String,
    pub machine_hash: String,
    pub scenarios: usize,
    pub configuration: Option<String>,
    pub tool_version: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub basis: Basis,
    pub states: StateCov,
    pub transitions: TransCov,
    pub pairs: Ratio,
    pub guards: GuardCov,
    pub timers: TimerCov,
    pub requirements: ReqCov,
}

fn describe(m: &Machine, t: usize) -> String {
    let tr = &m.transitions[t];
    let to = tr.target.map(|x| m.states[x].path.as_str()).unwrap_or("(internal)");
    let trig = match &tr.trigger {
        Trigger::Eventless => "eventless".to_string(),
        Trigger::Event { name, .. } => format!("on {name}"),
        Trigger::Timer { delay_seconds } => format!("after {} s", crate::expr::format_number(*delay_seconds)),
        Trigger::Change { cond } => format!("when {}", cond.src),
        Trigger::Done { state } => format!("done {}", m.states[*state].path),
    };
    let guard = tr.guard.as_ref().map(|g| format!(" [{}]", g.src)).unwrap_or_default();
    format!("{} → {} {trig}{guard}", m.states[tr.source].path, to)
}

/// Build the report. `requirements` are `(requirement id, verified)` for the requirements the scenario
/// `TestCase`s verify, `verified` being that every scenario of one verifying TestCase passed.
pub fn report(m: &Machine, cov: &Coverage, scenarios: usize, configuration: Option<String>, requirements: &[(String, bool)]) -> Report {
    let states_uncovered: Vec<String> = m.states.iter().filter(|s| !cov.states.contains(&s.id)).map(|s| s.path.clone()).collect();
    let trans_uncovered: Vec<Uncovered> = m
        .transitions
        .iter()
        .filter(|t| !cov.transitions.contains(&t.id))
        .map(|t| Uncovered { id: t.id, description: describe(m, t.id) })
        .collect();
    let possible_pairs = m
        .transitions
        .iter()
        .flat_map(|a| m.transitions.iter().filter(move |b| a.target == Some(b.source)).map(move |b| (a.id, b.id)))
        .count();

    let mut exprs = Vec::new();
    let (mut gc, mut gt) = (0, 0);
    for tr in &m.transitions {
        let mut items: Vec<(u8, &crate::ir::IrExpr)> = Vec::new();
        if let Some(g) = &tr.guard {
            items.push((0, g));
        }
        if let Trigger::Change { cond } = &tr.trigger {
            items.push((1, cond));
        }
        for (kind, e) in items {
            let atoms = e.ast.atoms();
            let seen = cov.conds.get(&(tr.id, kind));
            let vecs: Vec<&(Vec<Option<bool>>, bool)> = seen.map(|s| s.iter().collect()).unwrap_or_default();
            let atom_cov: Vec<Atom> = atoms
                .iter()
                .enumerate()
                .map(|(i, a)| Atom {
                    text: a.to_source(),
                    seen_true: vecs.iter().any(|(v, _)| v.get(i) == Some(&Some(true))),
                    seen_false: vecs.iter().any(|(v, _)| v.get(i) == Some(&Some(false))),
                })
                .collect();
            for a in &atom_cov {
                gt += 2;
                gc += a.seen_true as usize + a.seen_false as usize;
            }
            // Unique-cause MC/DC: two evaluations that differ only in atom i and decide differently.
            let mcdc = if atoms.len() > 1 {
                (0..atoms.len())
                    .map(|i| Mcdc {
                        atom: i,
                        independent: vecs.iter().any(|(v1, o1)| {
                            vecs.iter().any(|(v2, o2)| o1 != o2 && v1.len() == v2.len() && v1[i].is_some() && v2[i].is_some() && (0..v1.len()).all(|j| if j == i { v1[j] != v2[j] } else { v1[j].is_some() && v1[j] == v2[j] }))
                        }),
                    })
                    .collect()
            } else {
                vec![]
            };
            exprs.push(ExprCov { transition: tr.id, kind: if kind == 0 { "guard" } else { "when" }, expr: e.src.clone(), atoms: atom_cov, mcdc });
        }
    }

    let timers: Vec<usize> = m.transitions.iter().filter(|t| matches!(t.trigger, Trigger::Timer { .. })).map(|t| t.id).collect();
    let timers_uncovered: Vec<Uncovered> = timers
        .iter()
        .filter(|t| !(cov.timers_fired.contains(t) && cov.timers_cancelled.contains(t)))
        .map(|t| Uncovered {
            id: *t,
            description: format!(
                "{} ({}{})",
                describe(m, *t),
                if cov.timers_fired.contains(t) { "" } else { "never fired" },
                if cov.timers_cancelled.contains(t) { "" } else if cov.timers_fired.contains(t) { "never cancelled" } else { ", never cancelled" }
            ),
        })
        .collect();

    Report {
        basis: Basis { machine: m.machine.clone(), machine_hash: m.hash.clone(), scenarios, configuration, tool_version: env!("CARGO_PKG_VERSION") },
        states: StateCov { covered: m.states.len() - states_uncovered.len(), total: m.states.len(), uncovered: states_uncovered },
        transitions: TransCov { covered: m.transitions.len() - trans_uncovered.len(), total: m.transitions.len(), uncovered: trans_uncovered },
        pairs: Ratio { covered: cov.pairs.len(), total: possible_pairs },
        guards: GuardCov { covered: gc, total: gt, expressions: exprs },
        timers: TimerCov { covered: timers.len() - timers_uncovered.len(), total: timers.len(), uncovered: timers_uncovered },
        requirements: ReqCov {
            covered: requirements.iter().filter(|r| r.1).map(|r| r.0.clone()).collect(),
            uncovered: requirements.iter().filter(|r| !r.1).map(|r| r.0.clone()).collect(),
        },
    }
}

impl Report {
    pub fn states_percent(&self) -> f64 {
        Ratio { covered: self.states.covered, total: self.states.total }.percent()
    }
    pub fn transitions_percent(&self) -> f64 {
        Ratio { covered: self.transitions.covered, total: self.transitions.total }.percent()
    }
    pub fn guards_percent(&self) -> f64 {
        Ratio { covered: self.guards.covered, total: self.guards.total }.percent()
    }
}

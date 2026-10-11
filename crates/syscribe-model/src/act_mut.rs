//! Mutation analysis of action-flow scenarios (REQ-TRS-ACTEXEC-001, GH #276): one fault at a time in the
//! action-graph IR, scored like the state-machine mutants (`sm_mut`).

use crate::act_ir::{ActionProgram, Edge, NodeKind};
use crate::act_run::{ActCoverage, ActOptions};
use crate::expr::Expr;
use crate::ir::Origin;
use crate::sm_mut::{expr_mutations, rebuild, MutReport, MutantResult, Uncovered};
use crate::sm_scen::{Scenario, Status};

#[derive(Clone, Copy)]
enum Elem {
    Node(usize),
    Edge(usize),
}

struct Mutant {
    id: usize,
    operator: &'static str,
    description: String,
    origin: Origin,
    elem: Elem,
    prog: ActionProgram,
}

fn recompute_start(p: &mut ActionProgram) {
    let d = &mut p.defs[0];
    let incoming: std::collections::HashSet<usize> = d.edges.iter().map(|e| e.to).collect();
    let keep: std::collections::HashSet<usize> = d.start.iter().copied().collect(); // `after: start` stays
    d.start = d.nodes.iter().map(|n| n.id).filter(|n| !incoming.contains(n) || keep.contains(n)).collect();
}

fn generate(p: &ActionProgram) -> Vec<Mutant> {
    let d = &p.defs[0];
    let mut out: Vec<Mutant> = Vec::new();
    let mut add = |operator: &'static str, description: String, origin: Origin, elem: Elem, prog: ActionProgram| {
        let id = out.len();
        out.push(Mutant { id, operator, description, origin, elem, prog });
    };
    let name = |i: usize| d.nodes[i].stmt.name.clone();
    let edge_text = |e: &Edge| format!("the succession {} → {}", name(e.from), name(e.to));

    for e in &d.edges {
        // conditions on successions
        if let Some(g) = &e.guard {
            let mut m = p.clone();
            m.defs[0].edges[e.id].guard = Some(rebuild(g, Expr::Unary("not", Box::new(g.ast.clone()))));
            add("negate-condition", format!("negate the guard `{}` of {}", g.src, edge_text(e)), e.origin.clone(), Elem::Edge(e.id), m);
            for (op, desc, ast) in expr_mutations(&g.ast) {
                let mut m = p.clone();
                m.defs[0].edges[e.id].guard = Some(rebuild(g, ast));
                add(op, format!("{desc} (guard of {})", edge_text(e)), e.origin.clone(), Elem::Edge(e.id), m);
            }
        }
        // remove the succession
        {
            let mut m = p.clone();
            m.defs[0].edges.remove(e.id);
            for (k, x) in m.defs[0].edges.iter_mut().enumerate() {
                x.id = k;
            }
            recompute_start(&mut m);
            add("remove-succession", format!("remove {}", edge_text(e)), e.origin.clone(), Elem::Edge(e.id), m);
        }
        // retarget the succession
        for alt in d.nodes.iter().filter(|n| n.id != e.to && n.id != e.from) {
            let mut m = p.clone();
            m.defs[0].edges[e.id].to = alt.id;
            recompute_start(&mut m);
            add("retarget", format!("retarget {} to {}", edge_text(e), alt.stmt.name), e.origin.clone(), Elem::Edge(e.id), m);
        }
    }
    // swap the targets of two branches of one decision
    for n in d.nodes.iter().filter(|n| matches!(n.stmt.kind, NodeKind::Decision)) {
        let outs: Vec<&Edge> = d.edges.iter().filter(|e| e.from == n.id).collect();
        for (i, a) in outs.iter().enumerate() {
            for b in &outs[i + 1..] {
                if a.to == b.to {
                    continue; // swapping equal targets changes nothing
                }
                let mut m = p.clone();
                m.defs[0].edges[a.id].to = b.to;
                m.defs[0].edges[b.id].to = a.to;
                add("swap-branches", format!("swap the targets of two branches of decision '{}'", n.stmt.name), a.origin.clone(), Elem::Edge(a.id), m);
            }
        }
    }
    // node behaviour
    for n in &d.nodes {
        match &n.stmt.kind {
            NodeKind::Assign { assigns } => {
                for (i, a) in assigns.iter().enumerate() {
                    let mut m = p.clone();
                    if let NodeKind::Assign { assigns } = &mut m.defs[0].nodes[n.id].stmt.kind {
                        assigns.remove(i);
                    }
                    add("remove-assignment", format!("remove `{} := {}` of '{}'", d.slots[a.slot].name, a.expr.src, n.stmt.name), n.stmt.origin.clone(), Elem::Node(n.id), m);
                    let mut m = p.clone();
                    if let NodeKind::Assign { assigns } = &mut m.defs[0].nodes[n.id].stmt.kind {
                        let nm = d.slots[a.slot].name.clone();
                        assigns[i].expr = crate::ir::IrExpr { src: nm.clone(), refs: vec![crate::ir::NameRef { name: nm.clone(), kind: "var", slot: Some(a.slot) }], ast: Expr::Name(nm) };
                    }
                    add("assign-self", format!("assign {} to itself instead of `{}` in '{}'", d.slots[a.slot].name, a.expr.src, n.stmt.name), n.stmt.origin.clone(), Elem::Node(n.id), m);
                    for (op, desc, ast) in expr_mutations(&a.expr.ast) {
                        let mut m = p.clone();
                        if let NodeKind::Assign { assigns } = &mut m.defs[0].nodes[n.id].stmt.kind {
                            assigns[i].expr = rebuild(&a.expr, ast);
                        }
                        add(op, format!("{desc} (assignment to {} in '{}')", d.slots[a.slot].name, n.stmt.name), n.stmt.origin.clone(), Elem::Node(n.id), m);
                    }
                }
            }
            NodeKind::If { cond, .. } | NodeKind::Loop { cond, .. } => {
                let mut m = p.clone();
                let negated = rebuild(cond, Expr::Unary("not", Box::new(cond.ast.clone())));
                match &mut m.defs[0].nodes[n.id].stmt.kind {
                    NodeKind::If { cond, .. } | NodeKind::Loop { cond, .. } => *cond = negated,
                    _ => {}
                }
                add("negate-condition", format!("negate the condition `{}` of '{}'", cond.src, n.stmt.name), n.stmt.origin.clone(), Elem::Node(n.id), m);
                for (op, desc, ast) in expr_mutations(&cond.ast) {
                    let mut m = p.clone();
                    let new = rebuild(cond, ast);
                    match &mut m.defs[0].nodes[n.id].stmt.kind {
                        NodeKind::If { cond, .. } | NodeKind::Loop { cond, .. } => *cond = new,
                        _ => {}
                    }
                    add(op, format!("{desc} (condition of '{}')", n.stmt.name), n.stmt.origin.clone(), Elem::Node(n.id), m);
                }
            }
            _ => {}
        }
        if matches!(n.stmt.kind, NodeKind::Assign { .. } | NodeKind::Perform { .. } | NodeKind::Send { .. } | NodeKind::If { .. } | NodeKind::Loop { .. }) {
            let mut m = p.clone();
            m.defs[0].nodes[n.id].stmt.kind = NodeKind::Action;
            add("remove-action", format!("make '{}' complete without effect", n.stmt.name), n.stmt.origin.clone(), Elem::Node(n.id), m);
        }
    }
    out
}

fn trace_distance(a: &[crate::sm_run::Step], b: &[crate::sm_run::Step]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x != y).count() + a.len().abs_diff(b.len())
}

pub fn analyse(p: &ActionProgram, scenarios: &[Scenario], opts: &ActOptions) -> Result<MutReport, String> {
    use crate::act_scen::run_action_scenario;
    let base: Vec<_> = scenarios.iter().map(|s| run_action_scenario(p, s, opts)).collect();
    if let Some(v) = base.iter().find(|v| v.status != Status::Pass) {
        return Err(format!("scenario '{}' does not pass on the unmutated action: {}", v.scenario, v.message.clone().unwrap_or_default()));
    }
    let mut cov = ActCoverage::default();
    base.iter().for_each(|v| cov.merge(&v.act_coverage));
    let covered = |e: Elem| match e {
        Elem::Node(n) => cov.nodes.contains(&n) || cov.conds.contains_key(&(p.defs[0].nodes[n].stmt.uid, 1)),
        Elem::Edge(e) => cov.edges.contains(&e) || cov.conds.contains_key(&(e, 0)),
    };
    let (mut killed, mut survived, mut equiv) = (0, 0, 0);
    let mut results = Vec::new();
    for mu in generate(p) {
        let mut killer = None;
        let mut closest: Option<(usize, String)> = None;
        for (i, sc) in scenarios.iter().enumerate() {
            let v = run_action_scenario(&mu.prog, sc, opts);
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
    let d = &p.defs[0];
    Ok(MutReport {
        machine: p.root.clone(),
        machine_hash: p.hash.clone(),
        scenarios: scenarios.len(),
        score,
        killed,
        survived,
        equivalent_suspect: equiv,
        mutants: results,
        uncovered: Uncovered {
            states: d.nodes.iter().filter(|n| !cov.nodes.contains(&n.id)).map(|n| n.stmt.name.clone()).collect(),
            transitions: d.edges.iter().filter(|e| !cov.edges.contains(&e.id)).map(|e| e.id).collect(),
        },
    })
}

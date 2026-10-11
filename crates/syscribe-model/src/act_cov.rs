//! Coverage report of action-flow scenarios (REQ-TRS-ACTEXEC-001, GH #276): nodes fired, edges taken,
//! decision branches, loops run zero and at least once, and guard / condition coverage with MC/DC.

use serde::Serialize;

use crate::act_ir::{ActionProgram, NodeKind};
use crate::act_run::ActCoverage;
use crate::sm_cov::{expr_coverage, Basis, Atom, Mcdc, ReqCov, Uncovered};

#[derive(Debug, Serialize)]
pub struct NameCov {
    pub covered: usize,
    pub total: usize,
    pub uncovered: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ItemCov {
    pub covered: usize,
    pub total: usize,
    pub uncovered: Vec<Uncovered>,
}

#[derive(Debug, Serialize)]
pub struct CondCov {
    pub node_or_edge: usize,
    pub kind: &'static str,
    pub expr: String,
    pub atoms: Vec<Atom>,
    pub mcdc: Vec<Mcdc>,
}

#[derive(Debug, Serialize)]
pub struct GuardCov {
    pub covered: usize,
    pub total: usize,
    pub expressions: Vec<CondCov>,
}

#[derive(Debug, Serialize)]
pub struct ActReport {
    pub basis: Basis,
    pub nodes: NameCov,
    pub edges: ItemCov,
    pub decisions: ItemCov,
    pub loops: NameCov,
    pub guards: GuardCov,
    pub requirements: ReqCov,
}

fn pct(c: usize, t: usize) -> f64 {
    if t == 0 { 100.0 } else { 100.0 * c as f64 / t as f64 }
}

impl ActReport {
    pub fn nodes_percent(&self) -> f64 {
        pct(self.nodes.covered, self.nodes.total)
    }
    pub fn edges_percent(&self) -> f64 {
        pct(self.edges.covered, self.edges.total)
    }
    pub fn guards_percent(&self) -> f64 {
        pct(self.guards.covered, self.guards.total)
    }
}

/// Report on the root definition of `prog`.
pub fn report(prog: &ActionProgram, cov: &ActCoverage, scenarios: usize, configuration: Option<String>, requirements: &[(String, bool)]) -> ActReport {
    let d = &prog.defs[0];
    let edge_text = |e: &crate::act_ir::Edge| {
        format!("{} → {}{}", d.nodes[e.from].stmt.name, d.nodes[e.to].stmt.name, e.guard.as_ref().map(|g| format!(" [{}]", g.src)).unwrap_or_else(|| if e.is_else { " [else]".into() } else { String::new() }))
    };
    let nodes_unc: Vec<String> = d.nodes.iter().filter(|n| !cov.nodes.contains(&n.id)).map(|n| n.stmt.name.clone()).collect();
    let edges_unc: Vec<Uncovered> = d.edges.iter().filter(|e| !cov.edges.contains(&e.id)).map(|e| Uncovered { id: e.id, description: edge_text(e) }).collect();
    let decision_ids: std::collections::HashSet<usize> = d.nodes.iter().filter(|n| matches!(n.stmt.kind, NodeKind::Decision)).map(|n| n.id).collect();
    let branches: Vec<&crate::act_ir::Edge> = d.edges.iter().filter(|e| decision_ids.contains(&e.from)).collect();
    let branches_unc: Vec<Uncovered> = branches.iter().filter(|e| !cov.edges.contains(&e.id)).map(|e| Uncovered { id: e.id, description: edge_text(e) }).collect();
    // every If / Loop statement of the root definition, nested ones included, keyed by uid
    fn collect<'a>(stmts: &'a [crate::act_ir::Stmt], out: &mut Vec<&'a crate::act_ir::Stmt>) {
        for s in stmts {
            out.push(s);
            match &s.kind {
                NodeKind::If { then, els, .. } => {
                    collect(then, out);
                    collect(els, out);
                }
                NodeKind::Loop { body, .. } => collect(body, out),
                _ => {}
            }
        }
    }
    let mut all: Vec<&crate::act_ir::Stmt> = Vec::new();
    for n in &d.nodes {
        collect(std::slice::from_ref(&n.stmt), &mut all);
    }
    let loops: Vec<&&crate::act_ir::Stmt> = all.iter().filter(|s| matches!(s.kind, NodeKind::Loop { .. })).collect();
    let loops_unc: Vec<String> = loops
        .iter()
        .filter(|n| !(cov.loop_zero.contains(&n.uid) && cov.loop_some.contains(&n.uid)))
        .map(|n| format!("{} ({}{})", n.name, if cov.loop_zero.contains(&n.uid) { "" } else { "never skipped or run once" }, if cov.loop_some.contains(&n.uid) { "" } else if cov.loop_zero.contains(&n.uid) { "never repeated" } else { ", never repeated" }))
        .collect();

    let mut exprs = Vec::new();
    let (mut gc, mut gt) = (0, 0);
    let mut items: Vec<(usize, u8, &crate::ir::IrExpr)> = Vec::new();
    for e in &d.edges {
        if let Some(g) = &e.guard {
            items.push((e.id, 0, g));
        }
    }
    for s in &all {
        match &s.kind {
            NodeKind::If { cond, .. } | NodeKind::Loop { cond, .. } => items.push((s.uid, 1, cond)),
            _ => {}
        }
    }
    for (id, kind, e) in items {
        let (atoms, mcdc) = expr_coverage(e, cov.conds.get(&(id, kind)));
        for a in &atoms {
            gt += 2;
            gc += a.seen_true as usize + a.seen_false as usize;
        }
        exprs.push(CondCov { node_or_edge: id, kind: if kind == 0 { "guard" } else { "condition" }, expr: e.src.clone(), atoms, mcdc });
    }

    ActReport {
        basis: Basis { machine: prog.root.clone(), machine_hash: prog.hash.clone(), scenarios, configuration, tool_version: env!("CARGO_PKG_VERSION") },
        nodes: NameCov { covered: d.nodes.len() - nodes_unc.len(), total: d.nodes.len(), uncovered: nodes_unc },
        edges: ItemCov { covered: d.edges.len() - edges_unc.len(), total: d.edges.len(), uncovered: edges_unc },
        decisions: ItemCov { covered: branches.len() - branches_unc.len(), total: branches.len(), uncovered: branches_unc },
        loops: NameCov { covered: loops.len() - loops_unc.len(), total: loops.len(), uncovered: loops_unc },
        guards: GuardCov { covered: gc, total: gt, expressions: exprs },
        requirements: ReqCov {
            covered: requirements.iter().filter(|r| r.1).map(|r| r.0.clone()).collect(),
            uncovered: requirements.iter().filter(|r| !r.1).map(|r| r.0.clone()).collect(),
        },
    }
}

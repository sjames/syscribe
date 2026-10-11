//! The action-flow token interpreter (REQ-TRS-ACTEXEC-001, GH #276): UML-activity / fUML token semantics over
//! the action-graph IR, on a virtual clock. Pure and deterministic: an IR plus a script of `set_input`, `run`,
//! `send` and `advance` calls determines the whole trace.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde::Serialize;

use crate::act_ir::{ActionProgram, NodeKind, Stmt};
use crate::expr::{format_value, Expr, Value};
use crate::ir::IrExpr;
use crate::sm_run::Change;

#[derive(Debug, Clone)]
pub struct ActOptions {
    /// Races between simultaneously enabled assignments and unselected tokens are errors.
    pub strict: bool,
    /// Node firings before the run stops as a livelock.
    pub max_firings: usize,
    /// Iterations of one loop.
    pub max_loop: usize,
}

impl Default for ActOptions {
    fn default() -> Self {
        ActOptions { strict: false, max_firings: 10_000, max_loop: 1000 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    NotStarted,
    Completed,
    Waiting,
    Terminated,
    /// Tokens remain on successions but no node is enabled (a node needs a token on every incoming succession).
    Deadlocked,
    Error,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ActStep {
    pub n: usize,
    pub time: f64,
    pub node: String,
    pub kind: String,
    pub consumed: Vec<usize>,
    pub offered: Vec<usize>,
    pub changed: Vec<Change>,
    pub sent: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Sent {
    pub event: String,
    pub args: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default)]
pub struct ActCoverage {
    pub nodes: BTreeSet<usize>,
    pub edges: BTreeSet<usize>,
    pub loop_zero: BTreeSet<usize>,
    pub loop_some: BTreeSet<usize>,
    /// Per (edge id, 0) guard and (node id, 1) condition: atomic-condition vectors seen with the outcome.
    pub conds: BTreeMap<(usize, u8), BTreeSet<(Vec<Option<bool>>, bool)>>,
}

impl ActCoverage {
    pub fn merge(&mut self, o: &ActCoverage) {
        self.nodes.extend(&o.nodes);
        self.edges.extend(&o.edges);
        self.loop_zero.extend(&o.loop_zero);
        self.loop_some.extend(&o.loop_some);
        for (k, v) in &o.conds {
            self.conds.entry(*k).or_default().extend(v.iter().cloned());
        }
    }
}

struct Waiting {
    node: usize,
    due: Option<f64>,
}

type R<T> = Result<T, String>;

pub struct ActRun<'p> {
    prog: &'p ActionProgram,
    def: usize,
    opts: ActOptions,
    slots: Vec<Option<Value>>,
    pins: HashMap<String, Value>,
    bound: HashMap<String, Value>,
    tokens: Vec<u32>,
    start_pending: Vec<bool>,
    waiting: Vec<Waiting>,
    ran: HashSet<String>,
    time: f64,
    status: Status,
    error: Option<String>,
    terminated: bool,
    started: bool,
    firings: usize,
    trace: Vec<ActStep>,
    sent: Vec<Sent>,
    cov: RefCell<ActCoverage>,
}

fn same_type(old: &Value, new: &Value) -> bool {
    match (old, new) {
        (Value::Num(a), Value::Num(b)) => a.dim == b.dim,
        (Value::Bool(_), Value::Bool(_)) | (Value::Str(_), Value::Str(_)) => true,
        _ => false,
    }
}

impl<'p> ActRun<'p> {
    pub fn new(prog: &'p ActionProgram, opts: ActOptions) -> ActRun<'p> {
        Self::for_def(prog, 0, opts)
    }

    fn for_def(prog: &'p ActionProgram, def: usize, opts: ActOptions) -> ActRun<'p> {
        let d = &prog.defs[def];
        ActRun {
            prog,
            def,
            opts,
            slots: d.slots.iter().map(|s| s.value.clone()).collect(),
            pins: HashMap::new(),
            bound: HashMap::new(),
            tokens: vec![0; d.edges.len()],
            start_pending: vec![false; d.nodes.len()],
            waiting: Vec::new(),
            ran: HashSet::new(),
            time: 0.0,
            status: Status::NotStarted,
            error: None,
            terminated: false,
            started: false,
            firings: 0,
            trace: Vec::new(),
            sent: Vec::new(),
            cov: RefCell::new(ActCoverage::default()),
        }
    }

    fn d(&self) -> &'p crate::act_ir::ActionIr {
        &self.prog.defs[self.def]
    }

    pub fn status(&self) -> Status {
        self.status
    }
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    pub fn trace(&self) -> &[ActStep] {
        &self.trace
    }
    pub fn sent(&self) -> &[Sent] {
        &self.sent
    }
    pub fn time(&self) -> f64 {
        self.time
    }
    pub fn coverage(&self) -> ActCoverage {
        self.cov.borrow().clone()
    }
    /// Whether the named node (or nested statement) has run.
    pub fn ran(&self, name: &str) -> bool {
        self.ran.contains(name)
    }
    pub fn var(&self, name: &str) -> Option<Value> {
        let i = self.d().slots.iter().position(|s| s.name == name)?;
        self.slots[i].clone()
    }
    /// The values of the `out` and `inout` parameters that are set.
    pub fn outputs(&self) -> Vec<(String, Value)> {
        self.d().slots.iter().filter(|s| s.kind == "out" || s.kind == "inout").filter_map(|s| self.slots[s.slot].clone().map(|v| (s.name.clone(), v))).collect()
    }

    /// Give an input parameter, `inout` parameter or variable its value before (or between) runs.
    pub fn set_input(&mut self, name: &str, v: Value) -> Result<(), String> {
        let Some(s) = self.d().slots.iter().find(|s| s.name == name) else { return Err(format!("'{name}' is not a parameter or variable of {}", self.d().qname)) };
        if s.kind == "out" {
            return Err(format!("'{name}' is an out parameter; it is produced by the action"));
        }
        let v = crate::constraints::coerce_param(v, s.typed_by.as_deref(), s.unit.as_deref());
        if let Some(old) = &self.slots[s.slot] {
            if !same_type(old, &v) {
                return Err(format!("'{name}' is a {}, not a {}", old.kind(), v.kind()));
            }
        }
        self.slots[s.slot] = Some(v);
        Ok(())
    }

    fn halted(&self) -> bool {
        self.error.is_some() || self.terminated
    }

    fn fail(&mut self, e: String) {
        if self.error.is_none() {
            let mut st = self.step("error", "error");
            st.note = Some(e.clone());
            self.trace.push(st);
            self.error = Some(e);
        }
        self.status = Status::Error;
    }

    fn step(&self, node: &str, kind: &str) -> ActStep {
        ActStep { n: self.trace.len() + 1, time: self.time, node: node.to_string(), kind: kind.to_string(), consumed: vec![], offered: vec![], changed: vec![], sent: None, note: None }
    }

    // ── external API ────────────────────────────────────────────────────────

    /// Start the action and run it until it completes, waits or fails.
    pub fn run(&mut self) {
        if self.started || self.halted() {
            return;
        }
        self.started = true;
        for &s in &self.d().start {
            self.start_pending[s] = true;
        }
        let r = self.settle();
        self.finish(r);
    }

    /// Inject an event: the first waiting `AcceptAction` of that event (document order) receives it.
    pub fn send(&mut self, event: &str, payload: &[(&str, Value)]) {
        if !self.started || self.halted() {
            return;
        }
        let d = self.d();
        // the awaiting accept earliest in document order receives it
        let target = self
            .waiting
            .iter()
            .enumerate()
            .filter(|(_, w)| matches!(&d.nodes[w.node].stmt.kind, NodeKind::Accept { event: Some(e), .. } if e == event))
            .min_by_key(|(_, w)| w.node)
            .map(|(i, _)| i);
        let r = match target {
            Some(i) => {
                let w = self.waiting.remove(i);
                if let NodeKind::Accept { bind: Some(b), .. } = &d.nodes[w.node].stmt.kind {
                    for (k, v) in payload {
                        self.bound.insert(format!("{b}.{k}"), v.clone());
                    }
                }
                self.complete_accept(w.node, format!("event {event}")).and_then(|_| self.settle())
            }
            None if self.opts.strict => Err(format!("strict: event {event} is not awaited by any AcceptAction")),
            None => {
                let mut st = self.step(event, "event");
                st.note = Some(format!("event {event} ignored"));
                self.trace.push(st);
                Ok(())
            }
        };
        self.finish(r);
    }

    /// Advance the virtual clock, completing timed accepts as they fall due.
    pub fn advance(&mut self, secs: f64) {
        if !self.started || self.halted() {
            return;
        }
        let target = self.time + secs;
        loop {
            let next = self.waiting.iter().filter_map(|w| w.due).filter(|d| *d <= target).fold(f64::INFINITY, f64::min);
            if !next.is_finite() {
                break;
            }
            self.time = next.max(self.time);
            let r = self.settle();
            self.finish(r);
            if self.halted() {
                return;
            }
        }
        self.time = target;
        let r = self.settle();
        self.finish(r);
    }

    fn finish(&mut self, r: R<()>) {
        if let Err(e) = r {
            self.fail(e);
        } else if self.terminated {
            self.status = Status::Terminated;
        } else if self.status != Status::Error {
            self.status = if !self.waiting.is_empty() {
                Status::Waiting
            } else if self.tokens.iter().any(|t| *t > 0) {
                if self.status != Status::Deadlocked {
                    let stuck: Vec<String> = self.tokens.iter().enumerate().filter(|(_, t)| **t > 0).map(|(i, _)| format!("{} → {}", self.d().nodes[self.d().edges[i].from].stmt.name, self.d().nodes[self.d().edges[i].to].stmt.name)).collect();
                    let mut st = self.step("deadlock", "deadlock");
                    st.note = Some(format!("deadlock: tokens remain on {} but no node is enabled", stuck.join(", ")));
                    self.trace.push(st);
                }
                Status::Deadlocked
            } else {
                Status::Completed
            };
        }
    }

    /// Evaluate an expression over the parameters, variables and pins.
    pub fn eval_str(&self, src: &str) -> Result<Value, String> {
        let ast = crate::expr::parse(src).map_err(|e| format!("`{src}`: {e}"))?;
        self.eval_ast(&ast).map_err(|e| format!("`{src}`: {e}"))
    }

    // ── machinery ───────────────────────────────────────────────────────────

    fn enabled(&self, n: usize) -> bool {
        let d = self.d();
        if self.start_pending[n] {
            return true;
        }
        let incoming: Vec<usize> = d.edges.iter().filter(|e| e.to == n).map(|e| e.id).collect();
        if incoming.is_empty() {
            return false;
        }
        if matches!(d.nodes[n].stmt.kind, NodeKind::Merge) {
            incoming.iter().any(|e| self.tokens[*e] > 0)
        } else {
            incoming.iter().all(|e| self.tokens[*e] > 0)
        }
    }

    /// Consume the tokens that enable `n`; returns the edges they came from.
    fn consume(&mut self, n: usize) -> Vec<usize> {
        let d = self.d();
        if self.start_pending[n] {
            self.start_pending[n] = false;
            return vec![];
        }
        let incoming: Vec<usize> = d.edges.iter().filter(|e| e.to == n).map(|e| e.id).collect();
        if matches!(d.nodes[n].stmt.kind, NodeKind::Merge) {
            let e = *incoming.iter().find(|e| self.tokens[**e] > 0).expect("enabled");
            self.tokens[e] -= 1;
            return vec![e];
        }
        for e in &incoming {
            self.tokens[*e] -= 1;
        }
        incoming
    }

    fn settle(&mut self) -> R<()> {
        loop {
            if self.halted() {
                return Ok(());
            }
            // timed accepts that have fallen due, in document order
            let due: Vec<usize> = {
                let mut v: Vec<usize> = self.waiting.iter().filter(|w| w.due.is_some_and(|d| d <= self.time)).map(|w| w.node).collect();
                v.sort_unstable();
                v
            };
            if let Some(&n) = due.first() {
                self.waiting.retain(|w| w.node != n);
                self.complete_accept(n, "timer".into())?;
                continue;
            }
            let enabled: Vec<usize> = (0..self.d().nodes.len()).filter(|n| self.enabled(*n)).collect();
            let Some(&n) = enabled.first() else { return Ok(()) };
            if self.opts.strict && enabled.len() > 1 {
                self.check_race(&enabled)?;
            }
            self.firings += 1;
            if self.firings > self.opts.max_firings {
                return Err(format!("exceeded {} node firings (livelock?)", self.opts.max_firings));
            }
            self.fire(n)?;
        }
    }

    /// Strict mode: two simultaneously enabled assignments writing the same slot are a race.
    fn check_race(&self, enabled: &[usize]) -> R<()> {
        let d = self.d();
        let writes = |n: usize| -> Vec<usize> {
            match &d.nodes[n].stmt.kind {
                NodeKind::Assign { assigns } => assigns.iter().map(|a| a.slot).collect(),
                _ => vec![],
            }
        };
        for (i, a) in enabled.iter().enumerate() {
            for b in &enabled[i + 1..] {
                if let Some(s) = writes(*a).iter().find(|s| writes(*b).contains(s)) {
                    return Err(format!("strict: '{}' and '{}' are enabled together and both assign '{}'", d.nodes[*a].stmt.name, d.nodes[*b].stmt.name, d.slots[*s].name));
                }
            }
        }
        Ok(())
    }

    fn fire(&mut self, n: usize) -> R<()> {
        let d = self.d();
        let node = &d.nodes[n];
        let consumed = self.consume(n);
        if let NodeKind::Accept { after_seconds, .. } = &node.stmt.kind {
            // an accept consumes its tokens and then waits
            let mut st = self.step(&node.stmt.name, "accept");
            st.consumed = consumed;
            st.note = Some("waiting".into());
            self.trace.push(st);
            self.waiting.push(Waiting { node: n, due: after_seconds.map(|a| self.time + a) });
            return Ok(());
        }
        let kind = serde_json::to_value(&node.stmt.kind).ok().and_then(|v| v["kind"].as_str().map(String::from)).unwrap_or_default();
        let mut st = self.step(&node.stmt.name, &kind);
        st.consumed = consumed;
        let before = self.slots.clone();
        let r = self.exec(&node.stmt, &mut st);
        self.record_changes(&before, &mut st);
        self.cov.borrow_mut().nodes.insert(n);
        self.ran.insert(node.stmt.name.clone());
        let r = r.and_then(|_| if self.terminated { Ok(()) } else { self.complete(n, &mut st) });
        self.trace.push(st);
        r
    }

    fn complete_accept(&mut self, n: usize, why: String) -> R<()> {
        let d = self.d();
        let mut st = self.step(&d.nodes[n].stmt.name, "accept");
        st.note = Some(format!("received ({why})"));
        self.cov.borrow_mut().nodes.insert(n);
        self.ran.insert(d.nodes[n].stmt.name.clone());
        let r = self.complete(n, &mut st);
        self.trace.push(st);
        r
    }

    /// A node has finished: apply its data flows, then offer tokens on its outgoing edges.
    fn complete(&mut self, n: usize, st: &mut ActStep) -> R<()> {
        let d = self.d();
        let name = d.nodes[n].stmt.name.clone();
        for f in d.flows.iter().filter(|f| f.from_node == n) {
            let Some(v) = self.pins.get(&format!("{name}.{}", f.from_pin)).cloned() else {
                st.note = Some(format!("flow source {name}.{} has no value", f.from_pin));
                continue;
            };
            if let (Some(tn), Some(tp)) = (f.to_node, &f.to_pin) {
                self.pins.insert(format!("{}.{tp}", d.nodes[tn].stmt.name), v);
            } else if let Some(slot) = f.to_slot {
                self.assign(slot, v, st)?;
            }
        }
        let out: Vec<&crate::act_ir::Edge> = d.edges.iter().filter(|e| e.from == n).collect();
        let mut chosen: Vec<usize> = Vec::new();
        match d.nodes[n].stmt.kind {
            NodeKind::Decision => {
                let mut els = None;
                for e in &out {
                    if e.is_else {
                        els = Some(e.id);
                        continue;
                    }
                    let ok = match &e.guard {
                        None => true,
                        Some(g) => self.truth(g, e.id, 0, &format!("guard of the succession to '{}'", d.nodes[e.to].stmt.name))?,
                    };
                    if ok {
                        chosen.push(e.id);
                        break;
                    }
                }
                if chosen.is_empty() {
                    chosen.extend(els);
                }
            }
            _ => {
                for e in &out {
                    let ok = match &e.guard {
                        None => !e.is_else,
                        Some(g) => self.truth(g, e.id, 0, &format!("guard of the succession to '{}'", d.nodes[e.to].stmt.name))?,
                    };
                    if ok {
                        chosen.push(e.id);
                    }
                }
            }
        }
        if chosen.is_empty() && !out.is_empty() {
            let msg = format!("no outgoing succession of '{name}' was selected; the token is dropped");
            if self.opts.strict {
                return Err(format!("strict: {msg}"));
            }
            st.note = Some(msg);
        }
        for e in chosen {
            self.tokens[e] += 1;
            self.cov.borrow_mut().edges.insert(e);
            st.offered.push(e);
        }
        Ok(())
    }

    fn record_changes(&self, before: &[Option<Value>], st: &mut ActStep) {
        for (i, v) in self.slots.iter().enumerate() {
            if &before[i] != v {
                if let Some(new) = v {
                    st.changed.push(Change { var: self.d().slots[i].name.clone(), old: before[i].as_ref().map(format_value).unwrap_or_else(|| "(unset)".into()), new: format_value(new) });
                }
            }
        }
    }

    fn assign(&mut self, slot: usize, v: Value, _st: &mut ActStep) -> R<()> {
        let name = &self.d().slots[slot].name;
        if let Some(old) = &self.slots[slot] {
            if !same_type(old, &v) {
                return Err(format!("variable '{name}': cannot assign a {} to a {}", v.kind(), old.kind()));
            }
        }
        self.slots[slot] = Some(v);
        Ok(())
    }

    // ── expressions ─────────────────────────────────────────────────────────

    fn eval_ast(&self, ast: &Expr) -> Result<Value, crate::expr::ExprError> {
        let d = self.d();
        let lookup = |name: &str| -> Option<Value> {
            if let Some(i) = d.slots.iter().position(|s| s.name == name) {
                return self.slots[i].clone();
            }
            self.pins.get(name).or_else(|| self.bound.get(name)).cloned()
        };
        ast.eval(&lookup)
    }

    fn eval(&self, e: &IrExpr, where_: &str) -> R<Value> {
        self.eval_ast(&e.ast).map_err(|x| format!("{where_}: `{}`: {x}", e.src))
    }

    fn truth(&self, e: &IrExpr, key: usize, kind: u8, where_: &str) -> R<bool> {
        let v = match self.eval(e, where_)? {
            Value::Bool(b) => b,
            other => return Err(format!("{where_}: `{}` is a {}, not a boolean", e.src, other.kind())),
        };
        let vec: Vec<Option<bool>> = e.ast.atoms().into_iter().map(|a| if let Ok(Value::Bool(b)) = self.eval_ast(a) { Some(b) } else { None }).collect();
        self.cov.borrow_mut().conds.entry((key, kind)).or_default().insert((vec, v));
        Ok(v)
    }

    // ── node execution ──────────────────────────────────────────────────────

    fn exec(&mut self, s: &Stmt, st: &mut ActStep) -> R<()> {
        match &s.kind {
            NodeKind::Assign { assigns } => {
                for a in assigns {
                    let v = self.eval(&a.expr, &format!("sub-action '{}'", s.name))?;
                    self.assign(a.slot, v, st)?;
                }
            }
            NodeKind::Perform { callee, args } => {
                let mut child = ActRun::for_def(self.prog, *callee, self.opts.clone());
                child.time = self.time;
                let cd = &self.prog.defs[*callee];
                for a in args {
                    let v = self.eval(&a.expr, &format!("sub-action '{}' parameter '{}'", s.name, a.name))?;
                    child.set_input(&a.name, v).map_err(|e| format!("sub-action '{}': {e}", s.name))?;
                }
                for slot in cd.slots.iter().filter(|x| x.kind == "in" || x.kind == "inout") {
                    if args.iter().any(|a| a.name == slot.name) {
                        continue;
                    }
                    if let Some(v) = self.pins.get(&format!("{}.{}", s.name, slot.name)).cloned() {
                        child.set_input(&slot.name, v).map_err(|e| format!("sub-action '{}': {e}", s.name))?;
                    }
                }
                child.run();
                match child.status {
                    Status::Error => return Err(format!("sub-action '{}' ({}): {}", s.name, cd.qname, child.error.clone().unwrap_or_default())),
                    Status::Waiting => return Err(format!("sub-action '{}': the called action {} waits for an event; a called action must complete", s.name, cd.qname)),
                    _ => {}
                }
                let prefix = format!("{}.", s.name);
                self.pins.retain(|k, _| !(k.starts_with(&prefix) && cd.slots.iter().any(|x| (x.kind == "out" || x.kind == "inout") && k == &format!("{prefix}{}", x.name))));
                for (k, v) in child.outputs() {
                    self.pins.insert(format!("{}.{k}", s.name), v);
                }
                st.note = Some(format!("performed {} in {} firing(s)", cd.qname, child.trace.len()));
            }
            NodeKind::Send { event, args } => {
                let mut shown = Vec::new();
                for a in args {
                    let v = self.eval(&a.expr, &format!("sub-action '{}' parameter '{}'", s.name, a.name))?;
                    shown.push((a.name.clone(), format_value(&v)));
                }
                st.sent = Some(event.clone());
                self.sent.push(Sent { event: event.clone(), args: shown });
            }
            NodeKind::If { cond, then, els } => {
                let b = self.truth(cond, s.uid, 1, &format!("sub-action '{}' condition", s.name))?;
                self.block(if b { then } else { els }, st)?;
            }
            NodeKind::Loop { loop_kind, cond, body } => {
                let mut iterations = 0usize;
                loop {
                    let w = format!("sub-action '{}' condition", s.name);
                    if *loop_kind == "while" && !self.truth(cond, s.uid, 1, &w)? {
                        break;
                    }
                    iterations += 1;
                    if iterations > self.opts.max_loop {
                        return Err(format!("sub-action '{}': more than {} loop iterations", s.name, self.opts.max_loop));
                    }
                    self.block(body, st)?;
                    if self.terminated {
                        break;
                    }
                    // `until` tests after the body
                    if *loop_kind == "until" && self.truth(cond, s.uid, 1, &w)? {
                        break;
                    }
                }
                // "zero" = no repetition beyond the minimum (a `while` that never entered, an `until` that ran once)
                let minimum = if *loop_kind == "until" { 1 } else { 0 };
                let mut c = self.cov.borrow_mut();
                if iterations <= minimum {
                    c.loop_zero.insert(s.uid);
                } else {
                    c.loop_some.insert(s.uid);
                }
            }
            NodeKind::Terminate => {
                self.terminated = true;
                st.note = Some("terminated".into());
            }
            NodeKind::Accept { .. } | NodeKind::Action | NodeKind::Fork | NodeKind::Join | NodeKind::Decision | NodeKind::Merge => {}
        }
        Ok(())
    }

    fn block(&mut self, stmts: &[Stmt], st: &mut ActStep) -> R<()> {
        for s in stmts {
            self.ran.insert(s.name.clone());
            self.exec(s, st)?;
            if self.terminated {
                break;
            }
        }
        Ok(())
    }
}

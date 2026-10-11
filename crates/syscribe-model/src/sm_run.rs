//! The state-machine interpreter (REQ-TRS-SMEXEC-001, GH #279): SCXML run-to-completion over the IR,
//! with SysML v2 / KerML change and time triggers (`docs/design/state-machine-execution-semantics.md`).
//! A virtual clock, no threads, no hidden state: an IR plus a script of `send`/`advance`/`set` calls
//! determines the whole trace.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::Serialize;

use crate::expr::{format_value, Expr, Value};
use crate::ir::{Action, IrExpr, Kind, Machine, Trigger};

#[derive(Debug, Clone)]
pub struct Options {
    /// Unhandled events and equal-priority conflicts are errors instead of being ignored / resolved by document order.
    pub strict: bool,
    /// Microsteps allowed in one macrostep before the run stops as a livelock.
    pub max_microsteps: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options { strict: false, max_microsteps: 1000 }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Change {
    pub var: String,
    pub old: String,
    pub new: String,
}

/// One entry of the trace: a microstep, or an event that was ignored, or an environment write.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Step {
    pub n: usize,
    pub time: f64,
    pub cause: String,
    /// The first transition taken, and all of them (parallel regions take several in one microstep).
    pub transition: Option<usize>,
    pub taken: Vec<usize>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub guard: Option<bool>,
    pub exited: Vec<String>,
    pub entered: Vec<String>,
    pub actions: Vec<String>,
    pub changed: Vec<Change>,
    pub note: Option<String>,
}

/// What a run exercised, for coverage reports (REQ-TRS-SMCOV-001).
#[derive(Debug, Clone, Default)]
pub struct Coverage {
    pub states: BTreeSet<usize>,
    pub transitions: BTreeSet<usize>,
    /// Consecutive transitions through a state: the first one's target is the second one's source.
    pub pairs: BTreeSet<(usize, usize)>,
    /// Per (transition, 0 = guard / 1 = `when`): the atomic-condition vectors seen with the outcome.
    pub conds: BTreeMap<(usize, u8), BTreeSet<(Vec<Option<bool>>, bool)>>,
    pub timers_fired: BTreeSet<usize>,
    pub timers_cancelled: BTreeSet<usize>,
}

impl Coverage {
    pub fn merge(&mut self, o: &Coverage) {
        self.states.extend(&o.states);
        self.transitions.extend(&o.transitions);
        self.pairs.extend(&o.pairs);
        for (k, v) in &o.conds {
            self.conds.entry(*k).or_default().extend(v.iter().cloned());
        }
        self.timers_fired.extend(&o.timers_fired);
        self.timers_cancelled.extend(&o.timers_cancelled);
    }
}

enum Ev {
    Named { name: String, payload: Vec<(String, Value)> },
    Signal(usize),
    Done(usize),
}

struct Watch {
    trans: usize,
    /// The condition was true when the observation was re-armed by an internal transition: wait for it to fall first.
    need_false: bool,
    fired: bool,
}

type R<T> = Result<T, String>;

pub struct Run<'m> {
    m: &'m Machine,
    opts: Options,
    active: Vec<bool>,
    children: Vec<Vec<usize>>,
    vars: Vec<Value>,
    time: f64,
    internal: VecDeque<Ev>,
    timers: Vec<(f64, usize)>,
    watches: Vec<Watch>,
    trace: Vec<Step>,
    error: Option<String>,
    terminated: bool,
    micro: usize,
    cov: RefCell<Coverage>,
    last_taken: Option<usize>,
    cosim: Option<crate::plant::CoSim>,
}

impl<'m> Run<'m> {
    /// Create a run and start the machine (enter the initial states and settle).
    pub fn new(m: &'m Machine, opts: Options) -> Run<'m> {
        Self::build(m, opts, None)
    }

    /// Like [`Run::new`], with plant models co-simulated on the shared clock (REQ-TRS-FMI-001): their initial
    /// outputs are read into the mapped variables before the machine starts.
    pub fn with_plants(m: &'m Machine, opts: Options, cosim: crate::plant::CoSim) -> Run<'m> {
        Self::build(m, opts, Some(cosim))
    }

    fn build(m: &'m Machine, opts: Options, cosim: Option<crate::plant::CoSim>) -> Run<'m> {
        let mut children = vec![Vec::new(); m.states.len()];
        for s in &m.states {
            if let Some(p) = s.parent {
                children[p].push(s.id);
            }
        }
        let mut r = Run {
            m,
            opts,
            active: vec![false; m.states.len()],
            children,
            vars: m.variables.iter().map(|v| v.value.clone()).collect(),
            time: 0.0,
            internal: VecDeque::new(),
            timers: Vec::new(),
            watches: Vec::new(),
            trace: Vec::new(),
            error: None,
            terminated: false,
            micro: 0,
            cov: RefCell::new(Coverage::default()),
            last_taken: None,
            cosim,
        };
        let init = r.plants_read();
        r.finish(init);
        if r.error.is_none() {
            let r0 = r.start();
            r.finish(r0);
        }
        r
    }

    pub fn active(&self) -> Vec<String> {
        self.m.states.iter().filter(|s| self.active[s.id]).map(|s| s.path.clone()).collect()
    }
    pub fn var(&self, name: &str) -> Option<Value> {
        self.m.variables.iter().position(|v| v.name == name).map(|i| self.vars[i].clone())
    }
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    pub fn trace(&self) -> &[Step] {
        &self.trace
    }
    pub fn time(&self) -> f64 {
        self.time
    }
    pub fn coverage(&self) -> Coverage {
        self.cov.borrow().clone()
    }
    pub fn terminated(&self) -> bool {
        self.terminated
    }

    fn finish(&mut self, r: R<()>) {
        if let Err(e) = r {
            if self.error.is_none() {
                let mut step = self.begin("error".into());
                step.note = Some(e.clone());
                self.trace.push(step);
                self.error = Some(e);
            }
        }
    }

    fn halted(&self) -> bool {
        self.error.is_some() || self.terminated
    }

    // ── external API ────────────────────────────────────────────────────────

    /// Inject an external event and run to completion.
    pub fn send(&mut self, name: &str, payload: &[(&str, Value)]) {
        if self.halted() {
            return;
        }
        let ev = Ev::Named { name: name.to_string(), payload: payload.iter().map(|(k, v)| (k.to_string(), v.clone())).collect() };
        let r = self.process_external(ev, format!("event {name}"));
        self.finish(r);
    }

    /// Advance the virtual clock, firing due timers in order of due time.
    pub fn advance(&mut self, secs: f64) {
        if self.halted() {
            return;
        }
        if self.cosim.is_some() {
            return self.advance_with_plants(secs);
        }
        let target = self.time + secs;
        loop {
            let next = self.timers.iter().map(|t| t.0).filter(|d| *d <= target).fold(f64::INFINITY, f64::min);
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

    /// What the plants report about themselves, for recorded evidence.
    pub fn plant_provenance(&self) -> Vec<serde_json::Value> {
        self.cosim.as_ref().map(|c| c.provenance()).unwrap_or_default()
    }

    /// The co-simulation loop of REQ-TRS-FMI-001: step to the next communication point (a multiple of the plant
    /// step), cut short at the earliest discrete due time; inputs are held constant over the step.
    fn advance_with_plants(&mut self, secs: f64) {
        const EPS: f64 = 1e-12;
        let target = self.time + secs;
        let h = self.cosim.as_ref().expect("plants").step;
        // a step this small (or a run this long) would take effectively forever, and float grid arithmetic degrades
        if !(secs / h <= 5e6 && (self.time + secs) / h < 9e15) {
            let r = Err(format!("{} s at a plant step of {h} s is more than 5,000,000 communication steps; use a larger `plantStep`", secs));
            return self.finish(r);
        }
        while self.time < target - EPS {
            let grid = ((self.time / h + 1e-9).floor() + 1.0) * h;
            let mut t_next = grid.min(target);
            let due = self.timers.iter().map(|t| t.0).filter(|d| *d > self.time + EPS && *d < t_next).fold(f64::INFINITY, f64::min);
            if due.is_finite() {
                t_next = due;
            }
            let t = self.time;
            let r = self.plants_write().and_then(|_| self.plants_step(t, t_next - t));
            self.time = t_next;
            let r = r.and_then(|_| self.plants_read()).and_then(|_| self.settle());
            self.finish(r);
            if self.halted() {
                return;
            }
        }
        self.time = target.max(self.time);
        let r = self.settle();
        self.finish(r);
    }

    fn plants_write(&mut self) -> R<()> {
        let Some(mut c) = self.cosim.take() else { return Ok(()) };
        let r = c.write(&self.vars);
        self.cosim = Some(c);
        r
    }

    fn plants_step(&mut self, t: f64, h: f64) -> R<()> {
        let Some(mut c) = self.cosim.take() else { return Ok(()) };
        let r = c.step_all(t, h);
        self.cosim = Some(c);
        r
    }

    /// Read the plants' outputs into the mapped variables; the values that changed are traced.
    fn plants_read(&mut self) -> R<()> {
        let Some(mut c) = self.cosim.take() else { return Ok(()) };
        let before = self.vars.clone();
        let r = c.read(&mut self.vars);
        self.cosim = Some(c);
        r?;
        let mut step = self.begin("plant".into());
        self.record_changes(&before, &mut step);
        if !step.changed.is_empty() {
            self.trace.push(step);
        }
        Ok(())
    }

    /// Write a variable from outside the machine (an environment input) and run to completion.
    pub fn set(&mut self, name: &str, value: Value) {
        if self.halted() {
            return;
        }
        let r = (|| -> R<()> {
            let Some(slot) = self.m.variables.iter().position(|v| v.name == name) else {
                return Err(format!("set: '{name}' is not a variable of the machine"));
            };
            check_assignable(name, &self.vars[slot], &value)?;
            let mut step = self.begin(format!("set {name}"));
            step.changed.push(Change { var: name.to_string(), old: format_value(&self.vars[slot]), new: format_value(&value) });
            self.vars[slot] = value;
            self.trace.push(step);
            self.settle()
        })();
        self.finish(r);
    }

    /// Evaluate an expression over the variables (no payload scope).
    pub fn eval_str(&self, src: &str) -> Result<Value, String> {
        let ast = crate::expr::parse(src).map_err(|e| format!("`{src}`: {e}"))?;
        let m = self.m;
        let lookup = |name: &str| m.variables.iter().position(|v| v.name == name).map(|i| self.vars[i].clone());
        ast.eval(&lookup).map_err(|e| format!("`{src}`: {e}"))
    }

    fn find_state(&self, name: &str) -> R<usize> {
        let hits: Vec<usize> = self
            .m
            .states
            .iter()
            .filter(|s| s.path == name || s.path.ends_with(&format!(".{name}")) || (!name.contains('.') && s.name == name))
            .map(|s| s.id)
            .collect();
        let exact: Vec<usize> = hits.iter().copied().filter(|i| self.m.states[*i].path == name).collect();
        match (exact.len(), hits.len()) {
            (1, _) => Ok(exact[0]),
            (_, 1) => Ok(hits[0]),
            (_, 0) => Err(format!("'{name}' is not a state of the machine")),
            _ => Err(format!("state name '{name}' is ambiguous; use a dotted path")),
        }
    }

    /// Whether the named state (a name, `a/b` or dotted path) is active.
    pub fn holds_state(&self, name: &str) -> Result<bool, String> {
        Ok(self.active[self.find_state(name)?])
    }

    /// Put the machine directly into a state (and its default descendants and ancestors) without running any
    /// actions: the set-up of a scenario `Given … is in state …`. Timers and change observations are armed as
    /// on entry.
    pub fn force(&mut self, name: &str) -> Result<(), String> {
        let id = self.find_state(name)?;
        let m = self.m;
        self.active.iter_mut().for_each(|a| *a = false);
        self.timers.clear();
        self.watches.clear();
        self.internal.clear();
        let mut on_path = self.ancestors(id);
        on_path.push(id);
        let tops: Vec<usize> = if m.parallel {
            m.states.iter().filter(|s| s.parent.is_none()).map(|s| s.id).collect()
        } else {
            vec![self.ancestors(id).last().copied().unwrap_or(id)]
        };
        let mut list = Vec::new();
        for t in tops {
            self.plan_enter(t, &on_path, &mut list);
        }
        for s in &list {
            self.active[*s] = true;
        }
        self.last_taken = None;
        let mut step = self.begin(format!("given {name}"));
        step.entered = list.iter().map(|s| m.states[*s].path.clone()).collect();
        step.note = Some("state set directly, no actions run".into());
        self.trace.push(step);
        let r = self.arm(&list).and_then(|_| self.settle());
        self.finish(r.clone());
        r
    }

    // ── machinery ───────────────────────────────────────────────────────────

    fn begin(&self, cause: String) -> Step {
        Step {
            n: self.trace.len() + 1,
            time: self.time,
            cause,
            transition: None,
            taken: vec![],
            from: None,
            to: None,
            guard: None,
            exited: vec![],
            entered: vec![],
            actions: vec![],
            changed: vec![],
            note: None,
        }
    }

    fn start(&mut self) -> R<()> {
        let m = self.m;
        let mut step = self.begin("start".into());
        let before = self.vars.clone();
        let acts = m.initial_actions.clone();
        self.run_actions(&acts, None, &mut step)?;
        let mut list = Vec::new();
        if m.parallel {
            for s in m.states.iter().filter(|s| s.parent.is_none()) {
                self.plan_enter(s.id, &[], &mut list);
            }
        } else if let Some(i) = m.initial {
            self.plan_enter(i, &[], &mut list);
        }
        let res = list.iter().try_for_each(|s| self.enter_one(*s, &mut step)).and_then(|_| self.arm(&list));
        self.record_changes(&before, &mut step);
        self.trace.push(step);
        res?;
        self.settle()
    }

    fn process_external(&mut self, ev: Ev, cause: String) -> R<()> {
        self.settle()?;
        if self.halted() {
            return Ok(());
        }
        let name = if let Ev::Named { name, .. } = &ev { name.clone() } else { String::new() };
        match self.select(Some(&ev))? {
            Some(sel) => self.microstep(sel, Some(&ev), cause)?,
            None => {
                let mut step = self.begin(cause);
                step.note = Some(format!("event {name} ignored"));
                self.trace.push(step);
                if self.opts.strict {
                    return Err(format!("strict: event {name} is not handled in {:?}", self.active()));
                }
            }
        }
        self.settle()
    }

    fn settle(&mut self) -> R<()> {
        self.micro = 0;
        loop {
            if self.halted() {
                return Ok(());
            }
            self.micro += 1;
            if self.micro > self.opts.max_microsteps {
                return Err(format!("exceeded {} microsteps in one macrostep (livelock?)", self.opts.max_microsteps));
            }
            self.collect_signals()?;
            if let Some(sel) = self.select(None)? {
                self.microstep(sel, None, "eventless".into())?;
                continue;
            }
            if let Some(ev) = self.internal.pop_front() {
                let (cause, stale) = match &ev {
                    Ev::Signal(t) => {
                        let tr = &self.m.transitions[*t];
                        let kind = if matches!(tr.trigger, Trigger::Timer { .. }) { "timer" } else { "change" };
                        (format!("{kind} t{t}"), !self.active[tr.source])
                    }
                    Ev::Done(s) => (format!("done {}", self.m.states[*s].path), false),
                    Ev::Named { name, .. } => (format!("event {name}"), false),
                };
                if stale {
                    continue;
                }
                match self.select(Some(&ev))? {
                    Some(sel) => self.microstep(sel, Some(&ev), cause)?,
                    None => {
                        if let Ev::Signal(_) = ev {
                            let mut step = self.begin(cause);
                            step.note = Some("signal consumed, no transition taken (guard false)".into());
                            self.trace.push(step);
                        }
                    }
                }
                continue;
            }
            return Ok(());
        }
    }

    /// Turn due timers and satisfied change observations into internal signals.
    fn collect_signals(&mut self) -> R<()> {
        let m = self.m;
        let mut due: Vec<(f64, usize, usize)> = self.timers.iter().filter(|t| t.0 <= self.time).map(|t| (t.0, m.transitions[t.1].source, t.1)).collect();
        due.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.timers.retain(|t| t.0 > self.time);
        for (_, _, t) in due {
            self.cov.borrow_mut().timers_fired.insert(t);
            self.internal.push_back(Ev::Signal(t));
        }
        for i in 0..self.watches.len() {
            if self.watches[i].fired {
                continue;
            }
            let t = self.watches[i].trans;
            let Trigger::Change { cond } = &m.transitions[t].trigger else { continue };
            let v = self.truth_rec(t, 1, cond, None, &format!("transition t{t} `when`"))?;
            let w = &mut self.watches[i];
            if w.need_false {
                if !v {
                    w.need_false = false;
                }
            } else if v {
                w.fired = true;
                self.internal.push_back(Ev::Signal(t));
            }
        }
        self.watches.retain(|w| !w.fired);
        Ok(())
    }

    fn truth(&self, e: &IrExpr, scope: Option<&Scope>, where_: &str) -> R<bool> {
        match self.eval(e, scope, where_)? {
            Value::Bool(b) => Ok(b),
            other => Err(format!("{where_}: `{}` is a {}, not a boolean", e.src, other.kind())),
        }
    }

    fn truth_rec(&self, t: usize, kind: u8, e: &IrExpr, scope: Option<&Scope>, where_: &str) -> R<bool> {
        let v = self.truth(e, scope, where_)?;
        self.record_cond(t, kind, e, scope, v);
        Ok(v)
    }

    fn eval(&self, e: &IrExpr, scope: Option<&Scope>, where_: &str) -> R<Value> {
        self.eval_ast(&e.ast, scope).map_err(|x| format!("{where_}: `{}`: {x}", e.src))
    }

    fn eval_ast(&self, ast: &Expr, scope: Option<&Scope>) -> Result<Value, crate::expr::ExprError> {
        let m = self.m;
        let lookup = |name: &str| -> Option<Value> {
            if let Some(i) = m.variables.iter().position(|v| v.name == name) {
                return Some(self.vars[i].clone());
            }
            if let Some(Scope { bind: Some(b), payload }) = scope {
                if let Some(field) = name.strip_prefix(&format!("{b}.")) {
                    return payload.iter().find(|(k, _)| k == field).map(|(_, v)| v.clone());
                }
            }
            None
        };
        ast.eval(&lookup)
    }

    /// Record the atomic conditions of a guard / `when` evaluation and its outcome.
    fn record_cond(&self, t: usize, kind: u8, e: &IrExpr, scope: Option<&Scope>, outcome: bool) {
        // An atom that cannot be evaluated (a short-circuited division by zero, say) is recorded as unknown.
        let vec: Vec<Option<bool>> = e
            .ast
            .atoms()
            .into_iter()
            .map(|a| match self.eval_ast(a, scope) {
                Ok(Value::Bool(b)) => Some(b),
                _ => None,
            })
            .collect();
        self.cov.borrow_mut().conds.entry((t, kind)).or_default().insert((vec, outcome));
    }

    // ── selection ───────────────────────────────────────────────────────────

    fn ancestors(&self, s: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut p = self.m.states[s].parent;
        while let Some(x) = p {
            out.push(x);
            p = self.m.states[x].parent;
        }
        out
    }

    /// Whether transition `t` is triggered by `ev` (None = eventless pass), and the payload scope it binds.
    fn matches<'e>(&self, t: usize, ev: Option<&'e Ev>) -> Option<Scope<'e>> {
        let tr = &self.m.transitions[t];
        match (&tr.trigger, ev) {
            (Trigger::Eventless, None) => Some(Scope { bind: None, payload: &[] }),
            (Trigger::Event { name, bind }, Some(Ev::Named { name: n, payload })) if name == n => {
                Some(Scope { bind: bind.clone(), payload })
            }
            (Trigger::Timer { .. }, Some(Ev::Signal(x))) | (Trigger::Change { .. }, Some(Ev::Signal(x))) if *x == t => {
                Some(Scope { bind: None, payload: &[] })
            }
            (Trigger::Done { state }, Some(Ev::Done(d))) if state == d => Some(Scope { bind: None, payload: &[] }),
            _ => None,
        }
    }

    /// The transition domain: the nearest proper ancestor of the source that is also a proper ancestor of the
    /// target and is a compound state; `None` is the machine itself.
    fn domain(&self, t: usize) -> Option<usize> {
        let tr = &self.m.transitions[t];
        let target = tr.target?;
        let tgt_anc = self.ancestors(target);
        self.ancestors(tr.source).into_iter().find(|a| tgt_anc.contains(a) && self.m.states[*a].kind == Kind::Compound)
    }

    fn exit_set(&self, t: usize) -> Vec<usize> {
        if self.m.transitions[t].internal {
            return vec![];
        }
        let dom = self.domain(t);
        self.m
            .states
            .iter()
            .filter(|s| self.active[s.id] && match dom {
                Some(d) => self.ancestors(s.id).contains(&d),
                None => true,
            })
            .map(|s| s.id)
            .collect()
    }

    /// Select the optimal conflict-free set of enabled transitions for `ev`: per active leaf the first
    /// enabled transition from the leaf up through its ancestors (deeper preempts), document order, then
    /// transitions that exit the same states as an earlier one are dropped.
    fn select(&self, ev: Option<&Ev>) -> R<Option<Vec<usize>>> {
        let m = self.m;
        let leaves: Vec<usize> = m
            .states
            .iter()
            .filter(|s| self.active[s.id] && !self.children[s.id].iter().any(|c| self.active[*c]))
            .map(|s| s.id)
            .collect();
        let mut chosen: Vec<usize> = Vec::new();
        for leaf in leaves {
            let mut chain = vec![leaf];
            chain.extend(self.ancestors(leaf));
            for s in chain {
                let mut enabled: Vec<usize> = Vec::new();
                for tr in m.transitions.iter().filter(|t| t.source == s) {
                    let Some(scope) = self.matches(tr.id, ev) else { continue };
                    let ok = match &tr.guard {
                        None => true,
                        Some(g) => self.truth_rec(tr.id, 0, g, Some(&scope), &format!("transition t{} ({}) guard", tr.id, self.describe(tr.id)))?,
                    };
                    if ok {
                        enabled.push(tr.id);
                        if !self.opts.strict {
                            break;
                        }
                    }
                }
                if self.opts.strict {
                    if let Some(b) = enabled.iter().skip(1).find(|b| m.transitions[**b].target != m.transitions[enabled[0]].target) {
                        let a = enabled[0];
                        return Err(format!(
                            "strict: transitions t{a} ({}) and t{b} ({}) are both enabled with equal priority",
                            self.describe(a),
                            self.describe(*b)
                        ));
                    }
                }
                if let Some(t) = enabled.first() {
                    if !chosen.contains(t) {
                        chosen.push(*t);
                    }
                    break;
                }
            }
        }
        // SCXML removeConflictingTransitions: of two transitions that exit the same states, the one whose
        // source is the descendant wins; otherwise the earlier one.
        let mut kept: Vec<usize> = Vec::new();
        'next: for t in chosen {
            let ex = self.exit_set(t);
            let mut i = 0;
            while i < kept.len() {
                let k = kept[i];
                if self.exit_set(k).iter().any(|x| ex.contains(x)) {
                    let (ks, ts) = (m.transitions[k].source, m.transitions[t].source);
                    if self.ancestors(ks).contains(&ts) {
                        continue 'next; // k is deeper: it preempts t
                    } else if self.ancestors(ts).contains(&ks) {
                        kept.remove(i); // t is deeper: it preempts k
                        continue;
                    } else {
                        continue 'next;
                    }
                }
                i += 1;
            }
            kept.push(t);
        }
        Ok(if kept.is_empty() { None } else { Some(kept) })
    }

    fn describe(&self, t: usize) -> String {
        let tr = &self.m.transitions[t];
        let to = tr.target.map(|x| self.m.states[x].path.as_str()).unwrap_or("(internal)");
        format!("{} → {}", self.m.states[tr.source].path, to)
    }

    // ── microstep ───────────────────────────────────────────────────────────

    fn microstep(&mut self, sel: Vec<usize>, ev: Option<&Ev>, cause: String) -> R<()> {
        let m = self.m;
        let mut step = self.begin(cause);
        let before = self.vars.clone();
        for t in &sel {
            self.cov.borrow_mut().transitions.insert(*t);
            if let Some(l) = self.last_taken {
                if m.transitions[l].target == Some(m.transitions[*t].source) {
                    self.cov.borrow_mut().pairs.insert((l, *t));
                }
            }
            self.last_taken = Some(*t);
        }
        step.transition = sel.first().copied();
        step.taken = sel.clone();
        step.from = sel.first().map(|t| m.states[m.transitions[*t].source].path.clone());
        step.to = sel.first().and_then(|t| m.transitions[*t].target).map(|x| m.states[x].path.clone());
        if m.transitions[sel[0]].guard.is_some() {
            step.guard = Some(true);
        }
        let result = (|| -> R<()> {
            // exit
            let mut exits: Vec<usize> = sel.iter().flat_map(|t| self.exit_set(*t)).collect();
            exits.sort_unstable();
            exits.dedup();
            for s in exits.iter().rev() {
                self.exit_one(*s, &mut step)?;
            }
            // transition content, in document order
            for t in &sel {
                let tr = &m.transitions[*t];
                let (bind, payload): (Option<String>, &[(String, Value)]) = match (&tr.trigger, ev) {
                    (Trigger::Event { bind, .. }, Some(Ev::Named { payload, .. })) => (bind.clone(), payload),
                    _ => (None, &[]),
                };
                let scope = Scope { bind, payload };
                self.run_actions(&tr.actions, Some(&scope), &mut step)?;
            }
            // entry
            let mut entered_ids: Vec<usize> = Vec::new();
            for t in &sel {
                let tr = &m.transitions[*t];
                if tr.internal {
                    // an internal transition keeps its source; its timer / observation is re-armed
                    self.rearm(*t)?;
                    continue;
                }
                let Some(target) = tr.target else { continue };
                let mut on_path = self.ancestors(target);
                on_path.push(target);
                let dom = self.domain(*t);
                let top = on_path.iter().rev().find(|s| m.states[**s].parent == dom).copied().unwrap_or(target);
                let mut list = Vec::new();
                self.plan_enter(top, &on_path, &mut list);
                for s in list {
                    if !self.active[s] {
                        self.enter_one(s, &mut step)?;
                        entered_ids.push(s);
                    }
                }
            }
            self.arm(&entered_ids)?;
            Ok(())
        })();
        self.record_changes(&before, &mut step);
        self.trace.push(step);
        result
    }

    fn rearm(&mut self, t: usize) -> R<()> {
        let tr = &self.m.transitions[t];
        match &tr.trigger {
            Trigger::Timer { delay_seconds } => self.timers.push((self.time + delay_seconds, t)),
            Trigger::Change { cond } => {
                let v = self.truth(cond, None, &format!("transition t{t} `when`"))?;
                self.watches.push(Watch { trans: t, need_false: v, fired: false });
            }
            _ => {}
        }
        Ok(())
    }

    fn record_changes(&self, before: &[Value], step: &mut Step) {
        for (i, v) in self.vars.iter().enumerate() {
            if &before[i] != v {
                step.changed.push(Change { var: self.m.variables[i].name.clone(), old: format_value(&before[i]), new: format_value(v) });
            }
        }
    }

    fn plan_enter(&self, s: usize, on_path: &[usize], list: &mut Vec<usize>) {
        list.push(s);
        let kids = &self.children[s];
        match self.m.states[s].kind {
            Kind::Compound => {
                let next = kids.iter().find(|c| on_path.contains(c)).copied().or(self.m.states[s].initial);
                if let Some(c) = next {
                    self.plan_enter(c, on_path, list);
                }
            }
            Kind::Parallel => {
                for c in kids {
                    self.plan_enter(*c, on_path, list);
                }
            }
            _ => {}
        }
    }

    fn enter_one(&mut self, s: usize, step: &mut Step) -> R<()> {
        let m = self.m;
        self.active[s] = true;
        self.cov.borrow_mut().states.insert(s);
        step.entered.push(m.states[s].path.clone());
        let acts = m.states[s].entry.clone();
        self.run_actions(&acts, None, step)?;
        if m.states[s].kind == Kind::Final {
            match m.states[s].parent {
                None => self.terminated = true,
                Some(p) if m.states[p].parent.is_none() && m.parallel => {
                    self.internal.push_back(Ev::Done(p));
                    let all = m.states.iter().filter(|r| r.parent.is_none()).all(|r| self.children[r.id].iter().any(|c| self.active[*c] && m.states[*c].kind == Kind::Final));
                    if all {
                        self.terminated = true;
                    }
                }
                Some(p) => {
                    self.internal.push_back(Ev::Done(p));
                    if let Some(g) = m.states[p].parent {
                        if m.states[g].kind == Kind::Parallel {
                            let all = self.children[g].iter().all(|r| self.children[*r].iter().any(|c| self.active[*c] && m.states[*c].kind == Kind::Final));
                            if all {
                                self.internal.push_back(Ev::Done(g));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Arm the timers and change observations of just-entered states, after the whole entry set has run so
    /// that a `when` sees the variables as the entry actions left them (KerML: the observation starts when the
    /// accept becomes enabled).
    fn arm(&mut self, entered: &[usize]) -> R<()> {
        let m = self.m;
        for s in entered.iter().copied().filter(|s| self.active[*s]) {
        for tr in m.transitions.iter().filter(|t| t.source == s) {
            match &tr.trigger {
                Trigger::Timer { delay_seconds } => self.timers.push((self.time + delay_seconds, tr.id)),
                Trigger::Change { cond } => {
                    // KerML ObserveChange: already true when the observation starts means the signal is sent at once.
                    let v = self.truth_rec(tr.id, 1, cond, None, &format!("transition t{} `when`", tr.id))?;
                    if v {
                        self.internal.push_back(Ev::Signal(tr.id));
                    } else {
                        self.watches.push(Watch { trans: tr.id, need_false: false, fired: false });
                    }
                }
                _ => {}
            }
        }
        }
        Ok(())
    }

    fn exit_one(&mut self, s: usize, step: &mut Step) -> R<()> {
        let m = self.m;
        let acts = m.states[s].exit.clone();
        self.run_actions(&acts, None, step)?;
        self.active[s] = false;
        step.exited.push(m.states[s].path.clone());
        for t in self.timers.iter().filter(|t| m.transitions[t.1].source == s) {
            self.cov.borrow_mut().timers_cancelled.insert(t.1);
        }
        self.timers.retain(|t| m.transitions[t.1].source != s);
        self.watches.retain(|w| m.transitions[w.trans].source != s);
        Ok(())
    }

    /// Run an executable `ActionDef` to completion on the machine's variables of the same names: its `in` and
    /// `inout` parameters are read from them, its `out` and `inout` values written back.
    fn run_behavior(&mut self, prog: &crate::act_ir::ActionProgram, target: &str) -> R<String> {
        use crate::act_run::{ActOptions, ActRun, Status as AS};
        let mut ar = ActRun::new(prog, ActOptions::default());
        for slot in prog.defs[0].slots.iter().filter(|s| s.kind == "in" || s.kind == "inout") {
            if let Some(v) = self.var(&slot.name) {
                ar.set_input(&slot.name, v).map_err(|e| format!("behavior {target}: {e}"))?;
            }
        }
        ar.run();
        match ar.status() {
            AS::Error => return Err(format!("behavior {target}: {}", ar.error().unwrap_or(""))),
            AS::Waiting => return Err(format!("behavior {target}: the action waits for an event; a behaviour must complete")),
            _ => {}
        }
        for (name, v) in ar.outputs() {
            if let Some(i) = self.m.variables.iter().position(|x| x.name == name) {
                check_assignable(&name, &self.vars[i], &v).map_err(|e| format!("behavior {target}: {e}"))?;
                self.vars[i] = v;
            }
        }
        Ok(format!("action {target} executed in {} firing(s)", ar.trace().len()))
    }

    fn run_actions(&mut self, acts: &[Action], scope: Option<&Scope>, step: &mut Step) -> R<()> {
        for a in acts {
            match a {
                Action::Behavior { target, .. } => match self.m.behaviors.get(target) {
                    Some(prog) => {
                        let note = self.run_behavior(prog, target)?;
                        step.actions.push(note);
                    }
                    None => step.actions.push(format!("behavior {target}")),
                },
                Action::Assign { var, expr } => {
                    let name = self.m.variables[*var].name.clone();
                    let v = self.eval(expr, scope, &format!("assignment to '{name}'"))?;
                    check_assignable(&name, &self.vars[*var], &v)?;
                    step.actions.push(format!("{name} := {}", format_value(&v)));
                    self.vars[*var] = v;
                }
            }
        }
        Ok(())
    }
}

struct Scope<'a> {
    bind: Option<String>,
    payload: &'a [(String, Value)],
}

fn check_assignable(name: &str, old: &Value, new: &Value) -> R<()> {
    match (old, new) {
        (Value::Num(a), Value::Num(b)) if a.dim == b.dim => Ok(()),
        (Value::Num(a), Value::Num(b)) => Err(format!(
            "variable '{name}': cannot assign {} (dimension {:?}) to {} (dimension {:?})",
            format_value(new),
            b.dim.0,
            format_value(old),
            a.dim.0
        )),
        (Value::Bool(_), Value::Bool(_)) | (Value::Str(_), Value::Str(_)) => Ok(()),
        _ => Err(format!("variable '{name}': cannot assign a {} to a {}", new.kind(), old.kind())),
    }
}

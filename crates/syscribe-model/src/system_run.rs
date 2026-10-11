//! Executing a composite (REQ-TRS-SYSCOMP-001): every part with a state machine runs its own `Run`; messages travel along
//! the connections, values along the bindings, all on one virtual clock. No threads and no wall clock: every ordering is a
//! total order (part declaration order, then emission sequence), so a run is fully determined by the model and the script.

use std::collections::VecDeque;

use crate::element::RawElement;
use crate::expr::{format_value, Value};
use crate::ir::{self, Machine};
use crate::resolver::Resolver;
use crate::sm_run::{Options, Run, Step};
use crate::system::{self, Behavior, SystemSpec};

/// A composite resolved, checked and lowered: the wiring and one machine per part that has behaviour.
pub struct SystemModel {
    pub spec: SystemSpec,
    pub machines: Vec<Option<Machine>>,
    pub hash: String,
}

impl SystemModel {
    /// Resolve the composite `target` (a Part/PartDef). The errors are the `E930`–`E934` findings and any lowering
    /// diagnostic of a part's machine.
    pub fn build(elements: &[RawElement], target: &str) -> Result<SystemModel, Vec<String>> {
        let resolver = Resolver::new(elements);
        let Some(elem) = resolver.resolve_ref(elements, target) else { return Err(vec![format!("no element '{target}'")]) };
        let (spec, mut findings) = system::resolve(elements, &resolver, elem);
        let Some(spec) = spec else { return Err(vec![format!("'{}' is not a composite with an executable state machine", elem.qualified_name)]) };
        findings.extend(system::check(elements, &resolver, elem).into_iter().filter(|f| f.0 != "W936"));
        let mut errs: Vec<String> = findings.iter().filter(|f| f.0.starts_with('E')).map(|(c, m)| format!("{c}: {m}")).collect();
        errs.sort();
        errs.dedup();
        let mut machines = Vec::new();
        for i in &spec.instances {
            match &i.behavior {
                Behavior::Machine(q) => match ir::lower(elements, q) {
                    Ok(m) => machines.push(Some(m)),
                    Err(ds) => {
                        errs.extend(ds.into_iter().map(|(c, m)| format!("{c}: part '{}': {m}", i.name)));
                        machines.push(None);
                    }
                },
                Behavior::Plant(_) => {
                    errs.push(format!("part '{}' is an FMU plant; plants as parts are not supported yet", i.name));
                    machines.push(None);
                }
                Behavior::Inert => machines.push(None),
            }
        }
        if !errs.is_empty() {
            return Err(errs);
        }
        let hash = {
            let parts: Vec<_> = spec.instances.iter().zip(&machines).map(|(i, m)| serde_json::json!({"name": i.name, "machine": m.as_ref().map(|m| m.hash.clone())})).collect();
            let links: Vec<_> = spec.links.iter().map(|l| serde_json::json!([spec.instances[l.from.0].name, l.from.1, spec.instances[l.to.0].name, l.to.1, l.delay])).collect();
            let binds: Vec<_> = spec.bindings.iter().map(|b| serde_json::json!([spec.instances[b.from.0].name, b.from.1, spec.instances[b.to.0].name, b.to.1])).collect();
            blake3::hash(serde_json::json!({"parts": parts, "connections": links, "bindings": binds}).to_string().as_bytes()).to_hex().to_string()
        };
        Ok(SystemModel { spec, machines, hash })
    }
}

/// A message on its way, or delivered: the record the trace and `sent` assertions use.
#[derive(Debug, Clone)]
pub struct Message {
    pub seq: usize,
    pub sent: f64,
    pub due: f64,
    pub delivered: Option<f64>,
    pub from: usize,
    pub from_port: Option<String>,
    pub to: Option<(usize, String)>,
    pub event: String,
    pub payload: Vec<(String, Value)>,
}

const MICRO_STEP_LIMIT: usize = 100_000;
const BINDING_ROUNDS: usize = 1_000;

pub struct System<'m> {
    model: &'m SystemModel,
    runs: Vec<Option<Run<'m>>>,
    time: f64,
    queue: VecDeque<Message>,
    log: Vec<Message>,
    seq: usize,
    error: Option<String>,
}

impl<'m> System<'m> {
    /// Start every part's machine and run the system to quiescence at time 0.
    pub fn new(model: &'m SystemModel, opts: Options) -> System<'m> {
        let runs = model.machines.iter().map(|m| m.as_ref().map(|m| Run::new(m, opts.clone()))).collect();
        let mut s = System { model, runs, time: 0.0, queue: VecDeque::new(), log: Vec::new(), seq: 0, error: None };
        s.quiesce();
        s
    }

    pub fn time(&self) -> f64 {
        self.time
    }

    pub fn error(&self) -> Option<String> {
        self.error.clone().or_else(|| {
            self.runs.iter().zip(&self.model.spec.instances).find_map(|(r, i)| r.as_ref().and_then(|r| r.error().map(|e| format!("part '{}': {e}", i.name))))
        })
    }

    fn part(&self, name: &str) -> Result<usize, String> {
        self.model.spec.instance(name).ok_or_else(|| format!("no part '{name}' (parts: {})", self.model.spec.instances.iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join(", ")))
    }

    fn run(&self, name: &str) -> Result<&Run<'m>, String> {
        let i = self.part(name)?;
        self.runs[i].as_ref().ok_or_else(|| format!("part '{name}' has no state machine"))
    }

    pub fn active(&self, part: &str) -> Result<Vec<String>, String> {
        Ok(self.run(part)?.active())
    }

    pub fn holds_state(&self, part: &str, state: &str) -> Result<bool, String> {
        self.run(part)?.holds_state(state)
    }

    pub fn var(&self, part: &str, name: &str) -> Result<Value, String> {
        self.run(part)?.var(name).ok_or_else(|| format!("part '{part}' has no variable '{name}'"))
    }

    /// Force a part's machine into a state (a Given step).
    pub fn force(&mut self, part: &str, state: &str) {
        let i = match self.part(part) {
            Ok(i) => i,
            Err(e) => return self.fail(e),
        };
        let r = match self.runs[i].as_mut() {
            Some(r) => r.force(state),
            None => Err(format!("part '{part}' has no state machine")),
        };
        if let Err(e) = r {
            self.fail(format!("part '{part}': {e}"));
        }
    }

    /// Evaluate an expression over the variables of all parts, written `<part>.<variable>`.
    pub fn eval_qualified(&self, src: &str) -> Result<Value, String> {
        let ast = crate::expr::parse(src).map_err(|e| format!("`{src}`: {e}"))?;
        let lookup = |name: &str| {
            let (p, v) = name.split_once('.')?;
            self.var(p, v).ok()
        };
        ast.eval(&lookup).map_err(|e| format!("`{src}`: {e}"))
    }

    /// Evaluate an expression in one part's variables.
    pub fn eval(&self, part: &str, src: &str) -> Result<Value, String> {
        self.run(part)?.eval_str(src)
    }

    /// Every message sent so far (delivered or not), in emission order.
    pub fn messages(&self) -> &[Message] {
        &self.log
    }

    /// The per-part traces, each step tagged with its part.
    pub fn trace(&self) -> Vec<(String, Step)> {
        let mut out: Vec<(String, Step)> = Vec::new();
        for (r, i) in self.runs.iter().zip(&self.model.spec.instances) {
            if let Some(r) = r {
                out.extend(r.trace().iter().map(|s| (i.name.clone(), s.clone())));
            }
        }
        out.sort_by(|a, b| a.1.time.partial_cmp(&b.1.time).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    /// Inject an event into one part from outside, through `port` when given, and run to quiescence.
    pub fn receive(&mut self, part: &str, port: Option<&str>, event: &str, payload: &[(&str, Value)]) {
        if self.error().is_some() {
            return;
        }
        let i = match self.part(part) {
            Ok(i) => i,
            Err(e) => return self.fail(e),
        };
        let name = port.map(|p| format!("{p}/{event}")).unwrap_or_else(|| event.to_string());
        match self.runs[i].as_mut() {
            Some(r) => r.send(&name, payload),
            None => return self.fail(format!("part '{part}' has no state machine")),
        }
        self.quiesce();
    }

    /// Set a variable of one part from outside and run to quiescence.
    pub fn set(&mut self, part: &str, name: &str, value: Value) {
        if self.error().is_some() {
            return;
        }
        match self.part(part).map(|i| self.runs[i].as_mut()) {
            Ok(Some(r)) => r.set(name, value),
            Ok(None) => return self.fail(format!("part '{part}' has no state machine")),
            Err(e) => return self.fail(e),
        }
        self.quiesce();
    }

    fn fail(&mut self, e: String) {
        self.error.get_or_insert(e);
    }

    /// Advance the shared clock by `secs`, stopping at every instant at which anything is due.
    pub fn advance(&mut self, secs: f64) {
        let target = self.time + secs;
        while self.time < target && self.error().is_none() {
            let timers = self.runs.iter().flatten().filter_map(|r| r.next_timer());
            let due = self.queue.iter().map(|m| m.due).chain(timers).filter(|d| *d > self.time).fold(f64::INFINITY, f64::min);
            let t = due.min(target);
            for r in self.runs.iter_mut().flatten() {
                r.advance_to(t);
            }
            self.time = t;
            self.quiesce();
        }
    }

    /// Process everything that is due at the current instant until nothing changes.
    fn quiesce(&mut self) {
        let mut micro = 0usize;
        let mut rounds = 0usize;
        loop {
            if self.error().is_some() {
                return;
            }
            self.collect();
            let tol = 1e-12_f64.max(self.time.abs() * 1e-15);
            let next = self.queue.iter().enumerate().filter(|(_, m)| m.due <= self.time + tol).min_by(|a, b| a.1.due.partial_cmp(&b.1.due).unwrap_or(std::cmp::Ordering::Equal).then(a.1.seq.cmp(&b.1.seq))).map(|(i, _)| i);
            if let Some(pos) = next {
                micro += 1;
                if micro > MICRO_STEP_LIMIT {
                    return self.fail(format!("livelock: more than {MICRO_STEP_LIMIT} messages at t = {} s; the parts keep answering each other ({})", self.time, self.chatter()));
                }
                let m = self.queue.remove(pos).expect("found");
                self.deliver(m);
                continue;
            }
            if self.propagate() {
                rounds += 1;
                if rounds > BINDING_ROUNDS {
                    return self.fail(format!("bindings do not converge at t = {} s after {BINDING_ROUNDS} rounds", self.time));
                }
                continue;
            }
            break;
        }
    }

    fn chatter(&self) -> String {
        let mut names: Vec<String> = self.log.iter().rev().take(6).map(|m| format!("{} → {}", self.model.spec.instances[m.from].name, m.to.as_ref().map(|t| self.model.spec.instances[t.0].name.clone()).unwrap_or_default())).collect();
        names.dedup();
        names.join(", ")
    }

    /// Move the messages the parts have sent into the delivery queue, one copy per connection (fan-out in connection order).
    fn collect(&mut self) {
        let model = self.model;
        for from in 0..self.runs.len() {
            let sent = match self.runs[from].as_mut() {
                Some(r) => r.take_outbox(),
                None => continue,
            };
            for s in sent {
                let links: Vec<_> = model.spec.links.iter().filter(|l| l.from.0 == from && s.via.as_deref() == Some(l.from.1.as_str())).collect();
                let mk = |seq: usize, to: Option<(usize, String)>, delay: f64| Message { seq, sent: self.time, due: self.time + delay, delivered: None, from, from_port: s.via.clone(), to, event: s.event.clone(), payload: s.payload.clone() };
                if links.is_empty() {
                    // nothing is connected to that port: the message is recorded, and goes nowhere
                    self.seq += 1;
                    let m = mk(self.seq, None, 0.0);
                    self.log.push(m);
                    continue;
                }
                for l in links {
                    self.seq += 1;
                    let m = mk(self.seq, Some((l.to.0, l.to.1.clone())), l.delay);
                    self.log.push(m.clone());
                    self.queue.push_back(m);
                }
            }
        }
    }

    fn deliver(&mut self, m: Message) {
        let Some((to, port)) = m.to.clone() else { return };
        let payload: Vec<(&str, Value)> = m.payload.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        if let Some(r) = self.runs[to].as_mut() {
            r.send(&format!("{port}/{}", m.event), &payload);
        }
        // every sequence number is one log entry, in order
        if let Some(l) = self.log.get_mut(m.seq - 1) {
            l.delivered = Some(self.time);
        }
    }

    /// One pass over the bindings in order: copy each source value to its sink when they differ. Returns whether any sink changed.
    fn propagate(&mut self) -> bool {
        let model = self.model;
        let mut changed = false;
        for b in &model.spec.bindings {
            let Some(v) = self.runs[b.from.0].as_ref().and_then(|r| r.var(&b.from.1)) else { continue };
            // compared as text so that NaN equals NaN and a halted sink cannot keep the loop spinning
            let same = |a: &Value, b: &Value| format_value(a) == format_value(b);
            let Some(r) = self.runs[b.to.0].as_mut() else { continue };
            let Some(cur) = r.var(&b.to.1) else { continue };
            if r.is_halted() || same(&cur, &v) {
                continue;
            }
            r.set(&b.to.1, v);
            if r.var(&b.to.1).is_some_and(|now| !same(&now, &cur)) {
                changed = true;
            }
        }
        changed
    }

    /// Per-message text for traces and failures.
    pub fn describe(&self, m: &Message) -> String {
        let spec = &self.model.spec;
        let pl = m.payload.iter().map(|(k, v)| format!("{k}={}", format_value(v))).collect::<Vec<_>>().join(", ");
        format!(
            "{} sent {}{}{} at {} s{}",
            spec.instances[m.from].name,
            m.event,
            m.from_port.as_ref().map(|p| format!(" via {p}")).unwrap_or_default(),
            if pl.is_empty() { String::new() } else { format!(" ({pl})") },
            m.sent,
            match (&m.to, m.delivered) {
                (Some((t, p)), Some(d)) => format!(", delivered to {}.{p} at {d} s", spec.instances[*t].name),
                (Some((t, p)), None) => format!(", to {}.{p}, not yet delivered", spec.instances[*t].name),
                (None, _) => ", connected to nothing".into(),
            }
        )
    }
}

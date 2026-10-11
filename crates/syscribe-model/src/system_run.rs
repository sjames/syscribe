//! Executing a composite (REQ-TRS-SYSCOMP-001): every part with a state machine runs its own `Run`; messages travel along
//! the connections, values along the bindings, all on one virtual clock. No threads and no wall clock: every ordering is a
//! total order (part declaration order, then emission sequence), so a run is fully determined by the model and the script.

use std::collections::VecDeque;

use std::path::{Path, PathBuf};

use crate::element::RawElement;
use crate::expr::{format_value, Value};
use crate::ir::{self, Machine};
use crate::plant::{self, FmuInfo, Plant, VarInfo};
use crate::resolver::Resolver;
use crate::units::Dim;
use crate::sm_run::{Options, Run, Step};
use crate::system::{self, Behavior, SystemSpec};

/// An FMU plant part, resolved: its file, declared hash, parameters and what the file says about itself.
#[derive(Clone)]
pub struct PlantSpec {
    pub file: PathBuf,
    pub sha256: Option<String>,
    pub params: Vec<(String, f64)>,
    pub info: FmuInfo,
}

/// A binding between a plant variable and a machine variable, checked.
#[derive(Clone)]
pub enum Exchange {
    /// A plant output read into a machine variable at each communication point.
    Read { plant: usize, var: VarInfo, sink: (usize, String), machine_bool: bool, machine_dim: Dim },
    /// A machine variable written to a plant input before each step.
    Write { plant: usize, var: VarInfo, src: (usize, String) },
}

/// A composite resolved, checked and lowered: the wiring and one machine per part that has behaviour.
#[derive(Clone)]
pub struct SystemModel {
    pub spec: SystemSpec,
    pub machines: Vec<Option<Machine>>,
    pub plants: Vec<Option<PlantSpec>>,
    pub exchanges: Vec<Exchange>,
    /// The communication step of the plants, when there are any.
    pub plant_step: Option<f64>,
    pub hash: String,
}

impl SystemModel {
    /// Resolve the composite `target` (a Part/PartDef). The errors are the `E930`–`E934` findings and any lowering
    /// diagnostic of a part's machine.
    pub fn build(elements: &[RawElement], target: &str) -> Result<SystemModel, Vec<String>> {
        Self::build_in(elements, target, Path::new("."))
    }

    /// [`build`](Self::build) with the model root against which the FMU files of plant parts are resolved.
    pub fn build_in(elements: &[RawElement], target: &str, model_root: &Path) -> Result<SystemModel, Vec<String>> {
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
                Behavior::Plant(_) => machines.push(None),
                Behavior::Inert => machines.push(None),
            }
        }
        let mut plants: Vec<Option<PlantSpec>> = spec.instances.iter().map(|_| None).collect();
        let mut exchanges = Vec::new();
        let mut plant_step = None;
        if spec.instances.iter().any(|i| matches!(i.behavior, Behavior::Plant(_))) {
            match resolve_plants(elements, &resolver, elem, &spec, &machines, model_root) {
                Ok((p, ex, step)) => (plants, exchanges, plant_step) = (p, ex, Some(step)),
                Err(es) => errs.extend(es),
            }
        }
        if !errs.is_empty() {
            return Err(errs);
        }
        let hash = {
            let parts: Vec<_> = spec.instances.iter().zip(&machines).map(|(i, m)| serde_json::json!({"name": i.name, "machine": m.as_ref().map(|m| m.hash.clone())})).collect();
            let links: Vec<_> = spec.links.iter().map(|l| serde_json::json!([spec.instances[l.from.0].name, l.from.1, spec.instances[l.to.0].name, l.to.1, l.delay])).collect();
            let binds: Vec<_> = spec.bindings.iter().map(|b| serde_json::json!([spec.instances[b.from.0].name, b.from.1, spec.instances[b.to.0].name, b.to.1])).collect();
            let plant_cfg: Vec<_> = spec.instances.iter().zip(&plants).filter_map(|(i, p)| p.as_ref().map(|p| serde_json::json!({"name": i.name, "sha256": p.info.sha256, "fmiVersion": p.info.fmi_version, "parameters": p.params}))).collect();
            blake3::hash(serde_json::json!({"parts": parts, "connections": links, "bindings": binds, "plantStep": plant_step, "plants": plant_cfg}).to_string().as_bytes()).to_hex().to_string()
        };
        Ok(SystemModel { spec, machines, plants, exchanges, plant_step, hash })
    }
}

/// The plant parts of a composite: their FMU files (verified against the declared hash), the communication step, and
/// every binding between a plant variable and a machine variable, checked as a TestCase-level plant connection is.
#[allow(clippy::type_complexity)]
fn resolve_plants(elements: &[RawElement], resolver: &Resolver, composite: &RawElement, spec: &SystemSpec, machines: &[Option<Machine>], model_root: &Path) -> Result<(Vec<Option<PlantSpec>>, Vec<Exchange>, f64), Vec<String>> {
    use crate::config::{SourceLocation, ValidateConfig};
    use crate::plant_cfg::parse_fmu;
    let Some(be) = plant::backend() else {
        return Err(vec!["this composite has plant parts but this build has no FMI support (rebuild with `cargo build --features fmi`)".to_string()]);
    };
    let mut errs = Vec::new();
    let step = match composite.frontmatter.plant_step.as_ref().and_then(|v| match v {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        _ => None,
    }) {
        Some(s) => crate::system::duration(&s).filter(|d| *d > 0.0).ok_or_else(|| vec![format!("`plantStep: {s}` is not a positive duration")])?,
        None => return Err(vec![format!("{}: a composite with plant parts needs a `plantStep:` (the communication step)", composite.qualified_name)]),
    };
    let cfg = ValidateConfig::with_model_root(model_root);
    let mut plants: Vec<Option<PlantSpec>> = spec.instances.iter().map(|_| None).collect();
    for (idx, inst) in spec.instances.iter().enumerate() {
        let Behavior::Plant(def) = &inst.behavior else { continue };
        let Some(d) = resolver.resolve_ref(elements, def) else { continue };
        let decl = match d.frontmatter.fmu.as_ref().map(parse_fmu) {
            Some(Ok(decl)) => decl,
            Some(Err(es)) => {
                errs.extend(es.into_iter().map(|e| format!("E888: part '{}': {e}", inst.name)));
                continue;
            }
            None => continue,
        };
        let file = match cfg.classify_source(&decl.file) {
            SourceLocation::Local(f) => f,
            SourceLocation::Remote(u) => {
                errs.push(format!("E966: part '{}': the FMU '{u}' is remote; only local files are supported", inst.name));
                continue;
            }
        };
        let info = match be.inspect(&file) {
            Ok(i) => i,
            Err(e) => {
                errs.push(format!("E966: part '{}': {}: {e}", inst.name, file.display()));
                continue;
            }
        };
        if let Some(h) = &decl.sha256 {
            if !h.eq_ignore_ascii_case(&info.sha256) {
                errs.push(format!("E966: part '{}': declared sha256 {h} but the file hashes to {} (the FMU changed)", inst.name, info.sha256));
                continue;
            }
        }
        match decl.raw_parameters(&info) {
            Ok(params) => plants[idx] = Some(PlantSpec { file, sha256: decl.sha256.clone(), params, info }),
            Err(es) => errs.extend(es.into_iter().map(|e| format!("E888: part '{}': {e}", inst.name))),
        }
    }
    let mut exchanges = Vec::new();
    for b in &spec.bindings {
        let (fp, tp) = (matches!(spec.instances[b.from.0].behavior, Behavior::Plant(_)), matches!(spec.instances[b.to.0].behavior, Behavior::Plant(_)));
        let label = format!("binding {}.{} → {}.{}", spec.instances[b.from.0].name, b.from.1, spec.instances[b.to.0].name, b.to.1);
        let ((pi, pvar), (mi, mvar), to_plant) = match (fp, tp) {
            (true, true) => {
                errs.push(format!("E933: {label}: both ends are plants; one end must be a machine variable"));
                continue;
            }
            (true, false) => ((b.from.0, &b.from.1), (b.to.0, &b.to.1), false),
            (false, true) => ((b.to.0, &b.to.1), (b.from.0, &b.from.1), true),
            (false, false) => continue,
        };
        let Some(ps) = &plants[pi] else { continue };
        let pname = &spec.instances[pi].name;
        let Some(var) = ps.info.var(pvar) else {
            errs.push(format!("E933: {label}: plant '{pname}' has no variable '{pvar}'"));
            continue;
        };
        let Some(mv) = machines[mi].as_ref().and_then(|m| m.variables.iter().find(|v| &v.name == mvar)) else { continue }; // reported by the composite checks
        match plant::check_exchange(pname, var, to_plant, mvar, &mv.value) {
            Ok((machine_dim, machine_bool)) => exchanges.push(if to_plant {
                Exchange::Write { plant: pi, var: var.clone(), src: (mi, mvar.clone()) }
            } else {
                Exchange::Read { plant: pi, var: var.clone(), sink: (mi, mvar.clone()), machine_bool, machine_dim }
            }),
            Err(e) => errs.push(format!("E933: {label}: {e}")),
        }
    }
    if errs.is_empty() {
        Ok((plants, exchanges, step))
    } else {
        Err(errs)
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
/// All messages one run may send: a bound on loops that delays spread over time (the per-instant limit cannot see them).
const MESSAGE_BUDGET: usize = 250_000;

pub struct System<'m> {
    model: &'m SystemModel,
    runs: Vec<Option<Run<'m>>>,
    time: f64,
    queue: VecDeque<Message>,
    log: Vec<Message>,
    seq: usize,
    error: Option<String>,
    plants: Vec<Option<Box<dyn Plant>>>,
}

impl<'m> System<'m> {
    /// Start every part's machine and run the system to quiescence at time 0.
    pub fn new(model: &'m SystemModel, opts: Options) -> System<'m> {
        let mut s = System { model, runs: Vec::new(), time: 0.0, queue: VecDeque::new(), log: Vec::new(), seq: 0, error: None, plants: Vec::new() };
        s.start_plants();
        // the plants' initial outputs are known before any machine starts: they are the machines' starting values
        let mut initial: Vec<Vec<(String, Value)>> = model.machines.iter().map(|_| Vec::new()).collect();
        if s.error.is_none() {
            for ex in &model.exchanges {
                if let Exchange::Read { plant: pi, var, sink, machine_bool, machine_dim } = ex {
                    let Some(p) = s.plants.get_mut(*pi).and_then(|p| p.as_mut()) else { continue };
                    match plant::read_si(p.as_mut(), &model.spec.instances[*pi].name, var, *machine_bool, *machine_dim) {
                        Ok(v) => initial[sink.0].push((sink.1.clone(), v)),
                        Err(e) => {
                            s.fail(e);
                            break;
                        }
                    }
                }
            }
        }
        s.runs = model.machines.iter().zip(&initial).map(|(m, init)| m.as_ref().map(|m| Run::with_initial_values(m, opts.clone(), init))).collect();
        s.quiesce();
        s
    }

    /// Fresh plant instances for this run; every instantiation verifies the declared hash again.
    fn start_plants(&mut self) {
        let model = self.model;
        for (i, p) in model.plants.iter().enumerate() {
            let Some(ps) = p else {
                self.plants.push(None);
                continue;
            };
            let name = &model.spec.instances[i].name;
            let made = (|| -> Result<Box<dyn Plant>, String> {
                let be = plant::backend().ok_or("no FMI backend")?;
                if let Some(h) = &ps.sha256 {
                    let now = be.inspect(&ps.file)?;
                    if !h.eq_ignore_ascii_case(&now.sha256) {
                        return Err(format!("declared sha256 {h} but the file hashes to {} (the FMU changed)", now.sha256));
                    }
                }
                be.instantiate(&ps.file, &ps.params)
            })();
            match made {
                Ok(p) => self.plants.push(Some(p)),
                Err(e) => {
                    self.plants.push(None);
                    self.fail(format!("plant '{name}': {e}"));
                }
            }
        }
    }

    /// What identifies each plant of the run, for recorded evidence.
    pub fn plant_provenance(&self) -> Vec<serde_json::Value> {
        let step = self.model.plant_step;
        self.plants.iter().zip(&self.model.spec.instances).filter_map(|(p, i)| p.as_ref().map(|p| serde_json::json!({"name": i.name, "step": step, "plant": p.provenance()}))).collect()
    }

    /// Write the machine variables bound to plant inputs (held constant over the coming step).
    fn write_plants(&mut self) {
        let model = self.model;
        for ex in &model.exchanges {
            if let Exchange::Write { plant: pi, var, src } = ex {
                let Some(v) = self.runs[src.0].as_ref().and_then(|r| r.var(&src.1)) else { continue };
                let Some(p) = self.plants[*pi].as_mut() else { continue };
                if let Err(e) = plant::write_si(p.as_mut(), &model.spec.instances[*pi].name, var, &v) {
                    return self.fail(e);
                }
            }
        }
    }

    fn step_plants(&mut self, t: f64, h: f64) {
        let model = self.model;
        for (i, p) in self.plants.iter_mut().enumerate() {
            if let Some(p) = p {
                if let Err(e) = p.step(t, h) {
                    return self.fail(format!("plant '{}': {e}", model.spec.instances[i].name));
                }
                if p.terminated() {
                    return self.fail(format!("plant '{}' asked to terminate the simulation", model.spec.instances[i].name));
                }
            }
        }
    }

    /// Read the plant outputs at communication point `t` and hand them to the machines bound to them, all together and
    /// without settling: the next `advance_to` makes them visible at once, so a guard never sees half an update.
    fn read_plants(&mut self, t: f64) {
        let model = self.model;
        let mut staged: Vec<Vec<(String, Value)>> = model.machines.iter().map(|_| Vec::new()).collect();
        for ex in &model.exchanges {
            if let Exchange::Read { plant: pi, var, sink, machine_bool, machine_dim } = ex {
                let Some(p) = self.plants.get_mut(*pi).and_then(|p| p.as_mut()) else { continue };
                match plant::read_si(p.as_mut(), &model.spec.instances[*pi].name, var, *machine_bool, *machine_dim) {
                    Ok(v) => staged[sink.0].push((sink.1.clone(), v)),
                    Err(e) => return self.fail(e),
                }
            }
        }
        for (r, vals) in self.runs.iter_mut().zip(&staged) {
            if let (Some(r), false) = (r.as_mut(), vals.is_empty()) {
                r.stage_values(t, vals);
            }
        }
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

    /// What each part's machine has exercised so far.
    pub fn part_coverage(&self) -> Vec<crate::sm_scen::PartCoverage> {
        self.runs
            .iter()
            .zip(&self.model.spec.instances)
            .filter_map(|(r, i)| match (r, &i.behavior) {
                (Some(r), Behavior::Machine(q)) => Some(crate::sm_scen::PartCoverage { part: i.name.clone(), machine: q.clone(), coverage: r.coverage() }),
                _ => None,
            })
            .collect()
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
        let h = self.model.plant_step;
        if let Some(h) = h {
            // a step this small (or a run this long) would take effectively forever, and float grid arithmetic degrades
            if !(secs / h <= 5e6 && target / h < 9e15) {
                return self.fail(format!("{secs} s at a plant step of {h} s is more than 5,000,000 communication steps; use a larger `plantStep`"));
            }
        }
        while self.time < target - 1e-12 && self.error().is_none() {
            let timers = self.runs.iter().flatten().filter_map(|r| r.next_timer());
            let grid = h.map(|h| ((self.time / h + 1e-9).floor() + 1.0) * h);
            let due = self.queue.iter().map(|m| m.due).chain(timers).chain(grid).filter(|d| *d > self.time).fold(f64::INFINITY, f64::min);
            let t = due.min(target);
            if h.is_some() {
                self.write_plants();
                self.step_plants(self.time, t - self.time);
                if self.error().is_some() {
                    return;
                }
            }
            // the outputs of the step are visible to the machines before any timer due at `t` fires
            if h.is_some() {
                self.read_plants(t);
            }
            for r in self.runs.iter_mut().flatten() {
                r.advance_to(t);
            }
            self.time = t;
            self.quiesce();
        }
        if self.time < target && self.error().is_none() {
            // less than the loop's tolerance remains
            for r in self.runs.iter_mut().flatten() {
                r.advance_to(target);
            }
            self.time = target;
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
            if self.seq > MESSAGE_BUDGET {
                return self.fail(format!("more than {MESSAGE_BUDGET} messages in one run (at t = {} s); the parts keep answering each other ({})", self.time, self.chatter()));
            }
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

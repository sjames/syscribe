//! System-level composition (REQ-TRS-SYSCOMP-001): the parts of a composite `Part`/`PartDef`, the behaviour each
//! carries, and the connections and bindings between them. This module resolves and checks; `system_run` executes.

use std::collections::BTreeSet;

use serde_yaml::{Mapping, Value as Y};

use crate::element::{ElementType, RawElement};
use crate::expr::{self, Value};
use crate::ir::{self, Action, Machine, Trigger};
use crate::resolver::Resolver;
use crate::sm_exec::{field, text};

pub type Finding = (&'static str, String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    In,
    Out,
    InOut,
}

impl Dir {
    fn parse(s: Option<&str>) -> Dir {
        match s.unwrap_or("inout") {
            "in" => Dir::In,
            "out" => Dir::Out,
            _ => Dir::InOut,
        }
    }
    pub fn sends(self) -> bool {
        matches!(self, Dir::Out | Dir::InOut)
    }
    pub fn receives(self) -> bool {
        matches!(self, Dir::In | Dir::InOut)
    }
}

#[derive(Debug, Clone)]
pub struct PortSpec {
    pub name: String,
    pub dir: Dir,
}

#[derive(Debug, Clone)]
pub enum Behavior {
    /// The qualified name of the `StateDef`/`State` the part exhibits.
    Machine(String),
    /// The part is an FMU plant (REQ-TRS-FMI-001); the qualified name of the part definition carrying `fmu:`.
    Plant(String),
    Inert,
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub name: String,
    pub def: String,
    pub behavior: Behavior,
    pub ports: Vec<PortSpec>,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub from: (usize, String),
    pub to: (usize, String),
    pub delay: f64,
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub from: (usize, String),
    pub to: (usize, String),
}

#[derive(Debug, Clone)]
pub struct SystemSpec {
    pub composite: String,
    pub instances: Vec<Instance>,
    pub links: Vec<Link>,
    pub bindings: Vec<Binding>,
}

impl SystemSpec {
    pub fn instance(&self, name: &str) -> Option<usize> {
        self.instances.iter().position(|i| i.name == name)
    }
}

fn names(v: &Y) -> Vec<String> {
    match v {
        Y::String(s) => vec![s.clone()],
        Y::Sequence(xs) => xs.iter().filter_map(|x| x.as_str().map(String::from)).collect(),
        _ => Vec::new(),
    }
}

/// `def` and its supertypes, nearest first.
pub(crate) fn chain<'a>(elements: &'a [RawElement], resolver: &Resolver, def: &'a RawElement) -> Vec<&'a RawElement> {
    chain_with(elements, resolver, def, true)
}

/// [`chain`], optionally following only `supertype` (a composite inherits from its supertypes, not from what it is typed by).
pub fn chain_with<'a>(elements: &'a [RawElement], resolver: &Resolver, def: &'a RawElement, follow_typed_by: bool) -> Vec<&'a RawElement> {
    let mut out: Vec<&RawElement> = vec![def];
    let mut i = 0;
    while i < out.len() && out.len() < 32 {
        let fm = &out[i].frontmatter;
        let supers: Vec<String> = [fm.supertype.as_ref(), if follow_typed_by { fm.typed_by.as_ref() } else { None }].into_iter().flatten().flat_map(names).collect();
        for s in supers {
            if let Some(e) = resolver.resolve_ref(elements, &s) {
                if !out.iter().any(|x| x.qualified_name == e.qualified_name) {
                    out.push(e);
                }
            }
        }
        i += 1;
    }
    out
}

fn is_port(m: &Mapping) -> bool {
    field(m, "type").and_then(|t| t.as_str()) == Some("Port")
}

pub(crate) fn duration(src: &str) -> Option<f64> {
    match expr::parse(src).and_then(|e| e.eval(&|_: &str| None)) {
        Ok(Value::Num(q)) if Some(q.dim) == expr::unit_scale("s").map(|x| x.1) && q.v >= 0.0 && q.v.is_finite() => Some(q.v),
        _ => None,
    }
}

pub(crate) fn endpoint(s: &str) -> Option<(&str, &str)> {
    let (a, b) = s.split_once('.')?;
    (!a.is_empty() && !b.is_empty() && !b.contains('.')).then_some((a, b))
}

/// A composite as authored plus what it inherits: `features:` (parts and ports), `connections:`, `bindingConnections:` and
/// `plantStep:` merged along the supertype chain. A feature of the same name is overridden by the nearest definition;
/// the base's connections and bindings come first, and an identical entry is kept once.
pub struct CompositeView {
    pub features: Vec<Y>,
    pub connections: Vec<Y>,
    pub bindings: Vec<Y>,
    pub plant_step: Option<Y>,
}

pub fn composite_view(elements: &[RawElement], resolver: &Resolver, elem: &RawElement) -> CompositeView {
    let ch = chain_with(elements, resolver, elem, false);
    let mut v = CompositeView { features: Vec::new(), connections: Vec::new(), bindings: Vec::new(), plant_step: None };
    for d in ch.iter().rev() {
        for f in d.frontmatter.features.iter().flatten() {
            let name = f.as_mapping().and_then(|m| field(m, "name")).and_then(|n| n.as_str());
            match name.and_then(|n| v.features.iter().position(|x| x.as_mapping().and_then(|m| field(m, "name")).and_then(|n| n.as_str()) == Some(n))) {
                Some(i) => v.features[i] = f.clone(),
                None => v.features.push(f.clone()),
            }
        }
        // A connection is identified by its ends and a binding by its sink: an entry for the same key in a subtype replaces
        // the base's in place (so a variant can change a `delay:` or re-source a variable). The same entry written twice in
        // one definition stays twice (E933).
        let key = |c: &Y, a: &str, b: &str| -> Option<(String, String)> { c.as_mapping().and_then(|m| Some((field(m, a).and_then(text)?, field(m, b).and_then(text)?))) };
        let (base_conns, base_binds) = (v.connections.len(), v.bindings.len());
        for c in d.frontmatter.connections.iter().flatten() {
            match key(c, "from", "to").and_then(|k| v.connections[..base_conns].iter().position(|x| key(x, "from", "to").as_ref() == Some(&k))) {
                Some(i) => v.connections[i] = c.clone(),
                None => v.connections.push(c.clone()),
            }
        }
        for c in d.frontmatter.binding_connections.iter().flatten() {
            let sink = |x: &Y| x.as_mapping().and_then(|m| field(m, "to")).and_then(text);
            match sink(c).and_then(|k| v.bindings[..base_binds].iter().position(|x| sink(x).as_ref() == Some(&k))) {
                Some(i) => v.bindings[i] = c.clone(),
                None => v.bindings.push(c.clone()),
            }
        }
    }
    v.plant_step = ch.iter().find_map(|d| d.frontmatter.plant_step.clone());
    v
}

/// Every port of a part definition: inline `type: Port` features and child `Port`
/// elements along its definition chain, nearest first (GH #330).
pub fn def_ports(elements: &[RawElement], resolver: &Resolver, def: &RawElement) -> Vec<PortSpec> {
    let mut inst = [Instance { name: String::new(), def: def.qualified_name.clone(), behavior: Behavior::Inert, ports: Vec::new() }];
    for d in chain(elements, resolver, def) {
        for p in inline_ports(d) {
            if !inst[0].ports.iter().any(|x| x.name == p.name) {
                inst[0].ports.push(p);
            }
        }
    }
    add_child_ports(elements, resolver, &mut inst);
    let [i] = inst;
    i.ports
}

/// Child `Port` elements by the qualified name of their parent, built once for callers that ask for many definitions
/// ports ([`def_ports_in`]).
pub fn child_port_index(elements: &[RawElement]) -> std::collections::HashMap<String, Vec<PortSpec>> {
    let mut by_parent: std::collections::HashMap<String, Vec<PortSpec>> = std::collections::HashMap::new();
    for e in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Port))) {
        if let Some((parent, last)) = e.qualified_name.rsplit_once("::") {
            by_parent.entry(parent.to_string()).or_default().push(PortSpec { name: last.to_string(), dir: Dir::parse(e.frontmatter.direction.as_deref()) });
        }
    }
    by_parent
}

/// [`def_ports`] with the child-port index built once by [`child_port_index`].
pub fn def_ports_in(elements: &[RawElement], resolver: &Resolver, def: &RawElement, index: &std::collections::HashMap<String, Vec<PortSpec>>) -> Vec<PortSpec> {
    let ch = chain(elements, resolver, def);
    let mut ports: Vec<PortSpec> = Vec::new();
    for d in &ch {
        for p in inline_ports(d) {
            if !ports.iter().any(|x| x.name == p.name) {
                ports.push(p);
            }
        }
    }
    for d in &ch {
        for p in index.get(d.qualified_name.as_str()).into_iter().flatten() {
            if !ports.iter().any(|x| x.name == p.name) {
                ports.push(p.clone());
            }
        }
    }
    ports
}

/// The ports a part definition declares inline: `features:` of `type: Port`.
fn inline_ports(d: &RawElement) -> Vec<PortSpec> {
    let mut ports: Vec<PortSpec> = Vec::new();
    for pf in d.frontmatter.features.iter().flatten() {
        let Some(pm) = pf.as_mapping() else { continue };
        if let (true, Some(pn)) = (is_port(pm), field(pm, "name").and_then(|n| n.as_str())) {
            ports.push(PortSpec { name: pn.to_string(), dir: Dir::parse(field(pm, "direction").and_then(|d| d.as_str())) });
        }
    }
    ports
}

/// Add the child `Port` elements (`Def::out`, named by the identity segment) of each part's definition chain. Done once per
/// composite and only for executable ones, since it scans the model.
fn add_child_ports(elements: &[RawElement], resolver: &Resolver, instances: &mut [Instance]) {
    let mut by_parent: std::collections::HashMap<&str, Vec<(&str, Dir)>> = std::collections::HashMap::new();
    for e in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Port))) {
        if let Some((parent, last)) = e.qualified_name.rsplit_once("::") {
            by_parent.entry(parent).or_default().push((last, Dir::parse(e.frontmatter.direction.as_deref())));
        }
    }
    if by_parent.is_empty() {
        return;
    }
    for inst in instances {
        let Some(def) = resolver.resolve_ref(elements, &inst.def) else { continue };
        for d in chain(elements, resolver, def) {
            for (name, dir) in by_parent.get(d.qualified_name.as_str()).into_iter().flatten() {
                if !inst.ports.iter().any(|p| p.name == *name) {
                    inst.ports.push(PortSpec { name: name.to_string(), dir: *dir });
                }
            }
        }
    }
}

/// The parts among `features` (the composite's own and inherited ones) and what they carry.
pub fn parts_of(elements: &[RawElement], resolver: &Resolver, elem: &RawElement, features: &[Y], out: &mut Vec<Finding>) -> Vec<Instance> {
    let mut instances: Vec<Instance> = Vec::new();
    for (i, f) in features.iter().enumerate() {
        let Some(m) = f.as_mapping() else { continue };
        if is_port(m) {
            continue;
        }
        let Some(typed) = field(m, "typedBy").and_then(|t| t.as_str()) else { continue };
        let Some(name) = field(m, "name").and_then(|n| n.as_str()) else {
            out.push(("E934", format!("{}: features[{i}] has no name", elem.qualified_name)));
            continue;
        };
        let Some(def) = resolver.resolve_ref(elements, typed) else { continue };
        if !matches!(def.frontmatter.element_type, Some(ElementType::PartDef) | Some(ElementType::Part)) {
            continue;
        }
        if instances.iter().any(|x| x.name == name) {
            out.push(("E934", format!("{}: two parts are named '{name}'", elem.qualified_name)));
            continue;
        }
        let ch = chain(elements, resolver, def);
        let mut ports: Vec<PortSpec> = Vec::new();
        for d in &ch {
            for p in inline_ports(d) {
                if !ports.iter().any(|x| x.name == p.name) {
                    ports.push(p);
                }
            }
        }
        let machine: Vec<String> = ch.iter().find(|d| d.frontmatter.exhibits_states.as_ref().is_some_and(|s| !s.is_empty())).map(|d| d.frontmatter.exhibits_states.clone().unwrap_or_default()).unwrap_or_default();
        let has_fmu = ch.iter().any(|d| d.frontmatter.fmu.is_some());
        let mut targets: Vec<String> = Vec::new();
        for st in &machine {
            match resolver.resolve_ref(elements, st) {
                Some(e) if matches!(e.frontmatter.element_type, Some(ElementType::StateDef) | Some(ElementType::State)) => {
                    if !targets.contains(&e.qualified_name) {
                        targets.push(e.qualified_name.clone());
                    }
                }
                _ => out.push(("E930", format!("{}: the behaviour '{st}' of part '{name}' does not resolve to a StateDef", elem.qualified_name))),
            }
        }
        let behavior = match (targets.len(), has_fmu) {
            (0, false) => Behavior::Inert,
            (0, true) => Behavior::Plant(ch.iter().find(|d| d.frontmatter.fmu.is_some()).map(|d| d.qualified_name.clone()).unwrap_or_default()),
            (1, false) => Behavior::Machine(targets[0].clone()),
            (n, _) => {
                out.push(("E930", format!("{}: part '{name}' has {} behaviours ({}); a part carries exactly one of a state machine or an FMU", elem.qualified_name, if has_fmu { n + 1 } else { n }, if has_fmu { "state machine(s) and an fmu".to_string() } else { targets.join(", ") })));
                continue;
            }
        };
        instances.push(Instance { name: name.to_string(), def: def.qualified_name.clone(), behavior, ports });
    }
    instances
}

/// Whether the composite has a part whose machine is executable: only then do the composition checks apply.
fn has_executable_part(elements: &[RawElement], resolver: &Resolver, instances: &[Instance]) -> bool {
    instances.iter().any(|i| match &i.behavior {
        Behavior::Machine(q) => resolver
            .resolve_ref(elements, q)
            .is_some_and(|e| crate::sm_exec::is_executable(e.frontmatter.variables.as_deref(), e.frontmatter.initial_effect.as_ref(), e.frontmatter.sub_states.as_deref().unwrap_or(&[]), e.frontmatter.transitions.as_deref().unwrap_or(&[]))),
        _ => false,
    })
}

/// Resolve a composite, collecting findings (`E930`, `E931`, `E934`). `None` when it is not an executable composite.
pub fn resolve(elements: &[RawElement], resolver: &Resolver, elem: &RawElement) -> (Option<SystemSpec>, Vec<Finding>) {
    let mut out = Vec::new();
    if !matches!(elem.frontmatter.element_type, Some(ElementType::Part) | Some(ElementType::PartDef)) {
        return (None, out);
    }
    let view = composite_view(elements, resolver, elem);
    let instances = parts_of(elements, resolver, elem, &view.features, &mut out);
    if !has_executable_part(elements, resolver, &instances) {
        return (None, Vec::new());
    }
    let mut instances = instances;
    add_child_ports(elements, resolver, &mut instances);
    let q = &elem.qualified_name;
    let feature_names: Vec<&str> = view.features.iter().filter_map(|f| f.as_mapping().and_then(|m| field(m, "name")).and_then(|n| n.as_str())).collect();
    // an end `a.b` whose `a` is no feature of the composite at all is a typo, not a structural connection
    let unknown_part = |e: &str| endpoint(e).filter(|(p, _)| !feature_names.contains(p)).map(|(p, _)| p.to_string());
    let mut links = Vec::new();
    for (i, c) in view.connections.iter().enumerate() {
        let Some(m) = c.as_mapping() else { continue };
        let (Some(f), Some(t)) = (field(m, "from").and_then(text), field(m, "to").and_then(text)) else { continue };
        let ends = (endpoint(&f), endpoint(&t));
        if let Some(p) = unknown_part(&f).or_else(|| unknown_part(&t)) {
            {
                out.push(("E931", format!("{q}: connection[{i}] {f} → {t}: '{p}' is not a part of this composite")));
                continue;
            }
        }
        let (Some((fi, fp)), Some((ti, tp))) = ends else {
            // a connection that does not name `part.port` at both ends is the structural kind and not ours
            continue;
        };
        let (Some(a), Some(b)) = (instances.iter().position(|x| x.name == fi), instances.iter().position(|x| x.name == ti)) else { continue };
        let Some(pa) = instances[a].ports.iter().find(|p| p.name == fp) else {
            out.push(("E931", format!("{q}: connection[{i}] {f} → {t}: part '{fi}' has no port '{fp}'")));
            continue;
        };
        let Some(pb) = instances[b].ports.iter().find(|p| p.name == tp) else {
            out.push(("E931", format!("{q}: connection[{i}] {f} → {t}: part '{ti}' has no port '{tp}'")));
            continue;
        };
        if !pa.dir.sends() || !pb.dir.receives() {
            out.push(("E931", format!("{q}: connection[{i}] {f} → {t}: a message goes from an out/inout port to an in/inout port, but {fi}.{fp} is {:?} and {ti}.{tp} is {:?}", pa.dir, pb.dir)));
            continue;
        }
        let delay = match field(m, "delay").map(|d| text(d).and_then(|s| duration(&s))) {
            None => 0.0,
            Some(Some(d)) => d,
            Some(None) => {
                out.push(("E934", format!("{q}: connection[{i}]: `delay:` is not a duration such as \"5 ms\"")));
                continue;
            }
        };
        links.push(Link { from: (a, fp.to_string()), to: (b, tp.to_string()), delay });
    }
    let mut bindings = Vec::new();
    for (i, c) in view.bindings.iter().enumerate() {
        let Some(m) = c.as_mapping() else { continue };
        let (Some(f), Some(t)) = (field(m, "from").and_then(text), field(m, "to").and_then(text)) else { continue };
        if let Some(p) = endpoint(&f).filter(|(p, _)| !feature_names.contains(p)).or_else(|| endpoint(&t).filter(|(p, _)| !feature_names.contains(p))).map(|(p, _)| p) {
            out.push(("E933", format!("{q}: binding[{i}] {f} → {t}: '{p}' is not a part of this composite")));
            continue;
        }
        let (Some((fi, fv)), Some((ti, tv))) = (endpoint(&f), endpoint(&t)) else { continue };
        let (Some(a), Some(b)) = (instances.iter().position(|x| x.name == fi), instances.iter().position(|x| x.name == ti)) else { continue };
        bindings.push(Binding { from: (a, fv.to_string()), to: (b, tv.to_string()) });
    }
    (Some(SystemSpec { composite: q.clone(), instances, links, bindings }), out)
}

fn machine_of(elements: &[RawElement], i: &Instance) -> Option<Machine> {
    match &i.behavior {
        Behavior::Machine(q) => ir::lower(elements, q).ok(),
        _ => None,
    }
}

fn walk_actions<'a>(m: &'a Machine, f: &mut dyn FnMut(&'a Action)) {
    for a in &m.initial_actions {
        f(a);
    }
    for s in &m.states {
        s.entry.iter().chain(&s.exit).for_each(&mut *f);
    }
    for t in &m.transitions {
        t.actions.iter().for_each(&mut *f);
    }
}

/// `(port, event)` pairs a machine sends and accepts.
fn traffic(m: &Machine) -> (BTreeSet<(Option<String>, String)>, BTreeSet<(String, String)>) {
    let mut sent = BTreeSet::new();
    walk_actions(m, &mut |a| {
        if let Action::Send { event, via, .. } = a {
            sent.insert((via.clone(), event.clone()));
        }
    });
    let mut accepted = BTreeSet::new();
    for t in &m.transitions {
        if let Trigger::Event { name, .. } = &t.trigger {
            if let Some((p, e)) = name.split_once('/') {
                accepted.insert((p.to_string(), e.to_string()));
            }
        }
    }
    (sent, accepted)
}

/// The findings for one element: `E930`/`E931`/`E934` from resolution, `E932` (a port a machine uses that its part lacks),
/// `E933` (a binding that cannot hold) and `W936` (traffic nobody on the other end handles).
pub fn check(elements: &[RawElement], resolver: &Resolver, elem: &RawElement) -> Vec<Finding> {
    let mut mine = check_all(elements, resolver, elem);
    if mine.is_empty() || elem.frontmatter.supertype.is_none() {
        return mine;
    }
    // what a base composite already reports is reported there, not again on every subtype
    let bare = |q: &str, m: &str| m.strip_prefix(&format!("{q}: ")).unwrap_or(m).to_string();
    let mut inherited: std::collections::HashSet<(&'static str, String)> = std::collections::HashSet::new();
    for s in elem.frontmatter.supertype.iter().flat_map(names) {
        if let Some(base) = resolver.resolve_ref(elements, &s) {
            if matches!(base.frontmatter.element_type, Some(ElementType::Part) | Some(ElementType::PartDef)) {
                inherited.extend(check_all(elements, resolver, base).into_iter().map(|(c, m)| (c, bare(&base.qualified_name, &m))));
            }
        }
    }
    mine.retain(|(c, m)| !inherited.contains(&(*c, bare(&elem.qualified_name, m))));
    mine
}

/// Every finding of a composite, including those that come from wiring it inherits.
pub fn check_all(elements: &[RawElement], resolver: &Resolver, elem: &RawElement) -> Vec<Finding> {
    let (spec, mut out) = resolve(elements, resolver, elem);
    let Some(spec) = spec else { return out };
    let q = &spec.composite;
    let machines: Vec<Option<Machine>> = spec.instances.iter().map(|i| machine_of(elements, i)).collect();

    // E932 — every port a machine names exists on its part, in the right direction.
    for (inst, m) in spec.instances.iter().zip(&machines) {
        let Some(m) = m else { continue };
        let (sent, accepted) = traffic(m);
        for (via, ev) in &sent {
            if let Some(p) = via {
                match inst.ports.iter().find(|x| x.name == *p) {
                    None => out.push(("E932", format!("{q}: part '{}' sends {ev} via '{p}', which is not a port of {}", inst.name, inst.def))),
                    Some(ps) if !ps.dir.sends() => out.push(("E932", format!("{q}: part '{}' sends {ev} via '{p}', an in port", inst.name))),
                    _ => {}
                }
            }
        }
        for (p, ev) in &accepted {
            match inst.ports.iter().find(|x| x.name == *p) {
                None => out.push(("E932", format!("{q}: part '{}' accepts {ev} via '{p}', which is not a port of {}", inst.name, inst.def))),
                Some(ps) if !ps.dir.receives() => out.push(("E932", format!("{q}: part '{}' accepts {ev} via '{p}', an out port", inst.name))),
                _ => {}
            }
        }
    }

    // E933 — one source per sink, and no binding to a part that has no behaviour.
    let mut sinks: BTreeSet<(usize, String)> = BTreeSet::new();
    for b in &spec.bindings {
        let (fi, ti) = (&spec.instances[b.from.0], &spec.instances[b.to.0]);
        if !sinks.insert(b.to.clone()) {
            out.push(("E933", format!("{q}: binding {}.{} → {}.{}: {}.{} already has a source", fi.name, b.from.1, ti.name, b.to.1, ti.name, b.to.1)));
        }
        for i in [fi, ti] {
            if matches!(i.behavior, Behavior::Inert) {
                out.push(("E933", format!("{q}: binding {}.{} → {}.{}: part '{}' has no behaviour to bind to", fi.name, b.from.1, ti.name, b.to.1, i.name)));
            }
        }
    }
    if crate::plant::backend().is_some() && spec.instances.iter().any(|i| matches!(i.behavior, Behavior::Plant(_))) && composite_view(elements, resolver, elem).plant_step.is_none() {
        out.push(("E888", format!("{q}: a composite with plant parts needs a `plantStep:` (the communication step)")));
    }

    // E933 — bindings join variables of compatible type and dimension.
    for b in &spec.bindings {
        let (fi, ti) = (&spec.instances[b.from.0], &spec.instances[b.to.0]);
        let var = |idx: usize, name: &str| -> Result<Option<Value>, String> {
            match (&spec.instances[idx].behavior, &machines[idx]) {
                (Behavior::Machine(_), Some(m)) => m.variables.iter().find(|v| v.name == name).map(|v| Some(v.value.clone())).ok_or_else(|| format!("'{}' has no variable '{name}'", spec.instances[idx].name)),
                _ => Ok(None), // a plant's variables are checked against the FMU, a part without a machine has none
            }
        };
        let (a, b2) = match (var(b.from.0, &b.from.1), var(b.to.0, &b.to.1)) {
            (Err(e), _) | (_, Err(e)) => {
                out.push(("E933", format!("{q}: binding {}.{} → {}.{}: {e}", fi.name, b.from.1, ti.name, b.to.1)));
                continue;
            }
            (Ok(a), Ok(b)) => (a, b),
        };
        if let (Some(a), Some(b2)) = (a, b2) {
            let bad = match (&a, &b2) {
                (Value::Num(x), Value::Num(y)) if x.dim != y.dim => Some(format!("dimension {} vs {}", x.dim.human(), y.dim.human())),
                (Value::Num(_), Value::Num(_)) | (Value::Bool(_), Value::Bool(_)) | (Value::Str(_), Value::Str(_)) => None,
                _ => Some("different types".to_string()),
            };
            if let Some(why) = bad {
                out.push(("E933", format!("{q}: binding {}.{} → {}.{}: {why}", fi.name, b.from.1, ti.name, b.to.1)));
            }
        }
    }

    // W936 — traffic along a connection that the other end does not handle.
    for l in &spec.links {
        let (Some(ms), Some(mr)) = (&machines[l.from.0], &machines[l.to.0]) else { continue };
        let (sent, _) = traffic(ms);
        let (_, accepted) = traffic(mr);
        let (si, ri) = (&spec.instances[l.from.0].name, &spec.instances[l.to.0].name);
        for (via, ev) in sent.iter().filter(|(v, _)| v.as_deref() == Some(l.from.1.as_str())) {
            let _ = via;
            if !accepted.contains(&(l.to.1.clone(), ev.clone())) {
                out.push(("W936", format!("{q}: {si} sends {ev} via {}, but {ri} accepts no {ev} via {}", l.from.1, l.to.1)));
            }
        }
    }
    // an event accepted at a port that is connected, but that none of the connected senders ever sends
    let mut seen: BTreeSet<(usize, String)> = BTreeSet::new();
    for l in &spec.links {
        if !seen.insert(l.to.clone()) {
            continue;
        }
        let senders: Vec<&crate::system::Link> = spec.links.iter().filter(|x| x.to == l.to).collect();
        if senders.iter().any(|x| machines[x.from.0].is_none()) {
            continue;
        }
        let Some(mr) = &machines[l.to.0] else { continue };
        let sent: BTreeSet<String> = senders.iter().flat_map(|x| traffic(machines[x.from.0].as_ref().expect("checked")).0.into_iter().filter(|(v, _)| v.as_deref() == Some(x.from.1.as_str())).map(|(_, e)| e)).collect();
        let ri = &spec.instances[l.to.0].name;
        for (p, ev) in traffic(mr).1.iter().filter(|(p, _)| *p == l.to.1) {
            if !sent.contains(ev) {
                out.push(("W936", format!("{q}: {ri} accepts {ev} via {p}, but no connected part sends it")));
            }
        }
    }
    out
}

//! The state-machine intermediate representation and its lowering pass (REQ-TRS-SMIR-001, GH #278).
//!
//! `lower` turns an executable `StateDef` into a [`Machine`]: flat state and transition tables with ids,
//! resolved names, parsed expressions and a source pointer on every node. The interpreter, coverage,
//! mutation and the SCXML exporter all read this value and never the YAML. It serialises to a canonical
//! JSON form (struct field order is fixed, tables keep document order) whose BLAKE3 hash identifies the
//! machine in recorded evidence.

use std::collections::HashSet;

use serde::Serialize;
use serde_yaml::{Mapping, Value as Y};

use crate::element::{ElementType, RawElement};
use crate::expr::{self, Expr, Value};
use crate::resolver::Resolver;

/// A diagnostic from lowering: `(code, message)`.
pub type Diag = (&'static str, String);

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Origin {
    pub qname: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Var {
    pub slot: usize,
    pub name: String,
    #[serde(rename = "type")]
    pub ty: &'static str,
    /// Dimension label (`s`, `W`, …) of a number; absent for dimensionless numbers and other types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimension: Option<String>,
    pub init: String,
    pub origin: Origin,
    #[serde(skip)]
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NameRef {
    pub name: String,
    /// `var` (a variable slot), `event` (a field of the received payload), `attr` (a model attribute
    /// path) or `enum` (an enumeration literal).
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IrExpr {
    pub src: String,
    pub refs: Vec<NameRef>,
    #[serde(skip)]
    pub ast: Expr,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Action {
    Assign { var: usize, expr: IrExpr },
    /// A referenced behaviour (an `ActionDef` or a `typedBy`); recorded in the trace, not executed.
    Behavior {
        #[serde(rename = "ref")]
        target: String,
        /// `do` for a `doAction`, which runs to completion on entry.
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<&'static str>,
    },
    /// Send an event out of the machine (REQ-TRS-SYSCOMP-001), optionally through a port of the owning part; the payload
    /// expressions are evaluated at send time. Outside a composition the message is recorded in the trace only.
    Send {
        event: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        via: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        payload: Vec<(String, IrExpr)>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Trigger {
    /// No trigger: taken as soon as the source state is entered and its guard holds.
    Eventless,
    Event {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        bind: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Timer { delay_seconds: f64 },
    Change { cond: IrExpr },
    Done { state: usize },
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Atomic,
    Compound,
    Parallel,
    Final,
}

#[derive(Debug, Clone, Serialize)]
pub struct State {
    pub id: usize,
    pub name: String,
    pub path: String,
    pub parent: Option<usize>,
    pub kind: Kind,
    /// The initial child of a compound state.
    pub initial: Option<usize>,
    pub entry: Vec<Action>,
    pub exit: Vec<Action>,
    pub origin: Origin,
}

#[derive(Debug, Clone, Serialize)]
pub struct Transition {
    pub id: usize,
    pub source: usize,
    pub target: Option<usize>,
    pub internal: bool,
    pub trigger: Trigger,
    pub guard: Option<IrExpr>,
    pub actions: Vec<Action>,
    pub origin: Origin,
}

#[derive(Debug, Clone, Serialize)]
pub struct Machine {
    pub machine: String,
    pub hash: String,
    /// The machine's top-level states are parallel regions, all active together.
    pub parallel: bool,
    pub initial: Option<usize>,
    pub variables: Vec<Var>,
    pub events: Vec<String>,
    pub initial_actions: Vec<Action>,
    /// Executable `ActionDef`s named by behaviour references, lowered; they run at that point of a run.
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub behaviors: std::collections::BTreeMap<String, crate::act_ir::ActionProgram>,
    pub states: Vec<State>,
    pub transitions: Vec<Transition>,
}

impl Machine {
    /// Canonical JSON of the machine without its hash.
    pub fn canonical_json(&self) -> String {
        let mut m = self.clone();
        m.hash = String::new();
        serde_json::to_string(&m).expect("IR serialises")
    }

    pub fn compute_hash(&self) -> String {
        blake3::hash(self.canonical_json().as_bytes()).to_hex().to_string()
    }
}

fn field<'a>(m: &'a Mapping, k: &str) -> Option<&'a Y> {
    m.get(Y::String(k.to_string()))
}

fn text(v: &Y) -> Option<String> {
    match v {
        Y::String(s) => Some(s.clone()),
        Y::Number(n) => Some(n.to_string()),
        Y::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

struct Ctx<'a> {
    qname: &'a str,
    vars: Vec<Var>,
    states: Vec<State>,
    /// (state id, path of its YAML mapping) pending transition lowering.
    pending: Vec<(usize, String, &'a Mapping)>,
    diags: Vec<Diag>,
}

impl<'a> Ctx<'a> {
    fn origin(&self, path: &str) -> Origin {
        Origin { qname: self.qname.to_string(), path: path.to_string() }
    }

    fn slot(&self, name: &str) -> Option<usize> {
        self.vars.iter().position(|v| v.name == name)
    }

    fn expr(&mut self, src: &str, bound: &HashSet<String>, where_: &str) -> Option<IrExpr> {
        let ast = match expr::parse(src) {
            Ok(a) => a,
            Err(e) => {
                self.diags.push(("W933", format!("{where_}: `{src}` is not in the expression language: {e}")));
                return None;
            }
        };
        let mut refs = Vec::new();
        for n in ast.names() {
            let root = n.split('.').next().unwrap_or(&n);
            let r = if let Some(slot) = self.slot(&n) {
                NameRef { name: n, kind: "var", slot: Some(slot) }
            } else if n.contains('.') && bound.contains(root) {
                NameRef { name: n, kind: "event", slot: None }
            } else if n.contains("::") && !n.contains('.') {
                NameRef { name: n, kind: "enum", slot: None }
            } else if n.contains('.') {
                NameRef { name: n, kind: "attr", slot: None }
            } else {
                self.diags.push(("W933", format!("{where_}: `{src}` uses '{n}', which is not a declared variable")));
                continue;
            };
            refs.push(r);
        }
        Some(IrExpr { src: src.trim().to_string(), refs, ast })
    }

    fn actions(&mut self, m: &Mapping, keys: &[(&str, Option<&'static str>)], bound: &HashSet<String>, where_: &str) -> Vec<Action> {
        let mut out = Vec::new();
        for (k, role) in keys {
            match field(m, k) {
                Some(Y::String(s)) => out.push(Action::Behavior { target: s.clone(), role: *role }),
                Some(Y::Mapping(am)) => {
                    if let Some(t) = field(am, "typedBy").and_then(|v| v.as_str()) {
                        out.push(Action::Behavior { target: t.to_string(), role: *role });
                    }
                    if let Some(a) = field(am, "assign").and_then(text) {
                        out.extend(self.assigns(&a, bound, where_));
                    }
                    out.extend(self.send(am, bound, where_));
                }
                _ => {}
            }
        }
        out
    }

    /// The `send:` of an action map (`{send: Event, via: port, payload: {name: expression}}`), if it has one.
    fn send(&mut self, am: &Mapping, bound: &HashSet<String>, where_: &str) -> Option<Action> {
        let event = field(am, "send").and_then(text).filter(|s| !s.trim().is_empty())?;
        let via = field(am, "via").and_then(text);
        let mut payload = Vec::new();
        if let Some(Y::Mapping(pm)) = field(am, "payload") {
            for (k, v) in pm {
                if let (Some(k), Some(src)) = (k.as_str(), text(v)) {
                    if let Some(e) = self.expr(&src, bound, where_) {
                        payload.push((k.to_string(), e));
                    }
                }
            }
        }
        Some(Action::Send { event, via, payload })
    }

    fn assigns(&mut self, src: &str, bound: &HashSet<String>, where_: &str) -> Vec<Action> {
        let stmts = match expr::split_statements(src) {
            Ok(s) => s,
            Err(e) => {
                self.diags.push(("E884", format!("{where_}: `assign:` {e}")));
                return Vec::new();
            }
        };
        let mut out = Vec::new();
        for (name, rhs) in stmts {
            let Some(var) = self.slot(&name) else {
                self.diags.push(("E884", format!("{where_}: `assign:` assigns '{name}', which is not a declared variable")));
                continue;
            };
            if let Some(e) = self.expr(&rhs, bound, where_) {
                out.push(Action::Assign { var, expr: e });
            }
        }
        out
    }

    /// Walk the state tree, assigning ids in document order (parents before children).
    fn walk(&mut self, subs: &'a [Y], parent: Option<usize>, parent_path: &str, yaml_path: &str, parallel: bool) -> Option<usize> {
        let mut initial = None;
        let mut first = None;
        for (i, s) in subs.iter().enumerate() {
            let Some(sm) = s.as_mapping() else { continue };
            let Some(name) = field(sm, "name").and_then(|v| v.as_str()) else { continue };
            let path = if parent_path.is_empty() { name.to_string() } else { format!("{parent_path}.{name}") };
            let ypath = format!("{yaml_path}subStates[{i}]");
            let id = self.states.len();
            first.get_or_insert(id);
            let has_children = matches!(field(sm, "subStates"), Some(Y::Sequence(c)) if !c.is_empty());
            let is_final = field(sm, "isFinal").and_then(|v| v.as_bool()) == Some(true);
            let is_parallel = field(sm, "isParallel").and_then(|v| v.as_bool()) == Some(true);
            let kind = if is_final {
                Kind::Final
            } else if has_children && is_parallel {
                Kind::Parallel
            } else if has_children {
                Kind::Compound
            } else {
                Kind::Atomic
            };
            if field(sm, "isInitial").and_then(|v| v.as_bool()) == Some(true) && !parallel {
                initial = Some(id);
            }
            let origin = self.origin(&ypath);
            self.states.push(State { id, name: name.to_string(), path: path.clone(), parent, kind, initial: None, entry: vec![], exit: vec![], origin });
            self.pending.push((id, ypath.clone(), sm));
            if let Some(Y::Sequence(children)) = field(sm, "subStates") {
                let init = self.walk(children, Some(id), &path, &format!("{ypath}."), is_parallel);
                self.states[id].initial = init;
            }
        }
        // A compound state without an `isInitial` child starts in its first (W073/W074 report the omission).
        if parallel { initial } else { initial.or(first) }
    }

    /// Resolve a state name relative to `from`: a full dotted path first, then the children of `from`, of its
    /// parent, and so on up to the top level (nearest wins); an ambiguity at one level is an error.
    fn find_state(&mut self, name: &str, from: Option<usize>, where_: &str) -> Option<usize> {
        if let Some(s) = self.states.iter().find(|s| s.path == name) {
            return Some(s.id);
        }
        let mut scope = from;
        loop {
            let hits: Vec<usize> = self.states.iter().filter(|s| s.parent == scope && s.name == name).map(|s| s.id).collect();
            match hits.len() {
                1 => return Some(hits[0]),
                0 => {}
                _ => return self.ambiguous(name, &hits, where_),
            }
            match scope {
                None => break,
                Some(s) => scope = self.states[s].parent,
            }
        }
        let hits: Vec<usize> = self.states.iter().filter(|s| s.name == name).map(|s| s.id).collect();
        match hits.len() {
            1 => Some(hits[0]),
            0 => {
                self.diags.push(("W076", format!("{where_}: state '{name}' is not a state of the machine")));
                None
            }
            _ => self.ambiguous(name, &hits, where_),
        }
    }

    fn ambiguous(&mut self, name: &str, hits: &[usize], where_: &str) -> Option<usize> {
        let all: Vec<_> = hits.iter().map(|i| self.states[*i].path.as_str()).collect();
        let msg = format!("{where_}: state name '{name}' is ambiguous ({}); use a dotted path", all.join(", "));
        self.diags.push(("W076", msg));
        None
    }
}

/// Lower the `StateDef`/`State` element named `target` (qualified name, id or name). Errors are
/// diagnostics with the codes of REQ-TRS-SMSCHEMA-001; the machine is only returned when there are none.
pub fn lower(elements: &[RawElement], target: &str) -> Result<Machine, Vec<Diag>> {
    let resolver = Resolver::new(elements);
    let Some(el) = resolver.resolve_ref(elements, target) else {
        return Err(vec![("E001", format!("no element '{target}'"))]);
    };
    let fm = &el.frontmatter;
    if !matches!(fm.element_type, Some(ElementType::StateDef) | Some(ElementType::State)) {
        return Err(vec![("E001", format!("'{}' is not a StateDef or State", el.qualified_name))]);
    }
    let subs: &[Y] = fm.sub_states.as_deref().unwrap_or(&[]);
    let top: &[Y] = fm.transitions.as_deref().unwrap_or(&[]);

    // The schema checks are the first diagnostics: lowering refuses what validation rejects.
    let mut diags: Vec<Diag> = crate::sm_exec::check(fm.variables.as_deref(), fm.initial_effect.as_ref(), subs, top)
        .into_iter()
        .filter(|(c, _)| c.starts_with('E'))
        .collect();

    let mut cx = Ctx { qname: &el.qualified_name, vars: Vec::new(), states: Vec::new(), pending: Vec::new(), diags: Vec::new() };

    for (i, v) in fm.variables.as_deref().unwrap_or(&[]).iter().enumerate() {
        let Some(m) = v.as_mapping() else { continue };
        let (Some(name), Some(src)) = (field(m, "name").and_then(|n| n.as_str()), field(m, "value").and_then(text)) else { continue };
        let Ok(mut value) = expr::parse(&src).and_then(|e| e.eval(&|_: &str| None)) else { continue };
        if let (Value::Num(q), Some(unit)) = (&mut value, field(m, "unit").and_then(|u| u.as_str())) {
            if let Some((scale, dim)) = expr::unit_scale(unit.trim_start_matches("SI::")) {
                if q.dim == crate::units::Dim([0; 7]) {
                    q.v *= scale;
                    q.dim = dim;
                }
            }
        }
        let (ty, dimension) = match &value {
            Value::Num(q) => ("number", expr::unit_label(&q.dim).filter(|_| q.dim != crate::units::Dim([0; 7]))),
            Value::Bool(_) => ("boolean", None),
            Value::Str(_) => ("string", None),
        };
        if cx.vars.iter().any(|x| x.name == name) {
            continue;
        }
        let slot = cx.vars.len();
        let origin = cx.origin(&format!("variables[{i}]"));
        cx.vars.push(Var { slot, name: name.to_string(), ty, dimension, init: expr::format_value(&value), origin, value });
    }

    let mut machine_initial = cx.walk(subs, None, "", "", fm.is_parallel == Some(true));
    if machine_initial.is_none() && fm.is_parallel != Some(true) {
        // A machine without an `isInitial` substate starts in its first one (W073/W074 report the omission).
        machine_initial = cx.states.iter().find(|s| s.parent.is_none()).map(|s| s.id);
    }

    let none = HashSet::new();
    let mut events: Vec<String> = Vec::new();
    let mut transitions: Vec<Transition> = Vec::new();

    // State actions.
    let pending = std::mem::take(&mut cx.pending);
    for (id, ypath, sm) in &pending {
        let w = format!("state '{}'", cx.states[*id].path);
        let entry = cx.actions(sm, &[("entryAction", None), ("doAction", Some("do"))], &none, &w);
        let exit = cx.actions(sm, &[("exitAction", None)], &none, &w);
        cx.states[*id].entry = entry;
        cx.states[*id].exit = exit;
        let _ = ypath;
    }
    let initial_actions = match fm.initial_effect.as_ref() {
        Some(Y::Mapping(m)) => cx.actions(m, &[("effect", None)], &none, "initialEffect"),
        _ => Vec::new(),
    };
    let mut initial_actions = initial_actions;
    if let Some(Y::Mapping(m)) = fm.initial_effect.as_ref() {
        if let Some(a) = field(m, "assign").and_then(text) {
            initial_actions.extend(cx.assigns(&a, &none, "initialEffect"));
        }
        initial_actions.extend(cx.send(m, &none, "initialEffect"));
    }

    // Transitions: nested ones in state document order, then the machine's own list.
    let mut lower_transition = |cx: &mut Ctx, src: Option<usize>, t: &Mapping, ypath: String, events: &mut Vec<String>| {
        let w = format!("transition at {ypath}");
        let internal = field(t, "internal").and_then(|v| v.as_bool()) == Some(true);
        let source = match src {
            Some(s) => Some(s),
            None => {
                let n = field(t, "source").or_else(|| field(t, "from")).and_then(|v| v.as_str());
                n.and_then(|n| cx.find_state(n, None, &w))
            }
        };
        let Some(source) = source else { return };
        let target = if internal {
            None
        } else {
            match field(t, "target").or_else(|| field(t, "to")).and_then(|v| v.as_str()) {
                Some(n) => match cx.find_state(n, Some(source), &w) {
                    Some(id) => Some(id),
                    None => return,
                },
                None => {
                    cx.diags.push(("W929", format!("{w}: a transition that is not `internal` needs a `target:`")));
                    return;
                }
            }
        };
        let mut bound: HashSet<String> = HashSet::new();
        let trigger = match field(t, "accept") {
            None => match field(t, "trigger").and_then(|v| v.as_str()) {
                Some(n) => Trigger::Event { name: n.to_string(), bind: None },
                None => Trigger::Eventless,
            },
            Some(Y::String(s)) => Trigger::Event { name: s.clone(), bind: None },
            Some(Y::Mapping(a)) => {
                if let Some(v) = field(a, "after") {
                    let secs = text(v)
                        .and_then(|s| expr::parse(&s).ok())
                        .and_then(|e| e.eval(&|_: &str| None).ok())
                        .and_then(|v| match v {
                            Value::Num(q) => Some(q.v),
                            _ => None,
                        })
                        .unwrap_or(0.0);
                    Trigger::Timer { delay_seconds: secs }
                } else if let Some(v) = field(a, "when") {
                    let src = text(v).unwrap_or_default();
                    match cx.expr(&src, &bound, &w) {
                        Some(cond) => Trigger::Change { cond },
                        None => return,
                    }
                } else if let Some(v) = field(a, "done") {
                    match cx.find_state(&text(v).unwrap_or_default(), Some(source), &w) {
                        Some(state) => Trigger::Done { state },
                        None => return,
                    }
                } else {
                    let payload = field(a, "payload").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if payload.is_empty() {
                        cx.diags.push(("E882", format!("{w}: `accept:` has no trigger (payload, after, when or done)")));
                        return;
                    }
                    let name = match field(a, "via").and_then(|v| v.as_str()) {
                        Some(p) => format!("{p}/{payload}"),
                        None => payload,
                    };
                    let bind = field(a, "as").and_then(|v| v.as_str()).map(String::from);
                    if let Some(b) = &bind {
                        bound.insert(b.clone());
                    }
                    Trigger::Event { name, bind }
                }
            }
            _ => Trigger::Eventless,
        };
        if let Trigger::Event { name, .. } = &trigger {
            if !events.contains(name) {
                events.push(name.clone());
            }
        }
        let guard = field(t, "guard").and_then(text).filter(|g| !g.trim().is_empty()).and_then(|g| cx.expr(&g, &bound, &w));
        if field(t, "guard").and_then(text).is_some_and(|g| !g.trim().is_empty()) && guard.is_none() {
            return;
        }
        let mut actions = cx.actions(t, &[("effect", None)], &bound, &w);
        if let Some(a) = field(t, "assign").and_then(text) {
            actions.extend(cx.assigns(&a, &bound, &w));
        }
        let origin = cx.origin(&ypath);
        let id = transitions.len();
        transitions.push(Transition { id, source, target, internal, trigger, guard, actions, origin });
    };

    for (id, ypath, sm) in &pending {
        if let Some(Y::Sequence(ts)) = field(sm, "transitions") {
            for (j, t) in ts.iter().enumerate() {
                if let Some(tm) = t.as_mapping() {
                    lower_transition(&mut cx, Some(*id), tm, format!("{ypath}.transitions[{j}]"), &mut events);
                }
            }
        }
    }
    for (j, t) in top.iter().enumerate() {
        if let Some(tm) = t.as_mapping() {
            lower_transition(&mut cx, None, tm, format!("transitions[{j}]"), &mut events);
        }
    }

    // Behaviour references that name an executable ActionDef are lowered; they run when reached.
    let mut behaviors = std::collections::BTreeMap::new();
    {
        let mut targets: Vec<String> = Vec::new();
        let mut collect = |acts: &[Action]| {
            for a in acts {
                if let Action::Behavior { target, .. } = a {
                    if !targets.contains(target) {
                        targets.push(target.clone());
                    }
                }
            }
        };
        collect(&initial_actions);
        for st in &cx.states {
            collect(&st.entry);
            collect(&st.exit);
        }
        for t in &transitions {
            collect(&t.actions);
        }
        for t in targets {
            let Some(el) = resolver.resolve_ref(elements, &t) else { continue };
            if !matches!(el.frontmatter.element_type, Some(ElementType::ActionDef) | Some(ElementType::Action)) {
                continue;
            }
            if !crate::act_exec::is_executable(el.frontmatter.variables.as_deref(), el.frontmatter.sub_actions.as_deref().unwrap_or(&[])) {
                continue;
            }
            match crate::act_ir::lower_action(elements, &el.qualified_name) {
                Ok(p) => {
                    behaviors.insert(t.clone(), p);
                }
                Err(ds) => diags.extend(ds.into_iter().map(|(c, m)| (c, format!("behavior {t}: {m}")))),
            }
        }
    }
    diags.append(&mut cx.diags);
    if !diags.is_empty() {
        return Err(diags);
    }
    let mut m = Machine {
        machine: el.qualified_name.clone(),
        hash: String::new(),
        parallel: fm.is_parallel == Some(true),
        initial: machine_initial,
        variables: cx.vars,
        events,
        initial_actions,
        behaviors,
        states: cx.states,
        transitions,
    };
    m.hash = m.compute_hash();
    Ok(m)
}

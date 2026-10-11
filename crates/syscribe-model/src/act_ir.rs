//! The action-graph intermediate representation and its lowering (REQ-TRS-ACTEXEC-001, GH #276).
//!
//! `lower_action` turns an executable `ActionDef` (and, transitively, the `ActionDef`s it performs) into an
//! [`ActionProgram`]: per definition a slot table (parameters and variables), nodes (the sub-actions then the
//! control nodes, in document order), guarded edges (the successions) and data flows. Like the state-machine IR
//! it is a value: canonical JSON, a BLAKE3 hash, a source pointer on every node and edge.

use std::collections::{HashMap, HashSet};

use serde::Serialize;
use serde_yaml::{Mapping, Value as Y};

use crate::element::{ElementType, RawElement};
use crate::expr::{self, Value};
use crate::ir::{Diag, IrExpr, NameRef, Origin};
use crate::resolver::Resolver;
use crate::sm_exec::{field, text};

#[derive(Debug, Clone, Serialize)]
pub struct Slot {
    pub slot: usize,
    pub name: String,
    /// `in`, `out`, `inout` or `local`.
    pub kind: &'static str,
    pub typed_by: Option<String>,
    pub unit: Option<String>,
    pub init: Option<String>,
    #[serde(skip)]
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignIr {
    pub slot: usize,
    pub expr: IrExpr,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArgIr {
    pub name: String,
    pub expr: IrExpr,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NodeKind {
    /// A generic action with no behaviour of its own: completes at once.
    Action,
    Assign { assigns: Vec<AssignIr> },
    Perform { callee: usize, args: Vec<ArgIr> },
    Send { event: String, args: Vec<ArgIr> },
    #[serde(rename_all = "camelCase")]
    Accept {
        event: Option<String>,
        bind: Option<String>,
        after_seconds: Option<f64>,
    },
    If {
        cond: IrExpr,
        then: Vec<Stmt>,
        #[serde(rename = "else")]
        els: Vec<Stmt>,
    },
    #[serde(rename_all = "camelCase")]
    Loop { loop_kind: &'static str, cond: IrExpr, body: Vec<Stmt> },
    Terminate,
    Fork,
    Join,
    Decision,
    Merge,
}

/// A named piece of behaviour: a graph node, or a statement nested in an `IfAction`/`LoopAction` block.
#[derive(Debug, Clone, Serialize)]
pub struct Stmt {
    /// Unique within the program: the key of this statement's coverage.
    pub uid: usize,
    pub name: String,
    pub origin: Origin,
    #[serde(flatten)]
    pub kind: NodeKind,
}

#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub id: usize,
    #[serde(flatten)]
    pub stmt: Stmt,
}

#[derive(Debug, Clone, Serialize)]
pub struct Edge {
    pub id: usize,
    pub from: usize,
    pub to: usize,
    pub guard: Option<IrExpr>,
    #[serde(rename = "else")]
    pub is_else: bool,
    pub origin: Origin,
}

/// A copy of a value when a node completes: `pin` of `from` → `<to_node>.<to_pin>` or a slot.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataFlow {
    pub from_node: usize,
    pub from_pin: String,
    pub to_node: Option<usize>,
    pub to_pin: Option<String>,
    pub to_slot: Option<usize>,
    pub origin: Origin,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionIr {
    pub qname: String,
    pub slots: Vec<Slot>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub start: Vec<usize>,
    pub flows: Vec<DataFlow>,
    pub origin: Origin,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionProgram {
    pub root: String,
    pub hash: String,
    pub defs: Vec<ActionIr>,
}

impl ActionProgram {
    /// Names of every node and nested statement of the root definition.
    pub fn stmt_names(&self) -> std::collections::HashSet<String> {
        fn walk(stmts: &[Stmt], out: &mut std::collections::HashSet<String>) {
            for s in stmts {
                out.insert(s.name.clone());
                match &s.kind {
                    NodeKind::If { then, els, .. } => {
                        walk(then, out);
                        walk(els, out);
                    }
                    NodeKind::Loop { body, .. } => walk(body, out),
                    _ => {}
                }
            }
        }
        let mut out = std::collections::HashSet::new();
        for n in &self.defs[0].nodes {
            walk(std::slice::from_ref(&n.stmt), &mut out);
        }
        out
    }

    pub fn canonical_json(&self) -> String {
        let mut p = self.clone();
        p.hash = String::new();
        serde_json::to_string(&p).expect("IR serialises")
    }
    pub fn compute_hash(&self) -> String {
        blake3::hash(self.canonical_json().as_bytes()).to_hex().to_string()
    }
}

struct Cx<'a> {
    elements: &'a [RawElement],
    resolver: &'a Resolver,
    defs: Vec<Option<ActionIr>>,
    index: HashMap<String, usize>,
    stack: Vec<String>,
    diags: Vec<Diag>,
    next_uid: usize,
}

struct Scope<'a> {
    slots: &'a [Slot],
    /// Names of all sub-actions (a dotted `<sub>.<pin>` name is a pin).
    subs: &'a HashSet<String>,
    bound: &'a HashSet<String>,
}

fn expr_of(src: &str, sc: &Scope, where_: &str, diags: &mut Vec<Diag>) -> Option<IrExpr> {
    let ast = match expr::parse(src) {
        Ok(a) => a,
        Err(e) => {
            diags.push(("W935", format!("{where_}: `{src}` is not in the expression language: {e}")));
            return None;
        }
    };
    let mut refs = Vec::new();
    for n in ast.names() {
        let root = n.split('.').next().unwrap_or(&n).to_string();
        let r = if let Some(slot) = sc.slots.iter().position(|s| s.name == n) {
            NameRef { name: n, kind: "var", slot: Some(slot) }
        } else if n.contains('.') && sc.subs.contains(&root) {
            NameRef { name: n, kind: "pin", slot: None }
        } else if n.contains('.') && sc.bound.contains(&root) {
            NameRef { name: n, kind: "event", slot: None }
        } else if n.contains("::") && !n.contains('.') {
            NameRef { name: n, kind: "enum", slot: None }
        } else if n.contains('.') {
            NameRef { name: n, kind: "attr", slot: None }
        } else {
            diags.push(("W935", format!("{where_}: `{src}` uses '{n}', which is not a parameter or a declared variable")));
            continue;
        };
        refs.push(r);
    }
    Some(IrExpr { src: src.trim().to_string(), refs, ast })
}

fn collect_names(list: &[Y], subs: &mut HashSet<String>, bound: &mut HashSet<String>) {
    let mut all = Vec::new();
    crate::act_exec::walk(list, &mut all);
    for m in all {
        if let Some(n) = field(m, "name").and_then(|n| n.as_str()) {
            subs.insert(n.to_string());
        }
        if let Some(b) = field(m, "as").and_then(|n| n.as_str()) {
            bound.insert(b.to_string());
        }
    }
}

fn args_of(m: &Mapping, sc: &Scope, where_: &str, diags: &mut Vec<Diag>) -> Vec<ArgIr> {
    let mut out = Vec::new();
    for p in field(m, "parameters").and_then(|p| p.as_sequence()).into_iter().flatten().filter_map(|p| p.as_mapping()) {
        let (Some(name), Some(v)) = (field(p, "name").and_then(|n| n.as_str()), field(p, "value").and_then(text)) else { continue };
        if let Some(e) = expr_of(&v, sc, &format!("{where_} parameter '{name}'"), diags) {
            out.push(ArgIr { name: name.to_string(), expr: e });
        }
    }
    out
}

impl<'a> Cx<'a> {
    fn stmt(&mut self, m: &Mapping, path: &str, qname: &str, sc: &Scope, top: bool) -> Option<Stmt> {
        let name = field(m, "name").and_then(|n| n.as_str())?.to_string();
        let origin = Origin { qname: qname.to_string(), path: path.to_string() };
        let kind_s = field(m, "kind").and_then(|k| k.as_str()).unwrap_or("Action");
        let w = format!("sub-action '{name}'");
        let mut diags = Vec::new();
        let kind = match kind_s {
            "AssignmentAction" => {
                let src = field(m, "assign").and_then(text).unwrap_or_default();
                let mut assigns = Vec::new();
                if let Ok(stmts) = expr::split_statements(&src) {
                    for (target, rhs) in stmts {
                        if let (Some(slot), Some(e)) = (sc.slots.iter().position(|s| s.name == target), expr_of(&rhs, sc, &w, &mut diags)) {
                            assigns.push(AssignIr { slot, expr: e });
                        }
                    }
                }
                NodeKind::Assign { assigns }
            }
            "PerformAction" | "Action" if field(m, "typedBy").and_then(|t| t.as_str()).is_some() => {
                let t = field(m, "typedBy").and_then(|t| t.as_str()).unwrap();
                match self.callee(t, &w) {
                    Some(callee) => NodeKind::Perform { callee, args: args_of(m, sc, &w, &mut diags) },
                    None => NodeKind::Action,
                }
            }
            "SendAction" => {
                let payload = field(m, "payload").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let event = match field(m, "via").and_then(|v| v.as_str()) {
                    Some(p) => format!("{p}/{payload}"),
                    None => payload,
                };
                NodeKind::Send { event, args: args_of(m, sc, &w, &mut diags) }
            }
            "AcceptAction" => {
                if !top {
                    self.diags.push(("E887", format!("{w}: an AcceptAction inside an if/loop block cannot suspend; place it at the top level")));
                }
                let after_seconds = field(m, "after").and_then(text).and_then(|a| expr::parse(&a).ok()).and_then(|e| e.eval(&|_: &str| None).ok()).and_then(|v| if let Value::Num(q) = v { Some(q.v) } else { None });
                let event = field(m, "payload").and_then(|v| v.as_str()).map(|p| match field(m, "via").and_then(|v| v.as_str()) {
                    Some(port) => format!("{port}/{p}"),
                    None => p.to_string(),
                });
                NodeKind::Accept { event, bind: field(m, "as").and_then(|v| v.as_str()).map(String::from), after_seconds }
            }
            "IfAction" => {
                let cond = field(m, "condition").and_then(text).and_then(|c| expr_of(&c, sc, &w, &mut diags));
                let then = self.block(m, "then", path, qname, sc);
                let els = self.block(m, "else", path, qname, sc);
                match cond {
                    Some(cond) => NodeKind::If { cond, then, els },
                    None => {
                        self.diags.push(("E887", format!("{w}: an IfAction needs a `condition:`")));
                        NodeKind::Action
                    }
                }
            }
            "LoopAction" => {
                let lk = if field(m, "loopKind").and_then(|k| k.as_str()) == Some("until") { "until" } else { "while" };
                let cond = field(m, "condition").and_then(text).and_then(|c| expr_of(&c, sc, &w, &mut diags));
                let body = self.block(m, "body", path, qname, sc);
                match cond {
                    Some(cond) => NodeKind::Loop { loop_kind: lk, cond, body },
                    None => {
                        self.diags.push(("E887", format!("{w}: a loop needs a `condition:`")));
                        NodeKind::Action
                    }
                }
            }
            "TerminateAction" => NodeKind::Terminate,
            "DecisionNode" => NodeKind::Decision,
            "ForkNode" => NodeKind::Fork,
            "JoinNode" => NodeKind::Join,
            "MergeNode" => NodeKind::Merge,
            _ => NodeKind::Action,
        };
        self.diags.append(&mut diags);
        let uid = self.next_uid;
        self.next_uid += 1;
        Some(Stmt { uid, name, origin, kind })
    }

    fn block(&mut self, m: &Mapping, key: &str, path: &str, qname: &str, sc: &Scope) -> Vec<Stmt> {
        let mut out = Vec::new();
        for (i, v) in field(m, key).and_then(|b| b.as_sequence()).into_iter().flatten().enumerate() {
            if let Some(sm) = v.as_mapping() {
                if let Some(s) = self.stmt(sm, &format!("{path}.{key}[{i}]"), qname, sc, false) {
                    out.push(s);
                }
            }
        }
        out
    }

    /// Resolve and lower the `ActionDef` named `r`; returns its index in the program.
    fn callee(&mut self, r: &str, w: &str) -> Option<usize> {
        let Some(el) = self.resolver.resolve_ref(self.elements, r) else {
            self.diags.push(("E887", format!("{w}: typedBy '{r}' does not resolve")));
            return None;
        };
        if !matches!(el.frontmatter.element_type, Some(ElementType::ActionDef) | Some(ElementType::Action)) {
            self.diags.push(("E887", format!("{w}: typedBy '{r}' is not an ActionDef")));
            return None;
        }
        let q = el.qualified_name.clone();
        if let Some(i) = self.index.get(&q) {
            if self.stack.contains(&q) {
                self.diags.push(("E887", format!("{w}: recursive call of {q} (cycle: {})", self.stack.join(" → "))));
                return None;
            }
            return Some(*i);
        }
        self.def(el)
    }

    fn def(&mut self, el: &RawElement) -> Option<usize> {
        let q = el.qualified_name.clone();
        let idx = self.defs.len();
        self.defs.push(None);
        self.index.insert(q.clone(), idx);
        self.stack.push(q.clone());
        let fm = &el.frontmatter;

        // slots
        let mut slots: Vec<Slot> = Vec::new();
        for p in fm.parameters.iter().flatten().filter_map(|p| p.as_mapping()) {
            let Some(name) = field(p, "name").and_then(|n| n.as_str()) else { continue };
            let kind = match field(p, "direction").and_then(|d| d.as_str()).unwrap_or("in") {
                "out" | "return" => "out",
                "inout" => "inout",
                _ => "in",
            };
            let (ty, unit) = (field(p, "typedBy").and_then(|t| t.as_str()).map(String::from), field(p, "unit").and_then(|t| t.as_str()).map(String::from));
            let value = field(p, "value").and_then(text).and_then(|v| expr::parse(&v).ok()).and_then(|e| e.eval(&|_: &str| None).ok()).map(|v| crate::constraints::coerce_param(v, ty.as_deref(), unit.as_deref()));
            slots.push(Slot { slot: slots.len(), name: name.to_string(), kind, typed_by: ty, unit, init: value.as_ref().map(expr::format_value), value });
        }
        let (vf, _) = crate::sm_exec::check_variables(fm.variables.as_deref());
        self.diags.extend(vf);
        for v in fm.variables.iter().flatten().filter_map(|v| v.as_mapping()) {
            let (Some(name), Some(src)) = (field(v, "name").and_then(|n| n.as_str()), field(v, "value").and_then(text)) else { continue };
            let Ok(mut value) = expr::parse(&src).and_then(|e| e.eval(&|_: &str| None)) else { continue };
            let unit = field(v, "unit").and_then(|u| u.as_str()).map(String::from);
            if let (Value::Num(qn), Some(u)) = (&mut value, &unit) {
                if let Some((scale, dim)) = expr::unit_scale(u.trim_start_matches("SI::")) {
                    if qn.dim == crate::units::Dim([0; 7]) {
                        qn.v *= scale;
                        qn.dim = dim;
                    }
                }
            }
            if slots.iter().any(|s| s.name == name) {
                continue;
            }
            slots.push(Slot { slot: slots.len(), name: name.to_string(), kind: "local", typed_by: field(v, "typedBy").and_then(|t| t.as_str()).map(String::from), unit, init: Some(expr::format_value(&value)), value: Some(value) });
        }

        let subs_list: &[Y] = fm.sub_actions.as_deref().unwrap_or(&[]);
        let controls: &[Y] = fm.control_nodes.as_deref().unwrap_or(&[]);
        let mut subs = HashSet::new();
        let mut bound = HashSet::new();
        collect_names(subs_list, &mut subs, &mut bound);
        for c in controls.iter().filter_map(|c| c.as_mapping()) {
            if let Some(n) = field(c, "name").and_then(|n| n.as_str()) {
                subs.insert(n.to_string());
            }
        }
        let sc = Scope { slots: &slots, subs: &subs, bound: &bound };

        // nodes: sub-actions, then control nodes
        let mut nodes: Vec<Node> = Vec::new();
        let mut ids: HashMap<String, usize> = HashMap::new();
        for (i, v) in subs_list.iter().enumerate() {
            if let Some(m) = v.as_mapping() {
                if let Some(stmt) = self.stmt(m, &format!("subActions[{i}]"), &q, &sc, true) {
                    ids.insert(stmt.name.clone(), nodes.len());
                    nodes.push(Node { id: nodes.len(), stmt });
                }
            }
        }
        for (i, v) in controls.iter().enumerate() {
            if let Some(m) = v.as_mapping() {
                let mut m2 = m.clone();
                m2.entry(Y::String("kind".into())).or_insert(Y::String("ForkNode".into()));
                if let Some(stmt) = self.stmt(&m2, &format!("controlNodes[{i}]"), &q, &sc, true) {
                    ids.insert(stmt.name.clone(), nodes.len());
                    nodes.push(Node { id: nodes.len(), stmt });
                }
            }
        }

        // edges
        let mut edges: Vec<Edge> = Vec::new();
        let mut implicit_start: HashSet<usize> = HashSet::new();
        for (i, s) in fm.succession_connections.iter().flatten().enumerate() {
            let Some(m) = s.as_mapping() else { continue };
            let (Some(a), Some(b)) = (field(m, "after").and_then(|v| v.as_str()), field(m, "before").and_then(|v| v.as_str())) else { continue };
            let root = |x: &str| x.split('.').next().unwrap_or(x).to_string();
            let (a, b) = (root(a), root(b));
            let origin = Origin { qname: q.clone(), path: format!("successionConnections[{i}]") };
            if a == "start" {
                if let Some(t) = ids.get(&b) {
                    implicit_start.insert(*t);
                }
                continue;
            }
            if b == "done" {
                continue;
            }
            let (Some(&from), Some(&to)) = (ids.get(&a), ids.get(&b)) else {
                let missing = if ids.contains_key(&a) { &b } else { &a };
                self.diags.push(("E887", format!("{q}: succession {i} names '{missing}', which is not a sub-action or control node")));
                continue;
            };
            let g = field(m, "guard").and_then(text);
            let is_else = g.as_deref().is_some_and(|g| g.trim().eq_ignore_ascii_case("else"));
            let guard = if is_else { None } else { g.and_then(|g| expr_of(&g, &sc, &format!("succession {i}"), &mut self.diags)) };
            edges.push(Edge { id: edges.len(), from, to, guard, is_else, origin });
        }
        let incoming: HashSet<usize> = edges.iter().map(|e| e.to).collect();
        let start: Vec<usize> = nodes.iter().map(|n| n.id).filter(|n| !incoming.contains(n) || implicit_start.contains(n)).collect();

        // data flows
        let mut flows: Vec<DataFlow> = Vec::new();
        let pin = |s: &str| -> Option<(usize, String)> {
            let (n, p) = s.split_once('.')?;
            ids.get(n).map(|id| (*id, p.to_string()))
        };
        // The direction of a pin of a performed action, from the callee's parameter.
        let pin_dir = |this: &Self, node: usize, pn: &str| -> Option<&'static str> {
            match &nodes[node].stmt.kind {
                NodeKind::Perform { callee, .. } => this.defs.get(*callee)?.as_ref()?.slots.iter().find(|s| s.name == pn).map(|s| s.kind),
                _ => None,
            }
        };
        let name_expr = |slot: usize| {
            let nm = slots[slot].name.clone();
            IrExpr { src: nm.clone(), refs: vec![NameRef { name: nm.clone(), kind: "var", slot: Some(slot) }], ast: crate::expr::Expr::Name(nm) }
        };
        // (target node, pin, slot) inputs fed from a slot: become arguments of the performed action.
        let mut feeds: Vec<(usize, String, usize)> = Vec::new();
        let mut connect = |this: &mut Self, a: &str, b: &str, path: String, directed: bool, flows: &mut Vec<DataFlow>| {
            let origin = Origin { qname: q.clone(), path: path.clone() };
            let slot_of = |x: &str| slots.iter().position(|s| s.name == x);
            match (pin(a), pin(b), slot_of(a), slot_of(b)) {
                (Some((fa, pa)), Some((fb, pb)), _, _) => flows.push(DataFlow { from_node: fa, from_pin: pa, to_node: Some(fb), to_pin: Some(pb), to_slot: None, origin }),
                (Some((n, pn)), None, _, Some(sl)) | (None, Some((n, pn)), Some(sl), _) if !directed || pin(a).is_some() => {
                    // pin → slot (completion copy) or slot → pin (an argument), by the callee's parameter direction
                    let pin_is_a = pin(a).is_some();
                    let dir = pin_dir(this, n, &pn);
                    match (dir, directed, pin_is_a) {
                        (Some("out"), _, _) | (None, true, true) | (Some("inout"), _, true) => flows.push(DataFlow { from_node: n, from_pin: pn, to_node: None, to_pin: None, to_slot: Some(sl), origin }),
                        (Some("in"), _, _) | (None, true, false) | (Some("inout"), _, false) => feeds.push((n, pn, sl)),
                        _ => this.diags.push(("E887", format!("{q}: {path}: cannot tell the direction of '{a}' ↔ '{b}'; the performed action has no parameter '{pn}'"))),
                    }
                }
                (None, Some((n, pn)), Some(sl), None) => feeds.push((n, pn, sl)),
                _ => this.diags.push(("E887", format!("{q}: {path}: '{a}' and '{b}' must be `<sub-action>.<pin>` or a parameter/variable of the action"))),
            }
        };
        for (i, f) in fm.flow_connections.iter().flatten().enumerate() {
            let Some(m) = f.as_mapping() else { continue };
            let (Some(a), Some(b)) = (field(m, "from").and_then(|v| v.as_str()), field(m, "to").and_then(|v| v.as_str())) else { continue };
            connect(self, a, b, format!("flowConnections[{i}]"), true, &mut flows);
        }
        for (i, b) in fm.binding_connections.iter().flatten().enumerate() {
            let Some(m) = b.as_mapping() else { continue };
            let (Some(l), Some(r)) = (field(m, "left").and_then(|v| v.as_str()), field(m, "right").and_then(|v| v.as_str())) else { continue };
            connect(self, l, r, format!("bindingConnections[{i}]"), false, &mut flows);
        }
        for (n, pn, sl) in feeds {
            let expr = name_expr(sl);
            match &mut nodes[n].stmt.kind {
                NodeKind::Perform { args, .. } => {
                    if !args.iter().any(|a| a.name == pn) {
                        args.push(ArgIr { name: pn, expr });
                    }
                }
                _ => self.diags.push(("E887", format!("{q}: '{}.{pn}' is fed from '{}' but '{}' is not a performed action", nodes[n].stmt.name, slots[sl].name, nodes[n].stmt.name))),
            }
        }

        self.stack.pop();
        self.defs[idx] = Some(ActionIr { qname: q.clone(), slots, nodes, edges, start, flows, origin: Origin { qname: q, path: String::new() } });
        Some(idx)
    }
}

/// Lower the `ActionDef` named `target` and everything it performs.
pub fn lower_action(elements: &[RawElement], target: &str) -> Result<ActionProgram, Vec<Diag>> {
    let resolver = Resolver::new(elements);
    let Some(el) = resolver.resolve_ref(elements, target) else {
        return Err(vec![("E001", format!("no element '{target}'"))]);
    };
    if !matches!(el.frontmatter.element_type, Some(ElementType::ActionDef) | Some(ElementType::Action)) {
        return Err(vec![("E001", format!("'{}' is not an ActionDef", el.qualified_name))]);
    }
    let fm = &el.frontmatter;
    let mut cx = Cx { elements, resolver: &resolver, defs: Vec::new(), index: HashMap::new(), stack: Vec::new(), diags: Vec::new(), next_uid: 0 };
    // The schema checks first: lowering refuses what validation rejects.
    let mut diags: Vec<Diag> = crate::act_exec::check_with(
        false,
        fm.parameters.as_deref().unwrap_or(&[]),
        fm.variables.as_deref(),
        fm.sub_actions.as_deref().unwrap_or(&[]),
        fm.control_nodes.as_deref().unwrap_or(&[]),
        fm.succession_connections.as_deref().unwrap_or(&[]),
        fm.flow_connections.as_deref().unwrap_or(&[]),
        fm.binding_connections.as_deref().unwrap_or(&[]),
    )
    .into_iter()
    .filter(|(c, _)| c.starts_with('E'))
    .collect();
    cx.def(el);
    diags.append(&mut cx.diags);
    // A name that is not a declared parameter or variable is a diagnostic here (W935 is fatal when lowering).
    if !diags.is_empty() {
        diags.sort();
        diags.dedup();
        return Err(diags);
    }
    let defs: Vec<ActionIr> = cx.defs.into_iter().map(|d| d.expect("every reserved def is filled")).collect();
    let mut p = ActionProgram { root: el.qualified_name.clone(), hash: String::new(), defs };
    p.hash = p.compute_hash();
    Ok(p)
}

//! Parametric constraint checking (GH #274, REQ-TRS-PARAM-001): evaluates `ConstraintDef`
//! expressions and `requires:`/`assume:` clauses with the expression language of
//! [`crate::expr`] against attribute values, defaults and Configuration parameters.

use std::collections::{BTreeMap, HashMap};

use serde_yaml::Value as Y;

use crate::element::{ElementType, RawElement};
use crate::expr::{self, format_value, Quantity, Value};
use crate::resolver::Resolver;
use crate::units::{quantity_dimension, Dim};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    Unresolved,
    Error,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Fail => "fail",
            Status::Unresolved => "unresolved",
            Status::Error => "error",
        }
    }
}

/// One evaluated check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// The usage (`Owner::feature` or the element's qualified name) or the requirement clause.
    pub id: String,
    /// `constraint`, `requires` or `assume`.
    pub kind: &'static str,
    /// Qualified name of the `ConstraintDef`, when one is involved.
    pub constraint: Option<String>,
    pub expression: String,
    pub status: Status,
    /// Why a check is an error or unresolved; empty otherwise.
    pub message: String,
    /// Parameter → value as the expression saw it, in parameter order.
    pub bindings: Vec<(String, String)>,
    pub unresolved: Vec<String>,
}

pub type Env = HashMap<String, Value>;

fn yaml_str(v: &Y) -> Option<String> {
    match v {
        Y::String(s) => Some(s.clone()),
        Y::Number(n) => Some(n.to_string()),
        Y::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn get<'a>(m: &'a Y, key: &str) -> Option<&'a Y> {
    m.get(key)
}

/// A scalar read from `text` and given the dimension of `unit` (`SI::kg`, `kg`) or, without a unit,
/// of an `ISQ::…Value` type (SI base units).
fn typed_value(text: &str, unit: Option<&str>, type_ref: Option<&str>) -> Option<Value> {
    let v = expr::parse_value(text)?;
    let Value::Num(q) = v else { return Some(v) };
    if q.dim != Dim([0; 7]) {
        return Some(Value::Num(q)); // the text carried its own unit
    }
    if let Some(u) = unit.map(|u| u.strip_prefix("SI::").unwrap_or(u)).filter(|u| !u.is_empty()) {
        if let Some((scale, dim)) = expr::unit_scale(u) {
            return Some(Value::Num(Quantity { v: q.v * scale, dim }));
        }
        return Some(Value::Num(q));
    }
    if let Some(dim) = type_ref.and_then(quantity_dimension) {
        return Some(Value::Num(Quantity { v: q.v, dim }));
    }
    Some(Value::Num(q))
}

/// Attribute values of the model: an inline `features:` entry with a `value:` is
/// `<owner qualified name>::<feature>` (and `<owner>.<feature>`); an `Attribute` element's own
/// `value:` is its qualified name.
pub fn attribute_env(elements: &[RawElement]) -> Env {
    let mut env = Env::new();
    for e in elements {
        if e.frontmatter.element_type == Some(ElementType::Attribute) {
            if let Some(text) = e.frontmatter.value.as_ref().and_then(yaml_str) {
                let type_ref = e.frontmatter.typed_by.as_ref().and_then(yaml_str);
                if let Some(v) = typed_value(&text, e.frontmatter.unit.as_deref(), type_ref.as_deref()) {
                    env.insert(e.qualified_name.clone(), v);
                }
            }
        }
        for f in e.frontmatter.features.iter().flatten() {
            let (Some(name), Some(text)) = (get(f, "name").and_then(Y::as_str), get(f, "value").and_then(yaml_str)) else { continue };
            // A binding to another feature or a default-bound parameter is not an attribute value.
            let unit = get(f, "unit").and_then(Y::as_str);
            let type_ref = get(f, "typedBy").and_then(Y::as_str);
            if let Some(v) = typed_value(&text, unit, type_ref) {
                env.insert(format!("{}::{name}", e.qualified_name), v.clone());
                env.insert(format!("{}.{name}", e.qualified_name), v);
            }
        }
    }
    env
}

/// The parameter values of Configuration `cfg`: for every declared feature parameter,
/// `<feature qualified name>.<param>` and `<FEAT id>.<param>` → its value in `cfg`.
pub fn config_env(elements: &[RawElement], resolver: &Resolver, cfg: &RawElement) -> Env {
    let mut env = Env::new();
    for fd in elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef)) {
        for p in fd.frontmatter.parameters.iter().flatten() {
            let Some(name) = get(p, "name").and_then(Y::as_str) else { continue };
            let Some(text) = crate::placeholders::value_in_config(elements, resolver, cfg, &fd.qualified_name, name) else { continue };
            let unit = get(p, "unit").and_then(Y::as_str);
            let type_ref = get(p, "type").or_else(|| get(p, "typedBy")).and_then(Y::as_str);
            let Some(v) = typed_value(&text, unit, type_ref) else { continue };
            env.insert(format!("{}.{name}", fd.qualified_name), v.clone());
            if let Some(id) = fd.frontmatter.id.as_deref() {
                env.insert(format!("{id}.{name}"), v);
            }
        }
    }
    env
}

/// Whether `name` (`Enum::literal`) names a literal of an `EnumerationDef` of the model.
fn is_enum_literal(elements: &[RawElement], name: &str) -> bool {
    let Some((owner, lit)) = name.rsplit_once("::") else { return false };
    elements.iter().any(|e| {
        e.frontmatter.element_type == Some(ElementType::EnumerationDef)
            && (e.qualified_name == owner || e.qualified_name.ends_with(&format!("::{owner}")))
            && e.frontmatter.values.iter().flatten().any(|v| get(v, "name").and_then(Y::as_str) == Some(lit))
    })
}

/// Evaluate `src` with `local` over `global`. Returns the status, a message and the unresolved names.
fn evaluate(elements: &[RawElement], src: &str, local: &Env, global: &Env) -> (Status, String, Vec<String>, Option<bool>) {
    let e = match expr::parse(src) {
        Ok(e) => e,
        Err(err) => return (Status::Error, err.to_string(), Vec::new(), None),
    };
    let known = |n: &str| local.contains_key(n) || global.contains_key(n) || is_enum_literal(elements, n);
    let unresolved: Vec<String> = e.names().into_iter().filter(|n| !known(n)).collect();
    if !unresolved.is_empty() {
        return (Status::Unresolved, format!("no value for {}", unresolved.join(", ")), unresolved, None);
    }
    match e.eval(&|n: &str| local.get(n).or_else(|| global.get(n)).cloned()) {
        Ok(Value::Bool(b)) => (if b { Status::Pass } else { Status::Fail }, String::new(), Vec::new(), Some(b)),
        Ok(other) => (Status::Error, format!("the expression is {}, not a boolean", other.kind()), Vec::new(), None),
        Err(err) => (Status::Error, err.to_string(), Vec::new(), None),
    }
}

/// A `ConstraintDef`'s input parameters: `(name, type, unit, default)`.
fn def_params(def: &RawElement) -> Vec<(String, Option<String>, Option<String>, Option<String>)> {
    def.frontmatter
        .features
        .iter()
        .flatten()
        .filter(|f| matches!(get(f, "direction").and_then(Y::as_str), None | Some("in") | Some("inout")))
        .filter_map(|f| {
            let name = get(f, "name").and_then(Y::as_str)?.to_string();
            Some((
                name,
                get(f, "typedBy").and_then(Y::as_str).map(str::to_string),
                get(f, "unit").and_then(Y::as_str).map(str::to_string),
                get(f, "value").and_then(yaml_str),
            ))
        })
        .collect()
}

fn is_scalar_type(t: Option<&str>) -> bool {
    t.map(|t| t.starts_with("ScalarValues::")).unwrap_or(true)
}

/// Check one application of `def`, bound by `connections` (a `bindingConnections:` list).
#[allow(clippy::too_many_arguments)]
fn check_def(
    elements: &[RawElement],
    def: &RawElement,
    connections: &[Y],
    global: &Env,
    id: String,
    kind: &'static str,
    negated: bool,
) -> CheckResult {
    let expression = def.frontmatter.expression.clone().unwrap_or_default();
    let params = def_params(def);
    let names: Vec<&str> = params.iter().map(|p| p.0.as_str()).collect();
    let mut local = Env::new();
    let mut bindings: Vec<(String, String)> = Vec::new();
    let mut unresolved: Vec<String> = Vec::new();
    let mut first_error: Option<String> = None;
    for (name, ty, unit, default) in &params {
        // The bound expression: `left: <param>, right: <expr>` (or the mirror image).
        let bound = connections.iter().find_map(|c| {
            let (l, r) = (get(c, "left").and_then(yaml_str)?, get(c, "right").and_then(yaml_str)?);
            if l == *name {
                Some(r)
            } else if r == *name && !names.contains(&l.as_str()) {
                Some(l)
            } else {
                None
            }
        });
        let has_binding = bound.is_some();
        let value: Option<Value> = match bound {
            Some(src) => {
                let (st, msg, unres, _) = evaluate(elements, &src, &Env::new(), global);
                let _ = st;
                if !unres.is_empty() {
                    unresolved.extend(unres);
                    None
                } else if !msg.is_empty() {
                    first_error.get_or_insert(format!("binding of '{name}': {msg}"));
                    None
                } else {
                    expr::parse(&src).ok().and_then(|e| e.eval(&|n: &str| global.get(n).cloned().or_else(|| is_enum_literal(elements, n).then(|| Value::Str(n.to_string())))).ok())
                }
            }
            None => default.as_deref().and_then(|d| typed_value(d, unit.as_deref(), ty.as_deref())),
        };
        let Some(mut v) = value else {
            if !has_binding && default.is_none() {
                unresolved.push(name.clone());
            }
            continue;
        };
        // A ScalarValues-typed (or untyped) parameter receives the SI-base number; an ISQ-typed one keeps the dimension.
        if is_scalar_type(ty.as_deref()) {
            if let Value::Num(q) = &v {
                v = Value::Num(Quantity::scalar(q.v));
            }
        }
        bindings.push((name.clone(), format_value(&v)));
        local.insert(name.clone(), v);
    }
    let constraint = Some(def.qualified_name.clone());
    if let Some(m) = first_error {
        return CheckResult { id, kind, constraint, expression, status: Status::Error, message: m, bindings, unresolved: Vec::new() };
    }
    if !unresolved.is_empty() {
        unresolved.sort();
        unresolved.dedup();
        let message = format!("no value for {}", unresolved.join(", "));
        return CheckResult { id, kind, constraint, expression, status: Status::Unresolved, message, bindings, unresolved };
    }
    let (mut status, message, unres, held) = evaluate(elements, &expression, &local, global);
    if negated {
        if let Some(h) = held {
            status = if h { Status::Fail } else { Status::Pass };
        }
    }
    CheckResult { id, kind, constraint, expression, status, message, bindings, unresolved: unres }
}

fn bool_flag(f: &Y, key: &str) -> bool {
    get(f, key).and_then(Y::as_bool).unwrap_or(false)
}

/// Evaluate every constraint check of the model against `env` (attribute values, configuration
/// parameters and overrides, already merged by the caller).
pub fn check_all(elements: &[RawElement], env: &Env) -> Vec<CheckResult> {
    let resolver = Resolver::new(elements);
    let def_of = |r: &str| {
        resolver
            .resolve_ref(elements, r)
            .filter(|d| d.frontmatter.element_type == Some(ElementType::ConstraintDef) && d.frontmatter.expression.is_some())
    };
    let mut out: Vec<CheckResult> = Vec::new();
    for e in elements {
        let owner = e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone());
        if e.frontmatter.element_type == Some(ElementType::Constraint) {
            if let Some(def) = e.frontmatter.typed_by.as_ref().and_then(yaml_str).and_then(|r| def_of(&r)) {
                let conns: Vec<Y> = e.frontmatter.binding_connections.clone().unwrap_or_default();
                out.push(check_def(elements, def, &conns, env, owner.clone(), "constraint", e.frontmatter.is_negated.unwrap_or(false)));
            }
        }
        for f in e.frontmatter.features.iter().flatten() {
            let is_constraint = get(f, "type").and_then(Y::as_str) == Some("Constraint");
            let def = get(f, "typedBy").and_then(Y::as_str).and_then(|r| def_of(r));
            if let (true, Some(def), Some(name)) = (is_constraint, def, get(f, "name").and_then(Y::as_str)) {
                let conns: Vec<Y> = get(f, "bindingConnections").and_then(Y::as_sequence).cloned().unwrap_or_default();
                out.push(check_def(elements, def, &conns, env, format!("{}::{name}", e.qualified_name), "constraint", bool_flag(f, "isNegated")));
            }
        }
        if matches!(e.frontmatter.element_type, Some(ElementType::Requirement) | Some(ElementType::RequirementDef)) {
            for (kind, list) in [("requires", &e.frontmatter.requires), ("assume", &e.frontmatter.assume)] {
                for (i, clause) in list.iter().flatten().enumerate() {
                    let id = format!("{owner}#{kind}[{i}]");
                    let negated = bool_flag(clause, "isNegated");
                    let typed = get(clause, "typedBy").and_then(Y::as_str).or_else(|| clause.as_str()).and_then(def_of);
                    if let Some(def) = typed {
                        let conns: Vec<Y> = get(clause, "bindingConnections").and_then(Y::as_sequence).cloned().unwrap_or_default();
                        out.push(check_def(elements, def, &conns, env, id, kind, negated));
                        continue;
                    }
                    let src = get(clause, "expression").and_then(yaml_str).or_else(|| clause.as_str().map(str::to_string));
                    let Some(src) = src else { continue };
                    let (mut status, message, unresolved, held) = evaluate(elements, &src, &Env::new(), env);
                    if negated {
                        if let Some(h) = held {
                            status = if h { Status::Fail } else { Status::Pass };
                        }
                    }
                    out.push(CheckResult { id, kind, constraint: None, expression: src, status, message, bindings: Vec::new(), unresolved });
                }
            }
        }
    }
    out.sort_by(|a, b| (&a.id, a.kind).cmp(&(&b.id, b.kind)));
    out
}

/// Count results by status.
pub fn summary(results: &[CheckResult]) -> BTreeMap<&'static str, usize> {
    let mut m: BTreeMap<&'static str, usize> = ["pass", "fail", "unresolved", "error"].into_iter().map(|k| (k, 0)).collect();
    for r in results {
        *m.entry(r.status.as_str()).or_insert(0) += 1;
    }
    m
}

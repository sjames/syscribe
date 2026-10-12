//! In-process continuous plants (GH #340, REQ-TRS-PLANT-001): a `PlantDef` element written in the expression language,
//! or one of a small library of standard plants, attached to a part with `plant:` and run by the composite engine at its
//! `plantStep:` through the same [`Plant`] seam as an FMU. No external toolchain and no feature flag.
//!
//! A `PlantDef` declares `parameters:` (`{name, value}`), `inputs:` and `outputs:` (`{name, unit}`), `states:`
//! (`{name, value}`, the initial value) and `equations:` of two forms: `der(x) = expression` for a state derivative and
//! `y := expression` for an algebraic output. Integration is fixed-step (`method: rk4`, the default, or `euler`), with
//! inputs held over each communication step and the step split so that no sub-step exceeds `maxStep:`. Everything is
//! IEEE arithmetic plus the pure-Rust `libm` functions of the expression language, so the same model and scenario give
//! the same trace, bit for bit, on every machine.
//!
//! Codes: `E938` a malformed `PlantDef` (structure, equation forms, unknown names, an algebraic loop), `E935` a unit or
//! type error in an equation, `E939` a `plant:` declaration that does not resolve or whose parameters do not fit.

use std::collections::VecDeque;
use std::sync::Arc;

use serde_yaml::{Mapping, Value as Y};

use crate::element::{ElementType, RawElement};
use crate::expr::{self, Expr, ExprError, Quantity, Value};
use crate::plant::{Causality, FmuInfo, Plant, VarInfo, VarType};
use crate::resolver::Resolver;
use crate::units::Dim;

pub type Finding = (&'static str, String);

const NONE: Dim = Dim([0; 7]);
const TIME: Dim = Dim([0, 0, 1, 0, 0, 0, 0]);

/// The integration method of a `PlantDef`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Euler,
    Rk4,
}

impl Method {
    pub fn name(self) -> &'static str {
        match self {
            Method::Euler => "euler",
            Method::Rk4 => "rk4",
        }
    }
}

/// A named quantity of a plant: a parameter's value, a state's initial value or an input's value before anything
/// writes it, in SI base units.
#[derive(Debug, Clone, PartialEq)]
pub struct PlantVar {
    pub name: String,
    pub dim: Dim,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub enum Dynamics {
    /// One derivative per state (in state order) and the outputs in an order in which each needs only earlier ones.
    Equations { derivs: Vec<Expr>, outputs: Vec<(usize, Expr)> },
    /// A transport delay of `delay` seconds from the one input to the one output, sampled at the communication points;
    /// the output is the `initial` parameter until the first sample is old enough.
    Delay { delay: f64 },
}

/// A plant compiled and checked: what the engine instantiates for every run.
#[derive(Debug, Clone)]
pub struct PlantModel {
    /// The `PlantDef`'s qualified name, or `library:<Name>`.
    pub name: String,
    pub parameters: Vec<PlantVar>,
    pub inputs: Vec<PlantVar>,
    pub outputs: Vec<PlantVar>,
    pub states: Vec<PlantVar>,
    pub dynamics: Dynamics,
    pub method: Method,
    pub max_step: Option<f64>,
    /// The equations as written, for the identity hash.
    pub sources: Vec<String>,
}

fn text(v: &Y) -> Option<String> {
    match v {
        Y::String(s) => Some(s.clone()),
        Y::Number(n) => Some(n.to_string()),
        Y::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn field<'a>(m: &'a Mapping, k: &str) -> Option<&'a Y> {
    m.get(Y::String(k.to_string()))
}

fn is_name(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A constant expression (`0.4 SI::ohm`, `2 ms`, `1`) as a quantity in SI base units.
fn constant(src: &str) -> Result<Quantity, String> {
    match expr::parse(src).map_err(|e| e.to_string())?.eval(&|_: &str| None) {
        Ok(Value::Num(q)) => Ok(q),
        Ok(other) => Err(format!("'{src}' is a {}, not a number", other.kind())),
        Err(e) => Err(format!("'{src}' does not evaluate: {e}")),
    }
}

/// The dimension a `unit:` names (`SI::V`, `A`, `ohm`); absent → dimensionless.
fn unit_dim(unit: Option<&str>) -> Result<Dim, String> {
    match unit.map(str::trim).filter(|u| !u.is_empty()) {
        None | Some("1") => Ok(NONE),
        Some(u) => match expr::unit_scale(u) {
            Some((scale, d)) if (scale - 1.0).abs() < 1e-12 => Ok(d),
            // a plant computes in SI base units: a scaled unit would silently be read as its base unit
            Some((scale, _)) => Err(format!("unit '{u}' is {scale} of an SI base unit; a plant variable is declared in coherent SI units (`V`, not `mV`)")),
            None => Err(format!("unknown unit '{u}'")),
        },
    }
}

/// The raw declaration of a plant, before its equations are checked: what a `PlantDef` element and a library entry
/// both reduce to.
struct Decl {
    name: String,
    parameters: Vec<PlantVar>,
    inputs: Vec<PlantVar>,
    outputs: Vec<PlantVar>,
    states: Vec<PlantVar>,
    equations: Vec<String>,
    method: Method,
    max_step: Option<f64>,
}

/// Read the entries of one list (`parameters:`, `inputs:`, …): each a mapping with `name` and the given keys.
fn entries(list: Option<&[Y]>, key: &str, allowed: &[&str], errs: &mut Vec<String>) -> Vec<(String, Mapping)> {
    let mut out = Vec::new();
    for (i, it) in list.unwrap_or(&[]).iter().enumerate() {
        let at = format!("`{key}[{i}]`");
        let m = match it {
            Y::Mapping(m) => m.clone(),
            // a bare name: `inputs: [u]`
            Y::String(s) => {
                let mut m = Mapping::new();
                m.insert(Y::String("name".into()), Y::String(s.clone()));
                m
            }
            _ => {
                errs.push(format!("{at} must be a mapping with a `name`"));
                continue;
            }
        };
        for k in m.keys().filter_map(|k| k.as_str()) {
            if k != "name" && !allowed.contains(&k) {
                errs.push(format!("{at} has an unknown key `{k}` (name, {})", allowed.join(", ")));
            }
        }
        match field(&m, "name").and_then(text) {
            Some(n) if is_name(&n) => out.push((n, m)),
            Some(n) => errs.push(format!("{at}: '{n}' is not a name (letters, digits and `_`, not starting with a digit)")),
            None => errs.push(format!("{at} has no `name`")),
        }
    }
    out
}

fn read_decl(elem: &RawElement) -> Result<Decl, Vec<String>> {
    let fm = &elem.frontmatter;
    let mut errs = Vec::new();
    let mut parameters = Vec::new();
    for (name, m) in entries(fm.parameters.as_deref(), "parameters", &["value", "description"], &mut errs) {
        match field(&m, "value").and_then(text) {
            Some(v) => match constant(&v) {
                Ok(q) => parameters.push(PlantVar { name, dim: q.dim, value: q.v }),
                Err(e) => errs.push(format!("parameter '{name}': {e}")),
            },
            None => errs.push(format!("parameter '{name}' has no `value`")),
        }
    }
    // `inputs:` of mappings is read into `plant_inputs`; a list of bare names stays in the fault-tree field `inputs`
    let raw_inputs: Option<Vec<Y>> = fm.plant_inputs.clone().or_else(|| fm.inputs.as_ref().map(|v| v.iter().map(|s| Y::String(s.clone())).collect()));
    let mut inputs = Vec::new();
    for (name, m) in entries(raw_inputs.as_deref(), "inputs", &["unit", "value", "description"], &mut errs) {
        match unit_dim(field(&m, "unit").and_then(text).as_deref()) {
            Ok(dim) => {
                let value = match field(&m, "value").and_then(text) {
                    None => 0.0,
                    Some(v) => match constant(&v) {
                        Ok(q) if q.dim == dim => q.v,
                        Ok(q) => {
                            errs.push(format!("input '{name}': `value` has dimension {} but the unit is {}", q.dim.human(), dim.human()));
                            0.0
                        }
                        Err(e) => {
                            errs.push(format!("input '{name}': {e}"));
                            0.0
                        }
                    },
                };
                inputs.push(PlantVar { name, dim, value });
            }
            Err(e) => errs.push(format!("input '{name}': {e}")),
        }
    }
    let mut outputs = Vec::new();
    for (name, m) in entries(fm.outputs.as_deref(), "outputs", &["unit", "description"], &mut errs) {
        match unit_dim(field(&m, "unit").and_then(text).as_deref()) {
            Ok(dim) => outputs.push(PlantVar { name, dim, value: 0.0 }),
            Err(e) => errs.push(format!("output '{name}': {e}")),
        }
    }
    let mut states = Vec::new();
    for (name, m) in entries(fm.states.as_deref(), "states", &["value", "description"], &mut errs) {
        match field(&m, "value").and_then(text) {
            Some(v) => match constant(&v) {
                Ok(q) => states.push(PlantVar { name, dim: q.dim, value: q.v }),
                Err(e) => errs.push(format!("state '{name}': {e}")),
            },
            None => errs.push(format!("state '{name}' has no initial `value`")),
        }
    }
    let mut equations = Vec::new();
    for (i, e) in fm.equations.iter().flatten().enumerate() {
        match text(e) {
            Some(s) => equations.push(s),
            None => errs.push(format!("`equations[{i}]` must be a string such as `der(x) = (u - x) / tau` or `y := x`")),
        }
    }
    let method = match fm.method.as_deref().map(str::trim) {
        None | Some("rk4") => Method::Rk4,
        Some("euler") => Method::Euler,
        Some(other) => {
            errs.push(format!("`method: {other}` is not rk4 or euler"));
            Method::Rk4
        }
    };
    let max_step = match fm.max_step.as_ref().and_then(text) {
        None => None,
        Some(s) => match crate::system::duration(&s).filter(|d| *d > 0.0) {
            Some(d) => Some(d),
            None => {
                errs.push(format!("`maxStep: {s}` is not a positive duration such as \"1 ms\""));
                None
            }
        },
    };
    if errs.is_empty() {
        Ok(Decl { name: elem.qualified_name.clone(), parameters, inputs, outputs, states, equations, method, max_step })
    } else {
        Err(errs)
    }
}

enum Eq {
    Der(String, String),
    Out(String, String),
}

fn split_equation(src: &str) -> Option<Eq> {
    let s = src.trim();
    // `der(x) = …`; a name that merely starts with `der` (`derate := …`) is an output
    if let Some(rest) = s.strip_prefix("der").and_then(|r| r.trim_start().strip_prefix('(')) {
        let (name, rhs) = rest.split_once(')')?;
        let rhs = rhs.trim_start().strip_prefix('=')?;
        if rhs.starts_with('=') {
            return None;
        }
        return Some(Eq::Der(name.trim().to_string(), rhs.trim().to_string()));
    }
    let (name, rhs) = s.split_once(":=")?;
    Some(Eq::Out(name.trim().to_string(), rhs.trim().to_string()))
}

/// Check a declaration's equations and lower it. `Err` holds `E938` and `E935` findings.
fn lower(d: Decl) -> Result<PlantModel, Vec<Finding>> {
    let mut errs: Vec<Finding> = Vec::new();
    let all: Vec<(&str, &PlantVar)> = d
        .parameters
        .iter()
        .map(|v| ("parameter", v))
        .chain(d.inputs.iter().map(|v| ("input", v)))
        .chain(d.outputs.iter().map(|v| ("output", v)))
        .chain(d.states.iter().map(|v| ("state", v)))
        .collect();
    for (i, (_, v)) in all.iter().enumerate() {
        if all[..i].iter().any(|(_, w)| w.name == v.name) {
            errs.push(("E938", format!("'{}' is declared twice; parameter, input, output and state names must be distinct", v.name)));
        }
    }
    if d.outputs.is_empty() {
        errs.push(("E938", "a plant needs at least one output".to_string()));
    }
    let mut derivs: Vec<Option<Expr>> = vec![None; d.states.len()];
    let mut outs: Vec<Option<Expr>> = vec![None; d.outputs.len()];
    for src in &d.equations {
        let Some(eq) = split_equation(src) else {
            errs.push(("E938", format!("equation `{src}` is neither `der(state) = expression` nor `output := expression`")));
            continue;
        };
        let (target, rhs) = match &eq {
            Eq::Der(n, r) | Eq::Out(n, r) => (n.as_str(), r.as_str()),
        };
        let parsed = match expr::parse(rhs) {
            Ok(e) => e,
            Err(e) => {
                errs.push(("E938", format!("equation `{src}`: {e}")));
                continue;
            }
        };
        for n in parsed.names() {
            if !all.iter().any(|(_, v)| v.name == n) {
                errs.push(("E938", format!("equation `{src}` uses '{n}', which is not a parameter, input, output or state of the plant")));
            }
        }
        match eq {
            Eq::Der(..) => match d.states.iter().position(|s| s.name == target) {
                Some(i) if derivs[i].is_some() => errs.push(("E938", format!("state '{target}' has two `der(...)` equations"))),
                Some(i) => derivs[i] = Some(parsed),
                None => errs.push(("E938", format!("equation `{src}`: '{target}' is not a state, so it has no derivative"))),
            },
            Eq::Out(..) => match d.outputs.iter().position(|o| o.name == target) {
                Some(i) if outs[i].is_some() => errs.push(("E938", format!("output '{target}' has two `:=` equations"))),
                Some(i) => outs[i] = Some(parsed),
                None => errs.push(("E938", format!("equation `{src}`: '{target}' is not an output (only outputs are assigned with `:=`)"))),
            },
        }
    }
    for (s, e) in d.states.iter().zip(&derivs) {
        if e.is_none() {
            errs.push(("E938", format!("state '{}' has no `der({}) = …` equation", s.name, s.name)));
        }
    }
    for (o, e) in d.outputs.iter().zip(&outs) {
        if e.is_none() {
            errs.push(("E938", format!("output '{}' has no `{} := …` equation", o.name, o.name)));
        }
    }
    if !errs.is_empty() {
        return Err(errs);
    }
    let derivs: Vec<Expr> = derivs.into_iter().map(|e| e.expect("checked")).collect();
    let outs: Vec<Expr> = outs.into_iter().map(|e| e.expect("checked")).collect();
    // outputs in an order where each uses only outputs before it; a cycle is an algebraic loop
    let mut order: Vec<usize> = Vec::new();
    let mut placed = vec![false; outs.len()];
    while order.len() < outs.len() {
        let next = (0..outs.len()).find(|&i| !placed[i] && outs[i].names().iter().all(|n| d.outputs.iter().position(|o| &o.name == n).is_none_or(|j| placed[j])));
        match next {
            Some(i) => {
                placed[i] = true;
                order.push(i);
            }
            None => {
                let looped: Vec<&str> = (0..outs.len()).filter(|&i| !placed[i]).map(|i| d.outputs[i].name.as_str()).collect();
                return Err(vec![("E938", format!("algebraic loop between the outputs {}: each needs another to be computed first", looped.join(", ")))]);
            }
        }
    }
    let model = PlantModel {
        name: d.name,
        parameters: d.parameters,
        inputs: d.inputs,
        outputs: d.outputs,
        states: d.states,
        dynamics: Dynamics::Equations { derivs, outputs: order.into_iter().map(|i| (i, outs[i].clone())).collect() },
        method: d.method,
        max_step: d.max_step,
        sources: d.equations,
    };
    let dims = model.check_dimensions();
    if dims.is_empty() {
        Ok(model)
    } else {
        Err(dims)
    }
}

/// Compile a `PlantDef` element.
pub fn compile(elem: &RawElement) -> Result<PlantModel, Vec<Finding>> {
    let d = read_decl(elem).map_err(|es| es.into_iter().map(|e| ("E938", e)).collect::<Vec<_>>())?;
    lower(d)
}

/// The most sub-steps of `maxStep` one communication step may take.
pub const MAX_SUBSTEPS: usize = 1_000_000;

/// The standard plants `plant: {library: <Name>}` names.
pub const LIBRARY: &[&str] = &["Gain", "FirstOrderLag", "Integrator", "Saturation", "DelayLine", "ImpedanceDivider"];

fn var(name: &str, dim: Dim, value: f64) -> PlantVar {
    PlantVar { name: name.to_string(), dim, value }
}

fn sub(a: Dim, b: Dim) -> Dim {
    let mut d = a.0;
    for (x, y) in d.iter_mut().zip(b.0) {
        *x -= y;
    }
    Dim(d)
}

/// A standard plant over signals of `unit` (dimensionless when absent).
pub fn library(name: &str, unit: Option<&str>) -> Result<PlantModel, String> {
    let u = unit_dim(unit)?;
    let (volt, amp, ohm) = (unit_dim(Some("V"))?, unit_dim(Some("A"))?, unit_dim(Some("ohm"))?);
    let decl = |parameters: Vec<PlantVar>, inputs: Vec<PlantVar>, outputs: Vec<PlantVar>, states: Vec<PlantVar>, equations: &[&str]| Decl {
        name: format!("library:{name}"),
        parameters,
        inputs,
        outputs,
        states,
        equations: equations.iter().map(|s| s.to_string()).collect(),
        method: Method::Rk4,
        max_step: None,
    };
    let d = match name {
        "Gain" => decl(vec![var("k", NONE, 1.0)], vec![var("u", u, 0.0)], vec![var("y", u, 0.0)], vec![], &["y := k * u"]),
        "FirstOrderLag" => decl(
            vec![var("k", NONE, 1.0), var("tau", TIME, 1.0)],
            vec![var("u", u, 0.0)],
            vec![var("y", u, 0.0)],
            vec![var("x", u, 0.0)],
            &["der(x) = (k * u - x) / tau", "y := x"],
        ),
        // `unit` is the output's; the input is its rate
        "Integrator" => decl(vec![var("k", NONE, 1.0)], vec![var("u", sub(u, TIME), 0.0)], vec![var("y", u, 0.0)], vec![var("x", u, 0.0)], &["der(x) = k * u", "y := x"]),
        "Saturation" => decl(vec![var("lo", u, 0.0), var("hi", u, 1.0)], vec![var("u", u, 0.0)], vec![var("y", u, 0.0)], vec![], &["y := min(max(u, lo), hi)"]),
        "ImpedanceDivider" => {
            if unit.is_some() {
                return Err("ImpedanceDivider takes no `unit` (its input is a current in A, its output a voltage in V)".to_string());
            }
            decl(vec![var("vSrc", volt, 0.0), var("z", ohm, 0.0)], vec![var("i", amp, 0.0)], vec![var("v", volt, 0.0)], vec![], &["v := vSrc - z * i"])
        }
        "DelayLine" => {
            return Ok(PlantModel {
                name: format!("library:{name}"),
                parameters: vec![var("delay", TIME, 0.0), var("initial", u, 0.0)],
                inputs: vec![var("u", u, 0.0)],
                outputs: vec![var("y", u, 0.0)],
                states: vec![],
                dynamics: Dynamics::Delay { delay: 0.0 },
                method: Method::Rk4,
                max_step: None,
                sources: vec![],
            })
        }
        other => return Err(format!("'{other}' is not a library plant ({})", LIBRARY.join(", "))),
    };
    lower(d).map_err(|es| es.into_iter().map(|(_, m)| m).collect::<Vec<_>>().join("; "))
}

/// A part's `plant:` declaration: a `PlantDef` reference or a library plant, with parameter overrides.
#[derive(Debug, Clone, PartialEq)]
pub struct PlantRef {
    pub def: Option<String>,
    pub library: Option<String>,
    pub unit: Option<String>,
    pub parameters: Vec<(String, String)>,
}

/// Read a `plant:` value: `Pkg::Def`, or `{def: Pkg::Def | library: Name, unit, parameters: {name: value}}`.
/// The PlantDef a `plant:` value names (`plant: X` or `plant: {def: X}`), as written: the cross-reference the
/// referrer guard, `refs` and `impact` follow.
pub fn plant_def_ref(v: Option<&Y>) -> Option<String> {
    match v? {
        Y::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Y::Mapping(m) => field(m, "def").and_then(text),
        _ => None,
    }
}

pub fn parse_plant_ref(v: &Y) -> Result<PlantRef, Vec<String>> {
    let m = match v {
        Y::String(s) if !s.trim().is_empty() => return Ok(PlantRef { def: Some(s.trim().to_string()), library: None, unit: None, parameters: Vec::new() }),
        Y::Mapping(m) => m,
        _ => return Err(vec!["`plant:` must name a PlantDef, or be a mapping {def | library, unit, parameters}".to_string()]),
    };
    let mut errs = Vec::new();
    for k in m.keys().filter_map(|k| k.as_str()) {
        if !["def", "library", "unit", "parameters"].contains(&k) {
            errs.push(format!("`plant:` has an unknown key `{k}` (def, library, unit, parameters)"));
        }
    }
    let def = field(m, "def").and_then(text);
    let library = field(m, "library").and_then(text);
    if def.is_some() == library.is_some() {
        errs.push("`plant:` needs exactly one of `def:` (a PlantDef) and `library:` (a standard plant)".to_string());
    }
    let unit = field(m, "unit").and_then(text);
    if unit.is_some() && def.is_some() {
        errs.push("`plant: unit:` applies only to a `library:` plant; a PlantDef declares its own units".to_string());
    }
    let mut parameters = Vec::new();
    match field(m, "parameters") {
        None => {}
        Some(Y::Mapping(pm)) => {
            for (k, val) in pm {
                match (k.as_str(), text(val)) {
                    (Some(k), Some(v)) => parameters.push((k.to_string(), v)),
                    _ => errs.push("`plant: parameters:` maps a parameter name to a value such as \"0.5 SI::ohm\"".to_string()),
                }
            }
        }
        Some(_) => errs.push("`plant: parameters:` must be a mapping {name: value}".to_string()),
    }
    if errs.is_empty() {
        Ok(PlantRef { def, library, unit, parameters })
    } else {
        Err(errs)
    }
}

/// The `plant:` declaration on `def` or the nearest definition it specialises, with its owner.
pub fn declaration<'a>(chain: &[&'a RawElement]) -> Option<(&'a RawElement, &'a Y)> {
    chain.iter().find_map(|d| d.frontmatter.plant.as_ref().map(|p| (*d, p)))
}

/// Resolve a `plant:` declaration to a checked model with its parameter overrides applied. `E939` for the declaration
/// itself; a referenced `PlantDef`'s own problems come back as one `E939` naming it (they are reported on it in full).
pub fn resolve(elements: &[RawElement], resolver: &Resolver, decl: &Y) -> Result<PlantModel, Vec<Finding>> {
    let r = parse_plant_ref(decl).map_err(|es| es.into_iter().map(|e| ("E939", e)).collect::<Vec<_>>())?;
    let mut model = match (&r.def, &r.library) {
        (Some(d), _) => match resolver.resolve_ref(elements, d) {
            Some(e) if e.frontmatter.element_type == Some(ElementType::PlantDef) => {
                compile(e).map_err(|_| vec![("E939", format!("`plant: {d}` names a PlantDef that has errors (see {})", e.qualified_name))])?
            }
            Some(e) => return Err(vec![("E939", format!("`plant: {d}` resolves to {}, which is not a PlantDef", e.qualified_name))]),
            None => return Err(vec![("E939", format!("`plant: {d}` does not resolve to an element"))]),
        },
        (None, Some(l)) => library(l, r.unit.as_deref()).map_err(|e| vec![("E939", format!("`plant: library: {l}`: {e}"))])?,
        (None, None) => unreachable!("parse_plant_ref requires one"),
    };
    let errs = model.set_parameters(&r.parameters);
    if errs.is_empty() {
        Ok(model)
    } else {
        Err(errs.into_iter().map(|e| ("E939", e)).collect())
    }
}

impl PlantModel {
    /// Override parameter values (`name`, constant expression); each must name a parameter and keep its dimension.
    pub fn set_parameters(&mut self, overrides: &[(String, String)]) -> Vec<String> {
        let mut errs = Vec::new();
        for (name, src) in overrides {
            let Some(p) = self.parameters.iter_mut().find(|p| &p.name == name) else {
                errs.push(format!("'{name}' is not a parameter of {} (parameters: {})", self.name, self.parameters.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")));
                continue;
            };
            match constant(src) {
                Ok(q) if q.dim == p.dim => p.value = q.v,
                Ok(q) => errs.push(format!("parameter '{name}' = {src} has dimension {} but the parameter has {}", q.dim.human(), p.dim.human())),
                Err(e) => errs.push(format!("parameter '{name}': {e}")),
            }
        }
        if let Dynamics::Delay { delay } = &mut self.dynamics {
            *delay = self.parameters.iter().find(|p| p.name == "delay").map_or(0.0, |p| p.value);
            if *delay < 0.0 {
                errs.push(format!("parameter 'delay' of {} must not be negative", self.name));
            }
        }
        errs
    }

    /// Unit and type errors of the equations (`E935`), found by evaluating each with stand-in values of the declared
    /// dimensions. A domain error at a stand-in value says nothing about the model and is not reported.
    fn check_dimensions(&self) -> Vec<Finding> {
        let Dynamics::Equations { derivs, outputs } = &self.dynamics else { return Vec::new() };
        let mut errs = Vec::new();
        let mut known: Vec<(String, Value)> = Vec::new();
        for p in &self.parameters {
            known.push((p.name.clone(), Value::Num(Quantity { v: p.value, dim: p.dim })));
        }
        for v in self.inputs.iter().chain(&self.states) {
            known.push((v.name.clone(), Value::Num(Quantity { v: 1.0, dim: v.dim })));
        }
        let report = |what: String, e: ExprError, errs: &mut Vec<Finding>| {
            if matches!(e, ExprError::Dimension(_) | ExprError::Type(_)) {
                errs.push(("E935", format!("{what}: {e}")));
            }
        };
        for (i, e) in outputs {
            let o = &self.outputs[*i];
            let lookup = |n: &str| known.iter().find(|(k, _)| k == n).map(|(_, v)| v.clone());
            match e.eval(&lookup) {
                Ok(Value::Num(q)) if q.dim == o.dim => known.push((o.name.clone(), Value::Num(Quantity { v: 1.0, dim: o.dim }))),
                Ok(Value::Num(q)) => errs.push(("E935", format!("output '{}' is declared {} but `{} := {}` has dimension {}", o.name, o.dim.human(), o.name, e.to_source(), q.dim.human()))),
                Ok(other) => errs.push(("E935", format!("`{} := {}` is a {}, not a number", o.name, e.to_source(), other.kind()))),
                Err(err) => report(format!("`{} := {}`", o.name, e.to_source()), err, &mut errs),
            }
            if !known.iter().any(|(k, _)| k == &o.name) {
                known.push((o.name.clone(), Value::Num(Quantity { v: 1.0, dim: o.dim })));
            }
        }
        let lookup = |n: &str| known.iter().find(|(k, _)| k == n).map(|(_, v)| v.clone());
        for (s, e) in self.states.iter().zip(derivs) {
            let want = sub(s.dim, TIME);
            match e.eval(&lookup) {
                Ok(Value::Num(q)) if q.dim == want => {}
                Ok(Value::Num(q)) => errs.push((
                    "E935",
                    format!("`der({}) = {}` has dimension {} but the derivative of '{}' ({}) per second is {}", s.name, e.to_source(), q.dim.human(), s.name, s.dim.human(), want.human()),
                )),
                Ok(other) => errs.push(("E935", format!("`der({}) = {}` is a {}, not a number", s.name, e.to_source(), other.kind()))),
                Err(err) => report(format!("`der({}) = {}`", s.name, e.to_source()), err, &mut errs),
            }
        }
        errs
    }

    /// What identifies the plant in the composite's evidence hash: every value and equation that shapes its behaviour.
    pub fn hash(&self) -> String {
        let vars = |vs: &[PlantVar]| vs.iter().map(|v| serde_json::json!([v.name, v.dim.0, v.value])).collect::<Vec<_>>();
        let delay = match &self.dynamics {
            Dynamics::Delay { delay } => Some(*delay),
            Dynamics::Equations { .. } => None,
        };
        let j = serde_json::json!({
            "name": self.name, "parameters": vars(&self.parameters), "inputs": vars(&self.inputs), "outputs": vars(&self.outputs),
            "states": vars(&self.states), "equations": self.sources, "method": self.method.name(), "maxStep": self.max_step, "delay": delay,
        });
        blake3::hash(j.to_string().as_bytes()).to_hex().to_string()
    }

    /// The plant's variables as the engine's connection checks see them.
    pub fn info(&self) -> FmuInfo {
        let v = |p: &PlantVar, causality: Causality| VarInfo { name: p.name.clone(), causality, ty: VarType::Real, dim: (p.dim != NONE).then_some(p.dim), factor: 1.0, offset: 0.0 };
        let variables = self
            .inputs
            .iter()
            .map(|p| v(p, Causality::Input))
            .chain(self.outputs.iter().map(|p| v(p, Causality::Output)))
            .chain(self.parameters.iter().map(|p| v(p, Causality::Parameter)))
            .chain(self.states.iter().map(|p| v(p, Causality::Local)))
            .collect();
        FmuInfo { fmi_version: "PlantDef".to_string(), guid: self.name.clone(), sha256: self.hash(), supports_cosimulation: true, variables }
    }
}

/// A running instance of a [`PlantModel`].
pub struct EquationPlant {
    model: Arc<PlantModel>,
    info: FmuInfo,
    params: Vec<f64>,
    inputs: Vec<f64>,
    states: Vec<f64>,
    /// `DelayLine`: the input held at each communication point, oldest first.
    history: VecDeque<(f64, f64)>,
    now: f64,
}

impl EquationPlant {
    pub fn new(model: Arc<PlantModel>) -> EquationPlant {
        EquationPlant {
            info: model.info(),
            params: model.parameters.iter().map(|p| p.value).collect(),
            inputs: model.inputs.iter().map(|p| p.value).collect(),
            states: model.states.iter().map(|p| p.value).collect(),
            history: VecDeque::new(),
            now: 0.0,
            model,
        }
    }

    fn lookup<'a>(&'a self, states: &'a [f64], outs: &'a [Option<f64>]) -> impl Fn(&str) -> Option<Value> + 'a {
        move |n: &str| {
            let m = &self.model;
            let q = |p: &PlantVar, v: f64| Some(Value::Num(Quantity { v, dim: p.dim }));
            if let Some(i) = m.states.iter().position(|p| p.name == n) {
                return q(&m.states[i], states[i]);
            }
            if let Some(i) = m.inputs.iter().position(|p| p.name == n) {
                return q(&m.inputs[i], self.inputs[i]);
            }
            if let Some(i) = m.parameters.iter().position(|p| p.name == n) {
                return q(&m.parameters[i], self.params[i]);
            }
            let i = m.outputs.iter().position(|p| p.name == n)?;
            outs[i].and_then(|v| q(&m.outputs[i], v))
        }
    }

    fn eval_num(&self, e: &Expr, states: &[f64], outs: &[Option<f64>], what: &str) -> Result<f64, String> {
        match e.eval(&self.lookup(states, outs)) {
            Ok(Value::Num(q)) if q.v.is_finite() => Ok(q.v),
            Ok(Value::Num(q)) => Err(format!("{what} is not finite ({}) at t = {} s", q.v, expr::format_number(self.now))),
            Ok(other) => Err(format!("{what} is a {}", other.kind())),
            Err(err) => Err(format!("{what}: {err} at t = {} s", expr::format_number(self.now))),
        }
    }

    /// The outputs at the given states (and the current inputs and parameters).
    fn outputs_at(&self, states: &[f64]) -> Result<Vec<f64>, String> {
        let m = &self.model;
        match &m.dynamics {
            Dynamics::Equations { outputs, .. } => {
                let mut outs: Vec<Option<f64>> = vec![None; m.outputs.len()];
                for (i, e) in outputs {
                    outs[*i] = Some(self.eval_num(e, states, &outs, &format!("output '{}'", m.outputs[*i].name))?);
                }
                Ok(outs.into_iter().map(|o| o.unwrap_or(0.0)).collect())
            }
            Dynamics::Delay { delay } => {
                let initial = m.parameters.iter().position(|p| p.name == "initial").map_or(0.0, |i| self.params[i]);
                let due = self.now - delay + 1e-12;
                Ok(vec![match self.history.front() {
                    Some((t, v)) if *t <= due => *v,
                    _ => initial,
                }])
            }
        }
    }

    fn derivatives(&self, states: &[f64]) -> Result<Vec<f64>, String> {
        let Dynamics::Equations { derivs, .. } = &self.model.dynamics else { return Ok(Vec::new()) };
        let outs: Vec<Option<f64>> = self.outputs_at(states)?.into_iter().map(Some).collect();
        derivs.iter().zip(&self.model.states).map(|(e, s)| self.eval_num(e, states, &outs, &format!("der({})", s.name))).collect()
    }

    /// One integration sub-step. `false` when every derivative is exactly zero: the states are at rest and, with the
    /// inputs held, stay bit-for-bit where they are for the rest of the communication step.
    fn integrate(&mut self, dt: f64) -> Result<bool, String> {
        let x = self.states.clone();
        let k1 = self.derivatives(&x)?;
        if k1.iter().all(|d| *d == 0.0) {
            return Ok(false);
        }
        let next = match self.model.method {
            Method::Euler => x.iter().zip(&k1).map(|(a, d)| a + dt * d).collect::<Vec<_>>(),
            Method::Rk4 => {
                let at = |x: &[f64], k: &[f64], f: f64| x.iter().zip(k).map(|(a, d)| a + f * dt * d).collect::<Vec<_>>();
                let k2 = self.derivatives(&at(&x, &k1, 0.5))?;
                let k3 = self.derivatives(&at(&x, &k2, 0.5))?;
                let k4 = self.derivatives(&at(&x, &k3, 1.0))?;
                (0..x.len()).map(|i| x[i] + dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i])).collect()
            }
        };
        self.states = next;
        Ok(true)
    }
}

impl Plant for EquationPlant {
    fn info(&self) -> &FmuInfo {
        &self.info
    }

    fn set(&mut self, name: &str, value: f64) -> Result<(), String> {
        let m = &self.model;
        if let Some(i) = m.inputs.iter().position(|p| p.name == name) {
            self.inputs[i] = value;
        } else if let Some(i) = m.parameters.iter().position(|p| p.name == name) {
            self.params[i] = value;
            if let (Dynamics::Delay { .. }, "delay") = (&m.dynamics, name) {
                if value < 0.0 || !value.is_finite() {
                    return Err(format!("parameter 'delay' of {} must be a non-negative number, not {value}", m.name));
                }
                // the delay line reads its delay from the model; a changed delay is honoured through a fresh model
                let mut changed = (**m).clone();
                changed.dynamics = Dynamics::Delay { delay: value };
                if let Some(p) = changed.parameters.iter_mut().find(|p| p.name == "delay") {
                    p.value = value;
                }
                self.model = Arc::new(changed);
            }
        } else {
            return Err(format!("'{name}' is not an input or parameter of {}", m.name));
        }
        Ok(())
    }

    fn get(&mut self, name: &str) -> Result<f64, String> {
        let m = &self.model;
        if let Some(i) = m.outputs.iter().position(|p| p.name == name) {
            return Ok(self.outputs_at(&self.states)?[i]);
        }
        if let Some(i) = m.parameters.iter().position(|p| p.name == name) {
            return Ok(self.params[i]);
        }
        if let Some(i) = m.inputs.iter().position(|p| p.name == name) {
            return Ok(self.inputs[i]);
        }
        if let Some(i) = m.states.iter().position(|p| p.name == name) {
            return Ok(self.states[i]);
        }
        Err(format!("'{name}' is not a variable of {}", m.name))
    }

    fn step(&mut self, t: f64, h: f64) -> Result<(), String> {
        if !(h >= 0.0 && h.is_finite()) {
            return Err(format!("step of {h} s"));
        }
        if let Dynamics::Delay { delay } = self.model.dynamics {
            self.history.push_back((t, self.inputs.first().copied().unwrap_or(0.0)));
            self.now = t + h;
            let due = self.now - delay + 1e-12;
            while self.history.len() > 1 && self.history[1].0 <= due {
                self.history.pop_front();
            }
            return Ok(());
        }
        if !self.states.is_empty() && h > 0.0 {
            let n = self.model.max_step.map_or(1.0, |m| (h / m).ceil().max(1.0));
            if n > MAX_SUBSTEPS as f64 {
                return Err(format!("a step of {h} s needs {n} sub-steps of maxStep, more than {MAX_SUBSTEPS}; raise `maxStep` or lower `plantStep`"));
            }
            let dt = h / n;
            for i in 0..(n as usize) {
                self.now = t + dt * i as f64;
                if !self.integrate(dt)? {
                    break;
                }
            }
        }
        self.now = t + h;
        Ok(())
    }

    fn provenance(&self) -> serde_json::Value {
        serde_json::json!({"plantDef": self.model.name, "hash": self.info.sha256, "method": self.model.method.name(), "maxStep": self.model.max_step})
    }
}

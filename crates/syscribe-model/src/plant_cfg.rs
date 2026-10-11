//! Parsing of the plant fields (`fmu:` on a `Part`; `plants:`, `plantStep:`, `plantConnections:` on a
//! `TestCase`) of REQ-TRS-FMI-001. Pure YAML reading, always compiled; nothing here needs an FMI library.

use serde_yaml::{Mapping, Value as Y};

use crate::expr::{self, Value};
use crate::plant::{Causality, FmuInfo};
use crate::units::Dim;

#[derive(Debug, Clone)]
pub struct FmuDecl {
    pub file: String,
    pub fmi_version: Option<String>,
    pub sha256: Option<String>,
    pub parameters: Vec<(String, Value)>,
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

/// Read an `fmu:` declaration. Errors are `E888` messages.
pub fn parse_fmu(v: &Y) -> Result<FmuDecl, Vec<String>> {
    let Y::Mapping(m) = v else { return Err(vec!["`fmu:` must be a mapping {file, fmiVersion, kind, sha256, parameters}".into()]) };
    let mut errs = Vec::new();
    for k in m.keys().filter_map(|k| k.as_str()) {
        if !["file", "fmiVersion", "kind", "sha256", "parameters"].contains(&k) {
            errs.push(format!("`fmu:` has an unknown key `{k}` (file, fmiVersion, kind, sha256, parameters)"));
        }
    }
    let file = field(m, "file").and_then(text);
    if file.is_none() {
        errs.push("`fmu:` needs a `file:`".into());
    }
    if let Some(k) = field(m, "kind").and_then(text) {
        if k != "cosimulation" {
            errs.push(format!("`fmu: kind: {k}` is not supported (only `cosimulation`)"));
        }
    }
    let mut parameters = Vec::new();
    match field(m, "parameters") {
        None => {}
        Some(Y::Mapping(pm)) => {
            for (k, val) in pm {
                let Some(name) = k.as_str() else { continue };
                match text(val).and_then(|t| expr::parse(&t).ok()).and_then(|e| e.eval(&|_: &str| None).ok()) {
                    Some(v @ (Value::Num(_) | Value::Bool(_))) => parameters.push((name.to_string(), v)),
                    _ => errs.push(format!("`fmu: parameters: {name}` does not evaluate to a number or boolean")),
                }
            }
        }
        Some(_) => errs.push("`fmu: parameters:` must be a mapping {name: value}".into()),
    }
    if errs.is_empty() {
        Ok(FmuDecl { file: file.unwrap(), fmi_version: field(m, "fmiVersion").and_then(text), sha256: field(m, "sha256").and_then(text), parameters })
    } else {
        Err(errs)
    }
}

impl FmuDecl {
    /// The declared parameters in the plant's own units, checked against its variables. A plain number is taken
    /// in the FMU's unit; a quantity must have the variable's dimension and is converted.
    pub fn raw_parameters(&self, info: &FmuInfo) -> Result<Vec<(String, f64)>, Vec<String>> {
        let mut out = Vec::new();
        let mut errs = Vec::new();
        for (name, value) in &self.parameters {
            let Some(var) = info.var(name) else {
                errs.push(format!("`fmu: parameters: {name}` is not a variable of the FMU"));
                continue;
            };
            if !matches!(var.causality, Causality::Parameter | Causality::Input) {
                errs.push(format!("`fmu: parameters: {name}` is not a parameter or input of the FMU"));
                continue;
            }
            match value {
                Value::Bool(b) => out.push((name.clone(), f64::from(*b))),
                Value::Num(q) if q.dim == Dim([0; 7]) => out.push((name.clone(), q.v)),
                Value::Num(q) if Some(q.dim) == var.dim => out.push((name.clone(), (q.v - var.offset) / var.factor)),
                Value::Num(q) => errs.push(format!("`fmu: parameters: {name}` has dimension {} but the FMU variable has {}", q.dim.human(), var.dim.map(|d| d.human()).unwrap_or_else(|| "none".into()))),
                Value::Str(_) => errs.push(format!("`fmu: parameters: {name}` must be a number or boolean")),
            }
        }
        if errs.is_empty() {
            Ok(out)
        } else {
            Err(errs)
        }
    }
}

/// The wiring fields of a TestCase.
#[derive(Debug, Clone, Default)]
pub struct Wiring {
    pub plants: Vec<String>,
    pub step: Option<f64>,
    pub connections: Vec<(String, String)>,
}

/// Whether a frontmatter declares any plant field.
pub fn declares_plants(fm: &crate::element::RawFrontmatter) -> bool {
    fm.fmu.is_some() || fm.plants.is_some() || fm.plant_step.is_some() || fm.plant_connections.is_some()
}

/// Read the wiring fields of a TestCase. Errors are `E888` messages.
pub fn parse_wiring(fm: &crate::element::RawFrontmatter) -> Result<Wiring, Vec<String>> {
    let mut errs = Vec::new();
    let mut w = Wiring::default();
    match &fm.plants {
        None => {}
        Some(Y::Sequence(items)) => {
            for (i, it) in items.iter().enumerate() {
                match it {
                    Y::String(s) => w.plants.push(s.clone()),
                    _ => errs.push(format!("`plants[{i}]` must be a name")),
                }
            }
        }
        Some(_) => errs.push("`plants:` must be a list of Part names".to_string()),
    }
    if let Some(v) = &fm.plant_step {
        let Some(s) = text(v) else {
            errs.push("`plantStep:` must be a duration such as \"10 ms\"".to_string());
            return Err(errs);
        };
        match expr::parse(&s).and_then(|e| e.eval(&|_: &str| None)) {
            Ok(Value::Num(q)) if Some(q.dim) == expr::unit_scale("s").map(|x| x.1) && q.v > 0.0 => w.step = Some(q.v),
            _ => errs.push(format!("`plantStep: {s}` is not a positive duration")),
        }
    }
    for (i, c) in fm.plant_connections.iter().flatten().enumerate() {
        match c.as_mapping().and_then(|m| Some((field(m, "from").and_then(text)?, field(m, "to").and_then(text)?))) {
            Some(pair) => w.connections.push(pair),
            None => errs.push(format!("`plantConnections[{i}]` must be {{from, to}}")),
        }
    }
    if errs.is_empty() {
        Ok(w)
    } else {
        Err(errs)
    }
}

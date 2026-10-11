//! Plant co-simulation (REQ-TRS-FMI-001, GH #291): the dependency-free seam between the state-machine
//! interpreter and plant models such as FMI Co-Simulation FMUs, and the master-side bookkeeping (connections,
//! unit and type checks, SI conversion) shared by every backend. The FMI backend itself lives in the optional
//! `syscribe-fmi` crate; this module has no FMI knowledge and is always compiled.

use std::path::Path;
use std::sync::OnceLock;

use serde::Serialize;

use crate::expr::{Quantity, Value};
use crate::ir::Machine;
use crate::units::Dim;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Causality {
    Input,
    Output,
    /// A tunable parameter: may be written, not read.
    Parameter,
    CalculatedParameter,
    Local,
    Independent,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VarType {
    Real,
    Integer,
    Boolean,
    /// Strings, binaries, enumerations, clocks: not supported in v1.
    Other,
}

/// One variable of a plant, as declared in its model description. `value_in_si = value * factor + offset`.
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub name: String,
    pub causality: Causality,
    pub ty: VarType,
    /// SI base-unit dimension; `None` for a dimensionless variable.
    pub dim: Option<Dim>,
    pub factor: f64,
    pub offset: f64,
}

#[derive(Debug, Clone)]
pub struct FmuInfo {
    pub fmi_version: String,
    pub guid: String,
    pub sha256: String,
    pub supports_cosimulation: bool,
    pub variables: Vec<VarInfo>,
}

impl FmuInfo {
    pub fn var(&self, name: &str) -> Option<&VarInfo> {
        self.variables.iter().find(|v| v.name == name)
    }
}

/// A plant instance. Values cross this interface in the plant's own units; the master converts.
pub trait Plant {
    fn info(&self) -> &FmuInfo;
    fn set(&mut self, name: &str, value: f64) -> Result<(), String>;
    fn get(&mut self, name: &str) -> Result<f64, String>;
    /// Advance from communication point `t` by `h`.
    fn step(&mut self, t: f64, h: f64) -> Result<(), String>;
    /// The plant asked to end the simulation.
    fn terminated(&self) -> bool {
        false
    }
    /// What identifies this plant in recorded evidence.
    fn provenance(&self) -> serde_json::Value;
}

/// A way to open plants. One backend is registered at start-up when the `fmi` feature is built.
pub trait PlantBackend: Send + Sync {
    fn name(&self) -> String;
    /// Read an FMU's description without instantiating it.
    fn inspect(&self, file: &Path) -> Result<FmuInfo, String>;
    /// Instantiate for Co-Simulation at time 0 with `parameters` (plant units) applied.
    fn instantiate(&self, file: &Path, parameters: &[(String, f64)]) -> Result<Box<dyn Plant>, String>;
}

static BACKEND: OnceLock<Box<dyn PlantBackend>> = OnceLock::new();

/// Register the plant backend; returns false if one was already registered.
pub fn register_backend(b: Box<dyn PlantBackend>) -> bool {
    BACKEND.set(b).is_ok()
}

pub fn backend() -> Option<&'static dyn PlantBackend> {
    BACKEND.get().map(|b| b.as_ref())
}

struct Link {
    plant: usize,
    var: VarInfo,
    machine_slot: usize,
    machine_dim: Dim,
    machine_bool: bool,
    to_plant: bool,
    spec: String,
}

/// The plants of a run, their connections to machine variables, and the communication step.
pub struct CoSim {
    plants: Vec<(String, Box<dyn Plant>)>,
    links: Vec<Link>,
    pub step: f64,
}

const NONE: Dim = Dim([0; 7]);

impl CoSim {
    /// Check the connections against the machine's variables and the plants' declared variables.
    /// `connections` are `(from, to)`; one end is `<plant>.<variable>`, the other a machine variable.
    pub fn new(plants: Vec<(String, Box<dyn Plant>)>, connections: Vec<(String, String)>, step: f64, m: &Machine) -> Result<CoSim, Vec<String>> {
        let infos: Vec<(&str, &FmuInfo)> = plants.iter().map(|(n, p)| (n.as_str(), p.info())).collect();
        match build_links(&infos, &connections, step, m) {
            (links, errs) if errs.is_empty() => Ok(CoSim { plants, links, step }),
            (_, errs) => Err(errs),
        }
    }



    /// Plant outputs → machine variables (converted to SI). Returns nothing; the caller diffs the variables.
    pub(crate) fn read(&mut self, vars: &mut [Value]) -> Result<(), String> {
        for l in self.links.iter().filter(|l| !l.to_plant) {
            let (pname, plant) = &mut self.plants[l.plant];
            vars[l.machine_slot] = read_si(plant.as_mut(), pname, &l.var, l.machine_bool, l.machine_dim)?;
        }
        Ok(())
    }

    /// Machine variables → plant inputs (converted from SI to the plant's units).
    pub(crate) fn write(&mut self, vars: &[Value]) -> Result<(), String> {
        for l in self.links.iter().filter(|l| l.to_plant) {
            let (pname, plant) = &mut self.plants[l.plant];
            write_si(plant.as_mut(), pname, &l.var, &vars[l.machine_slot]).map_err(|e| format!("connection {}: {e}", l.spec))?;
        }
        Ok(())
    }

    pub(crate) fn step_all(&mut self, t: f64, h: f64) -> Result<(), String> {
        for (name, p) in &mut self.plants {
            p.step(t, h).map_err(|e| format!("plant '{name}': {e}"))?;
            if p.terminated() {
                return Err(format!("plant '{name}' asked to terminate the simulation"));
            }
        }
        Ok(())
    }

    /// Per plant: its name, the communication step and what the plant reports about itself.
    pub fn provenance(&self) -> Vec<serde_json::Value> {
        self.plants.iter().map(|(n, p)| serde_json::json!({"name": n, "step": self.step, "plant": p.provenance()})).collect()
    }
}

/// Check the connections against the machine's variables and the plants' declared variables.
/// `connections` are `(from, to)`; one end is `<plant>.<variable>`, the other a machine variable.
fn build_links(infos: &[(&str, &FmuInfo)], connections: &[(String, String)], step: f64, m: &Machine) -> (Vec<Link>, Vec<String>) {
    let mut errs: Vec<String> = Vec::new();
    if !(step.is_finite() && step > 0.0) {
        errs.push(format!("the communication step must be a positive duration, got {step}"));
    }
    for (i, (n, _)) in infos.iter().enumerate() {
        if infos[..i].iter().any(|x| x.0 == *n) {
            errs.push(format!("two plants are named '{n}'; plant names must be distinct"));
        }
    }
    let mut links = Vec::new();
    for (from, to) in connections {
        let spec = format!("{from} → {to}");
        let plant_end = |s: &str| -> Option<(usize, String)> {
            let (p, v) = s.split_once('.')?;
            infos.iter().position(|x| x.0 == p).map(|i| (i, v.to_string()))
        };
        let (pe_from, pe_to) = (plant_end(from), plant_end(to));
        let (to_plant, (pi, pvar), mname) = match (pe_from, pe_to) {
            (Some(p), None) => (false, p, &to[..]),
            (None, Some(p)) => (true, p, &from[..]),
            (Some(_), Some(_)) => {
                errs.push(format!("connection {spec}: both ends are plant variables; one end must be a machine variable"));
                continue;
            }
            (None, None) => {
                errs.push(format!("connection {spec}: neither end names a plant variable (`<plant>.<variable>`); plants are: {}", infos.iter().map(|p| p.0).collect::<Vec<_>>().join(", ")));
                continue;
            }
        };
        let Some(var) = infos[pi].1.var(&pvar) else {
            errs.push(format!("connection {spec}: plant '{}' has no variable '{pvar}'", infos[pi].0));
            continue;
        };
        let Some(slot) = m.variables.iter().position(|v| v.name == mname) else {
            errs.push(format!("connection {spec}: '{mname}' is not a variable of the machine"));
            continue;
        };
        match check_exchange(infos[pi].0, var, to_plant, mname, &m.variables[slot].value) {
            Ok((mdim, mbool)) => {
                links.push(Link { plant: pi, var: var.clone(), machine_slot: slot, machine_dim: mdim, machine_bool: mbool, to_plant, spec });
            }
            Err(e) => errs.push(format!("connection {spec}: {e}")),
        }
    }
    (links, errs)
}

/// Check one exchange between a plant variable and a machine variable holding `machine_value`: the causality allows the
/// direction, the types agree and the unit dimensions are equal. Returns the machine side's dimension and whether it is boolean.
pub fn check_exchange(plant: &str, var: &VarInfo, to_plant: bool, machine_name: &str, machine_value: &Value) -> Result<(Dim, bool), String> {
    let pvar = &var.name;
    let ok_dir = if to_plant { matches!(var.causality, Causality::Input) } else { matches!(var.causality, Causality::Output | Causality::CalculatedParameter) };
    if !ok_dir {
        return Err(format!(
            "'{plant}.{pvar}' is {} and cannot be {}",
            match var.causality {
                Causality::Input => "an input",
                Causality::Output => "an output",
                Causality::Parameter => "a parameter",
                Causality::CalculatedParameter => "a calculated parameter",
                _ => "not exchangeable",
            },
            if to_plant { "written (only inputs can; set parameters with `fmu: parameters:`)" } else { "read (only outputs and calculated parameters can)" }
        ));
    }
    let (mdim, mbool) = match machine_value {
        Value::Num(q) => (q.dim, false),
        Value::Bool(_) => (NONE, true),
        Value::Str(_) => return Err(format!("machine variable '{machine_name}' is a string; strings are not supported")),
    };
    match (var.ty, mbool) {
        (VarType::Other, _) => return Err(format!("'{pvar}' has a type that is not supported (supported: Real, Integer, Boolean)")),
        (VarType::Boolean, false) | (VarType::Real | VarType::Integer, true) => {
            return Err(format!("type mismatch: '{pvar}' is {:?} but '{machine_name}' is {}", var.ty, if mbool { "boolean" } else { "a number" }));
        }
        _ => {}
    }
    if !mbool && var.dim.unwrap_or(NONE) != mdim {
        return Err(format!("dimension mismatch: '{pvar}' has dimension {} but '{machine_name}' has {}", var.dim.unwrap_or(NONE).human(), mdim.human()));
    }
    Ok((mdim, mbool))
}

/// Read a plant output as a machine value in SI base units.
pub fn read_si(plant: &mut dyn Plant, plant_name: &str, var: &VarInfo, machine_bool: bool, machine_dim: Dim) -> Result<Value, String> {
    let raw = plant.get(&var.name).map_err(|e| format!("plant '{plant_name}': reading '{}': {e}", var.name))?;
    if !raw.is_finite() {
        return Err(format!("plant '{plant_name}': '{}' is not finite ({raw})", var.name));
    }
    Ok(if machine_bool {
        Value::Bool(raw != 0.0)
    } else {
        let si = raw * var.factor + var.offset;
        Value::Num(Quantity { v: if var.ty == VarType::Integer { si.round() } else { si }, dim: machine_dim })
    })
}

/// Write a machine value (SI) to a plant input, converted to the plant's unit.
pub fn write_si(plant: &mut dyn Plant, plant_name: &str, var: &VarInfo, value: &Value) -> Result<(), String> {
    let raw = match value {
        Value::Bool(b) => f64::from(*b),
        Value::Num(q) => (q.v - var.offset) / var.factor,
        Value::Str(_) => return Err(format!("'{}' is a string", var.name)),
    };
    plant.set(&var.name, raw).map_err(|e| format!("plant '{plant_name}': writing '{}': {e}", var.name))
}

/// The errors the connections would raise, without instantiating anything (static validation).
pub fn check_connections(infos: &[(&str, &FmuInfo)], connections: &[(String, String)], step: f64, m: &Machine) -> Vec<String> {
    build_links(infos, connections, step, m).1
}

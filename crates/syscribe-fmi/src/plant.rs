//! Instantiating and stepping Co-Simulation FMUs through the `fmi` crate (FMI 2.0 and 3.0).

use std::collections::HashMap;
use std::path::Path;

use fmi::fmi2::import::Fmi2Import;
use fmi::fmi2::instance::{CoSimulation as _, Common as _};
use fmi::fmi3::import::Fmi3Import;
use fmi::fmi3::{CoSimulation as _, Common as _, GetSet as _};
use fmi::import;
use fmi::fmi3::Fmi3Model as _;
use syscribe_model::plant::{FmuInfo, Plant, VarType};

use crate::md::{self, MdVar};

enum Inner {
    V2(fmi::fmi2::instance::InstanceCS),
    V3(fmi::fmi3::instance::InstanceCS),
}

pub struct FmiPlant {
    // The instance must drop before the import that owns the extracted archive and the loaded library.
    inner: Inner,
    _import: Box<dyn std::any::Any>,
    info: FmuInfo,
    vars: HashMap<String, (u32, VarType)>,
    provenance: serde_json::Value,
}

impl Drop for FmiPlant {
    /// FMI requires `Terminate` before the instance is freed (also on the error path of `instantiate`).
    fn drop(&mut self) {
        match &mut self.inner {
            Inner::V2(i) => {
                let _ = i.terminate();
            }
            Inner::V3(i) => {
                let _ = i.terminate();
            }
        }
    }
}

pub fn instantiate(file: &Path, parameters: &[(String, f64)]) -> Result<Box<dyn Plant>, String> {
    let desc = md::read(file)?;
    let info = md::inspect(file)?;
    if !info.supports_cosimulation {
        return Err(format!("{} does not support Co-Simulation", file.display()));
    }
    let vars: HashMap<String, (u32, VarType)> = desc.variables.iter().map(|v: &MdVar| (v.name.clone(), (v.value_reference, v.ty))).collect();
    let provenance = serde_json::json!({
        "kind": "fmi",
        "file": file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        "parameters": parameters.iter().map(|(n, v)| serde_json::json!({"name": n, "value": v})).collect::<Vec<_>>(),
        "fmiVersion": desc.fmi_version,
        "mode": "cosimulation",
        "modelName": desc.model_name,
        "modelIdentifier": desc.co_simulation_identifier,
        "guid": desc.guid,
        "sha256": info.sha256,
        "backend": format!("syscribe-fmi {} (fmi crate 0.8)", env!("CARGO_PKG_VERSION")),
        "platform": format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
    });
    let mut plant = if desc.fmi_version.starts_with("2.") {
        let imp: Fmi2Import = import::from_path(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let mut inst = imp.instantiate_cs("syscribe", false, false).map_err(|e| format!("instantiating {}: {e}", file.display()))?;
        inst.setup_experiment(None, 0.0, None).map_err(|e| format!("setup_experiment: {e}"))?;
        inst.enter_initialization_mode().map_err(|e| format!("enter_initialization_mode: {e}"))?;
        FmiPlant { inner: Inner::V2(inst), _import: Box::new(imp), info, vars, provenance }
    } else if desc.fmi_version.starts_with("3.") {
        let imp: Fmi3Import = import::from_path(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let mut inst = imp.instantiate_cs("syscribe", false, false, false, false, &[]).map_err(|e| format!("instantiating {}: {e}", file.display()))?;
        inst.enter_initialization_mode(None, 0.0, None).map_err(|e| format!("enter_initialization_mode: {e}"))?;
        FmiPlant { inner: Inner::V3(inst), _import: Box::new(imp), info, vars, provenance }
    } else {
        return Err(format!("unsupported FMI version {}", desc.fmi_version));
    };
    for (n, v) in parameters {
        plant.set(n, *v)?;
    }
    match &mut plant.inner {
        Inner::V2(i) => i.exit_initialization_mode().map(|_| ()).map_err(|e| format!("exit_initialization_mode: {e}"))?,
        Inner::V3(i) => i.exit_initialization_mode().map(|_| ()).map_err(|e| format!("exit_initialization_mode: {e}"))?,
    }
    Ok(Box::new(plant))
}

impl Plant for FmiPlant {
    fn info(&self) -> &FmuInfo {
        &self.info
    }

    fn set(&mut self, name: &str, value: f64) -> Result<(), String> {
        let &(vr, ty) = self.vars.get(name).ok_or_else(|| format!("no variable '{name}'"))?;
        let r: Result<(), String> = match (&mut self.inner, ty) {
            (Inner::V2(i), VarType::Real) => i.set_real(&[vr], &[value]).map(|_| ()).map_err(|e| e.to_string()),
            (Inner::V2(i), VarType::Integer) => i.set_integer(&[vr], &[value.round() as i32]).map(|_| ()).map_err(|e| e.to_string()),
            (Inner::V2(i), VarType::Boolean) => i.set_boolean(&[vr], &[i32::from(value != 0.0)]).map(|_| ()).map_err(|e| e.to_string()),
            (Inner::V3(i), VarType::Real) => i.set_float64(&[vr], &[value]).map(|_| ()).map_err(|e| e.to_string()),
            (Inner::V3(i), VarType::Integer) => i.set_int32(&[vr], &[value.round() as i32]).map(|_| ()).map_err(|e| e.to_string()),
            (Inner::V3(i), VarType::Boolean) => i.set_boolean(&[vr], &[value != 0.0]).map(|_| ()).map_err(|e| e.to_string()),
            (_, VarType::Other) => Err(format!("'{name}' has an unsupported type")),
        };
        r
    }

    fn get(&mut self, name: &str) -> Result<f64, String> {
        let &(vr, ty) = self.vars.get(name).ok_or_else(|| format!("no variable '{name}'"))?;
        match (&mut self.inner, ty) {
            (Inner::V2(i), VarType::Real) => {
                let mut v = [0.0f64];
                i.get_real(&[vr], &mut v).map_err(|e| e.to_string())?;
                Ok(v[0])
            }
            (Inner::V2(i), VarType::Integer) => {
                let mut v = [0i32];
                i.get_integer(&[vr], &mut v).map_err(|e| e.to_string())?;
                Ok(f64::from(v[0]))
            }
            (Inner::V2(i), VarType::Boolean) => {
                let mut v = [0i32];
                i.get_boolean(&[vr], &mut v).map_err(|e| e.to_string())?;
                Ok(f64::from(v[0] != 0))
            }
            (Inner::V3(i), VarType::Real) => {
                let mut v = [0.0f64];
                i.get_float64(&[vr], &mut v).map_err(|e| e.to_string())?;
                Ok(v[0])
            }
            (Inner::V3(i), VarType::Integer) => {
                let mut v = [0i32];
                i.get_int32(&[vr], &mut v).map_err(|e| e.to_string())?;
                Ok(f64::from(v[0]))
            }
            (Inner::V3(i), VarType::Boolean) => {
                let mut v = [false];
                i.get_boolean(&[vr], &mut v).map_err(|e| e.to_string())?;
                Ok(f64::from(v[0]))
            }
            (_, VarType::Other) => Err(format!("'{name}' has an unsupported type")),
        }
    }

    fn step(&mut self, t: f64, h: f64) -> Result<(), String> {
        match &mut self.inner {
            Inner::V2(i) => i.do_step(t, h, true).map(|_| ()).map_err(|e| e.to_string()),
            Inner::V3(i) => {
                let (mut event, mut terminate, mut early, mut last) = (false, false, false, 0.0);
                i.do_step(t, h, true, &mut event, &mut terminate, &mut early, &mut last).map(|_| ()).map_err(|e| e.to_string())?;
                if terminate {
                    return Err("the FMU asked to terminate the simulation".into());
                }
                Ok(())
            }
        }
    }

    fn provenance(&self) -> serde_json::Value {
        self.provenance.clone()
    }
}


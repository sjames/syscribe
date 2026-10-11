//! `modelDescription.xml` reading: version, GUID, Co-Simulation support, units and variables.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use sha2::{Digest, Sha256};
use syscribe_model::plant::{Causality, FmuInfo, VarInfo, VarType};
use syscribe_model::units::Dim;

/// One variable as declared, with what the backend needs to read and write it.
#[derive(Debug, Clone)]
pub struct MdVar {
    pub name: String,
    pub value_reference: u32,
    pub ty: VarType,
    pub info: VarInfo,
}

#[derive(Debug, Clone, Default)]
pub struct ModelDescription {
    pub fmi_version: String,
    pub guid: String,
    pub model_name: String,
    pub co_simulation_identifier: Option<String>,
    pub variables: Vec<MdVar>,
}

#[derive(Default, Clone)]
struct Unit {
    dim: Option<Dim>,
    factor: f64,
    offset: f64,
}

fn attr(e: &BytesStart, name: &str) -> Option<String> {
    e.attributes().flatten().find(|a| a.key.as_ref() == name.as_bytes()).and_then(|a| a.normalized_value(quick_xml::XmlVersion::Implicit1_0).ok().map(|v| v.into_owned()))
}

fn num(e: &BytesStart, name: &str) -> Option<f64> {
    attr(e, name).and_then(|v| v.parse().ok())
}

fn int(e: &BytesStart, name: &str) -> i8 {
    attr(e, name).and_then(|v| v.parse::<i8>().ok()).unwrap_or(0)
}

/// FMI base-unit exponents → syscribe's `Dim` order [L, M, T, I, Θ, N, J]; `rad` is dimensionless here.
fn base_unit(e: &BytesStart) -> Unit {
    let dim = Dim([int(e, "m"), int(e, "kg"), int(e, "s"), int(e, "A"), int(e, "K"), int(e, "mol"), int(e, "cd")]);
    Unit { dim: Some(dim), factor: num(e, "factor").filter(|f| f.is_finite() && *f != 0.0).unwrap_or(1.0), offset: num(e, "offset").unwrap_or(0.0) }
}

fn causality(s: Option<&str>) -> Causality {
    match s.unwrap_or("local") {
        "input" => Causality::Input,
        "output" => Causality::Output,
        "parameter" | "structuralParameter" => Causality::Parameter,
        "calculatedParameter" => Causality::CalculatedParameter,
        "independent" => Causality::Independent,
        "local" => Causality::Local,
        _ => Causality::Other,
    }
}

fn var_type(tag: &str) -> VarType {
    match tag {
        // Float32 would need the 32-bit accessors; it is reported as unsupported for now
        "Real" | "Float64" => VarType::Real,
        "Integer" | "Int32" => VarType::Integer,
        "Boolean" => VarType::Boolean,
        _ => VarType::Other,
    }
}

/// Parse a `modelDescription.xml` (FMI 2.0 or 3.0).
pub fn parse_model_description(xml: &str) -> Result<ModelDescription, String> {
    let mut r = Reader::from_str(xml);
    let mut md = ModelDescription::default();
    let mut units: HashMap<String, Unit> = HashMap::new();
    let mut types: HashMap<String, Option<String>> = HashMap::new();
    let mut cur_unit: Option<String> = None;
    let mut cur_type: Option<String> = None;
    let mut cur_sv: Option<(String, u32, Causality)> = None; // FMI 2: the open ScalarVariable
    let mut raw_vars: Vec<(String, u32, Causality, String, Option<String>, Option<String>)> = Vec::new(); // name, vr, causality, tag, unit, declaredType
    let mut in_vars = false;
    let mut in_types = false;
    loop {
        let ev = r.read_event().map_err(|e| format!("modelDescription.xml: {e}"))?;
        let (e, empty) = match &ev {
            Event::Start(e) => (Some(e.clone()), false),
            Event::Empty(e) => (Some(e.clone()), true),
            Event::End(e) => {
                match e.name().as_ref() {
                    b"Unit" => cur_unit = None,
                    b"ModelVariables" => in_vars = false,
                    b"TypeDefinitions" => in_types = false,
                    b"SimpleType" => cur_type = None,
                    b"ScalarVariable" => cur_sv = None,
                    _ => {}
                }
                (None, false)
            }
            Event::Eof => break,
            _ => (None, false),
        };
        let Some(e) = e else { continue };
        let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
        match tag.as_str() {
            "fmiModelDescription" => {
                md.fmi_version = attr(&e, "fmiVersion").unwrap_or_default();
                md.guid = attr(&e, "guid").or_else(|| attr(&e, "instantiationToken")).unwrap_or_default();
                md.model_name = attr(&e, "modelName").unwrap_or_default();
            }
            "CoSimulation" if !in_vars => md.co_simulation_identifier = attr(&e, "modelIdentifier"),
            "Unit" => {
                if let Some(n) = attr(&e, "name") {
                    units.entry(n.clone()).or_default();
                    if !empty {
                        cur_unit = Some(n);
                    }
                }
            }
            "BaseUnit" => {
                if let Some(n) = &cur_unit {
                    units.insert(n.clone(), base_unit(&e));
                }
            }
            "TypeDefinitions" => in_types = !empty,
            "SimpleType" | "Float64Type" | "Float32Type" | "Int32Type" if in_types || tag != "SimpleType" => {
                if let Some(n) = attr(&e, "name") {
                    types.insert(n.clone(), attr(&e, "unit"));
                    if tag == "SimpleType" && !empty {
                        cur_type = Some(n);
                    }
                }
            }
            "Real" if cur_type.is_some() && in_types => {
                if let Some(t) = &cur_type {
                    types.insert(t.clone(), attr(&e, "unit"));
                }
            }
            "ModelVariables" => in_vars = !empty,
            "ScalarVariable" if in_vars => {
                let name = attr(&e, "name").unwrap_or_default();
                let vr = attr(&e, "valueReference").and_then(|v| v.parse().ok()).ok_or_else(|| format!("variable '{name}' has no valid valueReference"))?;
                let c = causality(attr(&e, "causality").as_deref());
                if !empty {
                    cur_sv = Some((name, vr, c));
                }
            }
            t if in_vars && cur_sv.is_some() && matches!(t, "Real" | "Integer" | "Boolean" | "String" | "Enumeration") => {
                let (name, vr, c) = cur_sv.clone().unwrap();
                raw_vars.push((name, vr, c, t.to_string(), attr(&e, "unit"), attr(&e, "declaredType")));
            }
            t if in_vars
                && cur_sv.is_none()
                && matches!(t, "Float32" | "Float64" | "Int8" | "UInt8" | "Int16" | "UInt16" | "Int32" | "UInt32" | "Int64" | "UInt64" | "Boolean" | "String" | "Binary" | "Enumeration" | "Clock") =>
            {
                // FMI 3.0: the variable is the element itself
                let name = attr(&e, "name").unwrap_or_default();
                let vr = attr(&e, "valueReference").and_then(|v| v.parse().ok()).ok_or_else(|| format!("variable '{name}' has no valid valueReference"))?;
                raw_vars.push((name, vr, causality(attr(&e, "causality").as_deref()), t.to_string(), attr(&e, "unit"), attr(&e, "declaredType")));
            }
            _ => {}
        }
    }
    if md.fmi_version.is_empty() {
        return Err("modelDescription.xml has no fmiModelDescription/@fmiVersion".into());
    }
    for (name, vr, c, tag, unit, declared) in raw_vars {
        let unit_name = unit.or_else(|| declared.and_then(|d| types.get(&d).cloned().flatten()));
        let (dim, factor, offset) = match unit_name.as_deref() {
            None => (None, 1.0, 0.0),
            Some(u) => match units.get(u) {
                Some(Unit { dim: Some(d), factor, offset }) => (Some(*d), *factor, *offset),
                // a unit with no BaseUnit: fall back to syscribe's own unit table by name
                _ => match syscribe_model::expr::unit_scale(u) {
                    Some((scale, d)) => (Some(d), scale, 0.0),
                    None => return Err(format!("variable '{name}' has unit '{u}', which has no BaseUnit and is not a unit syscribe knows")),
                },
            },
        };
        let ty = var_type(&tag);
        let dim = dim.filter(|d| *d != Dim([0; 7]));
        md.variables.push(MdVar { name: name.clone(), value_reference: vr, ty, info: VarInfo { name, causality: c, ty, dim, factor, offset } });
    }
    Ok(md)
}

fn read_md_from_zip(file: &Path) -> Result<String, String> {
    let f = std::fs::File::open(file).map_err(|e| e.to_string())?;
    let mut z = zip::ZipArchive::new(f).map_err(|e| format!("not a zip archive: {e}"))?;
    let mut entry = z.by_name("modelDescription.xml").map_err(|_| "no modelDescription.xml in the archive".to_string())?;
    let mut s = String::new();
    entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
    Ok(s)
}

pub fn sha256_hex(file: &Path) -> Result<String, String> {
    let bytes = std::fs::read(file).map_err(|e| e.to_string())?;
    Ok(Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect())
}

pub fn read(file: &Path) -> Result<ModelDescription, String> {
    parse_model_description(&read_md_from_zip(file)?)
}

pub fn inspect(file: &Path) -> Result<FmuInfo, String> {
    let md = read(file)?;
    Ok(FmuInfo {
        fmi_version: md.fmi_version.clone(),
        guid: md.guid.clone(),
        sha256: sha256_hex(file)?,
        supports_cosimulation: md.co_simulation_identifier.is_some(),
        variables: md.variables.iter().map(|v| v.info.clone()).collect(),
    })
}

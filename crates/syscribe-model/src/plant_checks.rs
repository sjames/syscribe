//! The static checks of REQ-TRS-FMI-001: `E888` (declarations and wiring), `E889` (connections), `E966` (the
//! FMU artifact). Run only when a plant backend is registered, i.e. in a build with the `fmi` feature.

use crate::config::{SourceLocation, ValidateConfig};
use crate::element::{ElementType, RawElement};
use crate::plant::{backend, check_connections, FmuInfo};
use crate::plant_cfg::{declares_plants, parse_fmu, parse_wiring};
use crate::resolver::Resolver;

pub type Finding = (&'static str, String);

/// Inspect the FMU file of `elem`'s declaration: the info, plus `E966`/`E888` findings.
fn inspect(elem: &RawElement, config: &ValidateConfig, out: &mut Vec<Finding>) -> Option<FmuInfo> {
    let decl = match parse_fmu(elem.frontmatter.fmu.as_ref()?) {
        Ok(d) => d,
        Err(es) => {
            out.extend(es.into_iter().map(|e| ("E888", format!("{}: {e}", elem.qualified_name))));
            return None;
        }
    };
    let be = backend()?;
    let path = match config.classify_source(&decl.file) {
        SourceLocation::Local(p) => p,
        SourceLocation::Remote(u) => {
            out.push(("E966", format!("{}: the FMU '{u}' is remote; only local files can be read in this version", elem.qualified_name)));
            return None;
        }
    };
    if !path.exists() {
        out.push(("E966", format!("{}: the FMU '{}' does not exist ({})", elem.qualified_name, decl.file, path.display())));
        return None;
    }
    let info = match be.inspect(&path) {
        Ok(i) => i,
        Err(e) => {
            out.push(("E966", format!("{}: '{}' cannot be read as an FMU: {e}", elem.qualified_name, decl.file)));
            return None;
        }
    };
    if !info.supports_cosimulation {
        out.push(("E966", format!("{}: '{}' does not support Co-Simulation", elem.qualified_name, decl.file)));
    }
    if let Some(v) = &decl.fmi_version {
        if *v != info.fmi_version {
            out.push(("E966", format!("{}: fmiVersion is '{v}' but the FMU's modelDescription.xml says '{}'", elem.qualified_name, info.fmi_version)));
        }
    }
    if let Some(h) = &decl.sha256 {
        if !h.eq_ignore_ascii_case(&info.sha256) {
            out.push(("E966", format!("{}: sha256 is {h} but the file hashes to {} (the FMU changed)", elem.qualified_name, info.sha256)));
        }
    }
    if let Err(es) = decl.raw_parameters(&info) {
        out.extend(es.into_iter().map(|e| ("E888", format!("{}: {e}", elem.qualified_name))));
    }
    Some(info)
}

/// Checks for one element that declares plant fields. Call only with a registered backend.
pub fn check(elements: &[RawElement], resolver: &Resolver, config: &ValidateConfig, elem: &RawElement) -> Vec<Finding> {
    let mut out = Vec::new();
    let fm = &elem.frontmatter;
    if !declares_plants(fm) {
        return out;
    }
    if fm.fmu.is_some() {
        if !matches!(fm.element_type, Some(ElementType::Part) | Some(ElementType::PartDef)) {
            out.push(("E888", format!("{}: `fmu:` belongs on a Part or PartDef", elem.qualified_name)));
        }
        inspect(elem, config, &mut out);
    }
    if fm.plants.is_none() && fm.plant_step.is_none() && fm.plant_connections.is_none() {
        return out;
    }
    // a composite part carries only the communication step of its plant parts (REQ-TRS-SYSCOMP-001)
    if matches!(fm.element_type, Some(ElementType::Part) | Some(ElementType::PartDef)) && fm.plants.is_none() && fm.plant_connections.is_none() {
        if let Some(v) = &fm.plant_step {
            let ok = match v {
                serde_yaml::Value::String(s) => crate::system::duration(s).is_some_and(|d| d > 0.0),
                _ => false,
            };
            if !ok {
                out.push(("E888", format!("{}: `plantStep:` must be a positive duration such as \"10 ms\"", elem.qualified_name)));
            }
        }
        return out;
    }
    if !matches!(fm.element_type, Some(ElementType::TestCase)) {
        out.push(("E888", format!("{}: `plants:`, `plantStep:` and `plantConnections:` belong on a TestCase", elem.qualified_name)));
        return out;
    }
    let wiring = match parse_wiring(fm) {
        Ok(w) => w,
        Err(es) => {
            out.extend(es.into_iter().map(|e| ("E888", format!("{}: {e}", elem.qualified_name))));
            return out;
        }
    };
    if wiring.plants.is_empty() {
        out.push(("E888", format!("{}: `plantStep:`/`plantConnections:` need `plants:`", elem.qualified_name)));
        return out;
    }
    if wiring.step.is_none() {
        out.push(("E888", format!("{}: `plants:` needs a `plantStep:` (the maximum communication step)", elem.qualified_name)));
    }
    let subject = fm.subject.as_deref().and_then(|s| resolver.resolve_ref(elements, s)).filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::StateDef) | Some(ElementType::State)));
    if subject.is_none() {
        out.push(("E888", format!("{}: a TestCase with plants needs a state-machine `subject:`", elem.qualified_name)));
    }
    let mut infos: Vec<(String, FmuInfo)> = Vec::new();
    for p in &wiring.plants {
        let Some(pe) = resolver.resolve_ref(elements, p) else {
            out.push(("E888", format!("{}: plant '{p}' does not resolve", elem.qualified_name)));
            continue;
        };
        if pe.frontmatter.fmu.is_none() {
            out.push(("E888", format!("{}: plant '{p}' has no `fmu:` declaration", elem.qualified_name)));
            continue;
        }
        let mut ignored = Vec::new(); // the declaration's own findings are reported on the Part
        if let Some(info) = inspect(pe, config, &mut ignored) {
            let name = pe.qualified_name.rsplit("::").next().unwrap_or(&pe.qualified_name).to_string();
            infos.push((name, info));
        }
    }
    if let (Some(subject), true) = (subject, infos.len() == wiring.plants.len()) {
        if let Ok(machine) = crate::ir::lower(elements, &subject.qualified_name) {
            let refs: Vec<(&str, &FmuInfo)> = infos.iter().map(|(n, i)| (n.as_str(), i)).collect();
            for e in check_connections(&refs, &wiring.connections, wiring.step.unwrap_or(1.0), &machine) {
                out.push(("E889", format!("{}: {e}", elem.qualified_name)));
            }
        }
    }
    out
}

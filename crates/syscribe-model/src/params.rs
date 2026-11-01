//! The late-bound parameter contract (`ADR-SYS-BT-001`, `REQ-TRS-BT-023..030`).
//!
//! A system with parameters bound at `load` or `runtime` needs a file to read them from.
//! This module derives, from a `Configuration` and a stage, the *contract* of that file —
//! which feature toggles and parameters it carries, each with its type, unit, range,
//! enum, requiredness, default, fallback and resolved value — and renders it as:
//!
//! * a deterministic YAML values file ([`Contract::to_yaml`]), keyed by stable `FEAT-*`
//!   id, stamped with a content hash;
//! * a JSON Schema for that file ([`Contract::schema`]);
//!
//! and checks a values file against it ([`check_file`]). The model stays the single
//! source of truth: the file is generated and never read back into the model.
//!
//! An entry is included when its feature is selected (a toggle: when it is a real choice)
//! and its binding window contains the stage. A parameter that is `derivedFrom:` an
//! expression is the system's to compute and is not in the file. A value resolves, in
//! order, from the Configuration chain's binding, the fixed `value:`, `bindTo:`
//! propagation, the `default:`, then the `fallback:`.

use serde_yaml::Value;

use crate::binding_time::{BindingTime, BindingWindow};
use crate::element::{ElementType, RawElement};
use crate::provenance::{provenance, Source};
use crate::staging::{label, StageContext};
use crate::validator::{Finding, Severity};

/// One parameter of the values file.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamEntry {
    /// The owning `FeatureDef`'s stable `FEAT-*` id (its qualified name when it has none).
    pub feature: String,
    pub feature_qname: String,
    pub name: String,
    /// The declared type, e.g. `ScalarValues::Real`.
    pub ty: String,
    pub unit: Option<String>,
    pub range: Option<(f64, f64)>,
    pub enum_values: Option<Vec<String>>,
    pub is_array: bool,
    /// `isRequired: true` and not fixed.
    pub required: bool,
    pub fixed: bool,
    pub default: Option<Value>,
    pub fallback: Option<Value>,
    pub window: BindingWindow,
    /// Elements or source files that read this parameter (`consumedBy:`).
    pub consumed_by: Vec<String>,
    /// The resolved value and where it came from; `None` when nothing supplies one.
    pub value: Option<(Value, String)>,
}

/// One feature toggle of the values file.
#[derive(Debug, Clone, PartialEq)]
pub struct ToggleEntry {
    pub feature: String,
    pub feature_qname: String,
    pub selected: bool,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct Contract {
    pub configuration: String,
    pub stage: BindingTime,
    pub toggles: Vec<ToggleEntry>,
    pub params: Vec<ParamEntry>,
}

fn scalar(v: &Value) -> String {
    match v {
        Value::Sequence(items) => format!("[{}]", items.iter().map(scalar).collect::<Vec<_>>().join(", ")),
        other => serde_yaml::to_string(other).unwrap_or_default().trim_end().trim_start_matches("---").trim().to_string(),
    }
}

fn strings(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Sequence(items)) => items.iter().filter_map(|i| i.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    }
}

fn parse_range(s: &str) -> Option<(f64, f64)> {
    let (lo, hi) = s.split_once("..")?;
    let hi = hi.trim().strip_prefix('=').unwrap_or(hi.trim()).trim();
    Some((lo.trim().parse().ok()?, hi.parse().ok()?))
}

/// Derive the contract of `elements[cfg_idx]` at `stage` (default: the Configuration's own
/// `bindingStage:`). `Err` when neither names a stage.
pub fn contract(elements: &[RawElement], cfg_idx: usize, stage: Option<BindingTime>) -> Result<Contract, String> {
    let ctx = StageContext::new(elements);
    let cfg = &elements[cfg_idx];
    let stage = stage.or_else(|| ctx.stage(cfg_idx)).ok_or_else(|| {
        format!("Configuration '{}' has no bindingStage; name the stage with --stage", label(cfg))
    })?;
    let prov = provenance(elements, cfg_idx);
    let sel = ctx.effective_selection(cfg_idx);
    let mut c = Contract { configuration: label(cfg), stage, toggles: Vec::new(), params: Vec::new() };

    let mut fdefs: Vec<&RawElement> =
        elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef))).collect();
    fdefs.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
    for fd in fdefs {
        let fm = &fd.frontmatter;
        let q = fd.qualified_name.as_str();
        let fid = fm.id.clone().unwrap_or_else(|| q.to_string());
        let is_choice = fm.is_abstract != Some(true) && fm.mandatory != Some(true) && fm.group_kind.as_deref() != Some("mandatory");
        if let (Some(w), true) = (BindingWindow::read(fm.binding_time.as_deref(), fm.latest_binding_time.as_deref()).0, is_choice) {
            if w.contains(stage) {
                let source = prov
                    .selections
                    .iter()
                    .find(|s| s.feature == q)
                    .map_or("unselected".to_string(), |s| source_label(&s.source, s.stage.as_deref()));
                c.toggles.push(ToggleEntry { feature: fid.clone(), feature_qname: q.to_string(), selected: sel.get(q).copied().unwrap_or(false), source });
            }
        }
        if sel.get(q) != Some(&true) {
            continue;
        }
        for p in fm.parameters.iter().flatten() {
            let Value::Mapping(m) = p else { continue };
            let get = |k: &str| m.get(Value::String(k.to_string()));
            let Some(name) = get("name").and_then(|v| v.as_str()) else { continue };
            let Some(window) = BindingWindow::read(get("bindingTime").and_then(|v| v.as_str()), get("latestBindingTime").and_then(|v| v.as_str())).0 else { continue };
            if !window.contains(stage) || get("derivedFrom").is_some() {
                continue;
            }
            let key = format!("{q}.{name}");
            let fixed = get("isFixed").and_then(|v| v.as_bool()).unwrap_or(false) || get("value").is_some();
            let fallback = get("fallback").cloned();
            let value = prov
                .parameters
                .iter()
                .find(|x| x.key == key)
                .map(|x| (x.value.clone(), source_label(&x.source, x.stage.as_deref())))
                .or_else(|| fallback.clone().map(|f| (f, "fallback".to_string())));
            c.params.push(ParamEntry {
                feature: fid.clone(),
                feature_qname: q.to_string(),
                name: name.to_string(),
                ty: get("type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                unit: get("unit").and_then(|v| v.as_str()).map(str::to_string),
                range: get("range").and_then(|v| v.as_str()).and_then(parse_range),
                enum_values: get("enumValues").and_then(|v| v.as_sequence()).map(|s| s.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()),
                is_array: get("isArray").and_then(|v| v.as_bool()).unwrap_or(false),
                required: get("isRequired").and_then(|v| v.as_bool()).unwrap_or(false) && !fixed,
                fixed,
                default: get("default").cloned(),
                fallback,
                window,
                consumed_by: strings(get("consumedBy")),
                value,
            });
        }
    }
    c.toggles.sort_by(|a, b| a.feature.cmp(&b.feature));
    c.params.sort_by(|a, b| (&a.feature, &a.name).cmp(&(&b.feature, &b.name)));
    Ok(c)
}

fn source_label(s: &Source, stage: Option<&str>) -> String {
    match stage {
        Some(st) => format!("{} ({st})", s.describe()),
        None => s.describe(),
    }
}

impl Contract {
    /// Required parameters nothing supplies a value for (no binding, default or fallback).
    pub fn unresolved(&self) -> Vec<&ParamEntry> {
        self.params.iter().filter(|p| p.required && p.value.is_none()).collect()
    }

    /// `blake3:<hex>` of everything the file's meaning depends on: the Configuration, the
    /// stage, and every entry's constraints and value. Independent of the generator
    /// version and of comments, so it changes exactly when the contract does.
    pub fn hash(&self) -> String {
        let mut text = format!("configuration\u{1f}{}\nstage\u{1f}{}\n", self.configuration, self.stage.name());
        for t in &self.toggles {
            text.push_str(&format!("toggle\u{1f}{}\u{1f}{}\n", t.feature, t.selected));
        }
        for p in &self.params {
            let cell = |v: &Option<Value>| v.as_ref().map(scalar).unwrap_or_default();
            text.push_str(&format!(
                "param\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{:?}\u{1f}{:?}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}\n",
                p.feature, p.name, p.ty, p.unit.clone().unwrap_or_default(), p.range, p.enum_values,
                p.is_array, p.required, p.fixed, cell(&p.default), cell(&p.fallback), p.window.describe(),
                p.value.as_ref().map(|(v, _)| scalar(v)).unwrap_or_default(),
            ));
        }
        format!("blake3:{}", blake3::hash(text.as_bytes()).to_hex())
    }

    /// The values file. `allow_open` writes an explicit, marked placeholder (`null`) for
    /// a required parameter nothing supplies; otherwise the caller has already refused.
    pub fn to_yaml(&self, version: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Generated by syscribe {version} — do not edit; the model is the source of truth.\n"));
        out.push_str(&format!("# configuration: {}\n# stage: {}\n# projection-hash: {}\n", self.configuration, self.stage.name(), self.hash()));
        if self.toggles.is_empty() {
            out.push_str("features: {}\n");
        } else {
            out.push_str("features:\n");
            for t in &self.toggles {
                out.push_str(&format!("  {}: {}  # {}\n", t.feature, t.selected, t.source));
            }
        }
        if self.params.is_empty() {
            out.push_str("parameters: {}\n");
            return out;
        }
        out.push_str("parameters:\n");
        let mut last: Option<&str> = None;
        for p in &self.params {
            if last != Some(p.feature.as_str()) {
                out.push_str(&format!("  {}:\n", p.feature));
                last = Some(p.feature.as_str());
            }
            let mut notes: Vec<String> = Vec::new();
            if let Some(u) = &p.unit {
                notes.push(format!("unit: {u}"));
            }
            if let Some((lo, hi)) = p.range {
                notes.push(format!("range: {lo}..{hi}"));
            }
            if let Some(e) = &p.enum_values {
                notes.push(format!("one of: {}", e.join(", ")));
            }
            match &p.value {
                Some((v, src)) => {
                    notes.push(format!("source: {src}"));
                    if src != "fallback" {
                        if let Some(f) = &p.fallback {
                            notes.push(format!("fallback: {}", scalar(f)));
                        }
                    }
                    out.push_str(&format!("    {}: {}  # {}\n", p.name, scalar(v), notes.join("; ")));
                }
                None => {
                    notes.insert(0, "OPEN: required, no value, default or fallback".to_string());
                    out.push_str(&format!("    {}: null  # {}\n", p.name, notes.join("; ")));
                }
            }
            if !p.consumed_by.is_empty() {
                out.push_str(&format!("    # {} is read by: {}\n", p.name, p.consumed_by.join(", ")));
            }
        }
        out
    }

    /// A JSON Schema (2020-12) for the values file.
    pub fn schema(&self) -> serde_json::Value {
        use serde_json::{json, Map, Value as J};
        let json_of = |v: &Value| serde_json::to_value(v).unwrap_or(J::Null);
        let mut toggles = Map::new();
        for t in &self.toggles {
            toggles.insert(t.feature.clone(), json!({"type": "boolean", "description": format!("whether {} is selected", t.feature_qname)}));
        }
        let mut features: std::collections::BTreeMap<&str, Map<String, J>> = std::collections::BTreeMap::new();
        let mut required: std::collections::BTreeMap<&str, Vec<String>> = std::collections::BTreeMap::new();
        for p in &self.params {
            let mut s = Map::new();
            let base = p.ty.rsplit("::").next().unwrap_or("");
            let ty = match base {
                "Real" | "Number" => Some("number"),
                "Integer" | "Natural" => Some("integer"),
                "Boolean" => Some("boolean"),
                "String" => Some("string"),
                _ => None,
            };
            let mut item = Map::new();
            if let Some(t) = ty {
                item.insert("type".into(), t.into());
            }
            if let Some((lo, hi)) = p.range {
                item.insert("minimum".into(), lo.into());
                item.insert("maximum".into(), hi.into());
            }
            if let Some(e) = &p.enum_values {
                item.insert("enum".into(), e.clone().into());
            }
            if p.is_array {
                s.insert("type".into(), "array".into());
                s.insert("items".into(), J::Object(item));
            } else {
                s.extend(item);
            }
            let mut desc = Vec::new();
            if let Some(u) = &p.unit {
                desc.push(format!("unit: {u}"));
            }
            desc.push(format!("bound at {}", p.window.describe()));
            s.insert("description".into(), desc.join("; ").into());
            if let Some(d) = &p.default {
                s.insert("default".into(), json_of(d));
            }
            if let Some(f) = &p.fallback {
                s.insert("x-syscribe-fallback".into(), json_of(f));
            }
            features.entry(p.feature.as_str()).or_default().insert(p.name.clone(), J::Object(s));
            if p.required {
                required.entry(p.feature.as_str()).or_default().push(p.name.clone());
            }
        }
        let mut per_feature = Map::new();
        for (fid, props) in features {
            let mut o = Map::new();
            o.insert("type".into(), "object".into());
            o.insert("properties".into(), J::Object(props));
            if let Some(r) = required.get(fid) {
                o.insert("required".into(), r.clone().into());
            }
            o.insert("additionalProperties".into(), false.into());
            per_feature.insert(fid.to_string(), J::Object(o));
        }
        let feats_required: Vec<&str> = required.keys().copied().collect();
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": format!("Late-bound parameters of {} at stage {}", self.configuration, self.stage.name()),
            "type": "object",
            "properties": {
                "features": {"type": "object", "properties": J::Object(toggles), "additionalProperties": false},
                "parameters": {"type": "object", "properties": J::Object(per_feature), "required": feats_required, "additionalProperties": false},
            },
            "required": ["features", "parameters"],
            "additionalProperties": false,
        })
    }
}

fn finding(code: &'static str, severity: Severity, file: &str, message: String) -> Finding {
    Finding { code, file: file.to_string(), message, severity }
}

/// The header fields of a generated values file (`# key: value` comment lines).
pub fn header_field(text: &str, key: &str) -> Option<String> {
    let prefix = format!("# {key}: ");
    text.lines().take_while(|l| l.starts_with('#')).find_map(|l| l.strip_prefix(prefix.as_str()).map(|v| v.trim().to_string()))
}

fn type_matches(p: &ParamEntry, v: &Value) -> bool {
    if p.is_array {
        return v.as_sequence().is_some_and(|s| s.iter().all(|i| scalar_type_matches(&p.ty, i)));
    }
    scalar_type_matches(&p.ty, v)
}

fn scalar_type_matches(ty: &str, v: &Value) -> bool {
    match ty.rsplit("::").next().unwrap_or("") {
        "Real" | "Number" => v.is_number(),
        "Integer" | "Natural" => v.is_i64() || v.is_u64() || v.as_f64().is_some_and(|f| f.fract() == 0.0),
        "Boolean" => v.is_bool(),
        "String" => v.is_string(),
        _ => !v.is_null(),
    }
}

fn same(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

/// Check a values file against the contract: unknown keys (`E760`), wrong types (`E761`),
/// a different value for a fixed parameter (`E204`), a value outside `range:` (`E205`) or
/// `enumValues:` (`E206`), a missing or still-open required parameter (`E762`), and a file
/// generated from a different model state (`W751`).
pub fn check_file(contract: &Contract, text: &str, file: &str) -> Vec<Finding> {
    let err = |code, msg: String| finding(code, Severity::Error, file, msg);
    let mut f = Vec::new();
    let root: Value = match serde_yaml::from_str(text) {
        Ok(v) => v,
        Err(e) => return vec![err("E761", format!("not valid YAML: {e}"))],
    };
    let Value::Mapping(root) = root else {
        return vec![err("E761", "the file must be a mapping with `features` and `parameters`".to_string())];
    };
    for k in root.keys().filter_map(|k| k.as_str()) {
        if k != "features" && k != "parameters" {
            f.push(err("E760", format!("unknown top-level key '{k}' (expected `features` and `parameters`)")));
        }
    }

    if let Some(Value::Mapping(feats)) = root.get("features") {
        for (k, v) in feats {
            let Some(k) = k.as_str() else { continue };
            match contract.toggles.iter().find(|t| t.feature == k) {
                None => f.push(err("E760", format!("features: unknown feature '{k}'"))),
                Some(_) if !v.is_bool() => f.push(err("E761", format!("features.{k} must be true or false"))),
                Some(_) => {}
            }
        }
    } else if root.contains_key("features") {
        f.push(err("E761", "`features` must be a mapping".to_string()));
    }

    let params_root = match root.get("parameters") {
        Some(Value::Mapping(m)) => Some(m),
        Some(_) => {
            f.push(err("E761", "`parameters` must be a mapping".to_string()));
            None
        }
        None => None,
    };
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for (fk, pv) in params_root.into_iter().flatten() {
        let Some(fid) = fk.as_str() else { continue };
        if !contract.params.iter().any(|p| p.feature == fid) {
            f.push(err("E760", format!("parameters: unknown feature '{fid}'")));
            continue;
        }
        let Value::Mapping(pm) = pv else {
            f.push(err("E761", format!("parameters.{fid} must be a mapping of parameter names to values")));
            continue;
        };
        for (pk, v) in pm {
            let Some(name) = pk.as_str() else { continue };
            let Some(p) = contract.params.iter().find(|p| p.feature == fid && p.name == name) else {
                f.push(err("E760", format!("parameters.{fid}: unknown parameter '{name}'")));
                continue;
            };
            let at = format!("parameters.{fid}.{name}");
            if v.is_null() {
                continue; // an open placeholder: reported below if the parameter is required
            }
            seen.insert((fid.to_string(), name.to_string()));
            if !type_matches(p, v) {
                f.push(err("E761", format!("{at} has the wrong type (declared {}{})", p.ty, if p.is_array { ", an array" } else { "" })));
                continue;
            }
            if p.fixed {
                if let Some((fixed, _)) = &p.value {
                    if !same(fixed, v) {
                        f.push(err("E204", format!("{at} = {} but the parameter is fixed at {}", scalar(v), scalar(fixed))));
                    }
                }
            }
            let items: Vec<&Value> = match v {
                Value::Sequence(s) => s.iter().collect(),
                other => vec![other],
            };
            for item in items {
                if let (Some((lo, hi)), Some(n)) = (p.range, item.as_f64()) {
                    if n < lo || n > hi {
                        f.push(err("E205", format!("{at} = {n} is outside range {lo}..{hi}")));
                    }
                }
                if let (Some(allowed), Some(s)) = (&p.enum_values, item.as_str()) {
                    if !allowed.iter().any(|a| a == s) {
                        f.push(err("E206", format!("{at} = '{s}' is not in enumValues {allowed:?}")));
                    }
                }
            }
        }
    }
    for p in contract.params.iter().filter(|p| p.required) {
        if !seen.contains(&(p.feature.clone(), p.name.clone())) {
            f.push(err("E762", format!("required parameter {}.{} is missing or still open", p.feature, p.name)));
        }
    }

    let expected = contract.hash();
    let stale = match header_field(text, "projection-hash") {
        None => Some("the file carries no projection-hash header".to_string()),
        Some(h) if h != expected => Some(format!("the file was generated from a different model state ({h}, the model now gives {expected})")),
        Some(_) => None,
    };
    if let Some(why) = stale {
        f.push(finding("W751", Severity::Warning, file, format!("stale values file: {why} — regenerate it with `params export`")));
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, ty: &str) -> ParamEntry {
        ParamEntry {
            feature: "FEAT-X".into(), feature_qname: "F::X".into(), name: name.into(), ty: ty.into(), unit: None, range: None,
            enum_values: None, is_array: false, required: false, fixed: false, default: None, fallback: None,
            window: BindingWindow::read(Some("compile"), None).0.unwrap(), consumed_by: vec![], value: None,
        }
    }

    #[test]
    fn scalar_formats_values_stably() {
        assert_eq!(scalar(&Value::from(1000)), "1000");
        assert_eq!(scalar(&Value::String("sport".into())), "sport");
        assert_eq!(scalar(&Value::Bool(true)), "true");
        assert_eq!(scalar(&Value::Sequence(vec![Value::from(1), Value::from(2)])), "[1, 2]");
    }

    #[test]
    fn types_and_header_fields() {
        let real = entry("a", "ScalarValues::Real");
        assert!(type_matches(&real, &Value::from(1.5)) && !type_matches(&real, &Value::String("x".into())));
        let int = entry("b", "ScalarValues::Integer");
        assert!(type_matches(&int, &Value::from(3)) && type_matches(&int, &Value::from(3.0)) && !type_matches(&int, &Value::from(3.5)));
        let mut arr = entry("c", "ScalarValues::Real");
        arr.is_array = true;
        assert!(type_matches(&arr, &Value::Sequence(vec![Value::from(1)])) && !type_matches(&arr, &Value::from(1)));
        let text = "# Generated by syscribe 1 — x\n# stage: load\n# projection-hash: blake3:ab\nfeatures: {}\n";
        assert_eq!(header_field(text, "stage").as_deref(), Some("load"));
        assert_eq!(header_field(text, "projection-hash").as_deref(), Some("blake3:ab"));
        assert_eq!(header_field(text, "missing"), None);
    }
}

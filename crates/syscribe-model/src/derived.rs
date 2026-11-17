//! Derived feature parameters (GH #490, `REQ-TRS-DERIVED-001`): a parameter declared
//! `derivedFrom: "<expression>"` takes the value of that expression over the Configuration's
//! effective parameter values, in the expression language of `syscribe eval`
//! ([`crate::expr`], with `ceil`, `max`, `if … then … else`, and — GH #491 — calculations).
//!
//! The value is computed wherever a bound value is read: `placeholders::value_for` (so `show`,
//! `configure`, `{{Feature.param}}` placeholders and the constraint environment see it),
//! `parameterConstraints`, `build-config` (through the parameter's `buildVar`) and the
//! `range:`/integer checks of [`findings`].

use std::cell::RefCell;
use std::collections::HashMap;

use serde_yaml::Mapping;

use crate::element::{ElementType, RawElement};
use crate::expr::{self, Value};
use crate::resolver::Resolver;
use crate::validator::{Finding, Severity};

thread_local! {
    /// Derived parameters being evaluated, outermost first (a repeat is a cycle).
    static STACK: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// The `derivedFrom:` expression of a parameter entry, when it is one (a string).
pub fn expression(entry: &Mapping) -> Option<&str> {
    entry.get("derivedFrom").and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty())
}

/// Evaluate `src` — the `derivedFrom:` of `feature_qname.param` — for `cfg`. The error says what is
/// missing (`no value for …`), circular or wrong.
pub fn evaluate(elements: &[RawElement], resolver: &Resolver, cfg: Option<&RawElement>, feature_qname: &str, param: &str, src: &str) -> Result<Value, String> {
    let key = format!("{feature_qname}.{param}");
    let cycle = STACK.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(at) = s.iter().position(|k| *k == key) {
            let mut chain: Vec<String> = s[at..].to_vec();
            chain.push(key.clone());
            return Some(chain.join(" -> "));
        }
        s.push(key.clone());
        None
    });
    if let Some(chain) = cycle {
        return Err(format!("circular derivedFrom {chain}"));
    }
    let result = (|| {
        let parsed = expr::parse(src).map_err(|e| e.to_string())?;
        let mut env = crate::constraints::Env::new();
        for name in parsed.names() {
            let Some((feat, p)) = name.rsplit_once('.') else { continue };
            // Only a `Feature.param` of a declared feature is read; anything else (a calculation, an
            // attribute, an enumeration literal) is left to the evaluator.
            let Some(text) = crate::placeholders::param_text(elements, resolver, cfg, feat, p) else { continue };
            if let Some(v) = expr::parse_value(&text) {
                env.insert(name.clone(), v);
            }
        }
        crate::constraints::evaluate_value(elements, src, &env)
    })();
    STACK.with(|s| s.borrow_mut().pop());
    result
}

/// `evaluate`, as the text a bound value is stored and shown as: a dimensionless number plainly
/// (`2000`), a quantity with its unit.
pub fn text(elements: &[RawElement], resolver: &Resolver, cfg: Option<&RawElement>, feature_qname: &str, param: &str, src: &str) -> Result<String, String> {
    evaluate(elements, resolver, cfg, feature_qname, param, src).map(|v| match v {
        Value::Num(q) if q.dim == crate::units::Dim([0; 7]) => expr::format_number(q.v),
        Value::Str(s) => s,
        other => expr::format_value(&other),
    })
}

/// Every derived parameter of the model as `(feature qname, parameter, expression)`.
pub fn declared(elements: &[RawElement]) -> Vec<(&RawElement, String, String)> {
    let mut out = Vec::new();
    for fd in elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef)) {
        for p in fd.frontmatter.parameters.iter().flatten() {
            let serde_yaml::Value::Mapping(m) = p else { continue };
            let (Some(name), Some(src)) = (m.get("name").and_then(|v| v.as_str()), expression(m)) else { continue };
            out.push((fd, name.to_string(), src.to_string()));
        }
    }
    out
}

/// The derived parameters of the model that evaluate to a dimensionless number in `cfg`, as
/// `<feature qname>.<param>` → value (the constraint environment).
pub fn numeric_values(elements: &[RawElement], resolver: &Resolver, cfg: &RawElement) -> HashMap<String, f64> {
    let mut out = HashMap::new();
    for (fd, name, src) in declared(elements) {
        if let Ok(Value::Num(q)) = evaluate(elements, resolver, Some(cfg), &fd.qualified_name, &name, &src) {
            if q.dim == crate::units::Dim([0; 7]) && q.v.is_finite() {
                out.insert(format!("{}.{name}", fd.qualified_name), q.v);
            }
        }
    }
    out
}

fn finding(code: &'static str, file: &str, message: String) -> Finding {
    Finding { code, file: file.to_string(), message, severity: Severity::Error }
}

fn parse_range(s: &str) -> Option<(f64, f64)> {
    let (lo, hi) = s.split_once("..")?;
    let hi = hi.trim();
    let hi = hi.strip_prefix('=').unwrap_or(hi).trim();
    Some((lo.trim().parse().ok()?, hi.parse().ok()?))
}

/// `E248` a `derivedFrom:` expression that does not parse, or that fails for a Configuration whose
/// inputs all have values (a circular reference, a dimension or domain error); `E249` a derived value
/// outside the parameter's `range:` or not whole for an `Integer` parameter. A missing input is not a
/// defect here — the parameter is simply not derivable yet.
pub fn findings(elements: &[RawElement]) -> Vec<Finding> {
    let declared = declared(elements);
    if declared.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (fd, name, src) in &declared {
        if let Err(e) = expr::parse(src) {
            out.push(finding("E248", &fd.file_path, format!("derivedFrom of '{}.{name}' does not parse: {e}", fd.qualified_name)));
        }
    }
    let resolver = Resolver::new(elements);
    let feat_alias = crate::variability::feature_id_to_qname(elements);
    for cfg in elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration)) {
        let sel = crate::variability::canon_selection(&cfg.frontmatter.feature_selections(), &feat_alias);
        let cfg_id = cfg.frontmatter.id.clone().unwrap_or_else(|| cfg.qualified_name.clone());
        for (fd, name, src) in &declared {
            if !sel.get(fd.qualified_name.as_str()).copied().unwrap_or(false) || expr::parse(src).is_err() {
                continue;
            }
            let entry = fd.frontmatter.parameters.iter().flatten().find_map(|p| match p {
                serde_yaml::Value::Mapping(m) if m.get("name").and_then(|v| v.as_str()) == Some(name) => Some(m),
                _ => None,
            });
            match evaluate(elements, &resolver, Some(cfg), &fd.qualified_name, name, src) {
                Err(m) if m.starts_with("no value for") => {}
                Err(m) => out.push(finding("E248", &cfg.file_path, format!("configuration '{cfg_id}': derivedFrom of '{}.{name}' ({src}) cannot be evaluated: {m}", fd.qualified_name))),
                Ok(Value::Num(q)) => {
                    let Some(entry) = entry else { continue };
                    if let Some((lo, hi)) = entry.get("range").and_then(|v| v.as_str()).and_then(parse_range) {
                        if q.v < lo || q.v > hi {
                            out.push(finding("E249", &cfg.file_path, format!("configuration '{cfg_id}': derived '{}.{name}' = {} is outside range {lo}..{hi}", fd.qualified_name, expr::format_number(q.v))));
                        }
                    }
                    let ty = entry.get("type").or_else(|| entry.get("typedBy")).and_then(|v| v.as_str()).unwrap_or("");
                    if ty.ends_with("Integer") && q.v.fract() != 0.0 {
                        out.push(finding("E249", &cfg.file_path, format!("configuration '{cfg_id}': derived '{}.{name}' = {} is not a whole number but the parameter is an Integer", fd.qualified_name, expr::format_number(q.v))));
                    }
                }
                Ok(_) => {}
            }
        }
    }
    out
}

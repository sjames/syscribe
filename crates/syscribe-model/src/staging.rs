//! Staged configuration (`ADR-SYS-BT-001`, `REQ-TRS-BT-011..015`).
//!
//! A `Configuration` may declare `bindingStage:`, the stage at which it binds
//! variability. Staged Configurations form a monotonic chain through the existing
//! `derivedFrom:` inheritance (§9.8): a derived Configuration binds at the same or a
//! later stage, may not rebind what a staged ancestor closed, and may only *narrow*
//! what remains open (`parameterNarrowing:`). An unstaged Configuration is checked
//! exactly as before.
//!
//! This module owns the chain-level rules and the [`StageContext`] other checks share:
//!
//! * `E751` `bindingStage:` names a stage the vocabulary lacks;
//! * `E752` a staged Configuration selects a feature (here) or binds a parameter (in
//!   `validator::parameter_binding_findings`) whose binding window excludes its stage;
//! * `E753` a derived Configuration's stage is earlier than its base's;
//! * `E754` a staged Configuration rebinds, to a different value, something a staged
//!   ancestor already bound;
//! * `E755` a `parameterNarrowing:` entry is malformed, names no parameter, or widens
//!   what the parameter (or an ancestor) allows;
//! * `E756` (in `parameter_binding_findings`) a binding lies outside what an ancestor
//!   narrowed the parameter to;
//! * `E757` (in `parameter_binding_findings`) a required parameter whose window closed
//!   before the Configuration's stage is still unbound.

use std::collections::{BTreeMap, HashMap};

use serde_yaml::Value;

use crate::binding_time::{BindingTime, BindingWindow};
use crate::config_inherit;
use crate::element::{ElementType, RawElement};
use crate::validator::{Finding, Severity};
use crate::variability;

fn err(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Error }
}

fn is_configuration(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::Configuration))
}

/// The label a finding uses for a Configuration: its stable id, else its qualified name.
pub fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// A Configuration's `bindingStage:`. `Ok(None)` is unstaged; `Err(name)` is a stage the
/// vocabulary lacks (reported as `E751`, and the Configuration is then treated as unstaged).
pub fn stage_of(e: &RawElement) -> Result<Option<BindingTime>, String> {
    match e.frontmatter.binding_stage.as_deref() {
        None => Ok(None),
        Some(s) => BindingTime::parse(s).map(Some).ok_or_else(|| s.to_string()),
    }
}

/// What a `parameterNarrowing:` entry narrows a parameter to.
#[derive(Debug, Clone, PartialEq)]
pub enum Narrow {
    /// Inclusive numeric range, as a parameter's `range:`.
    Range(f64, f64),
    /// A subset of the parameter's `enumValues:`.
    Enum(Vec<String>),
}

impl Narrow {
    /// `true` when every value this allows is also allowed by `outer`.
    pub fn within(&self, outer: &Narrow) -> bool {
        match (self, outer) {
            (Narrow::Range(lo, hi), Narrow::Range(olo, ohi)) => lo >= olo && hi <= ohi,
            (Narrow::Enum(v), Narrow::Enum(o)) => v.iter().all(|x| o.contains(x)),
            _ => false,
        }
    }

    /// The values both allow (an empty result allows nothing).
    fn intersect(&self, other: &Narrow) -> Narrow {
        match (self, other) {
            (Narrow::Range(a, b), Narrow::Range(c, d)) => Narrow::Range(a.max(*c), b.min(*d)),
            (Narrow::Enum(a), Narrow::Enum(b)) => Narrow::Enum(a.iter().filter(|x| b.contains(x)).cloned().collect()),
            _ => self.clone(),
        }
    }

    /// Whether `val` (a binding's value) satisfies this narrowing. A value of the wrong
    /// kind is not this check's business (`E205`/`E206` judge it against the declaration).
    pub fn admits(&self, val: &Value) -> bool {
        match self {
            Narrow::Range(lo, hi) => match val.as_f64().or_else(|| val.as_i64().map(|i| i as f64)) {
                Some(n) => n >= *lo && n <= *hi,
                None => true,
            },
            Narrow::Enum(allowed) => match val.as_str() {
                Some(s) => allowed.iter().any(|a| a == s),
                None => true,
            },
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Narrow::Range(lo, hi) => format!("{lo}..{hi}"),
            Narrow::Enum(v) => format!("[{}]", v.join(", ")),
        }
    }

    /// A numeric range written `lo..hi` or `lo..=hi`, or a list of strings.
    pub fn parse(v: &Value) -> Result<Narrow, String> {
        match v {
            Value::String(s) => {
                let (lo, hi) = s.split_once("..").ok_or_else(|| format!("'{s}' is not a range (expected `lo..hi` or `lo..=hi`)"))?;
                let hi = hi.trim().strip_prefix('=').unwrap_or(hi.trim()).trim();
                match (lo.trim().parse::<f64>(), hi.parse::<f64>()) {
                    (Ok(lo), Ok(hi)) if lo <= hi => Ok(Narrow::Range(lo, hi)),
                    (Ok(_), Ok(_)) => Err(format!("range '{s}' is empty (lower bound above the upper)")),
                    _ => Err(format!("'{s}' is not a numeric range")),
                }
            }
            Value::Sequence(items) => items
                .iter()
                .map(|i| i.as_str().map(str::to_string))
                .collect::<Option<Vec<_>>>()
                .map(Narrow::Enum)
                .ok_or_else(|| "an enum narrowing must be a list of strings".to_string()),
            _ => Err("expected a range string (`lo..hi`) or a list of enum values".to_string()),
        }
    }
}

/// A parameter's declared constraints, as `parameterNarrowing:` must stay inside them.
#[derive(Debug, Clone, Default)]
pub struct ParamDecl {
    pub range: Option<(f64, f64)>,
    pub enum_values: Option<Vec<String>>,
}

/// Declared constraints of every `FeatureDef` parameter, keyed `<qname>.<param>`.
fn declared_params(elements: &[RawElement]) -> HashMap<String, ParamDecl> {
    let mut out = HashMap::new();
    for fd in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef))) {
        for p in fd.frontmatter.parameters.iter().flatten() {
            let Value::Mapping(m) = p else { continue };
            let get = |k: &str| m.get(Value::String(k.to_string()));
            let Some(name) = get("name").and_then(|v| v.as_str()) else { continue };
            let range = get("range").and_then(|v| v.as_str()).and_then(|s| match Narrow::parse(&Value::String(s.to_string())) {
                Ok(Narrow::Range(lo, hi)) => Some((lo, hi)),
                _ => None,
            });
            let enum_values = get("enumValues").and_then(|v| v.as_sequence()).map(|s| s.iter().filter_map(|x| x.as_str().map(str::to_string)).collect());
            out.insert(format!("{}.{}", fd.qualified_name, name), ParamDecl { range, enum_values });
        }
    }
    out
}

fn same_value(a: &Value, b: &Value) -> bool {
    let num = |v: &Value| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64));
    match (num(a), num(b)) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

/// The `derivedFrom:` structure of the model's Configurations, with the per-Configuration
/// facts the staged rules need. Built once per check.
pub struct StageContext<'a> {
    elements: &'a [RawElement],
    base_of: HashMap<usize, usize>,
    alias: HashMap<String, String>,
}

impl<'a> StageContext<'a> {
    pub fn new(elements: &'a [RawElement]) -> Self {
        let (base_of, _) = config_inherit::analyse(elements);
        Self { elements, base_of, alias: variability::feature_id_to_qname(elements) }
    }

    /// Ancestors of Configuration `i`, nearest first (a valid, acyclic chain).
    pub fn ancestors(&self, i: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut cur = i;
        while let Some(&b) = self.base_of.get(&cur) {
            if out.contains(&b) || b == i {
                break;
            }
            out.push(b);
            cur = b;
        }
        out
    }

    pub fn base(&self, i: usize) -> Option<usize> {
        self.base_of.get(&i).copied()
    }

    /// The stage of Configuration `i`, ignoring an unknown one (that is `E751`).
    pub fn stage(&self, i: usize) -> Option<BindingTime> {
        stage_of(&self.elements[i]).ok().flatten()
    }

    /// Canonical `<FeatureDef qname>.<param>` for a binding key (a `FEAT-*` id names the same feature).
    pub fn canon_key(&self, key: &str) -> String {
        match key.rsplit_once('.') {
            Some((f, p)) => format!("{}.{}", variability::canon_feature_ref(f, &self.alias), p),
            None => key.to_string(),
        }
    }

    /// The **effective** (inherited plus own) feature selection of Configuration `i`, canonical keys.
    pub fn effective_selection(&self, i: usize) -> BTreeMap<String, bool> {
        variability::canon_selection(&self.elements[i].frontmatter.feature_selections(), &self.alias)
    }

    /// The `parameterBindings:` Configuration `i` declares itself (no inheritance), canonical keys.
    pub fn own_bindings(&self, i: usize) -> BTreeMap<String, Value> {
        match &self.elements[i].frontmatter.parameter_bindings {
            Some(Value::Mapping(m)) => m
                .iter()
                .filter_map(|(k, v)| k.as_str().map(|k| (self.canon_key(k), v.clone())))
                .collect(),
            _ => BTreeMap::new(),
        }
    }

    /// The `features:` selection Configuration `i` declares itself (no inheritance), canonical keys.
    pub fn own_selection(&self, i: usize) -> BTreeMap<String, bool> {
        variability::canon_selection(&self.elements[i].frontmatter.declared_feature_selections(), &self.alias)
    }

    /// Configuration `i`'s own `parameterNarrowing:` entries, canonical keys; each entry is
    /// the parsed narrowing or the reason it is malformed.
    pub fn own_narrowings(&self, i: usize) -> BTreeMap<String, Result<Narrow, String>> {
        match &self.elements[i].frontmatter.parameter_narrowing {
            Some(Value::Mapping(m)) => m
                .iter()
                .filter_map(|(k, v)| k.as_str().map(|k| (self.canon_key(k), Narrow::parse(v))))
                .collect(),
            _ => BTreeMap::new(),
        }
    }

    /// What `key` is narrowed to for Configuration `i`: the intersection of every
    /// well-formed narrowing along `i` and its ancestors. `None` when nothing narrows it.
    pub fn effective_narrowing(&self, i: usize, key: &str) -> Option<Narrow> {
        self.narrowing_over(std::iter::once(i).chain(self.ancestors(i)), key)
    }

    /// As [`Self::effective_narrowing`] but over the ancestors only — what `i` must stay inside.
    fn inherited_narrowing(&self, i: usize, key: &str) -> Option<(Narrow, String)> {
        let anc = self.ancestors(i);
        let n = self.narrowing_over(anc.iter().copied(), key)?;
        let from = anc
            .iter()
            .find(|&&a| matches!(self.own_narrowings(a).get(key), Some(Ok(_))))
            .map(|&a| label(&self.elements[a]))
            .unwrap_or_default();
        Some((n, from))
    }

    fn narrowing_over(&self, over: impl Iterator<Item = usize>, key: &str) -> Option<Narrow> {
        over.filter_map(|c| self.own_narrowings(c).remove(key).and_then(Result::ok))
            .reduce(|acc, n| acc.intersect(&n))
    }
}

/// The chain-level staged-configuration rules (`E751`, `E752` for features, `E753`,
/// `E754`, `E755`). Run by `validate`. Unstaged Configurations take part only in `E755`.
pub fn staged_configuration_findings(elements: &[RawElement]) -> Vec<Finding> {
    let mut f = Vec::new();
    let confs: Vec<usize> = elements.iter().enumerate().filter(|(_, e)| is_configuration(e)).map(|(i, _)| i).collect();
    if confs.is_empty() {
        return f;
    }
    let ctx = StageContext::new(elements);
    let params = declared_params(elements);
    let feature_win: HashMap<&str, BindingWindow> = elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef)))
        .filter_map(|fd| {
            BindingWindow::read(fd.frontmatter.binding_time.as_deref(), fd.frontmatter.latest_binding_time.as_deref())
                .0
                .map(|w| (fd.qualified_name.as_str(), w))
        })
        .collect();

    for &i in &confs {
        let cfg = &elements[i];
        let (file, me) = (cfg.file_path.as_str(), label(cfg));
        let stage = match stage_of(cfg) {
            Ok(s) => s,
            Err(name) => {
                f.push(err("E751", file, format!(
                    "Configuration '{me}' has bindingStage '{name}' which is not one of {}",
                    crate::binding_time::active().names_hint())));
                None
            }
        };

        if let Some(st) = stage {
            // E752 (features): every own selection must be bindable at this stage.
            for feat in ctx.own_selection(i).keys() {
                if let Some(w) = feature_win.get(feat.as_str()) {
                    if !w.contains(st) {
                        f.push(err("E752", file, format!(
                            "Configuration '{me}' (bindingStage {}) selects feature '{feat}', whose binding window is {} — it cannot be bound at this stage",
                            st.name(), w.describe())));
                    }
                }
            }

            // E753: a derived Configuration binds at the same or a later stage than its base.
            if let Some(b) = ctx.base(i) {
                if let Some(bs) = ctx.stage(b) {
                    if st < bs {
                        f.push(err("E753", file, format!(
                            "Configuration '{me}' (bindingStage {}) is derivedFrom '{}' (bindingStage {}) — a derived Configuration must bind at the same or a later stage",
                            st.name(), label(&elements[b]), bs.name())));
                    }
                }
            }

            // E754: do not rebind, to a different value, what a staged ancestor already closed.
            // The nearest ancestor that declares the key decides; if it is unstaged it closes nothing.
            let ancestors = ctx.ancestors(i);
            let nearest = |declared: &dyn Fn(usize) -> Option<Value>| -> Option<(usize, BindingTime, Value)> {
                let (a, v) = ancestors.iter().find_map(|&a| declared(a).map(|v| (a, v)))?;
                ctx.stage(a).map(|at| (a, at, v))
            };
            for (key, mine) in ctx.own_bindings(i) {
                if let Some((a, at, theirs)) = nearest(&|a| ctx.own_bindings(a).remove(&key)) {
                    if !same_value(&theirs, &mine) {
                        f.push(err("E754", file, format!(
                            "Configuration '{me}' rebinds '{key}' — already bound by '{}' (bindingStage {}); a value bound at an earlier stage is closed",
                            label(&elements[a]), at.name())));
                    }
                }
            }
            for (feat, mine) in ctx.own_selection(i) {
                if let Some((a, at, theirs)) = nearest(&|a| ctx.own_selection(a).get(&feat).map(|b| Value::Bool(*b))) {
                    if theirs != Value::Bool(mine) {
                        f.push(err("E754", file, format!(
                            "Configuration '{me}' sets feature '{feat}' to {mine} — already decided otherwise by '{}' (bindingStage {}); a selection made at an earlier stage is closed",
                            label(&elements[a]), at.name())));
                    }
                }
            }
        }

        // E755: a narrowing must be well-formed, name a parameter, and stay inside what
        // the parameter declares and what every ancestor narrowed it to.
        for (key, parsed) in ctx.own_narrowings(i) {
            let n = match parsed {
                Ok(n) => n,
                Err(why) => {
                    f.push(err("E755", file, format!("Configuration '{me}' parameterNarrowing '{key}': {why}")));
                    continue;
                }
            };
            let Some(decl) = params.get(&ctx.canon_key(&key)) else {
                f.push(err("E755", file, format!(
                    "Configuration '{me}' parameterNarrowing '{key}' does not name a declared FeatureDef parameter ({})",
                    variability::FORMS_PARAM_REF)));
                continue;
            };
            let outer = match (&n, decl) {
                (Narrow::Range(..), ParamDecl { enum_values: Some(_), .. }) => {
                    f.push(err("E755", file, format!("Configuration '{me}' parameterNarrowing '{key}' is a range but the parameter is an enumValues parameter — narrow it with a list")));
                    continue;
                }
                (Narrow::Enum(_), ParamDecl { enum_values: None, .. }) => {
                    f.push(err("E755", file, format!("Configuration '{me}' parameterNarrowing '{key}' is a list but the parameter declares no enumValues")));
                    continue;
                }
                (Narrow::Range(..), ParamDecl { range, .. }) => range.map(|(lo, hi)| Narrow::Range(lo, hi)),
                (Narrow::Enum(_), ParamDecl { enum_values, .. }) => enum_values.clone().map(Narrow::Enum),
            };
            if let Some(o) = outer {
                if !n.within(&o) {
                    f.push(err("E755", file, format!(
                        "Configuration '{me}' parameterNarrowing '{key}' = {} widens the parameter's declared {} — a narrowing may only shrink the allowed values",
                        n.describe(), o.describe())));
                    continue;
                }
            }
            if let Some((o, from)) = ctx.inherited_narrowing(i, &key) {
                if !n.within(&o) {
                    f.push(err("E755", file, format!(
                        "Configuration '{me}' parameterNarrowing '{key}' = {} widens what '{from}' narrowed it to ({}) — a narrowing may only shrink the allowed values",
                        n.describe(), o.describe())));
                }
            }
        }
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn narrowing_parses_ranges_and_lists_and_rejects_the_rest() {
        let s = |t: &str| Value::String(t.to_string());
        assert_eq!(Narrow::parse(&s("900..1000")), Ok(Narrow::Range(900.0, 1000.0)));
        assert_eq!(Narrow::parse(&s("900..=1000")), Ok(Narrow::Range(900.0, 1000.0)));
        assert!(Narrow::parse(&s("1000..900")).unwrap_err().contains("empty"));
        assert!(Narrow::parse(&s("a..b")).is_err() && Narrow::parse(&s("nonsense")).is_err());
        assert_eq!(Narrow::parse(&Value::Sequence(vec![s("a"), s("b")])), Ok(Narrow::Enum(vec!["a".into(), "b".into()])));
        assert!(Narrow::parse(&Value::Sequence(vec![Value::Bool(true)])).is_err());
        assert!(Narrow::parse(&Value::Bool(true)).is_err());
    }

    #[test]
    fn within_intersect_and_admits() {
        let r = |a, b| Narrow::Range(a, b);
        assert!(r(2.0, 3.0).within(&r(1.0, 4.0)) && !r(0.0, 3.0).within(&r(1.0, 4.0)) && r(1.0, 4.0).within(&r(1.0, 4.0)));
        assert_eq!(r(1.0, 5.0).intersect(&r(3.0, 9.0)), r(3.0, 5.0));
        let e = |v: &[&str]| Narrow::Enum(v.iter().map(|s| s.to_string()).collect());
        assert!(e(&["a"]).within(&e(&["a", "b"])) && !e(&["a", "c"]).within(&e(&["a", "b"])));
        assert_eq!(e(&["a", "b"]).intersect(&e(&["b", "c"])), e(&["b"]));
        assert!(!r(1.0, 2.0).within(&e(&["a"])), "mixed kinds never nest");
        assert!(r(1.0, 2.0).admits(&Value::from(1.5)) && !r(1.0, 2.0).admits(&Value::from(3)));
        assert!(e(&["a"]).admits(&Value::String("a".into())) && !e(&["a"]).admits(&Value::String("z".into())));
        assert!(r(1.0, 2.0).admits(&Value::String("x".into())), "wrong kind is E205/E206's business");
    }

    #[test]
    fn same_value_compares_numbers_by_value() {
        assert!(same_value(&Value::from(1), &Value::from(1.0)));
        assert!(!same_value(&Value::from(1), &Value::from(2)));
        assert!(same_value(&Value::String("a".into()), &Value::String("a".into())));
    }
}

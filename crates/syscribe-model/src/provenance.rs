//! Provenance of a `Configuration`'s effective values (`ADR-SYS-BT-001`,
//! `REQ-TRS-BT-021..022`): where each effective feature selection and parameter value
//! came from, and the stage at which it was bound.
//!
//! The *source* of a value is one of: the Configuration's own binding; a `derivedFrom:`
//! ancestor's (naming it); a `subConfigurations:` tier (a binding that reaches a
//! parameter of a consolidated lower tier); `bindTo:` propagation from a system
//! parameter; a `derivedFrom:` expression on the parameter; its fixed `value:`; or its
//! `default:`. The *stage* is the `bindingStage:` of the Configuration that bound it,
//! when that Configuration is staged; a model-level source (fixed, derived, default)
//! has none.

use std::collections::{BTreeMap, HashMap};

use serde_yaml::Value;

use crate::element::{ElementType, RawElement};
use crate::staging::{label, StageContext};

/// Where an effective value came from.
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    /// Bound by the Configuration itself.
    Own,
    /// Bound by a `derivedFrom:` ancestor.
    Inherited { from: String },
    /// Reaches a parameter of a `subConfigurations:` tier.
    SubConfiguration { tiers: Vec<String> },
    /// Propagated from the system parameter named by `bindTo:`.
    BindTo { from: String },
    /// Computed by the parameter's own `derivedFrom:` expression.
    Derived { expr: String },
    /// The parameter's fixed `value:`.
    Fixed,
    /// The parameter's `default:`.
    Default,
    /// An abstract feature whose selection follows from the concrete ones.
    AbstractFollows,
}

impl Source {
    pub fn kind(&self) -> &'static str {
        match self {
            Source::Own => "own",
            Source::Inherited { .. } => "inherited",
            Source::SubConfiguration { .. } => "subConfiguration",
            Source::BindTo { .. } => "bindTo",
            Source::Derived { .. } => "derived",
            Source::Fixed => "fixed",
            Source::Default => "default",
            Source::AbstractFollows => "abstract",
        }
    }

    /// A short human rendering: `own`, `inherited from CONF-X`, `default`, …
    pub fn describe(&self) -> String {
        match self {
            Source::Own => "own".into(),
            Source::Inherited { from } => format!("inherited from {from}"),
            Source::SubConfiguration { tiers } => format!("subConfiguration {}", tiers.join(", ")),
            Source::BindTo { from } => format!("bindTo {from}"),
            Source::Derived { expr } => format!("derived: {expr}"),
            Source::Fixed => "fixed".into(),
            Source::Default => "default".into(),
            Source::AbstractFollows => "follows the concrete features".into(),
        }
    }

    fn json(&self) -> serde_json::Value {
        let mut m = serde_json::Map::new();
        m.insert("kind".into(), self.kind().into());
        match self {
            Source::Inherited { from } | Source::BindTo { from } => {
                m.insert("from".into(), from.clone().into());
            }
            Source::SubConfiguration { tiers } => {
                m.insert("tiers".into(), tiers.clone().into());
            }
            Source::Derived { expr } => {
                m.insert("expr".into(), expr.clone().into());
            }
            _ => {}
        }
        serde_json::Value::Object(m)
    }
}

/// One effective feature selection and where it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionProvenance {
    pub feature: String,
    pub selected: bool,
    pub source: Source,
    pub stage: Option<String>,
}

/// One effective parameter value and where it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterProvenance {
    /// `<FeatureDef qname>.<param>`.
    pub key: String,
    pub value: Value,
    pub source: Source,
    pub stage: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigProvenance {
    /// The Configuration's own `bindingStage:`, when valid.
    pub stage: Option<String>,
    pub selections: Vec<SelectionProvenance>,
    pub parameters: Vec<ParameterProvenance>,
}

/// A local parameter's model-level value sources.
struct ParamDef {
    fixed: Option<Value>,
    derived: Option<String>,
    bind_to: Option<String>,
    default: Option<Value>,
}

fn local_params(elements: &[RawElement]) -> BTreeMap<String, ParamDef> {
    let mut out = BTreeMap::new();
    for fd in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef))) {
        for p in fd.frontmatter.parameters.iter().flatten() {
            let Value::Mapping(m) = p else { continue };
            let get = |k: &str| m.get(Value::String(k.to_string())).cloned();
            let Some(name) = get("name").and_then(|v| v.as_str().map(str::to_string)) else { continue };
            out.insert(
                format!("{}.{}", fd.qualified_name, name),
                ParamDef {
                    fixed: get("value"),
                    derived: get("derivedFrom").and_then(|v| v.as_str().map(str::to_string)),
                    bind_to: get("bindTo").and_then(|v| v.as_str().map(str::to_string)),
                    default: get("default"),
                },
            );
        }
    }
    out
}

/// Provenance of Configuration `cfg_idx` (an index into `elements`).
pub fn provenance(elements: &[RawElement], cfg_idx: usize) -> ConfigProvenance {
    let ctx = StageContext::new(elements);
    let cfg = &elements[cfg_idx];
    let stage_name = |i: usize| ctx.stage(i).map(|s| s.name());
    let chain: Vec<usize> = std::iter::once(cfg_idx).chain(ctx.ancestors(cfg_idx)).collect();
    // The nearest of self/ancestors declaring `declared(i)`, as (source, stage).
    let origin = |declares: &dyn Fn(usize) -> bool| -> Option<(Source, Option<String>)> {
        let &i = chain.iter().find(|&&i| declares(i))?;
        Some(if i == cfg_idx {
            (Source::Own, stage_name(i))
        } else {
            (Source::Inherited { from: label(&elements[i]) }, stage_name(i))
        })
    };

    let mut out = ConfigProvenance { stage: stage_name(cfg_idx), ..Default::default() };

    // Selections.
    let derived = cfg.frontmatter.inherited.as_ref().map(|h| h.derived.clone()).unwrap_or_default();
    for (feature, selected) in ctx.effective_selection(cfg_idx) {
        let (source, stage) = if derived.contains(&feature) {
            (Source::AbstractFollows, None)
        } else {
            origin(&|i| ctx.own_selection(i).contains_key(&feature)).unwrap_or((Source::Own, None))
        };
        out.selections.push(SelectionProvenance { feature, selected, source, stage });
    }

    // Parameters bound by a Configuration in the chain (local or reaching a lower tier).
    let params = local_params(elements);
    let tiers: Vec<String> = cfg.frontmatter.sub_configurations.clone().unwrap_or_default();
    let mut bound: HashMap<String, Value> = HashMap::new();
    for &i in chain.iter().rev() {
        bound.extend(ctx.own_bindings(i));
    }
    for (key, value) in &bound {
        let (source, stage) = if !params.contains_key(key) && !tiers.is_empty() {
            (Source::SubConfiguration { tiers: tiers.clone() }, None)
        } else {
            origin(&|i| ctx.own_bindings(i).contains_key(key)).unwrap_or((Source::Own, None))
        };
        out.parameters.push(ParameterProvenance { key: key.clone(), value: value.clone(), source, stage });
    }

    // Parameters of selected features that no Configuration bound: model-level values.
    let selected: HashMap<String, bool> = ctx.effective_selection(cfg_idx).into_iter().collect();
    for (key, def) in &params {
        if bound.contains_key(key) {
            continue;
        }
        let feat = key.rsplit_once('.').map_or(key.as_str(), |(f, _)| f);
        if selected.get(feat) != Some(&true) {
            continue;
        }
        let (value, source) = if let Some(v) = &def.fixed {
            (v.clone(), Source::Fixed)
        } else if let Some(e) = &def.derived {
            (Value::String(e.clone()), Source::Derived { expr: e.clone() })
        } else if let Some(v) = def.bind_to.as_ref().and_then(|t| bound.get(t)) {
            (v.clone(), Source::BindTo { from: def.bind_to.clone().unwrap_or_default() })
        } else if let Some(v) = &def.default {
            (v.clone(), Source::Default)
        } else {
            continue;
        };
        out.parameters.push(ParameterProvenance { key: key.clone(), value, source, stage: None });
    }
    out.parameters.sort_by(|a, b| a.key.cmp(&b.key));
    out
}

/// The JSON form shared by `configure --json`, and (later) `residual` and `params export`.
pub fn to_json(p: &ConfigProvenance) -> serde_json::Value {
    let src = |s: &Source| s.json();
    serde_json::json!({
        "bindingStage": p.stage,
        "selections": p.selections.iter().map(|s| serde_json::json!({
            "feature": s.feature, "selected": s.selected, "source": src(&s.source), "stage": s.stage,
        })).collect::<Vec<_>>(),
        "parameters": p.parameters.iter().map(|q| serde_json::json!({
            "key": q.key, "value": serde_json::to_value(&q.value).unwrap_or_default(),
            "source": src(&q.source), "stage": q.stage,
        })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_kinds_and_descriptions() {
        assert_eq!(Source::Own.kind(), "own");
        assert_eq!(Source::Inherited { from: "CONF-A".into() }.describe(), "inherited from CONF-A");
        assert_eq!(Source::Default.describe(), "default");
        assert_eq!(Source::BindTo { from: "S::P.x".into() }.kind(), "bindTo");
        assert_eq!(Source::Derived { expr: "a > 1".into() }.describe(), "derived: a > 1");
    }
}

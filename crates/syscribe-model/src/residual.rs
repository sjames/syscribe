//! Residual variability (`ADR-SYS-BT-001`, `REQ-TRS-BT-016`): what a `Configuration`
//! leaves open, and who binds it.
//!
//! A feature or parameter that declares a binding window and that the Configuration's
//! effective (inherited plus own) selection and bindings do not decide is *open*. Open
//! items are grouped by the stage at which they can next be bound — the later of the
//! window's start and the Configuration's own stage — and by that stage's `binder`. An
//! item whose window closed before the Configuration's stage can no longer be bound by
//! any later stage; it is listed apart as *closed*.
//!
//! Only a feature that is a real choice takes part (not an abstract or mandatory one,
//! which no Configuration decides), and a parameter only when its feature is selected and
//! the parameter is not fixed or derived. An item with no window is outside binding-time
//! analysis and is not listed.

use std::collections::{BTreeMap, HashSet};

use serde_yaml::Value;

use crate::binding_time::{active, BindingTime, BindingWindow};
use crate::element::{ElementType, RawElement};
use crate::feature_model::{constraint_states, ConstraintState};
use crate::staging::{label, StageContext};

/// One open feature selection or parameter value.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// `"feature"` or `"parameter"`.
    pub kind: &'static str,
    /// `<FeatureDef qname>` or `<FeatureDef qname>.<param>`.
    pub key: String,
    /// The owning `FeatureDef`'s stable `FEAT-*` id.
    pub id: Option<String>,
    pub name: String,
    pub window: BindingWindow,
    pub required: bool,
    pub has_default: bool,
}

/// Open items that can next be bound at the same stage.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// The stage the items can next be bound at; `None` for the closed group.
    pub stage: Option<BindingTime>,
    pub binder: Option<String>,
    pub closed: bool,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub struct Residual {
    pub configuration: String,
    pub stage: Option<BindingTime>,
    pub groups: Vec<Group>,
    pub constraints: Vec<ConstraintState>,
    /// Elements a projection at this stage keeps although their `appliesWhen:` is undecided,
    /// and those that rely on one being absent (REQ-TRS-BT-033).
    pub retention: crate::retention::RetentionReport,
}

/// The residual variability of `elements[cfg_idx]`.
pub fn residual(elements: &[RawElement], cfg_idx: usize) -> Residual {
    let ctx = StageContext::new(elements);
    let cfg = &elements[cfg_idx];
    let stage = ctx.stage(cfg_idx);
    let sel = ctx.effective_selection(cfg_idx);
    let bound: HashSet<String> = match cfg.frontmatter.effective_parameter_bindings() {
        Some(Value::Mapping(m)) => m.keys().filter_map(|k| k.as_str()).map(|k| ctx.canon_key(k)).collect(),
        _ => HashSet::new(),
    };

    let mut items: Vec<Item> = Vec::new();
    for fd in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef))) {
        let fm = &fd.frontmatter;
        let q = fd.qualified_name.as_str();
        let id = fm.id.clone();
        let is_choice = fm.is_abstract != Some(true) && fm.mandatory != Some(true) && fm.group_kind.as_deref() != Some("mandatory");
        if let (true, false, (Some(window), _)) = (
            is_choice,
            sel.contains_key(q),
            BindingWindow::read(fm.binding_time.as_deref(), fm.latest_binding_time.as_deref()),
        ) {
            items.push(Item {
                kind: "feature",
                key: q.to_string(),
                id: id.clone(),
                name: fd.qualified_name.rsplit("::").next().unwrap_or(q).to_string(),
                window,
                required: false,
                has_default: false,
            });
        }
        if sel.get(q) != Some(&true) {
            continue;
        }
        for p in fm.parameters.iter().flatten() {
            let Value::Mapping(m) = p else { continue };
            let get = |k: &str| m.get(Value::String(k.to_string()));
            let Some(name) = get("name").and_then(|v| v.as_str()) else { continue };
            let key = format!("{q}.{name}");
            let fixed = get("isFixed").and_then(|v| v.as_bool()).unwrap_or(false) || get("value").is_some() || get("derivedFrom").is_some();
            let (window, _) = BindingWindow::read(get("bindingTime").and_then(|v| v.as_str()), get("latestBindingTime").and_then(|v| v.as_str()));
            let Some(window) = window else { continue };
            if fixed || bound.contains(&key) {
                continue;
            }
            items.push(Item {
                kind: "parameter",
                key,
                id: id.clone(),
                name: name.to_string(),
                window,
                required: get("isRequired").and_then(|v| v.as_bool()).unwrap_or(false),
                has_default: get("default").is_some(),
            });
        }
    }

    // Group by the stage each can next be bound at; closed windows last.
    let vocab = active();
    let mut open: BTreeMap<usize, Vec<Item>> = BTreeMap::new();
    let mut closed: Vec<Item> = Vec::new();
    for it in items {
        match stage {
            Some(st) if it.window.latest < st => closed.push(it),
            _ => {
                let next = stage.map_or(it.window.earliest, |st| it.window.earliest.max(st));
                open.entry(next.rank()).or_default().push(it);
            }
        }
    }
    let mut groups: Vec<Group> = open
        .into_iter()
        .map(|(rank, mut items)| {
            items.sort_by(|a, b| a.key.cmp(&b.key));
            let st = vocab.stages().get(rank);
            Group {
                stage: BindingTime::parse(&st.map(|s| s.name.clone()).unwrap_or_default()),
                binder: st.and_then(|s| s.binder.clone()),
                closed: false,
                items,
            }
        })
        .collect();
    if !closed.is_empty() {
        closed.sort_by(|a, b| a.key.cmp(&b.key));
        groups.push(Group { stage: None, binder: None, closed: true, items: closed });
    }
    Residual { configuration: label(cfg), stage, groups, constraints: constraint_states(elements, cfg_idx), retention: crate::retention::report(elements, cfg_idx) }
}

impl Residual {
    /// Keys of every open (not closed) item, for cross-checks against other reports.
    pub fn open_keys(&self) -> HashSet<String> {
        self.groups.iter().filter(|g| !g.closed).flat_map(|g| g.items.iter().map(|i| i.key.clone())).collect()
    }

    pub fn to_json(&self) -> serde_json::Value {
        let win = |w: &BindingWindow| serde_json::json!({"earliest": w.earliest.name(), "latest": w.latest.name()});
        serde_json::json!({
            "configuration": self.configuration,
            "bindingStage": self.stage.map(|s| s.name()),
            "groups": self.groups.iter().map(|g| serde_json::json!({
                "stage": g.stage.map(|s| s.name()),
                "binder": g.binder,
                "closed": g.closed,
                "items": g.items.iter().map(|i| serde_json::json!({
                    "kind": i.kind, "key": i.key, "id": i.id, "name": i.name,
                    "window": win(&i.window), "required": i.required, "hasDefault": i.has_default,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "retained": self.retention.retained.iter().map(|r| serde_json::json!({
                "qname": r.qname, "id": r.id, "type": r.kind, "because": r.because,
            })).collect::<Vec<_>>(),
            "reliances": self.retention.reliances.iter().map(|r| serde_json::json!({
                "dependent": r.dependent, "retained": r.retained,
            })).collect::<Vec<_>>(),
            "constraints": self.constraints.iter().map(|c| serde_json::json!({
                "id": c.id, "expression": c.expression, "state": c.state.name(),
                "severity": if c.warning { "warning" } else { "error" }, "open": c.open,
            })).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_shape_is_stable_for_an_empty_residual() {
        let r = Residual { configuration: "CONF-X".into(), stage: None, groups: vec![], constraints: vec![], retention: Default::default() };
        let j = r.to_json();
        assert_eq!(j["configuration"], "CONF-X");
        assert!(j["bindingStage"].is_null() && j["groups"].as_array().unwrap().is_empty());
        assert!(r.open_keys().is_empty());
    }
}

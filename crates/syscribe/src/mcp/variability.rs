//! JSON shaping for the feature-model / projection MCP tools. Each function
//! reuses the `syscribe-model` library (feature_model / projection / variability)
//! which already returns structured values — no CLI/text round-trip.

use serde_json::{json, Value};

use syscribe_model::config::ValidateConfig;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::feature_model::{
    check_feature_model, check_feature_model_deep, configure, has_feature_model, ConfigureOutcome,
};
use syscribe_model::projection::{project, resolve_selection, validate_projected, SelectionOutcome};
use syscribe_model::validator::{Finding, Severity};
use syscribe_model::variability;

use crate::query::type_label;

fn severity_str(s: &Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

/// Render findings as JSON, normalising file paths relative to the model root.
fn findings_json(findings: &[Finding], root: &std::path::Path) -> Vec<Value> {
    let root_s = root.to_string_lossy();
    findings
        .iter()
        .map(|f| {
            let file = f
                .file
                .strip_prefix(root_s.as_ref())
                .map(|s| s.trim_start_matches(['/', '\\']).to_string())
                .unwrap_or_else(|| f.file.clone());
            json!({ "code": f.code, "severity": severity_str(&f.severity), "file": file, "message": f.message })
        })
        .collect()
}

/// A FeatureDef's parent feature: its explicit `parentFeature`, else the nearest
/// ancestor qualified name that is itself a FeatureDef.
fn parent_feature(elements: &[RawElement], e: &RawElement) -> Option<String> {
    if let Some(p) = &e.frontmatter.parent_feature {
        return Some(p.clone());
    }
    let mut q = e.qualified_name.as_str();
    while let Some(idx) = q.rfind("::") {
        let anc = &q[..idx];
        if elements.iter().any(|x| {
            x.qualified_name == anc
                && x.frontmatter.element_type.as_ref() == Some(&ElementType::FeatureDef)
        }) {
            return Some(anc.to_string());
        }
        q = anc;
    }
    None
}

fn feature_card(elements: &[RawElement], e: &RawElement) -> Value {
    let fm = &e.frontmatter;
    json!({
        "qname": e.qualified_name,
        "id": fm.id,
        "name": fm.name,
        "groupKind": fm.group_kind,
        "mandatory": fm.mandatory,
        "parent": parent_feature(elements, e),
        "requires": fm.requires.as_ref().and_then(|v| serde_json::to_value(v).ok()),
        "excludes": fm.excludes,
        "parameters": fm.parameters.as_ref().and_then(|v| serde_json::to_value(v).ok()),
    })
}

/// `features {feature?}` — enumerate the feature model (or one feature's card).
pub fn features(elements: &[RawElement], feature: Option<&str>) -> Value {
    let has = has_feature_model(elements);
    let alias = variability::feature_id_to_qname(elements);
    let want_qname = feature.map(|f| variability::canon_feature_ref(f, &alias));
    let cards: Vec<Value> = elements
        .iter()
        .filter(|e| e.frontmatter.element_type.as_ref() == Some(&ElementType::FeatureDef))
        .filter(|e| {
            want_qname
                .as_ref()
                .is_none_or(|q| &e.qualified_name == q || e.frontmatter.id.as_deref() == feature)
        })
        .map(|e| feature_card(elements, e))
        .collect();
    json!({ "hasFeatureModel": has, "features": cards })
}

/// `feature_check {deep?}` — feature-model validation, optionally SAT-backed.
pub fn feature_check(elements: &[RawElement], deep: bool, root: &std::path::Path) -> Value {
    let findings = findings_json(&check_feature_model(elements), root);
    let mut out = json!({ "findings": findings });
    if deep {
        let d = check_feature_model_deep(elements);
        out.as_object_mut().unwrap().insert(
            "deep".into(),
            json!({
                "void": d.void,
                "dead": d.dead,
                "core": d.core,
                "falseOptional": d.false_optional,
                "invalidConfigs": d.invalid_configs,
                "diagnoses": d.diagnoses,
                "skipped": d.skipped,
            }),
        );
    }
    out
}

/// `configure {config}` — forced/free features for a configuration.
pub fn configure_tool(elements: &[RawElement], config: &str) -> Value {
    match configure(elements, config) {
        ConfigureOutcome::Dormant => json!({ "dormant": true }),
        ConfigureOutcome::NotFound => json!({ "notFound": true }),
        ConfigureOutcome::Report {
            satisfiable,
            forced_true,
            forced_false,
            free,
            explanation,
        } => json!({
            "satisfiable": satisfiable,
            "forcedTrue": forced_true,
            "forcedFalse": forced_false,
            "free": free,
            "explanation": explanation,
        }),
    }
}

fn elem_node(e: &RawElement) -> Value {
    json!({
        "qname": e.qualified_name,
        "id": e.frontmatter.id,
        "type": e.frontmatter.element_type.as_ref().map(type_label),
    })
}

/// `project {config}` — the active element set + projected validation findings.
pub fn project_tool(elements: &[RawElement], config: &ValidateConfig, arg: &str, root: &std::path::Path) -> Value {
    match resolve_selection(elements, arg) {
        SelectionOutcome::Dormant => json!({ "dormant": true }),
        SelectionOutcome::Error(e) => json!({ "error": e }),
        SelectionOutcome::Resolved(sel) => {
            let active = project(elements, &sel);
            let findings = findings_json(&validate_projected(elements, config, &sel), root);
            json!({
                "selection": sel,
                "active": active.iter().map(elem_node).collect::<Vec<_>>(),
                "activeCount": active.len(),
                "findings": findings,
            })
        }
    }
}

/// `diff_configs {a, b}` — set difference of two projections' active sets.
pub fn diff_configs(elements: &[RawElement], a: &str, b: &str) -> Value {
    let resolve = |arg: &str| match resolve_selection(elements, arg) {
        SelectionOutcome::Resolved(sel) => Ok(project(elements, &sel)),
        SelectionOutcome::Dormant => Ok(elements.to_vec()),
        SelectionOutcome::Error(e) => Err(e),
    };
    let (ea, eb) = match (resolve(a), resolve(b)) {
        (Ok(ea), Ok(eb)) => (ea, eb),
        (Err(e), _) | (_, Err(e)) => return json!({ "error": e }),
    };
    let set_a: std::collections::HashSet<&str> = ea.iter().map(|e| e.qualified_name.as_str()).collect();
    let set_b: std::collections::HashSet<&str> = eb.iter().map(|e| e.qualified_name.as_str()).collect();
    let only_in_a: Vec<Value> = ea
        .iter()
        .filter(|e| !set_b.contains(e.qualified_name.as_str()))
        .map(|e| json!({ "qname": e.qualified_name, "id": e.frontmatter.id }))
        .collect();
    let only_in_b: Vec<Value> = eb
        .iter()
        .filter(|e| !set_a.contains(e.qualified_name.as_str()))
        .map(|e| json!({ "qname": e.qualified_name, "id": e.frontmatter.id }))
        .collect();
    let common = set_a.intersection(&set_b).count();
    json!({ "onlyInA": only_in_a, "onlyInB": only_in_b, "commonCount": common })
}

/// Stringify an `appliesWhen` YAML value (a bare string stays as-is).
fn aw_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
    }
}

/// `why_active {ref, config}` — explain an element's activation under a selection.
/// Returns `Err` (→ isError) only when the reference does not resolve.
pub fn why_active(elements: &[RawElement], elem: &RawElement, config: &str) -> Value {
    let pkg = variability::package_conditions(elements);
    let alias = variability::feature_id_to_qname(elements);
    let sel = match resolve_selection(elements, config) {
        SelectionOutcome::Resolved(s) => variability::canon_selection(&s, &alias),
        _ => std::collections::BTreeMap::new(),
    };

    let active = syscribe_model::projection::is_active_canon(elem, &sel, &pkg, &alias);

    match variability::effective_applies_when(elem, &pkg) {
        None => json!({
            "active": active,
            "effectiveAppliesWhen": Value::Null,
            "source": "none",
            "referencedFeatures": Vec::<String>::new(),
            "verdict": "always-active",
        }),
        Some((aw, owner)) => {
            let source = match owner {
                Some(p) => format!("inherited:{p}"),
                None => "own".to_string(),
            };
            let referenced: Vec<String> = variability::effective_expr_canon(elem, &pkg, &alias)
                .map(|e| e.operands())
                .unwrap_or_default();
            let verdict = if active { "active" } else { "inactive" };
            json!({
                "active": active,
                "effectiveAppliesWhen": aw_string(&aw),
                "source": source,
                "referencedFeatures": referenced,
                "verdict": verdict,
            })
        }
    }
}
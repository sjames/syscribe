//! Feature-model discovery commands (REQ-TRS-DISC-001..005):
//!   * `features`     — feature-model overview (tree of FeatureDefs).
//!   * `feature <q>`  — single-feature card (gates, selecting configs, params).
//!   * `why-active`   — explain an element's activation under a configuration.
//!
//! All commands are dormant-aware: with no feature model they print a notice and
//! exit 0 (the discovery overview), while `feature`/`why-active` still error on a
//! genuinely unknown/unresolvable argument.

use serde_json::json;
use syscribe_model::{
    element::{ElementType, RawElement},
    projection::{self, SelectionOutcome},
    variability::{self, FeatureExpr},
};

fn is_type(e: &RawElement, t: ElementType) -> bool {
    e.frontmatter.element_type.as_ref() == Some(&t)
}

fn disp_id(e: &RawElement) -> String {
    e.frontmatter
        .id
        .clone()
        .unwrap_or_else(|| e.qualified_name.clone())
}

fn has_feature_model(elements: &[RawElement]) -> bool {
    elements.iter().any(|e| is_type(e, ElementType::FeatureDef))
}

/// Parameter names declared on a FeatureDef's `parameters:` list.
fn param_names(fd: &RawElement) -> Vec<String> {
    let Some(params) = &fd.frontmatter.parameters else {
        return Vec::new();
    };
    params
        .iter()
        .filter_map(|p| {
            p.as_mapping()
                .and_then(|m| m.get(serde_yaml::Value::String("name".into())))
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .collect()
}

fn requires_of(fd: &RawElement) -> Vec<String> {
    match &fd.frontmatter.requires {
        None => Vec::new(),
        Some(seq) => seq
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
    }
}

fn excludes_of(fd: &RawElement) -> Vec<String> {
    fd.frontmatter.excludes.clone().unwrap_or_default()
}

/// Configurations (by display id) that select `q` true.
fn configs_selecting(elements: &[RawElement], q: &str) -> Vec<String> {
    let mut ids: Vec<String> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::Configuration))
        .filter(|c| c.frontmatter.feature_selections().get(q).copied().unwrap_or(false))
        .map(disp_id)
        .collect();
    ids.sort();
    ids
}

fn total_configs(elements: &[RawElement]) -> usize {
    elements
        .iter()
        .filter(|e| is_type(e, ElementType::Configuration))
        .count()
}

// ── features: feature-model overview ────────────────────────────────────────

pub fn cmd_features(elements: &[RawElement], json: bool) {
    if !has_feature_model(elements) {
        if json {
            println!("{}", json!({ "note": "no feature model present", "features": [] }));
        } else {
            println!("No feature model present — nothing to show.");
        }
        return;
    }

    let mut fdefs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::FeatureDef))
        .collect();
    fdefs.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
    let total = total_configs(elements);

    if json {
        let items: Vec<_> = fdefs
            .iter()
            .map(|fd| {
                let q = &fd.qualified_name;
                json!({
                    "qualifiedName": q,
                    "groupKind": fd.frontmatter.group_kind.as_deref().unwrap_or("optional"),
                    "mandatory": fd.frontmatter.mandatory,
                    "requires": requires_of(fd),
                    "excludes": excludes_of(fd),
                    "parameters": param_names(fd),
                    "selectedIn": configs_selecting(elements, q).len(),
                    "totalConfigurations": total,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({ "features": items })).unwrap()
        );
        return;
    }

    println!("# Feature Model");
    println!();
    for fd in &fdefs {
        let q = &fd.qualified_name;
        // Indentation follows namespace nesting depth.
        let depth = q.matches("::").count();
        let indent = "  ".repeat(depth);
        let gk = fd.frontmatter.group_kind.as_deref().unwrap_or("optional");
        let n = configs_selecting(elements, q).len();
        println!("{}- {} [{}] — selected in {}/{}", indent, q, gk, n, total);
        if fd.frontmatter.mandatory == Some(true) {
            println!("{}    mandatory: true", indent);
        }
        let reqs = requires_of(fd);
        if !reqs.is_empty() {
            println!("{}    requires: {}", indent, reqs.join(", "));
        }
        let exc = excludes_of(fd);
        if !exc.is_empty() {
            println!("{}    excludes: {}", indent, exc.join(", "));
        }
        let params = param_names(fd);
        if !params.is_empty() {
            println!("{}    parameters: {}", indent, params.join(", "));
        }
    }
}

// ── feature <q>: single-feature card ────────────────────────────────────────

pub fn cmd_feature(elements: &[RawElement], arg: &str, json: bool) {
    let Some(fd) = elements.iter().find(|e| {
        is_type(e, ElementType::FeatureDef)
            && (e.qualified_name == arg || e.frontmatter.name.as_deref() == Some(arg))
    }) else {
        eprintln!("Error: '{}' is not a known FeatureDef", arg);
        std::process::exit(1);
    };
    let q = &fd.qualified_name;

    // Gates: elements whose effective appliesWhen (own or via an ancestor package,
    // REQ-TRS-VAR-006) names this feature as an operand.
    let gpkg = variability::package_conditions(elements);
    let mut gates: Vec<(&str, &str)> = elements
        .iter()
        .filter(|e| {
            variability::effective_expr(e, &gpkg)
                .map(|expr| expr.operands().iter().any(|o| o == q))
                .unwrap_or(false)
        })
        .map(|e| {
            (
                e.qualified_name.as_str(),
                crate::query::type_label(e.frontmatter.element_type.as_ref().unwrap_or(&ElementType::Package)),
            )
        })
        .collect();
    gates.sort();

    let selecting = configs_selecting(elements, q);
    let params = param_names(fd);
    let reqs = requires_of(fd);
    let exc = excludes_of(fd);
    let gk = fd.frontmatter.group_kind.as_deref().unwrap_or("optional");

    if json {
        let gate_items: Vec<_> = gates
            .iter()
            .map(|(qn, ty)| json!({ "qualifiedName": qn, "type": ty }))
            .collect();
        let doc = json!({
            "qualifiedName": q,
            "groupKind": gk,
            "mandatory": fd.frontmatter.mandatory,
            "requires": reqs,
            "excludes": exc,
            "parameters": params,
            "gates": gate_items,
            "selectedIn": selecting,
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return;
    }

    println!("# Feature: {}", q);
    println!();
    println!("- groupKind: {}", gk);
    if fd.frontmatter.mandatory == Some(true) {
        println!("- mandatory: true");
    }
    if !reqs.is_empty() {
        println!("- requires: {}", reqs.join(", "));
    }
    if !exc.is_empty() {
        println!("- excludes: {}", exc.join(", "));
    }
    if !params.is_empty() {
        println!("- parameters: {}", params.join(", "));
    }
    println!();
    println!("## Gates");
    if gates.is_empty() {
        println!("(no element gates on this feature)");
    } else {
        for (qn, ty) in &gates {
            println!("- {} ({})", qn, ty);
        }
    }
    println!();
    println!("## Selected in");
    if selecting.is_empty() {
        println!("(no Configuration selects this feature)");
    } else {
        for c in &selecting {
            println!("- {}", c);
        }
    }
}

// ── why-active: explain element activation under a configuration ─────────────

pub fn cmd_why_active(elements: &[RawElement], key: &str, config: Option<&str>, json: bool) {
    // Resolve the element by qualified name or stable id.
    let Some(elem) = elements.iter().find(|e| {
        e.qualified_name == key || e.frontmatter.id.as_deref() == Some(key)
    }) else {
        eprintln!("Error: element not found: {}", key);
        std::process::exit(1);
    };

    let Some(cfg_arg) = config else {
        eprintln!("Error: why-active requires --config <Configuration|features>");
        std::process::exit(1);
    };

    // Resolve the configuration selection. With no feature model the element is
    // always active.
    let sel = match projection::resolve_selection(elements, cfg_arg) {
        SelectionOutcome::Dormant => None,
        SelectionOutcome::Resolved(s) => Some(s),
        SelectionOutcome::Error(m) => {
            eprintln!("Error: {}", m);
            std::process::exit(1);
        }
    };

    // Effective condition: the element's own appliesWhen, else the nearest
    // ancestor package's (transitive package conditioning, REQ-TRS-VAR-006).
    let pkg = variability::package_conditions(elements);
    let eff = variability::effective_applies_when(elem, &pkg);
    let aw_source: Option<String> = eff.as_ref().and_then(|(_, src)| src.clone());
    let expr: Option<FeatureExpr> = eff
        .as_ref()
        .and_then(|(v, _)| variability::applies_when_expr(v).ok().flatten());

    // Build the verdict.
    let (verdict, referenced): (&str, Vec<(String, bool)>) = match (&expr, &sel) {
        (None, _) => ("always active", Vec::new()),
        (Some(e), None) => {
            // No feature model — element treated as always active.
            ("always active", e.operands().into_iter().map(|o| (o, false)).collect())
        }
        (Some(e), Some(s)) => {
            let active = e.eval(&|q: &str| s.get(q).copied().unwrap_or(false));
            let refs: Vec<(String, bool)> = e
                .operands()
                .into_iter()
                .map(|o| {
                    let v = s.get(&o).copied().unwrap_or(false);
                    (o, v)
                })
                .collect();
            (if active { "active" } else { "inactive" }, refs)
        }
    };

    let aw_str = eff
        .as_ref()
        .map(|(v, _)| render_aw(v))
        .unwrap_or_else(|| "(none)".to_string());

    if json {
        let refs: Vec<_> = referenced
            .iter()
            .map(|(q, v)| json!({ "feature": q, "selected": v }))
            .collect();
        let doc = json!({
            "element": elem.qualified_name,
            "appliesWhen": aw_str,
            "appliesWhenSource": aw_source.clone().map(|p| format!("package {}", p)),
            "config": cfg_arg,
            "references": refs,
            "verdict": verdict,
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
    } else {
        println!("# why-active: {}", elem.qualified_name);
        println!();
        println!("- appliesWhen: {}", aw_str);
        if let Some(src) = &aw_source {
            println!("- inherited from: package {}", src);
        }
        println!("- config: {}", cfg_arg);
        if !referenced.is_empty() {
            println!("- referenced feature selections:");
            for (q, v) in &referenced {
                println!("    {} = {}", q, v);
            }
        }
        println!();
        println!("Verdict: {}", verdict);
    }
}

fn render_aw(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Sequence(seq) => seq
            .iter()
            .filter_map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join(" and "),
        other => format!("{:?}", other),
    }
}

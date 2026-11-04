//! `change-impact` — which Configurations change validity or projection if a feature, constraint or parameter is
//! edited as proposed (GH #423, `REQ-TRS-IMPACT-001`).
//!
//! The edit is applied to a **candidate copy** of the model by the guarded-write engine in dry-run mode, so the
//! model on disk is never touched. Each Configuration's variant (its validation findings and its set of active
//! elements) is computed on the live model and on the candidate and compared. One payload for the CLI, the MCP tool
//! and the LSP request `syscribe/changeImpact`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{json, Value};
use syscribe_model::config::ValidateConfig;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;

pub type Fail = (i32, String);

/// A proposed edit.
#[derive(Debug, Clone, Default)]
pub struct Request {
    /// The element to edit (qualified name or id): a `FeatureDef`, or a package carrying `parameterConstraints:`.
    pub element: String,
    /// Frontmatter fields to set (a JSON object; `null` removes a key), as `update_element` takes them.
    pub set: Option<Value>,
    /// Edit one entry of the element's `parameters:` list: `{name, set}`.
    pub parameter: Option<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Variant {
    errors: BTreeSet<(String, String, String)>,
    active: BTreeSet<String>,
}

/// Every Configuration's variant: its error findings (code, file, message) and its active qualified names.
fn variants(elements: &[RawElement], config: &ValidateConfig) -> BTreeMap<String, Variant> {
    let mut out = BTreeMap::new();
    // A Configuration is also valid or not as a selection of the feature model (a required feature left out, an
    // exclusion violated, a group's cardinality broken): those findings belong to its file.
    let fm = syscribe_model::feature_model::check_feature_model(elements);
    for cfg in elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration)) {
        let sel = syscribe_model::projection::canonical_selection(elements, cfg);
        let active = syscribe_model::projection::project(elements, &sel).iter().map(|e| e.qualified_name.clone()).collect();
        let errors = syscribe_model::projection::validate_projected(elements, config, &sel)
            .into_iter()
            .filter(|f| matches!(f.severity, syscribe_model::validator::Severity::Error))
            .map(|f| (f.code.to_string(), f.file, f.message))
            .chain(
                fm.iter()
                    .filter(|f| matches!(f.severity, syscribe_model::validator::Severity::Error) && f.file == cfg.file_path)
                    .map(|f| (f.code.to_string(), f.file.clone(), f.message.clone())),
            )
            .collect();
        out.insert(cfg.frontmatter.id.clone().unwrap_or_else(|| cfg.qualified_name.clone()), Variant { errors, active });
    }
    out
}

/// Where an element lives, ready to jump to: its file and the line of its `id:` (else 1).
pub fn location(e: &RawElement) -> Value {
    let line = std::fs::read_to_string(&e.file_path)
        .ok()
        .and_then(|t| t.lines().take(60).position(|l| l.starts_with("id:")).map(|i| i + 1))
        .unwrap_or(1);
    json!({ "file": e.file_path, "line": line })
}

pub fn compute(model_root: &Path, elements: &[RawElement], config: &ValidateConfig, req: &Request) -> Result<Value, Fail> {
    let resolver = Resolver::new(elements);
    let target = resolver
        .resolve_ref(elements, &req.element)
        .or_else(|| resolver.get(elements, &req.element))
        .ok_or_else(|| (1, format!("element not found: {}", req.element)))?;
    if req.set.is_none() && req.parameter.is_none() {
        return Err((2, "nothing to change: give `set` and/or `parameter`".to_string()));
    }
    if let Some(s) = &req.set {
        if !s.is_object() {
            return Err((2, "`set` must be an object of frontmatter fields".to_string()));
        }
    }
    let model_abs = std::fs::canonicalize(model_root).unwrap_or_else(|_| model_root.to_path_buf());
    let file_abs = std::fs::canonicalize(&target.file_path).unwrap_or_else(|_| Path::new(&target.file_path).to_path_buf());
    let rel = file_abs.strip_prefix(&model_abs).map_err(|_| (1, "element is outside the model root".to_string()))?.to_path_buf();

    let before = variants(elements, config);
    let apply = |cand: &Path| -> Result<(), String> {
        let path = cand.join(&rel);
        let content = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", rel.display()))?;
        let mut updated = syscribe_model::mutate::update::apply_update_fields(&content, req.set.as_ref(), None).map_err(|e| format!("{e:?}"))?;
        if let Some((name, fields)) = &req.parameter {
            let mut found = false;
            updated = syscribe_model::frontmatter::patch_frontmatter(&updated, None, |map| {
                let key = serde_yaml::Value::String("parameters".into());
                if let Some(serde_yaml::Value::Sequence(seq)) = map.get_mut(&key) {
                    for p in seq.iter_mut() {
                        if let serde_yaml::Value::Mapping(m) = p {
                            if m.get(serde_yaml::Value::String("name".into())).and_then(|v| v.as_str()) == Some(name.as_str()) {
                                found = true;
                                if let Value::Object(o) = fields {
                                    for (k, v) in o {
                                        let k = serde_yaml::Value::String(k.clone());
                                        if v.is_null() {
                                            m.remove(&k);
                                        } else {
                                            m.insert(k, serde_yaml::to_value(v).unwrap_or(serde_yaml::Value::Null));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            })
            .map_err(|e| format!("{e:?}"))?;
            if !found {
                return Err(format!("{} has no parameter '{name}'", req.element));
            }
        }
        // The candidate is a throwaway copy: write it directly, as the model-level guarded writes do.
        std::fs::write(&path, updated).map_err(|e| e.to_string())
    };
    let inspect = |cand: &[RawElement]| -> BTreeMap<String, Variant> {
        // `with_model_root` on the candidate would be the exact config, but the candidate's root is not known here;
        // the live config is the same content unless the edit touches `.syscribe.toml`.
        let mut out = variants(cand, config);
        // Findings in the candidate name files under its temporary root: report them under the real one.
        let cand_root = cand
            .iter()
            .find_map(|e| e.file_path.strip_suffix(&*rel.to_string_lossy()).map(|r| r.trim_end_matches('/').to_string()))
            .unwrap_or_default();
        if !cand_root.is_empty() {
            let live = model_abs.to_string_lossy().to_string();
            for v in out.values_mut() {
                v.errors = v.errors.iter().map(|(c, f, m)| (c.clone(), f.replace(&cand_root, &live), m.replace(&cand_root, &live))).collect();
            }
        }
        out
    };
    let (outcome, after) = syscribe_model::mutate::guard::guarded_write_inspect(
        model_root,
        elements,
        config,
        true, // dry run: never written
        false,
        false,
        apply,
        Some(&inspect),
        |_| false,
    );
    if let Some(reason) = &outcome.reason {
        return Err((1, reason.clone()));
    }
    let after = after.ok_or_else(|| (1, "the candidate model could not be evaluated".to_string()))?;

    let ids: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    let mut configs = Vec::new();
    let (mut invalid, mut valid, mut projected) = (0usize, 0usize, 0usize);
    for id in ids {
        let (b, a) = (before.get(id), after.get(id));
        let empty = Variant { errors: BTreeSet::new(), active: BTreeSet::new() };
        let (b, a) = (b.unwrap_or(&empty), a.unwrap_or(&empty));
        let new_errors: Vec<_> = a.errors.difference(&b.errors).collect();
        let resolved: Vec<_> = b.errors.difference(&a.errors).collect();
        let added: Vec<&String> = a.active.difference(&b.active).collect();
        let removed: Vec<&String> = b.active.difference(&a.active).collect();
        let became_invalid = b.errors.is_empty() && !a.errors.is_empty();
        let became_valid = !b.errors.is_empty() && a.errors.is_empty();
        let changed = !new_errors.is_empty() || !resolved.is_empty() || !added.is_empty() || !removed.is_empty();
        invalid += became_invalid as usize;
        valid += became_valid as usize;
        projected += (!added.is_empty() || !removed.is_empty()) as usize;
        let finding = |(c, f, m): &&(String, String, String)| json!({ "code": c, "file": f, "message": m });
        configs.push(json!({
            "configuration": id,
            "changed": changed,
            "validityBefore": { "errors": b.errors.len() },
            "validityAfter": { "errors": a.errors.len() },
            "becameInvalid": became_invalid,
            "becameValid": became_valid,
            "newErrors": new_errors.iter().map(finding).collect::<Vec<_>>(),
            "resolvedErrors": resolved.iter().map(finding).collect::<Vec<_>>(),
            "projection": { "added": added, "removed": removed },
        }));
    }
    let entry = |(c, f, m): &(String, String, String)| json!({ "code": c, "file": f, "message": m });
    Ok(json!({
        "schemaVersion": "1.0",
        "element": target.frontmatter.id.clone().unwrap_or_else(|| target.qualified_name.clone()),
        "qualifiedName": target.qualified_name,
        "location": location(target),
        "written": false,
        "diff": outcome.diff,
        "configurations": configs,
        "summary": {
            "configurations": before.len().max(after.len()),
            "changed": configs.iter().filter(|c| c["changed"] == true).count(),
            "becameInvalid": invalid,
            "becameValid": valid,
            "projectionChanged": projected,
        },
        "modelFindings": {
            "newErrors": outcome.new_errors.iter().map(entry).collect::<Vec<_>>(),
            "newWarnings": outcome.new_warnings.iter().map(entry).collect::<Vec<_>>(),
        },
    }))
}

const USAGE: &str = "Usage: syscribe -m <root> change-impact <element> [--set <field>=<yaml>]... [--param <name> --param-set <field>=<yaml>]... [--json]";

fn parse_yaml_value(s: &str) -> Result<Value, String> {
    let (k, v) = s.split_once('=').ok_or_else(|| format!("'{s}' is not <field>=<value>"))?;
    let y: serde_yaml::Value = serde_yaml::from_str(v).map_err(|e| format!("'{v}' is not YAML: {e}"))?;
    Ok(json!({ k.trim(): serde_json::to_value(y).map_err(|e| e.to_string())? }))
}

fn merge(into: &mut Value, from: Value) {
    if let (Value::Object(a), Value::Object(b)) = (into, from) {
        a.extend(b);
    }
}

pub fn parse_args(args: &[String]) -> Result<(Request, bool), String> {
    let (mut req, mut json) = (Request::default(), false);
    let mut param: Option<String> = None;
    let mut param_set = json!({});
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |f: &str| it.next().cloned().ok_or_else(|| format!("change-impact: {f} needs a value"));
        match a.as_str() {
            "--json" => json = true,
            "--set" => {
                let v = parse_yaml_value(&val("--set")?).map_err(|m| format!("change-impact: --set {m}"))?;
                merge(req.set.get_or_insert_with(|| json!({})), v);
            }
            "--param" => param = Some(val("--param")?),
            "--param-set" => {
                let v = parse_yaml_value(&val("--param-set")?).map_err(|m| format!("change-impact: --param-set {m}"))?;
                merge(&mut param_set, v);
            }
            f if f.starts_with('-') => return Err(format!("change-impact: unknown option '{f}'")),
            _ if req.element.is_empty() => req.element = a.clone(),
            _ => return Err(USAGE.to_string()),
        }
    }
    if req.element.is_empty() {
        return Err(USAGE.to_string());
    }
    match param {
        Some(p) => req.parameter = Some((p, param_set)),
        None if param_set.as_object().is_some_and(|o| !o.is_empty()) => return Err("change-impact: --param-set needs --param <name>".to_string()),
        None => {}
    }
    Ok((req, json))
}

pub fn cmd_change_impact(model_root: &Path, elements: &[RawElement], config: &ValidateConfig, args: &[String]) -> i32 {
    let (req, json) = match parse_args(args) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("{m}");
            return 2;
        }
    };
    let out = match compute(model_root, elements, config, &req) {
        Ok(o) => o,
        Err((c, m)) => {
            eprintln!("{m}");
            return c;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return 0;
    }
    let s = &out["summary"];
    println!("# Change impact on {}\n", out["element"].as_str().unwrap_or(""));
    println!("{} of {} configuration(s) change: {} become invalid, {} become valid, {} change projection. Nothing was written.\n", s["changed"], s["configurations"], s["becameInvalid"], s["becameValid"], s["projectionChanged"]);
    for c in out["configurations"].as_array().into_iter().flatten().filter(|c| c["changed"] == true) {
        println!("- **{}**: errors {} -> {}", c["configuration"].as_str().unwrap_or(""), c["validityBefore"]["errors"], c["validityAfter"]["errors"]);
        for e in c["newErrors"].as_array().into_iter().flatten() {
            println!("  - new {}: {}", e["code"].as_str().unwrap_or(""), e["message"].as_str().unwrap_or(""));
        }
        let (add, rem) = (c["projection"]["added"].as_array().map_or(0, Vec::len), c["projection"]["removed"].as_array().map_or(0, Vec::len));
        if add + rem > 0 {
            println!("  - projection: +{add} -{rem} element(s)");
        }
    }
    0
}

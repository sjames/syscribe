//! `bind` — write one `parameterBindings:` value or `features:` selection into a Configuration, checked before it is
//! written (GH #418, `REQ-TRS-BIND-001`).
//!
//! [`plan`] is pure: it resolves the Configuration and the target, classifies it (feature selection or parameter),
//! and produces the new file content. The CLI and the MCP `bind` tool both start from it; each then previews the
//! change on a candidate copy through the guarded-write engine (range, unit, stage window and narrowing are the
//! validator's own checks, so nothing is re-implemented here) and commits only when the preview is clean.

use serde_json::{json, Value};
use syscribe_model::element::{ElementType, RawElement};

pub type Fail = (i32, String);

#[derive(Debug, Clone)]
pub struct Request {
    /// The Configuration (id or qualified name).
    pub configuration: String,
    /// A feature (selection) or `<feature>.<parameter>` (binding), by qualified name or `FEAT-*` id.
    pub target: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub configuration: String,
    pub file: String,
    pub new_content: String,
    /// `selection` or `parameter`.
    pub kind: &'static str,
    /// The canonical key written (`Features::A` or `Features::A.param`).
    pub key: String,
    /// The effective value before the change and where it came from.
    pub previous: Option<Value>,
    pub stage: Option<String>,
}

fn ident(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Every feature qualified name and `feature.param` key, for a did-you-mean.
fn known_targets(elements: &[RawElement]) -> Vec<String> {
    let mut v = Vec::new();
    for fd in elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef)) {
        v.push(fd.qualified_name.clone());
        for p in fd.frontmatter.parameters.iter().flatten() {
            if let Some(n) = p.get("name").and_then(|n| n.as_str()) {
                v.push(format!("{}.{n}", fd.qualified_name));
            }
        }
    }
    v
}

pub fn plan(elements: &[RawElement], req: &Request) -> Result<Plan, Fail> {
    let cfg_idx = elements
        .iter()
        .position(|e| e.frontmatter.element_type == Some(ElementType::Configuration) && (e.frontmatter.id.as_deref() == Some(&req.configuration) || e.qualified_name == req.configuration))
        .ok_or_else(|| (1, format!("Configuration not found: {}", req.configuration)))?;
    let cfg = &elements[cfg_idx];
    let alias = syscribe_model::variability::feature_id_to_qname(elements);
    let canon = syscribe_model::variability::canon_feature_ref(&req.target, &alias);

    // A feature, or a `feature.parameter`.
    let feature = |q: &str| elements.iter().find(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef) && e.qualified_name == q);
    let (kind, key) = if feature(&canon).is_some() {
        ("selection", canon.clone())
    } else if let Some((f, p)) = req.target.rsplit_once('.') {
        let fq = syscribe_model::variability::canon_feature_ref(f, &alias);
        match feature(&fq) {
            Some(fd) if fd.frontmatter.parameters.iter().flatten().any(|x| x.get("name").and_then(|n| n.as_str()) == Some(p)) => ("parameter", format!("{fq}.{p}")),
            Some(fd) => {
                let names: Vec<String> = fd.frontmatter.parameters.iter().flatten().filter_map(|x| x.get("name").and_then(|n| n.as_str()).map(str::to_string)).collect();
                return Err((1, format!("{fq} has no parameter '{p}'.{} Its parameters: {}", crate::suggest::did_you_mean(p, names.iter().map(String::as_str)), if names.is_empty() { "(none)".into() } else { names.join(", ") })));
            }
            None => return Err((1, unknown_target(elements, &req.target))),
        }
    } else {
        return Err((1, unknown_target(elements, &req.target)));
    };
    if kind == "selection" && !req.value.is_boolean() {
        return Err((2, format!("a feature selection is true or false, not {}", req.value)));
    }
    if req.value.is_null() || req.value.is_object() || req.value.is_array() {
        return Err((2, "the value must be a scalar (number, string or boolean)".to_string()));
    }

    // What it is now, and where that came from.
    let prov = syscribe_model::provenance::provenance(elements, cfg_idx);
    let previous = if kind == "parameter" {
        prov.parameters.iter().find(|p| p.key == key).map(|p| json!({ "value": serde_json::to_value(&p.value).unwrap_or(Value::Null), "source": p.source.kind(), "stage": p.stage }))
    } else {
        prov.selections.iter().find(|s| s.feature == key).map(|s| json!({ "value": s.selected, "source": s.source.kind(), "stage": s.stage }))
    };

    let section = if kind == "parameter" { "parameterBindings" } else { "features" };
    let content = std::fs::read_to_string(&cfg.file_path).map_err(|e| (1, format!("cannot read {}: {e}", cfg.file_path)))?;
    let canon_key = |k: &str| -> String {
        match (kind, k.rsplit_once('.')) {
            ("parameter", Some((f, p))) => format!("{}.{p}", syscribe_model::variability::canon_feature_ref(f, &alias)),
            _ => syscribe_model::variability::canon_feature_ref(k, &alias),
        }
    };
    let value_text = scalar_text(&req.value);
    // Edit only the lines that change, so the rest of the file (quoting, comments, order) is byte-identical. A file
    // whose section is not a plain block mapping is rewritten through the YAML model instead.
    let new_content = match splice_entry(&content, section, &key, &value_text, &canon_key) {
        Some(c) => c,
        None => {
            let yaml = serde_yaml::to_value(&req.value).map_err(|e| (2, e.to_string()))?;
            syscribe_model::frontmatter::patch_frontmatter(&content, None, |map| {
                let skey = serde_yaml::Value::String(section.into());
                if !matches!(map.get(&skey), Some(serde_yaml::Value::Mapping(_))) {
                    map.insert(skey.clone(), serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
                }
                if let Some(serde_yaml::Value::Mapping(m)) = map.get_mut(&skey) {
                    let spelled = m.keys().find(|k| k.as_str().is_some_and(|s| canon_key(s) == key)).cloned().unwrap_or_else(|| serde_yaml::Value::String(key.clone()));
                    m.insert(spelled, yaml.clone());
                }
            })
            .map_err(|e| (1, format!("{e:?}")))?
        }
    };

    Ok(Plan {
        configuration: ident(cfg),
        file: cfg.file_path.clone(),
        new_content,
        kind,
        key,
        previous,
        stage: cfg.frontmatter.binding_stage.clone(),
    })
}

/// How a value is written in YAML: numbers and booleans plain, text quoted only when it must be.
fn scalar_text(v: &Value) -> String {
    match v {
        Value::String(t) => syscribe_model::frontmatter::yaml_scalar(t),
        other => other.to_string(),
    }
}

/// `key: rest` split at the first colon that ends the key (a `::` inside a qualified name does not).
fn split_key(line: &str) -> Option<(&str, &str)> {
    let b = line.as_bytes();
    let i = (0..b.len()).find(|&i| b[i] == b':' && (i + 1 == b.len() || b[i + 1] == b' '))?;
    Some((&line[..i], &line[i + 1..]))
}

/// Set `key: value` inside the top-level block mapping `section:` of the frontmatter, touching only that line (or
/// appending one, or the whole section). `None` when the section is not a plain block mapping.
fn splice_entry(content: &str, section: &str, key: &str, value: &str, canon_key: &dyn Fn(&str) -> String) -> Option<String> {
    let (yaml, _) = syscribe_model::frontmatter::split_frontmatter(content);
    let yaml = yaml?;
    let lines: Vec<&str> = yaml.lines().collect();
    let head = format!("{section}:");
    let start = lines.iter().position(|l| l.starts_with(&head));
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    match start {
        None => {
            out.push(format!("{section}:"));
            out.push(format!("  {key}: {value}"));
        }
        Some(i) => {
            // `section: {…}` or `section: value` on the same line is not a block mapping.
            if !lines[i][head.len()..].trim().is_empty() && !lines[i][head.len()..].trim().starts_with('#') {
                return None;
            }
            let end = lines[i + 1..].iter().position(|l| !l.is_empty() && !l.starts_with(char::is_whitespace)).map_or(lines.len(), |n| i + 1 + n);
            let body = &lines[i + 1..end];
            if body.iter().any(|l| l.trim_start().starts_with('-')) {
                return None; // a list, not a mapping
            }
            let indent: String = body.iter().find(|l| !l.trim().is_empty()).map_or("  ".to_string(), |l| l.chars().take_while(|c| c.is_whitespace()).collect());
            let hit = (i + 1..end).find(|&j| {
                let t = lines[j].trim_start();
                !t.starts_with('#') && split_key(t).is_some_and(|(k, _)| canon_key(k.trim().trim_matches(|c| c == '"' || c == '\'')) == key)
            });
            match hit {
                Some(j) => {
                    let (k, rest) = split_key(lines[j])?;
                    // Keep a trailing comment.
                    let comment = rest.find(" #").map(|c| rest[c..].to_string()).unwrap_or_default();
                    out[j] = format!("{k}: {value}{comment}");
                }
                None => {
                    // After the last non-blank line of the block.
                    let last = (i + 1..end).rev().find(|&j| !lines[j].trim().is_empty()).unwrap_or(i);
                    out.insert(last + 1, format!("{indent}{key}: {value}"));
                }
            }
        }
    }
    Some(syscribe_model::frontmatter::splice_frontmatter(content, yaml, &out.join("\n")))
}

fn unknown_target(elements: &[RawElement], target: &str) -> String {
    let known = known_targets(elements);
    format!("'{target}' is not a feature or a <feature>.<parameter> of this model.{}", crate::suggest::did_you_mean(target, known.iter().map(String::as_str)))
}

/// Parse a command-line value: `true`/`false`, a number, else a string.
pub fn parse_value(s: &str) -> Value {
    match s {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => serde_json::from_str::<Value>(s).ok().filter(|v| v.is_number()).unwrap_or_else(|| Value::String(s.to_string())),
    }
}

/// The report of a bind, from its plan and the guarded-write preview.
/// The entries of a finding list once each (the same finding can be raised by two checks).
fn uniq(v: &[(String, String, String)]) -> Vec<&(String, String, String)> {
    // The same finding can be raised by two checks and name the file two ways: once per (code, message).
    let mut seen = std::collections::HashSet::new();
    v.iter().filter(|e| seen.insert((e.0.clone(), e.2.clone()))).collect()
}

pub fn report(plan: &Plan, value: &Value, outcome: &syscribe_model::mutate::guard::GuardedWriteOutcome, written: bool, refused: Option<&str>) -> Value {
    let entry = |(c, f, m): &(String, String, String)| json!({ "code": c, "file": f, "message": m });
    json!({
        "schemaVersion": "1.0",
        "configuration": plan.configuration,
        "target": { "kind": plan.kind, "key": plan.key },
        "value": value,
        "previous": plan.previous,
        "stage": plan.stage,
        "written": written,
        "refused": refused,
        "diff": outcome.diff,
        "newErrors": uniq(&outcome.new_errors).into_iter().map(entry).collect::<Vec<_>>(),
        "resolvedErrors": uniq(&outcome.resolved_errors).into_iter().map(entry).collect::<Vec<_>>(),
        "newWarnings": uniq(&outcome.new_warnings).into_iter().map(entry).collect::<Vec<_>>(),
        "resolvedWarnings": uniq(&outcome.resolved_warnings).into_iter().map(entry).collect::<Vec<_>>(),
    })
}

const USAGE: &str = "Usage: syscribe -m <root> bind <Configuration> <feature | feature.parameter> <value> [--dry-run] [--json]";

pub fn cmd_bind(model_root: &std::path::Path, elements: &[RawElement], config: &syscribe_model::config::ValidateConfig, args: &[String]) -> i32 {
    let json = args.iter().any(|a| a == "--json");
    let pos: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    if let Some(bad) = args.iter().find(|a| a.starts_with("--") && *a != "--json" && *a != "--dry-run") {
        eprintln!("bind: unknown option '{bad}'");
        return 2;
    }
    let [conf, target, value] = pos.as_slice() else {
        eprintln!("{USAGE}");
        return 2;
    };
    let req = Request { configuration: (*conf).clone(), target: (*target).clone(), value: parse_value(value) };
    let plan = match plan(elements, &req) {
        Ok(p) => p,
        Err((c, m)) => {
            eprintln!("{m}");
            return c;
        }
    };
    if let Some(e) = elements.iter().find(|e| e.file_path == plan.file) {
        crate::provenance::check_expect_hash(std::path::Path::new(&plan.file));
        crate::policy_gate::check_path(e);
    }
    // Preview on a candidate copy: the model's own checks (range, unit, stage window, narrowing) decide.
    let rel = std::path::Path::new(&plan.file).strip_prefix(std::fs::canonicalize(model_root).unwrap_or_else(|_| model_root.to_path_buf())).map(|p| p.to_path_buf()).or_else(|_| std::path::Path::new(&plan.file).strip_prefix(model_root).map(|p| p.to_path_buf()));
    let Ok(rel) = rel else {
        eprintln!("{} is not under the model root", plan.file);
        return 1;
    };
    let content = plan.new_content.clone();
    let (outcome, _) = syscribe_model::mutate::guard::guarded_write_inspect(
        model_root,
        elements,
        config,
        true,
        false,
        true,
        move |root: &std::path::Path| std::fs::write(root.join(&rel), &content).map_err(|e| e.to_string()),
        None::<&dyn Fn(&[RawElement]) -> ()>,
        |_| false,
    );
    if let Some(r) = &outcome.reason {
        eprintln!("{r}");
        return 1;
    }
    let dry = syscribe_model::fsio::is_dry_run();
    let n_new = uniq(&outcome.new_errors).len();
    let refused = (n_new > 0).then(|| format!("refused: the binding would introduce {n_new} error(s)"));
    let mut written = false;
    if refused.is_none() && !dry {
        if let Err(e) = syscribe_model::fsio::write(&plan.file, &plan.new_content) {
            eprintln!("Write failed for {}: {e}", plan.file);
            return 1;
        }
        // Under --json the report is the output; a second document (the global envelope) would break its consumers.
        if !json {
            crate::provenance::record_diff(&outcome.diff);
        }
        written = true;
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&report(&plan, &req.value, &outcome, written, refused.as_deref())).unwrap());
    } else {
        println!("# Bind {} {} = {}{}\n", plan.configuration, plan.key, req.value, plan.stage.as_deref().map(|s| format!(" (stage {s})")).unwrap_or_default());
        if let Some(p) = &plan.previous {
            println!("Was {} ({}).", p["value"], p["source"].as_str().unwrap_or(""));
        }
        for e in uniq(&outcome.new_errors) {
            println!("- new error {}: {}", e.0, e.2);
        }
        for w in uniq(&outcome.new_warnings) {
            println!("- new warning {}: {}", w.0, w.2);
        }
        for e in uniq(&outcome.resolved_errors) {
            println!("- resolves {}: {}", e.0, e.2);
        }
        match (&refused, written, dry) {
            (Some(r), _, _) => println!("\n{r}. Nothing was written."),
            (None, true, _) => println!("\nWritten to {}.", plan.file),
            (None, false, true) => println!("\nDry run: nothing was written."),
            _ => {}
        }
    }
    if refused.is_some() {
        1
    } else {
        0
    }
}

//! Holistic feature-model validation (§9), surfaced via the explicit
//! `syscribe feature-check` command — deliberately **not** part of the default
//! `validate` pass, which stays per-element and fast.
//!
//! Structural rules (REQ-TRS-FM-002): `E212` requires/excludes resolution,
//! `E219`/`E220` requires/excludes satisfaction per Configuration, `W011`/`W012`
//! dead / always-on optional features.
//!
//! Parameter-integrity rules (REQ-TRS-FM-003): `E207` `derivedFrom:` cycles,
//! `E202` `bindTo:` propagation range, `E213` unresolved `parameterConstraints`
//! paths, `W014` constraint `appliesWhen:` features absent from every
//! Configuration.

use std::collections::{HashMap, HashSet};

use crate::element::{ElementType, RawElement};
use crate::validator::{Finding, Severity};

fn err(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Error }
}
fn warn(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Warning }
}

fn is(e: &RawElement, t: ElementType) -> bool {
    e.frontmatter.element_type.as_ref() == Some(&t)
}

fn strings_from_seq(v: &[serde_yaml::Value]) -> Vec<String> {
    v.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()
}

fn strings_from_value(v: &serde_yaml::Value) -> Vec<String> {
    match v {
        serde_yaml::Value::String(s) => vec![s.clone()],
        serde_yaml::Value::Sequence(seq) => strings_from_seq(seq),
        _ => vec![],
    }
}

fn num(v: &serde_yaml::Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))
}

fn parse_range(s: &str) -> Option<(f64, f64)> {
    let (lo, hi) = s.split_once("..")?;
    Some((lo.trim().parse().ok()?, hi.trim().parse().ok()?))
}

struct Param {
    name: String,
    range: Option<(f64, f64)>,
    derived_from: Option<String>,
    bind_to: Option<String>,
}

fn parse_params(fd: &RawElement) -> Vec<Param> {
    let mut out = Vec::new();
    if let Some(list) = &fd.frontmatter.parameters {
        for p in list {
            let serde_yaml::Value::Mapping(m) = p else { continue };
            let get = |k: &str| m.get(serde_yaml::Value::String(k.to_string()));
            let Some(name) = get("name").and_then(|v| v.as_str()) else { continue };
            out.push(Param {
                name: name.to_string(),
                range: get("range").and_then(|v| v.as_str()).and_then(parse_range),
                derived_from: get("derivedFrom").and_then(|v| v.as_str()).map(|s| s.to_string()),
                bind_to: get("bindTo").and_then(|v| v.as_str()).map(|s| s.to_string()),
            });
        }
    }
    out
}

/// Whether `name` appears as a whole identifier token in `expr`.
fn token_present(expr: &str, name: &str) -> bool {
    expr.split(|c: char| !(c.is_alphanumeric() || c == '_')).any(|t| t == name)
}

/// Extract qualified parameter paths (tokens containing `::`) from an expression.
fn extract_param_paths(expr: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in expr.chars() {
        if c.is_alphanumeric() || c == '_' || c == ':' {
            cur.push(c);
        } else {
            if cur.contains("::") {
                out.push(cur.clone());
            }
            cur.clear();
        }
    }
    if cur.contains("::") {
        out.push(cur);
    }
    out
}

/// DFS cycle detection over a param-name dependency graph.
fn has_cycle(deps: &HashMap<String, Vec<String>>) -> bool {
    fn dfs(n: &str, deps: &HashMap<String, Vec<String>>, state: &mut HashMap<String, u8>) -> bool {
        state.insert(n.to_string(), 1); // 1 = on stack
        if let Some(ds) = deps.get(n) {
            for d in ds {
                match state.get(d).copied().unwrap_or(0) {
                    1 => return true,
                    0 if dfs(d, deps, state) => return true,
                    _ => {}
                }
            }
        }
        state.insert(n.to_string(), 2); // 2 = done
        false
    }
    let mut state: HashMap<String, u8> = HashMap::new();
    for k in deps.keys() {
        if state.get(k).copied().unwrap_or(0) == 0 && dfs(k, deps, &mut state) {
            return true;
        }
    }
    false
}

/// True iff the model declares at least one `FeatureDef`.
pub fn has_feature_model(elements: &[RawElement]) -> bool {
    elements.iter().any(|e| is(e, ElementType::FeatureDef))
}

/// Run all feature-model validation rules. Returns an empty vector when no
/// feature model is present (the command treats that as a dormant no-op).
pub fn check_feature_model(elements: &[RawElement]) -> Vec<Finding> {
    let mut f = Vec::new();
    if !has_feature_model(elements) {
        return f;
    }

    let fdefs: Vec<&RawElement> = elements.iter().filter(|e| is(e, ElementType::FeatureDef)).collect();
    let configs: Vec<&RawElement> =
        elements.iter().filter(|e| is(e, ElementType::Configuration)).collect();
    let fnames: HashSet<&str> = fdefs.iter().map(|e| e.qualified_name.as_str()).collect();

    // ── E212: requires/excludes entries resolve to a FeatureDef ──────────────
    let mut req: HashMap<&str, Vec<String>> = HashMap::new();
    let mut exc: HashMap<&str, Vec<String>> = HashMap::new();
    for fd in &fdefs {
        let mut requires = Vec::new();
        if let Some(r) = &fd.frontmatter.requires {
            requires = strings_from_seq(r);
        }
        let excludes = fd.frontmatter.excludes.clone().unwrap_or_default();
        for r in requires.iter().chain(excludes.iter()) {
            if !fnames.contains(r.as_str()) {
                f.push(err("E212", &fd.file_path,
                    format!("requires/excludes entry '{}' does not resolve to a FeatureDef", r)));
            }
        }
        req.insert(fd.qualified_name.as_str(), requires);
        exc.insert(fd.qualified_name.as_str(), excludes);
    }

    // ── E219/E220 (per Configuration) + selection counts for W011/W012 ───────
    let mut sel_count: HashMap<&str, usize> = HashMap::new();
    for cfg in &configs {
        let sel = cfg.frontmatter.feature_selections();
        let is_sel = |q: &str| sel.get(q).copied().unwrap_or(false);
        for fd in &fdefs {
            let q = fd.qualified_name.as_str();
            if !is_sel(q) {
                continue;
            }
            *sel_count.entry(q).or_insert(0) += 1;
            for r in req.get(q).into_iter().flatten() {
                if !is_sel(r) {
                    f.push(err("E219", &cfg.file_path,
                        format!("feature '{}' is selected but its required feature '{}' is not", q, r)));
                }
            }
            for x in exc.get(q).into_iter().flatten() {
                if is_sel(x) {
                    f.push(err("E220", &cfg.file_path,
                        format!("feature '{}' is selected but its excluded feature '{}' is also selected", q, x)));
                }
            }
        }
    }

    // ── W011/W012: dead / always-on optional features ────────────────────────
    if !configs.is_empty() {
        for fd in &fdefs {
            if fd.frontmatter.group_kind.as_deref() != Some("optional") {
                continue;
            }
            let q = fd.qualified_name.as_str();
            match sel_count.get(q).copied().unwrap_or(0) {
                0 => f.push(warn("W011", &fd.file_path,
                    format!("optional feature '{}' is selected in no Configuration (possible dead feature)", q))),
                n if n == configs.len() => f.push(warn("W012", &fd.file_path,
                    format!("optional feature '{}' is selected in every Configuration (consider groupKind: mandatory)", q))),
                _ => {}
            }
        }
    }

    // ── E207: circular derivedFrom among a FeatureDef's own parameters ───────
    for fd in &fdefs {
        let params = parse_params(fd);
        let names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        for p in &params {
            if let Some(expr) = &p.derived_from {
                let referenced: Vec<String> = names
                    .iter()
                    .filter(|n| n.as_str() != p.name && token_present(expr, n))
                    .cloned()
                    .collect();
                deps.insert(p.name.clone(), referenced);
            }
        }
        if has_cycle(&deps) {
            f.push(err("E207", &fd.file_path,
                format!("circular derivedFrom dependency among parameters of FeatureDef '{}'", fd.qualified_name)));
        }
    }

    // ── E202: two-level bindTo propagation range ─────────────────────────────
    struct Bind<'a> {
        feature: &'a str,
        name: String,
        bind_to: String,
        range: (f64, f64),
    }
    let mut binds: Vec<Bind> = Vec::new();
    for fd in &fdefs {
        for p in parse_params(fd) {
            if let (Some(bt), Some(range)) = (p.bind_to, p.range) {
                binds.push(Bind { feature: fd.qualified_name.as_str(), name: p.name, bind_to: bt, range });
            }
        }
    }
    if !binds.is_empty() {
        for cfg in &configs {
            let Some(serde_yaml::Value::Mapping(b)) = &cfg.frontmatter.parameter_bindings else {
                continue;
            };
            for bp in &binds {
                let Some(v) = b.get(serde_yaml::Value::String(bp.bind_to.clone())) else {
                    continue;
                };
                let Some(n) = num(v) else { continue };
                let (lo, hi) = bp.range;
                if n < lo || n > hi {
                    f.push(err("E202", &cfg.file_path, format!(
                        "value {} bound to '{}' propagates to component parameter '{}::{}', outside its range {}..{}",
                        n, bp.bind_to, bp.feature, bp.name, lo, hi)));
                }
            }
        }
    }

    // ── E213/W014: cross-feature parameterConstraints (package _index.md) ────
    let mut param_paths: HashSet<String> = HashSet::new();
    for fd in &fdefs {
        for p in parse_params(fd) {
            param_paths.insert(format!("{}::{}", fd.qualified_name, p.name));
        }
    }
    let mut selected_anywhere: HashSet<String> = HashSet::new();
    for cfg in &configs {
        for (k, v) in cfg.frontmatter.feature_selections() {
            if v {
                selected_anywhere.insert(k);
            }
        }
    }
    for pkg in elements.iter().filter(|e| {
        matches!(
            e.frontmatter.element_type,
            Some(ElementType::Package) | Some(ElementType::LibraryPackage) | Some(ElementType::Namespace)
        )
    }) {
        let Some(serde_yaml::Value::Sequence(cons)) =
            pkg.frontmatter.extra.get("parameterConstraints")
        else {
            continue;
        };
        for c in cons {
            let serde_yaml::Value::Mapping(m) = c else { continue };
            if let Some(expr) = m
                .get(serde_yaml::Value::String("expression".into()))
                .and_then(|v| v.as_str())
            {
                for path in extract_param_paths(expr) {
                    if !param_paths.contains(&path) {
                        f.push(err("E213", &pkg.file_path, format!(
                            "parameterConstraints expression references unresolved parameter path '{}'", path)));
                    }
                }
            }
            if let Some(aw) = m.get(serde_yaml::Value::String("appliesWhen".into())) {
                for feat in strings_from_value(aw) {
                    if !selected_anywhere.contains(&feat) {
                        f.push(warn("W014", &pkg.file_path, format!(
                            "parameterConstraints appliesWhen references feature '{}' not selected in any Configuration", feat)));
                    }
                }
            }
        }
    }

    f
}

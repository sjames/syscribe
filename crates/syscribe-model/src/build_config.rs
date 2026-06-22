//! Build-system artifact generation from a `Configuration` (§9.9).
//!
//! Resolves build variables from a named `Configuration` by consulting:
//!   1. `buildExports:` on each selected/deselected `FeatureDef`
//!   2. `parameterBindings:` → `buildVar:` on `FeatureDef` parameters
//!   3. `buildOverrides:` on the `Configuration` (last writer wins)
//!
//! The public API is [`resolve`] for a single configuration and
//! [`validate_build_exports`] for the default `validate` pass (E050 only).

use std::collections::BTreeMap;

use crate::element::{ElementType, RawElement};
use crate::validator::{Finding, Severity};

// ── Public types ──────────────────────────────────────────────────────────────

/// A resolved build variable set. `BTreeMap` gives deterministic (alphabetical)
/// iteration for reproducible diffs across formats.
pub type BuildVars = BTreeMap<String, serde_yaml::Value>;

/// Result of resolving the build variables for one `Configuration`.
pub struct BuildConfigResult {
    /// The resolved variable set (alphabetical).
    pub vars: BuildVars,
    /// E050 (conflict) and W050 (no build mapping) findings from this resolution.
    pub findings: Vec<Finding>,
}

// ── Internal helper types ─────────────────────────────────────────────────────

/// A single entry from a `FeatureDef`'s `buildExports:` list.
struct BuildExport {
    var: String,
    when_selected: serde_yaml::Value,
    when_deselected: Option<serde_yaml::Value>,
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn get_str<'a>(m: &'a serde_yaml::Mapping, key: &str) -> Option<&'a str> {
    m.get(serde_yaml::Value::String(key.to_string()))
        .and_then(|v| v.as_str())
}

fn get_val<'a>(m: &'a serde_yaml::Mapping, key: &str) -> Option<&'a serde_yaml::Value> {
    m.get(serde_yaml::Value::String(key.to_string()))
}

/// Parse the `buildExports:` list from a `FeatureDef` element.
fn parse_build_exports(fd: &RawElement) -> Vec<BuildExport> {
    let Some(list) = &fd.frontmatter.build_exports else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in list {
        let serde_yaml::Value::Mapping(m) = entry else { continue };
        let Some(var) = get_str(m, "var") else { continue };
        let when_selected = get_val(m, "whenSelected")
            .cloned()
            .unwrap_or_else(|| serde_yaml::Value::Number(serde_yaml::Number::from(1)));
        let when_deselected = get_val(m, "whenDeselected").cloned();
        out.push(BuildExport {
            var: var.to_string(),
            when_selected,
            when_deselected,
        });
    }
    out
}

/// Parse `(buildVar, default_value)` pairs from a `FeatureDef`'s `parameters:` list.
fn param_build_vars(fd: &RawElement) -> Vec<(String, Option<serde_yaml::Value>)> {
    let Some(list) = &fd.frontmatter.parameters else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for p in list {
        let serde_yaml::Value::Mapping(m) = p else { continue };
        let Some(build_var) = get_str(m, "buildVar") else { continue };
        let Some(param_name) = get_str(m, "name") else { continue };
        // The key used in parameterBindings is "<FeatureDef qname>.<param name>"
        // We need param_name to look up the binding, but we return the tuple
        // (buildVar, param_name, default) — caller constructs the binding key.
        let default_val = get_val(m, "default")
            .or_else(|| get_val(m, "value"))
            .cloned();
        // Store (build_var, param_name, default) encoded in a tuple-3.
        // But our return type is Vec<(String, Option<Value>)>.
        // We need to return param_name too so the caller can look up parameterBindings.
        // Re-encode: return (build_var + "\0" + param_name, default_val) and split on \0 in caller.
        out.push((format!("{}\0{}", build_var, param_name), default_val));
    }
    out
}

/// Look up a scalar value from a `parameterBindings` mapping given a dotted key
/// `"<FeatureDef qname>.<param>"`.
fn binding_value(
    parameter_bindings: &serde_yaml::Value,
    fd_qname: &str,
    param_name: &str,
) -> Option<serde_yaml::Value> {
    let serde_yaml::Value::Mapping(m) = parameter_bindings else { return None };
    let key = format!("{}.{}", fd_qname, param_name);
    get_val(m, &key).cloned()
}

fn err(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Error }
}

fn warn(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Warning }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Resolve the build variable set for the named `Configuration`.
///
/// Matching: by `frontmatter.id` first, then by `qualified_name` suffix (last
/// segment equals `conf_id`), following the same pattern used elsewhere in the
/// codebase (e.g. `configure`, `configure --config`).
///
/// Returns `Err(String)` when no matching `Configuration` is found.
pub fn resolve(elements: &[RawElement], conf_id: &str) -> Result<BuildConfigResult, String> {
    // ── Find the Configuration ────────────────────────────────────────────────
    let cfg = elements
        .iter()
        .find(|e| {
            matches!(e.frontmatter.element_type, Some(ElementType::Configuration))
                && (e.frontmatter.id.as_deref() == Some(conf_id)
                    || e.qualified_name == conf_id
                    || e.qualified_name.ends_with(&format!("::{}", conf_id)))
        })
        .ok_or_else(|| format!("Configuration '{}' not found", conf_id))?;

    let cfg_file = cfg.file_path.clone();

    // ── Canonicalize feature selections (id → qname) ─────────────────────────
    let feat_alias = crate::variability::feature_id_to_qname(elements);
    let selections = crate::variability::canon_selection(
        &cfg.frontmatter.feature_selections(),
        &feat_alias,
    );

    // ── Collect FeatureDefs ───────────────────────────────────────────────────
    let feat_defs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef)))
        .collect();

    let mut vars: BuildVars = BTreeMap::new();
    let mut findings: Vec<Finding> = Vec::new();

    // E050 tracking: var_name → Vec<(value, feature_qname)> from selected features
    let mut conflict_tracker: BTreeMap<String, Vec<(serde_yaml::Value, String)>> = BTreeMap::new();

    // ── Step 1: buildExports on FeatureDefs ──────────────────────────────────
    // Track which features contributed at least one build var (for W050).
    let mut features_with_exports: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for fd in &feat_defs {
        let qname = &fd.qualified_name;
        let selected = selections.get(qname.as_str()).copied().unwrap_or(false);

        let exports = parse_build_exports(fd);
        if !exports.is_empty() {
            features_with_exports.insert(qname.clone());
        }

        for export in &exports {
            if selected {
                conflict_tracker
                    .entry(export.var.clone())
                    .or_default()
                    .push((export.when_selected.clone(), qname.clone()));
                vars.insert(export.var.clone(), export.when_selected.clone());
            } else if let Some(ref when_desel) = export.when_deselected {
                vars.insert(export.var.clone(), when_desel.clone());
            }
        }
    }

    // ── E050: detect conflicts among selected features ────────────────────────
    // Build the override key set (vars that buildOverrides will cover) so we can
    // suppress E050 for those. We need to read buildOverrides before the check.
    let override_keys: std::collections::HashSet<String> =
        if let Some(serde_yaml::Value::Mapping(m)) = &cfg.frontmatter.build_overrides {
            m.keys()
                .filter_map(|k| k.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            std::collections::HashSet::new()
        };

    for (var_name, contributors) in &conflict_tracker {
        if contributors.len() < 2 {
            continue;
        }
        // All same value? no conflict.
        let first_val = &contributors[0].0;
        if contributors.iter().all(|(v, _)| v == first_val) {
            continue;
        }
        // Resolved by buildOverrides? no E050.
        if override_keys.contains(var_name) {
            continue;
        }
        let feature_list: Vec<&str> = contributors.iter().map(|(_, q)| q.as_str()).collect();
        findings.push(err(
            "E050",
            &cfg_file,
            format!(
                "build variable '{}' has conflicting values from selected features [{}]; \
                 add a buildOverrides: entry to resolve",
                var_name,
                feature_list.join(", ")
            ),
        ));
    }

    // ── Step 2: parameterBindings → buildVar on parameters ───────────────────
    let param_bindings = cfg.frontmatter.parameter_bindings.as_ref();

    for fd in &feat_defs {
        let qname = &fd.qualified_name;
        let selected = selections.get(qname.as_str()).copied().unwrap_or(false);
        if !selected {
            continue;
        }

        let pbvars = param_build_vars(fd);
        if !pbvars.is_empty() {
            features_with_exports.insert(qname.clone());
        }
        for (encoded, default_val) in pbvars {
            let (build_var, param_name) = encoded.split_once('\0').unwrap();
            let val = param_bindings
                .and_then(|pb| binding_value(pb, qname, param_name))
                .or(default_val);
            if let Some(v) = val {
                vars.insert(build_var.to_string(), v);
            }
        }
    }

    // ── Step 3: buildOverrides win unconditionally ────────────────────────────
    if let Some(serde_yaml::Value::Mapping(m)) = &cfg.frontmatter.build_overrides {
        for (k, v) in m {
            if let Some(k_str) = k.as_str() {
                vars.insert(k_str.to_string(), v.clone());
            }
        }
    }

    // ── W050: selected features with no build mapping (opt-in) ───────────────
    for fd in &feat_defs {
        let qname = &fd.qualified_name;
        let selected = selections.get(qname.as_str()).copied().unwrap_or(false);
        if selected && !features_with_exports.contains(qname.as_str()) {
            findings.push(warn(
                "W050",
                &fd.file_path,
                format!(
                    "selected feature '{}' contributes no build variable \
                     (add buildExports: or a parameter with buildVar:, or gate with --deny W050)",
                    qname
                ),
            ));
        }
    }

    Ok(BuildConfigResult { vars, findings })
}

/// Validate `buildExports` across all `Configuration` elements and emit `E050`
/// for variable-name conflicts that are not resolved by `buildOverrides`.
///
/// Called from `validate_with_config` only when at least one element has
/// `build_exports.is_some()` or `build_overrides.is_some()`, so it is zero-cost
/// on models that do not use the build-system integration feature.
///
/// W050 (selected feature with no build mapping) is **not** emitted here because
/// it is opt-in (gate with `--deny W050`). It is only emitted by `resolve`.
pub fn validate_build_exports(elements: &[RawElement]) -> Vec<Finding> {
    let mut findings = Vec::new();

    let configs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Configuration)))
        .collect();

    for cfg in configs {
        // Use resolve() and harvest only E050 findings (not W050).
        let conf_id = cfg
            .frontmatter
            .id
            .as_deref()
            .unwrap_or(&cfg.qualified_name);
        match resolve(elements, conf_id) {
            Ok(result) => {
                for f in result.findings {
                    if f.code == "E050" {
                        findings.push(f);
                    }
                }
            }
            Err(_) => {
                // If the config can't be found by its own id, skip it.
            }
        }
    }

    findings
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse in test");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("model/{}.md", qname.replace("::", "/")),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
        }
    }

    // ── Basic whenSelected ────────────────────────────────────────────────────

    #[test]
    fn test_basic_when_selected() {
        let fd = make_elem(
            "Features::ABS",
            r#"
type: FeatureDef
id: FEAT-ABS
name: ABS
buildExports:
  - var: ENABLE_ABS
    whenSelected: 1
"#,
        );
        let cfg = make_elem(
            "Configs::Premium",
            r#"
type: Configuration
id: CONF-PREMIUM-001
name: Premium
features:
  Features::ABS: true
"#,
        );
        let elements = vec![fd, cfg];
        let result = resolve(&elements, "CONF-PREMIUM-001").unwrap();
        assert_eq!(
            result.vars.get("ENABLE_ABS"),
            Some(&serde_yaml::Value::Number(serde_yaml::Number::from(1)))
        );
        let errs: Vec<_> = result.findings.iter().filter(|f| f.code == "E050").collect();
        assert!(errs.is_empty(), "unexpected E050: {:?}", errs);
    }

    // ── whenDeselected absent → var omitted ───────────────────────────────────

    #[test]
    fn test_when_deselected_absent_omits_var() {
        let fd = make_elem(
            "Features::ABS",
            r#"
type: FeatureDef
id: FEAT-ABS
name: ABS
buildExports:
  - var: ENABLE_ABS
    whenSelected: 1
"#,
        );
        let cfg = make_elem(
            "Configs::Base",
            r#"
type: Configuration
id: CONF-BASE-001
name: Base
features:
  Features::ABS: false
"#,
        );
        let elements = vec![fd, cfg];
        let result = resolve(&elements, "CONF-BASE-001").unwrap();
        assert!(
            !result.vars.contains_key("ENABLE_ABS"),
            "var should be absent when deselected with no whenDeselected"
        );
    }

    // ── whenDeselected present → var emitted ─────────────────────────────────

    #[test]
    fn test_when_deselected_present_emits() {
        let fd = make_elem(
            "Features::ABS",
            r#"
type: FeatureDef
id: FEAT-ABS
name: ABS
buildExports:
  - var: ENABLE_ABS
    whenSelected: 1
    whenDeselected: 0
"#,
        );
        let cfg = make_elem(
            "Configs::Base",
            r#"
type: Configuration
id: CONF-BASE-001
name: Base
features:
  Features::ABS: false
"#,
        );
        let elements = vec![fd, cfg];
        let result = resolve(&elements, "CONF-BASE-001").unwrap();
        assert_eq!(
            result.vars.get("ENABLE_ABS"),
            Some(&serde_yaml::Value::Number(serde_yaml::Number::from(0)))
        );
    }

    // ── parameterBindings → buildVar ──────────────────────────────────────────

    #[test]
    fn test_parameter_build_var() {
        let fd = make_elem(
            "Features::Radar",
            r#"
type: FeatureDef
id: FEAT-RADAR
name: Radar
parameters:
  - name: maxRange
    type: ScalarValues::Real
    range: "0..200"
    default: 100
    buildVar: RADAR_MAX_RANGE
"#,
        );
        let cfg = make_elem(
            "Configs::Premium",
            r#"
type: Configuration
id: CONF-PREMIUM-001
name: Premium
features:
  Features::Radar: true
parameterBindings:
  Features::Radar.maxRange: 150
"#,
        );
        let elements = vec![fd, cfg];
        let result = resolve(&elements, "CONF-PREMIUM-001").unwrap();
        assert_eq!(
            result.vars.get("RADAR_MAX_RANGE"),
            Some(&serde_yaml::Value::Number(serde_yaml::Number::from(150)))
        );
    }

    // ── parameterBindings absent → use default ────────────────────────────────

    #[test]
    fn test_parameter_build_var_default_fallback() {
        let fd = make_elem(
            "Features::Radar",
            r#"
type: FeatureDef
id: FEAT-RADAR
name: Radar
parameters:
  - name: maxRange
    type: ScalarValues::Real
    range: "0..200"
    default: 42
    buildVar: RADAR_MAX_RANGE
"#,
        );
        let cfg = make_elem(
            "Configs::Premium",
            r#"
type: Configuration
id: CONF-PREMIUM-001
name: Premium
features:
  Features::Radar: true
"#,
        );
        let elements = vec![fd, cfg];
        let result = resolve(&elements, "CONF-PREMIUM-001").unwrap();
        assert_eq!(
            result.vars.get("RADAR_MAX_RANGE"),
            Some(&serde_yaml::Value::Number(serde_yaml::Number::from(42)))
        );
    }

    // ── buildOverrides win ────────────────────────────────────────────────────

    #[test]
    fn test_build_overrides_win() {
        let fd = make_elem(
            "Features::ABS",
            r#"
type: FeatureDef
id: FEAT-ABS
name: ABS
buildExports:
  - var: X
    whenSelected: 1
"#,
        );
        let cfg = make_elem(
            "Configs::Premium",
            r#"
type: Configuration
id: CONF-PREMIUM-001
name: Premium
features:
  Features::ABS: true
buildOverrides:
  X: 99
"#,
        );
        let elements = vec![fd, cfg];
        let result = resolve(&elements, "CONF-PREMIUM-001").unwrap();
        assert_eq!(
            result.vars.get("X"),
            Some(&serde_yaml::Value::Number(serde_yaml::Number::from(99)))
        );
    }

    // ── E050 conflict ─────────────────────────────────────────────────────────

    #[test]
    fn test_e050_conflict() {
        let fd1 = make_elem(
            "Features::ABS",
            r#"
type: FeatureDef
id: FEAT-ABS
name: ABS
buildExports:
  - var: SAFETY_MODE
    whenSelected: 1
"#,
        );
        let fd2 = make_elem(
            "Features::ESC",
            r#"
type: FeatureDef
id: FEAT-ESC
name: ESC
buildExports:
  - var: SAFETY_MODE
    whenSelected: 2
"#,
        );
        let cfg = make_elem(
            "Configs::Full",
            r#"
type: Configuration
id: CONF-FULL-001
name: Full
features:
  Features::ABS: true
  Features::ESC: true
"#,
        );
        let elements = vec![fd1, fd2, cfg];
        let result = resolve(&elements, "CONF-FULL-001").unwrap();
        let e050: Vec<_> = result.findings.iter().filter(|f| f.code == "E050").collect();
        assert!(!e050.is_empty(), "expected E050 conflict finding");
    }

    // ── E050 suppressed by buildOverrides ─────────────────────────────────────

    #[test]
    fn test_e050_suppressed_by_override() {
        let fd1 = make_elem(
            "Features::ABS",
            r#"
type: FeatureDef
id: FEAT-ABS
name: ABS
buildExports:
  - var: SAFETY_MODE
    whenSelected: 1
"#,
        );
        let fd2 = make_elem(
            "Features::ESC",
            r#"
type: FeatureDef
id: FEAT-ESC
name: ESC
buildExports:
  - var: SAFETY_MODE
    whenSelected: 2
"#,
        );
        let cfg = make_elem(
            "Configs::Full",
            r#"
type: Configuration
id: CONF-FULL-001
name: Full
features:
  Features::ABS: true
  Features::ESC: true
buildOverrides:
  SAFETY_MODE: 3
"#,
        );
        let elements = vec![fd1, fd2, cfg];
        let result = resolve(&elements, "CONF-FULL-001").unwrap();
        let e050: Vec<_> = result.findings.iter().filter(|f| f.code == "E050").collect();
        assert!(e050.is_empty(), "E050 should be suppressed by buildOverrides");
        assert_eq!(
            result.vars.get("SAFETY_MODE"),
            Some(&serde_yaml::Value::Number(serde_yaml::Number::from(3)))
        );
    }

    // ── W050: selected feature with no build vars ─────────────────────────────

    #[test]
    fn test_w050_no_build_vars() {
        let fd = make_elem(
            "Features::ABS",
            r#"
type: FeatureDef
id: FEAT-ABS
name: ABS
"#,
        );
        let cfg = make_elem(
            "Configs::Premium",
            r#"
type: Configuration
id: CONF-PREMIUM-001
name: Premium
features:
  Features::ABS: true
"#,
        );
        let elements = vec![fd, cfg];
        let result = resolve(&elements, "CONF-PREMIUM-001").unwrap();
        let w050: Vec<_> = result.findings.iter().filter(|f| f.code == "W050").collect();
        assert!(!w050.is_empty(), "expected W050 for selected feature with no build vars");
        assert_eq!(w050[0].severity, Severity::Warning);
    }

    // ── Output is alphabetical (BTreeMap) ─────────────────────────────────────

    #[test]
    fn test_output_alphabetical() {
        let fd = make_elem(
            "Features::F",
            r#"
type: FeatureDef
id: FEAT-F
name: F
buildExports:
  - var: ZZZ
    whenSelected: 1
  - var: AAA
    whenSelected: 2
  - var: MMM
    whenSelected: 3
"#,
        );
        let cfg = make_elem(
            "Configs::C",
            r#"
type: Configuration
id: CONF-C-001
name: C
features:
  Features::F: true
"#,
        );
        let elements = vec![fd, cfg];
        let result = resolve(&elements, "CONF-C-001").unwrap();
        let keys: Vec<&String> = result.vars.keys().collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "vars must be in alphabetical order");
    }

    // ── Multiple buildExports entries ─────────────────────────────────────────

    #[test]
    fn test_multiple_exports_per_feature() {
        let fd = make_elem(
            "Features::Camera",
            r#"
type: FeatureDef
id: FEAT-CAM
name: Camera
buildExports:
  - var: ENABLE_CAMERA
    whenSelected: 1
    whenDeselected: 0
  - var: CAM_RESOLUTION
    whenSelected: 1080
"#,
        );
        let cfg = make_elem(
            "Configs::Premium",
            r#"
type: Configuration
id: CONF-PREMIUM-001
name: Premium
features:
  Features::Camera: true
"#,
        );
        let elements = vec![fd, cfg];
        let result = resolve(&elements, "CONF-PREMIUM-001").unwrap();
        assert_eq!(
            result.vars.get("ENABLE_CAMERA"),
            Some(&serde_yaml::Value::Number(serde_yaml::Number::from(1)))
        );
        assert_eq!(
            result.vars.get("CAM_RESOLUTION"),
            Some(&serde_yaml::Value::Number(serde_yaml::Number::from(1080)))
        );
    }

    // ── Config not found ──────────────────────────────────────────────────────

    #[test]
    fn test_conf_not_found() {
        let elements: Vec<RawElement> = vec![];
        let result = resolve(&elements, "CONF-NONEXISTENT-001");
        assert!(result.is_err());
    }

    // ── Config lookup by qualified name ───────────────────────────────────────

    #[test]
    fn test_conf_lookup_by_qname() {
        let cfg = make_elem(
            "Configs::MyConf",
            r#"
type: Configuration
id: CONF-MY-001
name: My Config
features: {}
"#,
        );
        let elements = vec![cfg];
        // Lookup by full qname
        let result = resolve(&elements, "Configs::MyConf");
        assert!(result.is_ok(), "should find config by qualified name");
        // Lookup by last segment
        let result2 = resolve(&elements, "MyConf");
        assert!(result2.is_ok(), "should find config by last qname segment");
    }

    // ── validate_build_exports: E050 via the validator entry point ────────────

    #[test]
    fn test_validate_build_exports_emits_e050() {
        let fd1 = make_elem(
            "Features::A",
            r#"
type: FeatureDef
id: FEAT-A
name: A
buildExports:
  - var: VAR
    whenSelected: 1
"#,
        );
        let fd2 = make_elem(
            "Features::B",
            r#"
type: FeatureDef
id: FEAT-B
name: B
buildExports:
  - var: VAR
    whenSelected: 2
"#,
        );
        let cfg = make_elem(
            "Configs::C",
            r#"
type: Configuration
id: CONF-C-001
name: C
features:
  Features::A: true
  Features::B: true
"#,
        );
        let elements = vec![fd1, fd2, cfg];
        let findings = validate_build_exports(&elements);
        assert!(
            findings.iter().any(|f| f.code == "E050"),
            "expected E050 from validate_build_exports"
        );
    }

    // ── validate_build_exports: no E050 when model has no buildExports ────────

    #[test]
    fn test_validate_build_exports_silent_for_plain_model() {
        // A plain model with no buildExports/buildOverrides.
        let elements: Vec<RawElement> = vec![];
        let findings = validate_build_exports(&elements);
        assert!(findings.is_empty());
    }
}

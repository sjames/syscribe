//! Feature-parameter placeholders in element text (GH #264/#265, REQ-TRS-PHOLD-001).
//!
//! `{{Features::Path::Feature.param}}` (optionally `|unit`) in an element's Markdown body or
//! `name` is kept symbolic in the base model and replaced, in a projection that matches a stored
//! `Configuration`, by that configuration's binding, else the parameter's fixed `value:`, else its
//! `default:`. A placeholder that cannot be resolved is left as written.

use regex::Regex;

use crate::element::RawElement;
use crate::element::ElementType;
use crate::resolver::Resolver;

/// One `{{feature.param}}` occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placeholder {
    /// The text as written, braces included.
    pub raw: String,
    /// The feature reference (qualified name or `FEAT-*` id).
    pub feature: String,
    pub param: String,
    /// `|unit` suffix: append the parameter's unit.
    pub unit: bool,
}

fn re() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\{\{\s*([A-Za-z_][A-Za-z0-9_\-:]*)\.([A-Za-z_][A-Za-z0-9_]*)\s*(\|\s*unit)?\s*\}\}").unwrap())
}

/// Byte ranges of `text` that are Markdown code (fenced blocks and inline `` ` `` spans). A
/// placeholder inside code is literal, so documentation can show the syntax.
fn code_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut pos = 0;
    let mut fence: Option<(usize, char)> = None;
    for line in text.split_inclusive('\n') {
        let end = pos + line.len();
        let t = line.trim_start();
        let fence_char = if t.starts_with("```") {
            Some('`')
        } else if t.starts_with("~~~") {
            Some('~')
        } else {
            None
        };
        match (fence, fence_char) {
            (Some((s, c)), Some(fc)) if c == fc => {
                out.push((s, end));
                fence = None;
            }
            (None, Some(fc)) => fence = Some((pos, fc)),
            (None, None) => {
                // Inline spans on this line: a run of N backticks closes at the next run of N.
                let b = line.as_bytes();
                let mut i = 0;
                while i < b.len() {
                    if b[i] == b'`' {
                        let start = i;
                        while i < b.len() && b[i] == b'`' {
                            i += 1;
                        }
                        let n = i - start;
                        let mut j = i;
                        let mut close = None;
                        while j < b.len() {
                            if b[j] == b'`' {
                                let cs = j;
                                while j < b.len() && b[j] == b'`' {
                                    j += 1;
                                }
                                if j - cs == n {
                                    close = Some(j);
                                    break;
                                }
                            } else {
                                j += 1;
                            }
                        }
                        match close {
                            Some(c) => {
                                out.push((pos + start, pos + c));
                                i = c;
                            }
                            None => break,
                        }
                    } else {
                        i += 1;
                    }
                }
            }
            _ => {}
        }
        pos = end;
    }
    if let Some((s, _)) = fence {
        out.push((s, text.len())); // an unterminated fence runs to the end
    }
    out
}

fn in_code(ranges: &[(usize, usize)], at: usize) -> bool {
    ranges.iter().any(|&(a, b)| at >= a && at < b)
}

/// Every placeholder in `text` (outside code spans and fences), in order.
pub fn find(text: &str) -> Vec<Placeholder> {
    let code = code_ranges(text);
    re()
        .captures_iter(text)
        .filter(|c| !in_code(&code, c.get(0).map(|m| m.start()).unwrap_or(0)))
        // Only feature-shaped references count; `{{ user.name }}` template text is not ours.
        .filter(|c| c[1].contains("::") || c[1].starts_with("FEAT-"))
        .map(|c| Placeholder {
            raw: c[0].to_string(),
            feature: c[1].to_string(),
            param: c[2].to_string(),
            unit: c.get(3).is_some(),
        })
        .collect()
}

/// What a placeholder-capable typed field accepts (GH #268, REQ-TRS-PHOLDFIELD-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// A finite number ≥ 0.
    Rate,
    /// A number in 0..=1.
    Fraction,
    /// An integer 1..=4 (as E009 requires).
    Sil,
    /// `QM`, `A`..`D`.
    Asil,
    /// `CAL1`..`CAL4`.
    Cal,
}

/// The frontmatter keys (YAML spelling) that may hold a whole-value placeholder.
pub const FIELDS: &[(&str, FieldKind)] = &[
    ("failureRate", FieldKind::Rate),
    ("diagnosticCoverage", FieldKind::Fraction),
    ("latentDiagnosticCoverage", FieldKind::Fraction),
    ("silLevel", FieldKind::Sil),
    ("asilLevel", FieldKind::Asil),
    ("calLevel", FieldKind::Cal),
];

fn field_kind(key: &str) -> Option<FieldKind> {
    FIELDS.iter().find(|(k, _)| *k == key).map(|(_, k)| *k)
}

fn whole_re() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s*\{\{\s*([A-Za-z_][A-Za-z0-9_\-:]*)\.([A-Za-z_][A-Za-z0-9_]*)\s*\}\}\s*$").unwrap())
}

/// The placeholder `text` consists of, when it is exactly one (no `|unit`, nothing around it).
pub fn parse_whole(text: &str) -> Option<Placeholder> {
    let c = whole_re().captures(text)?;
    if !(c[1].contains("::") || c[1].starts_with("FEAT-")) {
        return None;
    }
    Some(Placeholder { raw: text.trim().to_string(), feature: c[1].to_string(), param: c[2].to_string(), unit: false })
}

/// Remove the whole-value placeholders of the placeholder-capable fields from frontmatter YAML text:
/// the top-level `key: "{{Feature.param}}"` line (double or single quoted, nothing after it but a
/// comment) of each capable key. Returns the remaining YAML — otherwise byte-identical, so every
/// other scalar keeps the text it was written with — and YAML key → placeholder text.
pub fn take_field_placeholders(yaml: &str) -> (String, std::collections::BTreeMap<String, String>) {
    let mut out = std::collections::BTreeMap::new();
    let mut kept = String::with_capacity(yaml.len());
    for line in yaml.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        let taken = FIELDS.iter().find_map(|(key, _)| {
            let rest = body.strip_prefix(key)?.strip_prefix(':')?;
            let rest = rest.trim();
            let (quote, inner) = match rest.chars().next()? {
                q @ ('"' | '\'') => (q, &rest[1..]),
                _ => return None,
            };
            let end = inner.find(quote)?;
            let after = inner[end + 1..].trim();
            if !(after.is_empty() || after.starts_with('#')) {
                return None;
            }
            let value = &inner[..end];
            parse_whole(value).map(|_| ((*key).to_string(), value.trim().to_string()))
        });
        match taken {
            Some((k, v)) => {
                out.insert(k, v);
            }
            None => kept.push_str(line),
        }
    }
    (kept, out)
}

/// The parameter's own `range:` / `enumValues:` (they also bind a fixed `value:` or `default:`).
fn check_param(d: &Decl, value: &str) -> Result<(), String> {
    let v = value.trim();
    if let Some(r) = d.entry.get("range").and_then(|r| r.as_str()) {
        if let Some((lo, hi)) = r.split_once("..") {
            let hi = hi.trim();
            let hi = hi.strip_prefix('=').unwrap_or(hi).trim();
            if let (Ok(lo), Ok(hi)) = (lo.trim().parse::<f64>(), hi.parse::<f64>()) {
                match v.parse::<f64>() {
                    Ok(n) if n >= lo && n <= hi => {}
                    _ => return Err(format!("the parameter's range {r}")),
                }
            }
        }
    }
    if let Some(serde_yaml::Value::Sequence(vals)) = d.entry.get("enumValues") {
        let allowed: Vec<String> = vals.iter().filter_map(scalar).collect();
        if !allowed.is_empty() && !allowed.iter().any(|a| a == v) {
            return Err(format!("the parameter's enumValues {allowed:?}"));
        }
    }
    Ok(())
}

/// Check `value` against `kind`; the error says what the field accepts.
fn check_field(kind: FieldKind, value: &str) -> Result<(), String> {
    let v = value.trim();
    match kind {
        FieldKind::Rate => match v.parse::<f64>() {
            Ok(n) if n.is_finite() && n >= 0.0 => Ok(()),
            _ => Err("a finite number >= 0".to_string()),
        },
        FieldKind::Fraction => match v.parse::<f64>() {
            Ok(n) if (0.0..=1.0).contains(&n) => Ok(()),
            _ => Err("a number from 0 to 1".to_string()),
        },
        FieldKind::Sil => match v.parse::<u8>() {
            Ok(n) if (1..=4).contains(&n) => Ok(()),
            _ => Err("an integer from 1 to 4".to_string()),
        },
        FieldKind::Asil => {
            if ["QM", "A", "B", "C", "D"].contains(&v.to_ascii_uppercase().as_str()) {
                Ok(())
            } else {
                Err("one of QM, A, B, C, D".to_string())
            }
        }
        FieldKind::Cal => {
            if ["CAL1", "CAL2", "CAL3", "CAL4"].contains(&v.to_ascii_uppercase().as_str()) {
                Ok(())
            } else {
                Err("one of CAL1, CAL2, CAL3, CAL4".to_string())
            }
        }
    }
}

/// Store an already checked `value` in the typed field `key` of `fm`.
fn apply_field(fm: &mut crate::element::RawFrontmatter, key: &str, value: &str) {
    let v = value.trim();
    match key {
        "failureRate" => fm.failure_rate = v.parse().ok(),
        "diagnosticCoverage" => fm.diagnostic_coverage = v.parse().ok(),
        "latentDiagnosticCoverage" => fm.latent_diagnostic_coverage = v.parse().ok(),
        "silLevel" => fm.sil_level = v.parse().ok(),
        "asilLevel" => fm.asil_level = Some(v.to_ascii_uppercase()),
        "calLevel" => fm.cal_level = Some(v.to_ascii_uppercase()),
        _ => {}
    }
}

fn texts(e: &RawElement) -> [&str; 2] {
    [e.doc.as_str(), e.frontmatter.name.as_deref().unwrap_or("")]
}

/// Placeholders in an element's body and name.
pub fn of_element(e: &RawElement) -> Vec<Placeholder> {
    let mut v: Vec<Placeholder> = texts(e).iter().flat_map(|t| find(t)).collect();
    v.extend(e.frontmatter.placeholder_fields.values().filter_map(|raw| parse_whole(raw)));
    v
}

/// Whether any element uses a placeholder (cheap pre-check).
pub fn any(elements: &[RawElement]) -> bool {
    elements.iter().any(|e| !e.frontmatter.placeholder_fields.is_empty() || texts(e).iter().any(|t| t.contains("{{") && !find(t).is_empty()))
}

/// A parameter declared on a `FeatureDef`.
struct Decl<'a> {
    feature_qname: String,
    entry: &'a serde_yaml::Mapping,
}

fn decl<'a>(elements: &'a [RawElement], resolver: &Resolver, feature: &str, param: &str) -> Result<Decl<'a>, String> {
    let Some(fd) = resolver.resolve_ref(elements, feature).filter(|f| f.frontmatter.element_type == Some(ElementType::FeatureDef)) else {
        return Err(format!("'{feature}' does not resolve to a FeatureDef"));
    };
    for p in fd.frontmatter.parameters.iter().flatten() {
        if let serde_yaml::Value::Mapping(m) = p {
            if m.get("name").and_then(|v| v.as_str()) == Some(param) {
                return Ok(Decl { feature_qname: fd.qualified_name.clone(), entry: m });
            }
        }
    }
    Err(format!("feature '{}' declares no parameter '{param}'", fd.qualified_name))
}

fn scalar(v: &serde_yaml::Value) -> Option<String> {
    match v {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// The configuration's own binding for `feature_qname.param` (keys may name the feature by id).
fn binding(elements: &[RawElement], resolver: &Resolver, cfg: &RawElement, feature_qname: &str, param: &str) -> Option<String> {
    let serde_yaml::Value::Mapping(m) = cfg.frontmatter.effective_parameter_bindings()? else { return None };
    for (k, v) in m {
        let Some(k) = k.as_str() else { continue };
        let Some((feat, p)) = k.rsplit_once('.') else { continue };
        if p != param {
            continue;
        }
        let q = resolver.resolve_ref(elements, feat).map(|f| f.qualified_name.as_str()).unwrap_or(feat);
        if q == feature_qname {
            return scalar(v);
        }
    }
    None
}

fn unit_of(d: &Decl) -> Option<String> {
    d.entry.get("unit").and_then(scalar)
}

/// The value of one placeholder for `cfg` (a stored Configuration): its binding, else the
/// fixed `value:`, else the `default:`. `None` when none is available.
fn value_for(elements: &[RawElement], resolver: &Resolver, cfg: Option<&RawElement>, ph: &Placeholder) -> Option<String> {
    let d = decl(elements, resolver, &ph.feature, &ph.param).ok()?;
    let v = cfg
        .and_then(|c| binding(elements, resolver, c, &d.feature_qname, &ph.param))
        .or_else(|| d.entry.get("value").and_then(scalar))
        .or_else(|| d.entry.get("default").and_then(scalar))?;
    Some(match (ph.unit, unit_of(&d)) {
        (true, Some(u)) => format!("{v} {u}"),
        _ => v,
    })
}

fn substitute_text(text: &str, f: &dyn Fn(&Placeholder) -> Option<String>) -> String {
    let code = code_ranges(text);
    re()
        .replace_all(text, |c: &regex::Captures| {
            if in_code(&code, c.get(0).map(|m| m.start()).unwrap_or(0)) {
                return c[0].to_string();
            }
            if !(c[1].contains("::") || c[1].starts_with("FEAT-")) {
                return c[0].to_string();
            }
            let ph = Placeholder { raw: c[0].to_string(), feature: c[1].to_string(), param: c[2].to_string(), unit: c.get(3).is_some() };
            f(&ph).unwrap_or_else(|| ph.raw.clone())
        })
        .into_owned()
}

/// The stored Configuration whose (canonical) selection equals `sel`, if any.
pub fn matching_config<'a>(elements: &'a [RawElement], sel: &crate::projection::Selection) -> Option<&'a RawElement> {
    // Configurations that select the same features differ only in their bindings; prefer the one
    // the caller just resolved (see `projection::current_config`) when it matches the selection.
    if let Some(cur) = crate::projection::current_config() {
        if let Some(c) = elements.iter().find(|e| {
            e.frontmatter.element_type == Some(ElementType::Configuration) && e.qualified_name == cur
        }) {
            if &crate::projection::canonical_selection(elements, c) == sel {
                return Some(c);
            }
        }
    }
    elements
        .iter()
        .filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration))
        .find(|c| &crate::projection::canonical_selection(elements, c) == sel)
}

/// Replace the placeholders of the projected elements `view` for the configuration matching
/// `sel` (a selection with no matching Configuration uses fixed values and defaults only).
/// `full` is the whole model (feature declarations and the Configuration live there).
pub fn substitute(view: &mut [RawElement], full: &[RawElement], sel: &crate::projection::Selection) {
    if !any(view) {
        return;
    }
    let resolver = Resolver::new(full);
    let cfg = matching_config(full, sel);
    let f = |ph: &Placeholder| value_for(full, &resolver, cfg, ph);
    for e in view.iter_mut() {
        // Placeholder-capable typed fields: resolved, checked against the field, stored typed. An
        // unresolved or invalid value leaves the field unset (E247/E243 report why).
        for (key, raw) in e.frontmatter.placeholder_fields.clone() {
            let (Some(ph), Some(kind)) = (parse_whole(&raw), field_kind(&key)) else { continue };
            if let Some(v) = f(&ph) {
                let param_ok = decl(full, &resolver, &ph.feature, &ph.param).map(|d| check_param(&d, &v).is_ok()).unwrap_or(false);
                if param_ok && check_field(kind, &v).is_ok() {
                    apply_field(&mut e.frontmatter, &key, &v);
                }
            }
        }
        if e.doc.contains("{{") {
            e.doc = substitute_text(&e.doc, &f);
        }
        if let Some(n) = e.frontmatter.name.as_deref() {
            if n.contains("{{") {
                e.frontmatter.name = Some(substitute_text(n, &f));
            }
        }
    }
}

/// `(is_error, code, file, message)` for every placeholder problem in the base model.
pub fn findings(elements: &[RawElement], resolver: &Resolver) -> Vec<(bool, &'static str, String, String)> {
    use crate::variability;
    let mut out = Vec::new();
    if !any(elements) {
        return out;
    }
    let has_features = elements.iter().any(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef));
    let configs: Vec<&RawElement> =
        elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration)).collect();
    let pkg = variability::package_conditions(elements);
    let alias = variability::feature_id_to_qname(elements);
    let sels: Vec<crate::projection::Selection> =
        configs.iter().map(|c| variability::canon_selection(&c.frontmatter.feature_selections(), &alias)).collect();
    for e in elements {
        let phs = of_element(e);
        if phs.is_empty() {
            continue;
        }
        if !has_features || configs.is_empty() {
            out.push((
                true,
                "E240",
                e.file_path.clone(),
                "element uses a feature-parameter placeholder, but the model has no FeatureDef or no Configuration".to_string(),
            ));
            continue;
        }
        let status = e.frontmatter.status.as_deref().unwrap_or("");
        let firm = matches!(status, "approved" | "implemented" | "verified");
        // A typed field's value must suit the field in every configuration that fills it (E247); an
        // unknown reference is E241 below and an unbound parameter E243/W245, as for text.
        for (key, raw) in &e.frontmatter.placeholder_fields {
            let (Some(ph), Some(kind)) = (parse_whole(raw), field_kind(key)) else { continue };
            let Ok(d) = decl(elements, resolver, &ph.feature, &ph.param) else { continue };
            for (c, sel) in configs.iter().zip(&sels) {
                if !crate::projection::is_active_canon(e, sel, &pkg, &alias) || sel.get(&d.feature_qname).copied() != Some(true) {
                    continue;
                }
                let Some(v) = value_for(elements, resolver, Some(c), &ph) else { continue };
                if let Err(expect) = check_field(kind, &v).and_then(|()| check_param(&d, &v)) {
                    let cid = c.frontmatter.id.clone().unwrap_or_else(|| c.qualified_name.clone());
                    out.push((
                        true,
                        "E247",
                        e.file_path.clone(),
                        format!("field '{key}': placeholder '{}' is '{v}' in configuration {cid}, but the field accepts {expect}", ph.raw),
                    ));
                }
            }
        }
        let mut seen: Vec<String> = Vec::new();
        for ph in &phs {
            if seen.contains(&ph.raw) {
                continue;
            }
            seen.push(ph.raw.clone());
            let d = match decl(elements, resolver, &ph.feature, &ph.param) {
                Ok(d) => d,
                Err(m) => {
                    out.push((true, "E241", e.file_path.clone(), format!("placeholder '{}': {m}", ph.raw)));
                    continue;
                }
            };
            if d.entry.get("bindingTime").and_then(|v| v.as_str()) == Some("runtime") {
                out.push((
                    false,
                    "W246",
                    e.file_path.clone(),
                    format!("placeholder '{}' references a runtime binding-time parameter, which has no value at projection time", ph.raw),
                ));
            }
            let mut escapes: Vec<String> = Vec::new();
            let mut unbound: Vec<String> = Vec::new();
            for (c, sel) in configs.iter().zip(&sels) {
                if !crate::projection::is_active_canon(e, sel, &pkg, &alias) {
                    continue;
                }
                let cid = c.frontmatter.id.clone().unwrap_or_else(|| c.qualified_name.clone());
                if sel.get(&d.feature_qname).copied() != Some(true) {
                    escapes.push(cid);
                } else if d.entry.get("derivedFrom").is_none()
                    && d.entry.get("bindTo").is_none()
                    && value_for(elements, resolver, Some(c), ph).is_none()
                {
                    unbound.push(cid);
                }
            }
            if !escapes.is_empty() {
                out.push((
                    true,
                    "E242",
                    e.file_path.clone(),
                    format!(
                        "placeholder '{}': the element is active in configuration(s) that do not select '{}': {} — gate it with appliesWhen",
                        ph.raw,
                        d.feature_qname,
                        escapes.join(", ")
                    ),
                ));
            }
            if !unbound.is_empty() {
                out.push((
                    firm,
                    if firm { "E243" } else { "W245" },
                    e.file_path.clone(),
                    format!(
                        "placeholder '{}' has no binding, fixed value or default in configuration(s): {}",
                        ph.raw,
                        unbound.join(", ")
                    ),
                ));
            }
        }
    }
    out
}

/// For the feature `feature_qname`: parameter name -> qualified names of the elements whose
/// body or name references it (`{{feature.param}}`), sorted.
pub fn consumers(elements: &[RawElement], resolver: &Resolver, feature_qname: &str) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut out: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    for e in elements {
        for ph in of_element(e) {
            let q = resolver.resolve_ref(elements, &ph.feature).map(|f| f.qualified_name.as_str()).unwrap_or(&ph.feature);
            // Only declared parameters; a mistyped one is E241, not a consumer.
            if q == feature_qname && decl(elements, resolver, feature_qname, &ph.param).is_ok() {
                let v = out.entry(ph.param.clone()).or_default();
                if !v.contains(&e.qualified_name) {
                    v.push(e.qualified_name.clone());
                }
            }
        }
    }
    for v in out.values_mut() {
        v.sort();
    }
    out
}

/// The value parameter `param` of `feature_qname` takes in configuration `cfg` (its binding,
/// else the fixed `value:`, else the `default:`), `None` when unbound.
pub fn value_in_config(elements: &[RawElement], resolver: &Resolver, cfg: &RawElement, feature_qname: &str, param: &str) -> Option<String> {
    let ph = Placeholder { raw: String::new(), feature: feature_qname.to_string(), param: param.to_string(), unit: false };
    value_for(elements, resolver, Some(cfg), &ph)
}

/// Whether the parameter takes its value from another parameter (`derivedFrom:`/`bindTo:`)
/// rather than from a binding, so "unbound" would be the wrong thing to report.
pub fn is_derived(elements: &[RawElement], resolver: &Resolver, feature_qname: &str, param: &str) -> bool {
    decl(elements, resolver, feature_qname, param)
        .map(|d| d.entry.get("derivedFrom").is_some() || d.entry.get("bindTo").is_some())
        .unwrap_or(false)
}

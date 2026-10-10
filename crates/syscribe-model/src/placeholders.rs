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
    let mut fence_start: Option<usize> = None;
    for line in text.split_inclusive('\n') {
        let end = pos + line.len();
        if line.trim_start().starts_with("```") {
            match fence_start.take() {
                Some(s) => out.push((s, end)),
                None => fence_start = Some(pos),
            }
        } else if fence_start.is_none() {
            // Inline spans on this line.
            let mut open: Option<usize> = None;
            for (i, ch) in line.char_indices() {
                if ch == '`' {
                    match open.take() {
                        Some(o) => out.push((pos + o, pos + i + 1)),
                        None => open = Some(i),
                    }
                }
            }
        }
        pos = end;
    }
    if let Some(s) = fence_start {
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
        .map(|c| Placeholder {
            raw: c[0].to_string(),
            feature: c[1].to_string(),
            param: c[2].to_string(),
            unit: c.get(3).is_some(),
        })
        .collect()
}

fn texts(e: &RawElement) -> [&str; 2] {
    [e.doc.as_str(), e.frontmatter.name.as_deref().unwrap_or("")]
}

/// Placeholders in an element's body and name.
pub fn of_element(e: &RawElement) -> Vec<Placeholder> {
    texts(e).iter().flat_map(|t| find(t)).collect()
}

/// Whether any element uses a placeholder (cheap pre-check).
pub fn any(elements: &[RawElement]) -> bool {
    elements.iter().any(|e| texts(e).iter().any(|t| t.contains("{{") && !find(t).is_empty()))
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
            for c in &configs {
                let sel = crate::projection::canonical_selection(elements, c);
                if !crate::projection::is_active_canon(e, &sel, &pkg, &alias) {
                    continue;
                }
                let cid = c.frontmatter.id.clone().unwrap_or_else(|| c.qualified_name.clone());
                if sel.get(&d.feature_qname).copied() != Some(true) {
                    escapes.push(cid);
                } else if value_for(elements, resolver, Some(c), ph).is_none() {
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

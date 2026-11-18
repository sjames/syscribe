//! The minimum syscribe version a model needs (GH #496, `REQ-TRS-MINVER-001`).
//!
//! `[syscribe] minVersion = "0.61.0"` (or `">=0.61.0"`) in `<model_root>/.syscribe.toml` declares the
//! oldest syscribe that understands the model. A syscribe older than that refuses to load it
//! ([`check`]): an older one would ignore the fields it does not know, so `validate` would pass and
//! `build-config` would silently export fewer variables. `validate` warns (`W329`) when the model
//! uses a feature newer than the declared minimum ([`findings`]), so the minimum is raised when the
//! feature starts being used.
//!
//! Versions older than the one that introduced the key cannot know it; the refusal protects every
//! raise from then on.

use std::path::Path;

use crate::element::{ElementType, RawElement};
use crate::validator::{Finding, Severity};

/// A declared minimum, as written and as numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinVersion {
    pub declared: String,
    pub min: [u64; 3],
}

/// `1.2.3`, `1.2` or `v1.2.3` (a pre-release or build suffix is ignored) as numbers.
pub fn parse_version(text: &str) -> Option<[u64; 3]> {
    let t = text.trim().trim_start_matches('v');
    let core = t.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let mut out = [0u64; 3];
    for slot in out.iter_mut() {
        match parts.next() {
            Some(p) => *slot = p.parse().ok()?,
            None => break,
        }
    }
    if parts.next().is_some() || core.is_empty() {
        return None;
    }
    Some(out)
}

fn show(v: [u64; 3]) -> String {
    format!("{}.{}.{}", v[0], v[1], v[2])
}

/// The `[syscribe] minVersion` of `<model_root>/.syscribe.toml`: `Ok(None)` when none is declared, an error
/// naming the file when the value is neither a version nor `>=<version>`.
pub fn load(model_root: &Path) -> Result<Option<MinVersion>, String> {
    let path = model_root.join(".syscribe.toml");
    let Ok(text) = std::fs::read_to_string(&path) else { return Ok(None) };
    let Ok(doc) = text.parse::<toml::Table>() else { return Ok(None) };
    let Some(table) = doc.get("syscribe").and_then(|v| v.as_table()) else { return Ok(None) };
    let Some(value) = table.get("minVersion").or_else(|| table.get("min_version")) else { return Ok(None) };
    let declared = value.as_str().map(str::to_string);
    let parsed = declared.as_deref().and_then(|d| parse_version(d.trim().strip_prefix(">=").unwrap_or(d.trim())));
    match (declared, parsed) {
        (Some(declared), Some(min)) => Ok(Some(MinVersion { declared, min })),
        _ => Err(format!(
            "{}: [syscribe] minVersion must be a version such as \"0.61.0\" or a requirement \">=0.61.0\"",
            path.display()
        )),
    }
}

/// Refuse a model that needs a newer syscribe than `running`: the message names both versions.
pub fn check(model_root: &Path, running: &str) -> Result<Option<MinVersion>, String> {
    let declared = load(model_root)?;
    let Some(m) = &declared else { return Ok(None) };
    let Some(now) = parse_version(running) else { return Ok(declared) };
    if now < m.min {
        return Err(format!(
            "this model requires syscribe {} but this is syscribe {running} (minVersion in {}); upgrade syscribe",
            m.declared,
            model_root.join(".syscribe.toml").display()
        ));
    }
    Ok(declared)
}

/// A feature that needs a newer syscribe than the oldest that existed when it was added.
struct Feature {
    since: [u64; 3],
    what: &'static str,
    uses: fn(&RawElement) -> bool,
}

const FEATURES: &[Feature] = &[
    Feature {
        since: [0, 61, 0],
        what: "a parameter with a `derivedFrom:` expression (computed and exported by 0.61.0; older versions ignore it)",
        uses: |e| {
            e.frontmatter.element_type == Some(ElementType::FeatureDef)
                && e.frontmatter.parameters.iter().flatten().any(|p| p.get("derivedFrom").and_then(|v| v.as_str()).is_some())
        },
    },
    Feature {
        since: [0, 61, 0],
        what: "`buildVar:` on a `CalculationDef` (exported by 0.61.0; older versions ignore it)",
        uses: |e| e.frontmatter.element_type == Some(ElementType::CalculationDef) && e.frontmatter.build_var.is_some(),
    },
    Feature {
        since: [0, 59, 0],
        what: "`amendments:` (0.59.0)",
        uses: |e| e.frontmatter.amendments.is_some(),
    },
];

/// `W329` — an element uses a feature newer than the declared `minVersion`. Nothing when no minimum is
/// declared (the key is opt-in).
pub fn findings(elements: &[RawElement], model_root: &Path) -> Vec<Finding> {
    let Ok(Some(m)) = load(model_root) else { return Vec::new() };
    let mut out = Vec::new();
    for e in elements {
        for f in FEATURES.iter().filter(|f| f.since > m.min) {
            if (f.uses)(e) {
                out.push(Finding {
                    code: "W329",
                    file: e.file_path.clone(),
                    message: format!(
                        "'{}' uses {} but .syscribe.toml declares minVersion {} — raise it to at least {}",
                        e.qualified_name,
                        f.what,
                        m.declared,
                        show(f.since)
                    ),
                    severity: Severity::Warning,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_parse_and_order() {
        assert_eq!(parse_version("0.61.0"), Some([0, 61, 0]));
        assert_eq!(parse_version("v1.2"), Some([1, 2, 0]));
        assert_eq!(parse_version("1.2.3-rc.1"), Some([1, 2, 3]));
        assert_eq!(parse_version("1.2.3.4"), None);
        assert_eq!(parse_version("x"), None);
        assert!(parse_version("0.60.9") < parse_version("0.61.0"));
        assert!(parse_version("0.100.0") > parse_version("0.61.0"));
    }
}

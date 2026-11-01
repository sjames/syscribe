//! Late-bound variability that reaches a safety requirement (`ADR-SYS-BT-001`,
//! `REQ-TRS-BT-034`).
//!
//! Testing one projected variant does not verify a feature or parameter that is only fixed
//! at `load` or `runtime`: the requirement must hold across everything it can still become.
//! `W753` (opt-in, draft-suppressed, gate with `--deny W753`) flags a requirement that
//! carries a SIL/ASIL/CAL level and is influenced by such an item — through its
//! `appliesWhen:` or a `{{Feature.param}}` placeholder in its text — unless a verifying
//! `TestCase` declares, in `coversVariability:`, that it exercises the item's declared range
//! or values. A `coversVariability:` entry that names nothing is `W754`.
//!
//! "Late" means the binding window extends to `load` or later. A model that declares no
//! binding windows has nothing late, so is unaffected.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde_yaml::Value;

use crate::binding_time::{BindingTime, BindingWindow};
use crate::element::{ElementType, RawElement};
use crate::validator::{Finding, Severity};
use crate::variability;

fn warn(code: &'static str, file: &str, message: String) -> Finding {
    Finding { code, file: file.to_string(), message, severity: Severity::Warning }
}

/// The safety level a requirement carries, for messages; `None` when it carries none.
fn level_of(e: &RawElement) -> Option<String> {
    let fm = &e.frontmatter;
    if let Some(a) = fm.asil_level.as_deref().filter(|a| !a.is_empty() && *a != "QM") {
        return Some(format!("ASIL {a}"));
    }
    if let Some(s) = fm.sil_level {
        return Some(format!("SIL {s}"));
    }
    fm.cal_level.as_deref().filter(|c| !c.is_empty()).map(str::to_string)
}

/// Canonical key of a `coversVariability:` entry: a feature (`FEAT-*` id or qualified name)
/// or a `<feature>.<param>` parameter.
fn canon_entry(entry: &str, alias: &HashMap<String, String>) -> String {
    match entry.rsplit_once('.') {
        Some((f, p)) => format!("{}.{}", variability::canon_feature_ref(f, alias), p),
        None => variability::canon_feature_ref(entry, alias),
    }
}

pub fn findings(elements: &[RawElement]) -> Vec<Finding> {
    let mut out = Vec::new();
    let Some(load) = BindingTime::parse("load") else { return out };
    let alias = variability::feature_id_to_qname(elements);
    let pkg = variability::package_conditions(elements);

    // Every feature and parameter that declares a window, keyed as a coverage entry names it.
    let mut windows: BTreeMap<String, (BindingWindow, &'static str)> = BTreeMap::new();
    for fd in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef))) {
        let fm = &fd.frontmatter;
        if let Some(w) = BindingWindow::read(fm.binding_time.as_deref(), fm.latest_binding_time.as_deref()).0 {
            windows.insert(fd.qualified_name.clone(), (w, "feature"));
        }
        for p in fm.parameters.iter().flatten() {
            let Value::Mapping(m) = p else { continue };
            let get = |k: &str| m.get(Value::String(k.to_string())).and_then(|v| v.as_str());
            let Some(name) = get("name") else { continue };
            if let Some(w) = BindingWindow::read(get("bindingTime"), get("latestBindingTime")).0 {
                windows.insert(format!("{}.{}", fd.qualified_name, name), (w, "parameter"));
            }
        }
    }
    if windows.is_empty() {
        return out;
    }
    let all_keys: HashSet<String> = windows.keys().cloned().collect();
    let feature_names: HashSet<&str> = elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef)))
        .map(|e| e.qualified_name.as_str())
        .collect();

    // What each non-draft TestCase claims to cover, by the requirement it verifies.
    let mut covers: HashMap<String, HashSet<String>> = HashMap::new();
    for tc in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::TestCase))) {
        let entries: HashSet<String> =
            tc.frontmatter.covers_variability.iter().flatten().map(|e| canon_entry(e, &alias)).collect();
        for raw in tc.frontmatter.covers_variability.iter().flatten() {
            let k = canon_entry(raw, &alias);
            if !all_keys.contains(&k) && !feature_names.contains(k.as_str()) {
                out.push(warn("W754", &tc.file_path, format!(
                    "coversVariability entry '{raw}' names no feature or parameter of the feature model")));
            }
        }
        if matches!(tc.frontmatter.status.as_deref(), Some("draft") | Some("retired") | Some("deprecated")) {
            continue;
        }
        for v in tc.frontmatter.verifies.iter().flatten() {
            covers.entry(v.clone()).or_default().extend(entries.iter().cloned());
        }
    }

    for r in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Requirement))) {
        if r.frontmatter.status.as_deref() == Some("draft") {
            continue;
        }
        let Some(level) = level_of(r) else { continue };
        // The late-bound items that influence this requirement.
        let mut influences: BTreeMap<String, (&'static str, String)> = BTreeMap::new();
        let mut note = |key: &str, via: &str| {
            if let Some((w, kind)) = windows.get(key) {
                if w.latest >= load {
                    influences.entry(key.to_string()).or_insert((*kind, format!("{via}, binding window {}", w.describe())));
                }
            }
        };
        if let Some(x) = variability::effective_expr_canon(r, &pkg, &alias) {
            for q in x.operands() {
                note(&q, "in its appliesWhen");
            }
        }
        for ph in crate::placeholders::of_element(r) {
            note(&format!("{}.{}", variability::canon_feature_ref(&ph.feature, &alias), ph.param), "in its text");
        }
        let have: HashSet<&String> = [r.frontmatter.id.as_ref(), Some(&r.qualified_name)]
            .into_iter()
            .flatten()
            .flat_map(|k| covers.get(k).into_iter().flatten())
            .collect();
        let shown = r.frontmatter.id.as_deref().unwrap_or(&r.qualified_name);
        for (key, (kind, how)) in influences {
            if !have.contains(&key) {
                out.push(warn("W753", &r.file_path, format!(
                    "requirement '{shown}' ({level}) depends on late-bound {kind} '{key}' ({how}); no verifying TestCase declares coversVariability for it — its range or values are not verified",
                )));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_canonicalise_a_feature_id_and_a_dotted_parameter() {
        let alias: HashMap<String, String> = [("FEAT-X".to_string(), "F::X".to_string())].into();
        assert_eq!(canon_entry("FEAT-X", &alias), "F::X");
        assert_eq!(canon_entry("FEAT-X.kv", &alias), "F::X.kv");
        assert_eq!(canon_entry("F::X.kv", &alias), "F::X.kv");
        assert_eq!(canon_entry("F::Y", &alias), "F::Y");
    }
}

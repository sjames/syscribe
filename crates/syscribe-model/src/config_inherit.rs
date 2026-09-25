//! `Configuration` inheritance through `derivedFrom:` (spec §9.8, GH #137).
//!
//! A `Configuration` may name **one** base `Configuration` in `derivedFrom:`.
//! It inherits the base's *effective* `features:` selections and
//! `parameterBindings:` (so inheritance chains compose), and its own entries
//! override the inherited ones:
//!
//! - **features** — the effective selection is the base's effective selection
//!   overlaid by the child's own `features:` map, matched on the canonical
//!   feature key (a `FEAT-*` id and the FeatureDef's qualified name name the
//!   same feature). The result is keyed by qualified name.
//! - **parameterBindings** — every child entry is kept; an inherited entry is
//!   kept unless the child binds the same `<feature>.<param>` itself, or the
//!   child's own `features:` map explicitly deselects (`false`) that feature
//!   (a binding for a feature the child switched off is not carried over).
//! - Nothing else is inherited (`subConfigurations:`, `featureModel:`,
//!   `buildOverrides:`, `status:` … stay per-file).
//!
//! The walker materializes the result ([`apply_configuration_inheritance`]) into
//! a non-serialized slot on the child's frontmatter, so every consumer that
//! reads [`RawFrontmatter::feature_selections`] /
//! [`RawFrontmatter::effective_parameter_bindings`] — projection, `matrix`,
//! `configure`, `validate --config`/`--all-configs`, `feature-check`,
//! `build-config`, `subConfigurations:` peers — sees the inherited selection,
//! while the authored file (and anything that re-serializes it) is untouched.
//!
//! Structural problems are reported by the validator from [`analyse`]:
//! `E234` dangling base, `E235` base is not a `Configuration`, `E236` cycle,
//! `E237` more than one base, and `E215` (§9.11) base not `approved`/`released`.
//! A child with any of E234–E237 inherits nothing.
//!
//! [`RawFrontmatter::feature_selections`]: crate::element::RawFrontmatter::feature_selections
//! [`RawFrontmatter::effective_parameter_bindings`]: crate::element::RawFrontmatter::effective_parameter_bindings

use std::collections::{BTreeMap, HashMap};

use crate::element::{ElementType, InheritedConfiguration, RawElement};
use crate::variability;

/// One structural problem with a `Configuration`'s `derivedFrom:`.
#[derive(Debug, Clone, PartialEq)]
pub enum InheritanceProblem {
    /// `E234` — the base reference resolves to no element.
    Dangling { child: usize, target: String },
    /// `E235` — the base reference resolves to an element that is not a `Configuration`.
    NotConfiguration { child: usize, target: String, found: String },
    /// `E236` — the child is on a `derivedFrom:` cycle (the chain, child first).
    Cycle { child: usize, chain: Vec<String> },
    /// `E237` — more than one base is named.
    MultipleBases { child: usize, count: usize },
    /// `E215` — the (valid) base is not `approved` or `released`.
    UnreleasedBase { child: usize, base: String, status: String },
}

fn is_configuration(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::Configuration))
}

fn display_id(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Resolve a reference by exact qualified name, then by stable id.
fn find(elements: &[RawElement], r: &str) -> Option<usize> {
    let r = r.trim();
    elements
        .iter()
        .position(|e| e.qualified_name == r)
        .or_else(|| elements.iter().position(|e| e.frontmatter.id.as_deref() == Some(r)))
}

/// The inheritance structure of the model: each `Configuration` index with a
/// valid, acyclic single base mapped to that base's index, plus every problem
/// found. Pure — reads only the authored `derivedFrom:` fields.
pub fn analyse(elements: &[RawElement]) -> (HashMap<usize, usize>, Vec<InheritanceProblem>) {
    let mut base_of: HashMap<usize, usize> = HashMap::new();
    let mut problems = Vec::new();
    for (i, e) in elements.iter().enumerate() {
        if !is_configuration(e) {
            continue;
        }
        let refs: Vec<&String> = e
            .frontmatter
            .derived_from
            .iter()
            .flatten()
            .filter(|s| !s.trim().is_empty())
            .collect();
        match refs.len() {
            0 => continue,
            1 => {}
            n => {
                problems.push(InheritanceProblem::MultipleBases { child: i, count: n });
                continue;
            }
        }
        let target = refs[0];
        match find(elements, target) {
            None => problems.push(InheritanceProblem::Dangling { child: i, target: target.clone() }),
            Some(b) if !is_configuration(&elements[b]) => {
                let found = elements[b]
                    .frontmatter
                    .element_type
                    .as_ref()
                    .map(|t| format!("{t:?}"))
                    .unwrap_or_else(|| "untyped element".into());
                problems.push(InheritanceProblem::NotConfiguration { child: i, target: target.clone(), found });
            }
            Some(b) => {
                base_of.insert(i, b);
            }
        }
    }
    // Cycle detection: follow each chain; any node that revisits itself is on a
    // cycle. Cycle members inherit nothing.
    let mut on_cycle: Vec<usize> = Vec::new();
    for &start in base_of.keys() {
        let mut chain = vec![start];
        let mut cur = start;
        while let Some(&next) = base_of.get(&cur) {
            if next == start {
                on_cycle.push(start);
                let names = chain.iter().map(|&k| display_id(&elements[k])).collect::<Vec<_>>();
                problems.push(InheritanceProblem::Cycle { child: start, chain: names });
                break;
            }
            if chain.contains(&next) {
                break; // leads into a cycle it is not part of
            }
            chain.push(next);
            cur = next;
        }
    }
    for c in &on_cycle {
        base_of.remove(c);
    }
    // A child whose chain still leads into a cycle has no well-defined
    // inheritance either: drop it (the cycle members carry the E236).
    let mut leads_into_cycle = Vec::new();
    for &start in base_of.keys() {
        let mut cur = start;
        let mut steps = 0usize;
        while let Some(&next) = base_of.get(&cur) {
            cur = next;
            steps += 1;
            if steps > elements.len() {
                leads_into_cycle.push(start);
                break;
            }
        }
        if on_cycle.contains(&cur) {
            leads_into_cycle.push(start);
        }
    }
    for c in &leads_into_cycle {
        base_of.remove(c);
    }
    // E215: a valid base must be approved or released.
    for (&child, &base) in &base_of {
        let status = elements[base].frontmatter.status.as_deref().unwrap_or("");
        if !matches!(status, "approved" | "released") {
            problems.push(InheritanceProblem::UnreleasedBase {
                child,
                base: display_id(&elements[base]),
                status: if status.is_empty() { "(none)".into() } else { status.to_string() },
            });
        }
    }
    problems.sort_by_key(|p| match p {
        InheritanceProblem::Dangling { child, .. }
        | InheritanceProblem::NotConfiguration { child, .. }
        | InheritanceProblem::Cycle { child, .. }
        | InheritanceProblem::MultipleBases { child, .. }
        | InheritanceProblem::UnreleasedBase { child, .. } => *child,
    });
    (base_of, problems)
}

/// The feature part of a `<feature>.<param>` binding key, canonicalized.
fn binding_feature(key: &str, alias: &HashMap<String, String>) -> String {
    match key.rsplit_once('.') {
        Some((f, _)) => variability::canon_feature_ref(f, alias),
        None => key.to_string(),
    }
}

fn canon_binding_key(key: &str, alias: &HashMap<String, String>) -> String {
    match key.rsplit_once('.') {
        Some((f, p)) => format!("{}.{}", variability::canon_feature_ref(f, alias), p),
        None => key.to_string(),
    }
}

/// Compute (memoized) the effective selection and bindings of element `i`.
fn effective(
    i: usize,
    elements: &[RawElement],
    base_of: &HashMap<usize, usize>,
    alias: &HashMap<String, String>,
    memo: &mut HashMap<usize, InheritedConfiguration>,
) -> InheritedConfiguration {
    if let Some(done) = memo.get(&i) {
        return done.clone();
    }
    let fm = &elements[i].frontmatter;
    let own_sel = variability::canon_selection(&fm.declared_feature_selections(), alias);
    let own_bind = fm.parameter_bindings.clone();
    let result = match base_of.get(&i) {
        None => InheritedConfiguration { features: own_sel, parameter_bindings: own_bind },
        Some(&b) => {
            let base = effective(b, elements, base_of, alias, memo);
            // features: base overlaid by own.
            let mut features: BTreeMap<String, bool> = base.features.clone();
            for (k, v) in &own_sel {
                features.insert(k.clone(), *v);
            }
            // parameterBindings: own entries + non-overridden inherited ones for
            // features the child did not explicitly deselect.
            let own_map = match &own_bind {
                Some(serde_yaml::Value::Mapping(m)) => m.clone(),
                _ => serde_yaml::Mapping::new(),
            };
            let own_keys: std::collections::HashSet<String> = own_map
                .keys()
                .filter_map(|k| k.as_str())
                .map(|k| canon_binding_key(k, alias))
                .collect();
            let mut merged = serde_yaml::Mapping::new();
            if let Some(serde_yaml::Value::Mapping(bm)) = &base.parameter_bindings {
                for (k, v) in bm {
                    let Some(ks) = k.as_str() else { continue };
                    if own_keys.contains(&canon_binding_key(ks, alias)) {
                        continue;
                    }
                    if own_sel.get(&binding_feature(ks, alias)) == Some(&false) {
                        continue;
                    }
                    merged.insert(k.clone(), v.clone());
                }
            }
            for (k, v) in own_map {
                merged.insert(k, v);
            }
            let parameter_bindings = if merged.is_empty() && own_bind.is_none() && base.parameter_bindings.is_none() {
                None
            } else {
                Some(serde_yaml::Value::Mapping(merged))
            };
            InheritedConfiguration { features, parameter_bindings }
        }
    };
    memo.insert(i, result.clone());
    result
}

/// Materialize inheritance: every `Configuration` with a valid base gets its
/// effective selection/bindings stored in the non-serialized
/// `frontmatter.inherited` slot. Configurations without a (valid) base are left
/// exactly as authored. Called by the walker after all elements load.
///
/// Idempotent over the **full** element set: any previously materialized slot
/// is cleared first, so it is safe — and required — to re-run after a
/// consumer rebuilds some elements' frontmatter from YAML (the slot is
/// `#[serde(skip)]`, so a re-parsed `Configuration` silently loses its
/// inherited selection) or edits a `derivedFrom:` base. The LSP's rename
/// candidate model is one such consumer (GH #146, `REQ-TRS-VAR-007`). A
/// single `Configuration` must never be re-derived on its own: its effective
/// selection depends on its base chain, so only the full set is meaningful.
pub fn apply_configuration_inheritance(elements: &mut [RawElement]) {
    for e in elements.iter_mut() {
        e.frontmatter.inherited = None;
    }
    let (base_of, _) = analyse(elements);
    if base_of.is_empty() {
        return;
    }
    let alias = variability::feature_id_to_qname(elements);
    let mut memo = HashMap::new();
    let mut out: Vec<(usize, InheritedConfiguration)> = Vec::new();
    for &child in base_of.keys() {
        out.push((child, effective(child, elements, &base_of, &alias, &mut memo)));
    }
    for (i, eff) in out {
        elements[i].frontmatter.inherited = Some(Box::new(eff));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::RawFrontmatter;

    fn conf(id: &str, status: &str, base: Option<&str>, feats: &str, binds: &str) -> RawElement {
        let yaml = format!(
            "type: Configuration\nid: {id}\nname: x\nstatus: {status}\nfeatureModel: F\n{}features:\n{feats}\n{binds}",
            base.map(|b| format!("derivedFrom: {b}\n")).unwrap_or_default()
        );
        let fm: RawFrontmatter = serde_yaml::from_str(&yaml).expect("yaml");
        RawElement {
            qualified_name: format!("C::{id}"),
            file_path: format!("{id}.md"),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: Vec::new(),
            locale_docs: Default::default(),
            about_notes: Default::default(),
        }
    }

    #[test]
    fn child_overrides_and_inherits() {
        let mut els = vec![
            conf("CONF-A-001", "approved", None, "  F::X: true\n  F::Y: true", "parameterBindings:\n  F::X.p: 1\n  F::Y.q: 2\n"),
            conf("CONF-B-001", "draft", Some("CONF-A-001"), "  F::Y: false", "parameterBindings:\n  F::X.p: 5\n"),
        ];
        apply_configuration_inheritance(&mut els);
        let sel = els[1].frontmatter.feature_selections();
        assert_eq!(sel.get("F::X"), Some(&true));
        assert_eq!(sel.get("F::Y"), Some(&false));
        let Some(serde_yaml::Value::Mapping(b)) = els[1].frontmatter.effective_parameter_bindings() else {
            panic!("bindings")
        };
        assert_eq!(b.get("F::X.p").and_then(|v| v.as_i64()), Some(5));
        assert!(b.get("F::Y.q").is_none(), "binding for a deselected feature is dropped");
        // The base is untouched.
        assert!(els[0].frontmatter.inherited.is_none());
    }

    #[test]
    fn reapplying_after_a_reparse_restores_the_inherited_selection() {
        // GH #146: the `inherited` slot is `#[serde(skip)]`, so a consumer that
        // re-deserializes a child's frontmatter (the LSP rename candidate) loses
        // it; re-running over the full set must restore it, and re-running must
        // be idempotent (a stale slot is cleared, not kept).
        let mut els = vec![
            conf("CONF-A-001", "approved", None, "  F::X: true", "parameterBindings:\n  F::X.p: 1\n"),
            conf("CONF-B-001", "draft", Some("CONF-A-001"), "  F::Y: true", ""),
        ];
        apply_configuration_inheritance(&mut els);
        let reparsed: RawFrontmatter =
            serde_yaml::from_value(serde_yaml::to_value(&els[1].frontmatter).expect("ser")).expect("de");
        els[1].frontmatter = reparsed;
        assert!(els[1].frontmatter.inherited.is_none(), "a re-parse drops the slot");
        apply_configuration_inheritance(&mut els);
        assert_eq!(els[1].frontmatter.feature_selections().get("F::X"), Some(&true));
        // Dropping the base link and re-applying clears the now-stale slot.
        els[1].frontmatter.derived_from = None;
        apply_configuration_inheritance(&mut els);
        assert!(els[1].frontmatter.inherited.is_none());
        assert_eq!(els[1].frontmatter.feature_selections().get("F::X"), None);
    }

    #[test]
    fn problems_are_classified() {
        let els = vec![
            conf("CONF-A-001", "draft", Some("CONF-B-001"), "  F::X: true", ""),
            conf("CONF-B-001", "draft", Some("CONF-A-001"), "  F::X: true", ""),
            conf("CONF-C-001", "draft", Some("CONF-NOPE-001"), "  F::X: true", ""),
            conf("CONF-D-001", "draft", Some("CONF-E-001"), "  F::X: true", ""),
            conf("CONF-E-001", "draft", None, "  F::X: true", ""),
        ];
        let (base_of, problems) = analyse(&els);
        assert_eq!(base_of.get(&3), Some(&4));
        assert!(problems.iter().any(|p| matches!(p, InheritanceProblem::Cycle { child: 0, .. })));
        assert!(problems.iter().any(|p| matches!(p, InheritanceProblem::Cycle { child: 1, .. })));
        assert!(problems.iter().any(|p| matches!(p, InheritanceProblem::Dangling { child: 2, .. })));
        assert!(problems.iter().any(|p| matches!(p, InheritanceProblem::UnreleasedBase { child: 3, .. })));
    }
}

//! Variant retention (`ADR-SYS-BT-001`, `REQ-TRS-BT-031..033`).
//!
//! Projection removes an element when its `appliesWhen:` is false for a Configuration. That
//! is only sound when the features it names are *decided*. A staged Configuration leaves
//! later-bound features open: an element whose condition they decide cannot be dropped yet,
//! because a later stage may switch it on. Such an element is **retained** and marked
//! variant-retained ([`crate::element::RawFrontmatter::variant_retained`]); one whose
//! condition is false whatever the open features become is removed, as before.
//!
//! [`state`] is the three-valued gate [`crate::projection::project_raw`] uses, and
//! [`report`] lists what was retained for a Configuration — the same list for
//! `validate --config` and `residual` — plus the elements that *rely on the absence* of a
//! retained one (`W752`): a dependent that references it yet applies only where it does not.

use std::collections::{HashMap, HashSet};

use crate::element::{ElementType, RawElement};
use crate::projection::{canonical_selection, outbound_refs, Selection};
use crate::resolver::Resolver;
use crate::variability::{self, FeatureExpr};

/// `Some(true)` active, `Some(false)` removed, `None` retained: the element's effective
/// `appliesWhen:` depends on a feature in `open`. With an empty `open` this is exactly
/// [`crate::projection::is_active_canon`] (never `None`).
pub fn state(
    elem: &RawElement,
    sel: &Selection,
    pkg: &HashMap<String, serde_yaml::Value>,
    alias: &HashMap<String, String>,
    open: &HashSet<String>,
) -> Option<bool> {
    let Some((aw, _)) = variability::effective_applies_when(elem, pkg) else { return Some(true) };
    match variability::applies_when_expr(&aw) {
        Ok(Some(expr)) => {
            let expr = expr.canonicalize(&|q: &str| variability::canon_feature_ref(q, alias));
            expr.eval_tri(&|q: &str| if open.contains(q) { None } else { Some(sel.get(q).copied().unwrap_or(false)) })
        }
        _ => Some(true), // malformed (E209 elsewhere): active, as in projection
    }
}

/// An element a staged projection keeps although its condition is not yet decided.
#[derive(Debug, Clone, PartialEq)]
pub struct Retained {
    pub qname: String,
    pub id: Option<String>,
    pub kind: String,
    /// The open features its `appliesWhen:` mentions.
    pub because: Vec<String>,
}

/// An element that references a retained one yet applies only where that one does not.
#[derive(Debug, Clone, PartialEq)]
pub struct Reliance {
    pub dependent: String,
    pub retained: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RetentionReport {
    pub retained: Vec<Retained>,
    pub reliances: Vec<Reliance>,
}

/// `true` when no assignment of the features in `a` and `b` satisfies both — one applies
/// only where the other does not. Skipped (`false`) beyond 16 distinct features.
fn mutually_exclusive(a: &FeatureExpr, b: &FeatureExpr) -> bool {
    let mut feats: Vec<String> = a.operands().into_iter().chain(b.operands()).collect();
    feats.sort();
    feats.dedup();
    if feats.len() > 16 {
        return false;
    }
    (0u32..(1u32 << feats.len())).all(|bits| {
        let on = |q: &str| feats.iter().position(|f| f == q).is_some_and(|i| bits >> i & 1 == 1);
        !(a.eval(&on) && b.eval(&on))
    })
}

/// What a staged projection of `elements[cfg_idx]` retains, and what relies on the absence
/// of any of it. Empty for an unstaged Configuration.
pub fn report(elements: &[RawElement], cfg_idx: usize) -> RetentionReport {
    let open = crate::staging::open_features(elements, cfg_idx);
    if open.is_empty() {
        return RetentionReport::default();
    }
    let sel = canonical_selection(elements, &elements[cfg_idx]);
    let pkg = variability::package_conditions(elements);
    let alias = variability::feature_id_to_qname(elements);
    let resolver = Resolver::new(elements);

    let expr_of = |e: &RawElement| variability::effective_expr_canon(e, &pkg, &alias);
    let mut retained: Vec<(usize, Retained)> = Vec::new();
    for (i, e) in elements.iter().enumerate() {
        if matches!(e.frontmatter.element_type, Some(ElementType::Configuration)) {
            continue;
        }
        if state(e, &sel, &pkg, &alias, &open).is_none() {
            let mut because: Vec<String> =
                expr_of(e).map(|x| x.operands()).unwrap_or_default().into_iter().filter(|q| open.contains(q)).collect();
            because.sort();
            because.dedup();
            retained.push((
                i,
                Retained {
                    qname: e.qualified_name.clone(),
                    id: e.frontmatter.id.clone(),
                    kind: e.frontmatter.element_type.as_ref().map(|t| format!("{t:?}")).unwrap_or_default(),
                    because,
                },
            ));
        }
    }
    retained.sort_by(|a, b| a.1.qname.cmp(&b.1.qname));

    let retained_idx: HashSet<usize> = retained.iter().map(|(i, _)| *i).collect();
    let mut reliances: Vec<Reliance> = Vec::new();
    for (di, d) in elements.iter().enumerate() {
        if state(d, &sel, &pkg, &alias, &open) == Some(false) {
            continue; // not in the projection
        }
        let Some(dx) = expr_of(d) else { continue };
        for (_, target) in outbound_refs(d) {
            let Some(t) = resolver.resolve_ref(elements, &target) else { continue };
            let Some(ti) = elements.iter().position(|e| std::ptr::eq(e, t)) else { continue };
            if ti == di || !retained_idx.contains(&ti) {
                continue;
            }
            if expr_of(t).is_some_and(|tx| mutually_exclusive(&dx, &tx)) {
                reliances.push(Reliance { dependent: d.qualified_name.clone(), retained: t.qualified_name.clone() });
            }
        }
    }
    reliances.sort_by(|a, b| (&a.dependent, &a.retained).cmp(&(&b.dependent, &b.retained)));
    reliances.dedup();
    RetentionReport { retained: retained.into_iter().map(|(_, r)| r).collect(), reliances }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variability::FeatureExpr::{And, Feat, Not, Or};

    fn f(q: &str) -> Box<FeatureExpr> {
        Box::new(Feat(q.into()))
    }

    #[test]
    fn kleene_logic_keeps_what_the_decided_part_already_settles() {
        let known = |q: &str| match q {
            "on" => Some(true),
            "off" => Some(false),
            _ => None,
        };
        assert_eq!(Feat("open".into()).eval_tri(&known), None);
        assert_eq!(And(f("off"), f("open")).eval_tri(&known), Some(false), "false and ? is false");
        assert_eq!(And(f("on"), f("open")).eval_tri(&known), None);
        assert_eq!(Or(f("on"), f("open")).eval_tri(&known), Some(true), "true or ? is true");
        assert_eq!(Or(f("off"), f("open")).eval_tri(&known), None);
        assert_eq!(Not(f("open")).eval_tri(&known), None);
        assert_eq!(Not(f("off")).eval_tri(&known), Some(true));
    }

    #[test]
    fn exclusivity_is_decided_over_every_assignment() {
        let a = Feat("F".into());
        let not_a = Not(f("F"));
        assert!(mutually_exclusive(&a, &not_a));
        assert!(!mutually_exclusive(&a, &Feat("G".into())));
        assert!(mutually_exclusive(&And(f("F"), f("G")), &Not(f("F"))));
        assert!(!mutually_exclusive(&Or(f("F"), f("G")), &Not(f("F"))), "G alone satisfies both");
    }
}

//! TestPlan membership and configuration-binding helpers (GH #38;
//! REQ-TRS-PLAN-002 / REQ-TRS-PLAN-003).
//!
//! A TestPlan (`type: TestPlan`, stable `TP-*` id) is a curated aggregation of
//! TestCases offered as a unit of verification evidence. This module computes the
//! two derived quantities both the validator and the (future) `testplan` command
//! need:
//!
//! * the **effective TestCase set** = explicit `testCases:` ∪ `selection:` matches
//!   (deduped by id); and
//! * the **bound Configurations** (`plan_configs`).
//!
//! Membership stays *computed, not stored* at the TestCase level — the plan only
//! declares which products it is a plan for, never stamping config membership onto
//! its members.

use std::collections::HashSet;

use crate::element::{ElementType, RawElement, TestPlanSelection};
use crate::projection::{self, Selection};
use crate::resolver::Resolver;

/// A reference key for deduping TestCases. Prefers the stable `id:`, falling back
/// to the qualified name when a (malformed) TestCase carries no id.
fn tc_key(tc: &RawElement) -> String {
    tc.frontmatter
        .id
        .clone()
        .unwrap_or_else(|| tc.qualified_name.clone())
}

/// True when `elem` is a `TestCase` element (by type — the validator separately
/// flags a bad id).
fn is_testcase(elem: &RawElement) -> bool {
    matches!(elem.frontmatter.element_type, Some(ElementType::TestCase))
}

/// The `reqDomain:` values reachable from a TestCase's `verifies:` targets.
/// Used to evaluate a `selection.domains` constraint transitively.
fn testcase_domains(
    tc: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> HashSet<String> {
    let mut out = HashSet::new();
    if let Some(verifies) = &tc.frontmatter.verifies {
        for v in verifies {
            if let Some(target) = resolver.resolve_ref(elements, v) {
                if let Some(d) = &target.frontmatter.req_domain {
                    out.insert(d.clone());
                }
            }
        }
    }
    out
}

/// Does a (non-draft) TestCase match the additive `selection:` query?
///
/// An *absent* sub-field is no constraint; a `selection` block with no sub-fields
/// at all matches nothing. All present sub-fields must hold (AND). Draft
/// TestCases are never swept by selection — only explicit naming pulls them in.
fn selection_matches(
    tc: &RawElement,
    sel: &TestPlanSelection,
    elements: &[RawElement],
    resolver: &Resolver,
) -> bool {
    if sel.is_empty() {
        return false;
    }
    // Draft TestCases are not swept by selection.
    if tc.frontmatter.status.as_deref() == Some("draft") {
        return false;
    }

    if let Some(levels) = &sel.test_levels {
        match &tc.frontmatter.test_level {
            Some(lvl) if levels.iter().any(|l| l == lvl) => {}
            _ => return false,
        }
    }

    if let Some(domains) = &sel.domains {
        let tc_domains = testcase_domains(tc, elements, resolver);
        if !domains.iter().any(|d| tc_domains.contains(d)) {
            return false;
        }
    }

    if let Some(want_tags) = &sel.tags {
        let tc_tags = tc.frontmatter.tags.clone().unwrap_or_default();
        if !want_tags.iter().any(|t| tc_tags.contains(t)) {
            return false;
        }
    }

    true
}

/// The plan's **effective TestCase set**: explicit `testCases:` members (resolved
/// to TestCases) ∪ `selection:` matches, deduped by id.
///
/// * `selection` is additive only — it never removes a named member.
/// * An explicitly named draft/retired TestCase is kept (explicit naming is
///   authoritative); a draft TestCase is never pulled in by `selection`.
/// * Unresolvable / non-TestCase explicit entries are skipped here (the validator
///   reports them as `E601`).
pub fn effective_testcases<'a>(
    plan: &RawElement,
    elements: &'a [RawElement],
    resolver: &Resolver,
) -> Vec<&'a RawElement> {
    let mut out: Vec<&'a RawElement> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // Explicit members.
    if let Some(refs) = &plan.frontmatter.test_cases {
        for r in refs {
            if let Some(tc) = resolver.resolve_ref(elements, r) {
                if is_testcase(tc) && seen.insert(tc_key(tc)) {
                    out.push(tc);
                }
            }
        }
    }

    // Additive selection matches.
    if let Some(sel) = &plan.frontmatter.selection {
        for elem in elements {
            if is_testcase(elem)
                && selection_matches(elem, sel, elements, resolver)
                && seen.insert(tc_key(elem))
            {
                out.push(elem);
            }
        }
    }

    out
}

/// The Configurations this plan is a plan for: the listed `configurations:`
/// entries (resolved to `Configuration` elements), or **all** stored
/// Configurations when `configurations:` is absent (config-agnostic plan).
///
/// Unresolvable / non-Configuration listed entries are skipped here (the
/// validator reports them as `E606`).
pub fn plan_configs<'a>(
    plan: &RawElement,
    elements: &'a [RawElement],
    resolver: &Resolver,
) -> Vec<&'a RawElement> {
    match &plan.frontmatter.configurations {
        Some(refs) => {
            let mut out = Vec::new();
            let mut seen = HashSet::new();
            for r in refs {
                if let Some(cfg) = resolver.resolve_ref(elements, r) {
                    if Resolver::is_configuration(cfg) && seen.insert(cfg.qualified_name.clone()) {
                        out.push(cfg);
                    }
                }
            }
            out
        }
        None => elements
            .iter()
            .filter(|e| Resolver::is_configuration(e))
            .collect(),
    }
}

/// Is `tc` active in *at least one* of the plan's bound configurations?
///
/// Reuses the projection engine ([`projection::is_active`]). When the plan binds
/// no resolvable configurations (e.g. a flat model with no feature model, or all
/// listed configs unresolved), the variability dimension is inactive and the
/// member is treated as active (dormant — no escaping check).
pub fn member_active_in_any_config(
    tc: &RawElement,
    configs: &[&RawElement],
    pkg: &std::collections::HashMap<String, serde_yaml::Value>,
    alias: &std::collections::HashMap<String, String>,
) -> bool {
    if configs.is_empty() {
        return true;
    }
    configs.iter().any(|cfg| {
        let sel: Selection = crate::variability::canon_selection(
            &cfg.frontmatter.feature_selections(),
            alias,
        );
        projection::is_active_canon(tc, &sel, pkg, alias)
    })
}

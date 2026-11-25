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

    if let Some(sels) = &sel.stereotypes {
        let sels: Vec<crate::tagged_values::Selector> = sels.iter().filter_map(|v| crate::tagged_values::parse_selector(v).ok()).collect();
        // The TestCase itself, or a requirement it verifies, satisfies a selector.
        let verified: Vec<&RawElement> = tc.frontmatter.verifies.iter().flatten().filter_map(|r| resolver.resolve_ref(elements, r)).collect();
        let hit = sels.iter().any(|s| {
            crate::tagged_values::selector_matches(elements, resolver, tc, s)
                || verified.iter().any(|v| crate::tagged_values::selector_matches(elements, resolver, v, s))
        });
        if !hit {
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

    // Additive selection matches. A test case that runs in none of the plan's bound products is not swept in (GH #314);
    // explicit members stay, so `W611` still reports a named one that escapes.
    if let Some(sel) = &plan.frontmatter.selection {
        let configs = plan_configs(plan, elements, resolver);
        let (pkg, alias) = (crate::variability::package_conditions(elements), crate::variability::feature_id_to_qname(elements));
        for elem in elements {
            if is_testcase(elem)
                && selection_matches(elem, sel, elements, resolver)
                && member_active_in_any_config(elem, &configs, &pkg, &alias)
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

/// One view of the ingested results a plan is judged on, and the Configuration it is for.
pub struct ResultView<'a, 'r> {
    /// The Configuration this view judges; `None` when the plan is judged once, over all of `configs`.
    pub config: Option<&'a RawElement>,
    /// The plan Configurations a member must be active in to be judged in this view.
    pub configs: Vec<&'a RawElement>,
    pub results: std::borrow::Cow<'r, crate::results::ResultsData>,
}

impl ResultView<'_, '_> {
    /// The id of the Configuration this view is for.
    pub fn config_id(&self) -> Option<String> {
        self.config.map(|c| c.frontmatter.id.clone().unwrap_or_else(|| c.qualified_name.clone()))
    }
}

/// The results a plan is judged on (GH #528, `REQ-TRS-PLANCFG-001`). A plan that names its `configurations:`
/// is judged on the evidence of those Configurations only — each on its own effective results
/// ([`ResultsData::for_config`](crate::results::ResultsData::for_config): the global sections overlaid with
/// its own), one view per Configuration, so evidence recorded for a Configuration the plan does not name never
/// reaches it. When none of the plan's Configurations has evidence of its own they all see the same global
/// results and one view stands for them. A plan without `configurations:` keeps the conservative reading over
/// everything recorded (`REQ-TRS-CFGRES-001`).
pub fn result_views<'a, 'r>(
    plan: &RawElement,
    configs: &[&'a RawElement],
    results: &'r crate::results::ResultsData,
) -> Vec<ResultView<'a, 'r>> {
    use std::borrow::Cow;
    let named = plan.frontmatter.configurations.as_ref().is_some_and(|c| !c.is_empty()) && !configs.is_empty();
    if !named || results.by_config.is_empty() {
        return vec![ResultView { config: None, configs: configs.to_vec(), results: Cow::Borrowed(results) }];
    }
    let id = |c: &RawElement| c.frontmatter.id.clone().unwrap_or_else(|| c.qualified_name.clone());
    if !configs.iter().any(|c| results.by_config.contains_key(&id(c))) {
        // Only other Configurations have evidence of their own: the plan's all read the global sections.
        return vec![ResultView { config: None, configs: configs.to_vec(), results: Cow::Owned(results.for_config("")) }];
    }
    configs
        .iter()
        .map(|c| ResultView { config: Some(*c), configs: vec![*c], results: Cow::Owned(results.for_config(&id(c))) })
        .collect()
}

//! `syscribe testplan` — list TestPlans, show plan detail, and roll up a
//! verification verdict (GH #38; REQ-TRS-PLAN-005). Also the home of the
//! `--plan TP-X` lens helper (REQ-TRS-PLAN-006) shared by `matrix`, `audit` and
//! `verification-depth`.
//!
//! This is the BINARY-crate module; it builds on the library module
//! `syscribe_model::testplan` (`effective_testcases`, `plan_configs`,
//! `member_active_in_any_config`) and REUSES the `matrix` coverage machinery
//! (`crate::matrix::Coverage`) and the executed-evidence fold
//! (`crate::query::{tc_verdict, TcVerdict}`) — no second coverage or verdict
//! definition lives here.

use std::collections::BTreeSet;

use serde_json::json;
use syscribe_model::{
    element::{ElementType, RawElement},
    resolver::Resolver,
    results::ResultsData,
    testplan as lib,
    variability,
};

use crate::export::SCHEMA_VERSION;
use crate::matrix::Coverage;
use crate::query::{tc_verdict, TcVerdict};

// ── shared helpers ──────────────────────────────────────────────────────────

/// Display id: stable `id:` when present, else qualified name.
fn disp_id(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Cross-reference identity keys an element answers to (qname + stable id).
fn keys(e: &RawElement) -> Vec<String> {
    let mut k = vec![e.qualified_name.clone()];
    if let Some(id) = &e.frontmatter.id {
        k.push(id.clone());
    }
    k
}

fn is_type(e: &RawElement, t: ElementType) -> bool {
    e.frontmatter.element_type.as_ref() == Some(&t)
}

/// Resolve a `TP-*` argument to its TestPlan element. Returns `None` (caller
/// reports a usage error) when the id is unknown or resolves to a non-TestPlan.
fn resolve_plan<'a>(
    elements: &'a [RawElement],
    resolver: &Resolver,
    tp: &str,
) -> Option<&'a RawElement> {
    let plan = resolver.resolve_ref(elements, tp)?;
    if Resolver::is_test_plan(plan) {
        Some(plan)
    } else {
        None
    }
}

// ── in-scope requirements (REQ-TRS-PLAN-005) ────────────────────────────────

/// The plan's **in-scope requirements**, as stable ids (deduped, sorted).
///
/// * With `demonstrates:` → the **goal-closure**: each demonstrated `Requirement`
///   plus the transitive closure of its `derivedChildren` (computed from the
///   `derivedFrom:` reverse direction); plus, for each demonstrated
///   `SafetyGoal`/`CybersecurityGoal`, the requirements that
///   `derivedFromSafetyGoal:`/`derivedFromSecurityGoal:` it (then their derived
///   closure too). `Argument` targets contribute nothing directly for v1.
/// * Without `demonstrates:` → the union of the `verifies:` targets (resolved to
///   Requirements) of the plan's effective TestCase set.
pub fn in_scope_requirements(
    plan: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> Vec<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();

    match &plan.frontmatter.demonstrates {
        Some(goals) if !goals.is_empty() => {
            // Seed requirements from the demonstrated goals.
            let mut seeds: Vec<&RawElement> = Vec::new();
            for g in goals {
                let Some(target) = resolver.resolve_ref(elements, g) else {
                    continue;
                };
                if Resolver::is_native_requirement(target) {
                    seeds.push(target);
                } else if Resolver::is_safety_goal(target)
                    || Resolver::is_cybersecurity_goal(target)
                {
                    for req in requirements_derived_from_goal(target, elements) {
                        seeds.push(req);
                    }
                }
                // Argument targets contribute no requirements directly (v1).
            }
            // Transitive derivedChildren closure of each seed.
            for seed in seeds {
                collect_derived_closure(seed, elements, &mut out);
            }
        }
        _ => {
            // No demonstrates: → verifies: targets of the effective TC set.
            let tcs = lib::effective_testcases(plan, elements, resolver);
            for tc in tcs {
                if let Some(verifies) = &tc.frontmatter.verifies {
                    for v in verifies {
                        if let Some(t) = resolver.resolve_ref(elements, v) {
                            if Resolver::is_native_requirement(t) {
                                out.insert(disp_id(t));
                            }
                        }
                    }
                }
            }
        }
    }

    out.into_iter().collect()
}

/// Requirements whose `derivedFromSafetyGoal:` / `derivedFromSecurityGoal:`
/// names `goal` (by stable id or qname).
fn requirements_derived_from_goal<'a>(
    goal: &RawElement,
    elements: &'a [RawElement],
) -> Vec<&'a RawElement> {
    let gkeys = keys(goal);
    elements
        .iter()
        .filter(|e| Resolver::is_native_requirement(e))
        .filter(|e| {
            let from_sg = e.frontmatter.derived_from_safety_goal.as_deref();
            let from_csg = e.frontmatter.derived_from_cybersecurity_goal.as_deref();
            [from_sg, from_csg]
                .into_iter()
                .flatten()
                .any(|d| gkeys.iter().any(|k| k == d))
        })
        .collect()
}

/// Insert `req` and the transitive closure of its `derivedChildren` (native
/// requirements whose `derivedFrom:` points back at an already-collected node)
/// into `out`, as display ids.
fn collect_derived_closure(
    req: &RawElement,
    elements: &[RawElement],
    out: &mut BTreeSet<String>,
) {
    let mut frontier: Vec<&RawElement> = vec![req];
    while let Some(node) = frontier.pop() {
        let id = disp_id(node);
        if !out.insert(id) {
            continue; // already visited — also guards against cycles.
        }
        let nkeys = keys(node);
        for child in elements.iter().filter(|e| Resolver::is_native_requirement(e)) {
            if let Some(df) = &child.frontmatter.derived_from {
                if df.iter().any(|d| nkeys.iter().any(|k| k == d)) {
                    frontier.push(child);
                }
            }
        }
    }
}

// ── plan-scoped element slice (REQ-TRS-PLAN-006 lens + coverage scoping) ─────

/// Build the plan-scoped element slice used both by the coverage rollup and the
/// `--plan` lens: the in-scope requirements ∪ the effective (member) TestCases ∪
/// the plan's bound Configurations ∪ the architecture elements that `satisfies:`
/// an in-scope requirement. The Configurations are included so the reused
/// `Coverage::rollup` / `matrix` machinery scopes its columns to the plan's
/// configs only.
fn plan_scoped_elements(
    plan: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> Vec<RawElement> {
    let in_scope: BTreeSet<String> =
        in_scope_requirements(plan, elements, resolver).into_iter().collect();
    let members: BTreeSet<String> = lib::effective_testcases(plan, elements, resolver)
        .iter()
        .map(|tc| disp_id(tc))
        .collect();
    let configs: BTreeSet<String> = lib::plan_configs(plan, elements, resolver)
        .iter()
        .map(|c| c.qualified_name.clone())
        .collect();

    let mut out: Vec<RawElement> = Vec::new();
    for e in elements {
        let keep = if Resolver::is_native_requirement(e) {
            in_scope.contains(&disp_id(e))
        } else if is_type(e, ElementType::TestCase) {
            members.contains(&disp_id(e))
        } else if Resolver::is_configuration(e) {
            configs.contains(&e.qualified_name)
        } else if Resolver::is_feature_def(e) {
            // FeatureDefs are kept so the reused variability machinery stays
            // active (per-config coverage grid); they are neither rows nor
            // columns of the coverage computation.
            true
        } else if e.frontmatter.satisfies.as_ref().is_some_and(|v| !v.is_empty()) {
            // Satisfying architecture elements whose satisfies: hits an in-scope req.
            e.frontmatter
                .satisfies
                .as_ref()
                .unwrap()
                .iter()
                .any(|s| in_scope.contains(s) || resolves_to_in_scope(s, elements, resolver, &in_scope))
        } else {
            false
        };
        if keep {
            out.push(e.clone());
        }
    }
    out
}

fn resolves_to_in_scope(
    r: &str,
    elements: &[RawElement],
    resolver: &Resolver,
    in_scope: &BTreeSet<String>,
) -> bool {
    resolver
        .resolve_ref(elements, r)
        .is_some_and(|t| in_scope.contains(&disp_id(t)))
}

/// Public `--plan TP-X` lens (REQ-TRS-PLAN-006): the element subset of `elements`
/// restricted to the plan's in-scope requirements ∪ effective TestCases ∪ their
/// satisfying architecture elements ∪ the plan's Configurations. Exits 1 on an
/// unknown / non-TestPlan id. Dormant-safe (works with no feature model).
pub fn plan_lens(elements: &[RawElement], tp: &str) -> Vec<RawElement> {
    let resolver = Resolver::new(elements);
    match resolve_plan(elements, &resolver, tp) {
        Some(plan) => plan_scoped_elements(plan, elements, &resolver),
        None => {
            eprintln!("Error: unknown TestPlan '{tp}'");
            std::process::exit(1);
        }
    }
}

// ── verdict roll-up (REQ-TRS-PLAN-005) ──────────────────────────────────────

/// Plan verdict ∈ `pass | fail | incomplete | empty`, folded over the effective
/// TestCase set within the plan's config projection, reusing the existing
/// `tc_verdict` evidence fold. Empty effective set → `empty`. Any member Fail →
/// `fail`. All members Pass → `pass`. Otherwise (missing / no results) →
/// `incomplete`.
fn plan_verdict(
    plan: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    results: Option<&ResultsData>,
) -> &'static str {
    let members = lib::effective_testcases(plan, elements, resolver);
    if members.is_empty() {
        return "empty";
    }
    let configs = lib::plan_configs(plan, elements, resolver);
    let pkg = variability::package_conditions(elements);
    let feat_alias = variability::feature_id_to_qname(elements);

    let mut all_pass = true;
    for tc in &members {
        // Members active in none of the plan's configs (escaping) are not part of
        // the executed scope for this projection; skip them from the fold.
        if !lib::member_active_in_any_config(tc, &configs, &pkg, &feat_alias) {
            all_pass = false;
            continue;
        }
        match tc_verdict(tc, results) {
            TcVerdict::Fail => return "fail",
            TcVerdict::Pass => {}
            TcVerdict::Unknown => all_pass = false,
        }
    }
    if all_pass {
        "pass"
    } else {
        "incomplete"
    }
}

// ── list (`testplan`) ───────────────────────────────────────────────────────

struct PlanRow {
    id: String,
    title: String,
    scope: String,
    configs: Vec<String>,
    effective_count: usize,
    coverage_pct: Option<f64>,
    verdict: &'static str,
}

fn plan_row(
    plan: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    results: Option<&ResultsData>,
) -> PlanRow {
    let members = lib::effective_testcases(plan, elements, resolver);
    let configs: Vec<String> = lib::plan_configs(plan, elements, resolver)
        .iter()
        .map(|c| disp_id(c))
        .collect();
    let scoped = plan_scoped_elements(plan, elements, resolver);
    let coverage = Coverage::rollup(&scoped, results, false);
    let (cov, app) = coverage.overall();
    PlanRow {
        id: disp_id(plan),
        title: plan.frontmatter.name.clone().unwrap_or_default(),
        scope: plan.frontmatter.scope.clone().unwrap_or_default(),
        configs,
        effective_count: members.len(),
        coverage_pct: Coverage::percent(cov, app),
        verdict: plan_verdict(plan, elements, resolver, results),
    }
}

/// `syscribe -m <root> testplan [--json]` — list every TestPlan.
pub fn cmd_testplan_list(elements: &[RawElement], json: bool, results: Option<&ResultsData>) {
    let resolver = Resolver::new(elements);
    let mut plans: Vec<&RawElement> =
        elements.iter().filter(|e| Resolver::is_test_plan(e)).collect();
    plans.sort_by_key(|p| disp_id(p));

    let rows: Vec<PlanRow> =
        plans.iter().map(|p| plan_row(p, elements, &resolver, results)).collect();

    if json {
        let items: Vec<_> = rows
            .iter()
            .map(|r| {
                json!({
                    "id": r.id,
                    "title": r.title,
                    "scope": r.scope,
                    "configurations": r.configs,
                    "effectiveTestCaseCount": r.effective_count,
                    "coveragePct": r.coverage_pct,
                    "verdict": r.verdict,
                })
            })
            .collect();
        let doc = json!({
            "schemaVersion": SCHEMA_VERSION,
            "testPlans": items,
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return;
    }

    if rows.is_empty() {
        println!("No TestPlans found.");
        return;
    }

    println!("# TestPlans ({})", rows.len());
    println!();
    println!("| ID | Title | Scope | Configurations | Effective TCs | Coverage | Verdict |");
    println!("|---|---|---|---|---|---|---|");
    for r in &rows {
        let cfgs = if r.configs.is_empty() { "—".to_string() } else { r.configs.join(", ") };
        let cov = r.coverage_pct.map_or_else(|| "n/a".to_string(), |p| format!("{p:.1}%"));
        let title = if r.title.is_empty() { "—".to_string() } else { r.title.clone() };
        let scope = if r.scope.is_empty() { "—".to_string() } else { r.scope.clone() };
        println!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            r.id, title, scope, cfgs, r.effective_count, cov, r.verdict
        );
    }
}

// ── detail (`testplan TP-X`) ────────────────────────────────────────────────

/// `syscribe -m <root> testplan TP-X [--json]` — detail for one plan. Returns
/// the process exit code (0 ok · 1 unknown / non-TestPlan id).
pub fn cmd_testplan_detail(
    elements: &[RawElement],
    tp: &str,
    json: bool,
    results: Option<&ResultsData>,
) -> i32 {
    let resolver = Resolver::new(elements);
    let Some(plan) = resolve_plan(elements, &resolver, tp) else {
        eprintln!("Error: unknown TestPlan '{tp}'");
        return 1;
    };

    let configs = lib::plan_configs(plan, elements, &resolver);
    let config_ids: Vec<String> = configs.iter().map(|c| disp_id(c)).collect();
    let pkg = variability::package_conditions(elements);
    let feat_alias = variability::feature_id_to_qname(elements);

    // Resolved members, flagged escaping (active in none of the plan's configs).
    let members = lib::effective_testcases(plan, elements, &resolver);
    let explicit: BTreeSet<String> = plan
        .frontmatter
        .test_cases
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(|r| resolver.resolve_ref(elements, r))
        .map(disp_id)
        .collect();
    struct MemberRow {
        id: String,
        via: &'static str,
        escaping: bool,
    }
    let member_rows: Vec<MemberRow> = members
        .iter()
        .map(|tc| MemberRow {
            id: disp_id(tc),
            via: if explicit.contains(&disp_id(tc)) { "explicit" } else { "selection" },
            escaping: !lib::member_active_in_any_config(tc, &configs, &pkg, &feat_alias),
        })
        .collect();

    let in_scope = in_scope_requirements(plan, elements, &resolver);

    // Per-config + overall coverage grid via the reused matrix machinery.
    let scoped = plan_scoped_elements(plan, elements, &resolver);
    let coverage = Coverage::rollup(&scoped, results, false);

    let verdict = plan_verdict(plan, elements, &resolver, results);

    let demonstrates: Vec<String> =
        plan.frontmatter.demonstrates.clone().unwrap_or_default();

    if json {
        let members_json: Vec<_> = member_rows
            .iter()
            .map(|m| json!({ "id": m.id, "via": m.via, "escaping": m.escaping }))
            .collect();
        // coverage map keyed by config id (or "overall" for the flat fallback).
        let mut cov_map = serde_json::Map::new();
        if coverage.is_flat() {
            let (c, a) = coverage.overall();
            cov_map.insert("overall".to_string(), json!({ "covered": c, "applicable": a }));
        } else {
            for (cid, c, a) in coverage.per_config() {
                cov_map.insert(cid.clone(), json!({ "covered": c, "applicable": a }));
            }
            let (c, a) = coverage.overall();
            cov_map.insert("overall".to_string(), json!({ "covered": c, "applicable": a }));
        }
        let doc = json!({
            "schemaVersion": SCHEMA_VERSION,
            "id": disp_id(plan),
            "title": plan.frontmatter.name.clone().unwrap_or_default(),
            "scope": plan.frontmatter.scope.clone().unwrap_or_default(),
            "status": plan.frontmatter.status.clone().unwrap_or_default(),
            "configurations": config_ids,
            "demonstrates": demonstrates,
            "effectiveTestCases": members_json,
            "inScopeRequirements": in_scope,
            "coverage": cov_map,
            "verdict": verdict,
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return 0;
    }

    println!("# TestPlan {}", disp_id(plan));
    println!();
    println!("- Title: {}", plan.frontmatter.name.as_deref().unwrap_or("—"));
    println!("- Scope: {}", plan.frontmatter.scope.as_deref().unwrap_or("—"));
    println!("- Status: {}", plan.frontmatter.status.as_deref().unwrap_or("—"));
    println!(
        "- Configurations: {}",
        if config_ids.is_empty() { "—".to_string() } else { config_ids.join(", ") }
    );
    println!(
        "- Demonstrates: {}",
        if demonstrates.is_empty() { "—".to_string() } else { demonstrates.join(", ") }
    );
    println!("- Verdict: {verdict}");
    println!();

    println!("## Member TestCases ({})", member_rows.len());
    println!();
    if member_rows.is_empty() {
        println!("(none)");
    } else {
        println!("| TestCase | Via | Escaping |");
        println!("|---|---|---|");
        for m in &member_rows {
            println!("| {} | {} | {} |", m.id, m.via, if m.escaping { "yes" } else { "" });
        }
    }
    println!();

    println!("## In-Scope Requirements ({})", in_scope.len());
    println!();
    if in_scope.is_empty() {
        println!("(none)");
    } else {
        for r in &in_scope {
            println!("- {r}");
        }
    }
    println!();

    println!("## Coverage");
    println!();
    if coverage.is_flat() {
        let (c, a) = coverage.overall();
        println!("No feature model — flat requirement/testcase coverage:");
        println!("  Overall: {c}/{a} ({})", fmt_pct(Coverage::percent(c, a)));
    } else {
        println!("covered / applicable (N/A excluded):");
        for (cid, c, a) in coverage.per_config() {
            println!("  {cid}: {c}/{a} ({})", fmt_pct(Coverage::percent(*c, *a)));
        }
        let (c, a) = coverage.overall();
        println!("  Overall: {c}/{a} ({})", fmt_pct(Coverage::percent(c, a)));
    }

    0
}

fn fmt_pct(p: Option<f64>) -> String {
    p.map_or_else(|| "n/a".to_string(), |v| format!("{v:.1}%"))
}

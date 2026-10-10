//! `syscribe audit` — read-only safety-readiness dashboard (REQ-TRS-OUT-013, GH #15).
//!
//! Aggregates existing data into one rollup plus a configurable PASS/FAIL
//! verdict. It REUSES, rather than reimplements:
//!   * `validator::validate_with_config` — the finding set (errors + W306 + …);
//!   * the `matrix` coverage computation (`matrix::Coverage::rollup`);
//!   * the issue-#18 profile loader/promotion (`config::load_profiles` +
//!     `query::profile_promoted`).
//!
//! Sections (mirrored in `--json`):
//!   1. Requirement status split — overall and per top-level package.
//!   2. SIL / ASIL distribution (with a QM/none bucket).
//!   3. Per-configuration coverage % (matrix coverage; flat fallback when no FM).
//!   4. Orphans — unverified / unsatisfied requirements, dangling TestCases,
//!      requirements with neither derivedFrom nor derivedChildren.
//!   5. Safety — hazards, goals by integrity level, FTA, FMEA, HW metrics (GH #216).
//!   6. Security — TARA, CAL, vulnerabilities, IEC 62443 zones (GH #216).
//!   7. Readiness verdict — PASS/FAIL naming the triggering codes/counts.
//!
//! Policy: FAIL (exit 2) when any Error-severity finding exists, OR any finding
//! selected by the `[audit]` table of `.syscribe.toml` (default: `W306`; `W033` and
//! `W805` on an ASIL C/D goal — GH #216), OR — under `--profile <name>` — any
//! finding the profile promotes is present. PASS → exit 0.

use std::collections::BTreeMap;

use serde_json::json;
use syscribe_model::{
    config::{Profile, ValidateConfig},
    element::{ElementType, RawElement},
    resolver::Resolver,
    validator::{self, Severity},
};

use crate::matrix::Coverage;

/// Requirement `status:` values reported in the status split, in lifecycle order.
const STATUS_ORDER: [&str; 5] = ["draft", "review", "approved", "implemented", "verified"];

fn is_type(e: &RawElement, t: ElementType) -> bool {
    e.frontmatter.element_type.as_ref() == Some(&t)
}

/// Display id: stable `id:` when present, else qualified name.
fn disp_id(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Top-level package: the first `::` segment of the qualified name, or `(root)`
/// for an element directly under the model root.
fn top_pkg(e: &RawElement) -> String {
    match e.qualified_name.split_once("::") {
        Some((head, _)) => head.to_string(),
        None => "(root)".to_string(),
    }
}

/// Cross-reference identity keys an inbound `verifies:`/`satisfies:` may use.
fn keys(e: &RawElement) -> Vec<String> {
    let mut k = vec![e.qualified_name.clone()];
    if let Some(id) = &e.frontmatter.id {
        k.push(id.clone());
    }
    k
}

/// One status counter (overall or per-package): counts keyed by status plus an
/// `other` bucket for any status outside [`STATUS_ORDER`].
#[derive(Default)]
struct StatusCounts {
    by_status: BTreeMap<String, u32>,
    other: u32,
    total: u32,
}

impl StatusCounts {
    fn add(&mut self, status: Option<&str>) {
        self.total += 1;
        match status {
            Some(s) if STATUS_ORDER.contains(&s) => {
                *self.by_status.entry(s.to_string()).or_insert(0) += 1;
            }
            Some(_) => self.other += 1,
            None => self.other += 1,
        }
    }

    fn to_json(&self) -> serde_json::Value {
        let mut m = serde_json::Map::new();
        for s in STATUS_ORDER {
            m.insert(s.to_string(), json!(self.by_status.get(s).copied().unwrap_or(0)));
        }
        m.insert("other".to_string(), json!(self.other));
        m.insert("total".to_string(), json!(self.total));
        serde_json::Value::Object(m)
    }

    fn print_inline(&self) {
        let mut parts: Vec<String> = STATUS_ORDER
            .iter()
            .map(|s| format!("{}={}", s, self.by_status.get(*s).copied().unwrap_or(0)))
            .collect();
        if self.other > 0 {
            parts.push(format!("other={}", self.other));
        }
        parts.push(format!("total={}", self.total));
        println!("  {}", parts.join("  "));
    }
}

/// Normalised ASIL letter (`"ASIL C"` / `"c"` → `"C"`).
fn asil_letter(a: &str) -> String {
    a.trim().trim_start_matches("ASIL").trim_start_matches("asil").trim().to_ascii_uppercase()
}

/// The finding set the audit counts: projection-aware validation, narrowed to
/// the plan scope when given.
///
/// When `sel` is `Some`, findings are computed via the **projection-aware**
/// validator (`projection::validate_projected`) — exactly as `validate --config`
/// does — so cross-reference-resolution codes (E102–E106) for references into the
/// projected-out part are suppressed, and `audit --config` agrees with
/// `validate --config` on error-severity findings (GH #36).
///
/// `audit --plan` (GH #40): validate the FULL model (so every reference resolves —
/// no escaping-reference artifacts), then count only findings whose element lies in
/// the plan's scope. Without a plan, all findings count.
pub fn audit_findings(
    elements: &[RawElement],
    config: &ValidateConfig,
    sel: Option<&syscribe_model::projection::Selection>,
    plan_scope: Option<&std::collections::HashSet<String>>,
) -> Vec<validator::Finding> {
    let raw: Vec<validator::Finding> = match sel {
        Some(s) => syscribe_model::projection::validate_projected(elements, config, s),
        None => validator::validate_with_config(elements, config).findings,
    };
    match plan_scope {
        Some(scope) => raw.into_iter().filter(|f| scope.contains(&f.file)).collect(),
        None => raw,
    }
}

/// The readiness verdict over an already-computed finding set: `(pass, reasons)`.
/// FAIL when any error-severity finding, any code in `[audit] fail_on` (default
/// `W306`), any `[audit.fail_on_asil]` code on an element of a listed ASIL
/// (default `W033`/`W805` at C/D), or any profile-promoted finding is present.
pub fn verdict_from_findings(
    findings: &[validator::Finding],
    elements: &[RawElement],
    config: &ValidateConfig,
    profile: Option<&Profile>,
    failing_goals: &[String],
) -> (bool, Vec<String>) {
    let errors = findings.iter().filter(|f| f.severity == Severity::Error).count();
    let candidates: Vec<&validator::Finding> =
        findings.iter().filter(|f| f.severity != Severity::Error).collect();
    let promoted = match profile {
        Some(p) => crate::query::profile_promoted(p, elements, &candidates),
        None => Vec::new(),
    };
    let mut reasons: Vec<String> = Vec::new();
    if errors > 0 {
        reasons.push(format!("{errors} error-severity finding(s)"));
    }
    if !failing_goals.is_empty() {
        reasons.push(format!(
            "{} safety goal(s) FAILING on ingested test results ({})",
            failing_goals.len(),
            failing_goals.join(", ")
        ));
    }
    let audit = &config.audit;
    for code in &audit.fail_on {
        let n = candidates.iter().filter(|f| f.code == code.as_str()).count();
        if n > 0 {
            let note = if code == "W306" { " (unsatisfied safety mechanism)" } else { "" };
            reasons.push(format!("{n} {code} finding(s){note}"));
        }
    }
    for (code, levels) in &audit.fail_on_asil {
        if audit.fail_on.contains(code) {
            continue; // already failing at every level
        }
        let mut by_level: BTreeMap<String, u32> = BTreeMap::new();
        for f in candidates.iter().filter(|f| f.code == code.as_str()) {
            let asil = elements
                .iter()
                .find(|e| e.file_path == f.file && e.frontmatter.asil_level.is_some())
                .and_then(|e| e.frontmatter.asil_level.as_deref().map(asil_letter));
            if let Some(a) = asil.filter(|a| levels.contains(a)) {
                *by_level.entry(a).or_insert(0) += 1;
            }
        }
        let n: u32 = by_level.values().sum();
        if n > 0 {
            let lv: Vec<String> = by_level.iter().map(|(a, c)| format!("ASIL {a}: {c}")).collect();
            reasons.push(format!("{n} {code} finding(s) on high-integrity goals ({})", lv.join(", ")));
        }
    }
    if !promoted.is_empty() {
        let codes: std::collections::BTreeSet<&str> = promoted.iter().map(|f| f.code).collect();
        reasons.push(format!(
            "{} profile-promoted finding(s) [{}]",
            promoted.len(),
            codes.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    (reasons.is_empty(), reasons)
}

/// The readiness verdict over `elements`: `(pass, reasons)`. Shared by
/// `cmd_audit` and `cmd_audit_all_configs` so the policy is defined once.
pub fn audit_verdict(
    elements: &[RawElement],
    config: &ValidateConfig,
    profile: Option<&Profile>,
    sel: Option<&syscribe_model::projection::Selection>,
    plan_scope: Option<&std::collections::HashSet<String>>,
) -> (bool, Vec<String>) {
    let findings = audit_findings(elements, config, sel, plan_scope);
    let projected: Option<Vec<RawElement>> = sel.map(|s| syscribe_model::projection::project_raw(elements, s));
    let view: &[RawElement] = projected.as_deref().unwrap_or(elements);
    let resolver = Resolver::new(view);
    let failing = failing_goals(view, &resolver, config, plan_scope.is_some());
    verdict_from_findings(&findings, elements, config, profile, &failing)
}

/// The safety case over the **reporting** view, so a `coverage = true` link type counts
/// the same here as in every dashboard section. `coverage_view` keeps the element order,
/// so `resolver` (built on `view`) stays valid. Shared by the verdict and the section so
/// they cannot disagree.
fn goal_verdicts(
    view: &[RawElement],
    resolver: &Resolver,
    config: &ValidateConfig,
    results: &syscribe_model::results::ResultsData,
) -> syscribe_model::safety_case::SafetyCase {
    let cov = syscribe_model::link_types::coverage_view(view, &config.link_types);
    syscribe_model::safety_case::build(
        &cov,
        resolver,
        "",
        syscribe_model::safety_case::BuildOptions::default(),
        &|tc| syscribe_model::results::testcase_verdict(tc, Some(results)),
    )
}

/// Safety goals whose argument is `failing` on the ingested results (GH #256). Empty
/// when no results are loaded.
fn failing_goals(view: &[RawElement], resolver: &Resolver, config: &ValidateConfig, plan_scoped: bool) -> Vec<String> {
    let Some(results) = config.results.as_ref() else { return Vec::new() };
    if plan_scoped {
        return Vec::new(); // goals are model-level; see `goal_verdicts`
    }
    let sc = goal_verdicts(view, resolver, config, results);
    let mut out: Vec<String> = sc
        .goals
        .iter()
        .filter(|g| g.verdict == syscribe_model::safety_case::GoalVerdict::Failing)
        .map(|g| g.root.id.clone())
        .collect();
    out.sort();
    out
}

/// Section "Verification results" (GH #256): active TestCases by verdict, SafetyGoals by
/// safety-case verdict, TestPlans by verdict. `None` when no results are loaded.
fn verification_section(
    view: &[RawElement],
    resolver: &Resolver,
    config: &ValidateConfig,
    in_scope: &dyn Fn(&RawElement) -> bool,
    plan_scoped: bool,
) -> Option<serde_json::Value> {
    let results = config.results.as_ref()?;
    let (mut pass, mut fail, mut unknown) = (0u32, 0u32, 0u32);
    for tc in view.iter().filter(|e| is_type(e, ElementType::TestCase) && in_scope(e)) {
        if tc.frontmatter.status.as_deref() != Some("active") {
            continue;
        }
        match crate::query::tc_verdict(tc, Some(results)) {
            crate::query::TcVerdict::Pass => pass += 1,
            crate::query::TcVerdict::Fail => fail += 1,
            crate::query::TcVerdict::Unknown => unknown += 1,
        }
    }
    // Goals and plans are model-level: under `--plan` they are left out (null) rather than
    // reported as misleading zeros, and no goal verdict is applied to the plan's verdict.
    let (goals, plans) = if plan_scoped {
        (serde_json::Value::Null, serde_json::Value::Null)
    } else {
        let sc = goal_verdicts(view, resolver, config, results);
        let mut goals: BTreeMap<&str, u32> = BTreeMap::from([("supported", 0), ("incomplete", 0), ("failing", 0)]);
        for g in &sc.goals {
            *goals.entry(g.verdict.as_str()).or_insert(0) += 1;
        }
        let mut plans: BTreeMap<&str, u32> =
            BTreeMap::from([("pass", 0), ("fail", 0), ("incomplete", 0), ("empty", 0)]);
        for p in view.iter().filter(|e| is_type(e, ElementType::TestPlan)) {
            *plans.entry(crate::testplan::plan_verdict(p, view, resolver, Some(results))).or_insert(0) += 1;
        }
        (json!(goals), json!(plans))
    };
    Some(json!({
        "tests": { "pass": pass, "fail": fail, "unknown": unknown },
        "goals": goals,
        "plans": plans,
    }))
}

fn print_verification(v: &serde_json::Value) {
    println!("## Verification results");
    println!();
    let t = &v["tests"];
    println!("- Active tests: {} pass, {} fail, {} unknown", t["pass"], t["fail"], t["unknown"]);
    let g = &v["goals"];
    if g.is_null() {
        println!("- Safety goals and test plans: model-level, not evaluated under --plan");
    } else {
        println!(
            "- Safety goals: {} supported, {} incomplete, {} FAILING",
            g["supported"], g["incomplete"], g["failing"]
        );
        let p = &v["plans"];
        println!(
            "- Test plans: {} pass, {} fail, {} incomplete, {} empty",
            p["pass"], p["fail"], p["incomplete"], p["empty"]
        );
    }
    println!();
}

/// `audit --all-configs`: audit each stored `Configuration`'s projected variant;
/// exit non-zero if any variant fails its readiness verdict (CI gate).
pub fn cmd_audit_all_configs(
    elements: &[RawElement],
    config: &ValidateConfig,
    profile: Option<&Profile>,
    json: bool,
) -> i32 {
    use syscribe_model::projection::{resolve_config_flag, SelectionOutcome};
    let configs: Vec<&RawElement> =
        elements.iter().filter(|e| is_type(e, ElementType::Configuration)).collect();
    if configs.is_empty() {
        if json {
            println!("{}", json!({ "configurations": [], "pass": true }));
        } else {
            println!("audit --all-configs: no Configuration elements in the model.");
        }
        return 0;
    }
    // (id, pass, reasons) per configuration — projection-aware verdict (GH #35/#36).
    let mut rows: Vec<(String, bool, Vec<String>)> = Vec::new();
    for c in &configs {
        let cid = disp_id(c);
        let sel = match resolve_config_flag(elements, &cid) {
            SelectionOutcome::Resolved(s) => Some(s),
            _ => None,
        };
        // Each variant is judged on its own evidence (GH #258, REQ-TRS-CFGRES-001).
        let own;
        let cfg_config = if config.results.as_ref().is_some_and(|r| !r.by_config.is_empty()) {
            let mut c = config.clone();
            c.results = c.results.as_ref().map(|r| r.for_config(&cid));
            own = c;
            &own
        } else {
            config
        };
        let (pass, reasons) = audit_verdict(elements, cfg_config, profile, sel.as_ref(), None);
        rows.push((cid, pass, reasons));
    }
    let any_fail = rows.iter().any(|(_, pass, _)| !pass);

    if json {
        let items: Vec<_> = rows
            .iter()
            .map(|(cid, pass, reasons)| json!({ "id": cid, "pass": pass, "reasons": reasons }))
            .collect();
        println!("{}", json!({ "configurations": items, "pass": !any_fail }));
    } else {
        println!("# Audit — all configurations ({})", rows.len());
        println!();
        for (cid, pass, reasons) in &rows {
            if *pass {
                println!("  PASS  {cid}");
            } else {
                println!("  FAIL  {cid} — {}", reasons.join("; "));
            }
        }
        println!();
        println!("Overall: {}", if any_fail { "**FAIL**" } else { "**PASS**" });
    }
    if any_fail {
        2
    } else {
        0
    }
}

pub fn cmd_audit(
    elements: &[RawElement],
    config: &ValidateConfig,
    model_root: &std::path::Path,
    profile: Option<&Profile>,
    sel: Option<&syscribe_model::projection::Selection>,
    plan_scope: Option<&std::collections::HashSet<String>>,
    json: bool,
) -> i32 {
    // The dashboard sections are computed over the **active** element set (the
    // variant when `--config` is given); the verdict and the dangling-TestCase
    // resolution use the projection-aware path / the full model so a reference
    // into the projected-out part is not mistaken for a defect (GH #35/#36).
    //
    // With `--plan` (GH #40), `elements` is the FULL model and `plan_scope` is the
    // set of file paths in the plan's scope: references resolve against the full
    // model (so nothing escapes), while the section rows and the verdict count only
    // in-scope elements.
    let projected: Option<Vec<RawElement>> = sel.map(|s| syscribe_model::projection::project_raw(elements, s));
    let view: &[RawElement] = projected.as_deref().unwrap_or(elements);
    let resolver = Resolver::new(view);
    let full_resolver = Resolver::new(elements);
    let in_scope = |e: &RawElement| -> bool {
        plan_scope.is_none_or(|s| s.contains(&e.file_path))
    };

    // ---- Readiness verdict (shared policy, projection-aware) --------------
    let findings = audit_findings(elements, config, sel, plan_scope);
    let failing = failing_goals(view, &resolver, config, plan_scope.is_some());
    let (pass, reasons) = verdict_from_findings(&findings, elements, config, profile, &failing);

    // REQ-TRS-LINKTYPE-006 — every dashboard section below reads the *reporting*
    // view: a `coverage = true` user-defined link extending satisfies/verifies/
    // derivedFrom/refines counts as that base link. Taken after the verdict, which
    // validates the authored elements (the validator applies extensions itself).
    // Same order as `view`, so `resolver` stays valid; borrowed when unused.
    let cov_view = syscribe_model::link_types::coverage_view(view, &config.link_types);
    let view: &[RawElement] = &cov_view;

    // ---- Section 1: requirement status split ------------------------------
    let reqs: Vec<&RawElement> =
        view.iter().filter(|e| is_type(e, ElementType::Requirement) && in_scope(e)).collect();
    let mut overall_status = StatusCounts::default();
    let mut per_pkg_status: BTreeMap<String, StatusCounts> = BTreeMap::new();
    for r in &reqs {
        overall_status.add(r.frontmatter.status.as_deref());
        per_pkg_status
            .entry(top_pkg(r))
            .or_default()
            .add(r.frontmatter.status.as_deref());
    }

    // ---- Section 2: SIL / ASIL distribution -------------------------------
    let mut sil: BTreeMap<String, u32> = BTreeMap::new();
    let mut asil: BTreeMap<String, u32> = BTreeMap::new();
    let mut qm_none = 0u32;
    for r in &reqs {
        let has_sil = r.frontmatter.sil_level.is_some();
        let has_asil = r.frontmatter.asil_level.is_some();
        if let Some(n) = r.frontmatter.sil_level {
            *sil.entry(n.to_string()).or_insert(0) += 1;
        }
        if let Some(a) = &r.frontmatter.asil_level {
            *asil.entry(a.clone()).or_insert(0) += 1;
        }
        if !has_sil && !has_asil {
            qm_none += 1;
        }
    }

    // ---- Section 3: coverage (reused matrix computation) ------------------
    // Under `--plan`, scope the coverage rows/TC-universe to the plan slice
    // (Coverage runs no validation, so the slice is safe here).
    let coverage = match plan_scope {
        Some(_) => {
            let scoped: Vec<RawElement> =
                view.iter().filter(|e| in_scope(e)).cloned().collect();
            Coverage::rollup(&scoped, config.results.as_ref(), false)
        }
        None => Coverage::rollup(view, config.results.as_ref(), false),
    };

    // ---- Section 3b: coverage by requirement class (GH #252) ---------------
    // The derivation-tree roll-up of `matrix --rollup`, per reqClass. One extra validation pass over
    // the same (scoped) view supplies the derivation and verification indices.
    let class_cov: Result<std::collections::BTreeMap<String, std::collections::BTreeMap<&'static str, usize>>, String> = {
        let scoped: Vec<RawElement>;
        let cov_view: &[RawElement] = match plan_scope {
            Some(_) => {
                scoped = view.iter().filter(|e| in_scope(e)).cloned().collect();
                &scoped
            }
            None => view,
        };
        // Under `--config` keep only the selected Configuration among the Configuration elements, as
        // `matrix` does, so the roll-up is evaluated for that variant and not ANDed over all of them.
        let one_config: Vec<RawElement>;
        let cov_view: &[RawElement] = match sel {
            Some(s) => {
                let wanted = syscribe_model::variability::canon_selection(s, &syscribe_model::variability::feature_id_to_qname(elements));
                let is_cfg = |e: &RawElement| e.frontmatter.element_type == Some(syscribe_model::element::ElementType::Configuration);
                let matches_sel = |e: &RawElement| syscribe_model::projection::canonical_selection(elements, e) == wanted;
                if cov_view.iter().any(|e| is_cfg(e) && matches_sel(e)) {
                    one_config = cov_view.iter().filter(|e| !is_cfg(e) || matches_sel(e)).cloned().collect();
                    &one_config
                } else {
                    cov_view
                }
            }
            None => cov_view,
        };
        let res = validator::validate_with_config(cov_view, config);
        crate::covtree::class_summary(cov_view, &res, config.results.as_ref(), &config.coverage)
    };
    let class_cov_json = match &class_cov {
        Ok(m) => {
            let mut o = serde_json::Map::new();
            for (class, v) in m {
                let g = |k: &str| v.get(k).copied().unwrap_or(0);
                let applicable = g("complete") + g("partial") + g("none");
                let pct = Coverage::percent(g("complete") as u32, applicable as u32).map_or(serde_json::Value::Null, |p| json!(p));
                o.insert(class.clone(), json!({"complete": g("complete"), "partial": g("partial"), "none": g("none"), "na": g("na"), "percentComplete": pct}));
            }
            serde_json::Value::Object(o)
        }
        Err(_) => serde_json::Value::Null,
    };
    let class_cov_error = class_cov.as_ref().err().cloned();

    // ---- Section 4: orphans ----------------------------------------------
    // Satisfaction map: any element's satisfies: target (by qname or id).
    let mut satisfied: std::collections::HashSet<String> = std::collections::HashSet::new();
    for e in view {
        if let Some(sat) = &e.frontmatter.satisfies {
            for s in sat {
                satisfied.insert(s.clone());
            }
        }
    }
    // Active (non-draft) TestCases and their verifies targets.
    let active_tcs: Vec<(&RawElement, Vec<String>)> = view
        .iter()
        .filter(|e| is_type(e, ElementType::TestCase))
        .filter(|e| e.frontmatter.status.as_deref() != Some("draft"))
        .map(|e| (e, e.frontmatter.verifies.clone().unwrap_or_default()))
        .collect();

    let verified = |r: &RawElement| -> bool {
        let rkeys = keys(r);
        active_tcs
            .iter()
            .any(|(_, ver)| ver.iter().any(|v| rkeys.iter().any(|k| k == v)))
    };
    let is_satisfied = |r: &RawElement| -> bool {
        keys(r).iter().any(|k| satisfied.contains(k))
    };

    let mut unverified: Vec<String> = Vec::new();
    let mut unsatisfied: Vec<String> = Vec::new();
    let mut untraced: Vec<String> = Vec::new();
    for r in &reqs {
        let has_parent = r.frontmatter.derived_from.as_ref().is_some_and(|d| !d.is_empty());
        // A requirement with derivedChildren is a parent; it is satisfied/verified
        // transitively through its leaves and can never be satisfied directly
        // (§12.4 / E312 forbid a parent appearing in any satisfies: list). Skip
        // parents from the unsatisfied/unverified orphan sets, mirroring the
        // parent suppression already applied to W002, W300 and W306 in the
        // validator (GH #37).
        let is_parent = !derived_children_of(r, &reqs, &resolver, view).is_empty();
        if !is_parent {
            if !verified(r) {
                unverified.push(disp_id(r));
            }
            if !is_satisfied(r) && !r.frontmatter.is_non_allocatable_requirement() {
                unsatisfied.push(disp_id(r));
            }
        }
        if !has_parent && !is_parent {
            untraced.push(disp_id(r));
        }
    }

    // Dangling TestCases: empty verifies, or none of its targets resolve.
    // Only the active (in-variant) TestCases are considered, but references are
    // resolved against the FULL model so a TestCase that verifies a requirement
    // projected out of this variant is not mis-counted as dangling (GH #36).
    let mut dangling_tcs: Vec<String> = Vec::new();
    for tc in view.iter().filter(|e| is_type(e, ElementType::TestCase) && in_scope(e)) {
        let ver = tc.frontmatter.verifies.clone().unwrap_or_default();
        let resolves = ver
            .iter()
            .any(|v| full_resolver.resolve_ref(elements, v).is_some());
        if ver.is_empty() || !resolves {
            dangling_tcs.push(disp_id(tc));
        }
    }
    unverified.sort();
    unsatisfied.sort();
    untraced.sort();
    dangling_tcs.sort();

    // ---- Sections 5/6: safety & security (GH #216) -------------------------
    let safety = safety_section(view, &resolver, &findings, &in_scope);
    let security = security_section(view, &findings, &in_scope);
    let verification = verification_section(view, &resolver, config, &in_scope, plan_scope.is_some());

    // ---- Section 7: verdict (computed above via verdict_from_findings) -----
    let exit_code = if pass { 0 } else { 2 };

    // ---- Output ----------------------------------------------------------
    if json {
        let mut per_pkg = serde_json::Map::new();
        for (pkg, c) in &per_pkg_status {
            per_pkg.insert(pkg.clone(), c.to_json());
        }
        let doc = json!({
            "statusSplit": {
                "overall": overall_status.to_json(),
                "perPackage": per_pkg,
            },
            "integrityDistribution": {
                "sil": sil_json(&sil),
                "asil": map_json(&asil),
                "qmOrNone": qm_none,
            },
            "coverage": coverage.json(),
            "coverageByClass": class_cov_json,
            "coverageByClassError": class_cov_error,
            "orphans": {
                "unverifiedRequirements": orphan_json(&unverified),
                "unsatisfiedRequirements": orphan_json(&unsatisfied),
                "danglingTestCases": orphan_json(&dangling_tcs),
                "untracedRequirements": orphan_json(&untraced),
            },
            "safety": safety,
            "security": security,
            "verification": verification,
            "verdict": { "pass": pass, "reasons": reasons },
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return exit_code;
    }

    println!("# Safety-Readiness Audit");
    println!();
    println!("Model root: {}", model_root.display());
    if let Some(p) = profile {
        let _ = p;
        println!("Profile: applied");
    }
    println!();

    // 1. Status split
    println!("## Requirement Status Split ({} requirements)", overall_status.total);
    println!();
    println!("Overall:");
    overall_status.print_inline();
    println!();
    println!("Per top-level package:");
    for (pkg, c) in &per_pkg_status {
        println!("  {pkg}:");
        c.print_inline();
    }
    println!();

    // 2. SIL / ASIL
    println!("## SIL / ASIL Distribution");
    println!();
    print!("SIL: ");
    if sil.is_empty() {
        print!("(none)");
    } else {
        let parts: Vec<String> = sil.iter().map(|(k, v)| format!("SIL{k}={v}")).collect();
        print!("{}", parts.join("  "));
    }
    println!();
    print!("ASIL: ");
    if asil.is_empty() {
        print!("(none)");
    } else {
        let parts: Vec<String> = asil.iter().map(|(k, v)| format!("ASIL-{k}={v}")).collect();
        print!("{}", parts.join("  "));
    }
    println!();
    println!("QM/none: {qm_none}");
    println!();

    // 3. Coverage
    println!("## Per-Configuration Coverage");
    println!();
    if coverage.is_flat() {
        let (cov, app) = coverage.overall();
        println!("No feature model — flat requirement/testcase coverage:");
        println!("  Overall: {cov}/{app} ({})", fmt_pct(Coverage::percent(cov, app)));
    } else {
        println!("covered / applicable (N/A excluded):");
        for (cid, cov, app) in coverage.per_config() {
            println!("  {cid}: {cov}/{app} ({})", fmt_pct(Coverage::percent(*cov, *app)));
        }
        let (cov, app) = coverage.overall();
        println!("  Overall: {cov}/{app} ({})", fmt_pct(Coverage::percent(cov, app)));
    }
    println!();

    // 3b. Coverage by requirement class
    println!("## Coverage by Requirement Class");
    println!();
    match &class_cov {
        Err(e) => println!("[coverage] policy problem: {e}"),
        Ok(m) if m.is_empty() => println!("No requirements."),
        Ok(m) => {
            println!("complete / partial / none (derivation-tree roll-up; parents are judged by their leaves and the [coverage] rule,");
            println!("so these figures can differ from the per-configuration grid above):");
            for (class, v) in m {
                let g = |k: &str| v.get(k).copied().unwrap_or(0);
                let applicable = g("complete") + g("partial") + g("none");
                println!("  {class}: {} complete, {} partial, {} none, {} n/a ({})", g("complete"), g("partial"), g("none"), g("na"), fmt_pct(Coverage::percent(g("complete") as u32, applicable as u32)));
            }
        }
    }
    println!();

    // 4. Orphans
    println!("## Orphans");
    println!();
    print_orphans("Requirements with no active verifying TestCase", &unverified);
    print_orphans("Requirements that no element satisfies", &unsatisfied);
    print_orphans("Dangling TestCases (empty/unresolved verifies)", &dangling_tcs);
    print_orphans("Requirements with neither derivedFrom nor derivedChildren", &untraced);
    println!();

    // 5./6. Safety and security
    print_safety(&safety);
    print_security(&security);
    if let Some(v) = &verification {
        print_verification(v);
    }

    // 7. Verdict
    println!("## Readiness Verdict");
    println!();
    if pass {
        println!("Verdict: **PASS** — no errors and no finding selected by the [audit] policy or a profile.");
    } else {
        println!("Verdict: **FAIL** — {}", reasons.join("; "));
    }

    exit_code
}

/// derivedChildren of a requirement: native requirements whose `derivedFrom:`
/// resolves back to `r` (by qname or stable id).
fn derived_children_of<'a>(
    r: &RawElement,
    reqs: &'a [&'a RawElement],
    resolver: &Resolver,
    elements: &'a [RawElement],
) -> Vec<&'a RawElement> {
    let rkeys = keys(r);
    reqs.iter()
        .filter(|child| {
            child.frontmatter.derived_from.as_ref().is_some_and(|df| {
                df.iter().any(|d| {
                    rkeys.iter().any(|k| k == d)
                        || resolver
                            .resolve_ref(elements, d)
                            .is_some_and(|t| std::ptr::eq(t, r))
                })
            })
        })
        .copied()
        .collect()
}

fn fmt_pct(p: Option<f64>) -> String {
    p.map_or_else(|| "n/a".to_string(), |v| format!("{v:.1}%"))
}

fn map_json(m: &BTreeMap<String, u32>) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    for (k, v) in m {
        out.insert(k.clone(), json!(v));
    }
    serde_json::Value::Object(out)
}

/// SIL keys are prefixed `SIL<n>` in JSON for readability.
fn sil_json(m: &BTreeMap<String, u32>) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    for (k, v) in m {
        out.insert(format!("SIL{k}"), json!(v));
    }
    serde_json::Value::Object(out)
}

fn orphan_json(ids: &[String]) -> serde_json::Value {
    json!({ "count": ids.len(), "ids": ids })
}

fn print_orphans(label: &str, ids: &[String]) {
    println!("{label}: {}", ids.len());
    for id in ids {
        println!("  - {id}");
    }
}

// ── Safety & security sections (GH #216) ─────────────────────────────────────

/// Ids of the elements carrying a finding with `code` (stable id, else qname).
fn finding_ids(findings: &[validator::Finding], code: &str, elements: &[RawElement]) -> Vec<String> {
    let mut ids: Vec<String> = findings
        .iter()
        .filter(|f| f.code == code)
        .map(|f| {
            elements
                .iter()
                .find(|e| e.file_path == f.file && !is_type(e, ElementType::FMEAEntry))
                .map(disp_id)
                .unwrap_or_else(|| f.file.clone())
        })
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

fn count_code(findings: &[validator::Finding], code: &str) -> usize {
    findings.iter().filter(|f| f.code == code).count()
}

fn integrity_label(e: &RawElement) -> String {
    let fm = &e.frontmatter;
    if let Some(a) = &fm.asil_level {
        format!("ASIL {}", asil_letter(a))
    } else if let Some(s) = fm.sil_level {
        format!("SIL {s}")
    } else if let Some(p) = &fm.pl_level {
        format!("PL {p}")
    } else {
        "none".to_string()
    }
}

/// Safety block: hazards, goals by integrity level, FTA, FMEA, HW metrics, FFI.
fn safety_section(
    view: &[RawElement],
    resolver: &Resolver,
    findings: &[validator::Finding],
    in_scope: &dyn Fn(&RawElement) -> bool,
) -> serde_json::Value {
    let of = |t: ElementType| view.iter().filter(|e| is_type(e, t.clone()) && in_scope(e)).count();
    let goals: Vec<&RawElement> =
        view.iter().filter(|e| Resolver::is_safety_goal(e) && in_scope(e)).collect();
    let mut by_integrity: BTreeMap<String, u32> = BTreeMap::new();
    for g in &goals {
        *by_integrity.entry(integrity_label(g)).or_insert(0) += 1;
    }
    let fmea_rows: Vec<&RawElement> =
        view.iter().filter(|e| is_type(e, ElementType::FMEAEntry) && in_scope(e)).collect();
    let max_rpn = fmea_rows.iter().filter_map(|e| e.frontmatter.rpn).max();

    let mut metrics = Vec::new();
    let mut failing = 0u32;
    for r in syscribe_model::metrics::report_all(view, resolver) {
        if !goals.iter().any(|g| g.file_path == r.file_path) {
            continue;
        }
        let integrity = r
            .asil
            .as_deref()
            .map(|a| format!("ASIL {}", asil_letter(a)))
            .or_else(|| r.sil.map(|s| format!("SIL {s}")))
            .unwrap_or_else(|| "none".into());
        let result = match (&r.metrics, &r.gate) {
            (Some(_), Some(g)) if g.passed() => "pass",
            (Some(_), Some(_)) => "fail",
            (Some(_), None) => "no-target",
            _ => "n/a",
        };
        if result == "fail" {
            failing += 1;
        }
        metrics.push(json!({
            "goal": r.id,
            "integrity": integrity,
            "spfm": r.metrics.as_ref().and_then(|m| m.spfm),
            "lfm": r.metrics.as_ref().and_then(|m| m.lfm),
            "pmhf": r.metrics.as_ref().map(|m| m.pmhf),
            "result": result,
        }));
    }

    json!({
        "hazardousEvents": of(ElementType::HazardousEvent),
        "safetyGoals": goals.len(),
        "goalsByIntegrity": map_json(&by_integrity),
        "goalsWithoutRequirements": orphan_json(&finding_ids(findings, "W805", view)),
        "unreferencedHazards": orphan_json(&finding_ids(findings, "W800", view)),
        "goalsWithoutIntegrityLevel": orphan_json(&finding_ids(findings, "W801", view)),
        "faultTrees": of(ElementType::FaultTree),
        "faultTreeEvents": of(ElementType::FaultTreeEvent),
        "fmea": {
            "sheets": of(ElementType::FMEASheet),
            "entries": fmea_rows.len(),
            "maxRpn": max_rpn,
            "rowsMissingSOD": count_code(findings, "W931"),
            "highRpnWithoutAction": count_code(findings, "W903"),
            "severityPriorityGaps": count_code(findings, "W932"),
        },
        "metrics": metrics,
        "metricsFailing": failing,
        "ffiGaps": count_code(findings, "W034"),
    })
}

/// Security block: TARA artefacts, CAL distribution, vulnerabilities, IEC 62443 zones.
fn security_section(
    view: &[RawElement],
    findings: &[validator::Finding],
    in_scope: &dyn Fn(&RawElement) -> bool,
) -> serde_json::Value {
    let of = |t: ElementType| view.iter().filter(|e| is_type(e, t.clone()) && in_scope(e)).count();
    let mut by_cal: BTreeMap<String, u32> = BTreeMap::new();
    for g in view.iter().filter(|e| Resolver::is_cybersecurity_goal(e) && in_scope(e)) {
        let cal = g.frontmatter.cal_level.clone().unwrap_or_else(|| "none".into());
        *by_cal.entry(cal).or_insert(0) += 1;
    }
    let vulns: Vec<&RawElement> =
        view.iter().filter(|e| is_type(e, ElementType::VulnerabilityReport) && in_scope(e)).collect();
    let open = vulns.iter().filter(|e| e.frontmatter.status.as_deref() == Some("open")).count();
    let zones: Vec<&RawElement> =
        view.iter().filter(|e| is_type(e, ElementType::Zone) && in_scope(e)).collect();
    let sl_gap = zones
        .iter()
        .filter(|z| matches!((z.frontmatter.achieved_sl, z.frontmatter.target_sl), (Some(a), Some(t)) if a < t))
        .count();
    json!({
        "assets": of(ElementType::Asset),
        "damageScenarios": of(ElementType::DamageScenario),
        "threatScenarios": of(ElementType::ThreatScenario),
        "cybersecurityGoals": by_cal.values().sum::<u32>(),
        "goalsByCal": map_json(&by_cal),
        "securityControls": of(ElementType::SecurityControl),
        "goalsNotImplemented": orphan_json(&finding_ids(findings, "W802", view)),
        "goalsWithoutRequirements": orphan_json(&finding_ids(findings, "W804", view)),
        "assetsWithoutDamageScenario": orphan_json(&finding_ids(findings, "W810", view)),
        "attackTrees": of(ElementType::AttackTree),
        "vulnerabilities": { "total": vulns.len(), "open": open },
        "zones": { "count": zones.len(), "withSlGap": sl_gap },
        "conduits": of(ElementType::Conduit),
    })
}

fn print_id_list(label: &str, v: &serde_json::Value) {
    let n = v["count"].as_u64().unwrap_or(0);
    println!("{label}: {n}");
    for id in v["ids"].as_array().into_iter().flatten().filter_map(|x| x.as_str()) {
        println!("  - {id}");
    }
}

fn print_map(label: &str, v: &serde_json::Value) {
    let parts: Vec<String> = v
        .as_object()
        .map(|m| m.iter().map(|(k, c)| format!("{k}={c}")).collect())
        .unwrap_or_default();
    println!("{label}: {}", if parts.is_empty() { "(none)".to_string() } else { parts.join("  ") });
}

fn print_safety(s: &serde_json::Value) {
    println!("## Safety");
    println!();
    println!("Hazardous events: {}   Safety goals: {}", s["hazardousEvents"], s["safetyGoals"]);
    print_map("Goals by integrity level", &s["goalsByIntegrity"]);
    print_id_list("Goals with no derived Requirement (W805)", &s["goalsWithoutRequirements"]);
    print_id_list("Hazardous events not referenced by a goal (W800)", &s["unreferencedHazards"]);
    print_id_list("Goals without an integrity level (W801)", &s["goalsWithoutIntegrityLevel"]);
    println!("Fault trees: {}   events: {}", s["faultTrees"], s["faultTreeEvents"]);
    let f = &s["fmea"];
    println!(
        "FMEA: {} sheet(s), {} row(s), max RPN {}; rows missing S/O/D (W931): {}; RPN>100 without action (W903): {}; severity>=9 without action (W932): {}",
        f["sheets"], f["entries"], f["maxRpn"].as_u64().map_or("n/a".to_string(), |v| v.to_string()), f["rowsMissingSOD"], f["highRpnWithoutAction"], f["severityPriorityGaps"]
    );
    println!("Hardware metrics ({} failing):", s["metricsFailing"]);
    let rows = s["metrics"].as_array().cloned().unwrap_or_default();
    if rows.is_empty() {
        println!("  (no safety goals)");
    }
    let fmt = |v: &serde_json::Value, sci: bool| -> String {
        match v.as_f64() {
            Some(x) if sci => format!("{x:.2e}"),
            Some(x) => format!("{x:.4}"),
            None => "n/a".to_string(),
        }
    };
    for m in &rows {
        println!(
            "  {} ({}): SPFM {}  LFM {}  PMHF {}  -> {}",
            m["goal"].as_str().unwrap_or("?"),
            m["integrity"].as_str().unwrap_or("?"),
            fmt(&m["spfm"], false),
            fmt(&m["lfm"], false),
            fmt(&m["pmhf"], true),
            m["result"].as_str().unwrap_or("?")
        );
    }
    println!("Freedom-from-interference gaps (W034): {}", s["ffiGaps"]);
    println!();
}

fn print_security(s: &serde_json::Value) {
    println!("## Security");
    println!();
    println!(
        "Assets: {}   Damage scenarios: {}   Threat scenarios: {}   Cybersecurity goals: {}   Controls: {}   Attack trees: {}",
        s["assets"], s["damageScenarios"], s["threatScenarios"], s["cybersecurityGoals"], s["securityControls"], s["attackTrees"]
    );
    print_map("Cybersecurity goals by CAL", &s["goalsByCal"]);
    print_id_list("Goals not implemented by a control (W802)", &s["goalsNotImplemented"]);
    print_id_list("Goals with no derived Requirement (W804)", &s["goalsWithoutRequirements"]);
    print_id_list("Assets not in any damage scenario (W810)", &s["assetsWithoutDamageScenario"]);
    println!("Vulnerabilities: {} total, {} open (W803)", s["vulnerabilities"]["total"], s["vulnerabilities"]["open"]);
    println!(
        "IEC 62443 zones: {} ({} with a security-level gap)   conduits: {}",
        s["zones"]["count"], s["zones"]["withSlGap"], s["conduits"]
    );
    println!();
}

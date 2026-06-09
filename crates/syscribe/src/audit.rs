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
//!   5. Readiness verdict — PASS/FAIL naming the triggering codes/counts.
//!
//! Policy: FAIL (exit 2) when any Error-severity finding exists, OR any W306
//! finding exists (default gate), OR — under `--profile <name>` — any finding the
//! profile promotes is present. PASS → exit 0.

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

/// The audit command. Returns the process exit code (0 PASS · 2 FAIL).
/// The readiness verdict over `elements`: `(pass, reasons)`. FAIL when any
/// error-severity finding, any `W306`, or any profile-promoted finding is present.
/// Shared by `cmd_audit` and `cmd_audit_all_configs` so the policy is defined once.
///
/// When `sel` is `Some`, findings are computed via the **projection-aware**
/// validator (`projection::validate_projected`) — exactly as `validate --config`
/// does — so cross-reference-resolution codes (E102–E106) for references into the
/// projected-out part are suppressed, and `audit --config` agrees with
/// `validate --config` on error-severity findings (GH #36).
pub fn audit_verdict(
    elements: &[RawElement],
    config: &ValidateConfig,
    profile: Option<&Profile>,
    sel: Option<&syscribe_model::projection::Selection>,
) -> (bool, Vec<String>) {
    let findings: Vec<validator::Finding> = match sel {
        Some(s) => syscribe_model::projection::validate_projected(elements, config, s),
        None => validator::validate_with_config(elements, config).findings,
    };
    let errors = findings.iter().filter(|f| f.severity == Severity::Error).count();
    let w306 = findings.iter().filter(|f| f.code == "W306").count();
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
    if w306 > 0 {
        reasons.push(format!("{w306} W306 finding(s) (unsatisfied safety mechanism)"));
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

/// `audit --all-configs`: audit each stored `Configuration`'s projected variant;
/// exit non-zero if any variant fails its readiness verdict (CI gate).
pub fn cmd_audit_all_configs(
    elements: &[RawElement],
    config: &ValidateConfig,
    profile: Option<&Profile>,
    json: bool,
) -> i32 {
    use syscribe_model::projection::{resolve_selection, SelectionOutcome};
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
        let sel = match resolve_selection(elements, &cid) {
            SelectionOutcome::Resolved(s) => Some(s),
            _ => None,
        };
        let (pass, reasons) = audit_verdict(elements, config, profile, sel.as_ref());
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
    json: bool,
) -> i32 {
    // The dashboard sections are computed over the **active** element set (the
    // variant when `--config` is given); the verdict and the dangling-TestCase
    // resolution use the projection-aware path / the full model so a reference
    // into the projected-out part is not mistaken for a defect (GH #35/#36).
    let projected: Option<Vec<RawElement>> = sel.map(|s| syscribe_model::projection::project(elements, s));
    let view: &[RawElement] = projected.as_deref().unwrap_or(elements);
    let resolver = Resolver::new(view);
    let full_resolver = Resolver::new(elements);

    // ---- Readiness verdict (shared policy, projection-aware) --------------
    let (pass, reasons) = audit_verdict(elements, config, profile, sel);

    // ---- Section 1: requirement status split ------------------------------
    let reqs: Vec<&RawElement> =
        view.iter().filter(|e| is_type(e, ElementType::Requirement)).collect();
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
    let coverage = Coverage::rollup(view, config.results.as_ref(), false);

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
            if !is_satisfied(r) {
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
    for tc in view.iter().filter(|e| is_type(e, ElementType::TestCase)) {
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

    // ---- Section 5: verdict (computed above via audit_verdict) ------------
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
            "orphans": {
                "unverifiedRequirements": orphan_json(&unverified),
                "unsatisfiedRequirements": orphan_json(&unsatisfied),
                "danglingTestCases": orphan_json(&dangling_tcs),
                "untracedRequirements": orphan_json(&untraced),
            },
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

    // 4. Orphans
    println!("## Orphans");
    println!();
    print_orphans("Requirements with no active verifying TestCase", &unverified);
    print_orphans("Requirements that no element satisfies", &unsatisfied);
    print_orphans("Dangling TestCases (empty/unresolved verifies)", &dangling_tcs);
    print_orphans("Requirements with neither derivedFrom nor derivedChildren", &untraced);
    println!();

    // 5. Verdict
    println!("## Readiness Verdict");
    println!();
    if pass {
        println!("Verdict: **PASS** — no errors, no W306, no profile-promoted findings.");
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

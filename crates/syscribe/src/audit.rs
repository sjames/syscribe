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
pub fn cmd_audit(
    elements: &[RawElement],
    config: &ValidateConfig,
    model_root: &std::path::Path,
    profile: Option<&Profile>,
    json: bool,
) -> i32 {
    let resolver = Resolver::new(elements);

    // ---- Validation reuse -------------------------------------------------
    let result = validator::validate_with_config(elements, config);
    let errors: Vec<&validator::Finding> =
        result.findings.iter().filter(|f| f.severity == Severity::Error).collect();
    let w306: Vec<&validator::Finding> =
        result.findings.iter().filter(|f| f.code == "W306").collect();
    // Profile-promoted findings (issue #18 logic), considering only warnings/infos.
    let candidates: Vec<&validator::Finding> = result
        .findings
        .iter()
        .filter(|f| f.severity != Severity::Error)
        .collect();
    let promoted: Vec<&validator::Finding> = match profile {
        Some(p) => crate::query::profile_promoted(p, elements, &candidates),
        None => Vec::new(),
    };

    // ---- Section 1: requirement status split ------------------------------
    let reqs: Vec<&RawElement> =
        elements.iter().filter(|e| is_type(e, ElementType::Requirement)).collect();
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
    let coverage = Coverage::rollup(elements, config.results.as_ref(), false);

    // ---- Section 4: orphans ----------------------------------------------
    // Satisfaction map: any element's satisfies: target (by qname or id).
    let mut satisfied: std::collections::HashSet<String> = std::collections::HashSet::new();
    for e in elements {
        if let Some(sat) = &e.frontmatter.satisfies {
            for s in sat {
                satisfied.insert(s.clone());
            }
        }
    }
    // Active (non-draft) TestCases and their verifies targets.
    let active_tcs: Vec<(&RawElement, Vec<String>)> = elements
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
        if !verified(r) {
            unverified.push(disp_id(r));
        }
        if !is_satisfied(r) {
            unsatisfied.push(disp_id(r));
        }
        let has_parent = r.frontmatter.derived_from.as_ref().is_some_and(|d| !d.is_empty());
        let has_children = !derived_children_of(r, &reqs, &resolver, elements).is_empty();
        if !has_parent && !has_children {
            untraced.push(disp_id(r));
        }
    }

    // Dangling TestCases: empty verifies, or none of its targets resolve.
    let mut dangling_tcs: Vec<String> = Vec::new();
    for tc in elements.iter().filter(|e| is_type(e, ElementType::TestCase)) {
        let ver = tc.frontmatter.verifies.clone().unwrap_or_default();
        let resolves = ver
            .iter()
            .any(|v| resolver.resolve_ref(elements, v).is_some());
        if ver.is_empty() || !resolves {
            dangling_tcs.push(disp_id(tc));
        }
    }
    unverified.sort();
    unsatisfied.sort();
    untraced.sort();
    dangling_tcs.sort();

    // ---- Section 5: verdict ----------------------------------------------
    let mut reasons: Vec<String> = Vec::new();
    if !errors.is_empty() {
        reasons.push(format!("{} error-severity finding(s)", errors.len()));
    }
    if !w306.is_empty() {
        reasons.push(format!("{} W306 finding(s) (unsatisfied safety mechanism)", w306.len()));
    }
    if !promoted.is_empty() {
        let codes: std::collections::BTreeSet<&str> = promoted.iter().map(|f| f.code).collect();
        let codes: Vec<&str> = codes.into_iter().collect();
        reasons.push(format!(
            "{} profile-promoted finding(s) [{}]",
            promoted.len(),
            codes.join(", ")
        ));
    }
    let pass = reasons.is_empty();
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

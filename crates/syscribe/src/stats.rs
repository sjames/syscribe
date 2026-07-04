//! `syscribe stats` — read-only corpus-shape digest (REQ-TRS-OUT-021).
//!
//! Aggregates the native `Requirement` population into per-facet histograms plus
//! coverage and orphan rollups, so an LLM client can grasp the shape of a model
//! with tens of thousands of requirements in one call, without reading element
//! files. It REUSES, rather than reimplements:
//!   * `coverage::coverage_summary` — the verified / unverified-leaf / parent
//!     partition (identical numbers to `coverage` and `matrix`);
//!   * the validator reverse indices (`derived_children`) to identify parents;
//!   * the `query` custom-field `--where` predicate and the tag/status filters;
//!   * `projection::project` for the `--config` lens.
//!
//! The same document producer (`compute_stats`) backs both the CLI command and the
//! MCP `stats` tool, so `stats --json` and the tool return byte-identical JSON.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use syscribe_model::{
    element::{ElementType, RawElement},
    results::ResultsData,
    validator::ValidationResult,
};

use crate::query::{self, CustomWhere};

/// Facets that `--group-by` may cross with the top-level package. `package` is a
/// facet of its own but not a valid `group_by` axis (crossing package by package
/// is degenerate).
const GROUP_BY_FACETS: [&str; 5] = ["status", "reqDomain", "silLevel", "asilLevel", "tags"];

/// Default cap on the number of distinct packages reported before the tail is
/// aggregated into an `other` bucket.
const DEFAULT_PACKAGE_TOP_N: usize = 20;

/// Scoping / pivoting options for a `stats` run.
#[derive(Default)]
pub struct StatsOptions<'a> {
    /// Re-key the primary histogram by this facet, crossed with top-level package.
    pub group_by: Option<&'a str>,
    /// Custom-field predicates (AND-combined); restrict the requirement set.
    pub wheres: &'a [CustomWhere],
    /// Restrict to requirements with this `status:`.
    pub status: Option<&'a str>,
    /// Restrict to requirements carrying this tag.
    pub tag: Option<&'a str>,
    /// Package top-N before the `other` bucket (defaults to [`DEFAULT_PACKAGE_TOP_N`]).
    pub package_top_n: Option<usize>,
}

fn is_type(e: &RawElement, t: ElementType) -> bool {
    e.frontmatter.element_type.as_ref() == Some(&t)
}

/// Display id: stable `id:` when present, else qualified name.
fn disp_id(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Top-level package: the first `::` segment of the qualified name, or `(root)`
/// for an element directly under the model root (matches `audit`).
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

/// Does this requirement pass the scoping filters (`--status`, `--tag`, `--where`)?
fn passes_filters(e: &RawElement, opts: &StatsOptions) -> bool {
    if let Some(s) = opts.status {
        if e.frontmatter.status.as_deref() != Some(s) {
            return false;
        }
    }
    if let Some(t) = opts.tag {
        let has = e.frontmatter.tags.iter().flatten().any(|x| x == t);
        if !has {
            return false;
        }
    }
    for pred in opts.wheres {
        if !query::custom_or_extra_matches(e, pred) {
            return false;
        }
    }
    true
}

/// A `value → count` histogram rendered to a JSON object, ordered by count
/// descending then key ascending (stable, LLM-friendly).
fn hist_json(h: &BTreeMap<String, u64>) -> Value {
    let mut pairs: Vec<(&String, &u64)> = h.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    let mut m = serde_json::Map::new();
    for (k, v) in pairs {
        m.insert(k.clone(), json!(v));
    }
    Value::Object(m)
}

/// Collapse a package histogram to the top-N packages plus an aggregated `other`.
fn package_json(h: &BTreeMap<String, u64>, top_n: usize) -> Value {
    let mut pairs: Vec<(String, u64)> = h.iter().map(|(k, v)| (k.clone(), *v)).collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut m = serde_json::Map::new();
    let mut other = 0u64;
    for (i, (k, v)) in pairs.iter().enumerate() {
        if i < top_n {
            m.insert(k.clone(), json!(v));
        } else {
            other += *v;
        }
    }
    if other > 0 {
        m.insert("other".to_string(), json!(other));
    }
    Value::Object(m)
}

/// The value(s) a requirement contributes to a facet. `silLevel`/`asilLevel` fold
/// a "declares neither integrity level" requirement into a `QM/none` bucket.
fn facet_values(e: &RawElement, facet: &str) -> Vec<String> {
    match facet {
        "status" => vec![e.frontmatter.status.clone().unwrap_or_else(|| "unset".to_string())],
        "reqDomain" => vec![e.frontmatter.req_domain.clone().unwrap_or_else(|| "unset".to_string())],
        "silLevel" => {
            let neither =
                e.frontmatter.sil_level.is_none() && e.frontmatter.asil_level.is_none();
            match e.frontmatter.sil_level {
                Some(n) => vec![n.to_string()],
                None if neither => vec!["QM/none".to_string()],
                None => vec![],
            }
        }
        "asilLevel" => {
            let neither =
                e.frontmatter.sil_level.is_none() && e.frontmatter.asil_level.is_none();
            match &e.frontmatter.asil_level {
                Some(a) => vec![a.clone()],
                None if neither => vec!["QM/none".to_string()],
                None => vec![],
            }
        }
        "tags" => e.frontmatter.tags.clone().unwrap_or_default(),
        _ => vec![],
    }
}

/// Compute the stats document over `view` (already projection-applied, if any).
/// `result` must come from validating `view`. Returns `Err(msg)` for an unknown
/// `group_by` facet.
pub fn compute_stats(
    view: &[RawElement],
    result: &ValidationResult,
    results: Option<&ResultsData>,
    opts: &StatsOptions,
) -> Result<Value, String> {
    if let Some(g) = opts.group_by {
        if !GROUP_BY_FACETS.contains(&g) {
            return Err(format!(
                "unknown --group-by facet '{g}'; expected one of: {}",
                GROUP_BY_FACETS.join(", ")
            ));
        }
    }
    let top_n = opts.package_top_n.unwrap_or(DEFAULT_PACKAGE_TOP_N);

    // Requirements in scope: native Requirements passing the scoping filters.
    let reqs: Vec<&RawElement> = view
        .iter()
        .filter(|e| is_type(e, ElementType::Requirement) && passes_filters(e, opts))
        .collect();
    let total = reqs.len() as u64;

    // ---- Facet histograms -------------------------------------------------
    let facet_names = ["status", "reqDomain", "silLevel", "asilLevel", "tags"];
    let mut facets: BTreeMap<&str, BTreeMap<String, u64>> =
        facet_names.iter().map(|f| (*f, BTreeMap::new())).collect();
    let mut package: BTreeMap<String, u64> = BTreeMap::new();
    for r in &reqs {
        for f in facet_names {
            for v in facet_values(r, f) {
                *facets.get_mut(f).unwrap().entry(v).or_insert(0) += 1;
            }
        }
        *package.entry(top_pkg(r)).or_insert(0) += 1;
    }

    let mut facets_json = serde_json::Map::new();
    if let Some(g) = opts.group_by {
        // Cross the chosen facet with top-level package: per-package histogram.
        let mut by_pkg: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
        for r in &reqs {
            let pkg = top_pkg(r);
            for v in facet_values(r, g) {
                *by_pkg.entry(pkg.clone()).or_default().entry(v).or_insert(0) += 1;
            }
        }
        let mut nested = serde_json::Map::new();
        for (pkg, h) in &by_pkg {
            nested.insert(pkg.clone(), hist_json(h));
        }
        // Emit the untouched facets plus a `byPackage` map for the grouped one.
        for f in facet_names {
            if f == g {
                continue;
            }
            facets_json.insert(f.to_string(), hist_json(&facets[f]));
        }
        facets_json.insert("package".to_string(), package_json(&package, top_n));
        facets_json.insert("byPackage".to_string(), Value::Object(nested));
    } else {
        for f in facet_names {
            facets_json.insert(f.to_string(), hist_json(&facets[f]));
        }
        facets_json.insert("package".to_string(), package_json(&package, top_n));
    }

    // ---- Coverage rollup (reused; over the full active model) --------------
    // NB: coverage always reflects the whole active model so it equals what the
    // `coverage`/`matrix` commands report — it is NOT narrowed by --where/--status
    // /--tag (those scope only the facet/orphan requirement set).
    let coverage = crate::coverage::coverage_summary(view, result, results);

    // ---- Orphans (audit-style, over the scoped requirement set) -----------
    let mut satisfied: std::collections::HashSet<String> = std::collections::HashSet::new();
    for e in view {
        if let Some(sat) = &e.frontmatter.satisfies {
            for s in sat {
                satisfied.insert(s.clone());
            }
        }
    }
    let active_tc_targets: Vec<Vec<String>> = view
        .iter()
        .filter(|e| is_type(e, ElementType::TestCase))
        .filter(|e| e.frontmatter.status.as_deref() != Some("draft"))
        .map(|e| e.frontmatter.verifies.clone().unwrap_or_default())
        .collect();
    let is_verified = |r: &RawElement| -> bool {
        let rkeys = keys(r);
        active_tc_targets.iter().any(|ver| ver.iter().any(|v| rkeys.iter().any(|k| k == v)))
    };
    let is_satisfied = |r: &RawElement| -> bool { keys(r).iter().any(|k| satisfied.contains(k)) };

    let mut unverified: Vec<String> = Vec::new();
    let mut unsatisfied: Vec<String> = Vec::new();
    let mut untraced: Vec<String> = Vec::new();
    for r in &reqs {
        let id = r.frontmatter.id.as_deref().unwrap_or("");
        // Parent ⇔ non-empty derivedChildren (validator reverse index). A parent is
        // satisfied/verified transitively through its leaves and is forbidden from
        // any satisfies: list (§12.4 / E312), so it is excluded from the
        // unsatisfied/unverified sets — mirroring the W002/W300 parent suppression
        // (REQ-TRS-OUT-013 / GH #37).
        let is_parent = result.derived_children.get(id).is_some_and(|c| !c.is_empty());
        let has_parent = r.frontmatter.derived_from.as_ref().is_some_and(|d| !d.is_empty());
        if !is_parent {
            if !is_verified(r) {
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
    unverified.sort();
    unsatisfied.sort();
    untraced.sort();

    Ok(json!({
        "total": total,
        "facets": Value::Object(facets_json),
        "coverage": {
            "verified": coverage.verified_count,
            "unverifiedLeaves": coverage.unverified_leaves.len(),
            "parentsMissingIntegration": coverage.parents_missing_integration.len(),
        },
        "orphans": {
            "unverifiedRequirements": unverified.len(),
            "unsatisfiedRequirements": unsatisfied.len(),
            "untraced": untraced.len(),
            "ids": {
                "unverifiedRequirements": unverified,
                "unsatisfiedRequirements": unsatisfied,
                "untraced": untraced,
            },
        },
    }))
}

/// Render the stats document as a compact human-readable digest.
fn print_text(doc: &Value) {
    println!("# Corpus Stats");
    println!();
    println!("Requirements: {}", doc["total"]);
    println!();

    let facets = &doc["facets"];
    let print_hist = |label: &str, v: &Value| {
        println!("## {label}");
        match v.as_object() {
            Some(m) if !m.is_empty() => {
                let mut pairs: Vec<(&String, u64)> =
                    m.iter().map(|(k, x)| (k, x.as_u64().unwrap_or(0))).collect();
                pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
                for (k, n) in pairs {
                    println!("  {k}: {n}");
                }
            }
            _ => println!("  (none)"),
        }
        println!();
    };
    for f in ["status", "reqDomain", "silLevel", "asilLevel", "package", "tags"] {
        if let Some(v) = facets.get(f) {
            print_hist(f, v);
        }
    }
    if let Some(bp) = facets.get("byPackage") {
        println!("## byPackage");
        if let Some(m) = bp.as_object() {
            for (pkg, h) in m {
                println!("  {pkg}:");
                if let Some(hm) = h.as_object() {
                    for (k, n) in hm {
                        println!("    {k}: {n}");
                    }
                }
            }
        }
        println!();
    }

    let cov = &doc["coverage"];
    println!("## Coverage");
    println!(
        "  verified: {}  unverifiedLeaves: {}  parentsMissingIntegration: {}",
        cov["verified"], cov["unverifiedLeaves"], cov["parentsMissingIntegration"]
    );
    println!();

    let orph = &doc["orphans"];
    println!("## Orphans");
    println!(
        "  unverified: {}  unsatisfied: {}  untraced: {}",
        orph["unverifiedRequirements"], orph["unsatisfiedRequirements"], orph["untraced"]
    );
}

/// Resolve the optional `--config` projection lens, validate the resulting view,
/// and compute the stats document. Returns `Err(msg)` for an unresolvable
/// `--config` or an unknown `--group-by` facet. Shared by the CLI command and the
/// MCP `stats` tool so both return byte-identical JSON.
pub fn stats_document(
    elements: &[RawElement],
    vcfg: &syscribe_model::config::ValidateConfig,
    config: Option<&str>,
    opts: &StatsOptions,
) -> Result<Value, String> {
    use syscribe_model::projection::{project, resolve_selection, SelectionOutcome};

    let projected: Option<Vec<RawElement>> = match config {
        None => None,
        Some(c) => match resolve_selection(elements, c) {
            SelectionOutcome::Dormant => None, // no feature model → whole-model view
            SelectionOutcome::Resolved(sel) => Some(project(elements, &sel)),
            SelectionOutcome::Error(m) => return Err(m),
        },
    };
    let view: &[RawElement] = projected.as_deref().unwrap_or(elements);
    let result = syscribe_model::validator::validate_with_config(view, vcfg);
    compute_stats(view, &result, vcfg.results.as_ref(), opts)
}

/// The `stats` command. Computes the digest, prints it (text or `--json`), and
/// returns the process exit code (0 on success, 1 on a usage error such as an
/// unknown `--group-by` facet or an unresolvable `--config`).
pub fn cmd_stats(
    elements: &[RawElement],
    vcfg: &syscribe_model::config::ValidateConfig,
    config: Option<&str>,
    opts: &StatsOptions,
    json: bool,
) -> i32 {
    let doc = match stats_document(elements, vcfg, config, opts) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("Error: {msg}");
            return 1;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
    } else {
        print_text(&doc);
    }
    0
}

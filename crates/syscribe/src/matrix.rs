//! `syscribe matrix` — feature-model-driven Requirement × Configuration coverage
//! grid (REQ-TRS-VAR-004).
//!
//! Rows are requirements; columns are the model's `Configuration` elements. Each
//! cell classifies the (requirement, configuration) pair as:
//!
//! * `na` — the requirement's `appliesWhen:` is not satisfied by the configuration's
//!   selections (it does not exist in this variant);
//! * `covered` — the requirement is active and a non-draft TestCase that runs in this
//!   configuration verifies it;
//! * `gap` — the requirement is active but no such TestCase exists.
//!
//! When the variability dimension is dormant (REQ-TRS-VAR-001) the command
//! prints a notice and falls back to a flat requirement/testcase view.

use serde_json::json;
use std::collections::BTreeMap;
use syscribe_model::{
    element::{ElementType, RawElement},
    results::ResultsData,
    variability::{self, FeatureExpr},
};

use crate::export::SCHEMA_VERSION;
use crate::query::{tc_verdict, TcVerdict};

fn is_type(e: &RawElement, t: ElementType) -> bool {
    e.frontmatter.element_type.as_ref() == Some(&t)
}

/// Display id: stable `id:` when present, else qualified name.
fn disp_id(e: &RawElement) -> String {
    e.frontmatter
        .id
        .clone()
        .unwrap_or_else(|| e.qualified_name.clone())
}

/// Cross-reference identity keys an inbound `verifies:` entry may use.
fn keys(e: &RawElement) -> Vec<String> {
    let mut k = vec![e.qualified_name.clone()];
    if let Some(id) = &e.frontmatter.id {
        k.push(id.clone());
    }
    k
}

fn has_tag(e: &RawElement, tag: &str) -> bool {
    e.frontmatter
        .tags
        .as_ref()
        .is_some_and(|ts| ts.iter().any(|t| t == tag))
}

pub fn cmd_matrix(
    elements: &[RawElement],
    json: bool,
    tag: Option<&str>,
    status: Option<&str>,
    gaps_only: bool,
    results: Option<&ResultsData>,
    linked_only: bool,
) {
    cmd_matrix_inner(elements, json, tag, status, gaps_only, false, results, linked_only)
}

/// `matrix --features`: Feature × Configuration selection grid. Rows are
/// `FeatureDef`s, columns are `Configuration`s; a cell is `✓` where the feature
/// is selected `true` in that configuration.
pub fn cmd_matrix_features(elements: &[RawElement], json: bool) {
    if !variability::is_active(elements) {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": SCHEMA_VERSION,
                    "note": "no feature model present",
                    "columns": Vec::<String>::new(),
                    "rows": Vec::<serde_json::Value>::new(),
                }))
                .unwrap()
            );
        } else {
            println!("No feature model present — no feature × configuration matrix.");
        }
        return;
    }

    let mut configs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::Configuration))
        .collect();
    configs.sort_by_key(|e| disp_id(e));
    // Selections keyed by a FeatureDef's FEAT-* id are normalized to the qname so
    // an id-keyed config selects the same feature as a qname-keyed one (REQ-TRS-ID-006).
    let feat_alias = variability::feature_id_to_qname(elements);
    let cfg_sel: Vec<(String, BTreeMap<String, bool>)> = configs
        .iter()
        .map(|c| (disp_id(c), variability::canon_selection(&c.frontmatter.feature_selections(), &feat_alias)))
        .collect();

    let mut feats: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::FeatureDef))
        .collect();
    feats.sort_by_key(|e| e.qualified_name.clone());

    let selected = |sel: &BTreeMap<String, bool>, q: &str| sel.get(q).copied().unwrap_or(false);

    if json {
        let columns: Vec<String> = cfg_sel.iter().map(|(c, _)| c.clone()).collect();
        let rows: Vec<_> = feats
            .iter()
            .map(|fd| {
                let mut cells = serde_json::Map::new();
                for (cid, sel) in &cfg_sel {
                    cells.insert(cid.clone(), json!(selected(sel, &fd.qualified_name)));
                }
                json!({ "feature": fd.qualified_name, "cells": cells })
            })
            .collect();
        let doc = json!({
            "schemaVersion": SCHEMA_VERSION,
            "columns": columns,
            "rows": rows,
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return;
    }

    println!(
        "# Feature Matrix ({} features × {} configurations)",
        feats.len(),
        cfg_sel.len()
    );
    println!();
    print!("| Feature |");
    for (cid, _) in &cfg_sel {
        print!(" {} |", cid);
    }
    println!();
    print!("|---|");
    for _ in &cfg_sel {
        print!("---|");
    }
    println!();
    for fd in &feats {
        print!("| {} |", fd.qualified_name);
        for (_, sel) in &cfg_sel {
            print!(" {} |", if selected(sel, &fd.qualified_name) { "✓" } else { "" });
        }
        println!();
    }
    println!();
    println!("Legend: ✓ selected");
}

/// Classify one (requirement, configuration-selection) cell as
/// `"na" | "covered" | "passing" | "gap"`. Single definition shared by the
/// `matrix` grid and the `audit` coverage rollup (REQ-TRS-OUT-013).
///
/// `pkg` is the transitive package appliesWhen map; `tcs` are the non-draft
/// TestCases as `(effective appliesWhen, verifies, verdict)`; `evidence` is the
/// executed-results sidecar (None ⇒ no `"passing"` upgrade).
fn cell_state(
    r: &RawElement,
    sel: &BTreeMap<String, bool>,
    pkg: &std::collections::HashMap<String, serde_yaml::Value>,
    alias: &std::collections::HashMap<String, String>,
    tcs: &[(Option<FeatureExpr>, Vec<String>, TcVerdict, bool)],
    evidence: Option<&ResultsData>,
) -> &'static str {
    let selected = |q: &str| sel.get(q).copied().unwrap_or(false);
    let rexpr = variability::effective_expr_canon(r, pkg, alias);
    let active = rexpr.as_ref().is_none_or(|e| e.eval(&selected));
    if !active {
        return "na";
    }
    let rkeys = keys(r);
    let mut nondraft_covered = false;
    let mut draft_covered = false;
    let mut any_pass = false;
    let mut any_fail = false;
    for (texpr, ver, verdict, is_draft) in tcs {
        let runs = texpr.as_ref().is_none_or(|e| e.eval(&selected));
        if runs && ver.iter().any(|v| rkeys.iter().any(|k| k == v)) {
            if *is_draft {
                draft_covered = true;
            } else {
                nondraft_covered = true;
                match verdict {
                    TcVerdict::Pass => any_pass = true,
                    TcVerdict::Fail => any_fail = true,
                    _ => {}
                }
            }
        }
    }
    // A non-draft TestCase is real coverage; a draft-only link is `planned` intent.
    if nondraft_covered {
        if evidence.is_some() && any_pass {
            "passing"
        } else if evidence.is_some() && any_fail {
            "failing"
        } else {
            "covered"
        }
    } else if draft_covered {
        "planned"
    } else {
        "gap"
    }
}

/// TestCases that participate in coverage, as `(effective appliesWhen, verifies,
/// verdict)`. Non-draft TestCases always participate; a *draft* TestCase joins
/// only once executed evidence shows it passing (issue #21) — without a results
/// sidecar a draft TestCase is excluded, matching the linked-coverage view.
/// Every TestCase as `(effective appliesWhen, verifies, verdict, is_draft)`. Unlike
/// before, draft TestCases are retained (flagged) so the classifier can report a
/// draft-only-linked requirement as `planned` rather than dropping the link.
fn participating_tcs(
    elements: &[RawElement],
    pkg: &std::collections::HashMap<String, serde_yaml::Value>,
    feat_alias: &std::collections::HashMap<String, String>,
    evidence: Option<&ResultsData>,
) -> Vec<(Option<FeatureExpr>, Vec<String>, TcVerdict, bool)> {
    elements
        .iter()
        .filter(|e| is_type(e, ElementType::TestCase))
        .map(|e| {
            (
                variability::effective_expr_canon(e, pkg, feat_alias),
                e.frontmatter.verifies.clone().unwrap_or_default(),
                tc_verdict(e, evidence),
                e.frontmatter.status.as_deref() == Some("draft"),
            )
        })
        .collect()
}

/// Compute the active (feature-model) coverage grid: the configuration columns,
/// the per-requirement cell states (honouring tag/status/`--gaps-only` filters),
/// and the coverage rollup. Shared by the text and JSON renderers.
#[allow(clippy::type_complexity)]
fn active_grid(
    elements: &[RawElement],
    tag: Option<&str>,
    status: Option<&str>,
    gaps_only: bool,
    evidence: Option<&ResultsData>,
) -> (
    Vec<(String, BTreeMap<String, bool>)>,
    Vec<(String, Vec<&'static str>)>,
    Coverage,
) {
    // Columns: the model's Configuration elements, sorted by id.
    let mut configs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::Configuration))
        .collect();
    configs.sort_by_key(|e| disp_id(e));
    let feat_alias = variability::feature_id_to_qname(elements);
    let cfg_sel: Vec<(String, BTreeMap<String, bool>)> = configs
        .iter()
        .map(|c| (disp_id(c), variability::canon_selection(&c.frontmatter.feature_selections(), &feat_alias)))
        .collect();

    // Rows: requirements (optionally tag- and status-filtered), sorted by id.
    let mut reqs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::Requirement))
        .filter(|e| tag.is_none_or(|t| has_tag(e, t)))
        .filter(|e| status.is_none_or(|s| e.frontmatter.status.as_deref() == Some(s)))
        .collect();
    reqs.sort_by_key(|e| disp_id(e));

    // Effective conditions honour transitive package appliesWhen (REQ-TRS-VAR-006).
    let pkg = variability::package_conditions(elements);
    let tcs = participating_tcs(elements, &pkg, &feat_alias, evidence);

    // Materialise each retained row's cell states once: (id, [state per column]).
    // `--gaps-only` keeps only rows that have at least one `gap` cell (dropping
    // fully-covered and all-N/A rows).
    let grid: Vec<(String, Vec<&'static str>)> = reqs
        .iter()
        .map(|r| {
            let cells: Vec<&'static str> = cfg_sel
                .iter()
                .map(|(_, sel)| cell_state(r, sel, &pkg, &feat_alias, &tcs, evidence))
                .collect();
            (disp_id(r), cells)
        })
        .filter(|(_, cells)| !gaps_only || cells.contains(&"gap"))
        .collect();

    let coverage = Coverage::compute(&cfg_sel, &grid);
    (cfg_sel, grid, coverage)
}

/// The flat (no-feature-model) `matrix --json` document. Shared by the CLI
/// dormant path and `matrix_json`, so the JSON is identical either way.
fn dormant_json(elements: &[RawElement], status: Option<&str>, gaps_only: bool) -> serde_json::Value {
    let mut reqs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::Requirement))
        .filter(|e| status.is_none_or(|s| e.frontmatter.status.as_deref() == Some(s)))
        .collect();
    reqs.sort_by_key(|e| disp_id(e));

    let tcs: Vec<Vec<String>> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::TestCase))
        .filter(|e| e.frontmatter.status.as_deref() != Some("draft"))
        .map(|e| e.frontmatter.verifies.clone().unwrap_or_default())
        .collect();
    let verified_by = |r: &RawElement| -> bool {
        let rkeys = keys(r);
        tcs.iter().any(|ver| ver.iter().any(|v| rkeys.iter().any(|k| k == v)))
    };
    let grid: Vec<(String, bool)> = reqs
        .iter()
        .map(|r| (disp_id(r), verified_by(r)))
        .filter(|(_, cov)| !gaps_only || !*cov)
        .collect();
    let covered = grid.iter().filter(|(_, c)| *c).count() as u32;
    let applicable = grid.len() as u32;
    let pct = Coverage::pct(covered, applicable);
    let rows: Vec<_> = grid
        .iter()
        .map(|(id, cov)| json!({ "id": id, "verified": cov }))
        .collect();
    json!({
        "schemaVersion": SCHEMA_VERSION,
        "note": "no feature model present — flat coverage view",
        "columns": Vec::<String>::new(),
        "rows": rows,
        "coverage": json!({
            "perConfig": serde_json::Map::new(),
            "overall": { "covered": covered, "applicable": applicable, "pct": pct },
        }),
    })
}

/// The `matrix --json` document for an arbitrary model: the Requirement ×
/// Configuration grid plus the coverage rollup, or the flat fallback when no
/// feature model is present. The single value producer shared by the CLI
/// `matrix --json` output and the MCP `coverage_matrix` tool.
pub fn matrix_json(
    elements: &[RawElement],
    tag: Option<&str>,
    status: Option<&str>,
    gaps_only: bool,
    results: Option<&ResultsData>,
    linked_only: bool,
) -> serde_json::Value {
    let evidence = if linked_only { None } else { results };
    if !variability::is_active(elements) {
        return dormant_json(elements, status, gaps_only);
    }
    let (cfg_sel, grid, coverage) = active_grid(elements, tag, status, gaps_only, evidence);
    let columns: Vec<String> = cfg_sel.iter().map(|(c, _)| c.clone()).collect();
    let rows: Vec<_> = grid
        .iter()
        .map(|(id, cells)| {
            let mut cell_map = serde_json::Map::new();
            for ((cid, _), s) in cfg_sel.iter().zip(cells) {
                cell_map.insert(cid.clone(), json!(*s));
            }
            json!({ "id": id, "cells": cell_map })
        })
        .collect();
    json!({
        "schemaVersion": SCHEMA_VERSION,
        "columns": columns,
        "rows": rows,
        "coverage": coverage.json(),
    })
}

/// Collapse the coverage grid to one verdict per requirement (keyed by `disp_id`):
/// `"verified"` | `"planned"` | `"unverified"` | `"na"`. This is the row-collapse
/// (AND across applicable configurations) of the same per-cell classifier that
/// `coverage_matrix` displays, so `coverage` cannot contradict the matrix.
pub fn requirement_rollup(
    elements: &[RawElement],
    results: Option<&ResultsData>,
) -> std::collections::HashMap<String, &'static str> {
    let alias = variability::feature_id_to_qname(elements);
    let pkg = variability::package_conditions(elements);
    let tcs = participating_tcs(elements, &pkg, &alias, results);
    // Project over the model's Configurations; a flat model collapses over a
    // single empty selection (every requirement active, no projection).
    let cfgs: Vec<BTreeMap<String, bool>> = if variability::is_active(elements) {
        elements
            .iter()
            .filter(|e| is_type(e, ElementType::Configuration))
            .map(|c| variability::canon_selection(&c.frontmatter.feature_selections(), &alias))
            .collect()
    } else {
        vec![BTreeMap::new()]
    };
    let mut out = std::collections::HashMap::new();
    for r in elements.iter().filter(|e| is_type(e, ElementType::Requirement)) {
        let cells: Vec<&'static str> = cfgs
            .iter()
            .map(|sel| cell_state(r, sel, &pkg, &alias, &tcs, results))
            .collect();
        out.insert(disp_id(r), rollup_cells(&cells));
    }
    out
}

/// AND row-collapse of a requirement's cells over the configurations it applies to.
fn rollup_cells(cells: &[&'static str]) -> &'static str {
    let active: Vec<&str> = cells.iter().copied().filter(|c| *c != "na").collect();
    if active.is_empty() {
        return "na";
    }
    if active.iter().any(|c| *c == "gap" || *c == "failing") {
        return "unverified";
    }
    if active.contains(&"planned") {
        return "planned";
    }
    "verified"
}

#[allow(clippy::too_many_arguments)]
fn cmd_matrix_inner(
    elements: &[RawElement],
    json: bool,
    tag: Option<&str>,
    status: Option<&str>,
    gaps_only: bool,
    _features: bool,
    results: Option<&ResultsData>,
    linked_only: bool,
) {
    // Executed-evidence refinement applies only when a results sidecar is loaded
    // and the caller did not force the linked-only view (issue #21).
    let evidence = if linked_only { None } else { results };
    if !variability::is_active(elements) {
        cmd_matrix_dormant(elements, json, status, gaps_only);
        return;
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&matrix_json(
                elements, tag, status, gaps_only, results, linked_only
            ))
            .unwrap()
        );
        return;
    }

    let (cfg_sel, grid, coverage) = active_grid(elements, tag, status, gaps_only, evidence);

    // Text grid. "passing" → ✓ (covered + passing evidence); "covered" → ✓ when
    // no executed evidence is in play, else ▣ (covered but not passing).
    let glyph = |s: &str| match s {
        "passing" => "✓",
        "covered" => {
            if evidence.is_some() {
                "▣"
            } else {
                "✓"
            }
        }
        "planned" => "▢",
        // Covered by a non-draft TestCase whose latest verdict is fail: the
        // requirement IS covered, just not passing — same ▣ glyph as an
        // evidence-less "covered" cell, and visually distinct from a true gap.
        "failing" => "▣",
        "gap" => "✗",
        _ => "—",
    };
    println!("# Coverage Matrix ({} requirements × {} configurations)", grid.len(), cfg_sel.len());
    println!();
    print!("| Requirement |");
    for (cid, _) in &cfg_sel {
        print!(" {} |", cid);
    }
    println!();
    print!("|---|");
    for _ in &cfg_sel {
        print!("---|");
    }
    println!();
    for (id, cells) in &grid {
        print!("| {} |", id);
        for s in cells {
            print!(" {} |", glyph(s));
        }
        println!();
    }
    println!();
    if evidence.is_some() {
        println!("Legend: ✓ covered, passing · ▣ covered, not passing · ✗ gap · — N/A");
    } else {
        println!("Legend: ✓ covered · ✗ gap · — N/A");
    }
    println!();
    coverage.print_footer();
}

/// Plain (unweighted) coverage rollup: per-configuration and overall
/// `covered / applicable`, where `applicable = covered + gap` (N/A excluded).
/// Follow-up: SIL-weighted coverage is intentionally not implemented here.
///
/// This is the single coverage definition shared by `matrix` and `audit`
/// (REQ-TRS-OUT-013): build it via [`Coverage::rollup`] rather than duplicating
/// the covered/gap/N-A computation.
pub struct Coverage {
    per_config: Vec<(String, u32, u32)>, // (cfg id, covered, applicable)
    overall_covered: u32,
    overall_applicable: u32,
    /// True when no feature model is present and this is the flat
    /// requirement→TestCase coverage view (a single synthetic column).
    flat: bool,
}

impl Coverage {
    fn compute(
        cfg_sel: &[(String, BTreeMap<String, bool>)],
        grid: &[(String, Vec<&'static str>)],
    ) -> Coverage {
        let mut per_config: Vec<(String, u32, u32)> = Vec::with_capacity(cfg_sel.len());
        let (mut overall_covered, mut overall_applicable) = (0u32, 0u32);
        for (col, (cid, _)) in cfg_sel.iter().enumerate() {
            let (mut covered, mut applicable) = (0u32, 0u32);
            for (_, cells) in grid {
                match cells[col] {
                    // Coverage counts linked coverage (issue #21 leaves the
                    // footer semantics unchanged): both "covered" and "passing"
                    // are covered/applicable.
                    "covered" | "passing" => {
                        covered += 1;
                        applicable += 1;
                    }
                    // Applicable but not (yet) covered.
                    "gap" | "planned" | "failing" => applicable += 1,
                    _ => {}
                }
            }
            overall_covered += covered;
            overall_applicable += applicable;
            per_config.push((cid.clone(), covered, applicable));
        }
        Coverage { per_config, overall_covered, overall_applicable, flat: false }
    }

    /// Coverage rollup for an arbitrary model, reusing the exact `matrix`
    /// computation (REQ-TRS-OUT-013). When a feature model is active this is the
    /// per-`Configuration` covered/gap/N-A grid; otherwise it is the flat
    /// requirement→TestCase view (one synthetic `overall` column). `results` /
    /// `linked_only` mirror the `matrix` evidence semantics (issue #21) but do
    /// not affect the covered/applicable counts.
    pub fn rollup(
        elements: &[RawElement],
        results: Option<&ResultsData>,
        linked_only: bool,
    ) -> Coverage {
        let evidence = if linked_only { None } else { results };
        if !variability::is_active(elements) {
            return Self::rollup_flat(elements);
        }

        let mut configs: Vec<&RawElement> = elements
            .iter()
            .filter(|e| is_type(e, ElementType::Configuration))
            .collect();
        configs.sort_by_key(|e| disp_id(e));
        let feat_alias = variability::feature_id_to_qname(elements);
        let cfg_sel: Vec<(String, BTreeMap<String, bool>)> = configs
            .iter()
            .map(|c| (disp_id(c), variability::canon_selection(&c.frontmatter.feature_selections(), &feat_alias)))
            .collect();

        let mut reqs: Vec<&RawElement> = elements
            .iter()
            .filter(|e| is_type(e, ElementType::Requirement))
            .collect();
        reqs.sort_by_key(|e| disp_id(e));

        let pkg = variability::package_conditions(elements);
        let tcs = participating_tcs(elements, &pkg, &feat_alias, evidence);

        let grid: Vec<(String, Vec<&'static str>)> = reqs
            .iter()
            .map(|r| {
                let cells: Vec<&'static str> = cfg_sel
                    .iter()
                    .map(|(_, sel)| cell_state(r, sel, &pkg, &feat_alias, &tcs, evidence))
                    .collect();
                (disp_id(r), cells)
            })
            .collect();

        Self::compute(&cfg_sel, &grid)
    }

    /// Flat fallback rollup (no feature model): every requirement is applicable;
    /// covered when a non-draft TestCase verifies it. A single synthetic column.
    fn rollup_flat(elements: &[RawElement]) -> Coverage {
        let reqs: Vec<&RawElement> = elements
            .iter()
            .filter(|e| is_type(e, ElementType::Requirement))
            .collect();
        let tcs: Vec<Vec<String>> = elements
            .iter()
            .filter(|e| is_type(e, ElementType::TestCase))
            .filter(|e| e.frontmatter.status.as_deref() != Some("draft"))
            .map(|e| e.frontmatter.verifies.clone().unwrap_or_default())
            .collect();
        let mut covered = 0u32;
        let mut applicable = 0u32;
        for r in &reqs {
            applicable += 1;
            let rkeys = keys(r);
            if tcs.iter().any(|ver| ver.iter().any(|v| rkeys.iter().any(|k| k == v))) {
                covered += 1;
            }
        }
        Coverage {
            per_config: Vec::new(),
            overall_covered: covered,
            overall_applicable: applicable,
            flat: true,
        }
    }

    /// True when this is the flat (no-feature-model) coverage view.
    pub fn is_flat(&self) -> bool {
        self.flat
    }

    /// Per-configuration rows: `(configuration id, covered, applicable)`.
    pub fn per_config(&self) -> &[(String, u32, u32)] {
        &self.per_config
    }

    /// Overall `(covered, applicable)` across all applicable cells.
    pub fn overall(&self) -> (u32, u32) {
        (self.overall_covered, self.overall_applicable)
    }

    /// Public JSON view of the rollup (shared shape with `matrix --json`).
    pub fn json(&self) -> serde_json::Value {
        self.to_json()
    }

    /// Percentage helper exposed for reuse: `covered*100/applicable` to one
    /// decimal, or `None` when nothing is applicable.
    pub fn percent(covered: u32, applicable: u32) -> Option<f64> {
        Self::pct(covered, applicable)
    }

    /// pct = covered*100/applicable rounded to one decimal, or None when applicable == 0.
    fn pct(covered: u32, applicable: u32) -> Option<f64> {
        if applicable == 0 {
            None
        } else {
            Some((f64::from(covered) * 1000.0 / f64::from(applicable)).round() / 10.0)
        }
    }

    fn to_json(&self) -> serde_json::Value {
        let mut per = serde_json::Map::new();
        for (cid, covered, applicable) in &self.per_config {
            per.insert(
                cid.clone(),
                json!({
                    "covered": covered,
                    "applicable": applicable,
                    "pct": Self::pct(*covered, *applicable),
                }),
            );
        }
        json!({
            "perConfig": per,
            "overall": {
                "covered": self.overall_covered,
                "applicable": self.overall_applicable,
                "pct": Self::pct(self.overall_covered, self.overall_applicable),
            },
        })
    }

    fn fmt_pct(p: Option<f64>) -> String {
        p.map_or_else(|| "n/a".to_string(), |v| format!("{v:.1}%"))
    }

    fn print_footer(&self) {
        println!("Coverage (covered / applicable; N/A excluded):");
        for (cid, covered, applicable) in &self.per_config {
            println!(
                "  {cid}: {covered}/{applicable} ({})",
                Self::fmt_pct(Self::pct(*covered, *applicable))
            );
        }
        println!(
            "  Overall: {}/{} ({})",
            self.overall_covered,
            self.overall_applicable,
            Self::fmt_pct(Self::pct(self.overall_covered, self.overall_applicable))
        );
    }
}

/// Dormant fallback: no feature model is linked, so emit the flat
/// requirement→testcase coverage view with a clear notice.
fn cmd_matrix_dormant(
    elements: &[RawElement],
    json: bool,
    status: Option<&str>,
    gaps_only: bool,
) {
    let mut reqs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::Requirement))
        .filter(|e| status.is_none_or(|s| e.frontmatter.status.as_deref() == Some(s)))
        .collect();
    reqs.sort_by_key(|e| disp_id(e));

    let tcs: Vec<Vec<String>> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::TestCase))
        .filter(|e| e.frontmatter.status.as_deref() != Some("draft"))
        .map(|e| e.frontmatter.verifies.clone().unwrap_or_default())
        .collect();

    let verified_by = |r: &RawElement| -> bool {
        let rkeys = keys(r);
        tcs.iter()
            .any(|ver| ver.iter().any(|v| rkeys.iter().any(|k| k == v)))
    };

    // Flat view: every requirement is "applicable" (no N/A dimension). A row is
    // covered or a gap; `--gaps-only` keeps only the gap rows.
    let grid: Vec<(String, bool)> = reqs
        .iter()
        .map(|r| (disp_id(r), verified_by(r)))
        .filter(|(_, cov)| !gaps_only || !*cov)
        .collect();

    let covered = grid.iter().filter(|(_, c)| *c).count() as u32;
    let applicable = grid.len() as u32;
    let pct = Coverage::pct(covered, applicable);

    if json {
        println!("{}", serde_json::to_string_pretty(&dormant_json(elements, status, gaps_only)).unwrap());
        return;
    }

    println!("No feature model present — falling back to flat requirement/testcase coverage.");
    println!();
    println!("| Requirement | Covered |");
    println!("|---|---|");
    for (id, cov) in &grid {
        println!("| {} | {} |", id, if *cov { "✓" } else { "✗" });
    }
    println!();
    println!("Coverage (covered / applicable):");
    println!("  Overall: {covered}/{applicable} ({})", Coverage::fmt_pct(pct));
}

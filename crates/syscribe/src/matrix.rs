//! `syscribe matrix` — feature-model-driven Requirement × Configuration coverage
//! grid (REQ-TRS-VAR-004).
//!
//! Rows are requirements; columns are the model's `Configuration` elements. Each
//! cell classifies the (requirement, configuration) pair as:
//!   * `na`      — the requirement's `appliesWhen:` is not satisfied by the
//!                 configuration's selections (it does not exist in this variant);
//!   * `covered` — the requirement is active and a non-draft TestCase that runs
//!                 in this configuration verifies it;
//!   * `gap`     — the requirement is active but no such TestCase exists.
//!
//! When the variability dimension is dormant (REQ-TRS-VAR-001) the command
//! prints a notice and falls back to a flat requirement/testcase view.

use serde_json::json;
use std::collections::BTreeMap;
use syscribe_model::{
    element::{ElementType, RawElement},
    variability::{self, FeatureExpr},
};

use crate::export::SCHEMA_VERSION;

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
) {
    cmd_matrix_inner(elements, json, tag, status, gaps_only, false)
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
    let cfg_sel: Vec<(String, BTreeMap<String, bool>)> = configs
        .iter()
        .map(|c| (disp_id(c), c.frontmatter.feature_selections()))
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

fn cmd_matrix_inner(
    elements: &[RawElement],
    json: bool,
    tag: Option<&str>,
    status: Option<&str>,
    gaps_only: bool,
    _features: bool,
) {
    if !variability::is_active(elements) {
        cmd_matrix_dormant(elements, json, status, gaps_only);
        return;
    }

    // Columns: the model's Configuration elements, sorted by id.
    let mut configs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::Configuration))
        .collect();
    configs.sort_by_key(|e| disp_id(e));
    let cfg_sel: Vec<(String, BTreeMap<String, bool>)> = configs
        .iter()
        .map(|c| (disp_id(c), c.frontmatter.feature_selections()))
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

    // Non-draft TestCases that participate in coverage: (appliesWhen, verifies).
    let tcs: Vec<(Option<FeatureExpr>, Vec<String>)> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::TestCase))
        .filter(|e| e.frontmatter.status.as_deref() != Some("draft"))
        .map(|e| (variability::effective_expr(e, &pkg), e.frontmatter.verifies.clone().unwrap_or_default()))
        .collect();

    // state(req, config selections) -> "na" | "covered" | "gap"
    let state = |r: &RawElement, sel: &BTreeMap<String, bool>| -> &'static str {
        let selected = |q: &str| sel.get(q).copied().unwrap_or(false);
        let rexpr = variability::effective_expr(r, &pkg);
        let active = rexpr.as_ref().map_or(true, |e| e.eval(&selected));
        if !active {
            return "na";
        }
        let rkeys = keys(r);
        let covered = tcs.iter().any(|(texpr, ver)| {
            let runs = texpr.as_ref().map_or(true, |e| e.eval(&selected));
            runs && ver.iter().any(|v| rkeys.iter().any(|k| k == v))
        });
        if covered {
            "covered"
        } else {
            "gap"
        }
    };

    // Materialise each retained row's cell states once: (id, [state per column]).
    // `--gaps-only` keeps only rows that have at least one `gap` cell (dropping
    // fully-covered and all-N/A rows).
    let grid: Vec<(String, Vec<&'static str>)> = reqs
        .iter()
        .map(|r| {
            let cells: Vec<&'static str> = cfg_sel.iter().map(|(_, sel)| state(r, sel)).collect();
            (disp_id(r), cells)
        })
        .filter(|(_, cells)| !gaps_only || cells.iter().any(|c| *c == "gap"))
        .collect();

    // Coverage rollup: covered / applicable, applicable = covered + gap (N/A
    // excluded). Computed over the retained rows. Plain (unweighted) coverage
    // only — SIL-weighted coverage is a deliberate follow-up (out of scope here).
    let coverage = Coverage::compute(&cfg_sel, &grid);

    if json {
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
        let doc = json!({
            "schemaVersion": SCHEMA_VERSION,
            "columns": columns,
            "rows": rows,
            "coverage": coverage.to_json(),
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return;
    }

    // Text grid.
    let glyph = |s: &str| match s {
        "covered" => "✓",
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
    println!("Legend: ✓ covered · ✗ gap · — N/A");
    println!();
    coverage.print_footer();
}

/// Plain (unweighted) coverage rollup: per-configuration and overall
/// `covered / applicable`, where `applicable = covered + gap` (N/A excluded).
/// Follow-up: SIL-weighted coverage is intentionally not implemented here.
struct Coverage {
    per_config: Vec<(String, u32, u32)>, // (cfg id, covered, applicable)
    overall_covered: u32,
    overall_applicable: u32,
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
                    "covered" => {
                        covered += 1;
                        applicable += 1;
                    }
                    "gap" => applicable += 1,
                    _ => {}
                }
            }
            overall_covered += covered;
            overall_applicable += applicable;
            per_config.push((cid.clone(), covered, applicable));
        }
        Coverage { per_config, overall_covered, overall_applicable }
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
        let rows: Vec<_> = grid
            .iter()
            .map(|(id, cov)| json!({ "id": id, "verified": cov }))
            .collect();
        let doc = json!({
            "schemaVersion": SCHEMA_VERSION,
            "note": "no feature model present — flat coverage view",
            "columns": Vec::<String>::new(),
            "rows": rows,
            "coverage": json!({
                "perConfig": serde_json::Map::new(),
                "overall": { "covered": covered, "applicable": applicable, "pct": pct },
            }),
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
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

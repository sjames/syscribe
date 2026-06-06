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

fn parse_aw(e: &RawElement) -> Option<FeatureExpr> {
    e.frontmatter
        .applies_when
        .as_ref()
        .and_then(|aw| variability::applies_when_expr(aw).ok().flatten())
}

fn has_tag(e: &RawElement, tag: &str) -> bool {
    e.frontmatter
        .tags
        .as_ref()
        .is_some_and(|ts| ts.iter().any(|t| t == tag))
}

pub fn cmd_matrix(elements: &[RawElement], json: bool, tag: Option<&str>) {
    if !variability::is_active(elements) {
        cmd_matrix_dormant(elements, json);
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

    // Rows: requirements (optionally tag-filtered), sorted by id.
    let mut reqs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::Requirement))
        .filter(|e| tag.is_none_or(|t| has_tag(e, t)))
        .collect();
    reqs.sort_by_key(|e| disp_id(e));

    // Non-draft TestCases that participate in coverage: (appliesWhen, verifies).
    let tcs: Vec<(Option<FeatureExpr>, Vec<String>)> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::TestCase))
        .filter(|e| e.frontmatter.status.as_deref() != Some("draft"))
        .map(|e| (parse_aw(e), e.frontmatter.verifies.clone().unwrap_or_default()))
        .collect();

    // state(req, config selections) -> "na" | "covered" | "gap"
    let state = |r: &RawElement, sel: &BTreeMap<String, bool>| -> &'static str {
        let selected = |q: &str| sel.get(q).copied().unwrap_or(false);
        let rexpr = parse_aw(r);
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

    if json {
        let columns: Vec<String> = cfg_sel.iter().map(|(c, _)| c.clone()).collect();
        let rows: Vec<_> = reqs
            .iter()
            .map(|r| {
                let mut cells = serde_json::Map::new();
                for (cid, sel) in &cfg_sel {
                    cells.insert(cid.clone(), json!(state(r, sel)));
                }
                json!({ "id": disp_id(r), "cells": cells })
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

    // Text grid.
    let glyph = |s: &str| match s {
        "covered" => "✓",
        "gap" => "✗",
        _ => "—",
    };
    println!("# Coverage Matrix ({} requirements × {} configurations)", reqs.len(), cfg_sel.len());
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
    for r in &reqs {
        print!("| {} |", disp_id(r));
        for (_, sel) in &cfg_sel {
            print!(" {} |", glyph(state(r, sel)));
        }
        println!();
    }
    println!();
    println!("Legend: ✓ covered · ✗ gap · — N/A");
}

/// Dormant fallback: no feature model is linked, so emit the flat
/// requirement→testcase coverage view with a clear notice.
fn cmd_matrix_dormant(elements: &[RawElement], json: bool) {
    let mut reqs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_type(e, ElementType::Requirement))
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

    if json {
        let rows: Vec<_> = reqs
            .iter()
            .map(|r| json!({ "id": disp_id(r), "verified": verified_by(r) }))
            .collect();
        let doc = json!({
            "schemaVersion": SCHEMA_VERSION,
            "note": "no feature model present — flat coverage view",
            "columns": Vec::<String>::new(),
            "rows": rows,
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return;
    }

    println!("No feature model present — falling back to flat requirement/testcase coverage.");
    println!();
    println!("| Requirement | Covered |");
    println!("|---|---|");
    for r in &reqs {
        println!("| {} | {} |", disp_id(r), if verified_by(r) { "✓" } else { "✗" });
    }
}

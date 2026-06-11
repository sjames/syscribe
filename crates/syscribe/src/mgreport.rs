//! MagicGrid read-only report commands (REQ-TRS-MG-003/006/007):
//!
//!   * `magicgrid [--json]`   — bucket every element by its `mg_cell` overlay into
//!                              the B/W/S × 1-4 grid and render it.
//!   * `matrix --allocations` — source × target allocation matrix (a mode of the
//!                              existing `matrix` command, handled here).
//!   * `trade-study [--json] [--config …]` — MoE-weighted trade study scoring the
//!                              model's `Configuration`s against its `mg_moe` MoEs.
//!
//! All three are read-only and available regardless of profile; the gated
//! validation (MG020/MG021) lives in the validator, not here.

use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use syscribe_model::{
    element::{ElementType, RawElement},
    feature_model,
};

use crate::export::SCHEMA_VERSION;

/// Human label for an element: `name` for name-identified types, `title` for
/// id-identified types, falling back to the last qualified-name segment.
fn label(e: &RawElement) -> String {
    e.frontmatter
        .name
        .clone()
        .or_else(|| e.frontmatter.title.clone())
        .unwrap_or_else(|| e.qualified_name.rsplit("::").next().unwrap_or("—").to_string())
}

fn is_type(e: &RawElement, t: ElementType) -> bool {
    e.frontmatter.element_type.as_ref() == Some(&t)
}

// ── magicgrid: B/W/S × 1-4 cell report (REQ-TRS-MG-003) ──────────────────────

const ROWS: [char; 3] = ['B', 'W', 'S'];
const COLS: [u8; 4] = [1, 2, 3, 4];
const COL_NAMES: [&str; 4] = ["Requirements", "Behavior", "Structure", "Parameters"];

/// Parse an `mg_cell` value into a `(row, col)` coordinate iff it is one of the
/// recognised `B1`-`B4`/`W1`-`W4`/`S1`-`S4` set.
fn parse_cell(v: &str) -> Option<(char, u8)> {
    let v = v.trim();
    let mut chars = v.chars();
    let row = chars.next()?.to_ascii_uppercase();
    let rest: String = chars.collect();
    let col: u8 = rest.parse().ok()?;
    if ROWS.contains(&row) && COLS.contains(&col) {
        Some((row, col))
    } else {
        None
    }
}

pub fn cmd_magicgrid(elems: &[RawElement], json: bool) {
    // Bucket element labels by coordinate "R<col>".
    let mut grid: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in ROWS {
        for c in COLS {
            grid.insert(format!("{}{}", r, c), Vec::new());
        }
    }
    for e in elems {
        if let Some(raw) = e.frontmatter.mg_str("mg_cell") {
            if let Some((r, c)) = parse_cell(&raw) {
                grid.entry(format!("{}{}", r, c)).or_default().push(label(e));
            }
        }
    }
    for v in grid.values_mut() {
        v.sort();
    }
    let empty: usize = grid.values().filter(|v| v.is_empty()).count();

    if json {
        let mut cells = serde_json::Map::new();
        for (k, v) in &grid {
            cells.insert(k.clone(), json!(v));
        }
        let out = json!({
            "schemaVersion": SCHEMA_VERSION,
            "report": "magicgrid",
            "rows": ROWS.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
            "columns": COLS.iter().map(|c| *c as u64).collect::<Vec<_>>(),
            "cells": cells,
            "emptyCells": empty,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return;
    }

    println!("MagicGrid — element classification (rows B/W/S × columns 1-4)");
    println!();
    let row_label = |r: char| match r {
        'B' => "B (problem / black-box)",
        'W' => "W (problem / white-box)",
        'S' => "S (solution)",
        _ => "?",
    };
    for r in ROWS {
        println!("{}", row_label(r));
        for (i, c) in COLS.iter().enumerate() {
            let coord = format!("{}{}", r, c);
            let names = &grid[&coord];
            let header = format!("  {} — col {} {} [{}]", coord, c, COL_NAMES[i], names.len());
            if names.is_empty() {
                println!("{}  (empty)", header);
            } else {
                println!("{}", header);
                for n in names {
                    println!("    - {}", n);
                }
            }
        }
        println!();
    }
    println!(
        "Summary: {} empty cell{} (a hint to model-completeness, not an error).",
        empty,
        if empty == 1 { "" } else { "s" }
    );
}

// ── matrix --allocations: source × target allocation matrix (REQ-TRS-MG-006) ─

/// One allocation edge: source qname → target qname.
struct AllocEdge {
    from: String,
    to: String,
}

/// Collect every allocation edge from `Allocation` elements: from each
/// `features:` entry's `allocatedFrom`/`allocatedTo`, and from the element's own
/// top-level `allocatedFrom`/`allocatedTo` lists (cartesian product).
fn alloc_edges(elems: &[RawElement]) -> Vec<AllocEdge> {
    let mut edges = Vec::new();
    for e in elems {
        if !is_type(e, ElementType::Allocation) {
            continue;
        }
        if let Some(ref feats) = e.frontmatter.features {
            for fv in feats {
                if let serde_yaml::Value::Mapping(m) = fv {
                    let from = m
                        .get(serde_yaml::Value::String("allocatedFrom".into()))
                        .and_then(|v| v.as_str());
                    let to = m
                        .get(serde_yaml::Value::String("allocatedTo".into()))
                        .and_then(|v| v.as_str());
                    if let (Some(f), Some(t)) = (from, to) {
                        edges.push(AllocEdge { from: f.to_string(), to: t.to_string() });
                    }
                }
            }
        }
        if let (Some(froms), Some(tos)) =
            (&e.frontmatter.allocated_from, &e.frontmatter.allocated_to)
        {
            for f in froms {
                for t in tos {
                    edges.push(AllocEdge { from: f.clone(), to: t.clone() });
                }
            }
        }
    }
    edges
}

/// Resolve an allocation endpoint qname to its display label (last segment if no
/// matching element is found).
fn endpoint_label(elems: &[RawElement], qn: &str) -> String {
    elems
        .iter()
        .find(|e| e.qualified_name == qn || e.qualified_name.ends_with(&format!("::{}", qn)))
        .map(label)
        .unwrap_or_else(|| qn.rsplit("::").next().unwrap_or(qn).to_string())
}

pub fn cmd_matrix_allocations(elems: &[RawElement], json: bool) {
    let edges = alloc_edges(elems);

    // Ordered, de-duplicated source and target labels.
    let mut sources: Vec<String> = Vec::new();
    let mut targets: Vec<String> = Vec::new();
    let mut allocated: BTreeSet<(String, String)> = BTreeSet::new();
    for e in &edges {
        let s = endpoint_label(elems, &e.from);
        let t = endpoint_label(elems, &e.to);
        if !sources.contains(&s) {
            sources.push(s.clone());
        }
        if !targets.contains(&t) {
            targets.push(t.clone());
        }
        allocated.insert((s, t));
    }
    sources.sort();
    targets.sort();

    // Layer overlay: any element carrying mg_layer activates the logical→physical
    // partition view.
    let has_layers = elems.iter().any(|e| e.frontmatter.mg_str("mg_layer").is_some());
    let logical: Vec<String> = elems
        .iter()
        .filter(|e| e.frontmatter.mg_str("mg_layer").as_deref() == Some("logical"))
        .map(label)
        .collect();
    let physical: Vec<String> = elems
        .iter()
        .filter(|e| e.frontmatter.mg_str("mg_layer").as_deref() == Some("physical"))
        .map(label)
        .collect();

    // Rollup: sources with no allocation, targets never allocated to. Computed
    // over every Allocation-eligible endpoint plus standalone structural parts.
    let unallocated_sources: Vec<String> = source_universe(elems)
        .into_iter()
        .filter(|s| !sources.contains(s))
        .collect();
    let unused_targets: Vec<String> = target_universe(elems)
        .into_iter()
        .filter(|t| !targets.contains(t) && !allocated.iter().any(|(_, tt)| tt == t))
        .collect();

    if json {
        let cells: Vec<_> = allocated
            .iter()
            .map(|(s, t)| json!({ "source": s, "target": t }))
            .collect();
        let mut out = json!({
            "schemaVersion": SCHEMA_VERSION,
            "report": "allocation-matrix",
            "sources": sources,
            "targets": targets,
            "cells": cells,
            "rollup": {
                "unallocatedSources": unallocated_sources,
                "unusedTargets": unused_targets,
            },
        });
        if has_layers {
            out["partition"] = json!({
                "logical": logical,
                "physical": physical,
            });
        }
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return;
    }

    println!("Allocation matrix (sources × targets)");
    println!();
    print_alloc_grid(&sources, &targets, &allocated);

    if has_layers {
        println!();
        println!("Logical → physical partition (mg_layer):");
        println!();
        let mut log_alloc: BTreeSet<(String, String)> = BTreeSet::new();
        for (s, t) in &allocated {
            if logical.contains(s) && physical.contains(t) {
                log_alloc.insert((s.clone(), t.clone()));
            }
        }
        print_alloc_grid(&logical, &physical, &log_alloc);
    }

    println!();
    println!("Rollup:");
    if unallocated_sources.is_empty() {
        println!("  Unallocated sources: (none)");
    } else {
        println!("  Unallocated sources: {}", unallocated_sources.join(", "));
    }
    if unused_targets.is_empty() {
        println!("  Unused targets: (none)");
    } else {
        println!("  Unused targets: {}", unused_targets.join(", "));
    }
}

/// Sources a flat allocation matrix could draw from: every behaviour element
/// (`Action`/`ActionDef`/etc.) plus anything appearing as an `allocatedFrom`.
fn source_universe(elems: &[RawElement]) -> Vec<String> {
    let mut v: Vec<String> = elems
        .iter()
        .filter(|e| {
            matches!(
                e.frontmatter.element_type,
                Some(ElementType::ActionDef)
                    | Some(ElementType::Action)
                    | Some(ElementType::UseCaseDef)
                    | Some(ElementType::UseCase)
                    | Some(ElementType::StateDef)
                    | Some(ElementType::State)
            ) || e.frontmatter.mg_str("mg_layer").as_deref() == Some("logical")
        })
        .map(label)
        .collect();
    v.sort();
    v.dedup();
    v
}

/// Targets a flat allocation matrix could draw from: every structural part plus
/// anything tagged `mg_layer: physical`. A part tagged `mg_layer: logical` is a
/// source, never a target, so it is excluded.
fn target_universe(elems: &[RawElement]) -> Vec<String> {
    let mut v: Vec<String> = elems
        .iter()
        .filter(|e| {
            let is_part = matches!(
                e.frontmatter.element_type,
                Some(ElementType::PartDef) | Some(ElementType::Part)
            );
            let layer = e.frontmatter.mg_str("mg_layer");
            (is_part && layer.as_deref() != Some("logical"))
                || layer.as_deref() == Some("physical")
        })
        .map(label)
        .collect();
    v.sort();
    v.dedup();
    v
}

fn print_alloc_grid(
    sources: &[String],
    targets: &[String],
    allocated: &BTreeSet<(String, String)>,
) {
    if sources.is_empty() || targets.is_empty() {
        println!("  (no allocation edges to display)");
        return;
    }
    let row_w = sources.iter().map(|s| s.len()).max().unwrap_or(6).max(6);
    // Header
    print!("  {:width$} ", "", width = row_w);
    for t in targets {
        print!("| {} ", t);
    }
    println!();
    for s in sources {
        print!("  {:width$} ", s, width = row_w);
        for t in targets {
            let mark = if allocated.contains(&(s.clone(), t.clone())) {
                "✓"
            } else {
                "·"
            };
            let pad = t.len();
            print!("| {:^width$} ", mark, width = pad);
        }
        println!();
    }
}

// ── trade-study: MoE-weighted configuration scoring (REQ-TRS-MG-007) ─────────

struct Moe {
    name: String,
    expression: String,
    direction: String, // "maximize" | "minimize"
    threshold: f64,
    objective: f64,
    weight: f64,
}

struct Cfg {
    label: String,
    bindings: BTreeMap<String, f64>,
    id: Option<String>,
    qname: String,
}

/// One evaluated cell.
struct Cell {
    value: Option<f64>,
    score: f64,
    violation: bool,
}

/// Resolve a variable token against a configuration's bindings: match on the
/// final `.`/`::` segment of a binding key, else an exact key match.
fn resolve_binding(bindings: &BTreeMap<String, f64>, var: &str) -> Option<f64> {
    if let Some(v) = bindings.get(var) {
        return Some(*v);
    }
    for (k, v) in bindings {
        let seg = k.rsplit(|c| c == '.' || c == ':').next().unwrap_or(k);
        if seg == var {
            return Some(*v);
        }
    }
    None
}

fn score_moe(moe: &Moe, value: f64) -> (f64, bool) {
    let clamp = |x: f64| x.clamp(0.0, 1.0);
    if moe.direction.eq_ignore_ascii_case("minimize") {
        let violation = value > moe.threshold;
        let denom = moe.threshold - moe.objective;
        let s = if denom == 0.0 { 0.0 } else { (moe.threshold - value) / denom };
        (clamp(s), violation)
    } else {
        let violation = value < moe.threshold;
        let denom = moe.objective - moe.threshold;
        let s = if denom == 0.0 { 0.0 } else { (value - moe.threshold) / denom };
        (clamp(s), violation)
    }
}

pub fn cmd_trade_study(elems: &[RawElement], json: bool, config_filter: &[String]) {
    // Rows: mg_moe == true.
    let moes: Vec<Moe> = elems
        .iter()
        .filter(|e| e.frontmatter.mg_bool("mg_moe") == Some(true))
        .map(|e| Moe {
            name: label(e),
            expression: e.frontmatter.expression.clone().unwrap_or_default(),
            direction: e
                .frontmatter
                .mg_str("mg_moe_direction")
                .unwrap_or_else(|| "maximize".to_string()),
            threshold: e.frontmatter.mg_f64("mg_moe_threshold").unwrap_or(0.0),
            objective: e.frontmatter.mg_f64("mg_moe_objective").unwrap_or(1.0),
            weight: e.frontmatter.mg_f64("mg_moe_weight").unwrap_or(1.0),
        })
        .collect();

    // Columns: Configuration elements, optionally restricted by --config.
    let mut cfgs: Vec<Cfg> = Vec::new();
    for e in elems {
        if !is_type(e, ElementType::Configuration) {
            continue;
        }
        if !config_filter.is_empty() {
            let matches = config_filter.iter().any(|c| {
                e.frontmatter.id.as_deref() == Some(c.as_str()) || e.qualified_name == *c
            });
            if !matches {
                continue;
            }
        }
        let mut bindings = BTreeMap::new();
        if let Some(serde_yaml::Value::Mapping(m)) = &e.frontmatter.parameter_bindings {
            for (k, v) in m {
                if let (Some(ks), Some(vn)) = (k.as_str(), v.as_f64()) {
                    bindings.insert(ks.to_string(), vn);
                }
            }
        }
        cfgs.push(Cfg {
            label: e
                .frontmatter
                .title
                .clone()
                .or_else(|| e.frontmatter.name.clone())
                .or_else(|| e.frontmatter.id.clone())
                .unwrap_or_else(|| e.qualified_name.clone()),
            bindings,
            id: e.frontmatter.id.clone(),
            qname: e.qualified_name.clone(),
        });
    }

    // Evaluate every (moe, cfg) cell.
    let mut cells: Vec<Vec<Cell>> = Vec::with_capacity(moes.len());
    for moe in &moes {
        let mut row = Vec::with_capacity(cfgs.len());
        for cfg in &cfgs {
            let resolve = |tok: &str| resolve_binding(&cfg.bindings, tok);
            let value = feature_model::eval_expr_rhs(&moe.expression, &resolve);
            match value {
                Some(v) => {
                    let (score, violation) = score_moe(moe, v);
                    row.push(Cell { value: Some(v), score, violation });
                }
                None => row.push(Cell { value: None, score: 0.0, violation: false }),
            }
        }
        cells.push(row);
    }

    // Per-config weighted total over evaluable rows (weights renormalised to 1).
    let mut totals: Vec<f64> = vec![0.0; cfgs.len()];
    let mut failing: Vec<bool> = vec![false; cfgs.len()];
    for (ci, _cfg) in cfgs.iter().enumerate() {
        let mut wsum = 0.0;
        for (ri, moe) in moes.iter().enumerate() {
            let cell = &cells[ri][ci];
            if cell.value.is_some() {
                wsum += moe.weight;
            }
            if cell.violation {
                failing[ci] = true;
            }
        }
        if wsum > 0.0 {
            let mut total = 0.0;
            for (ri, moe) in moes.iter().enumerate() {
                let cell = &cells[ri][ci];
                if cell.value.is_some() {
                    total += (moe.weight / wsum) * cell.score;
                }
            }
            totals[ci] = total;
        }
    }

    // Winner: highest-scoring non-failing configuration.
    let winner: Option<usize> = cfgs
        .iter()
        .enumerate()
        .filter(|(i, _)| !failing[*i])
        .max_by(|(ia, _), (ib, _)| {
            totals[*ia].partial_cmp(&totals[*ib]).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i);

    if json {
        let moe_json: Vec<_> = moes
            .iter()
            .map(|m| {
                json!({
                    "name": m.name,
                    "direction": m.direction,
                    "threshold": m.threshold,
                    "objective": m.objective,
                    "weight": m.weight,
                })
            })
            .collect();
        let cfg_json: Vec<_> = cfgs
            .iter()
            .map(|c| json!({ "label": c.label, "id": c.id, "qualifiedName": c.qname }))
            .collect();
        let mut cell_json = Vec::new();
        for (ri, moe) in moes.iter().enumerate() {
            for (ci, cfg) in cfgs.iter().enumerate() {
                let cell = &cells[ri][ci];
                cell_json.push(json!({
                    "moe": moe.name,
                    "configuration": cfg.label,
                    "value": cell.value,
                    "score": if cell.value.is_some() { Some(cell.score) } else { None },
                    "weightedContribution": if cell.value.is_some() {
                        Some(moe.weight * cell.score)
                    } else {
                        None
                    },
                    "thresholdViolation": cell.violation,
                    "evaluable": cell.value.is_some(),
                }));
            }
        }
        let rollup: Vec<_> = cfgs
            .iter()
            .enumerate()
            .map(|(i, c)| {
                json!({
                    "configuration": c.label,
                    "weightedTotal": totals[i],
                    "fail": failing[i],
                    "winner": winner == Some(i),
                })
            })
            .collect();
        let out = json!({
            "schemaVersion": SCHEMA_VERSION,
            "report": "trade-study",
            "moes": moe_json,
            "configurations": cfg_json,
            "cells": cell_json,
            "rollup": rollup,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return;
    }

    println!("Trade study — MoEs × Configurations");
    println!();
    // Grid: one row per MoE, one column per configuration.
    let row_w = moes.iter().map(|m| m.name.len()).max().unwrap_or(8).max(8);
    let col_w = cfgs.iter().map(|c| c.label.len()).max().unwrap_or(12).max(12);
    print!("  {:width$} ", "MoE", width = row_w);
    for c in &cfgs {
        print!("| {:^cw$} ", c.label, cw = col_w);
    }
    println!();
    for (ri, moe) in moes.iter().enumerate() {
        print!("  {:width$} ", moe.name, width = row_w);
        for ci in 0..cfgs.len() {
            let cell = &cells[ri][ci];
            let text = match cell.value {
                None => "n/a".to_string(),
                Some(v) => {
                    let mut s = format!("{} ({:.2})", fmt_num(v), cell.score);
                    if cell.violation {
                        s.push_str(" !");
                    }
                    s
                }
            };
            print!("| {:^cw$} ", text, cw = col_w);
        }
        println!();
    }

    println!();
    println!("Rollup (weighted total per configuration):");
    for (i, c) in cfgs.iter().enumerate() {
        let mut tags = String::new();
        if winner == Some(i) {
            tags.push_str(" WINNER");
        }
        if failing[i] {
            tags.push_str(" FAIL");
        }
        println!("  {:<width$} {:.3}{}", c.label, totals[i], tags, width = col_w);
    }
}

/// Format a numeric value without a trailing `.0` for integers.
fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

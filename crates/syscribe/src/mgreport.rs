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
    resolver::Resolver,
    validator::{allocation_edges, Finding, Severity, ValidationResult},
};

use crate::diagram::layout::metrics::load_metrics;
use crate::diagram::layout::theme::theme_for;
use crate::export::SCHEMA_VERSION;
use svg::node::element::{Group, Rectangle, Text, Title};
use svg::Document;

/// Human label for an element: `name` on every type (REQ-TRS-NAME-002), falling
/// back to the last qualified-name segment.
fn label(e: &RawElement) -> String {
    e.frontmatter
        .name
        .clone()
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

    // REQ-TRS-MG-009 — identify the System of Interest (mg_soi: true) when exactly
    // one is present, so the B3 System-Context boundary is legible in the report.
    let sois: Vec<&RawElement> = elems
        .iter()
        .filter(|e| e.frontmatter.mg_bool("mg_soi") == Some(true))
        .collect();
    let soi: Option<String> = if sois.len() == 1 {
        Some(label(sois[0]))
    } else {
        None
    };

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
            "systemOfInterest": soi,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return;
    }

    println!("MagicGrid — element classification (rows B/W/S × columns 1-4)");
    println!();
    if let Some(ref s) = soi {
        // REQ-TRS-MG-009 — note the SoI alongside the B3 Structure cell.
        println!("System of interest: {} (B3)", s);
        println!();
    }
    // REQ-TRS-MG-015 — the 3x4 grid matrix (rows B/W/S × the four pillars). Each
    // cell shows its element count; the B3 System of Interest is marked with ◆ and
    // empty cells with ·. The per-cell element detail follows below.
    println!(
        "## Grid\n\n| Row | 1 {} | 2 {} | 3 {} | 4 {} |",
        COL_NAMES[0], COL_NAMES[1], COL_NAMES[2], COL_NAMES[3]
    );
    println!("|---|---|---|---|---|");
    for r in ROWS {
        let mut cells = Vec::with_capacity(4);
        for c in COLS {
            let coord = format!("{}{}", r, c);
            let n = grid[&coord].len();
            let mut cell = n.to_string();
            if coord == "B3" && soi.is_some() {
                cell.push_str(" ◆");
            }
            if n == 0 {
                cell.push_str(" ·");
            }
            cells.push(cell);
        }
        let rl = match r {
            'B' => "**B** black-box",
            'W' => "**W** white-box",
            'S' => "**S** solution",
            _ => "?",
        };
        println!("| {} | {} | {} | {} | {} |", rl, cells[0], cells[1], cells[2], cells[3]);
    }
    println!();
    println!(
        "◆ = System of Interest{} · · = empty cell · {}/12 cells populated",
        soi.as_ref().map(|s| format!(" ({}, B3)", s)).unwrap_or_default(),
        12 - empty
    );
    println!();
    println!("## Detail");
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

// ── magicgrid --svg: standalone SVG of the grid, companion-ready (REQ-TRS-MG-016) ─────

/// `magicgrid --svg [-o <file>]` — render the 3×4 grid as a self-contained SVG.
pub fn cmd_magicgrid_svg(elems: &[RawElement], output_file: Option<&str>) {
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
    let sois: Vec<&RawElement> = elems
        .iter()
        .filter(|e| e.frontmatter.mg_bool("mg_soi") == Some(true))
        .collect();
    let soi: Option<String> = if sois.len() == 1 {
        Some(label(sois[0]))
    } else {
        None
    };

    let svg = render_grid_svg(&grid, soi.as_deref());
    match output_file {
        Some(path) => {
            if let Err(e) = std::fs::write(path, &svg) {
                eprintln!("error writing output file '{}': {}", path, e);
                std::process::exit(1);
            }
        }
        None => println!("{}", svg),
    }
}

/// Hard-break a single word that is wider than `max_w` into character chunks.
fn hard_break(word: &str, max_w: f64, fs: f64, m: &dyn crate::diagram::layout::metrics::TextMetrics) -> Vec<String> {
    if m.advance_width(word, fs, false) <= max_w {
        return vec![word.to_string()];
    }
    let mut out = Vec::new();
    let mut chunk = String::new();
    for ch in word.chars() {
        let t = format!("{}{}", chunk, ch);
        if !chunk.is_empty() && m.advance_width(&t, fs, false) > max_w {
            out.push(std::mem::take(&mut chunk));
            chunk = ch.to_string();
        } else {
            chunk = t;
        }
    }
    if !chunk.is_empty() {
        out.push(chunk);
    }
    out
}

/// Greedy word-wrap `s` to `max_w` pixels at font size `fs`, measured with the shared
/// font metrics (REQ-TRS-MG-016). Breaks on word boundaries; a single over-wide word is
/// hard-broken by character.
fn wrap_text(s: &str, max_w: f64, fs: f64, m: &dyn crate::diagram::layout::metrics::TextMetrics) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        for piece in hard_break(word, max_w, fs, m) {
            let cand = if cur.is_empty() { piece.clone() } else { format!("{} {}", cur, piece) };
            if cur.is_empty() || m.advance_width(&cand, fs, false) <= max_w {
                cur = cand;
            } else {
                lines.push(std::mem::take(&mut cur));
                cur = piece;
            }
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// One laid-out cell of the grid.
enum CellKind {
    /// Empty corner (top-left).
    Corner,
    /// Pillar header: bold pillar name.
    Header { text: String, fg: &'static str, bg: &'static str, border: &'static str },
    /// Row label (B/W/S description).
    Label { text: String },
    /// A grid cell: badge + wrapped content lines.
    Body { badge: String, lines: Vec<(String, &'static str)>, fill: &'static str, border: &'static str, sw: u8 },
}

fn render_grid_svg(grid: &BTreeMap<String, Vec<String>>, soi: Option<&str>) -> String {
    use taffy::prelude::*;

    let m = load_metrics();
    let pillar_type = [
        ElementType::Requirement,
        ElementType::UseCase,
        ElementType::PartDef,
        ElementType::ConstraintDef,
    ];
    let themes: Vec<_> = pillar_type.iter().map(theme_for).collect();

    let hdr_fs = 13.0;
    let body_fs = 11.0;
    let badge_fs = 10.0;
    let title_fs = 16.0;
    let line_h = m.line_height(body_fs).round();
    let pad = 8.0;

    // Fixed pillar column width (fits the header; labels wrap into it).
    let mut col_w = 0f64;
    for (ci, name) in COL_NAMES.iter().enumerate() {
        col_w = col_w.max(m.advance_width(&format!("{} {}", ci + 1, name), hdr_fs, true));
    }
    col_w = (col_w + pad * 2.0).clamp(180.0, 240.0).ceil();
    let text_w = col_w - pad * 2.0 - 4.0;

    let row_desc = |r: char| match r {
        'B' => "B — black-box",
        'W' => "W — white-box",
        'S' => "S — solution",
        _ => "?",
    };
    let lbl_w = ROWS
        .iter()
        .map(|r| m.advance_width(row_desc(*r), hdr_fs, true))
        .fold(0.0_f64, f64::max)
        .ceil()
        + pad * 2.0;

    // Build the per-cell content (badge + wrapped lines) and its natural height.
    let body_cell = |coord: &str| -> (CellKind, f64) {
        let ci = (coord.as_bytes()[1] - b'1') as usize;
        let th = &themes[ci];
        let names = &grid[coord];
        let is_soi = coord == "B3" && soi.is_some();
        let (fill, border, sw) = if names.is_empty() {
            ("#f5f5f5", th.border, 1)
        } else if is_soi {
            ("#fff3cd", "#f9a825", 2)
        } else {
            (th.body_bg, th.border, 1)
        };
        let badge = format!("{}{} [{}]", coord, if is_soi { " ◆" } else { "" }, names.len());
        let mut lines: Vec<(String, &'static str)> = Vec::new();
        if names.is_empty() {
            lines.push(("(empty)".to_string(), "#bdbdbd"));
        } else {
            // Show every element; the cell (and its row) stretches to fit.
            for nm in names {
                for wl in wrap_text(nm, text_w, body_fs, m.as_ref()) {
                    lines.push((wl, th.body_fg));
                }
            }
        }
        let h = (pad + badge_fs + line_h * lines.len() as f64 + pad).max(64.0);
        (CellKind::Body { badge, lines, fill, border, sw }, h)
    };

    // Assemble the 4×5 cell grid (header row + 3 body rows; label col + 4 pillars).
    let hdr_h = 32.0;
    let mut rows_cells: Vec<Vec<(CellKind, f64, f64)>> = Vec::new(); // (kind, width, height)
    // header row
    let mut hdr_row = vec![(CellKind::Corner, lbl_w, hdr_h)];
    for (ci, name) in COL_NAMES.iter().enumerate() {
        let th = &themes[ci];
        hdr_row.push((
            CellKind::Header { text: format!("{} {}", ci + 1, name), fg: th.header_fg, bg: th.header_bg, border: th.border },
            col_w,
            hdr_h,
        ));
    }
    rows_cells.push(hdr_row);
    // body rows
    for r in ROWS {
        let mut row = vec![(CellKind::Label { text: row_desc(r).to_string() }, lbl_w, line_h + pad * 2.0)];
        for c in COLS {
            let (kind, h) = body_cell(&format!("{}{}", r, c));
            row.push((kind, col_w, h));
        }
        rows_cells.push(row);
    }

    // ── taffy: a flex column of flex rows; fixed cell widths align the columns,
    //    align-items: stretch makes each row as tall as its tallest cell. ──────────
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let mut row_nodes = Vec::new();
    let mut cell_nodes: Vec<Vec<_>> = Vec::new();
    for row in &rows_cells {
        let mut cells = Vec::new();
        for (_, w, h) in row {
            let leaf = tree
                .new_leaf(Style {
                    size: Size { width: length(*w as f32), height: auto() },
                    min_size: Size { width: auto(), height: length(*h as f32) },
                    ..Default::default()
                })
                .unwrap();
            cells.push(leaf);
        }
        let row_node = tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: Some(AlignItems::STRETCH),
                    ..Default::default()
                },
                &cells,
            )
            .unwrap();
        row_nodes.push(row_node);
        cell_nodes.push(cells);
    }
    let root = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: Some(AlignItems::STRETCH),
                ..Default::default()
            },
            &row_nodes,
        )
        .unwrap();
    tree.compute_layout(
        root,
        Size { width: AvailableSpace::MaxContent, height: AvailableSpace::MaxContent },
    )
    .unwrap();

    let root_layout = *tree.layout(root).unwrap();
    let margin = 20.0;
    let title_h = 32.0;
    let origin_x = margin;
    let origin_y = margin + title_h;
    let total_w = margin * 2.0 + root_layout.size.width as f64;
    let total_h = origin_y + root_layout.size.height as f64 + margin;

    let title = soi
        .map(|s| format!("MagicGrid — System of Interest: {}", s))
        .unwrap_or_else(|| "MagicGrid".to_string());
    let mut doc = Document::new()
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("width", total_w.ceil())
        .set("height", total_h.ceil())
        .set("viewBox", format!("0 0 {:.0} {:.0}", total_w.ceil(), total_h.ceil()))
        .set("font-family", "'Segoe UI', system-ui, sans-serif")
        .add(Title::new(title.clone()))
        .add(
            Rectangle::new()
                .set("x", 0)
                .set("y", 0)
                .set("width", total_w.ceil())
                .set("height", total_h.ceil())
                .set("fill", "#ffffff"),
        )
        .add(
            Text::new(title)
                .set("x", margin)
                .set("y", margin + title_fs)
                .set("font-size", title_fs)
                .set("font-weight", "bold")
                .set("fill", "#1a1a2e"),
        );

    for (ri, row) in rows_cells.iter().enumerate() {
        let row_loc = tree.layout(row_nodes[ri]).unwrap().location;
        for (ci, (kind, _, _)) in row.iter().enumerate() {
            let cl = tree.layout(cell_nodes[ri][ci]).unwrap();
            let x = origin_x + row_loc.x as f64 + cl.location.x as f64;
            let y = origin_y + row_loc.y as f64 + cl.location.y as f64;
            let w = cl.size.width as f64;
            let h = cl.size.height as f64;
            doc = doc.add(draw_cell(kind, x, y, w, h, pad, hdr_fs, body_fs, badge_fs, line_h));
        }
    }

    doc.to_string()
}

#[allow(clippy::too_many_arguments)]
fn draw_cell(
    kind: &CellKind,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    pad: f64,
    hdr_fs: f64,
    body_fs: f64,
    badge_fs: f64,
    line_h: f64,
) -> Group {
    let mut g = Group::new();
    match kind {
        CellKind::Corner => {
            g = g.add(rect(x, y, w, h, "#ffffff", "#cfd8dc", 1));
        }
        CellKind::Header { text, fg, bg, border } => {
            g = g
                .add(rect(x, y, w, h, bg, border, 1))
                .add(
                    Text::new(text.clone())
                        .set("x", x + pad)
                        .set("y", y + h * 0.66)
                        .set("font-size", hdr_fs)
                        .set("font-weight", "bold")
                        .set("fill", *fg),
                );
        }
        CellKind::Label { text } => {
            g = g
                .add(rect(x, y, w, h, "#eceff1", "#90a4ae", 1))
                .add(
                    Text::new(text.clone())
                        .set("x", x + pad)
                        .set("y", y + pad + hdr_fs)
                        .set("font-size", hdr_fs)
                        .set("font-weight", "bold")
                        .set("fill", "#37474f"),
                );
        }
        CellKind::Body { badge, lines, fill, border, sw } => {
            g = g
                .add(rect(x, y, w, h, fill, border, *sw as i32))
                .add(
                    Text::new(badge.clone())
                        .set("x", x + pad)
                        .set("y", y + pad + badge_fs)
                        .set("font-size", badge_fs)
                        .set("fill", "#78909c"),
                );
            for (i, (text, fillc)) in lines.iter().enumerate() {
                g = g.add(
                    Text::new(text.clone())
                        .set("x", x + pad + 4.0)
                        .set("y", y + pad + badge_fs + line_h * (i as f64 + 1.0))
                        .set("font-size", body_fs)
                        .set("fill", *fillc),
                );
            }
        }
    }
    g
}

fn rect(x: f64, y: f64, w: f64, h: f64, fill: &str, stroke: &str, sw: i32) -> Rectangle {
    Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", w)
        .set("height", h)
        .set("fill", fill)
        .set("stroke", stroke)
        .set("stroke-width", sw)
}

// ── magicgrid --audit: MagicGrid findings rollup + readiness + verdict (REQ-TRS-MG-013) ─

/// Category for a MagicGrid-relevant finding code, or `None` if not MagicGrid.
fn mg_category(code: &str) -> Option<&'static str> {
    match code {
        "MG020" | "MG021" => Some("Grid"),
        "E316" | "W307" => Some("Refines"),
        "MG010" | "MG011" | "MG012" | "MG013" => Some("Context"),
        "MG060" | "MG061" | "MG062" => Some("SoI"),
        "MG030" | "MG031" | "MG032" | "MG033" => Some("MoE"),
        "MG050" | "MG051" | "MG052" => Some("MoP"),
        "MG040" | "MG041" | "MG042" => Some("Layer"),
        "MG070" => Some("Variant"),
        "MG080" | "MG081" | "MG082" | "MG083" => Some("Coverage"),
        _ => None,
    }
}

/// `magicgrid --audit`: roll up the MagicGrid findings (collected with the gate
/// active), a readiness summary, and a PASS/FAIL verdict. Returns the process
/// exit code (0 PASS · 2 FAIL). REQ-TRS-MG-013.
pub fn cmd_magicgrid_audit(elems: &[RawElement], result: &ValidationResult, json: bool) -> i32 {
    let relevant: Vec<&Finding> = result
        .findings
        .iter()
        .filter(|f| mg_category(f.code).is_some())
        .collect();
    let errors: Vec<&Finding> =
        relevant.iter().copied().filter(|f| f.severity == Severity::Error).collect();
    let warnings: Vec<&Finding> =
        relevant.iter().copied().filter(|f| f.severity == Severity::Warning).collect();

    let mut by_code: BTreeMap<&str, usize> = BTreeMap::new();
    let mut by_cat: BTreeMap<&str, usize> = BTreeMap::new();
    for f in &relevant {
        *by_code.entry(f.code).or_default() += 1;
        *by_cat.entry(mg_category(f.code).unwrap()).or_default() += 1;
    }

    // Readiness — grid completeness (distinct populated coordinates of 12).
    let populated = {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for e in elems {
            if let Some(raw) = e.frontmatter.mg_str("mg_cell") {
                if let Some((r, c)) = parse_cell(&raw) {
                    seen.insert(format!("{}{}", r, c));
                }
            }
        }
        seen.len()
    };
    let total_cells = ROWS.len() * COLS.len();
    let empty_cells = total_cells - populated;

    // Readiness — System of Interest.
    let sois: Vec<&RawElement> = elems
        .iter()
        .filter(|e| e.frontmatter.mg_bool("mg_soi") == Some(true))
        .collect();
    let (soi_status, soi_label): (&str, Option<String>) = match sois.len() {
        0 => ("none", None),
        1 => ("unique", Some(label(sois[0]))),
        _ => ("ambiguous", None),
    };

    let moe_count = elems.iter().filter(|e| e.frontmatter.mg_bool("mg_moe") == Some(true)).count();
    let mop_count = elems.iter().filter(|e| e.frontmatter.mg_bool("mg_mop") == Some(true)).count();
    let cfg_count = elems.iter().filter(|e| is_type(e, ElementType::Configuration)).count();

    // Verdict — FAIL when the magicgrid gate would fail: any MagicGrid error, or a
    // promoted W307.
    let promoted_w307 = relevant.iter().any(|f| f.code == "W307");
    let fail = !errors.is_empty() || promoted_w307;
    let verdict = if fail { "FAIL" } else { "PASS" };
    let exit_code = if fail { 2 } else { 0 };

    if json {
        let findings: Vec<serde_json::Value> = relevant
            .iter()
            .map(|f| {
                json!({
                    "code": f.code,
                    "severity": match f.severity {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                        Severity::Info => "info",
                    },
                    "category": mg_category(f.code).unwrap(),
                    "file": f.file,
                    "message": f.message,
                })
            })
            .collect();
        let by_code_json: serde_json::Map<String, serde_json::Value> =
            by_code.iter().map(|(k, v)| (k.to_string(), json!(v))).collect();
        let by_cat_json: serde_json::Map<String, serde_json::Value> =
            by_cat.iter().map(|(k, v)| (k.to_string(), json!(v))).collect();
        let out = json!({
            "report": "magicgrid-audit",
            "schemaVersion": SCHEMA_VERSION,
            "errors": errors.len(),
            "warnings": warnings.len(),
            "byCode": by_code_json,
            "byCategory": by_cat_json,
            "findings": findings,
            "readiness": {
                "cellsPopulated": populated,
                "cellsEmpty": empty_cells,
                "systemOfInterest": soi_label,
                "soiStatus": soi_status,
                "moes": moe_count,
                "mops": mop_count,
                "configurations": cfg_count,
            },
            "verdict": verdict,
            "exitCode": exit_code,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return exit_code;
    }

    println!("# MagicGrid Audit (profile: magicgrid)");
    println!();
    println!("Errors: {} · Warnings: {}", errors.len(), warnings.len());
    println!();
    if relevant.is_empty() {
        println!("No MagicGrid findings.");
        println!();
    } else {
        println!("| Code | Count | Category |");
        println!("|---|---|---|");
        for (&code, count) in &by_code {
            println!("| {} | {} | {} |", code, count, mg_category(code).unwrap());
        }
        println!();
        if !errors.is_empty() {
            println!("## Errors");
            for f in &errors {
                println!("- {}  {}  {}", f.code, f.file, f.message);
            }
            println!();
        }
        if !warnings.is_empty() {
            println!("## Warnings");
            for f in &warnings {
                println!("- {}  {}  {}", f.code, f.file, f.message);
            }
            println!();
        }
    }

    println!("## Readiness");
    println!("- Grid: {}/{} cells populated ({} empty)", populated, total_cells, empty_cells);
    match soi_status {
        "unique" => println!("- System of interest: {}", soi_label.as_deref().unwrap_or("—")),
        "none" => println!("- System of interest: none (no mg_soi marker)"),
        _ => println!("- System of interest: ambiguous ({} marked)", sois.len()),
    }
    println!("- MoEs: {} · MoPs: {} · Configurations: {}", moe_count, mop_count, cfg_count);
    println!();
    if fail {
        let mut codes: Vec<&str> = errors.iter().map(|f| f.code).collect();
        if promoted_w307 {
            codes.push("W307");
        }
        codes.sort();
        codes.dedup();
        println!("## Verdict: FAIL — {}", codes.join(", "));
    } else {
        println!("## Verdict: PASS");
    }
    exit_code
}

// ── matrix --allocations: source × target allocation matrix (REQ-TRS-MG-006) ─

/// One allocation edge: source qname → target qname.
struct AllocEdge {
    from: String,
    to: String,
}

/// Collect every allocation edge from the shared, unified extractor
/// (REQ-TRS-ALLOC-001): both `allocatedTo`-on-source (form 1) and the standalone
/// `Allocation` element (form 2), resolved and de-duplicated, so the matrix and
/// the `MG041`/`MG081` gate consume identical edges.
fn alloc_edges(elems: &[RawElement]) -> Vec<AllocEdge> {
    let resolver = Resolver::new(elems);
    allocation_edges(elems, &resolver)
        .into_iter()
        .map(|(from, to)| AllocEdge { from, to })
        .collect()
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

/// Resolve a variable token against a configuration's bindings: an exact key
/// match wins; otherwise match on the final `.`/`::` segment of a binding key,
/// but only when exactly one binding's final segment matches. A bare token that
/// matches the final segment of two or more bindings is **ambiguous** and left
/// unresolved (the cell becomes `n/a`) rather than silently picking one
/// (REQ-TRS-MG-012).
fn resolve_binding(bindings: &BTreeMap<String, f64>, var: &str) -> Option<f64> {
    if let Some(v) = bindings.get(var) {
        return Some(*v);
    }
    let mut hit: Option<f64> = None;
    for (k, v) in bindings {
        let seg = k.rsplit(|c| c == '.' || c == ':').next().unwrap_or(k);
        if seg == var {
            if hit.is_some() {
                return None; // ambiguous final-segment match
            }
            hit = Some(*v);
        }
    }
    hit
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

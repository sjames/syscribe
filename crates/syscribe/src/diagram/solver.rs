/// Cassowary-based layout solver for the `diagram layout` subcommand.
///
/// Input:  a placement JSON with semantic `col`/`row` positions (no pixel arithmetic).
/// Output: a resolved layout JSON with absolute `x`/`y` that `diagram compose` accepts.
///
/// Constraint model:
///   - One Variable per column (col_x[c]) and per row (row_y[r]).
///   - REQUIRED: col_x[c+1] >= col_x[c] + max_width_in_col[c] + COL_GAP
///   - REQUIRED: row_y[r+1] >= row_y[r] + max_height_in_row[r] + ROW_GAP
///   - REQUIRED: col_x[0] >= 0, row_y[0] >= 0
///   - WEAK:     col_x[c] == ideal_x[c]  (prefer minimum-area packing)
///   - WEAK:     row_y[r] == ideal_y[r]
///   - STRONG:   pin_x / pin_y per element if specified
use cassowary::WeightedRelation::*;
use cassowary::strength::*;
use cassowary::{Solver, Variable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syscribe_model::element::RawElement;

use crate::diagram::layout::{
    build_element_node, load_metrics, render_element, ViewConfig,
};

// ── Layout constants ──────────────────────────────────────────────────────────
const COL_GAP: f64 = 32.0;
const ROW_GAP: f64 = 24.0;

// ── Input schema ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct PlacementFile {
    pub title: Option<String>,
    /// Diagram kind forwarded to compose: "ibd" | "bdd" | "arch"
    pub kind: Option<String>,
    pub canvas: Option<PlacementCanvas>,
    pub elements: Vec<ElementPlacement>,
    /// Passed through unchanged to the resolved layout JSON.
    pub edges: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlacementCanvas {
    pub padding: Option<f64>,
    pub bg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementPlacement {
    pub qname: String,
    pub col: u32,
    pub row: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pin_x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pin_y: Option<f64>,
}

// ── Output schema (compatible with compose's LayoutFile) ─────────────────────

#[derive(Debug, Serialize)]
pub struct ResolvedLayout {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub canvas: ResolvedCanvas,
    pub elements: Vec<ResolvedElement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edges: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ResolvedCanvas {
    pub padding: f64,
    pub bg: String,
}

#[derive(Debug, Serialize)]
pub struct ResolvedElement {
    pub qname: String,
    pub x: f64,
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<serde_json::Value>,
}

// ── CLI entry point ───────────────────────────────────────────────────────────

pub fn cmd_diagram_layout(
    elements: &[RawElement],
    placement_file: &str,
    output_file: Option<&str>,
    compose_after: bool,
    compose_kind: Option<&str>,
    compose_output: Option<&str>,
) {
    let content = if placement_file == "-" {
        use std::io::Read;
        let mut buf = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            eprintln!("error reading placement from stdin: {}", e);
            std::process::exit(1);
        }
        buf
    } else {
        match std::fs::read_to_string(placement_file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error reading placement file '{}': {}", placement_file, e);
                std::process::exit(1);
            }
        }
    };
    let placement: PlacementFile = match serde_json::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error parsing placement JSON: {}", e);
            std::process::exit(1);
        }
    };

    let resolved = solve_layout(elements, &placement);
    let json = serde_json::to_string_pretty(&resolved).unwrap();

    if compose_after {
        // Write resolved layout to a temp file, then pipe through compose.
        let tmp = output_file.unwrap_or("/tmp/_syscribe_resolved.layout.json");
        if let Err(e) = std::fs::write(tmp, &json) {
            eprintln!("error writing temp layout '{}': {}", tmp, e);
            std::process::exit(1);
        }
        let kind = compose_kind.or(placement.kind.as_deref()).unwrap_or("arch");
        let ibd = kind == "ibd";
        super::compose::cmd_diagram_compose(elements, tmp, compose_output, ibd);
    } else {
        match output_file {
            Some(path) => {
                if let Err(e) = std::fs::write(path, &json) {
                    eprintln!("error writing output '{}': {}", path, e);
                    std::process::exit(1);
                }
            }
            None => println!("{}", json),
        }
    }
}

// ── Solver ────────────────────────────────────────────────────────────────────

pub fn solve_layout(elements: &[RawElement], placement: &PlacementFile) -> ResolvedLayout {
    let padding = placement.canvas.as_ref().and_then(|c| c.padding).unwrap_or(40.0);
    let bg = placement
        .canvas
        .as_ref()
        .and_then(|c| c.bg.clone())
        .unwrap_or_else(|| "#ffffff".to_string());
    let ibd = placement.kind.as_deref() == Some("ibd");

    // ── Measure elements ──────────────────────────────────────────────────────
    let metrics = load_metrics();
    let mut dims: HashMap<String, (f64, f64)> = HashMap::new();

    for ep in &placement.elements {
        let view = view_from_json(ep.view.as_ref(), ibd);
        if let Some(elem) = elements.iter().find(|e| e.qualified_name == ep.qname) {
            let node = build_element_node(elem, &view);
            let r = render_element(&node, metrics.as_ref());
            dims.insert(ep.qname.clone(), (r.width, r.height));
        } else {
            eprintln!("warn: element '{}' not found in model, using default size", ep.qname);
            dims.insert(ep.qname.clone(), (160.0, 80.0));
        }
    }

    // ── Max width per column, max height per row ──────────────────────────────
    let max_col = placement.elements.iter().map(|e| e.col).max().unwrap_or(0);
    let max_row = placement.elements.iter().map(|e| e.row).max().unwrap_or(0);

    let mut col_max_w: HashMap<u32, f64> = HashMap::new();
    let mut row_max_h: HashMap<u32, f64> = HashMap::new();

    for ep in &placement.elements {
        let (w, h) = dims.get(&ep.qname).copied().unwrap_or((160.0, 80.0));
        col_max_w.entry(ep.col).and_modify(|e| *e = e.max(w)).or_insert(w);
        row_max_h.entry(ep.row).and_modify(|e| *e = e.max(h)).or_insert(h);
    }

    // ── Cassowary variables: one x per column, one y per row ─────────────────
    let col_vars: Vec<Variable> = (0..=max_col).map(|_| Variable::new()).collect();
    let row_vars: Vec<Variable> = (0..=max_row).map(|_| Variable::new()).collect();

    let mut solver = Solver::new();

    // Origin constraints (REQUIRED): columns and rows start at 0.
    solver.add_constraint(col_vars[0] | GE(REQUIRED) | 0.0).unwrap();
    solver.add_constraint(row_vars[0] | GE(REQUIRED) | 0.0).unwrap();

    // Column ordering (REQUIRED): col[c+1] >= col[c] + max_width[c] + COL_GAP
    for c in 0..max_col as usize {
        let w = col_max_w.get(&(c as u32)).copied().unwrap_or(160.0);
        solver
            .add_constraint(col_vars[c + 1] | GE(REQUIRED) | (col_vars[c] + w + COL_GAP))
            .unwrap();
    }

    // Row ordering (REQUIRED): row[r+1] >= row[r] + max_height[r] + ROW_GAP
    for r in 0..max_row as usize {
        let h = row_max_h.get(&(r as u32)).copied().unwrap_or(80.0);
        solver
            .add_constraint(row_vars[r + 1] | GE(REQUIRED) | (row_vars[r] + h + ROW_GAP))
            .unwrap();
    }

    // Preferred positions (WEAK): push towards compact, minimum-area packing.
    for c in 0..=max_col as usize {
        let ideal: f64 = (0..c)
            .map(|i| col_max_w.get(&(i as u32)).copied().unwrap_or(160.0) + COL_GAP)
            .sum();
        solver.add_constraint(col_vars[c] | EQ(WEAK) | ideal).unwrap();
    }
    for r in 0..=max_row as usize {
        let ideal: f64 = (0..r)
            .map(|i| row_max_h.get(&(i as u32)).copied().unwrap_or(80.0) + ROW_GAP)
            .sum();
        solver.add_constraint(row_vars[r] | EQ(WEAK) | ideal).unwrap();
    }

    // Pin overrides (STRONG): hard-fix a column's x or a row's y when requested.
    // Collects pins per column/row; last one wins if multiple elements in the same
    // col/row specify conflicting pins.
    let mut col_pins: HashMap<u32, f64> = HashMap::new();
    let mut row_pins: HashMap<u32, f64> = HashMap::new();
    for ep in &placement.elements {
        if let Some(px) = ep.pin_x { col_pins.insert(ep.col, px); }
        if let Some(py) = ep.pin_y { row_pins.insert(ep.row, py); }
    }
    for (c, px) in &col_pins {
        if let Some(&var) = col_vars.get(*c as usize) {
            solver.add_constraint(var | EQ(STRONG) | *px).unwrap_or(());
        }
    }
    for (r, py) in &row_pins {
        if let Some(&var) = row_vars.get(*r as usize) {
            solver.add_constraint(var | EQ(STRONG) | *py).unwrap_or(());
        }
    }

    // ── Fetch solved values ───────────────────────────────────────────────────
    let changes: HashMap<Variable, f64> = solver.fetch_changes().iter().copied().collect();

    let col_x: Vec<f64> = col_vars
        .iter()
        .enumerate()
        .map(|(c, &var)| {
            let fallback: f64 = (0..c)
                .map(|i| col_max_w.get(&(i as u32)).copied().unwrap_or(160.0) + COL_GAP)
                .sum();
            changes.get(&var).copied().unwrap_or(fallback)
        })
        .collect();

    let row_y: Vec<f64> = row_vars
        .iter()
        .enumerate()
        .map(|(r, &var)| {
            let fallback: f64 = (0..r)
                .map(|i| row_max_h.get(&(i as u32)).copied().unwrap_or(80.0) + ROW_GAP)
                .sum();
            changes.get(&var).copied().unwrap_or(fallback)
        })
        .collect();

    // ── Build resolved layout ─────────────────────────────────────────────────
    let resolved_elements: Vec<ResolvedElement> = placement
        .elements
        .iter()
        .map(|ep| ResolvedElement {
            qname: ep.qname.clone(),
            x: nonzero(col_x.get(ep.col as usize).copied().unwrap_or(0.0)),
            y: nonzero(row_y.get(ep.row as usize).copied().unwrap_or(0.0)),
            view: ep.view.clone(),
        })
        .collect();

    ResolvedLayout {
        title: placement.title.clone(),
        canvas: ResolvedCanvas { padding, bg },
        elements: resolved_elements,
        edges: placement.edges.clone(),
    }
}

/// Normalise -0.0 to 0.0 so JSON output is clean.
#[inline]
fn nonzero(x: f64) -> f64 {
    if x == 0.0 { 0.0 } else { x }
}

fn view_from_json(v: Option<&serde_json::Value>, ibd: bool) -> ViewConfig {
    let preset_str = v.and_then(|v| v.as_str()).unwrap_or("full");
    let mut vc = ViewConfig::from_preset(preset_str);
    vc.ibd = ibd;
    vc
}

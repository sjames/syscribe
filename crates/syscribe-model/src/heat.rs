//! Heat tables (GH #223): matrix / heat views for the safety and security
//! analyses.
//!
//! One small grid data type ([`HeatGrid`]: rows, columns, cells carrying element
//! ids, a [`Tone`] and a label) with three builders and three renderers:
//!
//! * [`fmea_grid`]  - FMEA severity x occurrence, cell tone from the S x O
//!   product, each placed entry coloured by its RPN band;
//! * [`hara_grid`]  - HARA S/E rows x C columns -> ASIL (ISO 26262-3 Table 4, via
//!   [`crate::asil::derive`]) with hazardous events and safety goals placed in the
//!   cell they derive;
//! * [`tara_grid`]  - TARA impact x feasibility risk matrix (the configured
//!   method of [`CyberConfig`]) with threat scenarios placed in their cell;
//!
//! and [`HeatGrid::to_markdown`], [`HeatGrid::to_json`], [`html_page`] (standalone,
//! self-contained HTML with inline CSS, no scripts and no network).
//!
//! Colour is never the only carrier of meaning: every cell shows its textual
//! label (ASIL letter, risk level, ...) as well as its tone.

use crate::asil;
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::risk::{
    threat_feasibility_rank, threat_risk, threat_severity_rank, CyberConfig,
};
use serde_json::{json, Value};
use std::collections::HashSet;

/// Semantic colour band of a cell or item (green -> red).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tone {
    Ok,
    Low,
    Medium,
    High,
    Critical,
}

impl Tone {
    pub fn as_str(self) -> &'static str {
        match self {
            Tone::Ok => "ok",
            Tone::Low => "low",
            Tone::Medium => "medium",
            Tone::High => "high",
            Tone::Critical => "critical",
        }
    }
}

/// An element placed in a cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatItem {
    /// Stable id (falls back to the qualified name).
    pub id: String,
    /// Short suffix shown after the id, e.g. `RPN 120` or `goal`.
    pub note: Option<String>,
    /// Per-item tone (FMEA RPN colouring); `None` = use the cell tone.
    pub tone: Option<Tone>,
}

impl HeatItem {
    fn display(&self) -> String {
        match &self.note {
            Some(n) => format!("{} ({})", self.id, n),
            None => self.id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatCell {
    pub label: String,
    pub tone: Tone,
    pub items: Vec<HeatItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatGrid {
    pub title: String,
    pub row_axis: String,
    pub col_axis: String,
    pub row_labels: Vec<String>,
    pub col_labels: Vec<String>,
    /// `cells[row][col]`.
    pub cells: Vec<Vec<HeatCell>>,
    /// Elements that could not be placed (missing / invalid axis values), with a reason.
    pub unplaced: Vec<(String, String)>,
    /// One-line description of the method (printed under the title).
    pub subtitle: String,
    pub legend: Vec<(Tone, String)>,
}

impl HeatGrid {
    fn new(
        title: &str,
        subtitle: String,
        row_axis: &str,
        col_axis: &str,
        row_labels: Vec<String>,
        col_labels: Vec<String>,
        legend: Vec<(Tone, String)>,
        mut cell: impl FnMut(usize, usize) -> (String, Tone),
    ) -> Self {
        let cells = (0..row_labels.len())
            .map(|r| {
                (0..col_labels.len())
                    .map(|c| {
                        let (label, tone) = cell(r, c);
                        HeatCell { label, tone, items: Vec::new() }
                    })
                    .collect()
            })
            .collect();
        HeatGrid {
            title: title.into(),
            subtitle,
            row_axis: row_axis.into(),
            col_axis: col_axis.into(),
            row_labels,
            col_labels,
            cells,
            unplaced: Vec::new(),
            legend,
        }
    }

    /// Number of placed items (an element appears once per cell it occupies).
    pub fn placed(&self) -> usize {
        self.cells.iter().flatten().map(|c| c.items.len()).sum()
    }

    fn sort_items(&mut self) {
        for c in self.cells.iter_mut().flatten() {
            c.items.sort_by(|a, b| a.id.cmp(&b.id));
        }
        self.unplaced.sort();
    }

    // ── renderers ────────────────────────────────────────────────────────────

    pub fn to_json(&self) -> Value {
        json!({
            "title": self.title,
            "method": self.subtitle,
            "rowAxis": self.row_axis,
            "colAxis": self.col_axis,
            "rows": self.row_labels,
            "cols": self.col_labels,
            "legend": self.legend.iter().map(|(t, l)| json!({"tone": t.as_str(), "label": l})).collect::<Vec<_>>(),
            "cells": self.cells.iter().enumerate().flat_map(|(r, row)| {
                row.iter().enumerate().map(move |(c, cell)| json!({
                    "row": self.row_labels[r],
                    "col": self.col_labels[c],
                    "label": cell.label,
                    "tone": cell.tone.as_str(),
                    "elements": cell.items.iter().map(|i| json!({
                        "id": i.id,
                        "note": i.note,
                        "tone": i.tone.map(|t| t.as_str()),
                    })).collect::<Vec<_>>(),
                }))
            }).collect::<Vec<_>>(),
            "unplaced": self.unplaced.iter().map(|(i, why)| json!({"id": i, "reason": why})).collect::<Vec<_>>(),
        })
    }

    pub fn to_markdown(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("## {}\n\n", self.title));
        if !self.subtitle.is_empty() {
            s.push_str(&format!("{}\n\n", self.subtitle));
        }
        s.push_str(&format!("Rows: {}. Columns: {}.\n\n", self.row_axis, self.col_axis));
        s.push_str(&format!("| {} \\ {} |", self.row_axis, self.col_axis));
        for c in &self.col_labels {
            s.push_str(&format!(" {} |", md_esc(c)));
        }
        s.push('\n');
        s.push_str(&"|---".repeat(self.col_labels.len() + 1));
        s.push_str("|\n");
        for (r, row) in self.cells.iter().enumerate() {
            s.push_str(&format!("| **{}** |", md_esc(&self.row_labels[r])));
            for cell in row {
                let mut t = format!(" {}", md_esc(&cell.label));
                if !cell.items.is_empty() {
                    let ids: Vec<String> = cell.items.iter().map(|i| md_esc(&i.display())).collect();
                    t.push_str(&format!(": {}", ids.join(", ")));
                }
                s.push_str(&format!("{t} |"));
            }
            s.push('\n');
        }
        s.push('\n');
        if !self.legend.is_empty() {
            let l: Vec<&str> = self.legend.iter().map(|(_, l)| l.as_str()).collect();
            s.push_str(&format!("Legend (low to high): {}.\n\n", l.join(", ")));
        }
        if !self.unplaced.is_empty() {
            s.push_str("Not placed:\n\n");
            for (id, why) in &self.unplaced {
                s.push_str(&format!("- {} - {}\n", md_esc(id), md_esc(why)));
            }
            s.push('\n');
        }
        s
    }

    /// The grid as an HTML fragment (a `<section>` with an accessible `<table>`).
    pub fn to_html_fragment(&self) -> String {
        let mut s = String::new();
        s.push_str("<section class=\"heat\">\n");
        s.push_str(&format!("<h2>{}</h2>\n", esc(&self.title)));
        if !self.subtitle.is_empty() {
            s.push_str(&format!("<p class=\"sub\">{}</p>\n", esc(&self.subtitle)));
        }
        s.push_str(&format!(
            "<table><caption>{} (rows) by {} (columns)</caption>\n<thead><tr><th scope=\"col\">{} \\ {}</th>",
            esc(&self.row_axis), esc(&self.col_axis), esc(&self.row_axis), esc(&self.col_axis)
        ));
        for c in &self.col_labels {
            s.push_str(&format!("<th scope=\"col\">{}</th>", esc(c)));
        }
        s.push_str("</tr></thead>\n<tbody>\n");
        for (r, row) in self.cells.iter().enumerate() {
            s.push_str(&format!("<tr><th scope=\"row\">{}</th>", esc(&self.row_labels[r])));
            for cell in row {
                s.push_str(&format!("<td class=\"t-{}\"><span class=\"lbl\">{}</span>", cell.tone.as_str(), esc(&cell.label)));
                if !cell.items.is_empty() {
                    s.push_str("<ul>");
                    for i in &cell.items {
                        let cls = i.tone.map(|t| format!(" class=\"i-{}\"", t.as_str())).unwrap_or_default();
                        s.push_str(&format!("<li{}>{}</li>", cls, esc(&i.display())));
                    }
                    s.push_str("</ul>");
                }
                s.push_str("</td>");
            }
            s.push_str("</tr>\n");
        }
        s.push_str("</tbody></table>\n");
        if !self.legend.is_empty() {
            s.push_str("<p class=\"legend\">");
            for (t, l) in &self.legend {
                s.push_str(&format!("<span class=\"sw t-{}\">{}</span> ", t.as_str(), esc(l)));
            }
            s.push_str("</p>\n");
        }
        if !self.unplaced.is_empty() {
            s.push_str("<p>Not placed:</p><ul class=\"unplaced\">");
            for (id, why) in &self.unplaced {
                s.push_str(&format!("<li>{} &mdash; {}</li>", esc(id), esc(why)));
            }
            s.push_str("</ul>\n");
        }
        s.push_str("</section>\n");
        s
    }
}

/// CSS shared by the standalone page and by embedding hosts (export-html, the
/// server page). Scoped to `.heat`.
pub const HEAT_CSS: &str = "\
.heat{margin:1.5em 0}.heat table{border-collapse:collapse}\
.heat caption{text-align:left;color:#555;font-size:.85em;padding-bottom:.3em}\
.heat th,.heat td{border:1px solid #888;padding:.3em .5em;vertical-align:top;font-size:.9em}\
.heat td ul{margin:.2em 0 0;padding-left:1.1em}.heat td .lbl{font-weight:600}\
.heat .t-ok{background:#cfe8cf;color:#102010}.heat .t-low{background:#e8efb0;color:#202000}\
.heat .t-medium{background:#ffe08a;color:#2a2000}.heat .t-high{background:#ffb070;color:#2a1000}\
.heat .t-critical{background:#f08080;color:#2a0000}\
.heat li.i-ok{border-left:4px solid #3c8c3c;padding-left:.2em;list-style:none;margin-left:-1em}\
.heat li.i-low{border-left:4px solid #a0b000;padding-left:.2em;list-style:none;margin-left:-1em}\
.heat li.i-medium{border-left:4px solid #d0a000;padding-left:.2em;list-style:none;margin-left:-1em}\
.heat li.i-high{border-left:4px solid #d06000;padding-left:.2em;list-style:none;margin-left:-1em}\
.heat li.i-critical{border-left:4px solid #b00000;padding-left:.2em;list-style:none;margin-left:-1em}\
.heat .sw{display:inline-block;padding:.1em .6em;border:1px solid #888;font-size:.85em}\
@media (prefers-color-scheme:dark){.heat caption{color:#aaa}\
.heat .t-ok{background:#1f4a1f;color:#e0f0e0}.heat .t-low{background:#4a5a10;color:#f0f4d0}\
.heat .t-medium{background:#6a5000;color:#fff0c0}.heat .t-high{background:#7a3a00;color:#ffe0c8}\
.heat .t-critical{background:#7a1010;color:#ffd8d8}}";

/// A complete standalone HTML document containing the given grids.
pub fn html_page(title: &str, grids: &[HeatGrid]) -> String {
    let mut s = format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
<title>{t}</title><style>body{{font-family:system-ui,sans-serif;margin:1.5em;background:#fff;color:#111}}\
@media (prefers-color-scheme:dark){{body{{background:#161616;color:#e8e8e8}}}}{css}</style></head><body>\n<h1>{t}</h1>\n",
        t = esc(title),
        css = HEAT_CSS
    );
    for g in grids {
        s.push_str(&g.to_html_fragment());
    }
    s.push_str("</body></html>\n");
    s
}

/// Several grids as one JSON document.
pub fn grids_json(grids: &[HeatGrid]) -> Value {
    if grids.len() == 1 {
        grids[0].to_json()
    } else {
        json!(grids.iter().map(|g| g.to_json()).collect::<Vec<_>>())
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn md_esc(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

fn disp_id(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

// ── FMEA ─────────────────────────────────────────────────────────────────────

/// Tone of a severity x occurrence product (1..=100).
pub fn so_tone(product: u32) -> Tone {
    match product {
        50.. => Tone::Critical,
        20..=49 => Tone::High,
        8..=19 => Tone::Medium,
        _ => Tone::Low,
    }
}

/// Tone of an RPN: >=200 critical, >=100 high, >=50 medium, else low.
pub fn rpn_tone(rpn: u32) -> Tone {
    match rpn {
        200.. => Tone::Critical,
        100..=199 => Tone::High,
        50..=99 => Tone::Medium,
        _ => Tone::Low,
    }
}

/// FMEA severity (rows, 10 at the top) x occurrence (columns 1..10). `prefix`
/// restricts to a sheet subtree (`"<sheet qname>::"`).
pub fn fmea_grid(elements: &[RawElement], prefix: Option<&str>) -> HeatGrid {
    let rows: Vec<String> = (1..=10).rev().map(|n| n.to_string()).collect();
    let cols: Vec<String> = (1..=10).map(|n| n.to_string()).collect();
    let mut g = HeatGrid::new(
        "FMEA severity x occurrence",
        "Cell tone is the severity x occurrence product (>=50 critical, >=20 high, >=8 medium, else low); each failure mode is marked with its RPN band (>=200 critical, >=100 high, >=50 medium, else low).".into(),
        "Severity",
        "Occurrence",
        rows,
        cols,
        vec![
            (Tone::Low, "low".into()),
            (Tone::Medium, "medium".into()),
            (Tone::High, "high".into()),
            (Tone::Critical, "critical".into()),
        ],
        |r, c| {
            let sev = 10 - r as u32;
            let occ = c as u32 + 1;
            let p = sev * occ;
            (format!("{p} {}", so_tone(p).as_str()), so_tone(p))
        },
    );
    let mut seen: HashSet<&str> = HashSet::new();
    for e in elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FMEAEntry)))
        .filter(|e| prefix.is_none_or(|p| e.qualified_name.starts_with(p)))
        .filter(|e| seen.insert(e.qualified_name.as_str()))
    {
        let fm = &e.frontmatter;
        let id = disp_id(e);
        match (fm.fmea_severity, fm.occurrence) {
            (Some(s @ 1..=10), Some(o @ 1..=10)) => {
                let rpn = fm.rpn.or_else(|| fm.detection.map(|d| s as u32 * o as u32 * d as u32));
                let item = HeatItem {
                    id,
                    note: rpn.map(|r| format!("RPN {r}")),
                    tone: rpn.map(rpn_tone),
                };
                g.cells[(10 - s) as usize][(o - 1) as usize].items.push(item);
            }
            _ => g.unplaced.push((id, "severity or occurrence missing or outside 1..10".into())),
        }
    }
    g.sort_items();
    g
}

// ── HARA ─────────────────────────────────────────────────────────────────────

/// ASIL -> tone (QM ok, A low, B medium, C high, D critical).
pub fn asil_tone(a: &str) -> Tone {
    match a {
        "A" => Tone::Low,
        "B" => Tone::Medium,
        "C" => Tone::High,
        "D" => Tone::Critical,
        _ => Tone::Ok,
    }
}

/// HARA matrix: rows are severity x exposure (S1..S3 x E1..E4), columns are
/// controllability C1..C3; each cell is the derived ASIL. Hazardous events sit in
/// the cell of their S/E/C; a safety goal sits in the cell of its highest-ASIL
/// linked event. Events with an `S0`/`E0`/`C0` class (always QM) or with
/// unparseable values are listed as not placed.
pub fn hara_grid(elements: &[RawElement]) -> HeatGrid {
    let mut rows = Vec::new();
    for s in 1..=3 {
        for e in 1..=4 {
            rows.push(format!("S{s} / E{e}"));
        }
    }
    let cols: Vec<String> = (1..=3).map(|c| format!("C{c}")).collect();
    let mut g = HeatGrid::new(
        "HARA ASIL matrix",
        "ASIL determination per ISO 26262-3 Table 4 from severity, exposure and controllability.".into(),
        "Severity / Exposure",
        "Controllability",
        rows,
        cols,
        ["QM", "A", "B", "C", "D"].iter().map(|a| (asil_tone(a), a.to_string())).collect(),
        |r, c| {
            let s = r / 4 + 1;
            let e = r % 4 + 1;
            let a = asil::derive(&format!("S{s}"), &format!("E{e}"), &format!("C{}", c + 1)).unwrap_or("QM");
            (a.to_string(), asil_tone(a))
        },
    );
    let resolver = Resolver::new(elements);
    let place = |sv: &str, ex: &str, co: &str| -> Option<(usize, usize)> {
        let n = |s: &str, p: char| s.trim().strip_prefix(p)?.parse::<usize>().ok();
        let (s, e, c) = (n(sv, 'S')?, n(ex, 'E')?, n(co, 'C')?);
        ((1..=3).contains(&s) && (1..=4).contains(&e) && (1..=3).contains(&c))
            .then(|| ((s - 1) * 4 + (e - 1), c - 1))
    };
    for he in elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::HazardousEvent)))
    {
        let fm = &he.frontmatter;
        let id = disp_id(he);
        let (Some(sv), Some(ex), Some(co)) =
            (fm.severity.as_deref(), fm.exposure.as_deref(), fm.controllability.as_deref())
        else {
            g.unplaced.push((id, "severity, exposure or controllability missing".into()));
            continue;
        };
        match place(sv, ex, co) {
            Some((r, c)) => g.cells[r][c].items.push(HeatItem { id, note: None, tone: None }),
            None => {
                let why = match asil::derive(sv, ex, co) {
                    Some(a) => format!("{sv}/{ex}/{co} is outside the S1..S3, E1..E4, C1..C3 table (ASIL {a})"),
                    None => format!("{sv}/{ex}/{co} is not a valid S/E/C class"),
                };
                g.unplaced.push((id, why));
            }
        }
    }
    for sg in elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::SafetyGoal)))
    {
        let mut best: Option<(u8, (usize, usize))> = None;
        for r in sg.frontmatter.hazardous_events.iter().flatten() {
            let Some(he) = resolver.resolve_ref(elements, r) else { continue };
            let hf = &he.frontmatter;
            if let (Some(sv), Some(ex), Some(co)) =
                (hf.severity.as_deref(), hf.exposure.as_deref(), hf.controllability.as_deref())
            {
                if let (Some(rk), Some(pos)) =
                    (asil::derive(sv, ex, co).and_then(asil::rank), place(sv, ex, co))
                {
                    if best.is_none_or(|(b, _)| rk > b) {
                        best = Some((rk, pos));
                    }
                }
            }
        }
        let id = disp_id(sg);
        match best {
            Some((_, (r, c))) => {
                g.cells[r][c].items.push(HeatItem { id, note: Some("goal".into()), tone: None })
            }
            None => g.unplaced.push((id, "no linked hazardous event with a placeable S/E/C".into())),
        }
    }
    g.sort_items();
    g
}

// ── TARA ─────────────────────────────────────────────────────────────────────

fn risk_tone(level: crate::risk::RiskLevel) -> Tone {
    use crate::risk::RiskLevel::*;
    match level {
        Low => Tone::Ok,
        Medium => Tone::Low,
        High => Tone::High,
        Critical => Tone::Critical,
    }
}

/// TARA risk matrix: impact (rows, severe at the top) x attack feasibility
/// (columns, very_low..high) under the configured method; each cell shows the risk
/// level (and the matrix value when the method has one) and the threat scenarios
/// that fall in it. Threats whose impact or feasibility cannot be determined are
/// listed as not placed.
pub fn tara_grid(elements: &[RawElement], cfg: &CyberConfig) -> HeatGrid {
    use crate::cyber_config::{FEASIBILITIES, IMPACTS};
    let rows: Vec<String> = IMPACTS.iter().rev().map(|s| s.to_string()).collect();
    let cols: Vec<String> = FEASIBILITIES.iter().map(|s| s.to_string()).collect();
    let mut g = HeatGrid::new(
        "TARA risk matrix",
        if cfg.is_default() {
            "Risk method: simple (impact rank + feasibility rank: 0-1 low, 2-3 medium, 4 high, 5-6 critical).".to_string()
        } else {
            format!("Risk configuration: {}", cfg.describe())
        },
        "Impact",
        "Attack feasibility",
        rows,
        cols,
        vec![
            (Tone::Ok, "low".into()),
            (Tone::Low, "medium".into()),
            (Tone::High, "high".into()),
            (Tone::Critical, "critical".into()),
        ],
        |r, c| {
            let risk = cfg.risk_cell((3 - r) as u8, c as u8);
            let label = match risk.value {
                Some(v) => format!("{} ({v})", risk.level.as_str()),
                None => risk.level.as_str().to_string(),
            };
            (label, risk_tone(risk.level))
        },
    );
    let resolver = Resolver::new(elements);
    for ts in elements.iter().filter(|e| Resolver::is_threat_scenario(e)) {
        let id = disp_id(ts);
        let sev = threat_severity_rank(ts, elements, &resolver);
        let feas = threat_feasibility_rank(&ts.frontmatter, cfg);
        match (sev, feas, threat_risk(ts, elements, &resolver, cfg)) {
            (Some(s), Some(f), Some(_)) => {
                g.cells[(3 - s.min(3)) as usize][f.min(3) as usize]
                    .items
                    .push(HeatItem { id, note: None, tone: None });
            }
            (None, _, _) => g.unplaced.push((id, "impact unknown (no linked DamageScenario severity)".into())),
            _ => g.unplaced.push((id, "attack feasibility unknown".into())),
        }
    }
    g.sort_items();
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::RawFrontmatter;

    fn el(qn: &str, f: impl FnOnce(&mut RawFrontmatter)) -> RawElement {
        let mut fm = RawFrontmatter::default();
        f(&mut fm);
        RawElement {
            qualified_name: qn.into(),
            file_path: String::new(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: Vec::new(),
            locale_docs: Default::default(),
            about_notes: Vec::new(),
        }
    }

    #[test]
    fn so_and_rpn_bands() {
        assert_eq!(so_tone(100), Tone::Critical);
        assert_eq!(so_tone(20), Tone::High);
        assert_eq!(so_tone(8), Tone::Medium);
        assert_eq!(so_tone(7), Tone::Low);
        assert_eq!(rpn_tone(200), Tone::Critical);
        assert_eq!(rpn_tone(120), Tone::High);
        assert_eq!(rpn_tone(50), Tone::Medium);
        assert_eq!(rpn_tone(49), Tone::Low);
    }

    #[test]
    fn fmea_placement() {
        let els = vec![
            el("S::A", |f| {
                f.element_type = Some(ElementType::FMEAEntry);
                f.id = Some("FM-1".into());
                f.fmea_severity = Some(9);
                f.occurrence = Some(4);
                f.rpn = Some(216);
            }),
            el("S::B", |f| {
                f.element_type = Some(ElementType::FMEAEntry);
                f.id = Some("FM-2".into());
                f.fmea_severity = Some(2);
            }),
        ];
        let g = fmea_grid(&els, None);
        // severity 9 -> row index 1, occurrence 4 -> col index 3; S*O = 36 -> high
        let c = &g.cells[1][3];
        assert_eq!(c.label, "36 high");
        assert_eq!(c.tone, Tone::High);
        assert_eq!(c.items[0].id, "FM-1");
        assert_eq!(c.items[0].tone, Some(Tone::Critical));
        assert_eq!(g.unplaced.len(), 1);
        assert_eq!(g.placed(), 1);
        assert!(g.to_markdown().contains("FM-1 (RPN 216)"));
        assert!(fmea_grid(&els, Some("Other::")).placed() == 0);
    }

    #[test]
    fn hara_placement_and_goal() {
        let els = vec![
            el("H::E1", |f| {
                f.element_type = Some(ElementType::HazardousEvent);
                f.id = Some("HE-1".into());
                f.severity = Some("S3".into());
                f.exposure = Some("E4".into());
                f.controllability = Some("C3".into());
            }),
            el("H::E2", |f| {
                f.element_type = Some(ElementType::HazardousEvent);
                f.id = Some("HE-2".into());
                f.severity = Some("S0".into());
                f.exposure = Some("E4".into());
                f.controllability = Some("C3".into());
            }),
            el("H::SG", |f| {
                f.element_type = Some(ElementType::SafetyGoal);
                f.id = Some("SG-1".into());
                f.hazardous_events = Some(vec!["H::E1".into()]);
            }),
        ];
        let g = hara_grid(&els);
        // S3/E4 is the last row (index 11), C3 the last column.
        let c = &g.cells[11][2];
        assert_eq!(c.label, "D");
        assert_eq!(c.tone, Tone::Critical);
        let ids: Vec<_> = c.items.iter().map(|i| i.display()).collect();
        assert_eq!(ids, vec!["HE-1", "SG-1 (goal)"]);
        assert_eq!(g.unplaced.len(), 1);
        assert_eq!(g.unplaced[0].0, "HE-2");
        // S1/E1/C1 -> QM
        assert_eq!(g.cells[0][0].label, "QM");
        assert_eq!(g.cells[0][0].tone, Tone::Ok);
        // S2/E3/C2 -> A (row 6, col 1)
        assert_eq!(g.cells[6][1].label, "A");
    }

    #[test]
    fn tara_placement_default_method() {
        let els = vec![
            el("D::DS", |f| {
                f.element_type = Some(ElementType::DamageScenario);
                f.id = Some("DS-1".into());
                f.damage_severity = Some("severe".into());
            }),
            el("T::T1", |f| {
                f.element_type = Some(ElementType::ThreatScenario);
                f.id = Some("TS-1".into());
                f.damage_scenarios = Some(vec!["D::DS".into()]);
                f.attack_feasibility = Some("high".into());
            }),
            el("T::T2", |f| {
                f.element_type = Some(ElementType::ThreatScenario);
                f.id = Some("TS-2".into());
            }),
        ];
        let g = tara_grid(&els, &CyberConfig::default());
        // severe -> row 0, high -> col 3; simple score 6 -> critical
        let c = &g.cells[0][3];
        assert_eq!(c.label, "critical");
        assert_eq!(c.tone, Tone::Critical);
        assert_eq!(c.items[0].id, "TS-1");
        assert_eq!(g.unplaced.len(), 1);
        // negligible / very_low -> low
        assert_eq!(g.cells[3][0].label, "low");
        assert_eq!(g.cells[3][0].tone, Tone::Ok);
    }

    #[test]
    fn renderers_are_self_contained() {
        let g = fmea_grid(&[], None);
        let h = html_page("T <x>", &[g.clone()]);
        assert!(h.starts_with("<!doctype html>"));
        assert!(h.contains("T &lt;x&gt;"));
        assert!(!h.contains("http://") && !h.contains("https://") && !h.contains("<script"));
        let j = g.to_json();
        assert_eq!(j["rows"].as_array().unwrap().len(), 10);
        assert_eq!(j["cells"].as_array().unwrap().len(), 100);
    }
}

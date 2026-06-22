use std::collections::HashMap;
use syscribe_model::element::{ElementType, RawElement};

use crate::diagram::layout::{
    build_element_node, load_metrics, render_element, IncludeFilter, ViewConfig, ViewPreset,
};
use crate::diagram::layout::router::{arrowhead_defs, route_edge, route_edge_vert};

// ── Layout constants ──────────────────────────────────────────────────────────
const LEVEL_GAP: f64 = 40.0;     // vertical gap between tree levels
const SIBLING_GAP: f64 = 20.0;   // horizontal gap between siblings
const SIDE_COL_GAP: f64 = 48.0;  // gap from req tree right edge to side columns
const INTER_COL_GAP: f64 = 24.0; // gap between verify and satisfy columns
const CANVAS_PAD: f64 = 40.0;
const TITLE_H: f64 = 28.0;
const MIN_REQ_W: f64 = 200.0;
const MIN_SIDE_W: f64 = 180.0;

pub struct ReqDiagramOptions<'a> {
    pub root: &'a str,
    pub depth: Option<usize>,
    pub show_verify: bool,
    pub show_satisfy: bool,
    pub output: Option<&'a str>,
}

pub fn cmd_diagram_req(
    elements: &[RawElement],
    opts: ReqDiagramOptions,
    config: &syscribe_model::config::ValidateConfig,
) {
    let svg = build_req_diagram(elements, &opts, config);
    match opts.output {
        Some(path) => {
            if let Err(e) = std::fs::write(path, &svg) {
                eprintln!("error writing '{}': {}", path, e);
                std::process::exit(1);
            }
        }
        None => println!("{}", svg),
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

type Rendered = (f64, f64, String); // (width, height, svg)

/// REQ-TRS-LINK-002 — open the `<a>` wrapper for an element shape. When `[links]`
/// resolves a hosted URL for `qname`, emit an SVG hyperlink (both SVG 1.1
/// `xlink:href` and SVG 2 `href`, opening in a new tab); otherwise keep the live
/// server's `/ui/detail/` link. The URL is XML-attribute-escaped.
fn open_shape_anchor(
    qname: &str,
    file_by_qname: &HashMap<&str, &str>,
    config: &syscribe_model::config::ValidateConfig,
) -> String {
    if let Some(file) = file_by_qname.get(qname) {
        if let Some(url) = config.hosted_url_for(file, qname, "") {
            return format!(
                "<a xlink:href=\"{u}\" href=\"{u}\" target=\"_blank\" rel=\"noopener\" data-qname=\"{q}\">",
                u = esc(&url),
                q = esc(qname),
            );
        }
    }
    format!("<a href=\"/ui/detail/{q}\" data-qname=\"{q}\">", q = esc(qname))
}

fn build_req_diagram(
    elements: &[RawElement],
    opts: &ReqDiagramOptions,
    config: &syscribe_model::config::ValidateConfig,
) -> String {
    // qname → on-disk file path, for hosted-URL resolution (REQ-TRS-LINK-002).
    let file_by_qname: HashMap<&str, &str> = elements
        .iter()
        .map(|e| (e.qualified_name.as_str(), e.file_path.as_str()))
        .collect();
    // ── Lookup map: id or qname → element index ───────────────────────────────
    let mut lookup: HashMap<String, usize> = HashMap::new();
    for (i, e) in elements.iter().enumerate() {
        lookup.insert(e.qualified_name.clone(), i);
        if let Some(id) = &e.frontmatter.id {
            lookup.insert(id.clone(), i);
        }
    }

    let root_idx = match lookup.get(opts.root) {
        Some(&i) => i,
        None => {
            eprintln!("error: requirement '{}' not found", opts.root);
            std::process::exit(1);
        }
    };
    let root_qname = elements[root_idx].qualified_name.clone();

    // ── Parent→children map (keyed by parent id or qname) ────────────────────
    let mut children_of: HashMap<String, Vec<String>> = HashMap::new();
    for e in elements {
        let is_req = matches!(
            e.frontmatter.element_type,
            Some(ElementType::Requirement) | Some(ElementType::RequirementDef)
        );
        if !is_req {
            continue;
        }
        if let Some(parents) = &e.frontmatter.derived_from {
            for p in parents {
                children_of.entry(p.clone()).or_default().push(e.qualified_name.clone());
            }
        }
    }

    // ── Collect requirement subtree ───────────────────────────────────────────
    let mut req_qnames: Vec<String> = Vec::new();
    collect_subtree(
        &root_qname,
        elements,
        &lookup,
        &children_of,
        opts.depth,
        0,
        &mut req_qnames,
    );

    let req_set: std::collections::HashSet<String> = req_qnames.iter().cloned().collect();
    // Build a set of IDs for the requirements in our tree
    let req_id_set: std::collections::HashSet<String> = req_qnames
        .iter()
        .filter_map(|qn| lookup.get(qn).and_then(|&i| elements[i].frontmatter.id.clone()))
        .collect();

    // Helper: resolve an id-or-qname ref to a qname that's in our tree
    let resolve_to_req_qn = |r: &str| -> Option<String> {
        if req_set.contains(r) {
            return Some(r.to_string());
        }
        if req_id_set.contains(r) {
            return lookup
                .get(r)
                .map(|&i| elements[i].qualified_name.clone())
                .filter(|qn| req_set.contains(qn));
        }
        None
    };

    // ── Collect test cases (verify) ───────────────────────────────────────────
    let mut tc_pairs: Vec<(String, String)> = Vec::new(); // (tc_qname, req_qname)
    if opts.show_verify {
        for e in elements {
            if !matches!(e.frontmatter.element_type, Some(ElementType::TestCase)) {
                continue;
            }
            if let Some(verified) = &e.frontmatter.verifies {
                for v in verified {
                    if let Some(req_qn) = resolve_to_req_qn(v) {
                        tc_pairs.push((e.qualified_name.clone(), req_qn));
                        break; // one entry per test case (first matching req)
                    }
                }
            }
        }
    }

    // ── Collect arch elements (satisfy) ──────────────────────────────────────
    let mut arch_pairs: Vec<(String, String)> = Vec::new(); // (arch_qname, req_qname)
    if opts.show_satisfy {
        for e in elements {
            let is_arch = matches!(
                e.frontmatter.element_type,
                Some(ElementType::PartDef)
                    | Some(ElementType::Part)
                    | Some(ElementType::ItemDef)
                    | Some(ElementType::Item)
            );
            if !is_arch {
                continue;
            }
            if let Some(satisfied) = &e.frontmatter.satisfies {
                for s in satisfied {
                    if let Some(req_qn) = resolve_to_req_qn(s) {
                        arch_pairs.push((e.qualified_name.clone(), req_qn));
                        break;
                    }
                }
            }
        }
    }

    // ── Measure all elements ──────────────────────────────────────────────────
    let metrics = load_metrics();

    let req_view = ViewConfig {
        preset: ViewPreset::Requirement,
        include: IncludeFilter::default(),
        min_width: Some(MIN_REQ_W),
        ibd: false,
    };
    let side_view = ViewConfig {
        preset: ViewPreset::Compact,
        include: IncludeFilter::default(),
        min_width: Some(MIN_SIDE_W),
        ibd: false,
    };

    let mut req_rendered: HashMap<String, Rendered> = HashMap::new();
    for qn in &req_qnames {
        if let Some(&i) = lookup.get(qn) {
            let node = build_element_node(&elements[i], &req_view);
            let r = render_element(&node, metrics.as_ref());
            req_rendered.insert(qn.clone(), (r.width, r.height, r.svg));
        }
    }

    let mut tc_rendered: HashMap<String, Rendered> = HashMap::new();
    for (tc_qn, _) in &tc_pairs {
        if let Some(&i) = lookup.get(tc_qn) {
            let node = build_element_node(&elements[i], &side_view);
            let r = render_element(&node, metrics.as_ref());
            tc_rendered.insert(tc_qn.clone(), (r.width, r.height, r.svg));
        }
    }

    let mut arch_rendered: HashMap<String, Rendered> = HashMap::new();
    for (arch_qn, _) in &arch_pairs {
        if let Some(&i) = lookup.get(arch_qn) {
            let node = build_element_node(&elements[i], &side_view);
            let r = render_element(&node, metrics.as_ref());
            arch_rendered.insert(arch_qn.clone(), (r.width, r.height, r.svg));
        }
    }

    // ── Tree layout ───────────────────────────────────────────────────────────
    let mut spans: HashMap<String, f64> = HashMap::new();
    compute_spans(&root_qname, &req_rendered, &children_of, &lookup, elements, &mut spans);

    let root_span = spans.get(&root_qname).copied().unwrap_or(MIN_REQ_W);
    let mut positions: HashMap<String, (f64, f64)> = HashMap::new();
    assign_positions(
        &root_qname,
        0.0,
        0.0,
        root_span,
        &req_rendered,
        &children_of,
        &lookup,
        elements,
        &spans,
        &mut positions,
    );

    // ── Side column: test cases ───────────────────────────────────────────────
    let req_max_x = req_qnames
        .iter()
        .filter_map(|qn| {
            let (x, _) = positions.get(qn)?;
            let (w, _, _) = req_rendered.get(qn)?;
            Some(x + w)
        })
        .fold(0.0_f64, f64::max);

    let tc_col_x = req_max_x + SIDE_COL_GAP;
    let tc_max_w = tc_rendered.values().map(|(w, _, _)| *w).fold(0.0_f64, f64::max);

    // Sort test cases by y of their requirement, then stack in side column
    let mut tc_sorted: Vec<(&str, &str)> = tc_pairs
        .iter()
        .filter(|(tc, _)| tc_rendered.contains_key(tc.as_str()))
        .map(|(tc, req)| (tc.as_str(), req.as_str()))
        .collect();
    tc_sorted.sort_by(|(_, ra), (_, rb)| {
        let ya = positions.get(*ra).map(|(_, y)| *y).unwrap_or(0.0);
        let yb = positions.get(*rb).map(|(_, y)| *y).unwrap_or(0.0);
        ya.partial_cmp(&yb).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut tc_positions: HashMap<String, (f64, f64)> = HashMap::new();
    let mut tc_next_y: f64 = 0.0;
    for (tc_qn, req_qn) in &tc_sorted {
        let (_, tc_h, _) = tc_rendered.get(*tc_qn).unwrap();
        let default_rendered: Rendered = (0.0, 0.0, String::new());
        let (_, req_h, _) = req_rendered.get(*req_qn).unwrap_or(&default_rendered);
        let (_, req_y) = positions.get(*req_qn).copied().unwrap_or((0.0, 0.0));
        let ideal_y = req_y + (req_h - tc_h) / 2.0;
        let actual_y = ideal_y.max(tc_next_y);
        tc_positions.insert(tc_qn.to_string(), (tc_col_x, actual_y));
        tc_next_y = actual_y + tc_h + SIBLING_GAP;
    }

    // ── Side column: arch elements ────────────────────────────────────────────
    let arch_col_x = if tc_rendered.is_empty() {
        tc_col_x
    } else {
        tc_col_x + tc_max_w + INTER_COL_GAP
    };

    let mut arch_sorted: Vec<(&str, &str)> = arch_pairs
        .iter()
        .filter(|(arch, _)| arch_rendered.contains_key(arch.as_str()))
        .map(|(arch, req)| (arch.as_str(), req.as_str()))
        .collect();
    arch_sorted.sort_by(|(_, ra), (_, rb)| {
        let ya = positions.get(*ra).map(|(_, y)| *y).unwrap_or(0.0);
        let yb = positions.get(*rb).map(|(_, y)| *y).unwrap_or(0.0);
        ya.partial_cmp(&yb).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut arch_positions: HashMap<String, (f64, f64)> = HashMap::new();
    let mut arch_next_y: f64 = 0.0;
    for (arch_qn, req_qn) in &arch_sorted {
        let (_, arch_h, _) = arch_rendered.get(*arch_qn).unwrap();
        let default_rendered: Rendered = (0.0, 0.0, String::new());
        let (_, req_h, _) = req_rendered.get(*req_qn).unwrap_or(&default_rendered);
        let (_, req_y) = positions.get(*req_qn).copied().unwrap_or((0.0, 0.0));
        let ideal_y = req_y + (req_h - arch_h) / 2.0;
        let actual_y = ideal_y.max(arch_next_y);
        arch_positions.insert(arch_qn.to_string(), (arch_col_x, actual_y));
        arch_next_y = actual_y + arch_h + SIBLING_GAP;
    }

    // ── Canvas size ───────────────────────────────────────────────────────────
    let arch_max_w = arch_rendered.values().map(|(w, _, _)| *w).fold(0.0_f64, f64::max);
    let content_max_x = if !arch_rendered.is_empty() {
        arch_col_x + arch_max_w
    } else if !tc_rendered.is_empty() {
        tc_col_x + tc_max_w
    } else {
        req_max_x
    };

    let req_max_y = req_qnames
        .iter()
        .filter_map(|qn| {
            let (_, y) = positions.get(qn)?;
            let (_, h, _) = req_rendered.get(qn)?;
            Some(y + h)
        })
        .fold(0.0_f64, f64::max);
    let tc_max_y = tc_positions
        .iter()
        .filter_map(|(qn, (_, y))| tc_rendered.get(qn).map(|(_, h, _)| y + h))
        .fold(0.0_f64, f64::max);
    let arch_max_y = arch_positions
        .iter()
        .filter_map(|(qn, (_, y))| arch_rendered.get(qn).map(|(_, h, _)| y + h))
        .fold(0.0_f64, f64::max);
    let content_max_y = req_max_y.max(tc_max_y).max(arch_max_y);

    let canvas_w = content_max_x + CANVAS_PAD * 2.0;
    let canvas_h = content_max_y + CANVAS_PAD * 2.0 + TITLE_H;

    let ox = CANVAS_PAD;          // element x offset
    let oy = CANVAS_PAD + TITLE_H; // element y offset

    // ── Derive diagram title from root element ────────────────────────────────
    let title = {
        let root_elem = &elements[root_idx];
        let root_name = root_elem
            .frontmatter
            .name
            .clone()
            .unwrap_or_else(|| root_qname.split("::").last().unwrap_or(&root_qname).to_string());
        format!("{} — Requirement Diagram", root_name)
    };

    // ── Build SVG parts ───────────────────────────────────────────────────────
    let mut parts: Vec<String> = Vec::new();

    parts.push(format!(
        "<text x=\"{x:.1}\" y=\"{y:.1}\" font-size=\"16\" font-weight=\"700\" fill=\"#1a1a2e\">{t}</text>",
        x = CANVAS_PAD,
        y = CANVAS_PAD / 2.0 + 12.0,
        t = esc(&title)
    ));

    parts.push(arrowhead_defs());

    // Derive edges: child top-center → parent bottom-center (arrow at parent)
    for qn in &req_qnames {
        let e = match lookup.get(qn).map(|&i| &elements[i]) {
            Some(e) => e,
            None => continue,
        };
        if let Some(parents) = &e.frontmatter.derived_from {
            for p in parents {
                let parent_qn = if req_set.contains(p.as_str()) {
                    p.clone()
                } else if let Some(&i) = lookup.get(p.as_str()) {
                    let pqn = elements[i].qualified_name.clone();
                    if req_set.contains(&pqn) {
                        pqn
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let (cx, cy) = match positions.get(qn) { Some(p) => *p, None => continue };
                let (cw, _, _) = match req_rendered.get(qn) { Some(r) => r, None => continue };
                let (px, py) = match positions.get(&parent_qn) { Some(p) => *p, None => continue };
                let (pw, ph, _) = match req_rendered.get(&parent_qn) { Some(r) => r, None => continue };

                // child top-center → parent bottom-center; arrowhead at parent
                let x1 = cx + cw / 2.0 + ox;
                let y1 = cy + oy;              // child top
                let x2 = px + pw / 2.0 + ox;
                let y2 = py + ph + oy;         // parent bottom
                parts.push(route_edge_vert(x1, y1, x2, y2, "derive"));
            }
        }
    }

    // Verify edges: tc left-center → req right-center (arrow at requirement)
    for (tc_qn, req_qn) in &tc_pairs {
        let (tc_x, tc_y) = match tc_positions.get(tc_qn) { Some(p) => *p, None => continue };
        let (_, tc_h, _) = match tc_rendered.get(tc_qn) { Some(r) => r, None => continue };
        let (req_x, req_y) = match positions.get(req_qn) { Some(p) => *p, None => continue };
        let (req_w, req_h, _) = match req_rendered.get(req_qn) { Some(r) => r, None => continue };

        let x1 = tc_x + ox;                     // tc left edge
        let y1 = tc_y + tc_h / 2.0 + oy;
        let x2 = req_x + req_w + ox;            // req right edge
        let y2 = req_y + req_h / 2.0 + oy;
        parts.push(route_edge(x1, y1, x2, y2, "verify"));
    }

    // Satisfy edges: arch left-center → req right-center (arrow at requirement)
    for (arch_qn, req_qn) in &arch_pairs {
        let (arch_x, arch_y) = match arch_positions.get(arch_qn) { Some(p) => *p, None => continue };
        let (_, arch_h, _) = match arch_rendered.get(arch_qn) { Some(r) => r, None => continue };
        let (req_x, req_y) = match positions.get(req_qn) { Some(p) => *p, None => continue };
        let (req_w, req_h, _) = match req_rendered.get(req_qn) { Some(r) => r, None => continue };

        let x1 = arch_x + ox;
        let y1 = arch_y + arch_h / 2.0 + oy;
        let x2 = req_x + req_w + ox;
        let y2 = req_y + req_h / 2.0 + oy;
        parts.push(route_edge(x1, y1, x2, y2, "satisfy"));
    }

    // Requirement element blocks
    for qn in &req_qnames {
        if let Some((x, y)) = positions.get(qn) {
            if let Some((_, _, svg)) = req_rendered.get(qn) {
                parts.push(format!(
                    "{anchor}\
                     <g transform=\"translate({x:.1} {y:.1})\">\n{inner}\n</g>\
                     </a>",
                    anchor = open_shape_anchor(qn, &file_by_qname, config),
                    x = x + ox,
                    y = y + oy,
                    inner = strip_svg_wrapper(svg)
                ));
            }
        }
    }

    // Test case blocks
    for (tc_qn, _) in &tc_pairs {
        if let Some((x, y)) = tc_positions.get(tc_qn) {
            if let Some((_, _, svg)) = tc_rendered.get(tc_qn) {
                parts.push(format!(
                    "{anchor}\
                     <g transform=\"translate({x:.1} {y:.1})\">\n{inner}\n</g>\
                     </a>",
                    anchor = open_shape_anchor(tc_qn, &file_by_qname, config),
                    x = x + ox,
                    y = y + oy,
                    inner = strip_svg_wrapper(svg)
                ));
            }
        }
    }

    // Arch element blocks
    for (arch_qn, _) in &arch_pairs {
        if let Some((x, y)) = arch_positions.get(arch_qn) {
            if let Some((_, _, svg)) = arch_rendered.get(arch_qn) {
                parts.push(format!(
                    "{anchor}\
                     <g transform=\"translate({x:.1} {y:.1})\">\n{inner}\n</g>\
                     </a>",
                    anchor = open_shape_anchor(arch_qn, &file_by_qname, config),
                    x = x + ox,
                    y = y + oy,
                    inner = strip_svg_wrapper(svg)
                ));
            }
        }
    }

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" \
         xmlns:xlink=\"http://www.w3.org/1999/xlink\" \
         viewBox=\"0 0 {w:.0} {h:.0}\" width=\"{w:.0}\" height=\"{h:.0}\">\n\
         <style>text{{font-family:'Inter','Roboto',system-ui,sans-serif}}\
         @font-face{{font-family:'Inter';font-weight:400;\
         src:url('https://fonts.bunny.net/inter/files/inter-latin-400-normal.woff2') format('woff2')}}\
         @font-face{{font-family:'Inter';font-weight:700;\
         src:url('https://fonts.bunny.net/inter/files/inter-latin-700-normal.woff2') format('woff2')}}\
         </style>\n\
         <rect width=\"{w:.0}\" height=\"{h:.0}\" fill=\"#ffffff\"/>\n\
         {inner}\n\
         </svg>",
        w = canvas_w,
        h = canvas_h,
        inner = parts.join("\n")
    )
}

// ── Tree building ─────────────────────────────────────────────────────────────

fn collect_subtree(
    qname: &str,
    elements: &[RawElement],
    lookup: &HashMap<String, usize>,
    children_of: &HashMap<String, Vec<String>>,
    max_depth: Option<usize>,
    depth: usize,
    result: &mut Vec<String>,
) {
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }
    if !result.contains(&qname.to_string()) {
        result.push(qname.to_string());
    }
    for child_qn in req_children(qname, children_of, lookup, elements) {
        collect_subtree(&child_qn, elements, lookup, children_of, max_depth, depth + 1, result);
    }
}

/// Return qualified names of requirements that are direct children of `qname`.
fn req_children(
    qname: &str,
    children_of: &HashMap<String, Vec<String>>,
    lookup: &HashMap<String, usize>,
    elements: &[RawElement],
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    // Children may be keyed by either the parent's qname or its REQ-* id
    if let Some(children) = children_of.get(qname) {
        result.extend(children.clone());
    }
    if let Some(&i) = lookup.get(qname) {
        if let Some(id) = &elements[i].frontmatter.id {
            if let Some(children) = children_of.get(id) {
                for c in children {
                    if !result.contains(c) {
                        result.push(c.clone());
                    }
                }
            }
        }
    }
    result
}

// ── Tree layout ───────────────────────────────────────────────────────────────

/// Compute the minimum horizontal span needed to display `qname`'s subtree.
fn compute_spans(
    qname: &str,
    rendered: &HashMap<String, Rendered>,
    children_of: &HashMap<String, Vec<String>>,
    lookup: &HashMap<String, usize>,
    elements: &[RawElement],
    spans: &mut HashMap<String, f64>,
) -> f64 {
    let self_w = rendered.get(qname).map(|(w, _, _)| *w).unwrap_or(MIN_REQ_W);
    let children = req_children(qname, children_of, lookup, elements);

    if children.is_empty() {
        spans.insert(qname.to_string(), self_w);
        return self_w;
    }

    let children_total: f64 = children
        .iter()
        .map(|c| compute_spans(c, rendered, children_of, lookup, elements, spans))
        .sum::<f64>()
        + SIBLING_GAP * (children.len() - 1) as f64;

    let span = self_w.max(children_total);
    spans.insert(qname.to_string(), span);
    span
}

/// Assign absolute (x, y) top-left positions to each node in the subtree.
#[allow(clippy::too_many_arguments)]
fn assign_positions(
    qname: &str,
    span_left: f64,
    y: f64,
    span: f64,
    rendered: &HashMap<String, Rendered>,
    children_of: &HashMap<String, Vec<String>>,
    lookup: &HashMap<String, usize>,
    elements: &[RawElement],
    spans: &HashMap<String, f64>,
    positions: &mut HashMap<String, (f64, f64)>,
) {
    let (w, h, _) = match rendered.get(qname) {
        Some(r) => (r.0, r.1, &r.2),
        None => return,
    };

    let x = span_left + (span - w) / 2.0;
    positions.insert(qname.to_string(), (x, y));

    let children = req_children(qname, children_of, lookup, elements);
    if children.is_empty() {
        return;
    }

    let children_total: f64 = children
        .iter()
        .map(|c| spans.get(c.as_str()).copied().unwrap_or(MIN_REQ_W))
        .sum::<f64>()
        + SIBLING_GAP * (children.len() - 1) as f64;

    let child_y = y + h + LEVEL_GAP;
    let mut child_x = span_left + (span - children_total) / 2.0;

    for child_qn in &children {
        let child_span = spans.get(child_qn.as_str()).copied().unwrap_or(MIN_REQ_W);
        assign_positions(
            child_qn, child_x, child_y, child_span, rendered, children_of, lookup, elements,
            spans, positions,
        );
        child_x += child_span + SIBLING_GAP;
    }
}

// ── SVG utilities ─────────────────────────────────────────────────────────────

fn strip_svg_wrapper(svg: &str) -> String {
    if let Some(start) = svg.find('>') {
        let after = &svg[start + 1..];
        if let Some(end) = after.rfind("</svg>") {
            return after[..end].trim().to_string();
        }
    }
    svg.to_string()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

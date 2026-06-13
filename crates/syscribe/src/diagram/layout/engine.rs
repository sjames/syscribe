use taffy::prelude::*;

use super::{
    builder::{feature_row_text, port_row_text},
    metrics::TextMetrics,
    types::*,
};

// ── Nested port geometry helpers ──────────────────────────────────────────────

/// Height of the port frame (outer rect) for a port, flat or nested.
fn port_frame_h(row: &PortRow) -> f64 {
    if row.sub_ports.is_empty() {
        PORT_SQ
    } else {
        // Stack inner squares vertically inside the frame
        row.sub_ports.len() as f64 * (PORT_SQ_INNER + PORT_INNER_PAD) + PORT_INNER_PAD
    }
}

/// Total row height (center-to-center spacing) consumed by this port.
fn port_row_h(row: &PortRow) -> f64 {
    if row.sub_ports.is_empty() {
        IBD_PORT_ROW_H
    } else {
        // Scale the gap around the frame the same as flat ports
        port_frame_h(row) + (IBD_PORT_ROW_H - PORT_SQ)
    }
}

// ── SVG helpers ───────────────────────────────────────────────────────────────

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn font_face_style() -> &'static str {
    "<style>\
     @font-face{font-family:'Inter';font-weight:400;\
     src:url('https://fonts.bunny.net/inter/files/inter-latin-400-normal.woff2') format('woff2')}\
     @font-face{font-family:'Inter';font-weight:700;\
     src:url('https://fonts.bunny.net/inter/files/inter-latin-700-normal.woff2') format('woff2')}\
     text{font-family:'Inter','Roboto',system-ui,sans-serif}\
     </style>"
}

// ── Measured compartment heights ─────────────────────────────────────────────

const HEADER_PAD_TOP: f64 = 10.0;
const HEADER_PAD_BOT: f64 = 10.0;
const SECTION_PAD_TOP: f64 = 6.0;
const SECTION_PAD_BOT: f64 = 8.0;
const SECTION_TITLE_H: f64 = 14.0;
const ROW_H: f64 = 18.0;
const STATUS_PAD_V: f64 = 6.0;
const BADGE_H: f64 = 18.0;
const BADGE_PAD_H: f64 = 8.0;
const BADGE_R: f64 = 4.0;
const BADGE_GAP: f64 = 6.0;

/// x offset where all text content starts (accent strip + left pad)
const CONTENT_X: f64 = ACCENT_STRIP_W + PAD_H;

/// Compute all content text widths and return the required minimum box width.
fn required_width(node: &ElementNode, m: &dyn TextMetrics) -> f64 {
    let mut max_w: f64 = node.min_width;

    for c in &node.compartments {
        match c {
            Compartment::Header { stereotype, applied_stereotypes, name, badges, .. } => {
                let mut stereo_w = stereotype.as_deref().map_or(0.0, |s| {
                    m.advance_width(&format!("«{}»", s), FS_STEREOTYPE, false)
                });
                for s in applied_stereotypes {
                    stereo_w = stereo_w
                        .max(m.advance_width(&format!("«{}»", s), FS_STEREOTYPE, false));
                }
                let name_w = m.advance_width(name, FS_NAME, true);
                let badge_w: f64 = badges.iter().map(|b| {
                    m.advance_width(&b.text, FS_BADGE, b.mono) + BADGE_PAD_H * 2.0
                }).sum::<f64>()
                    + if badges.is_empty() { 0.0 } else { BADGE_GAP * (badges.len() - 1) as f64 };
                let row_w = stereo_w.max(name_w) + if badge_w > 0.0 { badge_w + PAD_H } else { 0.0 };
                max_w = max_w.max(row_w + ACCENT_STRIP_W + PAD_H * 2.0);
            }
            Compartment::StatusRow { badges } => {
                let total: f64 = badges.iter().map(|b| {
                    m.advance_width(&b.text, FS_BADGE, b.mono) + BADGE_PAD_H * 2.0
                }).sum::<f64>()
                    + if badges.is_empty() { 0.0 } else { BADGE_GAP * (badges.len() - 1) as f64 };
                max_w = max_w.max(total + ACCENT_STRIP_W + PAD_H * 2.0);
            }
            Compartment::PortsList { items } => {
                if !node.ibd {
                    for row in items {
                        let text = port_row_text(row);
                        max_w = max_w.max(m.advance_width(&text, FS_ROW, false) + ACCENT_STRIP_W + PAD_H * 2.0);
                    }
                }
                // IBD mode: ports are border squares, not text — don't affect width
            }
            Compartment::Features { items } => {
                for row in items {
                    let text = feature_row_text(row);
                    max_w = max_w.max(m.advance_width(&text, FS_ROW, false) + ACCENT_STRIP_W + PAD_H * 2.0);
                }
            }
            Compartment::DocPreview { lines } => {
                for line in lines {
                    max_w = max_w.max(m.advance_width(line, FS_DOC, false) + ACCENT_STRIP_W + PAD_H * 2.0);
                }
            }
            Compartment::GherkinPreview { given, when, then } => {
                for line in [given, when, then].iter().filter_map(|l| l.as_deref()) {
                    max_w = max_w.max(m.advance_width(line, FS_DOC, false) + ACCENT_STRIP_W + PAD_H * 2.0);
                }
            }
        }
    }

    // Round up to nearest 4px
    (max_w / 4.0).ceil() * 4.0
}

// ── Taffy layout for header (flex row: left | right) ─────────────────────────

struct HeaderLayout {
    total_height: f64,
    #[allow(dead_code)]
    badge_x_start: f64,
}

fn layout_header(
    stereotype: Option<&str>,
    applied_count: usize,
    _name: &str,
    is_abstract: bool,
    badges: &[Badge],
    box_width: f64,
    m: &dyn TextMetrics,
) -> HeaderLayout {
    let mut taffy = TaffyTree::<()>::new();

    let has_stereo_line = stereotype.is_some() || applied_count > 0 || is_abstract;
    let stereo_h = if has_stereo_line {
        m.line_height(FS_STEREOTYPE)
    } else {
        0.0
    };
    // Each applied-metadata «Name» banner adds one stereotype line (REQ-TRS-META-002).
    let applied_h = applied_count as f64 * m.line_height(FS_STEREOTYPE);
    let name_h = m.line_height(FS_NAME);
    let abstract_h = if is_abstract && (stereotype.is_some() || applied_count > 0) {
        m.line_height(FS_STEREOTYPE)
    } else {
        0.0
    };
    let left_h =
        stereo_h + applied_h + name_h + abstract_h + GAP * (if stereo_h > 0.0 { 1.0 } else { 0.0 });

    let left_col = taffy
        .new_leaf(Style {
            size: Size {
                width: auto(),
                height: length(left_h as f32),
            },
            flex_grow: 1.0,
            ..Default::default()
        })
        .unwrap();

    let mut badge_nodes = Vec::new();
    for b in badges {
        let bw = m.advance_width(&b.text, FS_BADGE, b.mono) + BADGE_PAD_H * 2.0;
        let node = taffy
            .new_leaf(Style {
                size: Size {
                    width: length(bw as f32),
                    height: length(BADGE_H as f32),
                },
                ..Default::default()
            })
            .unwrap();
        badge_nodes.push(node);
    }

    let right_col = taffy
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                gap: Size {
                    width: length(BADGE_GAP as f32),
                    height: length(0.0),
                },
                align_items: Some(AlignItems::CENTER),
                flex_shrink: 1.0,
                ..Default::default()
            },
            &badge_nodes,
        )
        .unwrap();

    let root = taffy
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                size: Size {
                    width: length(box_width as f32),
                    height: auto(),
                },
                padding: Rect {
                    left: length(CONTENT_X as f32),
                    right: length(PAD_H as f32),
                    top: length(HEADER_PAD_TOP as f32),
                    bottom: length(HEADER_PAD_BOT as f32),
                },
                gap: Size {
                    width: length(PAD_H as f32),
                    height: length(0.0),
                },
                align_items: Some(AlignItems::CENTER),
                ..Default::default()
            },
            &[left_col, right_col],
        )
        .unwrap();

    taffy
        .compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(box_width as f32),
                height: AvailableSpace::MaxContent,
            },
        )
        .unwrap();

    let root_layout = taffy.layout(root).unwrap();
    let total_height = root_layout.size.height as f64;

    let badge_x = if !badge_nodes.is_empty() {
        let right_layout = taffy.layout(right_col).unwrap();
        right_layout.location.x as f64 + CONTENT_X
    } else {
        box_width
    };

    HeaderLayout { total_height, badge_x_start: badge_x }
}

// ── IBD port label helper ─────────────────────────────────────────────────────

fn port_name_label(row: &PortRow) -> String {
    match &row.type_ref {
        Some(t) => {
            let short = t.split("::").last().unwrap_or(t.as_str());
            format!("{} : {}", row.name, short)
        }
        None => row.name.clone(),
    }
}

// ── SVG emission ─────────────────────────────────────────────────────────────

pub fn render_element(node: &ElementNode, m: &dyn TextMetrics) -> RenderedElement {
    let box_width = required_width(node, m);
    let theme = &node.theme;

    let mut svg_parts: Vec<String> = Vec::new();
    let mut port_anchors: Vec<PortAnchor> = Vec::new();

    // ── Defs ─────────────────────────────────────────────────────────────────
    let defs = format!("<defs>{}</defs>", font_face_style());
    svg_parts.push(defs);

    let mut y: f64 = 0.0;
    let mut header_bottom: f64 = 0.0;
    let mut ibd_left_ports: Vec<PortRow> = Vec::new();
    let mut ibd_right_ports: Vec<PortRow> = Vec::new();

    // ── Per-compartment SVG ───────────────────────────────────────────────────
    for (ci, compartment) in node.compartments.iter().enumerate() {
        match compartment {
            Compartment::Header { stereotype, applied_stereotypes, name, is_abstract, badges } => {
                let hl = layout_header(
                    stereotype.as_deref(),
                    applied_stereotypes.len(),
                    name,
                    *is_abstract,
                    badges,
                    box_width,
                    m,
                );
                let h = hl.total_height;

                // Header background — plain rect (no clip, no rounded corners needed)
                svg_parts.push(format!(
                    "<rect x=\"0\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" fill=\"{fill}\"/>",
                    y = y, w = box_width, h = h, fill = theme.header_bg
                ));

                // Stereotype label
                let mut text_y = y + HEADER_PAD_TOP;
                if let Some(stereo) = stereotype {
                    let stereo_text = format!("«{}»", stereo);
                    let lh = m.line_height(FS_STEREOTYPE);
                    text_y += m.cap_height(FS_STEREOTYPE);
                    svg_parts.push(format!(
                        "<text x=\"{x:.1}\" y=\"{ty:.1}\" font-size=\"{fs}\" \
                         fill=\"{fg}\" font-style=\"italic\">{text}</text>",
                        x = CONTENT_X, ty = text_y, fs = FS_STEREOTYPE,
                        fg = theme.stereotype_fg, text = esc(&stereo_text)
                    ));
                    text_y += lh - m.cap_height(FS_STEREOTYPE) + GAP;
                }

                // Applied-metadata «Name» stereotype banners (REQ-TRS-META-002), one line
                // each, reusing the shared stereotype styling (stereotype_fg, italic).
                if !applied_stereotypes.is_empty() {
                    // When there is no type-keyword banner, the first applied banner is the
                    // top line and still needs the leading gap before the name below it.
                    let lh = m.line_height(FS_STEREOTYPE);
                    for applied in applied_stereotypes {
                        let applied_text = format!("«{}»", applied);
                        text_y += m.cap_height(FS_STEREOTYPE);
                        svg_parts.push(format!(
                            "<text x=\"{x:.1}\" y=\"{ty:.1}\" font-size=\"{fs}\" \
                             fill=\"{fg}\" font-style=\"italic\">{text}</text>",
                            x = CONTENT_X, ty = text_y, fs = FS_STEREOTYPE,
                            fg = theme.stereotype_fg, text = esc(&applied_text)
                        ));
                        text_y += lh - m.cap_height(FS_STEREOTYPE);
                    }
                    text_y += GAP;
                }

                // Name
                let name_style = if *is_abstract { " font-style=\"italic\"" } else { "" };
                text_y += m.cap_height(FS_NAME);
                svg_parts.push(format!(
                    "<text x=\"{x:.1}\" y=\"{ty:.1}\" font-size=\"{fs}\" \
                     font-weight=\"700\" fill=\"{fg}\"{style}>{text}</text>",
                    x = CONTENT_X, ty = text_y, fs = FS_NAME,
                    fg = theme.header_fg, style = name_style, text = esc(name)
                ));

                // Abstract sub-label
                if *is_abstract {
                    text_y += m.line_height(FS_STEREOTYPE);
                    svg_parts.push(format!(
                        "<text x=\"{x:.1}\" y=\"{ty:.1}\" font-size=\"{fs}\" \
                         fill=\"{fg}\" font-style=\"italic\">{{abstract}}</text>",
                        x = CONTENT_X, ty = text_y + m.cap_height(FS_STEREOTYPE),
                        fs = FS_STEREOTYPE, fg = theme.stereotype_fg
                    ));
                }

                // Header badges (right-aligned)
                if !badges.is_empty() {
                    let badge_center_y = y + h / 2.0;
                    let total_badge_w: f64 = badges.iter().map(|b| {
                        m.advance_width(&b.text, FS_BADGE, b.mono) + BADGE_PAD_H * 2.0
                    }).sum::<f64>()
                        + BADGE_GAP * (badges.len() - 1) as f64;
                    let mut bx = box_width - PAD_H - total_badge_w;
                    for badge in badges {
                        let bw = m.advance_width(&badge.text, FS_BADGE, badge.mono) + BADGE_PAD_H * 2.0;
                        let font_weight = if badge.mono { " font-weight=\"400\"" } else { "" };
                        let mono_family = if badge.mono {
                            " font-family=\"'JetBrains Mono','Roboto Mono',monospace\""
                        } else {
                            ""
                        };
                        svg_parts.push(format!(
                            "<rect x=\"{bx:.1}\" y=\"{by:.1}\" width=\"{bw:.1}\" \
                             height=\"{bh}\" rx=\"{r}\" fill=\"{bg}\"/>\
                             <text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"{fs}\"{fw} \
                             fill=\"{fg}\" text-anchor=\"middle\"{mf}>{text}</text>",
                            bx = bx, by = badge_center_y - BADGE_H / 2.0,
                            bw = bw, bh = BADGE_H, r = BADGE_R,
                            bg = badge.bg,
                            tx = bx + bw / 2.0,
                            ty = badge_center_y + m.cap_height(FS_BADGE) / 2.0,
                            fs = FS_BADGE, fw = font_weight, fg = badge.fg,
                            mf = mono_family, text = esc(&badge.text)
                        ));
                        bx += bw + BADGE_GAP;
                    }
                }

                y += h;
                header_bottom = y;
            }

            Compartment::StatusRow { badges } => {
                // Draw divider before this compartment (ci==1 uses accent color, others use divider color)
                if ci > 0 {
                    let divider_color = if ci == 1 { theme.accent } else { theme.divider };
                    svg_parts.push(divider_line(y, box_width, divider_color));
                    y += DIVIDER_H;
                }

                let h = STATUS_PAD_V * 2.0 + BADGE_H;
                svg_parts.push(format!(
                    "<rect x=\"0\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" fill=\"{fill}\"/>",
                    y = y, w = box_width, h = h, fill = theme.body_bg
                ));

                let mut bx = CONTENT_X;
                let center_y = y + h / 2.0;
                for badge in badges {
                    let bw = m.advance_width(&badge.text, FS_BADGE, badge.mono) + BADGE_PAD_H * 2.0;
                    svg_parts.push(format!(
                        "<rect x=\"{bx:.1}\" y=\"{by:.1}\" width=\"{bw:.1}\" \
                         height=\"{bh}\" rx=\"{r}\" fill=\"{bg}\"/>\
                         <text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"{fs}\" \
                         fill=\"{fg}\" text-anchor=\"middle\">{text}</text>",
                        bx = bx, by = center_y - BADGE_H / 2.0,
                        bw = bw, bh = BADGE_H, r = BADGE_R, bg = badge.bg,
                        tx = bx + bw / 2.0,
                        ty = center_y + m.cap_height(FS_BADGE) / 2.0,
                        fs = FS_BADGE, fg = badge.fg, text = esc(&badge.text)
                    ));
                    bx += bw + BADGE_GAP;
                }
                y += h;
            }

            Compartment::PortsList { items } => {
                if node.ibd {
                    // In IBD mode: collect ports into left/right lists, do not render as text
                    for row in items {
                        match row.direction {
                            PortDirection::Out => ibd_right_ports.push(row.clone()),
                            _ => ibd_left_ports.push(row.clone()),
                        }
                    }
                    // No y advance, no divider
                } else {
                    if ci > 0 {
                        let divider_color = if ci == 1 { theme.accent } else { theme.divider };
                        svg_parts.push(divider_line(y, box_width, divider_color));
                        y += DIVIDER_H;
                    }
                    svg_parts.push(format!(
                        "<rect x=\"0\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" fill=\"{fill}\"/>",
                        y = y,
                        w = box_width,
                        h = SECTION_PAD_TOP + SECTION_TITLE_H + items.len() as f64 * ROW_H + SECTION_PAD_BOT,
                        fill = theme.body_bg
                    ));

                    y += SECTION_PAD_TOP;
                    svg_parts.push(section_title("ports", y, theme.muted_fg, m));
                    y += SECTION_TITLE_H;

                    for row in items {
                        let text = port_row_text(row);
                        let anchor_y = y + ROW_H / 2.0;
                        let side = PortSide::from(&row.direction);
                        let anchor_x = match &side {
                            PortSide::Left => 0.0,
                            PortSide::Right => box_width,
                        };

                        svg_parts.push(format!(
                            "<text x=\"{x:.1}\" y=\"{ty:.1}\" font-size=\"{fs}\" \
                             fill=\"{fg}\">{text}</text>",
                            x = CONTENT_X,
                            ty = y + m.cap_height(FS_ROW),
                            fs = FS_ROW, fg = theme.body_fg,
                            text = esc(&text)
                        ));

                        port_anchors.push(PortAnchor {
                            name: row.name.clone(),
                            x: anchor_x,
                            y: anchor_y,
                            side: match side {
                                PortSide::Left => "left".to_string(),
                                PortSide::Right => "right".to_string(),
                            },
                            direction: format!("{:?}", row.direction).to_lowercase(),
                        });

                        y += ROW_H;
                    }
                    y += SECTION_PAD_BOT;
                }
            }

            Compartment::Features { items } => {
                if ci > 0 {
                    let divider_color = if ci == 1 { theme.accent } else { theme.divider };
                    svg_parts.push(divider_line(y, box_width, divider_color));
                    y += DIVIDER_H;
                }
                svg_parts.push(format!(
                    "<rect x=\"0\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" fill=\"{fill}\"/>",
                    y = y,
                    w = box_width,
                    h = SECTION_PAD_TOP + SECTION_TITLE_H + items.len() as f64 * ROW_H + SECTION_PAD_BOT,
                    fill = theme.body_bg
                ));

                y += SECTION_PAD_TOP;
                svg_parts.push(section_title("features", y, theme.muted_fg, m));
                y += SECTION_TITLE_H;

                for row in items {
                    let text = feature_row_text(row);
                    svg_parts.push(format!(
                        "<text x=\"{x:.1}\" y=\"{ty:.1}\" font-size=\"{fs}\" fill=\"{fg}\">{text}</text>",
                        x = CONTENT_X,
                        ty = y + m.cap_height(FS_ROW),
                        fs = FS_ROW, fg = theme.body_fg,
                        text = esc(&text)
                    ));
                    y += ROW_H;
                }
                y += SECTION_PAD_BOT;
            }

            Compartment::DocPreview { lines } => {
                if ci > 0 {
                    let divider_color = if ci == 1 { theme.accent } else { theme.divider };
                    svg_parts.push(divider_line(y, box_width, divider_color));
                    y += DIVIDER_H;
                }
                svg_parts.push(format!(
                    "<rect x=\"0\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" fill=\"{fill}\"/>",
                    y = y,
                    w = box_width,
                    h = SECTION_PAD_TOP + lines.len() as f64 * ROW_H + SECTION_PAD_BOT,
                    fill = theme.body_bg
                ));
                y += SECTION_PAD_TOP;
                for line in lines {
                    svg_parts.push(format!(
                        "<text x=\"{x:.1}\" y=\"{ty:.1}\" font-size=\"{fs}\" \
                         fill=\"{fg}\" font-style=\"italic\">{text}</text>",
                        x = CONTENT_X,
                        ty = y + m.cap_height(FS_DOC),
                        fs = FS_DOC, fg = theme.muted_fg,
                        text = esc(line)
                    ));
                    y += ROW_H;
                }
                y += SECTION_PAD_BOT;
            }

            Compartment::GherkinPreview { given, when, then } => {
                if ci > 0 {
                    let divider_color = if ci == 1 { theme.accent } else { theme.divider };
                    svg_parts.push(divider_line(y, box_width, divider_color));
                    y += DIVIDER_H;
                }
                let lines: Vec<&str> = [given, when, then]
                    .iter()
                    .filter_map(|l| l.as_deref())
                    .collect();
                svg_parts.push(format!(
                    "<rect x=\"0\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" fill=\"{fill}\"/>",
                    y = y, w = box_width,
                    h = SECTION_PAD_TOP + lines.len() as f64 * ROW_H + SECTION_PAD_BOT,
                    fill = theme.body_bg
                ));
                y += SECTION_PAD_TOP;
                for line in lines {
                    let keyword = line.split_whitespace().next().unwrap_or("");
                    let keyword_color = match keyword {
                        "Given" => "#7a3ea5",
                        "When" => "#c47a1e",
                        "Then" => "#1a6b30",
                        _ => theme.muted_fg,
                    };
                    let rest = line[keyword.len()..].trim();
                    svg_parts.push(format!(
                        "<text x=\"{x:.1}\" y=\"{ty:.1}\" font-size=\"{fs}\" font-style=\"italic\">\
                         <tspan font-weight=\"700\" fill=\"{kc}\">{kw} </tspan>\
                         <tspan fill=\"{fg}\">{rest}</tspan></text>",
                        x = CONTENT_X,
                        ty = y + m.cap_height(FS_DOC),
                        fs = FS_DOC,
                        kc = keyword_color,
                        kw = esc(keyword),
                        fg = theme.muted_fg,
                        rest = esc(rest)
                    ));
                    y += ROW_H;
                }
                y += SECTION_PAD_BOT;
            }
        }
    }

    // ── IBD port area: extend body height and render port squares ────────────
    if node.ibd {
        let left_area_h: f64 = ibd_left_ports.iter().map(port_row_h).sum::<f64>();
        let right_area_h: f64 = ibd_right_ports.iter().map(port_row_h).sum::<f64>();
        let port_area_h = left_area_h.max(right_area_h) + IBD_PORT_PAD_V * 2.0;
        let min_h = header_bottom + port_area_h;
        if y < min_h {
            y = min_h;
        }

        let body_h = y - header_bottom;

        // Body background
        svg_parts.push(format!(
            "<rect x=\"0\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" fill=\"{fill}\"/>",
            y = header_bottom, w = box_width, h = body_h, fill = theme.body_bg
        ));

        // Left border ports (In / InOut / Undirected)
        let mut ly = header_bottom + IBD_PORT_PAD_V;
        for row in &ibd_left_ports {
            let rh = port_row_h(row);
            let cy = ly + rh / 2.0;
            render_ibd_port(
                &mut svg_parts,
                &mut port_anchors,
                row,
                cy,
                box_width,
                theme,
                true, // left side
            );
            ly += rh;
        }

        // Right border ports (Out)
        let mut ry = header_bottom + IBD_PORT_PAD_V;
        for row in &ibd_right_ports {
            let rh = port_row_h(row);
            let cy = ry + rh / 2.0;
            render_ibd_port(
                &mut svg_parts,
                &mut port_anchors,
                row,
                cy,
                box_width,
                theme,
                false, // right side
            );
            ry += rh;
        }
    }

    let box_height = y;

    // ── Accent strip (left edge, full height) — drawn before border ──────────
    let accent_strip = format!(
        "<rect x=\"0\" y=\"0\" width=\"{w:.0}\" height=\"{h:.1}\" fill=\"{fill}\"/>",
        w = ACCENT_STRIP_W as u32, h = box_height, fill = theme.accent
    );

    // ── Outer border ─────────────────────────────────────────────────────────
    let border_rect = format!(
        "<rect x=\"{bw:.1}\" y=\"{bw:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" \
         rx=\"{r}\" ry=\"{r}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{bw2}\"/>",
        bw = BORDER_W / 2.0,
        w = box_width - BORDER_W,
        h = box_height - BORDER_W,
        r = CORNER_R,
        stroke = theme.border,
        bw2 = BORDER_W,
    );

    // ── Assemble SVG ─────────────────────────────────────────────────────────
    let inner = svg_parts.join("\n");
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" \
         viewBox=\"0 0 {w:.0} {h:.0}\" width=\"{w:.0}\" height=\"{h:.0}\">\n\
         {inner}\n\
         {accent}\n\
         {border}\n\
         </svg>",
        w = box_width, h = box_height,
        inner = inner,
        accent = accent_strip,
        border = border_rect
    );

    RenderedElement {
        qualified_name: node.qualified_name.clone(),
        svg,
        width: box_width,
        height: box_height,
        port_anchors,
    }
}

/// Render one IBD port (flat or nested) and push its anchor(s).
///
/// `cy` is the vertical center of this port's row (in element-local coords).
/// `is_left` true → port on left border (In/InOut), false → right border (Out).
///
/// Anchor x for a left port:  `-PORT_SQ/2`   (outer edge of port square, to the left of the block)
/// Anchor x for a right port: `box_width + PORT_SQ/2` (outer edge, to the right)
fn render_ibd_port(
    svg_parts: &mut Vec<String>,
    port_anchors: &mut Vec<PortAnchor>,
    row: &PortRow,
    cy: f64,
    box_width: f64,
    theme: &ElementTheme,
    is_left: bool,
) {
    let fh = port_frame_h(row);

    if row.sub_ports.is_empty() {
        // ── Flat port ───────────────────────────────────────────────────────
        let sq_x = if is_left {
            -PORT_SQ / 2.0
        } else {
            box_width - PORT_SQ / 2.0
        };
        let sq_y = cy - PORT_SQ / 2.0;

        svg_parts.push(format!(
            "<rect x=\"{sx:.1}\" y=\"{sy:.1}\" width=\"{sz:.0}\" height=\"{sz:.0}\" \
             fill=\"#ffffff\" stroke=\"{stroke}\" stroke-width=\"1.2\"/>",
            sx = sq_x, sy = sq_y, sz = PORT_SQ, stroke = theme.border
        ));

        let label = port_name_label(row);
        if is_left {
            svg_parts.push(format!(
                "<text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"9\" fill=\"{fg}\">{text}</text>",
                tx = PORT_SQ / 2.0 + 3.0, ty = cy + 3.5,
                fg = theme.muted_fg, text = esc(&label)
            ));
        } else {
            svg_parts.push(format!(
                "<text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"9\" fill=\"{fg}\" \
                 text-anchor=\"end\">{text}</text>",
                tx = box_width - PORT_SQ / 2.0 - 3.0, ty = cy + 3.5,
                fg = theme.muted_fg, text = esc(&label)
            ));
        }

        // Anchor at the outer edge of the port square
        let anchor_x = if is_left { -PORT_SQ / 2.0 } else { box_width + PORT_SQ / 2.0 };
        port_anchors.push(PortAnchor {
            name: row.name.clone(),
            x: anchor_x,
            y: cy,
            side: if is_left { "left" } else { "right" }.to_string(),
            direction: format!("{:?}", row.direction).to_lowercase(),
        });
    } else {
        // ── Nested port ─────────────────────────────────────────────────────
        // Outer frame: hollow rect straddling the block border
        let fw = PORT_SQ_OUTER_W;
        let frame_x = if is_left { -fw / 2.0 } else { box_width - fw / 2.0 };
        let frame_y = cy - fh / 2.0;

        svg_parts.push(format!(
            "<rect x=\"{fx:.1}\" y=\"{fy:.1}\" width=\"{fw:.0}\" height=\"{fh:.1}\" \
             fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"1.2\"/>",
            fx = frame_x, fy = frame_y, fw = fw, fh = fh,
            fill = theme.body_bg, stroke = theme.border
        ));

        // Outer port name label
        let label = port_name_label(row);
        if is_left {
            svg_parts.push(format!(
                "<text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"9\" fill=\"{fg}\">{text}</text>",
                tx = fw / 2.0 + 3.0, ty = cy + 3.5,
                fg = theme.muted_fg, text = esc(&label)
            ));
        } else {
            svg_parts.push(format!(
                "<text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"9\" fill=\"{fg}\" \
                 text-anchor=\"end\">{text}</text>",
                tx = box_width - fw / 2.0 - 3.0, ty = cy + 3.5,
                fg = theme.muted_fg, text = esc(&label)
            ));
        }

        // Inner sub-port squares, stacked inside the outer frame
        for (i, sub) in row.sub_ports.iter().enumerate() {
            let inner_y = frame_y + PORT_INNER_PAD + i as f64 * (PORT_SQ_INNER + PORT_INNER_PAD);
            let inner_x = if is_left {
                frame_x + (fw - PORT_SQ_INNER) / 2.0
            } else {
                frame_x + (fw - PORT_SQ_INNER) / 2.0
            };
            let inner_color = sub.direction.color();
            svg_parts.push(format!(
                "<rect x=\"{ix:.1}\" y=\"{iy:.1}\" width=\"{sz:.0}\" height=\"{sz:.0}\" \
                 fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"1\"/>",
                ix = inner_x, iy = inner_y, sz = PORT_SQ_INNER,
                fill = inner_color, stroke = theme.border
            ));

            // Sub-port anchor: same outer edge as parent, at sub-port's vertical centre
            let sub_cy = inner_y + PORT_SQ_INNER / 2.0;
            let anchor_x = if is_left { -fw / 2.0 } else { box_width + fw / 2.0 };
            port_anchors.push(PortAnchor {
                name: format!("{}.{}", row.name, sub.name),
                x: anchor_x,
                y: sub_cy,
                side: if is_left { "left" } else { "right" }.to_string(),
                direction: format!("{:?}", sub.direction).to_lowercase(),
            });
        }

        // Outer port anchor: mid-height of the entire frame, at outer edge
        let anchor_x = if is_left { -fw / 2.0 } else { box_width + fw / 2.0 };
        port_anchors.push(PortAnchor {
            name: row.name.clone(),
            x: anchor_x,
            y: cy,
            side: if is_left { "left" } else { "right" }.to_string(),
            direction: format!("{:?}", row.direction).to_lowercase(),
        });
    }
}

fn divider_line(y: f64, width: f64, color: &str) -> String {
    format!(
        "<line x1=\"0\" y1=\"{y:.1}\" x2=\"{w:.1}\" y2=\"{y:.1}\" \
         stroke=\"{c}\" stroke-width=\"{dh}\"/>",
        y = y, w = width, c = color, dh = DIVIDER_H
    )
}

fn section_title(title: &str, y: f64, muted_fg: &str, m: &dyn TextMetrics) -> String {
    format!(
        "<text x=\"{x:.1}\" y=\"{ty:.1}\" font-size=\"{fs}\" fill=\"{fg}\" \
         font-style=\"italic\">{text}</text>",
        x = CONTENT_X,
        ty = y + m.cap_height(FS_SECTION_TITLE),
        fs = FS_SECTION_TITLE,
        fg = muted_fg,
        text = esc(title)
    )
}

/// Sequence diagram renderer (V1: lifelines + messages, no activation bars/fragments).
///
/// Input: a `Diagram` element with `diagramKind: Sequence`, containing:
///   shapes: ordered map of ll-* (actor/lifeline) keys
///   edges:  ordered map of e-* message keys
///
/// Output: SVG string rendered via the `svg` crate.
use std::collections::HashMap;

use svg::Document;
use svg::node::element::{Anchor, Definitions, Group, Line, Marker, Path, Polygon, Rectangle, Text, Title};
use syscribe_model::element::RawElement;

// ── Layout constants ──────────────────────────────────────────────────────────
const LL_W: f64 = 140.0;
const LL_H: f64 = 44.0;
const LL_GAP: f64 = 60.0;
const MSG_GAP: f64 = 36.0;
const MSG_START_Y: f64 = 20.0;
const ACCENT_W: f64 = 4.0;
const PADDING: f64 = 32.0;
const SELF_LOOP_W: f64 = 32.0;
const FONT: &str = "ui-monospace, 'Cascadia Code', 'Fira Code', monospace";
const HEADER_FONT: &str = "'Segoe UI', system-ui, sans-serif";

// ── Theme colours ─────────────────────────────────────────────────────────────
const ACCENT: &str = "#3a6ea5";
const HEADER_BG: &str = "#eef3fa";
const HEADER_FG: &str = "#1a2540";
const BORDER: &str = "#a8bcd4";
const LIFELINE_STROKE: &str = "#a0b0c8";
const MSG_SOLID: &str = "#3a6ea5";
const MSG_RETURN: &str = "#7a8aaa";
const TEXT_FG: &str = "#2a3a5a";
const BG: &str = "#ffffff";

// ── Input parsing ─────────────────────────────────────────────────────────────

struct Lifeline {
    key: String,
    name: String,
    is_actor: bool,
    ref_qname: String,
}

struct Message {
    label: String,
    source: String,
    target: String,
    is_return: bool,
}

fn parse_shapes(shapes: &serde_yaml::Value) -> Vec<Lifeline> {
    let mut lifelines = Vec::new();
    let Some(map) = shapes.as_mapping() else { return lifelines };
    for (k, v) in map {
        let key = k.as_str().unwrap_or("").to_string();
        let kind = v.get("kind").and_then(|x| x.as_str()).unwrap_or("lifeline");
        if kind != "lifeline" && kind != "actor" {
            continue;
        }
        let ref_val = v.get("ref").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let ref_val = if ref_val.is_empty() { key.clone() } else { ref_val };
        let name = short_name(&ref_val);
        lifelines.push(Lifeline { key, name, is_actor: kind == "actor", ref_qname: ref_val });
    }
    lifelines
}

fn parse_edges(edges: &serde_yaml::Value) -> Vec<Message> {
    let mut messages = Vec::new();
    let Some(map) = edges.as_mapping() else { return messages };
    for (_k, v) in map {
        let ref_val = v.get("ref").and_then(|x| x.as_str()).unwrap_or("");
        let source = v.get("source").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let target = v.get("target").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let kind = v.get("kind").and_then(|x| x.as_str()).unwrap_or("message");
        messages.push(Message {
            label: short_name(ref_val),
            source,
            target,
            is_return: kind == "return",
        });
    }
    messages
}

fn short_name(qname: &str) -> String {
    qname.rsplit("::").next().unwrap_or(qname).to_string()
}

// ── Geometry helpers ──────────────────────────────────────────────────────────

fn ll_cx(i: usize) -> f64 {
    PADDING + i as f64 * (LL_W + LL_GAP) + LL_W / 2.0
}

fn ll_header_bottom() -> f64 {
    PADDING + LL_H
}

fn first_msg_y() -> f64 {
    ll_header_bottom() + MSG_START_Y + MSG_GAP
}

fn msg_y(idx: usize) -> f64 {
    first_msg_y() + idx as f64 * MSG_GAP
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn cmd_diagram_seq(elements: &[RawElement], qname: &str, output: Option<&str>) {
    let elem = match elements.iter().find(|e| e.qualified_name == qname) {
        Some(e) => e,
        None => {
            eprintln!("error: diagram element '{}' not found", qname);
            std::process::exit(1);
        }
    };

    let fm = &elem.frontmatter;
    let shapes_val = match &fm.shapes {
        Some(s) => s,
        None => {
            eprintln!("error: '{}' has no shapes", qname);
            std::process::exit(1);
        }
    };
    let edges_val = match &fm.edges {
        Some(e) => e,
        None => {
            eprintln!("error: '{}' has no edges", qname);
            std::process::exit(1);
        }
    };

    let lifelines = parse_shapes(shapes_val);
    let messages = parse_edges(edges_val);

    if lifelines.is_empty() {
        eprintln!("warn: no lifelines found in '{}'", qname);
    }

    let ll_index: HashMap<&str, usize> =
        lifelines.iter().enumerate().map(|(i, ll)| (ll.key.as_str(), i)).collect();

    let n_ll = lifelines.len();
    let n_msg = messages.len();

    let canvas_w =
        PADDING * 2.0 + n_ll as f64 * LL_W + n_ll.saturating_sub(1) as f64 * LL_GAP;
    let lifeline_len = MSG_START_Y + (n_msg as f64 + 1.0) * MSG_GAP + 20.0;
    let canvas_h = PADDING + LL_H + lifeline_len + PADDING;

    let title_str = fm.title.clone()
        .or_else(|| fm.name.clone())
        .unwrap_or_else(|| qname.to_string());

    let mut doc = Document::new()
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("width", canvas_w)
        .set("height", canvas_h)
        .set("viewBox", format!("0 0 {canvas_w:.0} {canvas_h:.0}"))
        .set("font-family", HEADER_FONT)
        .set("font-size", "12")
        .add(Title::new(title_str))
        .add(
            Rectangle::new()
                .set("x", 0)
                .set("y", 0)
                .set("width", canvas_w)
                .set("height", canvas_h)
                .set("fill", BG),
        )
        .add(arrowhead_defs());

    // Lifeline headers and dashed lines — each wrapped in an anchor + data-qname
    for (i, ll) in lifelines.iter().enumerate() {
        if let Some(elem) = elements.iter().find(|e| e.qualified_name == ll.ref_qname) {
            let qname = &elem.qualified_name;
            let anchor = Anchor::new()
                .set("href", format!("/ui/detail/{}", qname))
                .set("data-qname", qname.clone())
                .add(lifeline_group(i, ll, lifeline_len));
            doc = doc.add(anchor);
        } else {
            doc = doc.add(lifeline_group(i, ll, lifeline_len));
        }
    }

    // Messages
    for (idx, msg) in messages.iter().enumerate() {
        let y = msg_y(idx);
        let src_i = ll_index.get(msg.source.as_str()).copied();
        let tgt_i = ll_index.get(msg.target.as_str()).copied();
        match (src_i, tgt_i) {
            (Some(si), Some(ti)) if si == ti => {
                doc = doc.add(self_message_group(si, y, &msg.label, msg.is_return));
            }
            (Some(si), Some(ti)) => {
                doc = doc.add(message_group(si, ti, y, &msg.label, msg.is_return));
            }
            _ => {
                eprintln!(
                    "warn: message '{}' references unknown lifeline(s) src='{}' tgt='{}'",
                    msg.label, msg.source, msg.target
                );
            }
        }
    }

    let svg_str = format!("{doc}");
    match output {
        Some(path) => {
            if let Err(e) = std::fs::write(path, svg_str) {
                eprintln!("error writing SVG to '{}': {}", path, e);
                std::process::exit(1);
            }
        }
        None => print!("{}", svg_str),
    }
}

// ── Lifeline rendering ────────────────────────────────────────────────────────

fn lifeline_group(i: usize, ll: &Lifeline, line_len: f64) -> Group {
    let x = PADDING + i as f64 * (LL_W + LL_GAP);
    let y = PADDING;
    let cx = ll_cx(i);
    let line_top = ll_header_bottom();
    let line_bot = ll_header_bottom() + line_len;

    let mut g = Group::new();

    // Header box
    g = g.add(
        Rectangle::new()
            .set("x", x)
            .set("y", y)
            .set("width", LL_W)
            .set("height", LL_H)
            .set("fill", HEADER_BG)
            .set("stroke", BORDER)
            .set("stroke-width", 1)
            .set("rx", 1),
    );
    // Accent strip
    g = g.add(
        Rectangle::new()
            .set("x", x)
            .set("y", y)
            .set("width", ACCENT_W)
            .set("height", LL_H)
            .set("fill", ACCENT)
            .set("rx", 1),
    );

    if ll.is_actor {
        g = g.add(
            Text::new("«actor»")
                .set("x", cx)
                .set("y", y + 14.0)
                .set("text-anchor", "middle")
                .set("font-size", "9")
                .set("fill", ACCENT)
                .set("font-style", "italic"),
        );
        g = g.add(
            Text::new(ll.name.clone())
                .set("x", cx)
                .set("y", y + 30.0)
                .set("text-anchor", "middle")
                .set("fill", HEADER_FG)
                .set("font-weight", "600")
                .set("font-size", "11"),
        );
    } else {
        g = g.add(
            Text::new(ll.name.clone())
                .set("x", cx)
                .set("y", y + 26.0)
                .set("text-anchor", "middle")
                .set("fill", HEADER_FG)
                .set("font-weight", "600")
                .set("font-size", "11"),
        );
    }

    // Dashed lifeline
    g = g.add(
        Line::new()
            .set("x1", cx)
            .set("y1", line_top)
            .set("x2", cx)
            .set("y2", line_bot)
            .set("stroke", LIFELINE_STROKE)
            .set("stroke-width", 1)
            .set("stroke-dasharray", "6 4"),
    );

    g
}

// ── Message rendering ─────────────────────────────────────────────────────────

fn message_group(src: usize, tgt: usize, y: f64, label: &str, is_return: bool) -> Group {
    let x1 = ll_cx(src);
    let x2 = ll_cx(tgt);
    let stroke = if is_return { MSG_RETURN } else { MSG_SOLID };
    let marker = if is_return { "url(#arr-return)" } else { "url(#arr-msg)" };

    let mut arrow = Path::new()
        .set("d", format!("M {x1:.1} {y:.1} L {x2:.1} {y:.1}"))
        .set("stroke", stroke)
        .set("stroke-width", 1.5)
        .set("fill", "none")
        .set("marker-end", marker);
    if is_return {
        arrow = arrow.set("stroke-dasharray", "6 3");
    }

    let lx = (x1 + x2) / 2.0;
    Group::new()
        .add(arrow)
        .add(
            Text::new(label.to_string())
                .set("x", lx)
                .set("y", y - 5.0)
                .set("text-anchor", "middle")
                .set("font-family", FONT)
                .set("font-size", "10")
                .set("fill", TEXT_FG),
        )
}

fn self_message_group(ll: usize, y: f64, label: &str, is_return: bool) -> Group {
    let cx = ll_cx(ll);
    let x1 = cx;
    let x2 = cx + SELF_LOOP_W;
    let y1 = y;
    let y2 = y + MSG_GAP * 0.6;
    let stroke = if is_return { MSG_RETURN } else { MSG_SOLID };
    let marker = if is_return { "url(#arr-return)" } else { "url(#arr-msg)" };

    let d = format!("M {x1:.1} {y1:.1} L {x2:.1} {y1:.1} L {x2:.1} {y2:.1} L {x1:.1} {y2:.1}");
    let mut arrow = Path::new()
        .set("d", d)
        .set("stroke", stroke)
        .set("stroke-width", 1.5)
        .set("fill", "none")
        .set("marker-end", marker);
    if is_return {
        arrow = arrow.set("stroke-dasharray", "6 3");
    }

    Group::new()
        .add(arrow)
        .add(
            Text::new(label.to_string())
                .set("x", cx + SELF_LOOP_W + 4.0)
                .set("y", (y1 + y2) / 2.0 + 4.0)
                .set("font-family", FONT)
                .set("font-size", "10")
                .set("fill", TEXT_FG),
        )
}

// ── Arrowhead marker defs ─────────────────────────────────────────────────────

fn arrowhead_defs() -> Definitions {
    let arr_msg = Marker::new()
        .set("id", "arr-msg")
        .set("markerWidth", 8)
        .set("markerHeight", 6)
        .set("refX", 8)
        .set("refY", 3)
        .set("orient", "auto")
        .add(Polygon::new().set("points", "0 0, 8 3, 0 6").set("fill", MSG_SOLID));

    let arr_return = Marker::new()
        .set("id", "arr-return")
        .set("markerWidth", 8)
        .set("markerHeight", 6)
        .set("refX", 8)
        .set("refY", 3)
        .set("orient", "auto")
        .add(
            Path::new()
                .set("d", "M 0 0 L 8 3 L 0 6")
                .set("fill", "none")
                .set("stroke", MSG_RETURN)
                .set("stroke-width", 1.5),
        );

    Definitions::new().add(arr_msg).add(arr_return)
}

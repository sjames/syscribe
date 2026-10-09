//! The outlines of the safety diagram symbols (GH #223): fault-tree gates and
//! events, the attack-tree step, and the GSN node shapes.
//!
//! One definition of the geometry, used by the SVG writer; the browser client
//! (`frontend/src/views.tsx`, `safetySymbol`) mirrors it path for path, so the
//! static export and the editor draw the same symbol.
//!
//! A fault-tree gate or event is drawn the way the standard draws it: a
//! description box (the id, the wrapped name, the analysis overlay) with the
//! logic symbol hung beneath it on a short stub. The symbol is a constant
//! size, so it never has to hold text; a node's box is its text stack plus
//! [`glyph_extent`].

use super::ir::NodeKind;

/// The closed outline of a symbol.
#[derive(Debug, Clone, PartialEq)]
pub enum Outline {
    /// A rectangle with corner radius `rx` (0 = square).
    Rect { rx: f64 },
    /// The ellipse inscribed in the box.
    Ellipse,
    /// A closed SVG path in absolute coordinates (possibly several subpaths).
    Path(String),
}

/// A decoration drawn with the node's stroke on top of the outline.
#[derive(Debug, Clone, PartialEq)]
pub enum Extra {
    /// An open stroke (`d` attribute).
    Stroke(String),
    /// A circle with a white fill (the NOT bubble).
    Circle { cx: f64, cy: f64, r: f64 },
    /// A closed path with a white fill (GSN's undeveloped diamond).
    Diamond(String),
    /// A single letter at `(x, y)` (GSN's `J` and `A`).
    Letter { x: f64, y: f64, text: &'static str },
    /// A short bold word centred on `x` with its baseline at `y` (the `AND`,
    /// `OR`, … inside a gate symbol).
    Word { x: f64, y: f64, text: &'static str },
}

/// A symbol: its outline and decorations.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub outline: Outline,
    pub extras: Vec<Extra>,
}

/// Width and height of a gate symbol.
pub const GATE_W: f64 = 56.0;
pub const GATE_H: f64 = 48.0;
/// The line joining the description box to its symbol.
pub const STUB: f64 = 10.0;
/// Height of GSN's undeveloped-goal diamond strip under the goal box.
pub const UNDEVELOPED_STRIP: f64 = 18.0;
/// The stub and the gate symbol: what a gate adds below its text stack.
const GLYPH_GATE: f64 = STUB + GATE_H;

fn n(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r.fract() == 0.0 {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

fn polygon(pts: &[(f64, f64)]) -> String {
    let mut d = String::new();
    for (i, (x, y)) in pts.iter().enumerate() {
        d.push_str(&format!("{} {},{} ", if i == 0 { "M" } else { "L" }, n(*x), n(*y)));
    }
    d.push('Z');
    d
}

/// The closed rectangle `(x, y, w, h)` as a subpath.
fn rect_path(x: f64, y: f64, w: f64, h: f64) -> String {
    polygon(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)])
}

/// A full circle as a closed subpath.
fn circle_path(cx: f64, cy: f64, r: f64) -> String {
    format!("M {},{} A {},{} 0 1 0 {},{} A {},{} 0 1 0 {},{} Z", n(cx - r), n(cy), n(r), n(r), n(cx + r), n(cy), n(r), n(r), n(cx - r), n(cy))
}

/// The `(width, height)` of the event symbol hung under an event's box.
fn event_symbol(kind: NodeKind) -> Option<(f64, f64)> {
    match kind {
        NodeKind::EventBasic => Some((28.0, 28.0)),
        NodeKind::EventUndeveloped => Some((40.0, 26.0)),
        NodeKind::EventHouse => Some((34.0, 30.0)),
        _ => None,
    }
}

fn is_gate(kind: NodeKind) -> bool {
    matches!(kind, NodeKind::GateAnd | NodeKind::GateOr | NodeKind::GateXor | NodeKind::GateNot | NodeKind::GateInhibit)
}

/// Extra height a symbol adds below the text stack of its node (the stub and
/// the logic symbol of a fault-tree gate or event, the diamond strip of an
/// undeveloped goal), or `None` for a kind drawn within its text box.
/// `size.rs` adds it; [`shape_of`] and `safety-shape.ts` derive the
/// description box from it.
pub fn glyph_extent(kind: NodeKind) -> Option<f64> {
    if is_gate(kind) {
        return Some(GLYPH_GATE);
    }
    if kind == NodeKind::UndevelopedGoal {
        return Some(UNDEVELOPED_STRIP);
    }
    event_symbol(kind).map(|(_, h)| STUB + h)
}

/// The least width the symbol of `kind` needs (the symbol and a margin), 0
/// when it has none.
pub fn glyph_min_width(kind: NodeKind) -> f64 {
    if is_gate(kind) {
        return GATE_W + 16.0;
    }
    event_symbol(kind).map(|(w, _)| w + 16.0).unwrap_or(0.0)
}

/// The OR-gate shield: pointed top, bowed sides, concave bottom.
fn or_path(x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        "M {},{} Q {},{} {},{} C {},{} {},{} {},{} C {},{} {},{} {},{} Z",
        n(x),
        n(y + h),
        n(x + w / 2.0),
        n(y + 0.72 * h),
        n(x + w),
        n(y + h),
        n(x + w),
        n(y + 0.5 * h),
        n(x + 0.7 * w),
        n(y + 0.12 * h),
        n(x + w / 2.0),
        n(y),
        n(x + 0.3 * w),
        n(y + 0.12 * h),
        n(x),
        n(y + 0.5 * h),
        n(x),
        n(y + h),
    )
}

/// How much larger than its label stack a symbol's box must be for the text to
/// sit inside the outline: `(width factor, height factor)`, or `None` for a kind
/// drawn as a plain box (or, for a gate and an event, as a text box with its
/// symbol beneath it: [`glyph_extent`]). The labels of a symbol are centred in
/// it ([`is_symbol`]; `frontend/src/layout.ts` `isSymbolKind` mirrors this).
pub fn text_scale(kind: NodeKind) -> Option<(f64, f64)> {
    Some(match kind {
        // The text box inscribed in an ellipse is 1/sqrt(2) of its axes.
        NodeKind::Solution | NodeKind::Justification | NodeKind::Assumption => (1.42, 1.5),
        NodeKind::Strategy => (1.25, 1.1),
        NodeKind::Context => (1.1, 1.15),
        _ => return None,
    })
}

/// Whether the kind's label stack is centred inside an outline narrower than its box.
pub fn is_symbol(kind: NodeKind) -> bool {
    text_scale(kind).is_some()
}

/// The widest a line of text of `kind` may be before it wraps (the name of a
/// fault-tree event, the statement of a GSN node), or `None` for a kind whose
/// text is never wrapped. `size.rs` wraps at this width.
pub fn wrap_width(kind: NodeKind) -> Option<f64> {
    match kind {
        k if is_gate(k) => Some(150.0),
        NodeKind::EventBasic | NodeKind::EventUndeveloped | NodeKind::EventHouse | NodeKind::Step | NodeKind::Goal | NodeKind::UndevelopedGoal => Some(150.0),
        NodeKind::Strategy | NodeKind::Context => Some(130.0),
        NodeKind::Solution | NodeKind::Justification | NodeKind::Assumption => Some(110.0),
        _ => None,
    }
}

/// The gate glyph of `kind` with its top-left at `(gx, gy)`: its closed
/// outline, then the strokes, bubble and word drawn over it.
fn gate_glyph(kind: NodeKind, gx: f64, gy: f64) -> (String, Vec<Extra>) {
    let (w, h) = (GATE_W, GATE_H);
    let cx = gx + w / 2.0;
    let word = |text: &'static str, f: f64| Extra::Word { x: cx, y: gy + f * h, text };
    match kind {
        NodeKind::GateAnd => (
            format!(
                "M {},{} L {},{} A {},{} 0 0 1 {},{} L {},{} Z",
                n(gx),
                n(gy + h),
                n(gx),
                n(gy + 0.5 * h),
                n(w / 2.0),
                n(0.5 * h),
                n(gx + w),
                n(gy + 0.5 * h),
                n(gx + w),
                n(gy + h)
            ),
            vec![word("AND", 0.78)],
        ),
        NodeKind::GateOr => (or_path(gx, gy, w, h), vec![word("OR", 0.72)]),
        NodeKind::GateXor => {
            // The OR shield, shortened by the gap of the extra arc beneath it.
            let hh = h - 6.0;
            (
                or_path(gx, gy, w, hh),
                vec![
                    Extra::Stroke(format!("M {},{} Q {},{} {},{}", n(gx), n(gy + h), n(cx), n(gy + 0.72 * hh + 6.0), n(gx + w), n(gy + h))),
                    word("XOR", 0.62),
                ],
            )
        }
        NodeKind::GateNot => (
            polygon(&[(cx, gy + 10.0), (gx + w, gy + h), (gx, gy + h)]),
            vec![Extra::Circle { cx, cy: gy + 5.0, r: 5.0 }, word("NOT", 0.93)],
        ),
        // The inhibit gate: a hexagon.
        _ => (
            polygon(&[(gx + 0.2 * w, gy), (gx + 0.8 * w, gy), (gx + w, gy + h / 2.0), (gx + 0.8 * w, gy + h), (gx + 0.2 * w, gy + h), (gx, gy + h / 2.0)]),
            vec![word("INH", 0.58)],
        ),
    }
}

/// The symbol of a safety-diagram node kind inside the box `(x, y, w, h)`,
/// or `None` for every other kind (drawn as a plain box).
pub fn shape_of(kind: NodeKind, x: f64, y: f64, w: f64, h: f64) -> Option<Shape> {
    let shape = |outline: Outline, extras: Vec<Extra>| Some(Shape { outline, extras });
    let cx = x + w / 2.0;
    match kind {
        k if is_gate(k) => {
            let rh = h - GLYPH_GATE;
            let (gate, mut extras) = gate_glyph(k, cx - GATE_W / 2.0, y + rh + STUB);
            extras.insert(0, Extra::Stroke(format!("M {},{} L {},{}", n(cx), n(y + rh), n(cx), n(y + rh + STUB))));
            shape(Outline::Path(format!("{} {}", rect_path(x, y, w, rh), gate)), extras)
        }
        NodeKind::EventBasic | NodeKind::EventUndeveloped | NodeKind::EventHouse => {
            let (sw, sh) = event_symbol(kind).unwrap_or((0.0, 0.0));
            let rh = h - STUB - sh;
            let (sx, sy) = (cx - sw / 2.0, y + rh + STUB);
            let sym = match kind {
                NodeKind::EventBasic => circle_path(cx, sy + sh / 2.0, sh / 2.0),
                NodeKind::EventUndeveloped => polygon(&[(cx, sy), (sx + sw, sy + sh / 2.0), (cx, sy + sh), (sx, sy + sh / 2.0)]),
                _ => polygon(&[(sx, sy + 0.4 * sh), (cx, sy), (sx + sw, sy + 0.4 * sh), (sx + sw, sy + sh), (sx, sy + sh)]),
            };
            shape(
                Outline::Path(format!("{} {}", rect_path(x, y, w, rh), sym)),
                vec![Extra::Stroke(format!("M {},{} L {},{}", n(cx), n(y + rh), n(cx), n(y + rh + STUB)))],
            )
        }
        NodeKind::Solution => shape(Outline::Ellipse, vec![]),
        NodeKind::Step => shape(Outline::Rect { rx: 3.0 }, vec![]),
        NodeKind::Goal => shape(Outline::Rect { rx: 0.0 }, vec![]),
        NodeKind::UndevelopedGoal => {
            let rh = h - UNDEVELOPED_STRIP;
            let (cy, r) = (y + rh + UNDEVELOPED_STRIP / 2.0, 8.0);
            shape(Outline::Path(rect_path(x, y, w, rh)), vec![Extra::Diamond(polygon(&[(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)]))])
        }
        NodeKind::Strategy => shape(Outline::Path(polygon(&[(x + 0.1 * w, y), (x + w, y), (x + 0.9 * w, y + h), (x, y + h)])), vec![]),
        NodeKind::Context => shape(Outline::Rect { rx: h / 2.0 }, vec![]),
        NodeKind::Justification => shape(Outline::Ellipse, vec![Extra::Letter { x: x + w - 9.0, y: y + h - 2.0, text: "J" }]),
        NodeKind::Assumption => shape(Outline::Ellipse, vec![Extra::Letter { x: x + w - 9.0, y: y + h - 2.0, text: "A" }]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_safety_kind_has_a_symbol_and_other_kinds_none() {
        for k in [
            NodeKind::GateAnd,
            NodeKind::GateOr,
            NodeKind::GateXor,
            NodeKind::GateNot,
            NodeKind::GateInhibit,
            NodeKind::EventBasic,
            NodeKind::EventUndeveloped,
            NodeKind::EventHouse,
            NodeKind::Step,
            NodeKind::Goal,
            NodeKind::UndevelopedGoal,
            NodeKind::Strategy,
            NodeKind::Solution,
            NodeKind::Context,
            NodeKind::Justification,
            NodeKind::Assumption,
        ] {
            assert!(shape_of(k, 0.0, 0.0, 120.0, 100.0).is_some(), "{k:?}");
        }
        assert!(shape_of(NodeKind::Block, 0.0, 0.0, 120.0, 40.0).is_none());
    }

    fn path(k: NodeKind) -> String {
        match shape_of(k, 0.0, 0.0, 100.0, 90.0).unwrap().outline {
            Outline::Path(d) => d,
            other => panic!("{other:?}"),
        }
    }

    /// The same literals `frontend/test/safety-shape.test.mjs` pins, so the
    /// editor's `safety-shape.ts` and this writer cannot drift apart.
    #[test]
    fn outlines_match_the_clients_literals() {
        // A 100x90 box: a 100x32 description box, a 10 stub and the 56x48 gate at x 22..78.
        assert_eq!(
            path(NodeKind::GateAnd),
            "M 0,0 L 100,0 L 100,32 L 0,32 Z M 22,90 L 22,66 A 28,24 0 0 1 78,66 L 78,90 Z"
        );
        assert_eq!(
            path(NodeKind::GateOr),
            "M 0,0 L 100,0 L 100,32 L 0,32 Z M 22,90 Q 50,76.56 78,90 C 78,66 61.2,47.76 50,42 C 38.8,47.76 22,66 22,90 Z"
        );
        assert_eq!(
            path(NodeKind::GateInhibit),
            "M 0,0 L 100,0 L 100,32 L 0,32 Z M 33.2,42 L 66.8,42 L 78,66 L 66.8,90 L 33.2,90 L 22,66 Z"
        );
        assert_eq!(path(NodeKind::GateNot), "M 0,0 L 100,0 L 100,32 L 0,32 Z M 50,52 L 78,90 L 22,90 Z");
        // Events: a 100x(90-10-h) box over the symbol.
        assert_eq!(
            path(NodeKind::EventBasic),
            "M 0,0 L 100,0 L 100,52 L 0,52 Z M 36,76 A 14,14 0 1 0 64,76 A 14,14 0 1 0 36,76 Z"
        );
        assert_eq!(path(NodeKind::EventUndeveloped), "M 0,0 L 100,0 L 100,54 L 0,54 Z M 50,64 L 70,77 L 50,90 L 30,77 Z");
        assert_eq!(path(NodeKind::EventHouse), "M 0,0 L 100,0 L 100,50 L 0,50 Z M 33,72 L 50,60 L 67,72 L 67,90 L 33,90 Z");
        assert_eq!(path(NodeKind::Strategy), "M 10,0 L 100,0 L 90,90 L 0,90 Z");
        assert_eq!(path(NodeKind::UndevelopedGoal), "M 0,0 L 100,0 L 100,72 L 0,72 Z");
        let x = shape_of(NodeKind::GateXor, 0.0, 0.0, 100.0, 90.0).unwrap();
        assert_eq!(
            x.extras,
            vec![
                Extra::Stroke("M 50,32 L 50,42".to_string()),
                Extra::Stroke("M 22,90 Q 50,78.24 78,90".to_string()),
                Extra::Word { x: 50.0, y: 42.0 + 0.62 * 48.0, text: "XOR" },
            ]
        );
        let u = shape_of(NodeKind::UndevelopedGoal, 0.0, 0.0, 100.0, 90.0).unwrap();
        assert_eq!(u.extras, vec![Extra::Diamond("M 50,73 L 58,81 L 50,89 L 42,81 Z".to_string())]);
        assert!(is_symbol(NodeKind::Context) && !is_symbol(NodeKind::Goal) && !is_symbol(NodeKind::Step) && !is_symbol(NodeKind::GateOr));
    }

    #[test]
    fn xor_is_the_or_shield_plus_one_arc_and_not_carries_a_bubble() {
        let or = shape_of(NodeKind::GateOr, 0.0, 0.0, 100.0, 90.0).unwrap();
        let xor = shape_of(NodeKind::GateXor, 0.0, 0.0, 100.0, 90.0).unwrap();
        assert_ne!(or.outline, xor.outline, "the XOR shield is shortened to make room for its arc");
        assert!(xor.extras.iter().filter(|e| matches!(e, Extra::Stroke(_))).count() == 2);
        let not = shape_of(NodeKind::GateNot, 0.0, 0.0, 100.0, 90.0).unwrap();
        assert!(not.extras.iter().any(|e| matches!(e, Extra::Circle { cx, cy, r } if *cx == 50.0 && *cy == 47.0 && *r == 5.0)));
    }

    #[test]
    fn a_symbol_adds_its_extent_below_the_text_and_a_minimum_width() {
        assert_eq!(glyph_extent(NodeKind::GateAnd), Some(58.0));
        assert_eq!(glyph_extent(NodeKind::EventBasic), Some(38.0));
        assert_eq!(glyph_extent(NodeKind::UndevelopedGoal), Some(18.0));
        assert_eq!(glyph_extent(NodeKind::Goal), None);
        assert_eq!(glyph_min_width(NodeKind::GateOr), 72.0);
        assert_eq!(wrap_width(NodeKind::Block), None);
    }
}

//! The outlines of the safety diagram symbols (GH #223): fault-tree gates and
//! events, the attack-tree step, and the GSN node shapes.
//!
//! One definition of the geometry, used by the SVG writer; the browser client
//! (`frontend/src/views.tsx`, `safetyShape`) mirrors it path for path, so the
//! static export and the editor draw the same symbol.

use super::ir::NodeKind;

/// The closed outline of a symbol.
#[derive(Debug, Clone, PartialEq)]
pub enum Outline {
    /// A rectangle with corner radius `rx` (0 = square).
    Rect { rx: f64 },
    /// The ellipse inscribed in the box.
    Ellipse,
    /// A closed SVG path in absolute coordinates.
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
}

/// A symbol: its outline and decorations.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub outline: Outline,
    pub extras: Vec<Extra>,
}

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
/// drawn as a plain box. The labels are centred in the symbol
/// ([`is_symbol`]; `frontend/src/layout.ts` `isSymbolKind` mirrors this).
pub fn text_scale(kind: NodeKind) -> Option<(f64, f64)> {
    Some(match kind {
        // The text box inscribed in an ellipse is 1/sqrt(2) of its axes.
        NodeKind::EventBasic | NodeKind::Solution | NodeKind::Justification | NodeKind::Assumption => (1.42, 1.5),
        NodeKind::EventUndeveloped => (1.15, 1.35),
        NodeKind::EventHouse => (1.05, 1.5),
        NodeKind::GateOr | NodeKind::GateXor => (1.3, 1.6),
        NodeKind::GateAnd => (1.05, 1.5),
        NodeKind::GateNot => (1.05, 1.3),
        NodeKind::GateInhibit => (1.25, 1.3),
        NodeKind::Strategy => (1.25, 1.1),
        NodeKind::Context => (1.1, 1.15),
        _ => return None,
    })
}

/// Whether the kind's label stack is centred inside an outline narrower than its box.
pub fn is_symbol(kind: NodeKind) -> bool {
    text_scale(kind).is_some()
}

/// The symbol of a safety-diagram node kind inside the box `(x, y, w, h)`,
/// or `None` for every other kind (drawn as a plain box).
pub fn shape_of(kind: NodeKind, x: f64, y: f64, w: f64, h: f64) -> Option<Shape> {
    let shape = |outline: Outline, extras: Vec<Extra>| Some(Shape { outline, extras });
    match kind {
        NodeKind::GateAnd => shape(
            Outline::Path(format!(
                "M {},{} L {},{} A {},{} 0 0 1 {},{} L {},{} Z",
                n(x),
                n(y + h),
                n(x),
                n(y + 0.5 * h),
                n(w / 2.0),
                n(0.5 * h),
                n(x + w),
                n(y + 0.5 * h),
                n(x + w),
                n(y + h)
            )),
            vec![],
        ),
        NodeKind::GateOr => shape(Outline::Path(or_path(x, y, w, h)), vec![]),
        NodeKind::GateXor => shape(
            Outline::Path(or_path(x, y, w, h)),
            vec![Extra::Stroke(format!(
                "M {},{} Q {},{} {},{}",
                n(x),
                n(y + 0.86 * h),
                n(x + w / 2.0),
                n(y + 0.58 * h),
                n(x + w),
                n(y + 0.86 * h)
            ))],
        ),
        NodeKind::GateNot => shape(Outline::Rect { rx: 8.0 }, vec![Extra::Circle { cx: x + w / 2.0, cy: y + h, r: 5.0 }]),
        NodeKind::GateInhibit => shape(
            Outline::Path(polygon(&[
                (x + 0.12 * w, y),
                (x + 0.88 * w, y),
                (x + w, y + h / 2.0),
                (x + 0.88 * w, y + h),
                (x + 0.12 * w, y + h),
                (x, y + h / 2.0),
            ])),
            vec![],
        ),
        NodeKind::EventBasic | NodeKind::Solution => shape(Outline::Ellipse, vec![]),
        NodeKind::EventUndeveloped => shape(
            Outline::Path(polygon(&[
                (x + h / 2.0, y),
                (x + w - h / 2.0, y),
                (x + w, y + h / 2.0),
                (x + w - h / 2.0, y + h),
                (x + h / 2.0, y + h),
                (x, y + h / 2.0),
            ])),
            vec![],
        ),
        NodeKind::EventHouse => shape(
            Outline::Path(polygon(&[(x, y + 0.28 * h), (x + w / 2.0, y), (x + w, y + 0.28 * h), (x + w, y + h), (x, y + h)])),
            vec![],
        ),
        NodeKind::Step => shape(Outline::Rect { rx: 3.0 }, vec![]),
        NodeKind::Goal => shape(Outline::Rect { rx: 0.0 }, vec![]),
        NodeKind::UndevelopedGoal => {
            let (cx, cy, r) = (x + w / 2.0, y + h + 8.0, 8.0);
            shape(Outline::Rect { rx: 0.0 }, vec![Extra::Diamond(polygon(&[(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)]))])
        }
        NodeKind::Strategy => shape(
            Outline::Path(polygon(&[(x + 0.1 * w, y), (x + w, y), (x + 0.9 * w, y + h), (x, y + h)])),
            vec![],
        ),
        NodeKind::Context => shape(Outline::Rect { rx: h / 2.0 }, vec![]),
        NodeKind::Justification => shape(Outline::Ellipse, vec![Extra::Letter { x: x + w - 4.0, y: y + h + 11.0, text: "J" }]),
        NodeKind::Assumption => shape(Outline::Ellipse, vec![Extra::Letter { x: x + w - 4.0, y: y + h + 11.0, text: "A" }]),
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
            assert!(shape_of(k, 0.0, 0.0, 120.0, 40.0).is_some(), "{k:?}");
        }
        assert!(shape_of(NodeKind::Block, 0.0, 0.0, 120.0, 40.0).is_none());
    }

    /// The same literals `frontend/test/safety-shape.test.mjs` pins, so the
    /// editor's `safety-shape.ts` and this writer cannot drift apart.
    #[test]
    fn outlines_match_the_clients_literals() {
        let d = |k| match shape_of(k, 0.0, 0.0, 100.0, 50.0).unwrap().outline {
            Outline::Path(d) => d,
            other => panic!("{other:?}"),
        };
        assert_eq!(d(NodeKind::GateOr), "M 0,50 Q 50,36 100,50 C 100,25 70,6 50,0 C 30,6 0,25 0,50 Z");
        assert_eq!(d(NodeKind::GateAnd), "M 0,50 L 0,25 A 50,25 0 0 1 100,25 L 100,50 Z");
        assert_eq!(d(NodeKind::GateInhibit), "M 12,0 L 88,0 L 100,25 L 88,50 L 12,50 L 0,25 Z");
        assert_eq!(d(NodeKind::EventUndeveloped), "M 25,0 L 75,0 L 100,25 L 75,50 L 25,50 L 0,25 Z");
        assert_eq!(d(NodeKind::EventHouse), "M 0,14 L 50,0 L 100,14 L 100,50 L 0,50 Z");
        assert_eq!(d(NodeKind::Strategy), "M 10,0 L 100,0 L 90,50 L 0,50 Z");
        let x = shape_of(NodeKind::GateXor, 0.0, 0.0, 100.0, 50.0).unwrap();
        assert_eq!(x.extras, vec![Extra::Stroke("M 0,43 Q 50,29 100,43".to_string())]);
        let u = shape_of(NodeKind::UndevelopedGoal, 0.0, 0.0, 100.0, 50.0).unwrap();
        assert_eq!(u.extras, vec![Extra::Diamond("M 50,50 L 58,58 L 50,66 L 42,58 Z".to_string())]);
        assert!(is_symbol(NodeKind::Context) && !is_symbol(NodeKind::Goal) && !is_symbol(NodeKind::Step));
    }

    #[test]
    fn xor_is_the_or_shield_plus_one_arc_and_not_carries_a_bubble() {
        let or = shape_of(NodeKind::GateOr, 0.0, 0.0, 100.0, 50.0).unwrap();
        let xor = shape_of(NodeKind::GateXor, 0.0, 0.0, 100.0, 50.0).unwrap();
        assert_eq!(or.outline, xor.outline);
        assert_eq!(xor.extras.len(), 1);
        let not = shape_of(NodeKind::GateNot, 0.0, 0.0, 100.0, 50.0).unwrap();
        assert!(matches!(not.extras[0], Extra::Circle { cx, cy, .. } if cx == 50.0 && cy == 50.0));
    }
}

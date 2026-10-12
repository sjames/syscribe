//! The variant view (GH #338): one diagram drawn for two products, marking what only one of them has.
//!
//! Each product's graph is built from the model projected onto its `Configuration` (`--config A --config B`), and
//! the two are merged by node and edge id: what both have is drawn as usual, a node only `A` has is marked
//! `only A` (tone `bad`: it goes away from `A` to `B`) and one only `B` has `only B` (tone `ok`); an edge only one
//! has carries the same words in its label. A parameter that differs between the products but changes no element
//! changes nothing here: `diff --config A --config B` reports it.

use super::ir::{DiagramGraph, NodeMark, Tone};

/// The badge of a node or edge only one product has.
pub fn only_badge(label: &str) -> String {
    format!("only {label}")
}

/// Merge `a` (product `la`) and `b` (product `lb`) into one graph (see the module doc).
pub fn diff(a: DiagramGraph, b: DiagramGraph, la: &str, lb: &str) -> DiagramGraph {
    let mut out = a.clone();
    out.name = format!("{} ({la} vs {lb})", a.name);
    let mark = |n: &mut super::ir::Node, label: &str, tone: Tone| {
        let m = n.mark.get_or_insert_with(NodeMark::default);
        m.badges.push(only_badge(label));
        m.tone = tone;
        m.emphasis = true;
    };
    // only the outermost node a product lacks is marked: a part's ports and inner parts go with it
    let outermost = |n: &super::ir::Node, other: &DiagramGraph| n.parent.as_deref().map_or(true, |p| other.node(p).is_some());
    for n in out.nodes.iter_mut() {
        if b.node(&n.id).is_none() && outermost(n, &b) {
            mark(n, la, Tone::Bad);
        }
    }
    for n in &b.nodes {
        if a.node(&n.id).is_none() {
            let mut n = n.clone();
            if outermost(&n, &a) {
                mark(&mut n, lb, Tone::Ok);
            }
            out.nodes.push(n);
        }
    }
    let tag = |label: &Option<String>, who: &str| Some(match label {
        Some(l) => format!("{l} [{}]", only_badge(who)),
        None => format!("[{}]", only_badge(who)),
    });
    for e in out.edges.iter_mut() {
        if !b.edges.iter().any(|x| x.id == e.id) {
            e.label = tag(&e.label, la);
        }
    }
    for e in &b.edges {
        if !a.edges.iter().any(|x| x.id == e.id) {
            let mut e = e.clone();
            e.label = tag(&e.label, lb);
            out.edges.push(e);
        }
    }
    out.parents_first();
    out
}

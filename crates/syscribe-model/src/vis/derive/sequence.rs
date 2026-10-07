//! The Sequence generator (`REQ-TRS-VIS-021`).
//!
//! Subject: an `ActionDef`/`Action` (or a `UseCaseDef`/`UseCase` with
//! `actors:`). Produces one `Lifeline` per participant in first-appearance
//! order — the subject first, then every element a `SendAction`'s `to:`
//! chain or a `SendAction`'s/`AcceptAction`'s `via:` chain resolves to (a
//! port chain resolves to the part that owns the port; an unresolved chain
//! becomes a dashed lifeline labelled by the chain text), then the subject's
//! `actors:` as `Actor` nodes — one `Message` edge per `SendAction`
//! (subject → participant) and `AcceptAction` (participant → subject) in
//! execution order (`successionConnections:` topological order when
//! declared, else declaration order, descending into `IfAction`
//! `then`/`else` and `LoopAction` `body`), one `Fragment` per `IfAction`
//! (`alt`) and `LoopAction` (`loop`) spanning the messages it contains, and
//! an `Activation` on the subject's lifeline spanning its messages.
//!
//! A sequence diagram's geometry is fixed by its order, so the generator
//! places everything itself: lifelines left to right at [`LIFELINE_PITCH`],
//! messages top to bottom at [`ROW_PITCH`] below the headers, fragments
//! around their span, the activation around the subject's messages — every
//! node carries a pin and every message edge its two horizontal waypoints
//! (source stem to target stem at the message's row), so every renderer
//! draws the graph with the `fixed` algorithm and no ELK run. Shape ids are
//! `derived_shape_id("<subject>::<participant|action>")`; the subject's own
//! lifeline is `derived_shape_id("<subject>")`.
//!
//! Scope (the requirement's): no return, create or destroy messages (no
//! model field declares them); a nested `PerformAction` is a step in the
//! order but is not expanded into its definition's messages.

use std::collections::HashMap;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, stereotype_for_type, DiagramGraph, Edge, EdgeKind, Node, NodeKind, Point, Rect};
use super::super::manifest::Issue;
use super::{display_name, features_of, map_str, short_name, w418, yaml_strings, FeatureRole, Filters};

/// Horizontal distance between two lifeline headers' left edges.
pub const LIFELINE_PITCH: f64 = 200.0;
/// A lifeline header box.
pub const HEADER_W: f64 = 120.0;
pub const HEADER_H: f64 = 40.0;
/// An actor's header: the name on top, the stick figure beneath.
pub const ACTOR_H: f64 = 56.0;
/// Vertical distance between two consecutive messages.
pub const ROW_PITCH: f64 = 48.0;
/// Gap between the tallest header and the first row.
const FIRST_GAP: f64 = 16.0;
/// Room a fragment reserves above its first row for its keyword tab.
const FRAGMENT_HEAD: f64 = 36.0;
/// Room a fragment keeps below its last row.
const FRAGMENT_FOOT: f64 = 12.0;
/// Clearance between a fragment's box and the stems of the messages it spans.
const FRAGMENT_PAD_X: f64 = 24.0;
/// Extra clearance of an outer fragment around a nested one.
const FRAGMENT_NEST: f64 = 16.0;
/// Half-width of a fragment that spans no message (centred on the subject).
const FRAGMENT_EMPTY_HALF_W: f64 = 60.0;
/// The activation bar's width and its reach above/below the subject's messages.
pub const ACTIVATION_W: f64 = 12.0;
const ACTIVATION_PAD: f64 = 8.0;
/// A self-message's loop: how far right it reaches and how tall it is.
const SELF_MESSAGE_W: f64 = 40.0;
const SELF_MESSAGE_H: f64 = 16.0;

/// A participant of the interaction: the subject, a resolved element or
/// usage, or an unresolved chain.
#[derive(Debug, Clone)]
struct Participant<'a> {
    /// What identifies the participant across messages: the element's
    /// qualified name, `<subject>::<usage>` for an inline part usage of the
    /// subject, or the chain text when unresolved.
    key: String,
    label: String,
    element: Option<&'a RawElement>,
    /// The stereotype shown in the header: the element's type, or `part`
    /// for an inline usage.
    stereotype: Option<String>,
    element_type: Option<String>,
    resolved: bool,
    kind: NodeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MessageKind {
    Send,
    Accept,
}

/// One step of the subject's behaviour, in execution order.
#[derive(Debug, Clone)]
enum Step {
    Message {
        name: String,
        kind: MessageKind,
        label: String,
        /// The `to:`/`via:` chain, when declared.
        chain: Option<String>,
    },
    Fragment {
        name: String,
        /// `alt` or `loop`.
        operator: &'static str,
        condition: String,
        children: Vec<Step>,
    },
}

pub fn generate(
    graph: &mut DiagramGraph,
    subject: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    filters: &Filters,
    issues: &mut Vec<Issue>,
) {
    let Some(st) = subject.frontmatter.element_type.as_ref() else { return };
    if !matches!(st, ElementType::ActionDef | ElementType::Action | ElementType::UseCaseDef | ElementType::UseCase) {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — a Sequence subject must be an ActionDef or Action (or a UseCaseDef/UseCase with `actors`)",
            subject.qualified_name,
            st.name()
        )));
        return;
    }
    let sq = subject.qualified_name.as_str();

    // ── the behaviour, in execution order ───────────────────────────────
    // A usage without its own sub-actions shows its definition's.
    let behaviour: &RawElement = if subject.frontmatter.sub_actions.is_none() && matches!(st, ElementType::Action | ElementType::UseCase) {
        yaml_strings(subject.frontmatter.typed_by.as_ref()).first().and_then(|t| resolver.resolve_ref(elements, t)).unwrap_or(subject)
    } else {
        subject
    };
    let steps = match behaviour.frontmatter.sub_actions.as_deref() {
        Some(list) => ordered_steps(list, behaviour.frontmatter.succession_connections.as_deref(), true),
        None => Vec::new(),
    };

    // ── participants, in first-appearance order ─────────────────────────
    let mut participants: Vec<Participant> = vec![Participant {
        key: sq.to_string(),
        label: display_name(subject),
        element: Some(subject),
        stereotype: Some(stereotype_for_type(st)),
        element_type: Some(st.name().to_string()),
        resolved: true,
        kind: NodeKind::Lifeline,
    }];
    let mut chain_keys: HashMap<String, String> = HashMap::new();
    fn visit<'a>(
        steps: &[Step],
        subject: &'a RawElement,
        elements: &'a [RawElement],
        resolver: &Resolver,
        participants: &mut Vec<Participant<'a>>,
        chain_keys: &mut HashMap<String, String>,
    ) {
        for s in steps {
            match s {
                Step::Message { chain, .. } => {
                    let Some(chain) = chain else { continue };
                    if chain_keys.contains_key(chain) {
                        continue;
                    }
                    let p = resolve_participant(chain, subject, elements, resolver);
                    chain_keys.insert(chain.clone(), p.key.clone());
                    if !participants.iter().any(|q| q.key == p.key) {
                        participants.push(p);
                    }
                }
                Step::Fragment { children, .. } => visit(children, subject, elements, resolver, participants, chain_keys),
            }
        }
    }
    visit(&steps, subject, elements, resolver, &mut participants, &mut chain_keys);
    for a in subject.frontmatter.actors.as_deref().unwrap_or(&[]) {
        let p = match resolver.resolve_ref(elements, a) {
            Some(e) => Participant {
                key: e.qualified_name.clone(),
                label: display_name(e),
                element: Some(e),
                stereotype: None,
                element_type: e.frontmatter.element_type.as_ref().map(|t| t.name().to_string()),
                resolved: true,
                kind: NodeKind::Actor,
            },
            None => Participant { key: a.clone(), label: a.clone(), element: None, stereotype: None, element_type: None, resolved: false, kind: NodeKind::Actor },
        };
        if !participants.iter().any(|q| q.key == p.key) {
            participants.push(p);
        }
    }

    // ── filters (the subject is never filtered) ─────────────────────────
    let candidates: Vec<(String, String)> = participants.iter().skip(1).map(|p| (p.key.clone(), short_name(&p.key).to_string())).collect();
    filters.unmatched_issues(&candidates, issues);
    let kept: Vec<Participant> = participants
        .iter()
        .enumerate()
        .filter(|(i, p)| *i == 0 || filters.keeps(&p.key, short_name(&p.key)))
        .map(|(_, p)| p.clone())
        .collect();
    let index_of = |key: &str| kept.iter().position(|p| p.key == key);

    // ── lifelines ───────────────────────────────────────────────────────
    let header_h = |p: &Participant| if p.kind == NodeKind::Actor { ACTOR_H } else { HEADER_H };
    let id_of = |p: &Participant| {
        if p.key == sq {
            derived_shape_id(sq)
        } else if p.key.starts_with(&format!("{sq}::")) {
            derived_shape_id(&p.key)
        } else {
            derived_shape_id(&format!("{sq}::{}", p.key))
        }
    };
    let stem_x = |i: usize| i as f64 * LIFELINE_PITCH + HEADER_W / 2.0;
    for (i, p) in kept.iter().enumerate() {
        graph.nodes.push(Node {
            id: id_of(p),
            element_ref: p.key.clone(),
            resolved: p.resolved,
            element_type: p.element_type.clone(),
            kind: p.kind,
            label: p.label.clone(),
            stereotype: p.stereotype.clone(),
            parent: None,
            direction: None,
            side: None,
            lines: Vec::new(),
            is_abstract: p.element.map(|e| e.frontmatter.is_abstract.unwrap_or(false)).unwrap_or(false),
            pin: Some(Rect { x: i as f64 * LIFELINE_PITCH, y: 0.0, w: Some(HEADER_W), h: Some(header_h(p)) }),
            // A lifeline header names its participant and nothing more: the
            // applied-stereotype banners of `REQ-TRS-VIS-012` belong to the
            // block views, and would not fit the fixed header.
            banners: Vec::new(),
        });
    }
    let tallest = kept.iter().map(header_h).fold(0.0, f64::max);

    // ── rows, fragments and messages ────────────────────────────────────
    struct Placed {
        edges: Vec<Edge>,
        /// `(node, absolute box)` per fragment, outer before inner.
        fragments: Vec<(Node, Rect)>,
        first_row: Option<f64>,
        last_row: Option<f64>,
        counters: HashMap<String, usize>,
    }
    struct Walker<'w> {
        sq: &'w str,
        chain_keys: &'w HashMap<String, String>,
        index_of: &'w dyn Fn(&str) -> Option<usize>,
        stem_x: &'w dyn Fn(usize) -> f64,
        cursor: f64,
        out: Placed,
    }
    impl Walker<'_> {
        fn unique(&mut self, base: String) -> String {
            let n = self.out.counters.entry(base.clone()).or_insert(0);
            *n += 1;
            if *n == 1 {
                base
            } else {
                format!("{base}-{n}")
            }
        }

        /// Walk `steps`, returning the horizontal extent `(min x, max x)` of
        /// the stems the kept messages and nested fragments reach.
        fn walk(&mut self, steps: &[Step], parent: Option<&str>) -> Option<(f64, f64)> {
            let mut extent: Option<(f64, f64)> = None;
            let widen = |lo: f64, hi: f64, extent: &mut Option<(f64, f64)>| {
                *extent = Some(match extent {
                    Some((a, b)) => (a.min(lo), b.max(hi)),
                    None => (lo, hi),
                });
            };
            for s in steps {
                match s {
                    Step::Message { name, kind, label, chain } => {
                        let other = chain.as_deref().and_then(|c| self.chain_keys.get(c)).and_then(|k| (self.index_of)(k));
                        // A message whose participant is filtered out (or
                        // never resolved to a column) draws nothing.
                        let Some(other) = other else { continue };
                        let (src, tgt) = match kind {
                            MessageKind::Send => (0usize, other),
                            MessageKind::Accept => (other, 0usize),
                        };
                        let y = self.cursor + ROW_PITCH / 2.0;
                        self.cursor += ROW_PITCH;
                        let (sx, tx) = ((self.stem_x)(src), (self.stem_x)(tgt));
                        let waypoints = if src == tgt {
                            vec![
                                Point { x: sx, y },
                                Point { x: sx + SELF_MESSAGE_W, y },
                                Point { x: sx + SELF_MESSAGE_W, y: y + SELF_MESSAGE_H },
                                Point { x: sx, y: y + SELF_MESSAGE_H },
                            ]
                        } else {
                            vec![Point { x: sx, y }, Point { x: tx, y }]
                        };
                        widen(sx.min(tx), sx.max(tx), &mut extent);
                        if self.out.first_row.is_none() {
                            self.out.first_row = Some(y);
                        }
                        self.out.last_row = Some(y + if src == tgt { SELF_MESSAGE_H } else { 0.0 });
                        let id = self.unique(format!("e-{}", &derived_shape_id(&format!("{}::{name}", self.sq))[2..]));
                        // The ends are column indices for now; the caller,
                        // which knows the lifeline ids, swaps them in.
                        self.out.edges.push(Edge {
                            id,
                            element_ref: Some(format!("{}::{name}", self.sq)),
                            source: format!("#{src}"),
                            target: format!("#{tgt}"),
                            kind: EdgeKind::Message,
                            label: Some(label.clone()),
                            waypoints: Some(waypoints),
                        });
                    }
                    Step::Fragment { name, operator, condition, children } => {
                        let top = self.cursor + 4.0;
                        self.cursor += FRAGMENT_HEAD + 4.0;
                        let id = self.unique(derived_shape_id(&format!("{}::{name}", self.sq)));
                        // Reserve the slot so the fragment precedes what it contains.
                        let slot = self.out.fragments.len();
                        self.out.fragments.push((
                            Node {
                                id: id.clone(),
                                element_ref: format!("{}::{name}", self.sq),
                                resolved: true,
                                element_type: None,
                                kind: NodeKind::Fragment,
                                label: if condition.is_empty() { String::new() } else { format!("[{condition}]") },
                                stereotype: Some(operator.to_string()),
                                parent: parent.map(str::to_string),
                                direction: None,
                                side: None,
                                lines: Vec::new(),
                                is_abstract: false,
                                pin: None,
                                banners: Vec::new(),
                            },
                            Rect { x: 0.0, y: top, w: None, h: None },
                        ));
                        let inner = self.walk(children, Some(&id));
                        let bottom = self.cursor + 4.0;
                        self.cursor += FRAGMENT_FOOT;
                        let (lo, hi) = match inner {
                            Some((a, b)) => (a - FRAGMENT_PAD_X, b + FRAGMENT_PAD_X),
                            None => {
                                let cx = (self.stem_x)(0);
                                (cx - FRAGMENT_EMPTY_HALF_W, cx + FRAGMENT_EMPTY_HALF_W)
                            }
                        };
                        self.out.fragments[slot].1 = Rect { x: lo, y: top, w: Some(hi - lo), h: Some(bottom - top) };
                        widen(lo - FRAGMENT_NEST, hi + FRAGMENT_NEST, &mut extent);
                    }
                }
            }
            extent
        }
    }
    let subject_id = derived_shape_id(sq);
    let mut walker = Walker {
        sq,
        chain_keys: &chain_keys,
        index_of: &index_of,
        stem_x: &stem_x,
        cursor: tallest + FIRST_GAP,
        out: Placed { edges: Vec::new(), fragments: Vec::new(), first_row: None, last_row: None, counters: HashMap::new() },
    };
    walker.walk(&steps, None);
    let Placed { mut edges, fragments, first_row, last_row, .. } = walker.out;

    // ── the activation on the subject's lifeline (parent-relative pin) ──
    if let (Some(first), Some(last)) = (first_row, last_row) {
        let y = first - ACTIVATION_PAD;
        graph.nodes.push(Node {
            id: format!("{subject_id}-activation"),
            element_ref: sq.to_string(),
            resolved: true,
            element_type: None,
            kind: NodeKind::Activation,
            label: String::new(),
            stereotype: None,
            parent: Some(subject_id.clone()),
            direction: None,
            side: None,
            lines: Vec::new(),
            is_abstract: false,
            pin: Some(Rect { x: (HEADER_W - ACTIVATION_W) / 2.0, y, w: Some(ACTIVATION_W), h: Some(last + ACTIVATION_PAD - y) }),
            banners: Vec::new(),
        });
    }

    // ── fragments: absolute boxes made parent-relative ──────────────────
    let abs: HashMap<String, Rect> = fragments.iter().map(|(n, r)| (n.id.clone(), *r)).collect();
    for (mut node, r) in fragments {
        let (px, py) = node.parent.as_deref().and_then(|p| abs.get(p)).map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0));
        node.pin = Some(Rect { x: r.x - px, y: r.y - py, w: r.w, h: r.h });
        graph.nodes.push(node);
    }

    // ── messages: column indices → lifeline ids ─────────────────────────
    let ids: Vec<String> = kept.iter().map(id_of).collect();
    for e in &mut edges {
        let col = |s: &str| s.strip_prefix('#').and_then(|n| n.parse::<usize>().ok()).and_then(|i| ids.get(i).cloned()).unwrap_or_default();
        e.source = col(&e.source);
        e.target = col(&e.target);
    }
    graph.edges.extend(edges);
}

/// The sub-actions of `list` as steps in execution order: the
/// `successionConnections:` topological order (ties and unmentioned steps in
/// declaration order, a cycle broken in declaration order) at the top level,
/// declaration order inside `then`/`else`/`body`. Only `SendAction`,
/// `AcceptAction`, `IfAction` and `LoopAction` yield a step.
fn ordered_steps(list: &[serde_yaml::Value], successions: Option<&[serde_yaml::Value]>, top: bool) -> Vec<Step> {
    let order: Vec<usize> = match successions.filter(|_| top) {
        Some(s) => topological_order(list, s),
        None => (0..list.len()).collect(),
    };
    let mut out = Vec::new();
    for i in order {
        let serde_yaml::Value::Mapping(m) = &list[i] else { continue };
        let Some(name) = map_str(m, "name") else { continue };
        let payload = map_str(m, "payload").map(|p| short_name(p).to_string());
        match map_str(m, "kind") {
            Some("SendAction") => {
                let label = match &payload {
                    Some(p) => format!("{name}({p})"),
                    None => name.to_string(),
                };
                out.push(Step::Message {
                    name: name.to_string(),
                    kind: MessageKind::Send,
                    label,
                    chain: map_str(m, "to").or_else(|| map_str(m, "via")).map(str::to_string),
                });
            }
            Some("AcceptAction") => {
                let trigger = m.get(serde_yaml::Value::String("trigger".into())).and_then(|t| t.as_mapping()).and_then(|t| {
                    let kind = map_str(t, "kind").unwrap_or("");
                    match kind {
                        "message" => map_str(t, "payload").map(|p| short_name(p).to_string()),
                        "timeOut" => map_str(t, "when").map(|w| format!("after {w}")),
                        "at" => map_str(t, "when").map(|w| format!("at {w}")),
                        "change" => map_str(t, "condition").map(|c| format!("when {c}")),
                        _ => None,
                    }
                });
                let label = match payload.or(trigger) {
                    Some(p) => format!("{name}({p})"),
                    None => name.to_string(),
                };
                out.push(Step::Message { name: name.to_string(), kind: MessageKind::Accept, label, chain: map_str(m, "via").map(str::to_string) });
            }
            Some("IfAction") => {
                let mut children = Vec::new();
                for branch in ["then", "else"] {
                    if let Some(serde_yaml::Value::Sequence(seq)) = m.get(serde_yaml::Value::String(branch.into())) {
                        children.extend(ordered_steps(seq, None, false));
                    }
                }
                out.push(Step::Fragment {
                    name: name.to_string(),
                    operator: "alt",
                    condition: map_str(m, "condition").unwrap_or("").to_string(),
                    children,
                });
            }
            Some("LoopAction") => {
                let children = match m.get(serde_yaml::Value::String("body".into())) {
                    Some(serde_yaml::Value::Sequence(seq)) => ordered_steps(seq, None, false),
                    _ => Vec::new(),
                };
                let condition = match (map_str(m, "loopKind"), map_str(m, "condition")) {
                    (Some("for"), _) => match (map_str(m, "variable"), map_str(m, "sequence")) {
                        (Some(v), Some(s)) => format!("for {v} in {s}"),
                        (_, Some(s)) => format!("for {s}"),
                        _ => String::new(),
                    },
                    (Some("until"), Some(c)) => format!("until {c}"),
                    (_, Some(c)) => c.to_string(),
                    _ => String::new(),
                };
                out.push(Step::Fragment { name: name.to_string(), operator: "loop", condition, children });
            }
            _ => {}
        }
    }
    out
}

/// Kahn's algorithm over the `after`/`before` pairs that name two different
/// entries of `list`, always taking the lowest declaration index among the
/// ready entries; whatever a cycle leaves is appended in declaration order.
fn topological_order(list: &[serde_yaml::Value], successions: &[serde_yaml::Value]) -> Vec<usize> {
    let name_of = |v: &serde_yaml::Value| v.as_mapping().and_then(|m| map_str(m, "name")).map(str::to_string);
    let names: Vec<Option<String>> = list.iter().map(name_of).collect();
    let index = |n: &str| names.iter().position(|x| x.as_deref() == Some(n));
    let mut indeg = vec![0usize; list.len()];
    let mut succ: Vec<Vec<usize>> = vec![Vec::new(); list.len()];
    for s in successions {
        let Some(m) = s.as_mapping() else { continue };
        let (Some(a), Some(b)) = (map_str(m, "after"), map_str(m, "before")) else { continue };
        let (Some(a), Some(b)) = (index(a.trim()), index(b.trim())) else { continue };
        if a == b || succ[a].contains(&b) {
            continue;
        }
        succ[a].push(b);
        indeg[b] += 1;
    }
    let mut done = vec![false; list.len()];
    let mut out = Vec::with_capacity(list.len());
    while out.len() < list.len() {
        match (0..list.len()).find(|&i| !done[i] && indeg[i] == 0) {
            Some(i) => {
                done[i] = true;
                out.push(i);
                for &b in &succ[i] {
                    // A predecessor emitted to break a cycle may already be at zero.
                    indeg[b] = indeg[b].saturating_sub(1);
                }
            }
            None => {
                // A cycle: break it at the first undone entry.
                let i = (0..list.len()).find(|&i| !done[i]).expect("some entry is undone");
                done[i] = true;
                indeg[i] = 0;
                out.push(i);
                for &b in &succ[i] {
                    indeg[b] = indeg[b].saturating_sub(1);
                }
            }
        }
    }
    out
}

fn is_part_like(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::PartDef) | Some(ElementType::Part) | Some(ElementType::ItemDef) | Some(ElementType::Item))
}

fn element_participant(e: &RawElement) -> Participant<'_> {
    let et = e.frontmatter.element_type.as_ref();
    Participant {
        key: e.qualified_name.clone(),
        label: display_name(e),
        element: Some(e),
        stereotype: et.map(stereotype_for_type),
        element_type: et.map(|t| t.name().to_string()),
        resolved: true,
        kind: NodeKind::Lifeline,
    }
}

/// Whether `e` performs `subject` (a `performs:` entry typed by it).
fn performs(e: &RawElement, subject: &RawElement) -> bool {
    e.frontmatter.performs.as_deref().unwrap_or(&[]).iter().any(|p| match p {
        serde_yaml::Value::Mapping(m) => map_str(m, "typedBy") == Some(subject.qualified_name.as_str()),
        serde_yaml::Value::String(s) => s == &subject.qualified_name,
        _ => false,
    })
}

/// Resolve a `to:`/`via:` feature chain to its participant (module doc):
/// a model reference (a port element resolving to its owner), an inline
/// feature of the subject (a part usage, or a port — owned by the subject
/// itself), a port name owned by a part in the model (one that performs the
/// subject first), else the chain itself, unresolved.
fn resolve_participant<'a>(chain: &str, subject: &'a RawElement, elements: &'a [RawElement], resolver: &Resolver) -> Participant<'a> {
    let norm = chain.trim().replace('.', "::");
    let segments: Vec<&str> = norm.split("::").filter(|s| !s.is_empty()).collect();

    // 1. The whole chain names an element.
    if let Some(e) = resolver.resolve_ref(elements, &norm) {
        if matches!(e.frontmatter.element_type, Some(ElementType::PortDef) | Some(ElementType::Port)) {
            if let Some(owner) = e.qualified_name.rsplit_once("::").and_then(|(o, _)| resolver.resolve_ref(elements, o)) {
                return element_participant(owner);
            }
        }
        return element_participant(e);
    }

    // 2. The first segment is an inline feature of the subject.
    if let Some(first) = segments.first() {
        let feats = features_of(subject, elements, resolver);
        if let Some(f) = feats.iter().find(|f| f.name == *first) {
            match f.role {
                FeatureRole::Part => {
                    let def = f.typed_by.as_deref().and_then(|t| resolver.resolve_ref(elements, t));
                    let label = match def {
                        Some(d) => format!("{} : {}", f.name, display_name(d)),
                        None => f.name.clone(),
                    };
                    return Participant {
                        key: format!("{}::{}", subject.qualified_name, f.name),
                        label,
                        element: def,
                        stereotype: Some("part".to_string()),
                        element_type: Some("Part".to_string()),
                        resolved: true,
                        kind: NodeKind::Lifeline,
                    };
                }
                FeatureRole::Port => return element_participant(subject),
                _ => {}
            }
        }
        // 3. A prefix names an element and the rest a port it owns; or a lone
        //    port name owned by a part in the model (a performer first).
        if segments.len() >= 2 {
            let (owner, port) = (segments[..segments.len() - 1].join("::"), segments[segments.len() - 1]);
            if let Some(e) = resolver.resolve_ref(elements, &owner) {
                if features_of(e, elements, resolver).iter().any(|f| f.name == port && f.role == FeatureRole::Port) {
                    return element_participant(e);
                }
            }
        } else {
            let owners: Vec<&RawElement> = elements
                .iter()
                .filter(|e| is_part_like(e))
                .filter(|e| features_of(e, elements, resolver).iter().any(|f| f.name == *first && f.role == FeatureRole::Port))
                .collect();
            if let Some(e) = owners.iter().find(|e| performs(e, subject)).or_else(|| owners.first()) {
                return element_participant(e);
            }
        }
    }

    // 4. Unresolved: a dashed lifeline named by the chain text.
    Participant { key: chain.trim().to_string(), label: chain.trim().to_string(), element: None, stereotype: None, element_type: None, resolved: false, kind: NodeKind::Lifeline }
}

#[cfg(test)]
mod tests {
    use super::super::testkit::*;
    use super::*;

    fn edges_of(g: &DiagramGraph) -> Vec<(String, String, String)> {
        g.edges.iter().map(|e| (e.source.clone(), e.target.clone(), e.label.clone().unwrap_or_default())).collect()
    }

    #[test]
    fn participants_appear_in_execution_order_subject_first_then_actors() {
        let d = diagram("Sequence", "Flow::Startup", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        let lifelines: Vec<(&str, NodeKind, &str, bool)> = g
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Lifeline | NodeKind::Actor))
            .map(|n| (n.id.as_str(), n.kind, n.label.as_str(), n.resolved))
            .collect();
        assert_eq!(
            lifelines,
            vec![
                ("s-flow-startup", NodeKind::Lifeline, "Startup", true),
                // awaitReady via statusIn: the port's owner, Controller (it performs Startup).
                ("s-flow-startup-flow-controller", NodeKind::Lifeline, "Controller", true),
                // sendPower to Flow::Motor.
                ("s-flow-startup-flow-motor", NodeKind::Lifeline, "Motor", true),
                // coolDown to fan.ctrl: nothing owns it.
                ("s-flow-startup-fan-ctrl", NodeKind::Lifeline, "fan.ctrl", false),
                ("s-flow-startup-flow-operator", NodeKind::Actor, "Operator", true),
            ]
        );
        assert_eq!(g.node("s-flow-startup").unwrap().stereotype.as_deref(), Some("action def"));
        assert_eq!(g.node("s-flow-startup-flow-controller").unwrap().stereotype.as_deref(), Some("part def"));
        assert!(g.node("s-flow-startup-fan-ctrl").unwrap().stereotype.is_none());
    }

    #[test]
    fn messages_follow_the_successions_and_descend_into_branches_and_bodies() {
        let d = diagram("Sequence", "Flow::Startup", |_| {});
        let (g, _) = derive_it(&d);
        // Declaration order is spinUp, sendPower, awaitReady, checkTemp, pollLoop,
        // finish; the successions reorder to spinUp, awaitReady, sendPower,
        // pollLoop, checkTemp, finish.
        assert_eq!(
            edges_of(&g),
            vec![
                ("s-flow-startup-flow-controller".into(), "s-flow-startup".into(), "awaitReady(Ready)".into()),
                ("s-flow-startup".into(), "s-flow-startup-flow-motor".into(), "sendPower(PowerCmd)".into()),
                ("s-flow-startup".into(), "s-flow-startup-flow-controller".into(), "poll(Poll)".into()),
                ("s-flow-startup".into(), "s-flow-startup-fan-ctrl".into(), "coolDown(Cool)".into()),
                ("s-flow-startup".into(), "s-flow-startup-flow-motor".into(), "proceed(Go)".into()),
            ]
        );
        assert!(g.edges.iter().all(|e| e.kind == EdgeKind::Message));
        assert_eq!(g.edges[0].id, "e-flow-startup-awaitready");
        assert_eq!(g.edges[0].element_ref.as_deref(), Some("Flow::Startup::awaitReady"));
        // Rows: tallest header 56 (an actor) + 16, then a row per message with
        // a fragment head before `poll` and `coolDown`.
        let ys: Vec<f64> = g.edges.iter().map(|e| e.waypoints.as_ref().unwrap()[0].y).collect();
        assert!(ys.windows(2).all(|w| w[1] > w[0]), "rows descend: {ys:?}");
        assert_eq!(ys[0], 56.0 + 16.0 + 24.0);
        assert_eq!(ys[1], ys[0] + ROW_PITCH);
    }

    #[test]
    fn fragments_span_their_messages_and_the_activation_spans_the_subjects() {
        let d = diagram("Sequence", "Flow::Startup", |_| {});
        let (g, _) = derive_it(&d);
        let frags: Vec<(&str, &str, &str)> = g
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Fragment)
            .map(|n| (n.id.as_str(), n.stereotype.as_deref().unwrap_or(""), n.label.as_str()))
            .collect();
        assert_eq!(frags, vec![("s-flow-startup-pollloop", "loop", "[not ready]"), ("s-flow-startup-checktemp", "alt", "[temp > 90]")]);
        let lp = g.node("s-flow-startup-pollloop").unwrap().pin.unwrap();
        let poll = &g.edges[2].waypoints.as_ref().unwrap();
        // The loop box encloses its one message horizontally and vertically.
        assert!(lp.x < poll[0].x.min(poll[1].x) && lp.x + lp.w.unwrap() > poll[0].x.max(poll[1].x));
        assert!(lp.y < poll[0].y && lp.y + lp.h.unwrap() > poll[0].y);
        let alt = g.node("s-flow-startup-checktemp").unwrap().pin.unwrap();
        let cool = &g.edges[3].waypoints.as_ref().unwrap();
        let go = &g.edges[4].waypoints.as_ref().unwrap();
        assert!(alt.x < cool[1].x.min(go[1].x) - 20.0 && alt.x + alt.w.unwrap() > cool[1].x.max(go[1].x) + 20.0);
        assert!(alt.y < cool[0].y && alt.y + alt.h.unwrap() > go[0].y);
        assert!(alt.y > lp.y + lp.h.unwrap(), "fragments do not overlap");
        // The activation sits on the subject's lifeline, parent-relative, from the first to the last row.
        let act = g.node("s-flow-startup-activation").unwrap();
        assert_eq!(act.kind, NodeKind::Activation);
        assert_eq!(act.parent.as_deref(), Some("s-flow-startup"));
        let ap = act.pin.unwrap();
        assert_eq!(ap.x, (HEADER_W - ACTIVATION_W) / 2.0);
        let first = g.edges[0].waypoints.as_ref().unwrap()[0].y;
        let last = g.edges[4].waypoints.as_ref().unwrap()[0].y;
        assert_eq!(ap.y, first - 8.0);
        assert_eq!(ap.y + ap.h.unwrap(), last + 8.0);
    }

    #[test]
    fn everything_is_pinned_and_messages_run_horizontally_between_stems() {
        let d = diagram("Sequence", "Flow::Startup", |_| {});
        let (g, _) = derive_it(&d);
        assert!(g.is_fully_pinned());
        assert_eq!(g.layout_hints.algorithm, crate::vis::ir::LayoutAlgorithm::Fixed);
        for (i, n) in g.nodes.iter().filter(|n| matches!(n.kind, NodeKind::Lifeline | NodeKind::Actor)).enumerate() {
            let p = n.pin.unwrap();
            assert_eq!((p.x, p.y, p.w), (i as f64 * LIFELINE_PITCH, 0.0, Some(HEADER_W)));
            assert_eq!(p.h, Some(if n.kind == NodeKind::Actor { ACTOR_H } else { HEADER_H }));
        }
        for e in &g.edges {
            let w = e.waypoints.as_ref().expect("waypoints");
            assert_eq!(w.len(), 2);
            assert_eq!(w[0].y, w[1].y);
            let col = |id: &str| g.nodes.iter().filter(|n| matches!(n.kind, NodeKind::Lifeline | NodeKind::Actor)).position(|n| n.id == id).unwrap();
            assert_eq!(w[0].x, col(&e.source) as f64 * LIFELINE_PITCH + HEADER_W / 2.0);
            assert_eq!(w[1].x, col(&e.target) as f64 * LIFELINE_PITCH + HEADER_W / 2.0);
        }
    }

    #[test]
    fn filters_drop_a_participant_and_its_messages_and_name_strays() {
        let d = diagram("Sequence", "Flow::Startup", |fm| {
            fm.exclude = Some(vec!["Flow::Controller".into(), "Ghost".into()]);
        });
        let (g, issues) = derive_it(&d);
        assert!(g.node("s-flow-startup-flow-controller").is_none());
        assert!(g.edges.iter().all(|e| !e.source.contains("controller") && !e.target.contains("controller")));
        assert_eq!(g.edges.len(), 3);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "W417");
        assert!(issues[0].message.contains("'Ghost'"));
        // Short names match too; the subject is never filtered.
        let d = diagram("Sequence", "Flow::Startup", |fm| fm.include = Some(vec!["Motor".into()]));
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        let ids: Vec<&str> = g.nodes.iter().filter(|n| matches!(n.kind, NodeKind::Lifeline | NodeKind::Actor)).map(|n| n.id.as_str()).collect();
        assert_eq!(ids, vec!["s-flow-startup", "s-flow-startup-flow-motor"]);
        assert_eq!(g.edges.len(), 2);
    }

    #[test]
    fn wrong_subject_type_is_w418() {
        let d = diagram("Sequence", "Sys::Engine", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(g.nodes.is_empty());
        assert_eq!(issues[0].code, "W418");
        assert!(issues[0].message.contains("PartDef"));
    }

    #[test]
    fn a_subject_without_messages_has_a_lone_lifeline_and_no_activation() {
        let d = diagram("Sequence", "Sys::Startup", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty());
        assert_eq!(g.nodes.len(), 1);
        assert_eq!(g.nodes[0].kind, NodeKind::Lifeline);
        assert!(g.edges.is_empty());
        assert!(g.is_fully_pinned());
    }

    #[test]
    fn topological_order_breaks_ties_and_cycles_by_declaration() {
        let list = yaml_list("- {name: a}\n- {name: b}\n- {name: c}\n- {name: d}\n");
        let s = yaml_list("- {after: c, before: a}\n- {after: start, before: c}\n- {after: d, before: d}\n");
        assert_eq!(topological_order(&list, &s), vec![1, 2, 0, 3]);
        // A cycle a ⇄ b: the ready entries go first, then the cycle is broken
        // at its first declared entry.
        let cyc = yaml_list("- {after: a, before: b}\n- {after: b, before: a}\n");
        assert_eq!(topological_order(&list, &cyc), vec![2, 3, 0, 1]);
    }
}

//! The Allocation generator (`REQ-TRS-VIS-022`).
//!
//! Subject: a `Package`/`LibraryPackage`/`Namespace`, an `AllocationDef` or
//! an `Allocation`. Collects every allocation pair under the subject — an
//! `Allocation` element's top-level `allocatedFrom:`/`allocatedTo:`, each
//! `features:` entry of `type: Allocation` carrying those two fields, each
//! `allocations:` entry (`allocatedFrom`/`allocatedTo`, or the bulk
//! `from`/`to` spelling), and each `allocatedTo:` on a `Part`/`PartDef`/
//! `Action`/`ActionDef` (its source being the element itself) — and draws two
//! `Swimlane` nodes, `logical` and `physical`, holding one `Block` per
//! distinct source and target element (its real type's stereotype; an
//! unresolved end is a dashed block labelled by the reference text), joined
//! by one `Allocation` edge per pair, labelled by the usage's `name` when
//! present. `include:`/`exclude:` apply to the end elements.

use std::collections::{BTreeSet, HashMap};

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, Node, NodeKind};
use super::super::manifest::Issue;
use super::requirement::{element_keys, is_under, keeps_keys, unmatched_key_issues};
use super::{block_node, display_name, map_str, short_name, w418, yaml_strings, Filters};

fn is_package(t: &ElementType) -> bool {
    matches!(t, ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace)
}

/// The kinds whose own `allocatedTo:` is an allocation pair (§12.9 form 1).
fn carries_allocated_to(t: &ElementType) -> bool {
    matches!(t, ElementType::Part | ElementType::PartDef | ElementType::Action | ElementType::ActionDef)
}

/// One allocation pair as written: the two references, the usage name and
/// the element that declares it.
#[derive(Debug, Clone, PartialEq)]
struct Pair {
    from: String,
    to: String,
    name: Option<String>,
    owner: String,
}

/// Every allocation pair declared by `e`, in declaration order.
fn pairs_of(e: &RawElement) -> Vec<Pair> {
    let fm = &e.frontmatter;
    let owner = e.qualified_name.clone();
    let mut out = Vec::new();
    let cross = |from: &[String], to: &[String], name: Option<String>, out: &mut Vec<Pair>| {
        for f in from {
            for t in to {
                out.push(Pair { from: f.clone(), to: t.clone(), name: name.clone(), owner: owner.clone() });
            }
        }
    };
    match fm.element_type.as_ref() {
        // An Allocation element: its own two ends, named by the element.
        Some(ElementType::Allocation) => {
            if let (Some(from), Some(to)) = (fm.allocated_from.as_ref(), fm.allocated_to.as_ref()) {
                cross(from, to, Some(display_name(e)), &mut out);
            }
        }
        // A part or action allocated by its own `allocatedTo:`.
        Some(t) if carries_allocated_to(t) => {
            if let Some(to) = fm.allocated_to.as_ref() {
                cross(std::slice::from_ref(&e.qualified_name), to, None, &mut out);
            }
        }
        _ => {}
    }
    // Inline `features:` entries of `type: Allocation`.
    for m in fm.features.iter().flatten().filter_map(|v| v.as_mapping()) {
        if map_str(m, "type") != Some("Allocation") {
            continue;
        }
        let from = yaml_strings(m.get(serde_yaml::Value::String("allocatedFrom".into())));
        let to = yaml_strings(m.get(serde_yaml::Value::String("allocatedTo".into())));
        cross(&from, &to, map_str(m, "name").map(str::to_string), &mut out);
    }
    // `allocations:` entries (an AllocationDef's, or the bulk `from`/`to` form).
    for m in fm.allocations.iter().flatten().filter_map(|v| v.as_mapping()) {
        let key = |a: &str, b: &str| {
            let v = m.get(serde_yaml::Value::String(a.into())).or_else(|| m.get(serde_yaml::Value::String(b.into())));
            yaml_strings(v)
        };
        cross(&key("allocatedFrom", "from"), &key("allocatedTo", "to"), map_str(m, "name").map(str::to_string), &mut out);
    }
    out
}

/// One end of a pair: the element it resolves to, or the reference text.
struct End<'a> {
    reference: String,
    element: Option<&'a RawElement>,
}

impl End<'_> {
    /// The identity the end is deduplicated and filtered by.
    fn key(&self) -> &str {
        self.element.map(|e| e.qualified_name.as_str()).unwrap_or(&self.reference)
    }

    fn filter_keys(&self) -> Vec<String> {
        match self.element {
            Some(e) => element_keys(e),
            None => vec![self.reference.clone(), short_name(&self.reference).to_string()],
        }
    }

    fn node(&self, id: String, lane: &str, elements: &[RawElement], resolver: &Resolver) -> Node {
        match self.element {
            Some(e) => block_node(id, e, NodeKind::Block, Some(lane.to_string()), display_name(e), elements, resolver),
            None => Node {
                id,
                element_ref: self.reference.clone(),
                resolved: false,
                element_type: None,
                kind: NodeKind::Block,
                label: self.reference.clone(),
                stereotype: None,
                parent: Some(lane.to_string()),
                direction: None,
                side: None,
                lines: Vec::new(),
                is_abstract: false,
                pin: None,
                banners: Vec::new(),
                feature: None,
                mark: None,
            },
        }
    }
}

fn lane(id: String, label: &str, subject: &RawElement) -> Node {
    Node {
        id,
        element_ref: subject.qualified_name.clone(),
        resolved: true,
        element_type: None,
        kind: NodeKind::Swimlane,
        label: label.to_string(),
        stereotype: None,
        parent: None,
        direction: None,
        side: None,
        lines: Vec::new(),
        is_abstract: false,
        pin: None,
        banners: Vec::new(),
        feature: None,
        mark: None,
    }
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
    if !(is_package(st) || matches!(st, ElementType::AllocationDef | ElementType::Allocation)) {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — an Allocation diagram subject must be a Package, AllocationDef or Allocation",
            subject.qualified_name,
            st.name()
        )));
        return;
    }
    let sq = subject.qualified_name.as_str();

    // ── the pairs under the subject, in element order ───────────────────
    let mut owners: Vec<&RawElement> = elements.iter().filter(|e| is_under(&e.qualified_name, sq)).collect();
    owners.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
    let pairs: Vec<Pair> = owners.iter().flat_map(|e| pairs_of(e)).collect();
    let end = |r: &str| End { reference: r.to_string(), element: resolver.resolve_ref(elements, r) };

    // ── filters over the distinct end elements ──────────────────────────
    let mut candidates: Vec<Vec<String>> = Vec::new();
    let mut seen_keys: BTreeSet<String> = BTreeSet::new();
    for p in &pairs {
        for e in [end(&p.from), end(&p.to)] {
            if seen_keys.insert(e.key().to_string()) {
                candidates.push(e.filter_keys());
            }
        }
    }
    unmatched_key_issues(filters, &candidates, issues);
    let kept = |e: &End| keeps_keys(filters, &e.filter_keys());

    // ── lanes and blocks: one per distinct kept source / target ─────────
    let subject_id = derived_shape_id(sq);
    let logical = format!("{subject_id}-logical");
    let physical = format!("{subject_id}-physical");
    graph.nodes.push(lane(logical.clone(), "Logical", subject));
    graph.nodes.push(lane(physical.clone(), "Physical", subject));
    let mut sources: Vec<End> = Vec::new();
    let mut targets: Vec<End> = Vec::new();
    for p in &pairs {
        let (s, t) = (end(&p.from), end(&p.to));
        if kept(&s) && !sources.iter().any(|x| x.key() == s.key()) {
            sources.push(s);
        }
        if kept(&t) && !targets.iter().any(|x| x.key() == t.key()) {
            targets.push(t);
        }
    }
    let source_id = |key: &str| derived_shape_id(key);
    // An element in both lanes keeps the plain id on the logical side and
    // takes `-physical` on the physical side.
    let target_id = |key: &str| {
        let id = derived_shape_id(key);
        if sources.iter().any(|s| s.key() == key) {
            format!("{id}-physical")
        } else {
            id
        }
    };
    for s in &sources {
        graph.nodes.push(s.node(source_id(s.key()), &logical, elements, resolver));
    }
    for t in &targets {
        graph.nodes.push(t.node(target_id(t.key()), &physical, elements, resolver));
    }

    // ── one edge per pair whose ends are both kept ──────────────────────
    let mut counters: HashMap<String, usize> = HashMap::new();
    for p in &pairs {
        let (s, t) = (end(&p.from), end(&p.to));
        if !(kept(&s) && kept(&t)) {
            continue;
        }
        let (src, tgt) = (source_id(s.key()), target_id(t.key()));
        let mut base = format!("e-allocation-{src}-{tgt}");
        if let Some(n) = p.name.as_deref() {
            base.push('-');
            base.push_str(&n.to_ascii_lowercase());
        }
        let n = counters.entry(base.clone()).or_insert(0);
        *n += 1;
        let id = if *n == 1 { base } else { format!("{base}-{n}") };
        graph.edges.push(Edge {
            id,
            element_ref: Some(p.owner.clone()),
            source: src,
            target: tgt,
            kind: EdgeKind::Allocation,
            label: p.name.clone(),
            waypoints: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::testkit::*;
    use super::*;

    fn blocks_in<'a>(g: &'a DiagramGraph, lane: &'a str) -> Vec<(&'a str, &'a str)> {
        g.children_of(lane).map(|n| (n.id.as_str(), n.element_ref.as_str())).collect()
    }

    #[test]
    fn package_subject_collects_every_pair_form_into_two_lanes() {
        let d = diagram("Allocation", "Alloc", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        let lanes: Vec<(&str, &str, NodeKind)> = g.roots().map(|n| (n.id.as_str(), n.label.as_str(), n.kind)).collect();
        assert_eq!(lanes, vec![("s-alloc-logical", "Logical", NodeKind::Swimlane), ("s-alloc-physical", "Physical", NodeKind::Swimlane)]);
        assert!(g.roots().all(|n| n.element_ref == "Alloc" && n.resolved));
        // Blocks in order of first appearance over the pairs, which come from
        // the owners in qualified-name order: the part's own `allocatedTo:`
        // (CtrlSw), the AllocationDef's `allocations:` (FnDef), then the
        // Allocation element's own pair and its inline feature (FnToHw).
        assert_eq!(
            blocks_in(&g, "s-alloc-logical"),
            vec![("s-alloc-ctrlsw", "Alloc::CtrlSw"), ("s-sys-startup", "Sys::Startup"), ("s-reqs-controller", "Reqs::Controller")]
        );
        assert_eq!(
            blocks_in(&g, "s-alloc-physical"),
            vec![("s-sys-motor", "Sys::Motor"), ("s-alloc-ctrlsw-physical", "Alloc::CtrlSw"), ("s-sys-engine", "Sys::Engine"), ("s-ghost-hw", "Ghost::Hw")]
        );
        // Real type stereotypes; a dashed block for the unresolved end.
        assert_eq!(g.node("s-sys-startup").unwrap().stereotype.as_deref(), Some("action def"));
        assert_eq!(g.node("s-sys-engine").unwrap().element_type.as_deref(), Some("PartDef"));
        let ghost = g.node("s-ghost-hw").unwrap();
        assert!(!ghost.resolved && ghost.stereotype.is_none() && ghost.element_type.is_none());
        assert_eq!(ghost.label, "Ghost::Hw");
        assert_eq!(ghost.kind, NodeKind::Block);
        // One drawn twice: once per lane, with the `-physical` suffix on the second.
        assert_eq!(g.node("s-alloc-ctrlsw").unwrap().parent.as_deref(), Some("s-alloc-logical"));
        assert_eq!(g.node("s-alloc-ctrlsw-physical").unwrap().parent.as_deref(), Some("s-alloc-physical"));
        // Edges, one per pair, labelled by the usage name when present.
        let edges: Vec<(&str, &str, &str, Option<&str>, Option<&str>)> = g
            .edges
            .iter()
            .map(|e| (e.id.as_str(), e.source.as_str(), e.target.as_str(), e.label.as_deref(), e.element_ref.as_deref()))
            .collect();
        assert_eq!(
            edges,
            vec![
                ("e-allocation-s-alloc-ctrlsw-s-sys-motor", "s-alloc-ctrlsw", "s-sys-motor", None, Some("Alloc::CtrlSw")),
                ("e-allocation-s-sys-startup-s-alloc-ctrlsw-physical-navtoctrl", "s-sys-startup", "s-alloc-ctrlsw-physical", Some("navToCtrl"), Some("Alloc::FnDef")),
                ("e-allocation-s-sys-startup-s-sys-motor-navtomotor", "s-sys-startup", "s-sys-motor", Some("navToMotor"), Some("Alloc::FnDef")),
                ("e-allocation-s-sys-startup-s-sys-engine-fntohw", "s-sys-startup", "s-sys-engine", Some("FnToHw"), Some("Alloc::FnToHw")),
                ("e-allocation-s-reqs-controller-s-ghost-hw-ctrltoghost", "s-reqs-controller", "s-ghost-hw", Some("ctrlToGhost"), Some("Alloc::FnToHw")),
            ]
        );
        assert!(g.edges.iter().all(|e| e.kind == EdgeKind::Allocation));
        assert!(g.layout_hints.hierarchical);
    }

    #[test]
    fn allocation_and_allocation_def_subjects_scope_to_their_own_pairs() {
        let d = diagram("Allocation", "Alloc::FnToHw", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.roots().map(|n| n.id.as_str()).collect::<Vec<_>>(), vec!["s-alloc-fntohw-logical", "s-alloc-fntohw-physical"]);
        assert_eq!(g.edges.len(), 2);
        assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Block).count(), 4);

        let d = diagram("Allocation", "Alloc::FnDef", |_| {});
        let (g, _) = derive_it(&d);
        assert_eq!(g.edges.iter().filter_map(|e| e.label.as_deref()).collect::<Vec<_>>(), vec!["navToCtrl", "navToMotor"]);
        // The lanes keep the plain id here: CtrlSw is a target only.
        assert!(g.node("s-alloc-ctrlsw").is_some() && g.node("s-alloc-ctrlsw-physical").is_none());
    }

    #[test]
    fn filters_apply_to_the_end_elements_and_flag_unknown_entries() {
        let d = diagram("Allocation", "Alloc", |fm| {
            fm.include = Some(vec!["Startup".into(), "Sys::Engine".into(), "Ghost::Hw".into(), "Nobody".into()]);
        });
        let (g, issues) = derive_it(&d);
        assert_eq!(blocks_in(&g, "s-alloc-logical"), vec![("s-sys-startup", "Sys::Startup")]);
        assert_eq!(blocks_in(&g, "s-alloc-physical"), vec![("s-sys-engine", "Sys::Engine"), ("s-ghost-hw", "Ghost::Hw")]);
        assert_eq!(g.edges.len(), 1, "only Startup → Engine has both ends kept: {:?}", g.edges);
        assert_eq!(issues, vec![super::super::w417("`include` entry 'Nobody' names no member of the subject".into())]);

        let d = diagram("Allocation", "Alloc", |fm| fm.exclude = Some(vec!["Sys::Startup".into()]));
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty());
        assert!(g.node("s-sys-startup").is_none());
        assert_eq!(g.edges.len(), 2);
    }

    #[test]
    fn wrong_subject_type_is_w418_and_empty() {
        let d = diagram("Allocation", "Sys::Engine", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(g.nodes.is_empty());
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "W418");
        assert!(issues[0].message.contains("PartDef"));
    }
}

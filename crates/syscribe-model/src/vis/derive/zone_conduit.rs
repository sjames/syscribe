//! The ZoneConduit (IEC 62443) generator (GH #223).
//!
//! Subject: a `Package` (every `Zone` under it), a `Zone` (it, its conduits
//! and the zones at their far ends) or a `Conduit` (it and its two zones). Each
//! zone is a **compound node** containing its member parts (`members:` and
//! `inZone:`, `syscribe_model::zones::zone_members`) and the
//! `SecurityControl`s that contribute to it (`zones::controls_by_zone`); each
//! conduit an **edge** between its zones labelled
//! `CD-… · SL <achieved>/<required>`.
//!
//! The levels and verdicts are `syscribe_model::zones` — the same functions the
//! `zones` / `conduits` reports use. A zone is toned `bad` with an `SL gap`
//! badge when `achievedSL < targetSL` (`W950`), `ok` when it meets its target,
//! `warn` when either is missing; a conduit whose `achievedSL` is below the
//! highest `targetSL` of the zones it joins is a **weak conduit**
//! (`W951`): drawn as a red, heavy, dashed `weakConduit` edge.

use std::collections::HashMap;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::zones::{conduit_required_sl, conduit_status, controls_by_zone, id_of, is_conduit, is_zone, zone_gap, zone_members, ConduitStatus};

use super::super::ir::{derived_shape_id, DiagramGraph, EdgeKind, NodeKind, NodeMark, Tone};
use super::super::manifest::Issue;
use super::analysis::{add_edge, add_node, keys_of, mark, node_for, unresolved_node};
use super::requirement::{is_under, keeps_keys, unmatched_key_issues};
use super::{w418, Filters};

fn is_package(t: &ElementType) -> bool {
    matches!(t, ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace)
}

fn sl(v: Option<u8>) -> String {
    v.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string())
}

fn push_zone<'a>(zones: &mut Vec<&'a RawElement>, z: &'a RawElement) {
    if !zones.iter().any(|x| x.qualified_name == z.qualified_name) {
        zones.push(z);
    }
}

fn zone_mark(z: &RawElement, controls: usize) -> NodeMark {
    let fm = &z.frontmatter;
    let mut badges = Vec::new();
    let tone = match zone_gap(z) {
        Some(true) => {
            badges.push("SL gap".to_string());
            Tone::Bad
        }
        Some(false) => Tone::Ok,
        None => Tone::Warn,
    };
    if controls == 0 {
        badges.push("no controls".to_string());
    }
    let mut m = mark(format!("SL target {} / achieved {}", sl(fm.target_sl), sl(fm.achieved_sl)), fm.status.clone(), tone, badges);
    m.emphasis = tone == Tone::Bad;
    m
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
    let (sub_zone, sub_conduit) = (is_zone(subject), is_conduit(subject));
    if !(is_package(st) || sub_zone || sub_conduit) {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — a ZoneConduit diagram subject must be a Package, a Zone or a Conduit",
            subject.qualified_name,
            st.name()
        )));
        return;
    }
    let sq = subject.qualified_name.as_str();
    let zone_of = |r: &Option<String>| r.as_deref().and_then(|r| resolver.resolve_ref(elements, r)).filter(|z| is_zone(z));

    // Seed conduits and zones.
    let mut conduits: Vec<&RawElement> = Vec::new();
    let mut zones: Vec<&RawElement> = Vec::new();
    if sub_conduit {
        conduits.push(subject);
    } else {
        let seeds: Vec<&RawElement> = elements
            .iter()
            .filter(|e| is_zone(e) && (sub_zone && e.qualified_name == sq || !sub_zone && is_under(&e.qualified_name, sq)))
            .collect();
        for z in &seeds {
            push_zone(&mut zones, z);
        }
        for c in elements.iter().filter(|e| is_conduit(e)) {
            let touches = [&c.frontmatter.from_zone, &c.frontmatter.to_zone]
                .into_iter()
                .any(|r| zone_of(r).is_some_and(|z| seeds.iter().any(|s| s.qualified_name == z.qualified_name)));
            // A package also draws conduits that live under it, with whatever zones they join.
            if touches || (!sub_zone && is_under(&c.qualified_name, sq)) {
                conduits.push(c);
            }
        }
    }
    for c in &conduits {
        for r in [&c.frontmatter.from_zone, &c.frontmatter.to_zone] {
            if let Some(z) = zone_of(r) {
                push_zone(&mut zones, z);
            }
        }
    }
    if zones.is_empty() && conduits.is_empty() {
        issues.push(w418(format!("`subject` '{}' covers no Zone or Conduit — nothing to draw", subject.qualified_name)));
        return;
    }

    let controls = controls_by_zone(elements, resolver);
    let by_q: HashMap<&str, &RawElement> = elements.iter().map(|e| (e.qualified_name.as_str(), e)).collect();
    let mut candidates: Vec<Vec<String>> = Vec::new();
    let mut zone_ids: HashMap<String, String> = HashMap::new();

    for z in &zones {
        candidates.push(keys_of(z));
        if !keeps_keys(filters, &keys_of(z)) {
            continue;
        }
        let ctl = controls.get(&z.qualified_name).cloned().unwrap_or_default();
        let zid = add_node(graph, node_for(z, NodeKind::Zone, Some(zone_mark(z, ctl.len()))));
        zone_ids.insert(z.qualified_name.clone(), zid.clone());
        for m in zone_members(z, elements, resolver) {
            let child_id = derived_shape_id(&format!("{}::{m}", z.qualified_name));
            let mut node = match by_q.get(m.as_str()) {
                Some(p) => {
                    candidates.push(keys_of(p));
                    if !keeps_keys(filters, &keys_of(p)) {
                        continue;
                    }
                    let mut n = node_for(p, NodeKind::Block, None);
                    n.stereotype = p.frontmatter.element_type.as_ref().map(crate::vis::ir::stereotype_for_type);
                    n
                }
                None => unresolved_node(&m, NodeKind::Block),
            };
            node.id = child_id;
            node.parent = Some(zid.clone());
            add_node(graph, node);
        }
        for cid in &ctl {
            // A control id (resolved) or a conduit's `implementedBy:` text kept as written.
            let found = elements.iter().find(|e| Resolver::is_security_control(e) && id_of(e) == cid);
            let mut node = match found {
                Some(c) => {
                    candidates.push(keys_of(c));
                    if !keeps_keys(filters, &keys_of(c)) {
                        continue;
                    }
                    // The id, not the (long) name: a zone holds many controls and the
                    // element's own page has the name.
                    let mut n = node_for(c, NodeKind::Block, Some(mark(c.frontmatter.control_type.clone().unwrap_or_else(|| "control".to_string()), None, Tone::Ok, Vec::new())));
                    n.label = id_of(c).to_string();
                    n.stereotype = Some("security control".to_string());
                    n
                }
                None => {
                    let mut n = unresolved_node(cid, NodeKind::Block);
                    n.mark = None;
                    n.resolved = true;
                    n.element_type = None;
                    n.stereotype = Some("implementedBy".to_string());
                    n
                }
            };
            node.id = derived_shape_id(&format!("{}::ctl::{cid}", z.qualified_name));
            node.parent = Some(zid.clone());
            add_node(graph, node);
        }
    }

    for c in &conduits {
        candidates.push(keys_of(c));
        if !keeps_keys(filters, &keys_of(c)) {
            continue;
        }
        let end = |graph: &mut DiagramGraph, r: &Option<String>| -> Option<String> {
            match zone_of(r) {
                Some(z) => zone_ids.get(&z.qualified_name).cloned(),
                None => r.as_deref().map(|t| add_node(graph, unresolved_node(t, NodeKind::Zone))),
            }
        };
        let (Some(from), Some(to)) = (end(graph, &c.frontmatter.from_zone), end(graph, &c.frontmatter.to_zone)) else { continue };
        let required = conduit_required_sl(c, elements, resolver);
        let status = conduit_status(c, required);
        let kind = if status == ConduitStatus::Weak { EdgeKind::WeakConduit } else { EdgeKind::Conduit };
        let mut label = format!("{} · SL {}/{}", id_of(c), sl(c.frontmatter.achieved_sl), sl(required));
        if status == ConduitStatus::Weak {
            label.push_str(" weak");
        }
        if let Some(p) = c.frontmatter.protocols.as_ref().filter(|p| !p.is_empty()) {
            label.push_str(&format!(" · {}", p.join(", ")));
        }
        add_edge(graph, kind, &from, &to, Some(c.qualified_name.clone()), Some(label));
    }
    unmatched_key_issues(filters, &candidates, issues);
}

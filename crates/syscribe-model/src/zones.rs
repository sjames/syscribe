//! IEC 62443 zone / conduit analysis shared by the `zones` / `conduits` CLI
//! reports and the `ZoneConduit` diagram deriver (GH #223).
//!
//! One definition of "zone gap", "required SL of a conduit", "weak conduit"
//! and "which SecurityControls contribute to a zone", so the reports and the
//! picture can never disagree.

use std::collections::{BTreeMap, BTreeSet};

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::validator::allocation_edges;

pub fn is_zone(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::Zone))
}

pub fn is_conduit(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::Conduit))
}

/// The stable id of a zone / conduit / control, falling back to its qualified name.
pub fn id_of(e: &RawElement) -> &str {
    e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name)
}

/// `Some(true)` when a zone's `achievedSL` is below its `targetSL`, `Some(false)`
/// when it is not, `None` when either is missing (unknown, never a silent pass).
pub fn zone_gap(z: &RawElement) -> Option<bool> {
    z.frontmatter.achieved_sl.zip(z.frontmatter.target_sl).map(|(a, t)| a < t)
}

/// The security level a conduit must provide: the highest `targetSL` of the
/// zones it joins.
pub fn conduit_required_sl(c: &RawElement, elements: &[RawElement], resolver: &Resolver) -> Option<u8> {
    [&c.frontmatter.from_zone, &c.frontmatter.to_zone]
        .into_iter()
        .flatten()
        .filter_map(|z| resolver.resolve_ref(elements, z).and_then(|t| t.frontmatter.target_sl))
        .max()
}

/// How a conduit's achieved level compares with what it must provide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConduitStatus {
    /// `achievedSL` meets the requirement.
    Ok,
    /// `achievedSL` is below the requirement: a weak conduit.
    Weak,
    /// `achievedSL` or a zone's `targetSL` is missing.
    Unknown,
}

pub fn conduit_status(c: &RawElement, required: Option<u8>) -> ConduitStatus {
    match (c.frontmatter.achieved_sl, required) {
        (Some(a), Some(r)) if a < r => ConduitStatus::Weak,
        (Some(_), Some(_)) => ConduitStatus::Ok,
        _ => ConduitStatus::Unknown,
    }
}

/// The parts that belong to a zone: its `members:` plus every `PartDef`/`Part`
/// whose `inZone:` names it, as qualified names (resolved when they resolve,
/// else the authored text), in first-seen order.
pub fn zone_members(z: &RawElement, elements: &[RawElement], resolver: &Resolver) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |q: String| {
        if !out.contains(&q) {
            out.push(q);
        }
    };
    for m in z.frontmatter.members.as_deref().unwrap_or(&[]) {
        push(resolver.resolve_ref(elements, m).map(|t| t.qualified_name.clone()).unwrap_or_else(|| m.clone()));
    }
    for e in elements {
        let Some(iz) = e.frontmatter.in_zone.as_deref() else { continue };
        if resolver.resolve_ref(elements, iz).is_some_and(|t| t.qualified_name == z.qualified_name) {
            push(e.qualified_name.clone());
        }
    }
    out
}

/// The `SecurityControl`s contributing to each zone, keyed by the zone's qualified name.
///
/// A control contributes to a zone when it is allocated (any §12.9 form —
/// `Allocation` element, `allocatedTo:`, legacy `allocatedFrom:`) to a part of
/// the zone (`members:` / `inZone:`), to the zone itself, or to a conduit
/// touching the zone, or when a conduit touching the zone names it in
/// `implementedBy:`. A zone's `members:` can only be parts (`E955`), so the
/// original "a member that is a SecurityControl" rule could never fire;
/// allocation is how a control is bound to the architecture
/// (`prompts/spec/safety.md`).
///
/// A conduit `implementedBy:` entry that is not a control (a source path) is
/// kept as written, as before.
pub fn controls_by_zone(elements: &[RawElement], resolver: &Resolver) -> BTreeMap<String, BTreeSet<String>> {
    let controls: BTreeMap<&str, &RawElement> =
        elements.iter().filter(|e| Resolver::is_security_control(e)).map(|e| (e.qualified_name.as_str(), e)).collect();
    // target qname -> controls allocated to it.
    let mut allocated: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (from, to) in allocation_edges(elements, resolver) {
        if let Some(c) = controls.get(from.as_str()) {
            allocated.entry(to).or_default().insert(id_of(c).to_string());
        }
    }
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for z in elements.iter().filter(|e| is_zone(e)) {
        let mut set: BTreeSet<String> = BTreeSet::new();
        for m in zone_members(z, elements, resolver) {
            if let Some(cs) = allocated.get(&m) {
                set.extend(cs.iter().cloned());
            }
            if let Some(c) = controls.get(m.as_str()) {
                set.insert(id_of(c).to_string());
            }
        }
        if let Some(cs) = allocated.get(&z.qualified_name) {
            set.extend(cs.iter().cloned());
        }
        for c in elements.iter().filter(|e| is_conduit(e)) {
            let touches = [&c.frontmatter.from_zone, &c.frontmatter.to_zone]
                .into_iter()
                .flatten()
                .any(|zr| resolver.resolve_ref(elements, zr).is_some_and(|t| t.qualified_name == z.qualified_name));
            if !touches {
                continue;
            }
            if let Some(cs) = allocated.get(&c.qualified_name) {
                set.extend(cs.iter().cloned());
            }
            for ib in c.frontmatter.implemented_by.as_deref().unwrap_or(&[]) {
                match resolver.resolve_ref(elements, ib).filter(|t| Resolver::is_security_control(t)) {
                    Some(t) => set.insert(id_of(t).to_string()),
                    None => set.insert(ib.clone()),
                };
            }
        }
        out.insert(z.qualified_name.clone(), set);
    }
    out
}

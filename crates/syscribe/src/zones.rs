//! IEC 62443 Zone/Conduit surface (§13, GH #61). Read-only: lists security zones with
//! their SL gap status, conduits with SL adequacy, and a Zone × SecurityControl coverage
//! cross-table.

use std::collections::BTreeSet;
use syscribe_model::{
    element::{ElementType, RawElement},
    resolver::Resolver,
};

fn is_zone(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::Zone))
}
fn is_conduit(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::Conduit))
}
fn id_of(e: &RawElement) -> &str {
    e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name)
}

pub fn cmd_zones(elements: &[RawElement], coverage: bool, json: bool) {
    if coverage {
        return zone_coverage(elements, json);
    }
    let mut zones: Vec<&RawElement> = elements.iter().filter(|e| is_zone(e)).collect();
    zones.sort_by(|a, b| id_of(a).cmp(id_of(b)));

    if json {
        let arr: Vec<serde_json::Value> = zones
            .iter()
            .map(|z| {
                let fm = &z.frontmatter;
                serde_json::json!({
                    "id": id_of(z), "name": fm.name, "status": fm.status,
                    "targetSL": fm.target_sl, "achievedSL": fm.achieved_sl,
                    "members": fm.members.as_ref().map(|m| m.len()).unwrap_or(0),
                    "gap": fm.achieved_sl.zip(fm.target_sl).map(|(a, t)| a < t).unwrap_or(false)
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "zones": arr })).unwrap());
        return;
    }
    if zones.is_empty() {
        println!("No Zone elements in the model.");
        return;
    }
    println!("| Zone | tSL | aSL | Members | Gap |");
    println!("|---|---|---|---|---|");
    for z in &zones {
        let fm = &z.frontmatter;
        let gap = match (fm.achieved_sl, fm.target_sl) {
            (Some(a), Some(t)) if a < t => "⚠ SL gap",
            (Some(_), Some(_)) => "✓",
            _ => "—",
        };
        println!(
            "| {} | {} | {} | {} | {} |",
            id_of(z),
            fm.target_sl.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
            fm.achieved_sl.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
            fm.members.as_ref().map(|m| m.len()).unwrap_or(0),
            gap,
        );
    }
}

pub fn cmd_conduits(elements: &[RawElement], json: bool) {
    let resolver = Resolver::new(elements);
    let mut conduits: Vec<&RawElement> = elements.iter().filter(|e| is_conduit(e)).collect();
    conduits.sort_by(|a, b| id_of(a).cmp(id_of(b)));

    // Required SL of a conduit = max targetSL of its connected zones.
    let required_sl = |c: &RawElement| -> Option<u8> {
        [&c.frontmatter.from_zone, &c.frontmatter.to_zone]
            .into_iter()
            .flatten()
            .filter_map(|z| resolver.resolve_ref(elements, z).and_then(|t| t.frontmatter.target_sl))
            .max()
    };

    if json {
        let arr: Vec<serde_json::Value> = conduits
            .iter()
            .map(|c| {
                let fm = &c.frontmatter;
                let req = required_sl(c);
                let pass = match (fm.achieved_sl, req) {
                    (Some(a), Some(r)) => a >= r,
                    _ => true,
                };
                serde_json::json!({
                    "id": id_of(c), "name": fm.name, "fromZone": fm.from_zone, "toZone": fm.to_zone,
                    "achievedSL": fm.achieved_sl, "requiredSL": req, "pass": pass
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "conduits": arr })).unwrap());
        return;
    }
    if conduits.is_empty() {
        println!("No Conduit elements in the model.");
        return;
    }
    println!("| Conduit | From | To | aSL | Required | Status |");
    println!("|---|---|---|---|---|---|");
    for c in &conduits {
        let fm = &c.frontmatter;
        let req = required_sl(c);
        let status = match (fm.achieved_sl, req) {
            (Some(a), Some(r)) if a < r => "⚠ weak",
            (Some(_), Some(_)) => "✓",
            _ => "—",
        };
        println!(
            "| {} | {} | {} | {} | {} | {} |",
            id_of(c),
            fm.from_zone.as_deref().unwrap_or("—"),
            fm.to_zone.as_deref().unwrap_or("—"),
            fm.achieved_sl.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
            req.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
            status,
        );
    }
}

fn zone_coverage(elements: &[RawElement], json: bool) {
    let resolver = Resolver::new(elements);
    let mut zones: Vec<&RawElement> = elements.iter().filter(|e| is_zone(e)).collect();
    zones.sort_by(|a, b| id_of(a).cmp(id_of(b)));

    // For each zone, the SecurityControls contributing to it: conduit implementedBy of any
    // conduit touching the zone, plus zone members that are SecurityControls.
    let controls_for = |z: &RawElement| -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for c in elements.iter().filter(|e| is_conduit(e)) {
            let touches = [&c.frontmatter.from_zone, &c.frontmatter.to_zone]
                .into_iter()
                .flatten()
                .any(|zr| resolver.resolve_ref(elements, zr).map(|t| t.qualified_name == z.qualified_name).unwrap_or(false));
            if touches {
                for ib in c.frontmatter.implemented_by.as_deref().unwrap_or(&[]) {
                    out.insert(ib.clone());
                }
            }
        }
        for m in z.frontmatter.members.as_deref().unwrap_or(&[]) {
            if let Some(t) = resolver.resolve_ref(elements, m) {
                if matches!(t.frontmatter.element_type, Some(ElementType::SecurityControl)) {
                    out.insert(id_of(t).to_string());
                }
            }
        }
        out
    };

    if json {
        let arr: Vec<serde_json::Value> = zones
            .iter()
            .map(|z| serde_json::json!({ "zone": id_of(z), "targetSL": z.frontmatter.target_sl, "controls": controls_for(z).into_iter().collect::<Vec<_>>() }))
            .collect();
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "coverage": arr })).unwrap());
        return;
    }
    if zones.is_empty() {
        println!("No Zone elements in the model.");
        return;
    }
    println!("| Zone | targetSL | Security controls |");
    println!("|---|---|---|");
    for z in &zones {
        let ctrls = controls_for(z);
        println!(
            "| {} | {} | {} |",
            id_of(z),
            z.frontmatter.target_sl.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
            if ctrls.is_empty() { "—".to_string() } else { ctrls.into_iter().collect::<Vec<_>>().join(", ") },
        );
    }
}

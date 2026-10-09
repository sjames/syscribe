//! IEC 62443 Zone/Conduit surface (§13, GH #61). Read-only: lists security zones with
//! their SL gap status, conduits with SL adequacy, and a Zone × SecurityControl coverage
//! cross-table.

use syscribe_model::{
    element::RawElement,
    resolver::Resolver,
    zones::{conduit_required_sl, conduit_status, controls_by_zone, id_of, is_conduit, is_zone, zone_gap, zone_members, ConduitStatus},
};

pub fn cmd_zones(elements: &[RawElement], coverage: bool, json: bool) {
    let resolver = Resolver::new(elements);
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
                    "members": zone_members(z, elements, &resolver).len(),
                    "gap": zone_gap(z).unwrap_or(false)
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
        let gap = match zone_gap(z) {
            Some(true) => "⚠ SL gap",
            Some(false) => "✓",
            None => "—",
        };
        println!(
            "| {} | {} | {} | {} | {} |",
            id_of(z),
            fm.target_sl.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
            fm.achieved_sl.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
            zone_members(z, elements, &resolver).len(),
            gap,
        );
    }
}

pub fn cmd_conduits(elements: &[RawElement], json: bool) {
    let resolver = Resolver::new(elements);
    let mut conduits: Vec<&RawElement> = elements.iter().filter(|e| is_conduit(e)).collect();
    conduits.sort_by(|a, b| id_of(a).cmp(id_of(b)));

    let required_sl = |c: &RawElement| conduit_required_sl(c, elements, &resolver);

    if json {
        let arr: Vec<serde_json::Value> = conduits
            .iter()
            .map(|c| {
                let fm = &c.frontmatter;
                let req = required_sl(c);
                // `null` = unknown (achievedSL or a zone targetSL is missing); never a
                // silent pass on missing data (GH #220).
                let pass: Option<bool> = match conduit_status(c, req) {
                    ConduitStatus::Weak => Some(false),
                    ConduitStatus::Ok => Some(true),
                    ConduitStatus::Unknown => None,
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
        let status = match conduit_status(c, req) {
            ConduitStatus::Weak => "⚠ weak",
            ConduitStatus::Ok => "✓",
            ConduitStatus::Unknown => "—",
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

    // Controls allocated to a zone's parts / the zone / its conduits, plus conduit
    // `implementedBy:` entries (`syscribe_model::zones::controls_by_zone`).
    let by_zone = controls_by_zone(elements, &resolver);
    let controls_for = |z: &RawElement| by_zone.get(&z.qualified_name).cloned().unwrap_or_default();

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

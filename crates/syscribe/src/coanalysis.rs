//! `co-analysis` — safety↔security co-engineering view (ISO 26262 ⇄ ISO/SAE 21434).
//!
//! Builds the cross-domain chain
//!
//! ```text
//! ThreatScenario --damageScenarios--> DamageScenario --hazardRef--> HazardousEvent/SafetyGoal
//! ```
//!
//! (plus a `ThreatScenario`'s own direct `hazardRef`, if set) and answers the
//! first question a dual functional-safety + cybersecurity assessor asks:
//! *"which cyber threats can violate this safety goal/hazard, and where is that
//! analysed?"*
//!
//! Read-only view: it traverses frontmatter and the [`Resolver`] directly and
//! does NOT touch `build_graph`.
//!
//! DEFERRED (issue #28 check (b)): "a SafetyGoal whose realising architecture
//! has an attack surface with no security consideration." That needs
//! goal→architecture→vulnerability reachability and is out of scope here —
//! tracked as future work in REQ-TRS-SEC-001.

use serde_json::json;
use std::collections::BTreeMap;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;

/// Display id: stable `id:` when present, else qualified name.
fn disp_id(e: &RawElement) -> String {
    e.frontmatter
        .id
        .clone()
        .unwrap_or_else(|| e.qualified_name.clone())
}

fn title_of(e: &RawElement) -> String {
    e.frontmatter
        .title
        .clone()
        .or_else(|| e.frontmatter.name.clone())
        .unwrap_or_else(|| disp_id(e))
}

fn type_label(e: &RawElement) -> &'static str {
    match e.frontmatter.element_type {
        Some(ElementType::SafetyGoal) => "SafetyGoal",
        Some(ElementType::HazardousEvent) => "HazardousEvent",
        _ => "Hazard",
    }
}

/// One safety-relevant DamageScenario node in the report.
struct DamageNode {
    id: String,
    title: String,
    /// Threats that lead to this damage scenario (via `damageScenarios`).
    threats: Vec<(String, String)>, // (id, title)
}

/// A goal/hazard target with the safety-relevant damage scenarios and the
/// cyber threats that can lead to its violation.
struct GoalNode {
    id: String,
    type_label: &'static str,
    title: String,
    damage_scenarios: Vec<DamageNode>,
    /// Distinct threats across all linked damage scenarios + direct hazardRefs.
    threats: Vec<(String, String)>, // (id, title)
}

pub fn cmd_coanalysis(elements: &[RawElement], json_out: bool) {
    let resolver = Resolver::new(elements);

    // Collect ThreatScenarios so we can find which damage scenarios they lead to,
    // and any direct hazardRef they carry.
    let threats: Vec<&RawElement> = elements
        .iter()
        .filter(|e| e.frontmatter.element_type == Some(ElementType::ThreatScenario))
        .collect();

    // damage scenario id (disp_id) -> threats that reference it.
    let mut threats_by_damage: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    // goal/hazard key -> threats referencing it directly via hazardRef.
    let mut direct_threats_by_goal: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

    for ts in &threats {
        let ts_pair = (disp_id(ts), title_of(ts));
        if let Some(ds_refs) = &ts.frontmatter.damage_scenarios {
            for r in ds_refs {
                if let Some(ds) = resolver.resolve_ref(elements, r) {
                    if ds.frontmatter.element_type == Some(ElementType::DamageScenario) {
                        threats_by_damage
                            .entry(disp_id(ds))
                            .or_default()
                            .push(ts_pair.clone());
                    }
                }
            }
        }
        if let Some(hrefs) = &ts.frontmatter.hazard_ref {
            for r in hrefs {
                if let Some(goal) = resolver.resolve_ref(elements, r) {
                    if matches!(
                        goal.frontmatter.element_type,
                        Some(ElementType::SafetyGoal) | Some(ElementType::HazardousEvent)
                    ) {
                        direct_threats_by_goal
                            .entry(disp_id(goal))
                            .or_default()
                            .push(ts_pair.clone());
                    }
                }
            }
        }
    }

    // goal key -> GoalNode (built incrementally as we discover damage scenarios).
    let mut goals: BTreeMap<String, GoalNode> = BTreeMap::new();
    // Safety-tagged damage scenarios with no hazardRef (the W030 gaps).
    let mut unlinked: Vec<(String, String)> = Vec::new();

    for ds in elements
        .iter()
        .filter(|e| e.frontmatter.element_type == Some(ElementType::DamageScenario))
    {
        let safety_tagged = ds
            .frontmatter
            .impact_categories
            .as_ref()
            .map(|c| c.iter().any(|x| x == "safety"))
            .unwrap_or(false);
        let hrefs = ds.frontmatter.hazard_ref.clone().unwrap_or_default();

        if safety_tagged && hrefs.is_empty() {
            unlinked.push((disp_id(ds), title_of(ds)));
            continue;
        }

        let ds_id = disp_id(ds);
        let ds_threats = threats_by_damage.get(&ds_id).cloned().unwrap_or_default();

        for r in &hrefs {
            if let Some(goal) = resolver.resolve_ref(elements, r) {
                if !matches!(
                    goal.frontmatter.element_type,
                    Some(ElementType::SafetyGoal) | Some(ElementType::HazardousEvent)
                ) {
                    continue; // E844 reports the bad ref; co-analysis just skips it.
                }
                let gkey = disp_id(goal);
                let node = goals.entry(gkey.clone()).or_insert_with(|| GoalNode {
                    id: gkey.clone(),
                    type_label: type_label(goal),
                    title: title_of(goal),
                    damage_scenarios: Vec::new(),
                    threats: Vec::new(),
                });
                node.damage_scenarios.push(DamageNode {
                    id: ds_id.clone(),
                    title: title_of(ds),
                    threats: ds_threats.clone(),
                });
            }
        }
    }

    // Fold in direct ThreatScenario hazardRefs (a threat may name a goal that no
    // damage scenario reached). Ensure the goal node exists.
    for gkey in direct_threats_by_goal.keys() {
        if !goals.contains_key(gkey) {
            if let Some(goal) = resolver.resolve_ref(elements, gkey) {
                goals.insert(
                    gkey.clone(),
                    GoalNode {
                        id: gkey.clone(),
                        type_label: type_label(goal),
                        title: title_of(goal),
                        damage_scenarios: Vec::new(),
                        threats: Vec::new(),
                    },
                );
            }
        }
    }

    // Aggregate the distinct threat set per goal (from its damage scenarios and
    // any direct hazardRefs).
    for (gkey, node) in goals.iter_mut() {
        let mut seen: BTreeMap<String, String> = BTreeMap::new();
        for dn in &node.damage_scenarios {
            for (tid, ttitle) in &dn.threats {
                seen.insert(tid.clone(), ttitle.clone());
            }
        }
        if let Some(direct) = direct_threats_by_goal.get(gkey) {
            for (tid, ttitle) in direct {
                seen.insert(tid.clone(), ttitle.clone());
            }
        }
        node.threats = seen.into_iter().collect();
    }

    unlinked.sort();
    let goal_list: Vec<&GoalNode> = goals.values().collect();

    if json_out {
        emit_json(&goal_list, &unlinked);
    } else {
        emit_text(&goal_list, &unlinked);
    }
}

fn emit_json(goals: &[&GoalNode], unlinked: &[(String, String)]) {
    let goals_json: Vec<_> = goals
        .iter()
        .map(|g| {
            json!({
                "id": g.id,
                "type": g.type_label,
                "title": g.title,
                "damageScenarios": g.damage_scenarios.iter().map(|d| json!({
                    "id": d.id,
                    "title": d.title,
                    "threats": d.threats.iter().map(|(id, title)| json!({"id": id, "title": title})).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
                "threats": g.threats.iter().map(|(id, title)| json!({"id": id, "title": title})).collect::<Vec<_>>(),
            })
        })
        .collect();

    let doc = json!({
        "goals": goals_json,
        "unlinkedSafetyDamage": unlinked.iter().map(|(id, title)| json!({"id": id, "title": title})).collect::<Vec<_>>(),
    });
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}

fn emit_text(goals: &[&GoalNode], unlinked: &[(String, String)]) {
    if goals.is_empty() && unlinked.is_empty() {
        println!("# Safety ↔ Security Co-Analysis\n");
        println!("No safety-relevant damage scenarios found (no DamageScenario with impactCategories: safety).");
        return;
    }

    println!("# Safety ↔ Security Co-Analysis (ISO 26262 ⇄ ISO/SAE 21434)\n");
    println!("Which cyber threats can violate each safety goal / hazardous event.\n");

    if goals.is_empty() {
        println!("(No safety goal / hazardous event has a linked safety-relevant damage scenario.)\n");
    }

    for g in goals {
        println!("## {} {} — {}", g.type_label, g.id, g.title);
        if g.threats.is_empty() {
            println!("  threats: (none lead here yet)");
        } else {
            println!("  threats that can violate it:");
            for (tid, ttitle) in &g.threats {
                println!("    - {tid} — {ttitle}");
            }
        }
        if g.damage_scenarios.is_empty() {
            println!("  via: (linked directly by a ThreatScenario hazardRef)");
        } else {
            println!("  via safety-relevant damage scenarios:");
            for d in &g.damage_scenarios {
                println!("    - {} — {}", d.id, d.title);
                for (tid, ttitle) in &d.threats {
                    println!("        <- {tid} — {ttitle}");
                }
            }
        }
        println!();
    }

    println!("## Gaps — safety-tagged DamageScenarios with no hazardRef (W030)");
    if unlinked.is_empty() {
        println!("  (none — every safety-tagged damage scenario is linked)");
    } else {
        for (id, title) in unlinked {
            println!("    - {id} — {title}");
        }
    }
}

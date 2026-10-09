//! The ThreatGraph (TARA) generator (GH #223).
//!
//! Subject: a `Package` or `TARASheet` (every chain under it), or one
//! `ThreatScenario`, `DamageScenario`, `Asset`, `CybersecurityGoal` or
//! `SecurityControl` (the chains through it). The picture is the ISO/SAE 21434
//! chain, left to right:
//!
//! `ThreatScenario → DamageScenario → Asset → CybersecurityGoal → SecurityControl`
//!
//! A goal protects an asset when one of the goal's `threatScenarios:` leads, by
//! `damageScenarios:`, to a damage scenario whose `assets:` names it; a damage
//! scenario with no `assets:` links straight to the goal (and a threat with no
//! damage scenario too), so no chain is lost for want of an asset.
//!
//! Colour: a threat shows its **risk** under the project's configured method
//! (`[cyber]` in `.syscribe.toml`: `risk::threat_risk` over the impact of its
//! damage scenarios and its attack feasibility, explicit or computed from the
//! attack-potential factors) — `low` ok, `medium` warn, `high`/`critical` bad,
//! `critical` drawn heavier. A damage scenario is toned by its impact, a goal by
//! whether a control implements it (`W802` badge otherwise), a threat no goal
//! treats carries a `no goal` badge. The configuration travels in
//! [`Filters::cyber`]; a caller without one gets the `simple` default.

use std::collections::{BTreeSet, HashSet};

use crate::cyber_config::{CyberConfig, IMPACTS};
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::risk::{damage_impact_rank, threat_feasibility_rank, threat_risk, RiskLevel};

use super::super::ir::{DiagramGraph, EdgeKind, NodeKind, NodeMark, Tone};
use super::super::manifest::Issue;
use super::analysis::{add_edge, add_node, keys_of, mark, node_for};
use super::requirement::{is_under, keeps_keys, unmatched_key_issues};
use super::{w418, Filters};

fn is_package(t: &ElementType) -> bool {
    matches!(t, ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace | ElementType::TARASheet)
}

fn is_goal(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::CybersecurityGoal))
}

fn is_member_type(e: &RawElement) -> bool {
    Resolver::is_threat_scenario(e)
        || Resolver::is_damage_scenario(e)
        || Resolver::is_asset(e)
        || is_goal(e)
        || Resolver::is_security_control(e)
}

fn risk_tone(level: RiskLevel) -> Tone {
    match level {
        RiskLevel::Low => Tone::Ok,
        RiskLevel::Medium => Tone::Warn,
        RiskLevel::High | RiskLevel::Critical => Tone::Bad,
    }
}

fn impact_tone(rank: u8) -> Tone {
    match rank {
        3 => Tone::Bad,
        2 => Tone::Warn,
        1 => Tone::Ok,
        _ => Tone::Neutral,
    }
}

/// One path through the model: a threat, the damage it can cause, the asset
/// harmed and the goal that treats it (any of them may be absent).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Chain {
    threat: Option<String>,
    damage: Option<String>,
    asset: Option<String>,
    goal: Option<String>,
}

fn resolve<'a>(elements: &'a [RawElement], resolver: &Resolver, r: &str) -> Option<&'a RawElement> {
    resolver.resolve_ref(elements, r)
}

fn damages_of(t: &RawElement, elements: &[RawElement], resolver: &Resolver) -> Vec<String> {
    t.frontmatter
        .damage_scenarios
        .iter()
        .flatten()
        .filter_map(|r| resolve(elements, resolver, r).filter(|d| Resolver::is_damage_scenario(d)))
        .map(|d| d.qualified_name.clone())
        .collect()
}

fn assets_of(d: &RawElement, elements: &[RawElement], resolver: &Resolver) -> Vec<String> {
    d.frontmatter
        .assets
        .iter()
        .flatten()
        .filter_map(|r| resolve(elements, resolver, r).filter(|a| Resolver::is_asset(a)))
        .map(|a| a.qualified_name.clone())
        .collect()
}

/// Every chain of the model, deterministic order.
fn chains(elements: &[RawElement], resolver: &Resolver) -> Vec<Chain> {
    let by_q = |q: &str| elements.iter().find(|e| e.qualified_name == q);
    let mut out: BTreeSet<Chain> = BTreeSet::new();
    let mut treated: HashSet<String> = HashSet::new();
    for g in elements.iter().filter(|e| is_goal(e)) {
        let threats: Vec<&RawElement> = g
            .frontmatter
            .threat_scenarios
            .iter()
            .flatten()
            .filter_map(|r| resolve(elements, resolver, r).filter(|t| Resolver::is_threat_scenario(t)))
            .collect();
        if threats.is_empty() {
            out.insert(Chain { threat: None, damage: None, asset: None, goal: Some(g.qualified_name.clone()) });
        }
        for t in threats {
            treated.insert(t.qualified_name.clone());
            push_threat_chains(&mut out, t, Some(&g.qualified_name), elements, resolver, &by_q);
        }
    }
    for t in elements.iter().filter(|e| Resolver::is_threat_scenario(e)) {
        if !treated.contains(&t.qualified_name) {
            push_threat_chains(&mut out, t, None, elements, resolver, &by_q);
        }
    }
    out.into_iter().collect()
}

fn push_threat_chains<'a>(
    out: &mut BTreeSet<Chain>,
    t: &RawElement,
    goal: Option<&String>,
    elements: &'a [RawElement],
    resolver: &Resolver,
    by_q: &dyn Fn(&str) -> Option<&'a RawElement>,
) {
    let damages = damages_of(t, elements, resolver);
    if damages.is_empty() {
        out.insert(Chain { threat: Some(t.qualified_name.clone()), damage: None, asset: None, goal: goal.cloned() });
    }
    for d in damages {
        let assets = by_q(&d).map(|de| assets_of(de, elements, resolver)).unwrap_or_default();
        if assets.is_empty() {
            out.insert(Chain { threat: Some(t.qualified_name.clone()), damage: Some(d.clone()), asset: None, goal: goal.cloned() });
        }
        for a in assets {
            out.insert(Chain { threat: Some(t.qualified_name.clone()), damage: Some(d.clone()), asset: Some(a), goal: goal.cloned() });
        }
    }
}

fn label_impact(rank: u8) -> &'static str {
    IMPACTS[rank.min(3) as usize]
}

fn threat_mark(t: &RawElement, elements: &[RawElement], resolver: &Resolver, cfg: &CyberConfig, treated: bool) -> NodeMark {
    let mut badges = Vec::new();
    if !treated {
        badges.push("no goal".to_string());
    }
    let feas = threat_feasibility_rank(&t.frontmatter, cfg).map(|r| crate::attack_tree::feasibility_label(r).to_string());
    let value = feas.map(|f| format!("feasibility {f}"));
    match threat_risk(t, elements, resolver, cfg) {
        Some(r) => {
            let status = match r.value {
                Some(v) => format!("risk {} ({v})", r.level.as_str()),
                None => format!("risk {}", r.level.as_str()),
            };
            let mut m = mark(status, value, risk_tone(r.level), badges);
            m.emphasis = r.level == RiskLevel::Critical;
            m
        }
        None => mark("risk unknown", value, Tone::Warn, badges),
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
    if !(is_package(st) || is_member_type(subject)) {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — a ThreatGraph diagram subject must be a Package, TARASheet, ThreatScenario, DamageScenario, Asset, CybersecurityGoal or SecurityControl",
            subject.qualified_name,
            st.name()
        )));
        return;
    }
    // An explicit configuration wins; otherwise the registered model root's `[cyber]`.
    let ctx_cfg;
    let cfg = if filters.cyber.is_default() {
        ctx_cfg = super::context::cyber_config();
        &ctx_cfg
    } else {
        &filters.cyber
    };
    let sq = subject.qualified_name.as_str();
    let all = chains(elements, resolver);
    let controls_of = |goal: &str| -> Vec<&RawElement> {
        elements
            .iter()
            .filter(|c| Resolver::is_security_control(c))
            .filter(|c| {
                c.frontmatter.implements_goals.iter().flatten().any(|r| resolve(elements, resolver, r).is_some_and(|g| g.qualified_name == goal))
            })
            .collect()
    };

    let pkg = is_package(st);
    let touches = |c: &Chain| -> bool {
        let members = [c.threat.as_deref(), c.damage.as_deref(), c.asset.as_deref(), c.goal.as_deref()];
        if pkg {
            return members.iter().flatten().any(|q| is_under(q, sq));
        }
        if Resolver::is_security_control(subject) {
            return c.goal.as_deref().is_some_and(|g| controls_of(g).iter().any(|ct| ct.qualified_name == sq));
        }
        members.iter().flatten().any(|q| *q == sq)
    };
    let mut selected: Vec<&Chain> = all.iter().filter(|c| touches(c)).collect();
    // Elements with no chain at all (a lone asset or control) still deserve a node.
    let mut lone: Vec<&RawElement> = Vec::new();
    if selected.is_empty() {
        if pkg {
            lone.extend(elements.iter().filter(|e| is_member_type(e) && is_under(&e.qualified_name, sq)));
        } else {
            lone.push(subject);
        }
    }
    if selected.is_empty() && lone.is_empty() {
        issues.push(w418(format!("`subject` '{}' reaches no threat, damage, asset or goal — nothing to draw", subject.qualified_name)));
        return;
    }
    selected.sort();

    let by_q = |q: &str| elements.iter().find(|e| e.qualified_name == q);
    let treated: HashSet<&str> = all.iter().filter(|c| c.goal.is_some()).filter_map(|c| c.threat.as_deref()).collect();
    let mut candidates: Vec<Vec<String>> = Vec::new();
    let mut allowed = |e: &RawElement| {
        candidates.push(keys_of(e));
        keeps_keys(filters, &keys_of(e))
    };

    let mut emitted_goals: HashSet<String> = HashSet::new();
    let mut ids: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    // Emit each element once, in chain order: threats, damages, assets, goals, controls.
    let mut node_of = |graph: &mut DiagramGraph, q: &str, allowed: &mut dyn FnMut(&RawElement) -> bool| -> Option<String> {
        if let Some(id) = ids.get(q) {
            return Some(id.clone());
        }
        let e = by_q(q)?;
        if !allowed(e) {
            return None;
        }
        let id = match () {
            _ if Resolver::is_threat_scenario(e) => {
                let m = threat_mark(e, elements, resolver, cfg, treated.contains(q));
                add_node(graph, node_for(e, NodeKind::Block, Some(m)))
            }
            _ if Resolver::is_damage_scenario(e) => {
                let rank = damage_impact_rank(&e.frontmatter);
                let status = rank.map(|r| format!("impact {}", label_impact(r))).unwrap_or_else(|| "impact unknown".to_string());
                let tone = rank.map(impact_tone).unwrap_or(Tone::Warn);
                let value = e.frontmatter.impact_categories.as_ref().map(|c| c.join(", ")).filter(|s| !s.is_empty());
                add_node(graph, node_for(e, NodeKind::Block, Some(mark(status, value, tone, Vec::new()))))
            }
            _ if Resolver::is_asset(e) => {
                let value = e.frontmatter.cybersecurity_properties.as_ref().map(|c| c.join(", ")).filter(|s| !s.is_empty());
                let status = e.frontmatter.status.clone().unwrap_or_else(|| "asset".to_string());
                add_node(graph, node_for(e, NodeKind::Block, Some(mark(status, value, Tone::Neutral, Vec::new()))))
            }
            _ if is_goal(e) => {
                let controls = controls_of(q);
                let mut badges = Vec::new();
                let tone = if controls.is_empty() {
                    badges.push("W802 no control".to_string());
                    Tone::Bad
                } else {
                    Tone::Ok
                };
                let status = e.frontmatter.cal_level.clone().unwrap_or_else(|| "no CAL".to_string());
                let value = e.frontmatter.security_property.as_ref().map(|p| p.join(", ")).filter(|s| !s.is_empty());
                add_node(graph, node_for(e, NodeKind::Goal, Some(mark(status, value, tone, badges))))
            }
            _ => {
                let status = e.frontmatter.control_type.clone().unwrap_or_else(|| "control".to_string());
                let tone = if e.frontmatter.status.as_deref() == Some("draft") { Tone::Warn } else { Tone::Ok };
                add_node(graph, node_for(e, NodeKind::Block, Some(mark(status, e.frontmatter.status.clone(), tone, Vec::new()))))
            }
        };
        ids.insert(q.to_string(), id.clone());
        Some(id)
    };

    for c in &selected {
        let t = c.threat.as_deref().and_then(|q| node_of(graph, q, &mut allowed));
        let d = c.damage.as_deref().and_then(|q| node_of(graph, q, &mut allowed));
        let a = c.asset.as_deref().and_then(|q| node_of(graph, q, &mut allowed));
        let g = c.goal.as_deref().and_then(|q| node_of(graph, q, &mut allowed));
        let link = |graph: &mut DiagramGraph, kind: EdgeKind, from: &Option<String>, to: &Option<String>, r: Option<&str>| {
            if let (Some(f), Some(t)) = (from, to) {
                add_edge(graph, kind, f, t, r.map(str::to_string), None);
            }
        };
        link(graph, EdgeKind::Impacts, &t, &d, c.threat.as_deref());
        link(graph, EdgeKind::Affects, &d, &a, c.damage.as_deref());
        // The goal hangs off the last link of the chain that exists.
        let last = a.clone().or(d.clone()).or(t.clone());
        link(graph, EdgeKind::ProtectedBy, &last, &g, c.goal.as_deref());
        if let (Some(gq), Some(gid)) = (c.goal.as_deref(), g.as_ref()) {
            if emitted_goals.insert(gq.to_string()) {
                for ctl in controls_of(gq) {
                    if !allowed(ctl) {
                        continue;
                    }
                    let cid = node_of(graph, &ctl.qualified_name, &mut allowed);
                    link(graph, EdgeKind::ImplementedBy, &Some(gid.clone()), &cid, Some(&ctl.qualified_name));
                }
            }
        }
    }
    for e in lone {
        node_of(graph, &e.qualified_name, &mut allowed);
    }
    unmatched_key_issues(filters, &candidates, issues);
}

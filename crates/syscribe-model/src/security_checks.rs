//! Cybersecurity-analysis completeness and consistency checks (GH #219, #220,
//! #221; ISO/SAE 21434, IEC 62443). Everything here is additive to the older
//! per-type checks in `validator.rs`:
//!
//! * `E960` — a typed security field has the wrong YAML shape (emitted by the
//!   parse-issue pass; named here for the catalogue);
//! * `E961` / `E962` — `TARASheet` row without an `id` / with an unknown key or
//!   a row that does not deserialize (mirrors FMEA `E923` / `E922`);
//! * `E963` / `E964` — `Asset.assetOwner` / `Asset.relatedSafetyGoal` dangling or
//!   of the wrong type;
//! * `E965` — cycle (including a self-input) in an attack tree's gate `inputs`;
//! * `E967` — `VulnerabilityReport.threatScenarios` dangling / not a ThreatScenario;
//! * `W960`–`W973` — risk-input completeness, `riskTreatment` consistency,
//!   attack-tree shape, orphan analysis elements, zone/conduit consistency;
//! * `W974`–`W979` — `VulnerabilityReport` hygiene (CVE id, status vocabulary,
//!   resolution evidence, CVSS vector/severity).
//!
//! Weakest-link note (`W960` context): an `AND` attack-tree gate rolls up as the
//! MIN of its inputs' feasibility, an `OR` gate as the MAX (see `attack_tree`).

use std::collections::{HashMap, HashSet};

use crate::element::{ElementType as T, RawElement, RawFrontmatter};
use crate::resolver::Resolver;
use crate::risk::{threat_risk_level, RiskLevel};
use crate::validator::{Finding, Severity};

fn warning(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Warning }
}

fn error(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Error }
}

fn disp(e: &RawElement) -> &str {
    e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name)
}

fn is_draft(e: &RawElement) -> bool {
    e.frontmatter.status.as_deref() == Some("draft")
}

fn nonblank(s: &Option<String>) -> bool {
    s.as_deref().is_some_and(|v| !v.trim().is_empty())
}

/// All security-analysis findings. `elements` is the full (exploded) model.
pub fn security_findings(elements: &[RawElement], resolver: &Resolver) -> Vec<Finding> {
    let mut out = Vec::new();
    tara_row_findings(elements, &mut out);
    asset_findings(elements, resolver, &mut out);
    risk_input_findings(elements, resolver, &mut out);
    attack_tree_findings(elements, resolver, &mut out);
    orphan_findings(elements, resolver, &mut out);
    zone_findings(elements, resolver, &mut out);
    vulnerability_findings(elements, resolver, &mut out);
    out
}

// ── TARASheet rows (E961, E962) ─────────────────────────────────────────────

fn tara_row_findings(elements: &[RawElement], out: &mut Vec<Finding>) {
    for sheet in elements.iter().filter(|e| e.frontmatter.element_type == Some(T::TARASheet)) {
        let fm = &sheet.frontmatter;
        let sections: [(&str, &Option<Vec<serde_yaml::Value>>); 5] = [
            ("assetTable", &fm.asset_table),
            ("damageTable", &fm.damage_table),
            ("threatTable", &fm.threat_table),
            ("goalTable", &fm.goal_table),
            ("controlTable", &fm.control_table),
        ];
        for (name, rows) in sections {
            for (idx, row) in rows.iter().flatten().enumerate() {
                let n = idx + 1;
                let Some(map) = row.as_mapping() else {
                    out.push(error(
                        "E961",
                        &sheet.file_path,
                        format!("TARA row {n} in `{name}:` is not a mapping — it has no `id:` and is dropped from the analysis"),
                    ));
                    continue;
                };
                let id = map.get(serde_yaml::Value::String("id".into())).and_then(|v| v.as_str());
                let Some(id) = id else {
                    out.push(error(
                        "E961",
                        &sheet.file_path,
                        format!("TARA row {n} in `{name}:` has no `id:` — it cannot become an element and is dropped from validation and the risk views; give it an id"),
                    ));
                    continue;
                };
                match serde_yaml::from_value::<RawFrontmatter>(row.clone()) {
                    Err(e) => out.push(error(
                        "E962",
                        &sheet.file_path,
                        format!("TARA row '{id}' in `{name}:` does not parse as an element ({e}); the row is silently reduced to an empty element"),
                    )),
                    Ok(parsed) => {
                        let mut keys: Vec<&str> = parsed.extra.keys().map(String::as_str).collect();
                        keys.sort_unstable();
                        for k in keys {
                            out.push(error(
                                "E962",
                                &sheet.file_path,
                                format!("TARA row '{id}' in `{name}:` has unknown key '{k}' — this field is silently ignored"),
                            ));
                        }
                    }
                }
            }
        }
    }
}

// ── Asset references (E963, E964) ───────────────────────────────────────────

fn asset_findings(elements: &[RawElement], resolver: &Resolver, out: &mut Vec<Finding>) {
    for a in elements.iter().filter(|e| Resolver::is_asset(e)) {
        let fm = &a.frontmatter;
        if let Some(owner) = fm.asset_owner.as_deref().filter(|s| !s.trim().is_empty()) {
            if resolver.resolve_ref(elements, owner).is_none() {
                out.push(error(
                    "E963",
                    &a.file_path,
                    format!("Asset.assetOwner '{owner}' does not resolve to any model element"),
                ));
            }
        }
        if let Some(sg) = fm.related_safety_goal.as_deref().filter(|s| !s.trim().is_empty()) {
            match resolver.resolve_ref(elements, sg) {
                None => out.push(error(
                    "E964",
                    &a.file_path,
                    format!("Asset.relatedSafetyGoal '{sg}' does not resolve to any model element"),
                )),
                Some(t) if !Resolver::is_safety_goal(t) => out.push(error(
                    "E964",
                    &a.file_path,
                    format!("Asset.relatedSafetyGoal '{sg}' does not resolve to a SafetyGoal"),
                )),
                _ => {}
            }
        }
    }
}

// ── Risk inputs and treatment consistency (W960–W965) ───────────────────────

fn risk_input_findings(elements: &[RawElement], resolver: &Resolver, out: &mut Vec<Finding>) {
    // Threats / damage scenarios referenced by someone, plus the threats
    // addressed by a CybersecurityGoal.
    let mut addressed: HashSet<String> = HashSet::new();
    for csg in elements.iter().filter(|e| Resolver::is_cybersecurity_goal(e)) {
        for r in csg.frontmatter.threat_scenarios.iter().flatten() {
            if let Some(ts) = resolver.resolve_ref(elements, r) {
                addressed.insert(ts.qualified_name.clone());
            }
        }
    }
    let mut ds_referenced: HashSet<String> = HashSet::new();
    for ts in elements.iter().filter(|e| Resolver::is_threat_scenario(e)) {
        for r in ts.frontmatter.damage_scenarios.iter().flatten() {
            if let Some(ds) = resolver.resolve_ref(elements, r) {
                ds_referenced.insert(ds.qualified_name.clone());
            }
        }
    }

    for ts in elements.iter().filter(|e| Resolver::is_threat_scenario(e)) {
        if is_draft(ts) {
            continue;
        }
        let fm = &ts.frontmatter;
        let id = disp(ts);

        // W960 — risk computes to `unknown` because an input is missing.
        let mut missing: Vec<&str> = Vec::new();
        let mut any_severity = false;
        let mut any_ds = false;
        for r in fm.damage_scenarios.iter().flatten() {
            if let Some(ds) = resolver.resolve_ref(elements, r) {
                if Resolver::is_damage_scenario(ds) {
                    any_ds = true;
                    if ds.frontmatter.damage_severity.is_some() {
                        any_severity = true;
                    }
                }
            }
        }
        if !any_ds {
            missing.push("no resolvable `damageScenarios`");
        } else if !any_severity {
            missing.push("no linked DamageScenario declares `damageSeverity`");
        }
        if fm.attack_feasibility.is_none() {
            missing.push("no `attackFeasibility`");
        }
        if !missing.is_empty() {
            out.push(warning(
                "W960",
                &ts.file_path,
                format!(
                    "ThreatScenario '{id}' cannot have its risk determined ({}) — `cyber-risk` reports risk=unknown (ISO/SAE 21434 §15.8)",
                    missing.join("; ")
                ),
            ));
        }

        let treatment = fm.risk_treatment.as_deref();
        let is_addressed = addressed.contains(&ts.qualified_name);
        // W963 — `reduce` needs a CybersecurityGoal (and through it a control).
        if treatment == Some("reduce") && !is_addressed {
            out.push(warning(
                "W963",
                &ts.file_path,
                format!("ThreatScenario '{id}' has riskTreatment: reduce but no CybersecurityGoal lists it in `threatScenarios` — the reduction is not realised by any goal/control"),
            ));
        }
        // W964 — retaining a high/critical risk needs a recorded rationale.
        if treatment == Some("retain") && !nonblank(&fm.residual_risk) {
            if let Some(l) = threat_risk_level(ts, elements, resolver) {
                if matches!(l, RiskLevel::High | RiskLevel::Critical) {
                    out.push(warning(
                        "W964",
                        &ts.file_path,
                        format!("ThreatScenario '{id}' retains a {} risk without a `residualRisk` rationale", l.as_str()),
                    ));
                }
            }
        }
        // W965 — a residual-risk note without a treatment decision.
        if nonblank(&fm.residual_risk) && treatment.is_none() {
            out.push(warning(
                "W965",
                &ts.file_path,
                format!("ThreatScenario '{id}' declares `residualRisk` but no `riskTreatment` — residual risk is the outcome of a treatment decision"),
            ));
        }
    }

    for ds in elements.iter().filter(|e| Resolver::is_damage_scenario(e)) {
        if is_draft(ds) {
            continue;
        }
        let id = disp(ds);
        // W961 — orphan: no ThreatScenario realises this damage.
        if !ds_referenced.contains(&ds.qualified_name) {
            out.push(warning(
                "W961",
                &ds.file_path,
                format!("DamageScenario '{id}' is not listed in any ThreatScenario.damageScenarios — it contributes to no risk"),
            ));
        }
        // W962 — no asset.
        if ds.frontmatter.assets.as_ref().is_none_or(|a| a.is_empty()) {
            out.push(warning(
                "W962",
                &ds.file_path,
                format!("DamageScenario '{id}' names no `assets` — damage must be attributed to an asset (ISO/SAE 21434 §15.3)"),
            ));
        }
    }
}

// ── Attack-tree shape (E965, W966–W968) ─────────────────────────────────────

fn attack_tree_findings(elements: &[RawElement], resolver: &Resolver, out: &mut Vec<Finding>) {
    for tree in elements.iter().filter(|e| e.frontmatter.element_type == Some(T::AttackTree)) {
        let prefix = format!("{}::", tree.qualified_name);
        let nodes: Vec<&RawElement> = elements
            .iter()
            .filter(|e| {
                e.qualified_name.starts_with(&prefix)
                    && matches!(e.frontmatter.element_type, Some(T::AttackTreeGate) | Some(T::AttackStep))
            })
            .collect();
        if nodes.is_empty() {
            continue;
        }
        let tid = disp(tree);

        // Edges (resolved inputs) and duplicate inputs.
        let mut edges: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut referenced: HashSet<&str> = HashSet::new();
        for g in nodes.iter().filter(|n| n.frontmatter.element_type == Some(T::AttackTreeGate)) {
            let mut seen: HashSet<&str> = HashSet::new();
            for r in g.frontmatter.inputs.iter().flatten() {
                let Some(child) = resolver.resolve_ref(elements, r) else { continue };
                if !seen.insert(child.qualified_name.as_str()) {
                    out.push(warning(
                        "W968",
                        &g.file_path,
                        format!("AttackTreeGate '{}' lists input '{r}' more than once", disp(g)),
                    ));
                }
                if child.qualified_name == g.qualified_name {
                    out.push(error(
                        "E965",
                        &g.file_path,
                        format!("AttackTreeGate '{}' lists itself as an input (cycle in attack tree '{tid}')", disp(g)),
                    ));
                    continue;
                }
                edges.entry(g.qualified_name.as_str()).or_default().push(child.qualified_name.as_str());
                referenced.insert(child.qualified_name.as_str());
            }
        }

        // Cycle detection (iterative DFS with colours).
        let mut state: HashMap<&str, u8> = HashMap::new(); // 1 = on stack, 2 = done
        let mut cyclic: Option<&str> = None;
        'outer: for n in &nodes {
            let start = n.qualified_name.as_str();
            if state.contains_key(start) {
                continue;
            }
            let mut stack: Vec<(&str, usize)> = vec![(start, 0)];
            state.insert(start, 1);
            while let Some(&(cur, i)) = stack.last() {
                let kids = edges.get(cur).map(Vec::as_slice).unwrap_or(&[]);
                if i < kids.len() {
                    stack.last_mut().unwrap().1 += 1;
                    let k = kids[i];
                    match state.get(k) {
                        Some(1) => {
                            cyclic = Some(k);
                            break 'outer;
                        }
                        Some(_) => {}
                        None => {
                            state.insert(k, 1);
                            stack.push((k, 0));
                        }
                    }
                } else {
                    state.insert(cur, 2);
                    stack.pop();
                }
            }
        }
        if let Some(at) = cyclic {
            out.push(error(
                "E965",
                &tree.file_path,
                format!("attack tree '{tid}' has a cycle in its gate `inputs` (through '{at}') — the feasibility roll-up is undefined"),
            ));
        } else {
            // W966 — more than one root (or none): a forest has no single feasibility.
            let roots: Vec<&&RawElement> =
                nodes.iter().filter(|n| !referenced.contains(n.qualified_name.as_str())).collect();
            if roots.len() > 1 {
                out.push(warning(
                    "W966",
                    &tree.file_path,
                    format!(
                        "attack tree '{tid}' has {} root nodes ({}) — the tree needs exactly one top gate/step, otherwise its feasibility is not computed",
                        roots.len(),
                        roots.iter().map(|r| disp(r)).collect::<Vec<_>>().join(", ")
                    ),
                ));
            }
        }

        // W967 — unscored leaf: the roll-up (and W035) silently skip the tree.
        for s in nodes.iter().filter(|n| n.frontmatter.element_type == Some(T::AttackStep)) {
            if s.frontmatter.attack_feasibility.is_none() {
                out.push(warning(
                    "W967",
                    &s.file_path,
                    format!("AttackStep '{}' has no `attackFeasibility` — attack tree '{tid}' cannot be rolled up", disp(s)),
                ));
            }
        }
    }
}

// ── Orphan goals / controls (W969, W970) ────────────────────────────────────

fn orphan_findings(elements: &[RawElement], _resolver: &Resolver, out: &mut Vec<Finding>) {
    for sc in elements.iter().filter(|e| Resolver::is_security_control(e)) {
        if is_draft(sc) {
            continue;
        }
        if sc.frontmatter.implements_goals.as_ref().is_none_or(|g| g.is_empty()) {
            out.push(warning(
                "W969",
                &sc.file_path,
                format!("SecurityControl '{}' has no `implementsGoals` — it is not justified by any CybersecurityGoal", disp(sc)),
            ));
        }
    }
    for g in elements.iter().filter(|e| Resolver::is_cybersecurity_goal(e)) {
        if is_draft(g) {
            continue;
        }
        if g.frontmatter.threat_scenarios.as_ref().is_none_or(|t| t.is_empty()) {
            out.push(warning(
                "W970",
                &g.file_path,
                format!("CybersecurityGoal '{}' has no `threatScenarios` — it is not grounded in a threat analysis", disp(g)),
            ));
        }
    }
}

// ── Zones and conduits (W971–W973) ──────────────────────────────────────────

fn zone_findings(elements: &[RawElement], resolver: &Resolver, out: &mut Vec<Finding>) {
    let is_zone = |e: &RawElement| e.frontmatter.element_type == Some(T::Zone);
    let is_conduit = |e: &RawElement| e.frontmatter.element_type == Some(T::Conduit);

    // part qname -> zones (qname) it belongs to via Zone.members or inZone.
    let mut membership: HashMap<String, Vec<String>> = HashMap::new();
    let mut add = |part: &RawElement, zone: &RawElement| {
        let v = membership.entry(part.qualified_name.clone()).or_default();
        if !v.contains(&zone.qualified_name) {
            v.push(zone.qualified_name.clone());
        }
    };
    for z in elements.iter().filter(|e| is_zone(e)) {
        for m in z.frontmatter.members.iter().flatten() {
            if let Some(p) = resolver.resolve_ref(elements, m) {
                add(p, z);
            }
        }
    }
    for p in elements.iter().filter(|e| e.frontmatter.in_zone.is_some()) {
        if let Some(z) = p.frontmatter.in_zone.as_deref().and_then(|z| resolver.resolve_ref(elements, z)) {
            if is_zone(z) {
                add(p, z);
            }
        }
    }
    let mut parts: Vec<&String> = membership.keys().collect();
    parts.sort();
    for p in parts {
        let zones = &membership[p];
        if zones.len() > 1 {
            let file = elements
                .iter()
                .find(|e| &e.qualified_name == p)
                .map(|e| e.file_path.as_str())
                .unwrap_or("");
            out.push(warning(
                "W971",
                file,
                format!("'{p}' belongs to {} zones ({}) — a component must be in exactly one IEC 62443 zone", zones.len(), zones.join(", ")),
            ));
        }
    }

    for c in elements.iter().filter(|e| is_conduit(e)) {
        let fm = &c.frontmatter;
        if let (Some(f), Some(t)) = (fm.from_zone.as_deref(), fm.to_zone.as_deref()) {
            let (rf, rt) = (resolver.resolve_ref(elements, f), resolver.resolve_ref(elements, t));
            let same = match (rf, rt) {
                (Some(a), Some(b)) => a.qualified_name == b.qualified_name,
                _ => f == t,
            };
            if same {
                out.push(warning(
                    "W972",
                    &c.file_path,
                    format!("Conduit '{}' connects zone '{f}' to itself — a conduit joins two different zones", disp(c)),
                ));
            }
        }
    }

    for e in elements.iter().filter(|e| is_zone(e) || is_conduit(e)) {
        if e.frontmatter.status.as_deref() == Some("approved") && e.frontmatter.achieved_sl.is_none() {
            let kind = if is_zone(e) { "Zone" } else { "Conduit" };
            out.push(warning(
                "W973",
                &e.file_path,
                format!("approved {kind} '{}' declares no `achievedSL` — the security level actually achieved is unrecorded", disp(e)),
            ));
        }
    }
}

// ── VulnerabilityReport (E967, W974–W979) ───────────────────────────────────

/// `CVE-YYYY-NNNN+`.
pub fn is_cve_id(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("CVE-") else { return false };
    let Some((year, seq)) = rest.split_once('-') else { return false };
    year.len() == 4
        && year.bytes().all(|b| b.is_ascii_digit())
        && seq.len() >= 4
        && seq.bytes().all(|b| b.is_ascii_digit())
}

/// Documented `VulnerabilityReport.status` vocabulary.
pub const VR_STATUSES: &[&str] = &[
    "draft", "open", "triaged", "investigating", "in_progress", "mitigated", "resolved", "fixed",
    "accepted", "wont_fix", "closed", "not_affected", "false_positive", "deprecated",
];

/// CVSS severity bucket (v3.x / v4.0 qualitative scale) for a base score.
pub fn cvss_bucket(score: f64) -> &'static str {
    if score <= 0.0 {
        "none"
    } else if score < 4.0 {
        "low"
    } else if score < 7.0 {
        "medium"
    } else if score < 9.0 {
        "high"
    } else {
        "critical"
    }
}

/// A CVSS vector: optional `CVSS:x.y/` prefix, then `/`-separated `KEY:VALUE` metrics.
pub fn is_cvss_vector(s: &str) -> bool {
    let body = match s.strip_prefix("CVSS:") {
        Some(rest) => match rest.split_once('/') {
            Some((ver, b)) if matches!(ver, "3.0" | "3.1" | "4.0") => b,
            _ => return false,
        },
        None => s,
    };
    let mut n = 0;
    for seg in body.split('/') {
        let Some((k, v)) = seg.split_once(':') else { return false };
        if k.is_empty() || v.is_empty() || !k.bytes().all(|b| b.is_ascii_alphabetic()) || !v.bytes().all(|b| b.is_ascii_alphanumeric()) {
            return false;
        }
        n += 1;
    }
    n >= 3
}

fn vulnerability_findings(elements: &[RawElement], resolver: &Resolver, out: &mut Vec<Finding>) {
    for vr in elements.iter().filter(|e| e.frontmatter.element_type == Some(T::VulnerabilityReport)) {
        let fm = &vr.frontmatter;
        let id = disp(vr);
        let f = &vr.file_path;

        for r in fm.threat_scenarios.iter().flatten() {
            match resolver.resolve_ref(elements, r) {
                None => out.push(error("E967", f, format!("VulnerabilityReport.threatScenarios '{r}' does not resolve to any model element"))),
                Some(t) if !Resolver::is_threat_scenario(t) => {
                    out.push(error("E967", f, format!("VulnerabilityReport.threatScenarios '{r}' does not resolve to a ThreatScenario")))
                }
                _ => {}
            }
        }

        if is_draft(vr) {
            continue;
        }
        if let Some(cve) = fm.cve_id.as_deref() {
            if !is_cve_id(cve) {
                out.push(warning("W974", f, format!("VulnerabilityReport '{id}' cveId '{cve}' is not of the form CVE-YYYY-NNNN")));
            }
        }
        let status = fm.status.as_deref();
        if let Some(s) = status {
            if !VR_STATUSES.contains(&s) {
                out.push(warning(
                    "W975",
                    f,
                    format!("VulnerabilityReport '{id}' status '{s}' is not one of {}", VR_STATUSES.join(", ")),
                ));
            }
        }
        let has_mitigation = fm.mitigated_by.as_ref().is_some_and(|m| !m.is_empty());
        if matches!(status, Some("mitigated") | Some("resolved") | Some("fixed") | Some("closed"))
            && !has_mitigation
            && !nonblank(&fm.fixed_in)
        {
            out.push(warning(
                "W976",
                f,
                format!("VulnerabilityReport '{id}' is {} but names no `mitigatedBy` control and no `fixedIn` version", status.unwrap_or("")),
            ));
        }
        if matches!(status, Some("accepted") | Some("wont_fix"))
            && !nonblank(&fm.rationale)
            && !vr.doc.to_ascii_lowercase().contains("rationale")
        {
            out.push(warning(
                "W977",
                f,
                format!("VulnerabilityReport '{id}' is {} without a recorded `rationale` (frontmatter or a Rationale section)", status.unwrap_or("")),
            ));
        }
        // W978 — CVSS vector / severity bucket.
        if let Some(v) = fm.cvss_vector.as_deref() {
            if !is_cvss_vector(v.trim()) {
                out.push(warning("W978", f, format!("VulnerabilityReport '{id}' cvssVector '{v}' is not a CVSS vector string (e.g. CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H)")));
            }
        }
        if let Some(sev) = fm.cvss_severity.as_deref() {
            if !["none", "low", "medium", "high", "critical"].contains(&sev) {
                out.push(warning("W978", f, format!("VulnerabilityReport '{id}' cvssSeverity '{sev}' must be none, low, medium, high, or critical")));
            } else if let Some(score) = fm.cvss_score {
                let want = cvss_bucket(score);
                if want != sev {
                    out.push(warning("W978", f, format!("VulnerabilityReport '{id}' cvssSeverity '{sev}' disagrees with cvssScore {score} ({want} on the CVSS qualitative scale)")));
                }
            }
        }
        // W979 — a scored vulnerability with no link into the threat analysis.
        if fm.cvss_score.is_some_and(|s| s >= 7.0)
            && fm.threat_scenarios.as_ref().is_none_or(|t| t.is_empty())
            && matches!(status, None | Some("open") | Some("triaged") | Some("investigating") | Some("in_progress"))
        {
            out.push(warning(
                "W979",
                f,
                format!("VulnerabilityReport '{id}' has CVSS >= 7.0 but names no `threatScenarios` — link it to the threat it realises (ISO/SAE 21434 §8 / §15)"),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cve_ids() {
        assert!(is_cve_id("CVE-2024-12345"));
        assert!(is_cve_id("CVE-2021-3156"));
        assert!(!is_cve_id("CVE-24-1"));
        assert!(!is_cve_id("cve-2024-12345"));
        assert!(!is_cve_id("GHSA-xxxx"));
    }

    #[test]
    fn cvss_buckets_and_vectors() {
        assert_eq!(cvss_bucket(0.0), "none");
        assert_eq!(cvss_bucket(3.9), "low");
        assert_eq!(cvss_bucket(4.0), "medium");
        assert_eq!(cvss_bucket(7.0), "high");
        assert_eq!(cvss_bucket(9.0), "critical");
        assert!(is_cvss_vector("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H"));
        assert!(is_cvss_vector("AV:N/AC:L/Au:N/C:P/I:P/A:P"));
        assert!(!is_cvss_vector("CVSS:9.9/AV:N/AC:L/PR:N"));
        assert!(!is_cvss_vector("garbage"));
    }
}

//! `fmea report` — renders an FMEA risk table sorted by RPN descending.
//! `fault-tree render` — renders a Mermaid flowchart for a FaultTree.

use syscribe_model::element::{ElementType, RawElement};

// ── fmea report ──────────────────────────────────────────────────────────────

/// Returns the process exit code: 0 on success, 1 when `sheet_filter` names no
/// `FMEASheet` (GH #218 — a typo must not look like an empty, clean report).
pub fn cmd_fmea_report(elements: &[RawElement], sheet_filter: Option<&str>, json: bool) -> i32 {
    // Resolve the sheet (by id or qualified name) and scope rows to its subtree.
    let prefix: Option<String> = match sheet_filter {
        None => None,
        Some(sf) => {
            let sheet = elements.iter().find(|e| {
                matches!(e.frontmatter.element_type, Some(ElementType::FMEASheet))
                    && (e.frontmatter.id.as_deref() == Some(sf) || e.qualified_name == sf)
            });
            match sheet {
                Some(sh) => Some(format!("{}::", sh.qualified_name)),
                None => {
                    eprintln!("Error: no FMEASheet found with id or qualified name '{sf}'");
                    return 1;
                }
            }
        }
    };
    // A duplicate row id inside a sheet synthesises two elements with one qualified
    // name (E108 reports it); list the first only so the table is not inflated.
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut entries: Vec<&RawElement> = elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FMEAEntry)))
        .filter(|e| prefix.as_deref().is_none_or(|p| e.qualified_name.starts_with(p)))
        .filter(|e| seen.insert(e.qualified_name.as_str()))
        .collect();

    // Sort by RPN descending (highest risk first), then by id for stability.
    entries.sort_by(|a, b| {
        let ra = a.frontmatter.rpn.unwrap_or(0);
        let rb = b.frontmatter.rpn.unwrap_or(0);
        rb.cmp(&ra).then(a.qualified_name.cmp(&b.qualified_name))
    });

    if json {
        let items: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                let fm = &e.frontmatter;
                serde_json::json!({
                    "id": fm.id,
                    "failureMode": fm.failure_mode,
                    "effect": fm.effect,
                    "cause": fm.cause,
                    "fmeaSeverity": fm.fmea_severity,
                    "occurrence": fm.occurrence,
                    "detection": fm.detection,
                    "rpn": fm.rpn,
                    "recommendedAction": fm.recommended_action,
                    "failureRate": fm.failure_rate,
                    "diagnosticCoverage": fm.diagnostic_coverage,
                    "latentDiagnosticCoverage": fm.latent_diagnostic_coverage,
                    "status": fm.status,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
        return 0;
    }

    if entries.is_empty() {
        let scope = sheet_filter.map(|s| format!(" in sheet '{s}'")).unwrap_or_default();
        println!("No FMEAEntry elements found{}.", scope);
        return 0;
    }

    let fmeda = entries.iter().any(|e| {
        e.frontmatter.failure_rate.is_some() || e.frontmatter.diagnostic_coverage.is_some()
    });
    let dash = || "—".to_string();
    if fmeda {
        println!("| ID | Failure Mode | Effect | Severity | Occurrence | Detection | RPN | Recommended Action | λ (/h) | DC | Status |");
        println!("|---|---|---|---|---|---|---|---|---|---|---|");
    } else {
        println!("| ID | Failure Mode | Effect | Severity | Occurrence | Detection | RPN | Recommended Action | Status |");
        println!("|---|---|---|---|---|---|---|---|---|");
    }
    for e in &entries {
        let fm = &e.frontmatter;
        let id = fm.id.as_deref().unwrap_or("—");
        // `name` is the failure-mode label for a row; fall back to it only when
        // `failureMode` is absent.
        let failure_mode = fm.failure_mode.as_deref().or(fm.name.as_deref()).unwrap_or("—");
        let effect = fm.effect.as_deref().unwrap_or("—");
        let sev = fm.fmea_severity.map(|n| n.to_string()).unwrap_or_else(dash);
        let occ = fm.occurrence.map(|n| n.to_string()).unwrap_or_else(dash);
        let det = fm.detection.map(|n| n.to_string()).unwrap_or_else(dash);
        let rpn = fm.rpn.map(|n| n.to_string()).unwrap_or_else(dash);
        let action = fm.recommended_action.as_deref().unwrap_or("—");
        let status = fm.status.as_deref().unwrap_or("—");
        if fmeda {
            let lam = fm.failure_rate.map(|v| format!("{v:.2e}")).unwrap_or_else(dash);
            let dc = fm.diagnostic_coverage.map(|v| format!("{v}")).unwrap_or_else(dash);
            println!("| {id} | {failure_mode} | {effect} | {sev} | {occ} | {det} | {rpn} | {action} | {lam} | {dc} | {status} |");
        } else {
            println!("| {id} | {failure_mode} | {effect} | {sev} | {occ} | {det} | {rpn} | {action} | {status} |");
        }
    }
    println!();
    0
}

// ── fault-tree render ─────────────────────────────────────────────────────────

pub fn cmd_fault_tree_render(elements: &[RawElement], ft_id: &str) {
    // Find the FaultTree element by id or qualified name.
    let ft = elements.iter().find(|e| {
        matches!(e.frontmatter.element_type, Some(ElementType::FaultTree))
            && (e.frontmatter.id.as_deref() == Some(ft_id)
                || e.qualified_name == ft_id)
    });

    let ft = match ft {
        Some(e) => e,
        None => {
            eprintln!("Error: no FaultTree element found with id or qualified name '{}'", ft_id);
            std::process::exit(1);
        }
    };

    let prefix = format!("{}::", ft.qualified_name);

    // Collect all child gates and events.
    let children: Vec<&RawElement> = elements
        .iter()
        .filter(|e| {
            e.qualified_name.starts_with(&prefix)
                && matches!(
                    e.frontmatter.element_type,
                    Some(ElementType::FaultTreeGate) | Some(ElementType::FaultTreeEvent)
                )
        })
        .collect();

    println!("flowchart TD");

    // Emit node declarations.
    for e in &children {
        let node_id = mermaid_id(&e.qualified_name);
        let label = e.frontmatter.name.as_deref()
            .or(e.frontmatter.id.as_deref())
            .unwrap_or(&e.qualified_name);
        let eid = e.frontmatter.id.as_deref().unwrap_or("?");
        match e.frontmatter.element_type {
            Some(ElementType::FaultTreeGate) => {
                let gate = e.frontmatter.gate_type.as_deref().unwrap_or("AND");
                println!("    {}[\"{} [{}] {}\"]", node_id, eid, gate, label);
            }
            Some(ElementType::FaultTreeEvent) => {
                let kind = e.frontmatter.event_kind.as_deref().unwrap_or("basic");
                // REQ-TRS-FTA-002: name the modelled element (ref:) on a second line.
                let modelled = e.frontmatter.event_ref.as_deref()
                    .map(|r| format!("<br/>ref: {}", r))
                    .unwrap_or_default();
                println!("    {}(\"[{}] {} {}{}\")", node_id, kind, eid, label, modelled);
            }
            _ => {}
        }
    }

    // Emit edges from gate inputs.
    for e in &children {
        if !matches!(e.frontmatter.element_type, Some(ElementType::FaultTreeGate)) {
            continue;
        }
        let gate_id = mermaid_id(&e.qualified_name);
        let inputs = e.frontmatter.inputs.as_deref().unwrap_or(&[]);
        for inp in inputs {
            // Resolve to a child element (by id or suffix).
            let target = children.iter().find(|c| {
                c.frontmatter.id.as_deref() == Some(inp.as_str())
                    || c.qualified_name == inp.as_str()
                    || c.qualified_name.ends_with(&format!("::{}", inp))
            });
            if let Some(t) = target {
                println!("    {} --> {}", gate_id, mermaid_id(&t.qualified_name));
            } else {
                println!("    {} --> {}", gate_id, mermaid_id(inp));
            }
        }
    }
}

fn mermaid_id(qname: &str) -> String {
    qname.replace("::", "_").replace(['-', ' ', '.'], "_")
}

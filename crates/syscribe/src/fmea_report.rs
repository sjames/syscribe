//! `fmea report` — renders an FMEA risk table sorted by RPN descending.
//! `fault-tree render` — renders a Mermaid flowchart for a FaultTree.

use syscribe_model::element::{ElementType, RawElement};

// ── fmea report ──────────────────────────────────────────────────────────────

pub fn cmd_fmea_report(elements: &[RawElement], sheet_filter: Option<&str>, json: bool) {
    let mut entries: Vec<&RawElement> = elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FMEAEntry)))
        .filter(|e| {
            // If a sheet filter is given, keep only entries whose qualified name
            // starts with "<sheet-id>::" or "<sheet-qname>::".
            sheet_filter.is_none_or(|sf| {
                e.qualified_name.contains(&format!("{}::", sf))
                    || e.qualified_name.starts_with(&format!("{}::", sf))
            })
        })
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
                    "name": fm.name,
                    "failureMode": fm.failure_mode,
                    "effect": fm.effect,
                    "fmeaSeverity": fm.fmea_severity,
                    "occurrence": fm.occurrence,
                    "detection": fm.detection,
                    "rpn": fm.rpn,
                    "recommendedAction": fm.recommended_action,
                    "status": fm.status,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
        return;
    }

    if entries.is_empty() {
        let scope = sheet_filter.map(|s| format!(" in sheet '{s}'")).unwrap_or_default();
        println!("No FMEAEntry elements found{}.", scope);
        return;
    }

    println!("| ID | Name | Failure Mode | Effect | Severity | Occurrence | Detection | RPN | Controls | Status |");
    println!("|---|---|---|---|---|---|---|---|---|---|");
    for e in &entries {
        let fm = &e.frontmatter;
        let id = fm.id.as_deref().unwrap_or("—");
        let name = fm.name.as_deref().unwrap_or("—");
        let failure_mode = fm.failure_mode.as_deref().unwrap_or("—");
        let effect = fm.effect.as_deref().unwrap_or("—");
        let sev = fm.fmea_severity.map(|n| n.to_string()).unwrap_or_else(|| "—".into());
        let occ = fm.occurrence.map(|n| n.to_string()).unwrap_or_else(|| "—".into());
        let det = fm.detection.map(|n| n.to_string()).unwrap_or_else(|| "—".into());
        let rpn = fm.rpn.map(|n| n.to_string()).unwrap_or_else(|| "—".into());
        let controls = fm.recommended_action.as_deref().unwrap_or("—");
        let status = fm.status.as_deref().unwrap_or("—");
        println!("| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            id, name, failure_mode, effect, sev, occ, det, rpn, controls, status);
    }
    println!();
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
                println!("    {}(\"[{}] {} {}\")", node_id, kind, eid, label);
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

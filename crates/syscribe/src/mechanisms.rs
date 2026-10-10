//! `mechanisms [--json] [--uncovered]` — what each SafetyMechanism covers (GH #236,
//! REQ-TRS-SAFEMECH-001).

use serde_json::json;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;

fn id_of(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

pub fn cmd_mechanisms(elems: &[RawElement], json_out: bool, uncovered: bool) -> i32 {
    let resolver = Resolver::new(elems);
    let mechs: Vec<&RawElement> = elems.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::SafetyMechanism)).collect();
    if uncovered {
        // FMEA rows no (non-retired) mechanism covers.
        let covered: std::collections::HashSet<String> = mechs
            .iter()
            .filter(|m| m.frontmatter.status.as_deref() != Some("retired"))
            .flat_map(|m| m.frontmatter.covers.iter().flatten())
            .filter_map(|r| resolver.resolve_ref(elems, r).map(|t| t.qualified_name.clone()))
            .collect();
        let mut rows: Vec<&RawElement> = elems
            .iter()
            .filter(|e| e.frontmatter.element_type == Some(ElementType::FMEAEntry) && !covered.contains(&e.qualified_name))
            .collect();
        rows.sort_by_key(|e| id_of(e));
        if json_out {
            let l: Vec<_> = rows.iter().map(|e| json!({"id": id_of(e), "name": e.frontmatter.name})).collect();
            println!("{}", serde_json::to_string_pretty(&json!({"uncovered": l})).unwrap_or_default());
        } else if rows.is_empty() {
            println!("Every FMEA row is covered by a safety mechanism.");
        } else {
            println!("FMEA rows covered by no safety mechanism ({}):", rows.len());
            for e in rows {
                println!("  {}  {}", id_of(e), e.frontmatter.name.as_deref().unwrap_or(""));
            }
        }
        return 0;
    }
    let mut mechs = mechs;
    mechs.sort_by_key(|e| id_of(e));
    if json_out {
        let l: Vec<_> = mechs
            .iter()
            .map(|m| {
                let fm = &m.frontmatter;
                json!({
                    "id": id_of(m), "name": fm.name, "status": fm.status,
                    "covers": fm.covers.clone().unwrap_or_default().iter().map(|r| resolver.resolve_ref(elems, r).map(id_of).unwrap_or_else(|| r.clone())).collect::<Vec<_>>(),
                    "diagnosticCoverage": fm.diagnostic_coverage, "latentDiagnosticCoverage": fm.latent_diagnostic_coverage,
                    "reactionTime": fm.reaction_time, "safeState": fm.safe_state, "allocatedTo": fm.allocated_to,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json!({"mechanisms": l})).unwrap_or_default());
    } else if mechs.is_empty() {
        println!("No SafetyMechanism elements.");
    } else {
        for m in mechs {
            let fm = &m.frontmatter;
            let pct = |v: Option<f64>| v.map(|x| format!("{:.0}%", x * 100.0)).unwrap_or_else(|| "-".into());
            println!(
                "{}  {}  DC {}  latent {}  reaction {}  safe state: {}",
                id_of(m),
                fm.name.as_deref().unwrap_or(""),
                pct(fm.diagnostic_coverage),
                pct(fm.latent_diagnostic_coverage),
                fm.reaction_time.as_deref().unwrap_or("-"),
                fm.safe_state.as_deref().unwrap_or("-")
            );
            for r in fm.covers.iter().flatten() {
                println!("    covers {}", resolver.resolve_ref(elems, r).map(id_of).unwrap_or_else(|| format!("{r} (unresolved)")));
            }
            for a in fm.allocated_to.iter().flatten() {
                println!("    allocated to {a}");
            }
        }
    }
    0
}

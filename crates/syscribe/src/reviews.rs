//! Read-only `reviews` / `review` surface for native `ReviewRecord` elements (§19, GH #71).
//!
//! - `reviews [<qname>] [--open-only] [--json]` — list review records (optionally only those
//!   with open action items, or only those covering a given element).
//! - `reviews --coverage [--json]` — native-Requirement review-coverage cross-table.
//! - `review <RR-id> [--json]` — full detail for one record.

use syscribe_model::{
    element::{ElementType, RawElement},
    resolver::Resolver,
};

fn is_review(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::ReviewRecord))
}

fn id_of(e: &RawElement) -> &str {
    e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name)
}

/// Disposition counts `(open, total)` over an element's `items:`.
fn item_counts(e: &RawElement) -> (usize, usize) {
    let items = match &e.frontmatter.items {
        Some(i) => i,
        None => return (0, 0),
    };
    let open = items
        .iter()
        .filter(|it| {
            it.as_mapping()
                .and_then(|m| m.get(serde_yaml::Value::String("disposition".into())))
                .and_then(|v| v.as_str())
                == Some("open")
        })
        .count();
    (open, items.len())
}

/// True when ReviewRecord `rr` covers element reference `target` (by id or qualified name).
fn covers(rr: &RawElement, target_id: &str, target_qn: &str) -> bool {
    rr.frontmatter
        .reviews
        .as_ref()
        .map(|rs| rs.iter().any(|r| r == target_id || r == target_qn))
        .unwrap_or(false)
}

pub fn cmd_reviews(elements: &[RawElement], filter: Option<&str>, open_only: bool, coverage: bool, json: bool) {
    let resolver = Resolver::new(elements);

    if coverage {
        return coverage_report(elements, &resolver, json);
    }

    // Resolve a positional element filter to its id + qualified name, if given.
    let filter_keys = filter.and_then(|f| {
        resolver
            .resolve_ref(elements, f)
            .map(|t| (t.frontmatter.id.clone().unwrap_or_default(), t.qualified_name.clone()))
            .or(Some((f.to_string(), f.to_string())))
    });

    let mut records: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_review(e))
        .filter(|e| {
            if let Some((id, qn)) = &filter_keys {
                covers(e, id, qn)
            } else {
                true
            }
        })
        .filter(|e| !open_only || item_counts(e).0 > 0)
        .collect();
    records.sort_by(|a, b| id_of(a).cmp(id_of(b)));

    if json {
        let arr: Vec<serde_json::Value> = records
            .iter()
            .map(|e| {
                let (open, total) = item_counts(e);
                serde_json::json!({
                    "id": id_of(e),
                    "name": e.frontmatter.name,
                    "status": e.frontmatter.status,
                    "reviewType": e.frontmatter.review_type,
                    "reviewDate": e.frontmatter.review_date,
                    "recordedAt": e.frontmatter.recorded_at,
                    "covers": e.frontmatter.reviews.clone().unwrap_or_default(),
                    "openItems": open,
                    "totalItems": total,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "reviews": arr })).unwrap());
        return;
    }

    if records.is_empty() {
        println!("No ReviewRecord elements{}.", filter.map(|f| format!(" covering '{}'", f)).unwrap_or_default());
        return;
    }
    println!("| ID | Status | Type | Date | Covers | Open items |");
    println!("|---|---|---|---|---|---|");
    for e in &records {
        let (open, total) = item_counts(e);
        let n = e.frontmatter.reviews.as_ref().map(|r| r.len()).unwrap_or(0);
        println!(
            "| {} | {} | {} | {} | {} | {}/{} |",
            id_of(e),
            e.frontmatter.status.as_deref().unwrap_or("—"),
            e.frontmatter.review_type.as_deref().unwrap_or("—"),
            e.frontmatter.review_date.as_deref().unwrap_or("—"),
            n,
            open,
            total,
        );
    }
}

fn coverage_report(elements: &[RawElement], resolver: &Resolver, json: bool) {
    let mut rows: Vec<(String, String, Vec<String>)> = Vec::new(); // (req id, status, covering RR ids)
    for el in elements {
        if !Resolver::is_native_requirement(el) {
            continue;
        }
        let id = id_of(el).to_string();
        let covering: Vec<String> = elements
            .iter()
            .filter(|e| is_review(e))
            .filter(|rr| covers(rr, &id, &el.qualified_name))
            .map(|rr| id_of(rr).to_string())
            .collect();
        rows.push((id, el.frontmatter.status.clone().unwrap_or_default(), covering));
    }
    rows.sort();

    if json {
        let arr: Vec<serde_json::Value> = rows
            .iter()
            .map(|(id, status, rrs)| {
                serde_json::json!({ "id": id, "status": status, "covered": !rrs.is_empty(), "reviews": rrs })
            })
            .collect();
        let covered = rows.iter().filter(|(_, _, r)| !r.is_empty()).count();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "total": rows.len(), "covered": covered, "requirements": arr
            }))
            .unwrap()
        );
        let _ = resolver;
        return;
    }

    if rows.is_empty() {
        println!("No native Requirement elements to report coverage for.");
        return;
    }
    let covered = rows.iter().filter(|(_, _, r)| !r.is_empty()).count();
    println!("| Requirement | Status | Covered | Reviews |");
    println!("|---|---|---|---|");
    for (id, status, rrs) in &rows {
        println!(
            "| {} | {} | {} | {} |",
            id,
            status,
            if rrs.is_empty() { "✗" } else { "✓" },
            if rrs.is_empty() { "—".to_string() } else { rrs.join(", ") },
        );
    }
    println!("\n**Coverage:** {}/{} requirements reviewed.", covered, rows.len());
}

/// `review <RR-id>` — detail view. Returns a process exit code (1 if not found).
pub fn cmd_review(elements: &[RawElement], rr_id: &str, json: bool) -> i32 {
    let rec = elements.iter().find(|e| is_review(e) && id_of(e) == rr_id);
    let rec = match rec {
        Some(r) => r,
        None => {
            eprintln!("No ReviewRecord with id '{}' found.", rr_id);
            return 1;
        }
    };
    let fm = &rec.frontmatter;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "id": id_of(rec),
                "name": fm.name,
                "status": fm.status,
                "reviewType": fm.review_type,
                "reviewDate": fm.review_date,
                "reviewedBy": fm.reviewed_by.clone().unwrap_or_default(),
                "recordedAt": fm.recorded_at,
                "reviews": fm.reviews.clone().unwrap_or_default(),
                "items": fm.items.clone().unwrap_or_default(),
            }))
            .unwrap()
        );
        return 0;
    }
    println!("# {} — {}", id_of(rec), fm.name.as_deref().unwrap_or(""));
    println!();
    println!("- **Status:** {}", fm.status.as_deref().unwrap_or("—"));
    println!("- **Type:** {}", fm.review_type.as_deref().unwrap_or("—"));
    println!("- **Date:** {}", fm.review_date.as_deref().unwrap_or("—"));
    if let Some(rb) = &fm.reviewed_by {
        println!("- **Reviewed by:** {}", rb.join(", "));
    }
    if let Some(at) = &fm.recorded_at {
        println!("- **Recorded at:** {}", at);
    }
    println!("\n## Reviews");
    for r in fm.reviews.clone().unwrap_or_default() {
        println!("- {}", r);
    }
    if let Some(items) = &fm.items {
        println!("\n## Action items");
        for it in items {
            if let Some(m) = it.as_mapping() {
                let g = |k: &str| {
                    m.get(serde_yaml::Value::String(k.into()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                };
                println!("- [{}] {} {}", g("disposition"), g("description"),
                    if g("closedBy").is_empty() { String::new() } else { format!("(closedBy: {})", g("closedBy")) });
            }
        }
    }
    0
}

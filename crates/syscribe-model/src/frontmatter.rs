use anyhow::{Context, Result};
use crate::element::RawFrontmatter;

/// Split a `.md` file content into (frontmatter_yaml, markdown_body).
/// Returns (None, full_content) if no YAML front matter block found.
pub fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    let content = content.trim_start_matches('\u{FEFF}'); // strip BOM — still borrows from param
    if !content.starts_with("---") {
        return (None, content);
    }
    // Find closing ---
    let after_open = &content[3..];
    let close = after_open.find("\n---").or_else(|| after_open.find("\r\n---"));
    match close {
        None => (None, content),
        Some(pos) => {
            let yaml = after_open[..pos].trim_start_matches('\n').trim_start_matches('\r');
            let rest_start = pos + 4; // skip "\n---"
            let body = after_open[rest_start..].trim_start_matches('\n').trim_start_matches('\r');
            (Some(yaml), body)
        }
    }
}

/// Parse YAML frontmatter string into `RawFrontmatter`.
pub fn parse_frontmatter(yaml: &str) -> Result<RawFrontmatter> {
    serde_yaml::from_str(yaml).context("Failed to parse YAML frontmatter")
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ-TRS-ORDER-001 — the generic `displayOrder` field parses as a first-class
    // numeric (integer or decimal), not swallowed into the `extra` catch-all.
    #[test]
    fn display_order_parses_integer_and_decimal() {
        let int_fm = parse_frontmatter("type: Requirement\nid: REQ-AA-001\ndisplayOrder: 20").unwrap();
        assert_eq!(int_fm.display_order, Some(20.0));
        assert!(!int_fm.extra.contains_key("displayOrder"));

        let dec_fm = parse_frontmatter("type: Requirement\nid: REQ-AA-002\ndisplayOrder: 15.5").unwrap();
        assert_eq!(dec_fm.display_order, Some(15.5));
    }

    // REQ-TRS-ORDER-001 — absent `displayOrder` yields `None`, and its sort key is
    // `+∞` so unordered elements sink below every element that declares an order.
    #[test]
    fn absent_display_order_sinks_last() {
        let fm = parse_frontmatter("type: Requirement\nid: REQ-AA-003").unwrap();
        assert_eq!(fm.display_order, None);
        assert_eq!(fm.display_order_key(), f64::INFINITY);
        assert!(fm.display_order_key() > 10_000.0);
    }

    // REQ-TRS-SCHEMA-002 — `reqClass` is a recognized field, bound to the model and
    // kept out of the `extra` catch-all.
    #[test]
    fn req_class_is_recognized_not_extra() {
        let fm = parse_frontmatter(
            "type: Requirement\nid: REQ-AA-001\nreqClass: stakeholder",
        )
        .unwrap();
        assert_eq!(fm.req_class.as_deref(), Some("stakeholder"));
        assert!(!fm.extra.contains_key("reqClass"));
    }

    // REQ-TRS-SCHEMA-001 — a genuinely unrecognized key is captured by `extra`
    // (where the validator picks it up for W047), not silently absent.
    #[test]
    fn unknown_key_lands_in_extra() {
        let fm = parse_frontmatter("type: PartDef\nname: Widget\nwibble: 3").unwrap();
        assert!(fm.extra.contains_key("wibble"));
        // A recognized field on the same element is NOT in extra.
        assert!(!fm.extra.contains_key("name"));
    }

    // REQ-TRS-ORDER-001 — the comparator orders ascending, sinks unset last, and
    // tie-breaks on the stable identifier (mirrors the report / matrix sort).
    #[test]
    fn display_order_comparator_matches_spec() {
        let mk = |id: &str, ord: Option<f64>| {
            let mut fm = RawFrontmatter::default();
            fm.id = Some(id.to_string());
            fm.display_order = ord;
            fm
        };
        let mut fms = vec![
            mk("REQ-AA-003", Some(30.0)),
            mk("REQ-AA-009", None),
            mk("REQ-AA-001", Some(10.0)),
            mk("REQ-AA-008", None),
            mk("REQ-AA-004", Some(15.0)),
            mk("REQ-AA-002", Some(20.0)),
        ];
        fms.sort_by(|a, b| {
            a.display_order_key()
                .total_cmp(&b.display_order_key())
                .then_with(|| a.id.as_deref().unwrap_or("").cmp(b.id.as_deref().unwrap_or("")))
        });
        let ids: Vec<&str> = fms.iter().map(|f| f.id.as_deref().unwrap()).collect();
        assert_eq!(
            ids,
            vec!["REQ-AA-001", "REQ-AA-004", "REQ-AA-002", "REQ-AA-003", "REQ-AA-008", "REQ-AA-009"]
        );
    }
}

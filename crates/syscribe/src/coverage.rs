//! Shared requirement-verification coverage computation.
//!
//! Extracted from the MCP `coverage` tool so the `export-html` coverage report
//! and the MCP tool agree on one partition. Mirrors W300/W305 (non-draft
//! gating): a leaf requirement is a gap when it is non-draft and has no
//! verifying TestCase; a parent (one with `derivedChildren`) needs at least one
//! integration-level (L3/L4/L5) verifying TestCase.

use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::ValidationResult;

/// One requirement listed in a coverage partition.
pub struct CoverageEntry {
    pub qname: String,
    pub id: Option<String>,
    pub name: Option<String>,
    /// Number of derived children — `Some` only for parent entries.
    pub child_count: Option<usize>,
}

/// The requirement-verification coverage partition over a model.
pub struct CoverageSummary {
    /// Requirements counted as verified (a leaf with any verifying TestCase, or
    /// a parent with an integration-level verifying TestCase).
    pub verified_count: u64,
    /// Non-draft leaf requirements with no verifying TestCase.
    pub unverified_leaves: Vec<CoverageEntry>,
    /// Non-draft parent requirements lacking an integration-level (L3+) TestCase.
    pub parents_missing_integration: Vec<CoverageEntry>,
}

/// Compute the coverage partition. `result` must come from
/// `validate_with_config` over the same `elements`.
pub fn coverage_summary(elements: &[RawElement], result: &ValidationResult) -> CoverageSummary {
    let resolver = Resolver::new(elements);
    let mut verified_count = 0u64;
    let mut unverified_leaves = Vec::new();
    let mut parents_missing_integration = Vec::new();

    for e in elements {
        if !Resolver::is_native_requirement(e) {
            continue;
        }
        let id = e.frontmatter.id.as_deref().unwrap_or("");
        // Parent ⇔ it has non-empty derivedChildren (two-level model / W305).
        let children = result.derived_children.get(id);
        let child_count = children.map(|c| c.len()).unwrap_or(0);
        let has_children = child_count > 0;
        let verifying_tcs = result.verified_by.get(id);

        // An "integration" test is testLevel L3/L4/L5 on a verifying TestCase.
        let has_integration_tc = verifying_tcs.is_some_and(|tcs| {
            tcs.iter().any(|tc_id| {
                resolver
                    .resolve_ref(elements, tc_id)
                    .and_then(|tc| tc.frontmatter.test_level.as_deref())
                    .is_some_and(|lvl| matches!(lvl, "L3" | "L4" | "L5"))
            })
        });

        // Gaps are reported only for non-draft requirements (mirror W300/W305,
        // which are suppressed on draft / planned work).
        let reportable = matches!(
            e.frontmatter.status.as_deref(),
            Some("approved") | Some("implemented") | Some("verified")
        );

        if has_children {
            if has_integration_tc {
                verified_count += 1;
            } else if reportable {
                parents_missing_integration.push(CoverageEntry {
                    qname: e.qualified_name.clone(),
                    id: e.frontmatter.id.clone(),
                    name: e.frontmatter.name.clone(),
                    child_count: Some(child_count),
                });
            }
        } else if verifying_tcs.is_some_and(|tcs| !tcs.is_empty()) {
            verified_count += 1;
        } else if reportable {
            unverified_leaves.push(CoverageEntry {
                qname: e.qualified_name.clone(),
                id: e.frontmatter.id.clone(),
                name: e.frontmatter.name.clone(),
                child_count: None,
            });
        }
    }

    CoverageSummary {
        verified_count,
        unverified_leaves,
        parents_missing_integration,
    }
}

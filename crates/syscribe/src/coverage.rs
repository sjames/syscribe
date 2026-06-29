//! Shared requirement-verification coverage computation.
//!
//! Extracted from the MCP `coverage` tool so the `export-html` coverage report
//! and the MCP tool agree on one partition. Mirrors W300/W305 (non-draft
//! gating): a leaf requirement is a gap when it is non-draft and has no
//! verifying TestCase; a parent (one with `derivedChildren`) needs at least one
//! integration-level (L3/L4/L5) verifying TestCase.

use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::results::ResultsData;
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
    /// Requirements counted as verified (a leaf covered by a non-draft TestCase in
    /// every applicable configuration, or a parent with an integration-level TestCase).
    pub verified_count: u64,
    /// Non-draft leaf requirements with a verification gap (no TestCase, or failing).
    pub unverified_leaves: Vec<CoverageEntry>,
    /// Non-draft leaf requirements linked only by draft TestCases (verification intent).
    pub planned: Vec<CoverageEntry>,
    /// Non-draft parent requirements lacking an integration-level (L3+) TestCase.
    pub parents_missing_integration: Vec<CoverageEntry>,
}

/// Compute the coverage partition. `result` must come from
/// `validate_with_config` over the same `elements`.
pub fn coverage_summary(
    elements: &[RawElement],
    result: &ValidationResult,
    results: Option<&ResultsData>,
) -> CoverageSummary {
    let resolver = Resolver::new(elements);
    // Leaves are classified by the unified per-cell classifier collapsed per
    // requirement (so `coverage` cannot contradict `coverage_matrix`).
    let rollup = crate::matrix::requirement_rollup(elements, results);
    let mut verified_count = 0u64;
    let mut unverified_leaves = Vec::new();
    let mut planned = Vec::new();
    let mut parents_missing_integration = Vec::new();

    let entry = |e: &RawElement, child_count: Option<usize>| CoverageEntry {
        qname: e.qualified_name.clone(),
        id: e.frontmatter.id.clone(),
        name: e.frontmatter.name.clone(),
        child_count,
    };

    for e in elements {
        if !Resolver::is_native_requirement(e) {
            continue;
        }
        let id = e.frontmatter.id.as_deref().unwrap_or("");
        // Parent ⇔ it has non-empty derivedChildren (two-level model / W305).
        let child_count = result.derived_children.get(id).map(|c| c.len()).unwrap_or(0);
        let has_children = child_count > 0;

        // Gaps are reported only for non-draft requirements (mirror W300/W305,
        // which are suppressed on draft / planned work).
        let reportable = matches!(
            e.frontmatter.status.as_deref(),
            Some("approved") | Some("implemented") | Some("verified")
        );

        if has_children {
            // Parent: needs an integration-level (L3/L4/L5) verifying TestCase.
            let has_integration_tc = result.verified_by.get(id).is_some_and(|tcs| {
                tcs.iter().any(|tc_id| {
                    resolver
                        .resolve_ref(elements, tc_id)
                        .and_then(|tc| tc.frontmatter.test_level.as_deref())
                        .is_some_and(|lvl| matches!(lvl, "L3" | "L4" | "L5"))
                })
            });
            if has_integration_tc {
                verified_count += 1;
            } else if reportable {
                parents_missing_integration.push(entry(e, Some(child_count)));
            }
        } else {
            // Leaf: row-collapse of the matrix classifier.
            let key = e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone());
            match rollup.get(&key).copied().unwrap_or("na") {
                "verified" => verified_count += 1,
                "planned" if reportable => planned.push(entry(e, None)),
                "unverified" if reportable => unverified_leaves.push(entry(e, None)),
                _ => {} // planned/unverified on a draft req, or na: not reported
            }
        }
    }

    CoverageSummary {
        verified_count,
        unverified_leaves,
        planned,
        parents_missing_integration,
    }
}

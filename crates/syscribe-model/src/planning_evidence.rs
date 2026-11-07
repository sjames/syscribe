//! PlanningItem evidence, read across the model (GH #456, `REQ-TRS-PIEV-001`).
//!
//! `validate` (`W319`, behind `[planning] require_testcase_evidence`) and `coverage` share
//! [`testcases_without_planning_evidence`], so the warning and the report never disagree.

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use std::collections::HashSet;

/// The `ref:` values of a PlanningItem's `evidence:` entries, in authored order.
pub fn evidence_refs(e: &RawElement) -> impl Iterator<Item = &str> {
    e.frontmatter
        .evidence
        .iter()
        .flatten()
        .filter_map(|v| v.as_mapping())
        .filter_map(|m| m.get(serde_yaml::Value::String("ref".into())).and_then(|v| v.as_str()))
}

/// Every `status: active` TestCase that no PlanningItem's `evidence: - ref:` names, in model order.
///
/// A `ref:` names a TestCase when it resolves to it by stable id or qualified name (the resolution
/// `E716` uses). A ref is counted whatever the entry's `rationale:` and whatever the item's status:
/// the question is whether the work is recorded against a plan, not whether the item is finished.
pub fn testcases_without_planning_evidence<'a>(elements: &'a [RawElement], resolver: &Resolver) -> Vec<&'a RawElement> {
    let named: HashSet<&str> = elements
        .iter()
        .filter(|e| e.frontmatter.element_type == Some(ElementType::PlanningItem))
        .flat_map(evidence_refs)
        .filter_map(|r| resolver.resolve_ref(elements, r))
        .map(|t| t.qualified_name.as_str())
        .collect();
    elements
        .iter()
        .filter(|e| e.frontmatter.element_type == Some(ElementType::TestCase))
        .filter(|e| e.frontmatter.status.as_deref() == Some("active"))
        .filter(|e| !named.contains(e.qualified_name.as_str()))
        .collect()
}

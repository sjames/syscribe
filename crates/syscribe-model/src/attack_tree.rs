//! ISO/SAE 21434 §15.7 attack path analysis — weakest-link feasibility roll-up.
//!
//! An `AttackTree` (id `AT-*`) substantiates a `ThreatScenario` (via `threatRef`)
//! and decomposes it into `AttackTreeGate`s (id `ATG-*`, `gateType` AND|OR) and
//! `AttackStep`s (id `ATS-*`, leaf with `attackFeasibility`). The feasibility of
//! a tree is rolled up — the security analog of FTA quantitative roll-up — using
//! the **weakest-link** rule:
//!
//! - feasibility rank (shared with the risk model): very_low=0, low=1, medium=2,
//!   high=3;
//! - an `AttackStep`'s value is the rank of its `attackFeasibility`;
//! - an `AttackTreeGate` `AND` (a sequential path — all sub-steps needed) is the
//!   **MIN** of its children's values (a chain is only as feasible as its hardest
//!   step);
//! - an `AttackTreeGate` `OR` (alternatives) is the **MAX** of its children's
//!   values (the attacker takes the easiest path);
//! - the `AttackTree`'s feasibility is the value of its single root child (the
//!   gate/step it contains), mapped back to a label.
//!
//! There is exactly ONE roll-up definition; the validator's W035 reconciliation
//! uses [`tree_feasibility`].

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::risk::feasibility_rank;

/// Map a feasibility rank (0..=3) back to its label.
/// 0→very_low, 1→low, 2→medium, 3→high.
pub fn feasibility_label(rank: u8) -> &'static str {
    match rank {
        0 => "very_low",
        1 => "low",
        2 => "medium",
        _ => "high",
    }
}

/// Roll up the feasibility rank of a single node (gate or step) of an attack
/// tree, recursing through `AttackTreeGate.inputs`. Returns `None` when the node
/// (or, transitively, a contributing leaf) carries no computable feasibility.
///
/// `depth` guards against cyclic `inputs` (treated as uncomputable past a sane
/// bound); the resolver itself reports dangling refs separately.
fn node_rank(
    node: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    depth: u32,
) -> Option<u8> {
    if depth > 256 {
        return None;
    }
    match node.frontmatter.element_type {
        Some(ElementType::AttackStep) => node
            .frontmatter
            .attack_feasibility
            .as_deref()
            .and_then(feasibility_rank),
        Some(ElementType::AttackTreeGate) => {
            let gate_type = node.frontmatter.gate_type.as_deref()?;
            let inputs = node.frontmatter.inputs.as_ref()?;
            let mut acc: Option<u8> = None;
            for r in inputs {
                let child = resolver.resolve_ref(elements, r)?;
                let v = node_rank(child, elements, resolver, depth + 1)?;
                acc = Some(match (acc, gate_type) {
                    (None, _) => v,
                    // AND = a path; weakest link = MIN.
                    (Some(a), "AND") => a.min(v),
                    // OR = alternatives; attacker's easiest = MAX.
                    (Some(a), "OR") => a.max(v),
                    // Unknown gate type → not computable.
                    (Some(_), _) => return None,
                });
            }
            acc
        }
        _ => None,
    }
}

/// Computed feasibility **rank** (0..=3) of an `AttackTree`: the value of its
/// single root child (the `AttackTreeGate`/`AttackStep` whose qualified name is
/// directly under the tree). `None` when the tree has no root child or the
/// roll-up is not computable. `tree` must be an `AttackTree`.
pub fn tree_feasibility_rank(
    tree: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> Option<u8> {
    if tree.frontmatter.element_type != Some(ElementType::AttackTree) {
        return None;
    }
    let prefix = format!("{}::", tree.qualified_name);
    // The root child is the gate/step directly under the tree (one segment of
    // qualified name below the tree prefix).
    let root = elements.iter().find(|e| {
        let qn = &e.qualified_name;
        qn.starts_with(&prefix)
            && !qn[prefix.len()..].contains("::")
            && matches!(
                e.frontmatter.element_type,
                Some(ElementType::AttackTreeGate) | Some(ElementType::AttackStep)
            )
    })?;
    node_rank(root, elements, resolver, 0)
}

/// Computed feasibility **label** of an `AttackTree`, or `None` if not computable.
pub fn tree_feasibility(
    tree: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> Option<&'static str> {
    tree_feasibility_rank(tree, elements, resolver).map(feasibility_label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ElementType, RawElement, RawFrontmatter};

    fn step(qn: &str, id: &str, feas: &str) -> RawElement {
        RawElement {
            qualified_name: qn.to_string(),
            file_path: format!("{}.md", qn),
            frontmatter: RawFrontmatter {
                element_type: Some(ElementType::AttackStep),
                id: Some(id.to_string()),
                attack_feasibility: Some(feas.to_string()),
                ..Default::default()
            },
            doc: String::new(),
            parse_issue: None,
            derived: std::collections::HashMap::new(),
            derive_findings: Vec::new(),
        }
    }

    fn gate(qn: &str, id: &str, gt: &str, inputs: &[&str]) -> RawElement {
        RawElement {
            qualified_name: qn.to_string(),
            file_path: format!("{}.md", qn),
            frontmatter: RawFrontmatter {
                element_type: Some(ElementType::AttackTreeGate),
                id: Some(id.to_string()),
                gate_type: Some(gt.to_string()),
                inputs: Some(inputs.iter().map(|s| s.to_string()).collect()),
                ..Default::default()
            },
            doc: String::new(),
            parse_issue: None,
            derived: std::collections::HashMap::new(),
            derive_findings: Vec::new(),
        }
    }

    fn tree(qn: &str, id: &str) -> RawElement {
        RawElement {
            qualified_name: qn.to_string(),
            file_path: format!("{}.md", qn),
            frontmatter: RawFrontmatter {
                element_type: Some(ElementType::AttackTree),
                id: Some(id.to_string()),
                threat_ref: Some("TS-DEMO-001".to_string()),
                ..Default::default()
            },
            doc: String::new(),
            parse_issue: None,
            derived: std::collections::HashMap::new(),
            derive_findings: Vec::new(),
        }
    }

    /// Worked example from GH #32:
    /// AT-DEMO-001 root OR ATG-DEMO-001 [ATG-DEMO-002, ATS-DEMO-003].
    /// ATG-DEMO-002 AND [ATS-DEMO-001(high=3), ATS-DEMO-002(low=1)] → min = 1 (low).
    /// ATS-DEMO-003 medium = 2.
    /// root OR = max(1, 2) = 2 → tree feasibility = medium.
    #[test]
    fn rollup_worked_example_is_medium() {
        let elements = vec![
            tree("AT-DEMO-001", "AT-DEMO-001"),
            gate(
                "AT-DEMO-001::ATG-DEMO-001",
                "ATG-DEMO-001",
                "OR",
                &["ATG-DEMO-002", "ATS-DEMO-003"],
            ),
            gate(
                "AT-DEMO-001::ATG-DEMO-002",
                "ATG-DEMO-002",
                "AND",
                &["ATS-DEMO-001", "ATS-DEMO-002"],
            ),
            step("AT-DEMO-001::ATS-DEMO-001", "ATS-DEMO-001", "high"),
            step("AT-DEMO-001::ATS-DEMO-002", "ATS-DEMO-002", "low"),
            step("AT-DEMO-001::ATS-DEMO-003", "ATS-DEMO-003", "medium"),
        ];
        let resolver = Resolver::new(&elements);
        let at = &elements[0];
        assert_eq!(tree_feasibility_rank(at, &elements, &resolver), Some(2));
        assert_eq!(tree_feasibility(at, &elements, &resolver), Some("medium"));
    }
}

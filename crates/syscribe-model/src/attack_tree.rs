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
//! - the `AttackTree`'s feasibility is the value of its root node — the single
//!   gate/step under the tree that no other gate of the tree lists in its
//!   `inputs:` (independent of file order, GH #149) — mapped back to a label.
//!
//! There is exactly ONE roll-up definition; the validator's W035 reconciliation
//! uses [`tree_feasibility`].

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::cyber_config::CyberConfig;
use crate::risk::threat_feasibility_rank;

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
    cfg: &CyberConfig,
    depth: u32,
) -> Option<u8> {
    if depth > 256 {
        return None;
    }
    match node.frontmatter.element_type {
        Some(ElementType::AttackStep) => threat_feasibility_rank(&node.frontmatter, cfg),
        Some(ElementType::AttackTreeGate) => {
            let gate_type = node.frontmatter.gate_type.as_deref()?;
            let inputs = node.frontmatter.inputs.as_ref()?;
            let mut acc: Option<u8> = None;
            for r in inputs {
                let child = resolver.resolve_ref(elements, r)?;
                let v = node_rank(child, elements, resolver, cfg, depth + 1)?;
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

/// The root node of an `AttackTree`: the single `AttackTreeGate`/`AttackStep`
/// under the tree's qualified-name prefix that no other gate of the same tree
/// lists in its `inputs:` (GH #149). The root is a property of the tree's
/// structure, never of directory/file order — a sub-gate that happens to sort
/// first is not the root. `None` when there is no such node or more than one
/// (a forest, or every node is some gate's input — a cycle): the roll-up is
/// then not computable rather than silently picking one.
pub fn tree_root<'a>(
    tree: &RawElement,
    elements: &'a [RawElement],
    resolver: &Resolver,
) -> Option<&'a RawElement> {
    let prefix = format!("{}::", tree.qualified_name);
    let nodes: Vec<&RawElement> = elements
        .iter()
        .filter(|e| {
            e.qualified_name.starts_with(&prefix)
                && matches!(
                    e.frontmatter.element_type,
                    Some(ElementType::AttackTreeGate) | Some(ElementType::AttackStep)
                )
        })
        .collect();
    let mut referenced: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for g in &nodes {
        for r in g.frontmatter.inputs.iter().flatten() {
            if let Some(child) = resolver.resolve_ref(elements, r) {
                if child.qualified_name != g.qualified_name {
                    referenced.insert(child.qualified_name.as_str());
                }
            }
        }
    }
    let mut roots = nodes
        .into_iter()
        .filter(|n| !referenced.contains(n.qualified_name.as_str()));
    let root = roots.next()?;
    if roots.next().is_some() {
        return None;
    }
    Some(root)
}

/// The rolled-up feasibility rank of one node (gate or step) of an attack
/// tree, under `cfg`: the same recursion [`tree_feasibility_rank_with`] runs
/// from the root, exposed so a diagram can colour every node of the tree
/// (GH #223). `None` when the node is not computable.
pub fn node_feasibility_rank(
    node: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    cfg: &CyberConfig,
) -> Option<u8> {
    node_rank(node, elements, resolver, cfg, 0)
}

/// How a tree's computed feasibility compares with its linked threat's
/// declared one (the W035 reconciliation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reconciliation {
    /// The `threatRef:` as written.
    pub threat_ref: String,
    /// The threat's declared (or attack-potential-derived) feasibility.
    pub declared: String,
    /// The tree's computed feasibility.
    pub computed: &'static str,
}

impl Reconciliation {
    /// Whether the two disagree (W035).
    pub fn mismatch(&self) -> bool {
        self.computed != self.declared
    }
}

/// The W035 reconciliation of `tree`: `Some` only when its `threatRef`
/// resolves to a `ThreatScenario` and both feasibilities are computable. The
/// validator's W035 and the attack-tree diagram both read this one definition.
pub fn reconcile(
    tree: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    cfg: &CyberConfig,
) -> Option<Reconciliation> {
    if tree.frontmatter.element_type != Some(ElementType::AttackTree) {
        return None;
    }
    let tr = tree.frontmatter.threat_ref.as_ref()?;
    let threat = resolver.resolve_ref(elements, tr)?;
    if !Resolver::is_threat_scenario(threat) {
        return None;
    }
    // Declared = explicit attackFeasibility, else the one the threat's own
    // attack-potential factors compute to (GH #222).
    let declared: String = match threat.frontmatter.attack_feasibility.as_deref() {
        Some(d) => d.to_string(),
        None => feasibility_label(threat_feasibility_rank(&threat.frontmatter, cfg)?).to_string(),
    };
    let computed = tree_feasibility_with(tree, elements, resolver, cfg)?;
    Some(Reconciliation { threat_ref: tr.clone(), declared, computed })
}

/// Computed feasibility **rank** (0..=3) of an `AttackTree`: the value of its
/// root node ([`tree_root`] — the gate/step no other gate of the tree lists as
/// an input). `None` when the tree has no unique root or the roll-up is not
/// computable. `tree` must be an `AttackTree`.
pub fn tree_feasibility_rank(
    tree: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> Option<u8> {
    tree_feasibility_rank_with(tree, elements, resolver, &CyberConfig::default())
}

/// [`tree_feasibility_rank`] under an explicit [`CyberConfig`] (attack-potential
/// scoring of steps that supply factor fields instead of `attackFeasibility`).
pub fn tree_feasibility_rank_with(
    tree: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    cfg: &CyberConfig,
) -> Option<u8> {
    if tree.frontmatter.element_type != Some(ElementType::AttackTree) {
        return None;
    }
    let root = tree_root(tree, elements, resolver)?;
    node_rank(root, elements, resolver, cfg, 0)
}

/// Computed feasibility **label** of an `AttackTree`, or `None` if not computable.
pub fn tree_feasibility(
    tree: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> Option<&'static str> {
    tree_feasibility_rank(tree, elements, resolver).map(feasibility_label)
}

/// [`tree_feasibility`] under an explicit [`CyberConfig`].
pub fn tree_feasibility_with(
    tree: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    cfg: &CyberConfig,
) -> Option<&'static str> {
    tree_feasibility_rank_with(tree, elements, resolver, cfg).map(feasibility_label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ElementType, RawElement, RawFrontmatter};

    fn step(qn: &str, id: &str, feas: &str) -> RawElement {
        RawElement {
            qualified_name: qn.to_string(),
            file_path: format!("{}.md", qn),
            frontmatter: {
                let mut fm = RawFrontmatter {
                element_type: Some(ElementType::AttackStep),
                id: Some(id.to_string()),
                ..Default::default()
                };
                fm.attack_feasibility = Some(feas.to_string());
                fm
            },
            doc: String::new(),
            parse_issue: None,
            derived: std::collections::HashMap::new(),
            derive_findings: Vec::new(),
            locale_docs: Default::default(),
            about_notes: Default::default(),
        }
    }

    fn gate(qn: &str, id: &str, gt: &str, inputs: &[&str]) -> RawElement {
        RawElement {
            qualified_name: qn.to_string(),
            file_path: format!("{}.md", qn),
            frontmatter: {
                let mut fm = RawFrontmatter {
                element_type: Some(ElementType::AttackTreeGate),
                id: Some(id.to_string()),
                ..Default::default()
                };
                fm.gate_type = Some(gt.to_string());
                fm.inputs = Some(inputs.iter().map(|s| s.to_string()).collect());
                fm
            },
            doc: String::new(),
            parse_issue: None,
            derived: std::collections::HashMap::new(),
            derive_findings: Vec::new(),
            locale_docs: Default::default(),
            about_notes: Default::default(),
        }
    }

    fn tree(qn: &str, id: &str) -> RawElement {
        RawElement {
            qualified_name: qn.to_string(),
            file_path: format!("{}.md", qn),
            frontmatter: {
                let mut fm = RawFrontmatter {
                element_type: Some(ElementType::AttackTree),
                id: Some(id.to_string()),
                ..Default::default()
                };
                fm.threat_ref = Some("TS-DEMO-001".to_string());
                fm
            },
            doc: String::new(),
            parse_issue: None,
            derived: std::collections::HashMap::new(),
            derive_findings: Vec::new(),
            locale_docs: Default::default(),
            about_notes: Default::default(),
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

    /// GH #149: the root is the node no gate lists as an input, whatever the
    /// element order. A sub-gate (AND = low) sorted before the root (OR =
    /// medium) must not be taken as the root, in either order.
    #[test]
    fn rollup_root_is_independent_of_element_order() {
        let nodes = vec![
            gate("AT-XY-001::ATG-XY-001", "ATG-XY-001", "AND", &["ATS-XY-001", "ATS-XY-002"]),
            gate("AT-XY-001::ATG-XY-002", "ATG-XY-002", "OR", &["ATG-XY-001", "ATS-XY-003"]),
            step("AT-XY-001::ATS-XY-001", "ATS-XY-001", "high"),
            step("AT-XY-001::ATS-XY-002", "ATS-XY-002", "low"),
            step("AT-XY-001::ATS-XY-003", "ATS-XY-003", "medium"),
        ];
        for reversed in [false, true] {
            let mut elements = vec![tree("AT-XY-001", "AT-XY-001")];
            let mut ns = nodes.clone();
            if reversed {
                ns.reverse();
            }
            elements.extend(ns);
            let resolver = Resolver::new(&elements);
            let at = &elements[0];
            assert_eq!(
                tree_root(at, &elements, &resolver).and_then(|r| r.frontmatter.id.as_deref()),
                Some("ATG-XY-002")
            );
            assert_eq!(tree_feasibility(at, &elements, &resolver), Some("medium"));
        }
    }

    /// Two unreferenced nodes (a forest) have no unique root: not computable.
    #[test]
    fn rollup_forest_has_no_root() {
        let elements = vec![
            tree("AT-YZ-001", "AT-YZ-001"),
            step("AT-YZ-001::ATS-YZ-001", "ATS-YZ-001", "high"),
            step("AT-YZ-001::ATS-YZ-002", "ATS-YZ-002", "low"),
        ];
        let resolver = Resolver::new(&elements);
        assert!(tree_root(&elements[0], &elements, &resolver).is_none());
        assert_eq!(tree_feasibility_rank(&elements[0], &elements, &resolver), None);
    }
}

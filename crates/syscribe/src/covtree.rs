//! `coverage tree <req>` — roll test coverage up the derivation tree (GH #252, REQ-TRS-COVTREE-001).
//! Read-only; reuses the matrix classifier for leaves and the validator's reverse indices.

use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use syscribe_model::config::{is_integrity_rated, CoveragePolicy, ParentRule};
use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::results::ResultsData;
use syscribe_model::validator::ValidationResult;

struct Node {
    id: String,
    name: Option<String>,
    status: Option<String>,
    leaf: bool,
    /// `verified` | `planned` | `unverified` | `na` — own state for a leaf.
    own: &'static str,
    /// Distinct leaf descendants (id -> state); a leaf holds itself. A diamond counts once.
    leaves: BTreeMap<String, &'static str>,
    direct: usize,
    rule: ParentRule,
    children: Vec<Node>,
}

impl Node {
    fn count(&self, state: &str) -> usize {
        self.leaves.values().filter(|s| **s == state).count()
    }
    fn active(&self) -> usize {
        self.count("verified")
    }
    fn planned(&self) -> usize {
        self.count("planned")
    }
    fn uncovered(&self) -> usize {
        self.count("unverified")
    }
    /// `●` complete · `◐` partial · `○` nothing · `·` not applicable in any configuration.
    fn glyph(&self) -> char {
        if self.leaf {
            return match self.own {
                "verified" => '●',
                "planned" => '◐',
                "na" => '·',
                _ => '○',
            };
        }
        let total = self.leaves.len();
        let all_leaves = total > 0 && self.active() == total;
        let complete = match self.rule {
            ParentRule::Both => all_leaves && self.direct > 0,
            ParentRule::Rollup => all_leaves,
            ParentRule::Direct => self.direct > 0,
        };
        if total == 0 && self.direct == 0 {
            '·'
        } else if complete {
            '●'
        } else if self.active() == 0 && self.planned() == 0 && self.direct == 0 {
            '○'
        } else {
            '◐'
        }
    }
    fn verdict(&self) -> &'static str {
        match self.glyph() {
            '●' => "complete",
            '◐' => "partial",
            '○' => "none",
            _ => "na",
        }
    }
    fn own_label(&self) -> &'static str {
        match self.own {
            "unverified" => "uncovered",
            o => o,
        }
    }
}

fn key(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn build(
    e: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    result: &ValidationResult,
    states: &std::collections::HashMap<String, &'static str>,
    path: &mut HashSet<String>,
    policy: &CoveragePolicy,
    violations: &mut Vec<String>,
) -> Node {
    let k = key(e);
    path.insert(k.clone());
    let (rule, source) = policy.rule_for(&e.frontmatter);
    let leaf_now = result.derived_children.get(&k).is_none_or(|v| v.is_empty());
    if !leaf_now && rule != ParentRule::Both && is_integrity_rated(&e.frontmatter) {
        violations.push(format!(
            "{k} is integrity-rated (ASIL/CAL/SIL) but {source} sets parent_rule = \"{}\"; an integrity-rated parent needs \"both\"",
            rule.as_str()
        ));
    }
    let mut child_ids: Vec<&String> = result.derived_children.get(&k).map(|v| v.iter().collect()).unwrap_or_default();
    child_ids.sort();
    // A leaf is a requirement with nothing derived from it — children skipped only because they
    // close a cycle on the current path do not make it one.
    let leaf = child_ids.is_empty();
    let mut kids: Vec<Node> = Vec::new();
    for cid in child_ids {
        if path.contains(cid) {
            continue;
        }
        if let Some(c) = resolver.resolve_ref(elements, cid) {
            kids.push(build(c, elements, resolver, result, states, path, policy, violations));
        }
    }
    path.remove(&k);
    // Only active TestCases count as a direct test (a draft one is intent, not evidence).
    let direct = result
        .verified_by
        .get(&k)
        .map(|v| {
            v.iter()
                .filter(|tc| resolver.resolve_ref(elements, tc).is_some_and(|t| t.frontmatter.status.as_deref() == Some("active")))
                .count()
        })
        .unwrap_or(0);
    let own = states.get(&k).copied().unwrap_or("na");
    let mut leaves: BTreeMap<String, &'static str> = BTreeMap::new();
    if leaf {
        if own != "na" {
            leaves.insert(k.clone(), own);
        }
    } else {
        for c in &kids {
            leaves.extend(c.leaves.iter().map(|(a, b)| (a.clone(), *b)));
        }
    }
    Node {
        id: k,
        name: e.frontmatter.name.clone(),
        status: e.frontmatter.status.clone(),
        leaf,
        own,
        leaves,
        direct,
        rule,
        children: kids,
    }
}

fn render(n: &Node, depth: usize, out: &mut String) {
    let ind = "  ".repeat(depth);
    let st = n.status.as_deref().unwrap_or("-");
    if n.leaf {
        out.push_str(&format!("{ind}{} {}  {} | direct tests {} [{st}]\n", n.glyph(), n.id, n.own_label(), n.direct));
    } else {
        let total = n.leaves.len();
        out.push_str(&format!(
            "{ind}{} {}  leaves {}/{total} active, {} planned | direct tests {} [{st}] (rule: {})\n",
            n.glyph(),
            n.id,
            n.active(),
            n.planned(),
            n.direct,
            n.rule.as_str()
        ));
    }
    for c in &n.children {
        render(c, depth + 1, out);
    }
}

fn to_json(n: &Node) -> Value {
    json!({
        "id": n.id,
        "name": n.name,
        "status": n.status,
        "leaf": n.leaf,
        "own": if n.leaf { json!(n.own_label()) } else { Value::Null },
        "verdict": n.verdict(),
        "glyph": n.glyph().to_string(),
        "rule": if n.leaf { Value::Null } else { json!(n.rule.as_str()) },
        "leavesActive": n.active(),
        "leavesPlanned": n.planned(),
        "leavesUncovered": n.uncovered(),
        "directTests": n.direct,
        "children": n.children.iter().map(to_json).collect::<Vec<_>>(),
    })
}

/// Entry point. Returns the process exit code.
pub fn cmd_coverage_tree(
    elements: &[RawElement],
    result: &ValidationResult,
    results: Option<&ResultsData>,
    policy: &CoveragePolicy,
    root: &str,
    json_out: bool,
) -> i32 {
    if !policy.problems.is_empty() {
        for p in &policy.problems {
            eprintln!("coverage tree: {p}");
        }
        return 1;
    }
    let resolver = Resolver::new(elements);
    let Some(r) = resolver.resolve_ref(elements, root).filter(|e| Resolver::is_native_requirement(e)) else {
        eprintln!("coverage tree: '{root}' does not resolve to a requirement.");
        return 1;
    };
    let states = crate::matrix::requirement_rollup(elements, results);
    let mut violations = Vec::new();
    let tree = build(r, elements, &resolver, result, &states, &mut HashSet::new(), policy, &mut violations);
    if !violations.is_empty() {
        for v in &violations {
            eprintln!("coverage tree: [coverage] policy error: {v}");
        }
        return 1;
    }
    if json_out {
        println!("{}", serde_json::to_string_pretty(&to_json(&tree)).unwrap_or_default());
    } else {
        let mut s = String::new();
        render(&tree, 0, &mut s);
        print!("{s}");
    }
    0
}

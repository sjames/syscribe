//! `coverage tree <req>` — roll test coverage up the derivation tree (GH #252, REQ-TRS-COVTREE-001).
//! Read-only; reuses the matrix classifier for leaves and the validator's reverse indices.

use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use syscribe_model::config::{is_integrity_rated, CoveragePolicy, ParentRule};
use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::results::ResultsData;
use syscribe_model::validator::ValidationResult;

#[derive(Clone)]
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
    let (rule, _source) = policy.rule_for(&e.frontmatter);
    if let Some(v) = guard_violation(e, result, policy) {
        violations.push(v);
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

/// The integrity guard of `[coverage]`: a parent that is integrity-rated must keep the `both` rule.
fn guard_violation(e: &RawElement, result: &ValidationResult, policy: &CoveragePolicy) -> Option<String> {
    let k = key(e);
    let (rule, source) = policy.rule_for(&e.frontmatter);
    let is_parent = result.derived_children.get(&k).is_some_and(|v| !v.is_empty());
    (is_parent && rule != ParentRule::Both && is_integrity_rated(&e.frontmatter)).then(|| {
        format!(
            "{k} is integrity-rated (ASIL/CAL/SIL) but {source} sets parent_rule = \"{}\"; an integrity-rated parent needs \"both\"",
            rule.as_str()
        )
    })
}

/// Coverage of one requirement without the child tree, memoised across the whole model so a
/// requirement reachable by many `derivedFrom` routes is computed once (diamonds would otherwise be
/// walked once per route). A node cut short by a cycle back-edge is not memoised.
#[allow(clippy::too_many_arguments)]
fn summarise(
    e: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    result: &ValidationResult,
    states: &std::collections::HashMap<String, &'static str>,
    path: &mut HashSet<String>,
    policy: &CoveragePolicy,
    memo: &mut std::collections::HashMap<String, Node>,
) -> (Node, bool) {
    let k = key(e);
    if let Some(n) = memo.get(&k) {
        return (n.clone(), false);
    }
    path.insert(k.clone());
    let mut child_ids: Vec<&String> = result.derived_children.get(&k).map(|v| v.iter().collect()).unwrap_or_default();
    child_ids.sort();
    let leaf = child_ids.is_empty();
    let mut cut = false;
    let mut leaves: BTreeMap<String, &'static str> = BTreeMap::new();
    for cid in child_ids {
        if path.contains(cid) {
            cut = true;
            continue;
        }
        if let Some(c) = resolver.resolve_ref(elements, cid) {
            let (cn, ccut) = summarise(c, elements, resolver, result, states, path, policy, memo);
            cut |= ccut;
            leaves.extend(cn.leaves);
        }
    }
    path.remove(&k);
    let own = states.get(&k).copied().unwrap_or("na");
    if leaf && own != "na" {
        leaves.insert(k.clone(), own);
    }
    let direct = result
        .verified_by
        .get(&k)
        .map(|v| v.iter().filter(|tc| resolver.resolve_ref(elements, tc).is_some_and(|t| t.frontmatter.status.as_deref() == Some("active"))).count())
        .unwrap_or(0);
    let (rule, _) = policy.rule_for(&e.frontmatter);
    let n = Node { id: k.clone(), name: e.frontmatter.name.clone(), status: e.frontmatter.status.clone(), leaf, own, leaves, direct, rule, children: Vec::new() };
    if !cut {
        memo.insert(k, n.clone());
    }
    (n, cut)
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
    violations.sort();
    violations.dedup();
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

/// `matrix --rollup`: one row per native requirement with its own and below-it coverage
/// (GH #252, REQ-TRS-COVROLL-001). Same semantics as `coverage tree`, applied to every requirement.
#[allow(clippy::too_many_arguments)]
pub fn cmd_rollup(
    elements: &[RawElement],
    result: &ValidationResult,
    results: Option<&ResultsData>,
    policy: &CoveragePolicy,
    tag: Option<&str>,
    status: Option<&str>,
    json_out: bool,
) -> i32 {
    if !policy.problems.is_empty() {
        for p in &policy.problems {
            eprintln!("matrix --rollup: {p}");
        }
        return 1;
    }
    let resolver = Resolver::new(elements);
    let states = crate::matrix::requirement_rollup(elements, results);
    let mut reqs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| Resolver::is_native_requirement(e))
        .filter(|e| status.is_none_or(|s| e.frontmatter.status.as_deref() == Some(s)))
        .filter(|e| tag.is_none_or(|t| e.frontmatter.tags.iter().flatten().any(|x| x == t)))
        .collect();
    reqs.sort_by_key(|e| key(e));
    // The integrity guard covers every requirement, whatever the display filters keep.
    let mut violations: Vec<String> = elements.iter().filter(|e| Resolver::is_native_requirement(e)).filter_map(|e| guard_violation(e, result, policy)).collect();
    violations.sort();
    violations.dedup();
    if !violations.is_empty() {
        for v in &violations {
            eprintln!("matrix --rollup: [coverage] policy error: {v}");
        }
        return 1;
    }
    let mut memo = std::collections::HashMap::new();
    let rows: Vec<(&RawElement, Node)> = reqs
        .into_iter()
        .map(|e| {
            let (n, _) = summarise(e, elements, &resolver, result, &states, &mut HashSet::new(), policy, &mut memo);
            (e, n)
        })
        .collect();
    let mut by_class: BTreeMap<String, BTreeMap<&'static str, usize>> = BTreeMap::new();
    for (e, n) in &rows {
        let class = e.frontmatter.req_class.clone().unwrap_or_else(|| "-".into());
        *by_class.entry(class).or_default().entry(n.verdict()).or_default() += 1;
    }
    if json_out {
        let list: Vec<_> = rows
            .iter()
            .map(|(e, n)| {
                json!({
                    "id": n.id, "name": n.name, "reqClass": e.frontmatter.req_class, "status": n.status, "leaf": n.leaf,
                    "own": if n.leaf { json!(n.own_label()) } else { Value::Null },
                    "directTests": n.direct, "leavesActive": n.active(), "leavesPlanned": n.planned(), "leavesUncovered": n.uncovered(),
                    "verdict": n.verdict(), "glyph": n.glyph().to_string(), "rule": if n.leaf { Value::Null } else { json!(n.rule.as_str()) },
                })
            })
            .collect();
        let foot: BTreeMap<&String, Value> = by_class
            .iter()
            .map(|(c, m)| {
                let g = |k: &str| m.get(k).copied().unwrap_or(0);
                (c, json!({"complete": g("complete"), "partial": g("partial"), "none": g("none"), "na": g("na")}))
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json!({"rows": list, "byClass": foot})).unwrap_or_default());
        return 0;
    }
    let w0 = rows.iter().map(|(_, n)| n.id.chars().count()).chain([2]).max().unwrap_or(2);
    println!("{:<w0$}  {:<11} {:<11} {:<14} {:<34} verdict (rule)", "id", "class", "status", "own", "below");
    for (e, n) in &rows {
        let own = if n.leaf { n.own_label().to_string() } else { format!("direct {}", n.direct) };
        let below = if n.leaf { "-".to_string() } else { format!("{}/{} active, {} planned", n.active(), n.leaves.len(), n.planned()) };
        println!(
            "{:<w0$}  {:<11} {:<11} {:<14} {:<34} {} ({})",
            n.id,
            e.frontmatter.req_class.as_deref().unwrap_or("-"),
            n.status.as_deref().unwrap_or("-"),
            own,
            below,
            n.glyph(),
            if n.leaf { "-" } else { n.rule.as_str() }
        );
    }
    println!("\nBy reqClass:");
    for (c, m) in &by_class {
        let g = |k: &str| m.get(k).copied().unwrap_or(0);
        println!("  {c:<12} ●{} ◐{} ○{} ·{}", g("complete"), g("partial"), g("none"), g("na"));
    }
    0
}

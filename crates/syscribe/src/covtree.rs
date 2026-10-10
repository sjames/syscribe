//! `coverage tree <req>` — roll test coverage up the derivation tree (GH #252, REQ-TRS-COVTREE-001).
//! Read-only; reuses the matrix classifier for leaves and the validator's reverse indices.

use serde_json::{json, Value};
use std::collections::HashSet;
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
    active: usize,
    planned: usize,
    uncovered: usize,
    direct: usize,
    children: Vec<Node>,
}

impl Node {
    fn glyph(&self) -> char {
        if self.leaf {
            return match self.own {
                "verified" => '●',
                "planned" => '◐',
                "na" => '·',
                _ => '○',
            };
        }
        let total = self.active + self.planned + self.uncovered;
        if total > 0 && self.active == total && self.direct > 0 {
            '●'
        } else if self.active == 0 && self.planned == 0 && self.direct == 0 {
            '○'
        } else {
            '◐'
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
    seen: &mut HashSet<String>,
) -> Node {
    let k = key(e);
    seen.insert(k.clone());
    let mut kids: Vec<Node> = Vec::new();
    let mut child_ids: Vec<&String> = result.derived_children.get(&k).map(|v| v.iter().collect()).unwrap_or_default();
    child_ids.sort();
    for cid in child_ids {
        if seen.contains(cid) {
            continue; // cycle or diamond already shown on this path
        }
        if let Some(c) = resolver.resolve_ref(elements, cid) {
            kids.push(build(c, elements, resolver, result, states, seen));
        }
    }
    for c in &kids {
        seen.remove(&c.id);
    }
    let direct = result.verified_by.get(&k).map(|v| v.len()).unwrap_or(0);
    let leaf = kids.is_empty();
    let own = states.get(&k).copied().unwrap_or("na");
    let (mut active, mut planned, mut uncovered) = (0, 0, 0);
    if leaf {
        match own {
            "verified" => active = 1,
            "planned" => planned = 1,
            "unverified" => uncovered = 1,
            _ => {}
        }
    } else {
        for c in &kids {
            active += c.active;
            planned += c.planned;
            uncovered += c.uncovered;
        }
    }
    Node {
        id: k,
        name: e.frontmatter.name.clone(),
        status: e.frontmatter.status.clone(),
        leaf,
        own,
        active,
        planned,
        uncovered,
        direct,
        children: kids,
    }
}

fn render(n: &Node, depth: usize, out: &mut String) {
    let ind = "  ".repeat(depth);
    let st = n.status.as_deref().unwrap_or("-");
    if n.leaf {
        out.push_str(&format!("{ind}{} {}  {} | direct tests {} [{st}]\n", n.glyph(), n.id, n.own, n.direct));
    } else {
        let total = n.active + n.planned + n.uncovered;
        out.push_str(&format!(
            "{ind}{} {}  leaves {}/{total} active, {} planned | direct tests {} [{st}]\n",
            n.glyph(),
            n.id,
            n.active,
            n.planned,
            n.direct
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
        "own": n.own,
        "verdict": n.glyph().to_string(),
        "leavesActive": n.active,
        "leavesPlanned": n.planned,
        "leavesUncovered": n.uncovered,
        "directTests": n.direct,
        "children": n.children.iter().map(to_json).collect::<Vec<_>>(),
    })
}

/// Entry point. Returns the process exit code.
pub fn cmd_coverage_tree(
    elements: &[RawElement],
    result: &ValidationResult,
    results: Option<&ResultsData>,
    root: &str,
    json_out: bool,
) -> i32 {
    let resolver = Resolver::new(elements);
    let Some(r) = resolver.resolve_ref(elements, root).filter(|e| Resolver::is_native_requirement(e)) else {
        eprintln!("coverage tree: '{root}' does not resolve to a requirement.");
        return 1;
    };
    let states = crate::matrix::requirement_rollup(elements, results);
    let tree = build(r, elements, &resolver, result, &states, &mut HashSet::new());
    if json_out {
        println!("{}", serde_json::to_string_pretty(&to_json(&tree)).unwrap_or_default());
    } else {
        let mut s = String::new();
        render(&tree, 0, &mut s);
        print!("{s}");
    }
    0
}

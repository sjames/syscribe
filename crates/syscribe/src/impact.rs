//! Change impact analysis (§17, GH #65). Read-only: traverses the traceability graph
//! from a named element and reports every reachable node, its hop distance, and the edge
//! kind that connects it. Downstream follows reverse links (who depends on me);
//! upstream follows forward links (what I depend on).

use std::collections::{BTreeMap, HashMap, HashSet};
use syscribe_model::{element::RawElement, resolver::Resolver};

/// One adjacency edge: the neighbour's qname, the base kind (for `--kinds` filtering),
/// and the human `via` label for the chosen direction.
#[derive(Clone)]
struct Adj {
    to: String,
    base: &'static str,
    via: &'static str,
}

#[derive(Clone, Copy)]
pub enum Direction {
    Downstream,
    Upstream,
    Both,
}

/// Forward (upstream) traceability fields of an element: (qnames it points to, base kind,
/// upstream `via`, downstream `via`).
fn upstream_targets<'a>(
    e: &'a RawElement,
    resolver: &Resolver,
    elements: &'a [RawElement],
) -> Vec<(String, &'static str, &'static str, &'static str)> {
    let fm = &e.frontmatter;
    let mut out: Vec<(String, &'static str, &'static str, &'static str)> = Vec::new();
    let add = |refs: &[String], base: &'static str, up: &'static str, down: &'static str, out: &mut Vec<_>| {
        for r in refs {
            if let Some(t) = resolver.resolve_ref(elements, r) {
                out.push((t.qualified_name.clone(), base, up, down));
            }
        }
    };
    // supertype (string or list)
    let supers: Vec<String> = match &fm.supertype {
        Some(serde_yaml::Value::String(s)) => vec![s.clone()],
        Some(serde_yaml::Value::Sequence(seq)) => seq.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
        _ => Vec::new(),
    };
    add(&supers, "supertype", "supertype", "specializedBy", &mut out);
    add(fm.derived_from.as_deref().unwrap_or(&[]), "derivedFrom", "derivedFrom", "derivedChildren", &mut out);
    add(fm.verifies.as_deref().unwrap_or(&[]), "verifies", "verifies", "verifiedBy", &mut out);
    add(fm.satisfies.as_deref().unwrap_or(&[]), "satisfies", "satisfies", "satisfiedBy", &mut out);
    add(fm.refines.as_deref().unwrap_or(&[]), "refines", "refines", "refinedBy", &mut out);
    add(fm.allocated_to.as_deref().unwrap_or(&[]), "allocatedTo", "allocatedTo", "allocatedFrom", &mut out);
    if let Some(sg) = &fm.derived_from_safety_goal {
        add(std::slice::from_ref(sg), "derivedFromSafetyGoal", "derivedFromSafetyGoal", "safetyGoalChildren", &mut out);
    }
    // appliesWhen: feature references in the (opaque) expression — best-effort token resolve.
    if let Some(expr) = applies_when_str(&fm.applies_when) {
        for tok in expr.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':' || c == '-')) {
            if tok.len() < 2 {
                continue;
            }
            if let Some(t) = resolver.resolve_ref(elements, tok) {
                if matches!(t.frontmatter.element_type, Some(syscribe_model::element::ElementType::FeatureDef)) {
                    out.push((t.qualified_name.clone(), "appliesWhen", "appliesWhen", "conditionalOn"));
                }
            }
        }
    }
    out
}

fn applies_when_str(v: &Option<serde_yaml::Value>) -> Option<String> {
    match v {
        Some(serde_yaml::Value::String(s)) => Some(s.clone()),
        Some(serde_yaml::Value::Sequence(seq)) => {
            Some(seq.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(" "))
        }
        _ => None,
    }
}

/// Build the up/down adjacency over all elements in one pass.
fn build_adjacency(
    elements: &[RawElement],
    resolver: &Resolver,
) -> (HashMap<String, Vec<Adj>>, HashMap<String, Vec<Adj>>) {
    let mut up: HashMap<String, Vec<Adj>> = HashMap::new();
    let mut down: HashMap<String, Vec<Adj>> = HashMap::new();
    for e in elements {
        let q = &e.qualified_name;
        for (target, base, up_via, down_via) in upstream_targets(e, resolver, elements) {
            up.entry(q.clone()).or_default().push(Adj { to: target.clone(), base, via: up_via });
            down.entry(target).or_default().push(Adj { to: q.clone(), base, via: down_via });
        }
    }
    (up, down)
}

pub struct ImpactOptions<'a> {
    pub root: &'a str,
    pub direction: Direction,
    pub depth: Option<usize>,
    pub format: &'a str,
    pub kinds: Option<Vec<String>>,
}

struct Node {
    qname: String,
    depth: usize,
    via: &'static str,
    parent: Option<String>,
}

pub fn cmd_impact(elements: &[RawElement], opts: &ImpactOptions) {
    let resolver = Resolver::new(elements);
    let root_el = match resolver.resolve_ref(elements, opts.root) {
        Some(e) => e,
        None => {
            eprintln!("impact: '{}' does not resolve to a known element.", opts.root);
            std::process::exit(1);
        }
    };
    let root_q = root_el.qualified_name.clone();
    let (up, down) = build_adjacency(elements, &resolver);

    let kind_ok = |base: &str| opts.kinds.as_ref().is_none_or(|ks| ks.iter().any(|k| k == base));

    // BFS spanning tree from the root in the chosen direction(s); cycle-safe via `seen`.
    let mut seen: HashSet<String> = HashSet::from([root_q.clone()]);
    let mut order: Vec<Node> = Vec::new();
    let mut queue: std::collections::VecDeque<(String, usize)> = std::collections::VecDeque::from([(root_q.clone(), 0usize)]);
    while let Some((cur, d)) = queue.pop_front() {
        if let Some(max) = opts.depth {
            if d >= max {
                continue;
            }
        }
        let mut neighbours: Vec<&Adj> = Vec::new();
        if matches!(opts.direction, Direction::Downstream | Direction::Both) {
            neighbours.extend(down.get(&cur).into_iter().flatten());
        }
        if matches!(opts.direction, Direction::Upstream | Direction::Both) {
            neighbours.extend(up.get(&cur).into_iter().flatten());
        }
        // Stable order for reproducible output.
        let mut sorted: Vec<&Adj> = neighbours.into_iter().filter(|a| kind_ok(a.base)).collect();
        sorted.sort_by(|a, b| a.to.cmp(&b.to));
        for a in sorted {
            if seen.insert(a.to.clone()) {
                order.push(Node { qname: a.to.clone(), depth: d + 1, via: a.via, parent: Some(cur.clone()) });
                queue.push_back((a.to.clone(), d + 1));
            }
        }
    }

    let type_of = |q: &str| -> String {
        resolver
            .resolve_ref(elements, q)
            .and_then(|e| e.frontmatter.element_type.as_ref().map(|t| format!("{:?}", t)))
            .unwrap_or_else(|| "?".into())
    };
    let id_of = |q: &str| -> Option<String> { resolver.resolve_ref(elements, q).and_then(|e| e.frontmatter.id.clone()) };
    let status_of = |q: &str| -> Option<String> { resolver.resolve_ref(elements, q).and_then(|e| e.frontmatter.status.clone()) };
    let disp = |q: &str| -> String { id_of(q).unwrap_or_else(|| q.to_string()) };

    match opts.format {
        "json" => {
            let nodes: Vec<serde_json::Value> = order
                .iter()
                .map(|n| {
                    let mut o = serde_json::json!({ "qname": n.qname, "type": type_of(&n.qname), "depth": n.depth, "via": n.via });
                    if let Some(id) = id_of(&n.qname) {
                        o["id"] = serde_json::json!(id);
                    }
                    o
                })
                .collect();
            let mut root = serde_json::json!({ "qname": root_q, "type": type_of(&root_q) });
            if let Some(id) = id_of(&root_q) {
                root["id"] = serde_json::json!(id);
            }
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "root": root, "nodes": nodes })).unwrap());
        }
        "dot" => {
            println!("digraph impact {{");
            println!("  rankdir=LR; node [shape=box, style=rounded];");
            println!("  \"{}\" [style=\"rounded,filled\", fillcolor=\"#cde\"];", root_q);
            for n in &order {
                if let Some(p) = &n.parent {
                    println!("  \"{}\" -> \"{}\" [label=\"{}\"];", p, n.qname, n.via);
                }
            }
            println!("}}");
        }
        _ => {
            let dirname = match opts.direction {
                Direction::Downstream => "downstream",
                Direction::Upstream => "upstream",
                Direction::Both => "both",
            };
            let depthname = opts.depth.map(|d| format!("depth {}", d)).unwrap_or_else(|| "unlimited depth".into());
            println!("Impact of {} ({}, {})\n", disp(&root_q), dirname, depthname);
            let st = status_of(&root_q).map(|s| format!(", {}", s)).unwrap_or_default();
            println!("{} [{}{}]", disp(&root_q), type_of(&root_q), st);
            // Children grouped by parent for a simple indented tree.
            let mut children: BTreeMap<String, Vec<&Node>> = BTreeMap::new();
            for n in &order {
                if let Some(p) = &n.parent {
                    children.entry(p.clone()).or_default().push(n);
                }
            }
            fn walk(
                q: &str,
                indent: &str,
                children: &BTreeMap<String, Vec<&Node>>,
                disp: &dyn Fn(&str) -> String,
                type_of: &dyn Fn(&str) -> String,
                status_of: &dyn Fn(&str) -> Option<String>,
            ) {
                if let Some(kids) = children.get(q) {
                    let n = kids.len();
                    for (i, k) in kids.iter().enumerate() {
                        let last = i + 1 == n;
                        let conn = if last { "└──" } else { "├──" };
                        let st = status_of(&k.qname).map(|s| format!(", {}", s)).unwrap_or_default();
                        println!("{}{} {} [{}{}] via {}", indent, conn, disp(&k.qname), type_of(&k.qname), st, k.via);
                        let next = format!("{}{}   ", indent, if last { " " } else { "│" });
                        walk(&k.qname, &next, children, disp, type_of, status_of);
                    }
                }
            }
            walk(&root_q, "", &children, &disp, &type_of, &status_of);
            println!("\n{} element(s) reachable", order.len());
        }
    }
}

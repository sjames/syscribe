//! N² (N-squared) interface matrix (§16, GH #64). Read-only: derives a square
//! interface matrix from the model's existing connection / flow / allocation edges.
//! Axis cell (row R, col C) lists the interfaces R sends to C.

use std::collections::{BTreeMap, HashSet};
use syscribe_model::{
    element::{ElementType, RawElement},
    resolver::Resolver,
    validator::allocation_edges,
};

fn is_part(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::PartDef) | Some(ElementType::Part))
}

fn disp(e: &RawElement) -> String {
    e.frontmatter
        .name
        .clone()
        .unwrap_or_else(|| e.qualified_name.rsplit("::").next().unwrap_or(&e.qualified_name).to_string())
}

fn sval<'a>(m: &'a serde_yaml::Mapping, k: &str) -> Option<&'a serde_yaml::Value> {
    m.get(serde_yaml::Value::String(k.to_string()))
}

/// Resolve a connection endpoint feature chain (e.g. `power.mainPowerOut`) to the axis
/// element it belongs to: the head segment is looked up in the connection-owner's
/// `features:` and resolved through `typedBy:`; otherwise the whole chain is resolved.
fn resolve_endpoint<'a>(
    owner: &RawElement,
    chain: &str,
    elements: &'a [RawElement],
    resolver: &Resolver,
) -> Option<&'a RawElement> {
    let head = chain.split('.').next().unwrap_or(chain);
    if let Some(feats) = &owner.frontmatter.features {
        for f in feats {
            if let serde_yaml::Value::Mapping(m) = f {
                if sval(m, "name").and_then(|v| v.as_str()) == Some(head) {
                    if let Some(tb) = sval(m, "typedBy").and_then(|v| v.as_str()) {
                        return resolver.resolve_ref(elements, tb);
                    }
                }
            }
        }
    }
    resolver.resolve_ref(elements, chain)
}

/// Last `::` segment of a `typedBy:` value — the interface (ConnectionDef/InterfaceDef) name.
fn last_seg(s: &str) -> &str {
    s.rsplit("::").next().unwrap_or(s)
}

#[derive(Clone)]
struct Edge {
    kind: &'static str,
    name: String,
}

/// Collect the subpart-type elements of `scope` recursively to `depth` (features typed
/// by a Part/PartDef), de-duplicated by qualified name.
fn subpart_axis<'a>(
    scope: &'a RawElement,
    depth: usize,
    elements: &'a [RawElement],
    resolver: &Resolver,
) -> Vec<&'a RawElement> {
    let mut out: Vec<&RawElement> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut frontier: Vec<&RawElement> = vec![scope];
    for _ in 0..depth {
        let mut next: Vec<&RawElement> = Vec::new();
        for e in &frontier {
            for f in e.frontmatter.features.iter().flatten() {
                let serde_yaml::Value::Mapping(m) = f else { continue };
                let Some(tb) = sval(m, "typedBy").and_then(|v| v.as_str()) else { continue };
                if let Some(t) = resolver.resolve_ref(elements, tb) {
                    if is_part(t) && seen.insert(t.qualified_name.clone()) {
                        out.push(t);
                        next.push(t);
                    }
                }
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }
    out
}

/// Extract every directed, labeled interface edge (src-qname → dst-qname) from the model's
/// connections and flows, restricted to endpoints in `axis`.
fn collect_edges(
    elements: &[RawElement],
    resolver: &Resolver,
    axis: &HashSet<String>,
    with_allocations: bool,
) -> BTreeMap<(String, String), Vec<Edge>> {
    let mut edges: BTreeMap<(String, String), Vec<Edge>> = BTreeMap::new();
    let push = |s: &str, d: &str, e: Edge, edges: &mut BTreeMap<(String, String), Vec<Edge>>| {
        if s != d && axis.contains(s) && axis.contains(d) {
            edges.entry((s.to_string(), d.to_string())).or_default().push(e);
        }
    };

    for owner in elements {
        for (list, kind) in [
            (&owner.frontmatter.connections, "Connection"),
            (&owner.frontmatter.flow_connections, "Flow"),
        ] {
            let Some(entries) = list else { continue };
            for entry in entries {
                let serde_yaml::Value::Mapping(m) = entry else { continue };
                // Label: typedBy (last segment) → name → kind.
                let label = sval(m, "typedBy")
                    .and_then(|v| v.as_str())
                    .map(last_seg)
                    .or_else(|| sval(m, "name").and_then(|v| v.as_str()))
                    .unwrap_or(kind)
                    .to_string();
                // Ordered endpoints: from/to or left/right or ends[].binds (source→receiver).
                let mut src_chain: Option<String> = None;
                let mut dst_chain: Option<String> = None;
                for k in ["from", "left"] {
                    if let Some(s) = sval(m, k).and_then(|v| v.as_str()) {
                        src_chain = Some(s.to_string());
                    }
                }
                for k in ["to", "right"] {
                    if let Some(s) = sval(m, k).and_then(|v| v.as_str()) {
                        dst_chain = Some(s.to_string());
                    }
                }
                if let Some(serde_yaml::Value::Sequence(seq)) = sval(m, "ends") {
                    let mut binds: Vec<(Option<String>, String)> = Vec::new();
                    for e in seq {
                        if let serde_yaml::Value::Mapping(em) = e {
                            if let Some(b) = sval(em, "binds").and_then(|v| v.as_str()) {
                                let role = sval(em, "end").and_then(|v| v.as_str()).map(String::from);
                                binds.push((role, b.to_string()));
                            }
                        }
                    }
                    // source/receiver roles, else first→rest.
                    if let Some((_, s)) = binds.iter().find(|(r, _)| r.as_deref() == Some("source")) {
                        src_chain = Some(s.clone());
                    }
                    if let Some((_, d)) = binds.iter().find(|(r, _)| matches!(r.as_deref(), Some("receiver") | Some("target") | Some("destination"))) {
                        dst_chain = Some(d.clone());
                    }
                    if src_chain.is_none() {
                        src_chain = binds.first().map(|(_, b)| b.clone());
                    }
                    if dst_chain.is_none() {
                        dst_chain = binds.get(1).map(|(_, b)| b.clone());
                    }
                }
                if let (Some(sc), Some(dc)) = (&src_chain, &dst_chain) {
                    let s = resolve_endpoint(owner, sc, elements, resolver);
                    let d = resolve_endpoint(owner, dc, elements, resolver);
                    if let (Some(s), Some(d)) = (s, d) {
                        push(&s.qualified_name, &d.qualified_name, Edge { kind, name: label.clone() }, &mut edges);
                    }
                }
            }
        }
    }

    if with_allocations {
        for (s, d) in allocation_edges(elements, resolver) {
            let (sq, dq) = (
                resolver.resolve_ref(elements, &s).map(|e| e.qualified_name.clone()),
                resolver.resolve_ref(elements, &d).map(|e| e.qualified_name.clone()),
            );
            if let (Some(sq), Some(dq)) = (sq, dq) {
                push(&sq, &dq, Edge { kind: "Allocation", name: "allocatedTo".into() }, &mut edges);
            }
        }
    }
    edges
}

pub struct N2Options<'a> {
    pub scope: Option<&'a str>,
    pub depth: usize,
    pub format: &'a str,
    pub interfaces_only: bool,
    pub allocations: bool,
}

pub fn cmd_n2(elements: &[RawElement], opts: &N2Options) {
    let resolver = Resolver::new(elements);

    // Axis elements.
    let scope_label;
    let mut axis: Vec<&RawElement> = match opts.scope {
        None => {
            scope_label = "<model>".to_string();
            elements.iter().filter(|e| is_part(e)).collect()
        }
        Some(q) => match resolver.resolve_ref(elements, q) {
            Some(s) => {
                scope_label = s.qualified_name.clone();
                subpart_axis(s, opts.depth.max(1), elements, &resolver)
            }
            None => {
                eprintln!("n2: scope '{}' does not resolve to a known element.", q);
                std::process::exit(1);
            }
        },
    };
    axis.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
    axis.dedup_by(|a, b| a.qualified_name == b.qualified_name);

    let axis_set: HashSet<String> = axis.iter().map(|e| e.qualified_name.clone()).collect();
    let edges = collect_edges(elements, &resolver, &axis_set, opts.allocations);

    // --interfaces-only: drop axis elements with no incoming/outgoing edge.
    if opts.interfaces_only {
        let active: HashSet<String> = edges
            .keys()
            .flat_map(|(s, d)| [s.clone(), d.clone()])
            .collect();
        axis.retain(|e| active.contains(&e.qualified_name));
    }

    match opts.format {
        "json" => render_json(&scope_label, &axis, &edges),
        "html" => render_html(&scope_label, &axis, &edges, opts.depth),
        _ => render_text(&scope_label, &axis, &edges, opts.depth),
    }
}

fn cell_text(edges: &BTreeMap<(String, String), Vec<Edge>>, r: &str, c: &str) -> String {
    match edges.get(&(r.to_string(), c.to_string())) {
        None => "—".to_string(),
        Some(es) if es.len() <= 2 => es.iter().map(|e| e.name.clone()).collect::<Vec<_>>().join(", "),
        Some(es) => format!("{} ifaces", es.len()),
    }
}

fn render_text(scope: &str, axis: &[&RawElement], edges: &BTreeMap<(String, String), Vec<Edge>>, depth: usize) {
    println!("N² Interface Matrix — {} (depth {})\n", scope, depth);
    if axis.is_empty() {
        println!("(no parts in scope)");
        return;
    }
    let names: Vec<String> = axis.iter().map(|e| disp(e)).collect();
    let label_w = names.iter().map(|n| n.len()).max().unwrap_or(8).max(8);
    let col_w = names
        .iter()
        .enumerate()
        .map(|(j, n)| {
            let cell_max = axis
                .iter()
                .map(|r| cell_text(edges, &r.qualified_name, &axis[j].qualified_name).len())
                .max()
                .unwrap_or(0);
            n.len().max(cell_max)
        })
        .collect::<Vec<_>>();

    // Header
    print!("{:width$}", "", width = label_w + 2);
    for (j, n) in names.iter().enumerate() {
        print!("  {:width$}", n, width = col_w[j]);
    }
    println!();
    // Rows
    for (i, r) in axis.iter().enumerate() {
        print!("{:width$}", names[i], width = label_w + 2);
        for (j, c) in axis.iter().enumerate() {
            let cell = if i == j {
                "■".to_string()
            } else {
                cell_text(edges, &r.qualified_name, &c.qualified_name)
            };
            print!("  {:width$}", cell, width = col_w[j]);
        }
        println!();
    }
}

fn render_json(scope: &str, axis: &[&RawElement], edges: &BTreeMap<(String, String), Vec<Edge>>) {
    let qn_to_name: BTreeMap<&str, String> = axis.iter().map(|e| (e.qualified_name.as_str(), disp(e))).collect();
    let mut matrix: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for ((r, c), es) in edges {
        let (Some(rn), Some(cn)) = (qn_to_name.get(r.as_str()), qn_to_name.get(c.as_str())) else { continue };
        let arr: Vec<serde_json::Value> = es
            .iter()
            .map(|e| serde_json::json!({ "kind": e.kind, "name": e.name }))
            .collect();
        matrix
            .entry(rn.clone())
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .unwrap()
            .insert(cn.clone(), serde_json::Value::Array(arr));
    }
    let elements: Vec<String> = axis.iter().map(|e| disp(e)).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "scope": scope, "elements": elements, "matrix": matrix
        }))
        .unwrap()
    );
}

fn render_html(scope: &str, axis: &[&RawElement], edges: &BTreeMap<(String, String), Vec<Edge>>, depth: usize) {
    let names: Vec<String> = axis.iter().map(|e| disp(e)).collect();
    println!("<table class=\"n2-matrix\" style=\"border-collapse:collapse\">");
    println!("<caption>N² Interface Matrix — {} (depth {})</caption>", html_escape(scope), depth);
    print!("<thead><tr><th></th>");
    for n in &names {
        print!("<th style=\"border:1px solid #ccc;padding:4px\">{}</th>", html_escape(n));
    }
    println!("</tr></thead><tbody>");
    for (i, r) in axis.iter().enumerate() {
        print!("<tr><th style=\"border:1px solid #ccc;padding:4px\">{}</th>", html_escape(&names[i]));
        for (j, c) in axis.iter().enumerate() {
            if i == j {
                print!("<td style=\"border:1px solid #ccc;padding:4px;background:#333\"></td>");
            } else {
                let cell = match edges.get(&(r.qualified_name.clone(), c.qualified_name.clone())) {
                    None => String::new(),
                    Some(es) => es.iter().map(|e| html_escape(&e.name)).collect::<Vec<_>>().join("<br>"),
                };
                print!("<td style=\"border:1px solid #ccc;padding:4px\">{}</td>", cell);
            }
        }
        println!("</tr>");
    }
    println!("</tbody></table>");
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

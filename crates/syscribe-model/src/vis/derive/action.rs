//! The Action generator (`REQ-TRS-VIS-019`).
//!
//! Subject: an `ActionDef`, or an `Action` usage (which reads the `ActionDef`
//! it is typed by). Produces one `Action` node per `subActions:` entry (spec
//! §8.7.3) stereotyped by its kind (`action`, `perform`, `send`, `accept`,
//! `assign`, `terminate`) with a compartment for its `typedBy`, `payload` and
//! `via`/`to` chains; an `IfAction` as a `Decision` labelled by its condition
//! whose `then`/`else` branches are nodes joined by `[then]`/`[else]`
//! successions and closed by a `Merge`; a `LoopAction` as a container `Action`
//! stereotyped `loop` holding its `body` in order; one `Fork`/`Join`/
//! `Decision`/`Merge` per `controlNodes:` entry; a `Succession` edge per
//! `successionConnections:` entry (labelled by its `guard`) and a `Flow` edge
//! per `flowConnections:` entry; and, only when the subject declares at least
//! one succession, an `Initial` node feeding every step with no incoming
//! succession and a `Final` node reached from every step with no outgoing one.
//! Shape ids are `derived_shape_id("<subject>::<step>")`, nested branch/body
//! names included in the path.

use std::collections::HashMap;

use crate::connections::parse_entry;
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, Node, NodeKind};
use super::super::manifest::Issue;
use super::{map_str, short_name, w418, yaml_strings, Filters};

/// Where successions enter and leave a step: the same node for a plain
/// action, the decision and the merge for an `IfAction`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Step {
    name: String,
    entry: String,
    exit: String,
}

fn node(id: String, qname: &str, kind: NodeKind, label: String, stereotype: Option<&str>, parent: Option<&str>) -> Node {
    Node {
        id,
        element_ref: qname.to_string(),
        resolved: true,
        element_type: None,
        kind,
        label,
        stereotype: stereotype.map(str::to_string),
        parent: parent.map(str::to_string),
        direction: None,
        side: None,
        lines: Vec::new(),
        is_abstract: false,
        pin: None,
        banners: Vec::new(),
        feature: None,
    }
}

fn compartment(owner_id: &str, qname: &str, lines: Vec<String>) -> Node {
    let mut c = node(format!("{owner_id}-compartment"), qname, NodeKind::Compartment, String::new(), None, Some(owner_id));
    c.lines = lines;
    c
}

fn non_empty(s: Option<&str>) -> Option<String> {
    s.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

/// The control-node kind named by a `kind:` value, if any.
fn control_kind(kind: &str) -> Option<NodeKind> {
    Some(match kind {
        "ForkNode" | "Fork" => NodeKind::Fork,
        "JoinNode" | "Join" => NodeKind::Join,
        "DecisionNode" | "Decision" => NodeKind::Decision,
        "MergeNode" | "Merge" => NodeKind::Merge,
        _ => return None,
    })
}

/// The stereotype of a step by its `kind:`.
fn step_stereotype(kind: &str) -> &'static str {
    match kind {
        "PerformAction" => "perform",
        "SendAction" => "send",
        "AcceptAction" => "accept",
        "AssignmentAction" => "assign",
        "TerminateAction" => "terminate",
        _ => "action",
    }
}

/// The compartment lines of a plain step: its `typedBy`, `payload`,
/// `via`/`to` chains, an accept trigger, and an assignment.
fn step_lines(m: &serde_yaml::Mapping, kind: &str) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(t) = yaml_strings(m.get(serde_yaml::Value::String("typedBy".into()))).into_iter().next() {
        lines.push(format!(": {}", short_name(&t)));
    }
    if let Some(p) = non_empty(map_str(m, "payload")) {
        let word = match kind {
            "SendAction" => "send",
            "AcceptAction" => "accept",
            _ => "payload",
        };
        lines.push(format!("{word} {}", short_name(&p)));
    }
    if let Some(serde_yaml::Value::Mapping(t)) = m.get(serde_yaml::Value::String("trigger".into())) {
        match map_str(t, "kind") {
            Some("timeOut") => lines.extend(non_empty(map_str(t, "when")).map(|w| format!("after {w}"))),
            Some("at") => lines.extend(non_empty(map_str(t, "when")).map(|w| format!("at {w}"))),
            Some("change") => lines.extend(non_empty(map_str(t, "condition")).map(|c| format!("when {c}"))),
            _ => {}
        }
    }
    if let Some(v) = non_empty(map_str(m, "via")) {
        lines.push(format!("via {v}"));
    }
    if let Some(t) = non_empty(map_str(m, "to")) {
        lines.push(format!("to {t}"));
    }
    if kind == "AssignmentAction" {
        if let (Some(target), Some(referent), Some(value)) = (non_empty(map_str(m, "target")), non_empty(map_str(m, "referent")), non_empty(map_str(m, "value"))) {
            lines.push(format!("{target}.{referent} := {value}"));
        }
    }
    lines
}

/// The label of a loop container: its name and condition.
fn loop_label(name: &str, m: &serde_yaml::Mapping) -> String {
    let kind = map_str(m, "loopKind").unwrap_or("while");
    let clause = match kind {
        "for" => match (non_empty(map_str(m, "variable")), non_empty(map_str(m, "sequence"))) {
            (Some(v), Some(s)) => Some(format!("for {v} in {s}")),
            (Some(v), None) => Some(format!("for {v}")),
            _ => None,
        },
        _ => non_empty(map_str(m, "condition")).map(|c| format!("{kind} {c}")),
    };
    match clause {
        Some(c) => format!("{name} [{c}]"),
        None => name.to_string(),
    }
}

fn edge(id: String, element_ref: &str, source: &str, target: &str, kind: EdgeKind, label: Option<String>) -> Edge {
    Edge { id, element_ref: Some(element_ref.to_string()), source: source.to_string(), target: target.to_string(), kind, label, waypoints: None }
}

/// Successions `a.exit → b.entry` along `steps` in order.
fn chain(graph: &mut DiagramGraph, owner: &str, steps: &[Step]) {
    for w in steps.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        graph.edges.push(edge(format!("e-succession-{}-{}", a.exit, b.entry), owner, &a.exit, &b.entry, EdgeKind::Succession, None));
    }
}

#[derive(Default)]
struct Builder {
    /// Dotted step paths (`check.abort`) and top-level names → entry/exit ids.
    by_path: HashMap<String, Step>,
}

impl Builder {
    /// Emit the steps of `list` under `path` (qualified-name prefix) and
    /// `dotted` (chain prefix), nested in `parent`; returns them in order.
    fn emit_steps(&mut self, graph: &mut DiagramGraph, list: &[serde_yaml::Value], path: &str, dotted: &str, parent: Option<&str>, keep: &dyn Fn(&str) -> bool) -> Vec<Step> {
        let mut steps = Vec::new();
        for v in list {
            let Some(m) = v.as_mapping() else { continue };
            let Some(name) = non_empty(map_str(m, "name")) else { continue };
            if !keep(&name) {
                continue;
            }
            let qn = format!("{path}::{name}");
            let id = derived_shape_id(&qn);
            let chain_name = if dotted.is_empty() { name.clone() } else { format!("{dotted}.{name}") };
            let kind = map_str(m, "kind").unwrap_or("Action");
            let step = if let Some(ck) = control_kind(kind) {
                graph.nodes.push(node(id.clone(), &qn, ck, name.clone(), None, parent));
                Step { name: name.clone(), entry: id.clone(), exit: id }
            } else if kind == "IfAction" {
                let label = non_empty(map_str(m, "condition")).unwrap_or_else(|| name.clone());
                graph.nodes.push(node(id.clone(), &qn, NodeKind::Decision, label, None, parent));
                let merge_id = derived_shape_id(&format!("{qn}::merge"));
                for (branch, word) in [("then", "[then]"), ("else", "[else]")] {
                    let list: &[serde_yaml::Value] = match m.get(serde_yaml::Value::String(branch.into())) {
                        Some(serde_yaml::Value::Sequence(s)) => s,
                        _ => &[],
                    };
                    let inner = self.emit_steps(graph, list, &qn, &chain_name, parent, &|_| true);
                    match (inner.first(), inner.last()) {
                        (Some(first), Some(last)) => {
                            graph.edges.push(edge(format!("e-succession-{id}-{}", first.entry), &qn, &id, &first.entry, EdgeKind::Succession, Some(word.into())));
                            chain(graph, &qn, &inner);
                            graph.edges.push(edge(format!("e-succession-{}-{merge_id}", last.exit), &qn, &last.exit, &merge_id, EdgeKind::Succession, None));
                        }
                        _ => graph.edges.push(edge(format!("e-succession-{id}-{merge_id}-{branch}"), &qn, &id, &merge_id, EdgeKind::Succession, Some(word.into()))),
                    }
                }
                graph.nodes.push(node(merge_id.clone(), &qn, NodeKind::Merge, String::new(), None, parent));
                Step { name: name.clone(), entry: id, exit: merge_id }
            } else if kind == "LoopAction" {
                graph.nodes.push(node(id.clone(), &qn, NodeKind::Action, loop_label(&name, m), Some("loop"), parent));
                let body: &[serde_yaml::Value] = match m.get(serde_yaml::Value::String("body".into())) {
                    Some(serde_yaml::Value::Sequence(s)) => s,
                    _ => &[],
                };
                let inner = self.emit_steps(graph, body, &qn, &chain_name, Some(&id), &|_| true);
                chain(graph, &qn, &inner);
                Step { name: name.clone(), entry: id.clone(), exit: id }
            } else {
                graph.nodes.push(node(id.clone(), &qn, NodeKind::Action, name.clone(), Some(step_stereotype(kind)), parent));
                let lines = step_lines(m, kind);
                if !lines.is_empty() {
                    graph.nodes.push(compartment(&id, &qn, lines));
                }
                Step { name: name.clone(), entry: id.clone(), exit: id }
            };
            self.by_path.insert(chain_name, step.clone());
            steps.push(step);
        }
        steps
    }

    /// The step an endpoint chain names: the full dotted path, else its first
    /// segment (a pin of a top-level step).
    fn step_of(&self, chain: &str) -> Option<&Step> {
        let chain = chain.trim();
        self.by_path.get(chain).or_else(|| self.by_path.get(chain.split('.').next().unwrap_or(chain)))
    }
}

pub fn generate(
    graph: &mut DiagramGraph,
    subject: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    filters: &Filters,
    issues: &mut Vec<Issue>,
) {
    let Some(st) = subject.frontmatter.element_type.as_ref() else { return };
    let def: &RawElement = match st {
        ElementType::ActionDef => subject,
        ElementType::Action => {
            let typed = yaml_strings(subject.frontmatter.typed_by.as_ref()).into_iter().next();
            match typed.as_deref().and_then(|t| resolver.resolve_ref(elements, t)) {
                Some(d) if matches!(d.frontmatter.element_type, Some(ElementType::ActionDef)) => d,
                _ => {
                    issues.push(w418(format!(
                        "`subject` '{}' is an Action whose `typedBy` does not resolve to an ActionDef",
                        subject.qualified_name
                    )));
                    return;
                }
            }
        }
        _ => {
            issues.push(w418(format!(
                "`subject` '{}' is a {} — an Action diagram subject must be an ActionDef or Action",
                subject.qualified_name,
                st.name()
            )));
            return;
        }
    };
    let sq = subject.qualified_name.as_str();
    let subject_id = derived_shape_id(sq);
    let fm = &def.frontmatter;
    let sub_actions: &[serde_yaml::Value] = fm.sub_actions.as_deref().unwrap_or(&[]);
    let control_nodes: &[serde_yaml::Value] = fm.control_nodes.as_deref().unwrap_or(&[]);

    let candidates: Vec<(String, String)> = sub_actions
        .iter()
        .chain(control_nodes.iter())
        .filter_map(|v| v.as_mapping().and_then(|m| map_str(m, "name")))
        .map(|n| (format!("{sq}::{n}"), n.to_string()))
        .collect();
    filters.unmatched_issues(&candidates, issues);
    let keep = |name: &str| filters.keeps(&format!("{sq}::{name}"), name);

    let mut b = Builder::default();
    let mut top: Vec<Step> = b.emit_steps(graph, sub_actions, sq, "", None, &keep);
    // Control nodes declared under `controlNodes:`.
    for v in control_nodes {
        let Some(m) = v.as_mapping() else { continue };
        let (Some(name), Some(kind)) = (non_empty(map_str(m, "name")), map_str(m, "kind").and_then(control_kind)) else { continue };
        if !keep(&name) {
            continue;
        }
        let qn = format!("{sq}::{name}");
        let id = derived_shape_id(&qn);
        if graph.node(&id).is_some() {
            continue;
        }
        graph.nodes.push(node(id.clone(), &qn, kind, name.clone(), None, None));
        let step = Step { name: name.clone(), entry: id.clone(), exit: id };
        b.by_path.insert(name, step.clone());
        top.push(step);
    }

    // ── successions and flows ───────────────────────────────────────────
    let mut counters: HashMap<String, usize> = HashMap::new();
    let mut next_id = |base: String| -> String {
        let n = counters.entry(base.clone()).or_insert(0);
        *n += 1;
        if *n == 1 {
            base
        } else {
            format!("{base}-{n}")
        }
    };
    let successions: &[serde_yaml::Value] = fm.succession_connections.as_deref().unwrap_or(&[]);
    let mut declared: Vec<Edge> = Vec::new();
    for entry in successions {
        let Some(m) = entry.as_mapping() else { continue };
        let (Some(after), Some(before)) = (map_str(m, "after"), map_str(m, "before")) else { continue };
        let (Some(a), Some(bf)) = (b.step_of(after).cloned(), b.step_of(before).cloned()) else { continue };
        let id = next_id(format!("e-succession-{}-{}", a.exit, bf.entry));
        declared.push(edge(id, sq, &a.exit, &bf.entry, EdgeKind::Succession, non_empty(map_str(m, "guard")).map(|g| format!("[{g}]"))));
    }
    for entry in fm.flow_connections.as_deref().unwrap_or(&[]) {
        let Some(p) = parse_entry(entry) else { continue };
        if p.endpoints.len() < 2 {
            continue;
        }
        let (Some(a), Some(bf)) = (b.step_of(&p.endpoints[0].chain).cloned(), b.step_of(&p.endpoints[1].chain).cloned()) else { continue };
        let label = entry
            .as_mapping()
            .and_then(|m| non_empty(map_str(m, "name")))
            .or_else(|| p.typed_by.as_deref().map(|t| short_name(t).to_string()));
        let id = next_id(format!("e-flow-{}-{}", a.exit, bf.entry));
        declared.push(edge(id, sq, &a.exit, &bf.entry, EdgeKind::Flow, label));
    }

    // ── initial and final, only when the flow is ordered at all ─────────
    if !successions.is_empty() {
        let has_in = |s: &Step| declared.iter().any(|e| e.kind == EdgeKind::Succession && e.target == s.entry);
        let has_out = |s: &Step| declared.iter().any(|e| e.kind == EdgeKind::Succession && e.source == s.exit);
        let initial_id = format!("{subject_id}-initial");
        let final_id = format!("{subject_id}-final");
        let starts: Vec<&Step> = top.iter().filter(|s| !has_in(s)).collect();
        let ends: Vec<&Step> = top.iter().filter(|s| !has_out(s)).collect();
        if !starts.is_empty() {
            graph.nodes.insert(0, node(initial_id.clone(), sq, NodeKind::Initial, String::new(), None, None));
            for s in starts {
                graph.edges.push(edge(format!("e-succession-{initial_id}-{}", s.entry), sq, &initial_id, &s.entry, EdgeKind::Succession, None));
            }
        }
        graph.edges.extend(declared);
        if !ends.is_empty() {
            graph.nodes.push(node(final_id.clone(), sq, NodeKind::Final, String::new(), None, None));
            for s in ends {
                graph.edges.push(edge(format!("e-succession-{}-{final_id}", s.exit), sq, &s.exit, &final_id, EdgeKind::Succession, None));
            }
        }
    } else {
        graph.edges.extend(declared);
    }
}

#[cfg(test)]
mod tests {
    use super::super::testkit::*;
    use super::*;

    fn edges_of(g: &DiagramGraph) -> Vec<(EdgeKind, String, String, Option<String>)> {
        g.edges.iter().map(|e| (e.kind, e.source.clone(), e.target.clone(), e.label.clone())).collect()
    }

    #[test]
    fn actiondef_subject_yields_steps_control_nodes_branches_loop_and_successions() {
        let d = diagram("Action", "Sys::Flight", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        // Steps with their stereotypes and compartments.
        let takeoff = g.node("s-sys-flight-takeoff").unwrap();
        assert_eq!((takeoff.kind, takeoff.stereotype.as_deref(), takeoff.label.as_str()), (NodeKind::Action, Some("perform"), "takeoff"));
        assert_eq!(g.node("s-sys-flight-takeoff-compartment").unwrap().lines, vec![": Startup"]);
        assert_eq!(g.node("s-sys-flight-land").unwrap().stereotype.as_deref(), Some("perform"));
        // The if: a decision labelled by the condition, branch nodes in the path, [then]/[else]
        // successions, and a merge both branches reach.
        let check = g.node("s-sys-flight-check").unwrap();
        assert_eq!((check.kind, check.label.as_str()), (NodeKind::Decision, "wind > 12"));
        let abort = g.node("s-sys-flight-check-abort").unwrap();
        assert_eq!((abort.kind, abort.stereotype.as_deref()), (NodeKind::Action, Some("send")));
        assert_eq!(g.node("s-sys-flight-check-abort-compartment").unwrap().lines, vec!["send Abort", "via ctrlOut"]);
        assert_eq!(g.node("s-sys-flight-check-proceed").unwrap().stereotype.as_deref(), Some("perform"));
        let merge = g.node("s-sys-flight-check-merge").unwrap();
        assert_eq!(merge.kind, NodeKind::Merge);
        let edges = edges_of(&g);
        let succ = |s: &str, t: &str, l: Option<&str>| (EdgeKind::Succession, s.to_string(), t.to_string(), l.map(str::to_string));
        assert!(edges.contains(&succ("s-sys-flight-check", "s-sys-flight-check-abort", Some("[then]"))));
        assert!(edges.contains(&succ("s-sys-flight-check", "s-sys-flight-check-proceed", Some("[else]"))));
        assert!(edges.contains(&succ("s-sys-flight-check-abort", "s-sys-flight-check-merge", None)));
        assert!(edges.contains(&succ("s-sys-flight-check-proceed", "s-sys-flight-check-merge", None)));
        // The loop: a container stereotyped loop, labelled with its for clause, holding its body in order.
        let cruise = g.node("s-sys-flight-cruise").unwrap();
        assert_eq!((cruise.kind, cruise.stereotype.as_deref(), cruise.label.as_str()), (NodeKind::Action, Some("loop"), "cruise [for wp in waypoints]"));
        let body: Vec<&str> = g.children_of("s-sys-flight-cruise").filter(|n| n.kind == NodeKind::Action).map(|n| n.id.as_str()).collect();
        assert_eq!(body, vec!["s-sys-flight-cruise-await", "s-sys-flight-cruise-advance"]);
        assert_eq!(g.node("s-sys-flight-cruise-await-compartment").unwrap().lines, vec!["accept Fix", "when near(wp)"]);
        assert!(edges.contains(&succ("s-sys-flight-cruise-await", "s-sys-flight-cruise-advance", None)));
        // Control nodes.
        assert_eq!(g.node("s-sys-flight-start").unwrap().kind, NodeKind::Fork);
        assert_eq!(g.node("s-sys-flight-end").unwrap().kind, NodeKind::Join);
        // Declared successions enter an if at its decision and leave it at its merge; the guard is the label.
        assert!(edges.contains(&succ("s-sys-flight-start", "s-sys-flight-takeoff", None)));
        assert!(edges.contains(&succ("s-sys-flight-takeoff", "s-sys-flight-check", None)));
        assert!(edges.contains(&succ("s-sys-flight-check-merge", "s-sys-flight-cruise", Some("[ok]"))));
        assert!(edges.contains(&succ("s-sys-flight-cruise", "s-sys-flight-land", None)));
        assert!(edges.contains(&succ("s-sys-flight-land", "s-sys-flight-end", None)));
        // A flow between pins of two steps.
        assert!(edges.contains(&(EdgeKind::Flow, "s-sys-flight-takeoff".into(), "s-sys-flight-cruise".into(), None)));
        // Initial feeds the fork (no incoming succession); the join reaches Final.
        assert_eq!(g.nodes[0].kind, NodeKind::Initial);
        assert_eq!(g.nodes[0].id, "s-sys-flight-initial");
        assert!(edges.contains(&succ("s-sys-flight-initial", "s-sys-flight-start", None)));
        assert!(edges.contains(&succ("s-sys-flight-end", "s-sys-flight-final", None)));
        assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Initial).count(), 1);
        assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Final).count(), 1);
        assert_eq!(g.edges.len(), 13, "{edges:?}");
        assert!(g.edges.iter().any(|e| e.id == "e-succession-s-sys-flight-check-merge-s-sys-flight-cruise"));
        assert_eq!(g.layout_hints.hierarchical, true);
    }

    #[test]
    fn an_action_usage_subject_reads_its_definition() {
        let d = diagram("Action", "Sys::flight", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert!(g.node("s-sys-flight-takeoff").is_some());
        assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Decision).count(), 1);
    }

    #[test]
    fn without_successions_the_steps_are_unordered_and_have_no_initial_or_final() {
        let mut elements = model();
        let f = elements.iter_mut().find(|e| e.qualified_name == "Sys::Flight").unwrap();
        f.frontmatter.succession_connections = None;
        f.frontmatter.flow_connections = None;
        let d = diagram("Action", "Sys::Flight", |_| {});
        elements.push(d.clone());
        let resolver = Resolver::new(&elements);
        let (g, issues) = super::super::derive(&d, crate::vis::ir::DiagramKind::Action, &elements, &resolver);
        assert!(issues.is_empty());
        assert!(g.nodes.iter().all(|n| !matches!(n.kind, NodeKind::Initial | NodeKind::Final)));
        // Only the if's branch and the loop body's internal successions remain.
        assert_eq!(g.edges.len(), 5, "{:?}", edges_of(&g));
    }

    #[test]
    fn an_if_without_an_else_still_routes_the_false_case_to_the_merge() {
        let mut elements = model();
        let f = elements.iter_mut().find(|e| e.qualified_name == "Sys::Flight").unwrap();
        f.frontmatter.sub_actions = Some(yaml_list("- {name: gate, kind: IfAction, condition: go, then: [{name: run, kind: Action}]}\n"));
        f.frontmatter.control_nodes = None;
        f.frontmatter.succession_connections = None;
        f.frontmatter.flow_connections = None;
        let d = diagram("Action", "Sys::Flight", |_| {});
        elements.push(d.clone());
        let resolver = Resolver::new(&elements);
        let (g, _) = super::super::derive(&d, crate::vis::ir::DiagramKind::Action, &elements, &resolver);
        let edges = edges_of(&g);
        assert!(edges.contains(&(EdgeKind::Succession, "s-sys-flight-gate".into(), "s-sys-flight-gate-merge".into(), Some("[else]".into()))), "{edges:?}");
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn include_and_exclude_filter_steps_and_flag_unknown_entries() {
        let d = diagram("Action", "Sys::Flight", |fm| fm.include = Some(vec!["takeoff".into(), "Sys::Flight::land".into(), "ghost".into()]));
        let (g, issues) = derive_it(&d);
        let actions: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::Action).map(|n| n.id.as_str()).collect();
        assert_eq!(actions, vec!["s-sys-flight-takeoff", "s-sys-flight-land"]);
        assert!(g.node("s-sys-flight-start").is_none(), "control nodes are filtered too");
        assert_eq!(issues, vec![super::super::w417("`include` entry 'ghost' names no member of the subject".into())]);
        // Both kept steps have neither incoming nor outgoing successions left: Initial feeds both, both reach Final.
        let edges = edges_of(&g);
        assert_eq!(edges.len(), 4, "{edges:?}");

        let d = diagram("Action", "Sys::Flight", |fm| fm.exclude = Some(vec!["check".into()]));
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty());
        assert!(g.node("s-sys-flight-check").is_none() && g.node("s-sys-flight-check-abort").is_none() && g.node("s-sys-flight-check-merge").is_none());
        assert!(g.edges.iter().all(|e| !e.source.contains("check") && !e.target.contains("check")));
    }

    #[test]
    fn wrong_subject_type_is_w418_and_empty() {
        let d = diagram("Action", "Sys::Engine", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(g.nodes.is_empty());
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "W418");
        assert!(issues[0].message.contains("PartDef"));
    }
}

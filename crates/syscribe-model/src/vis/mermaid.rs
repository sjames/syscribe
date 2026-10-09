//! IR → Mermaid text (`REQ-TRS-VIS-009`, design §7).
//!
//! A pure function of the [`DiagramGraph`]: BDD and Requirement diagrams
//! become a `classDiagram`, IBD/Allocation/UseCase/Custom a `flowchart`,
//! a StateMachine a `stateDiagram-v2`, a Sequence a `sequenceDiagram`.
//! Mermaid lays the picture out itself, so pins are ignored.
//!
//! Every node line is preceded by `%% ref: <QualifiedName>` — the same
//! annotation hand-written Mermaid blocks carry — so the validator's
//! `W408`/`W409` lints apply to generated text exactly as to authored text.
//! Node ids are the IR ids with every character outside `[A-Za-z0-9_]`
//! replaced by `_` (`s-sys-engine` → `s_sys_engine`).
//!
//! When the `links` closure yields a URL for a node's reference, a
//! `click <id> href "<url>" _blank` line is appended for class diagrams and
//! flowcharts (the two Mermaid grammars that accept it); other kinds carry no
//! links.

use super::ir::{DiagramGraph, DiagramKind, Edge, EdgeKind, Node, NodeKind, Tone};
use super::style::{edge_style, tone_colors};

/// A Mermaid-safe node id: the IR id with every character outside
/// `[A-Za-z0-9_]` replaced by `_`.
pub fn mermaid_id(id: &str) -> String {
    id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect()
}

/// Text inside a Mermaid double-quoted label: `"` becomes the `#quot;`
/// entity Mermaid decodes, and a newline a space.
fn text(s: &str) -> String {
    s.replace('"', "#quot;").replace('\n', " ")
}

/// A member line inside a `class … { }` body. Braces would end the body
/// early, so they are replaced; everything else is shown as written.
fn member(s: &str) -> String {
    s.replace(['{', '}'], "").replace('\n', " ")
}

fn ref_line(out: &mut String, indent: &str, node: &Node) {
    out.push_str(indent);
    out.push_str("%% ref: ");
    out.push_str(&node.element_ref);
    out.push('\n');
}

/// The `<<stereotype>>` of a class node: the IR's text, else a role word.
fn class_stereotype(node: &Node) -> Option<String> {
    node.stereotype.clone().or_else(|| match node.kind {
        NodeKind::Block => Some("part".to_string()),
        NodeKind::Requirement => Some("requirement".to_string()),
        NodeKind::TestCase => Some("test case".to_string()),
        NodeKind::UseCase => Some("use case".to_string()),
        NodeKind::Actor => Some("actor".to_string()),
        _ => None,
    })
}

/// The compartment lines of a class node: its own `lines` followed by those
/// of its `Compartment` children, in declaration order.
fn class_lines<'a>(graph: &'a DiagramGraph, node: &'a Node) -> Vec<&'a str> {
    let mut out: Vec<&str> = node.lines.iter().map(String::as_str).collect();
    for c in graph.children_of(&node.id).filter(|c| c.kind == NodeKind::Compartment) {
        out.extend(c.lines.iter().map(String::as_str));
    }
    out
}

/// Whether a node is drawn as a class in a `classDiagram`: everything but
/// the decorations that hang off another node.
fn is_class_node(node: &Node) -> bool {
    !matches!(node.kind, NodeKind::Compartment | NodeKind::Label | NodeKind::Port | NodeKind::Note)
}

/// The edge label an author sees: the explicit `label:`, else the kind's
/// `«keyword»` for the requirement/allocation relationships.
fn keyword_label(edge: &Edge) -> Option<String> {
    edge.label.clone().or_else(|| edge_style(edge.kind).keyword)
}

/// `click <id> href "<url>" _blank` for every node whose reference links.
fn click_lines(out: &mut String, nodes: &[&Node], links: &dyn Fn(&str) -> Option<String>) {
    let mut any = false;
    for n in nodes {
        if let Some(url) = links(&n.element_ref) {
            if !any {
                out.push('\n');
                any = true;
            }
            out.push_str(&format!("  click {} href \"{}\" _blank\n", mermaid_id(&n.id), text(&url)));
        }
    }
}

/// Render the graph as Mermaid text. Every IR kind has a mapping, so the
/// result is `Some` for any graph; the option is the shared writer contract
/// (the PlantUML writer declines some kinds).
pub fn render_mermaid(graph: &DiagramGraph, links: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    Some(match graph.kind {
        DiagramKind::Bdd | DiagramKind::Requirement => render_class(graph, links),
        DiagramKind::Ibd => render_flowchart(graph, "LR", links),
        DiagramKind::StateMachine => render_state(graph),
        DiagramKind::Sequence => render_sequence(graph),
        DiagramKind::Allocation | DiagramKind::UseCase => render_flowchart(graph, "LR", links),
        DiagramKind::Action
        | DiagramKind::Custom
        | DiagramKind::FeatureModel
        | DiagramKind::FaultTree
        | DiagramKind::AttackTree
        | DiagramKind::SafetyCase => render_flowchart(graph, "TD", links),
    })
}

// ── classDiagram (BDD, Requirement) ──────────────────────────────────────────

fn render_class(graph: &DiagramGraph, links: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = String::from("classDiagram\n");
    let classes: Vec<&Node> = graph.nodes.iter().filter(|n| is_class_node(n)).collect();
    for n in &classes {
        ref_line(&mut out, "  ", n);
        let id = mermaid_id(&n.id);
        let label = text(&n.label);
        let lines = class_lines(graph, n);
        let stereotype = class_stereotype(n);
        if stereotype.is_none() && lines.is_empty() {
            out.push_str(&format!("  class {id}[\"{label}\"]\n"));
            continue;
        }
        out.push_str(&format!("  class {id}[\"{label}\"] {{\n"));
        if let Some(st) = stereotype {
            out.push_str(&format!("    <<{}>>\n", member(&st)));
        }
        for l in lines {
            out.push_str(&format!("    +{}\n", member(l)));
        }
        out.push_str("  }\n");
    }
    if !graph.edges.is_empty() {
        out.push('\n');
    }
    for e in &graph.edges {
        let s = mermaid_id(&e.source);
        let t = mermaid_id(&e.target);
        let label = match e.kind {
            EdgeKind::Inheritance => None,
            _ => keyword_label(e),
        };
        let (a, connector, b) = match e.kind {
            // `Super <|-- Sub`: the IR's source is the specialising element.
            EdgeKind::Inheritance => (&t, "<|--", &s),
            EdgeKind::Composition => (&s, "*--", &t),
            EdgeKind::Aggregation => (&s, "o--", &t),
            EdgeKind::Association | EdgeKind::Containment | EdgeKind::Connection | EdgeKind::Binding => (&s, "--", &t),
            EdgeKind::Dependency | EdgeKind::Derive | EdgeKind::Satisfy | EdgeKind::Verify | EdgeKind::Refine
            | EdgeKind::Trace | EdgeKind::Copy | EdgeKind::Allocation | EdgeKind::Include | EdgeKind::Extend => {
                (&s, "..>", &t)
            }
            _ => (&s, "-->", &t),
        };
        out.push_str(&format!("  {a} {connector} {b}"));
        if let Some(l) = label {
            out.push_str(&format!(" : {}", text(&l)));
        }
        out.push('\n');
    }
    click_lines(&mut out, &classes, links);
    out
}

// ── flowchart (IBD, Allocation, UseCase, Custom) ─────────────────────────────

/// A flowchart node's shape by role: ports are small circles, use cases
/// stadiums, actors trapezoids, everything else a box.
/// A node's text with its mark (status, value, badges) on following lines
/// (`<br/>`, which Mermaid renders inside a quoted label).
fn flow_label(node: &Node) -> String {
    let mut label = text(&node.label);
    if let Some(m) = &node.mark {
        let badges = (!m.badges.is_empty()).then(|| m.badges.iter().map(|b| format!("[{b}]")).collect::<Vec<_>>().join(" "));
        for line in [m.status.clone(), m.value.clone(), badges].into_iter().flatten() {
            label.push_str("<br/>");
            label.push_str(&text(&line));
        }
    }
    label
}

fn flow_node(node: &Node) -> String {
    let id = mermaid_id(&node.id);
    let label = flow_label(node);
    match node.kind {
        // Safety diagrams (GH #223): the closest Mermaid shape to each symbol.
        NodeKind::GateAnd => format!("{id}([\"AND<br/>{label}\"])"),
        NodeKind::GateOr => format!("{id}([\"OR<br/>{label}\"])"),
        NodeKind::GateXor => format!("{id}([\"XOR<br/>{label}\"])"),
        NodeKind::GateNot => format!("{id}([\"NOT<br/>{label}\"])"),
        NodeKind::GateInhibit => format!("{id}{{{{\"INHIBIT<br/>{label}\"}}}}"),
        NodeKind::EventBasic | NodeKind::Solution => format!("{id}((\"{label}\"))"),
        NodeKind::EventUndeveloped => format!("{id}{{\"{label}\"}}"),
        NodeKind::EventHouse => format!("{id}[/\"{label}\"\\]"),
        NodeKind::Strategy => format!("{id}[/\"{label}\"/]"),
        NodeKind::Context => format!("{id}([\"{label}\"])"),
        NodeKind::Justification => format!("{id}((\"J: {label}\"))"),
        NodeKind::Assumption => format!("{id}((\"A: {label}\"))"),
        NodeKind::Port => format!("{id}(({label}))"),
        NodeKind::UseCase => format!("{id}([\"{label}\"])"),
        NodeKind::Actor => format!("{id}[/\"{label}\"/]"),
        NodeKind::Note => format!("{id}>\"{label}\"]"),
        NodeKind::Initial | NodeKind::Final => format!("{id}(( ))"),
        // Action flows (REQ-TRS-VIS-019): diamonds for decision/merge, bars for fork/join.
        NodeKind::Decision | NodeKind::Merge => format!("{id}{{{{\"{}\"}}}}", if label.is_empty() { " " } else { &label }),
        NodeKind::Fork | NodeKind::Join => format!("{id}[[\"{}\"]]", if label.is_empty() { " " } else { &label }),
        _ => format!("{id}[\"{label}\"]"),
    }
}

/// Whether a node is drawn as a `subgraph` with its children inside: a
/// container role, or anything that has drawable children (a compartment or
/// a label child is text, not a nested shape — an action step with its
/// `typedBy` compartment stays a box).
fn is_subgraph(graph: &DiagramGraph, node: &Node) -> bool {
    matches!(node.kind, NodeKind::Boundary | NodeKind::SystemBoundary | NodeKind::Swimlane)
        || graph.children_of(&node.id).any(|c| !matches!(c.kind, NodeKind::Compartment | NodeKind::Label))
}

fn flow_nodes(graph: &DiagramGraph, node: &Node, depth: usize, out: &mut String, drawn: &mut Vec<String>) {
    let indent = "  ".repeat(depth + 1);
    if matches!(node.kind, NodeKind::Compartment | NodeKind::Label) {
        return;
    }
    ref_line(out, &indent, node);
    drawn.push(node.id.clone());
    if is_subgraph(graph, node) && depth < graph.nodes.len() {
        out.push_str(&format!("{indent}subgraph {}[\"{}\"]\n", mermaid_id(&node.id), text(&node.label)));
        for child in graph.children_of(&node.id) {
            flow_nodes(graph, child, depth + 1, out, drawn);
        }
        out.push_str(&format!("{indent}end\n"));
    } else {
        out.push_str(&format!("{indent}{}\n", flow_node(node)));
    }
}

fn is_root(graph: &DiagramGraph, node: &Node) -> bool {
    match node.parent.as_deref() {
        None => true,
        Some(p) => graph.node(p).is_none(),
    }
}

fn render_flowchart(graph: &DiagramGraph, direction: &str, links: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = format!("flowchart {direction}\n");
    let mut drawn: Vec<String> = Vec::new();
    for n in graph.nodes.iter().filter(|n| is_root(graph, n)) {
        flow_nodes(graph, n, 0, &mut out, &mut drawn);
    }
    if !graph.edges.is_empty() {
        out.push('\n');
    }
    for e in &graph.edges {
        let s = mermaid_id(&e.source);
        let t = mermaid_id(&e.target);
        let connector = match e.kind {
            EdgeKind::Connection | EdgeKind::Association | EdgeKind::Containment | EdgeKind::FeatureChild | EdgeKind::GateInput => "---",
            EdgeKind::CriticalPath => "===",
            EdgeKind::InContextOf => "-.->",
            EdgeKind::Excludes => "-.-",
            EdgeKind::Requires => "-.->",
            EdgeKind::Binding => "-.-",
            EdgeKind::Dependency | EdgeKind::Allocation | EdgeKind::Derive | EdgeKind::Satisfy | EdgeKind::Verify
            | EdgeKind::Refine | EdgeKind::Trace | EdgeKind::Copy | EdgeKind::Include | EdgeKind::Extend => "-.->",
            _ => "-->",
        };
        // A binding is drawn with its `=` keyword, before any explicit label.
        let label = match (e.kind, &e.label) {
            (EdgeKind::Binding, Some(l)) => Some(format!("= {l}")),
            (EdgeKind::Binding, None) => Some("=".to_string()),
            _ => keyword_label(e),
        };
        out.push_str(&format!("  {s} {connector}"));
        if let Some(l) = label {
            out.push_str(&format!("|{}|", text(&l).replace('|', "/")));
        }
        out.push_str(&format!(" {t}\n"));
    }
    let nodes: Vec<&Node> = drawn.iter().filter_map(|id| graph.node(id)).collect();
    tone_classes(&mut out, &nodes);
    click_lines(&mut out, &nodes, links);
    out
}

/// `classDef`/`class` lines painting every marked node in its tone's colours
/// (GH #223), so the flowchart carries the same status colouring as the other
/// writers. Emitted only for the tones in use, in tone order.
fn tone_classes(out: &mut String, nodes: &[&Node]) {
    let mut any = false;
    for tone in [Tone::Ok, Tone::Warn, Tone::Bad, Tone::Neutral] {
        let ids: Vec<String> = nodes.iter().filter(|n| n.mark.as_ref().is_some_and(|m| m.tone == tone)).map(|n| mermaid_id(&n.id)).collect();
        if ids.is_empty() {
            continue;
        }
        if !any {
            out.push('\n');
            any = true;
        }
        let (fill, stroke) = tone_colors(tone);
        out.push_str(&format!("  classDef tone_{} fill:{fill},stroke:{stroke},color:#222\n", tone.as_str()));
        out.push_str(&format!("  class {} tone_{}\n", ids.join(","), tone.as_str()));
    }
}

// ── stateDiagram-v2 ──────────────────────────────────────────────────────────

fn is_pseudo_state(n: &Node) -> bool {
    matches!(n.kind, NodeKind::Initial | NodeKind::Final)
}

/// One state (or choice/history) declaration, with a composite state's
/// nested states inside `{ … }` and its entry/do/exit compartment lines as
/// `id : line` descriptions (a derived StateMachine, REQ-TRS-VIS-018).
fn state_nodes(graph: &DiagramGraph, node: &Node, depth: usize, out: &mut String) {
    if is_pseudo_state(node) || matches!(node.kind, NodeKind::Compartment | NodeKind::Label) {
        return;
    }
    let indent = "  ".repeat(depth + 1);
    ref_line(out, &indent, node);
    let id = mermaid_id(&node.id);
    match node.kind {
        NodeKind::Choice => out.push_str(&format!("{indent}state {id} <<choice>>\n")),
        _ => {
            let nested: Vec<&Node> = graph.children_of(&node.id).filter(|c| !matches!(c.kind, NodeKind::Compartment | NodeKind::Label)).collect();
            if nested.is_empty() || depth >= graph.nodes.len() {
                out.push_str(&format!("{indent}state \"{}\" as {id}\n", text(&node.label)));
            } else {
                out.push_str(&format!("{indent}state \"{}\" as {id} {{\n", text(&node.label)));
                for c in nested {
                    state_nodes(graph, c, depth + 1, out);
                }
                out.push_str(&format!("{indent}}}\n"));
            }
        }
    }
    for line in class_lines(graph, node) {
        out.push_str(&format!("{indent}{id} : {}\n", text(line)));
    }
}

fn render_state(graph: &DiagramGraph) -> String {
    let mut out = String::from("stateDiagram-v2\n");
    for n in graph.nodes.iter().filter(|n| is_root(graph, n)) {
        state_nodes(graph, n, 0, &mut out);
    }
    if !graph.edges.is_empty() {
        out.push('\n');
    }
    let end = |id: &str| match graph.node(id) {
        Some(n) if is_pseudo_state(n) => "[*]".to_string(),
        _ => mermaid_id(id),
    };
    for e in &graph.edges {
        out.push_str(&format!("  {} --> {}", end(&e.source), end(&e.target)));
        if let Some(l) = &e.label {
            out.push_str(&format!(" : {}", text(l)));
        }
        out.push('\n');
    }
    out
}

// ── sequenceDiagram ──────────────────────────────────────────────────────────

fn render_sequence(graph: &DiagramGraph) -> String {
    let mut out = String::from("sequenceDiagram\n");
    for n in graph.nodes.iter().filter(|n| matches!(n.kind, NodeKind::Lifeline | NodeKind::Actor)) {
        ref_line(&mut out, "  ", n);
        let word = if n.kind == NodeKind::Actor { "actor" } else { "participant" };
        out.push_str(&format!("  {word} {} as {}\n", mermaid_id(&n.id), text(&n.label)));
    }
    if !graph.edges.is_empty() {
        out.push('\n');
    }
    for e in &graph.edges {
        let arrow = match e.kind {
            EdgeKind::Return => "-->>",
            EdgeKind::Destroy => "-x",
            _ => "->>",
        };
        let label = e.label.clone().unwrap_or_else(|| e.id.strip_prefix("e-").unwrap_or(&e.id).to_string());
        out.push_str(&format!("  {}{arrow}{}: {}\n", mermaid_id(&e.source), mermaid_id(&e.target), text(&label)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vis::ir::{Edge, EdgeKind, NodeKind, Point, PortDirection};

    fn node(id: &str, kind: NodeKind, parent: Option<&str>) -> Node {
        Node {
            id: id.into(),
            element_ref: format!("Sys::{id}"),
            resolved: true,
            element_type: None,
            kind,
            label: id.to_uppercase(),
            stereotype: None,
            parent: parent.map(str::to_string),
            direction: None,
            side: None,
            lines: vec![],
            is_abstract: false,
            pin: None,
            banners: vec![],
            feature: None,
            mark: None,
        }
    }

    fn edge(id: &str, s: &str, t: &str, kind: EdgeKind, label: Option<&str>) -> Edge {
        Edge {
            id: id.into(),
            element_ref: None,
            source: s.into(),
            target: t.into(),
            kind,
            label: label.map(str::to_string),
            waypoints: Some(vec![Point { x: 0.0, y: 0.0 }]),
        }
    }

    fn no_links(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn ids_are_sanitised() {
        assert_eq!(mermaid_id("s-sys-engine"), "s_sys_engine");
        assert_eq!(mermaid_id("a.b c"), "a_b_c");
        assert_eq!(mermaid_id("ok_1"), "ok_1");
    }

    #[test]
    fn bdd_is_a_class_diagram_with_stereotypes_compartments_and_notation() {
        let mut g = DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None);
        let mut base = node("s-base", NodeKind::Block, None);
        base.stereotype = Some("part def".into());
        base.is_abstract = true;
        g.nodes.push(base);
        let mut engine = node("s-engine", NodeKind::Block, None);
        engine.stereotype = Some("part def".into());
        g.nodes.push(engine);
        let mut comp = node("s-engine-compartment", NodeKind::Compartment, Some("s-engine"));
        comp.lines = vec!["mass : Real [kg]".into(), "port out : P (out)".into()];
        g.nodes.push(comp);
        g.nodes.push(node("s-sys", NodeKind::Block, None));
        g.edges.push(edge("e1", "s-engine", "s-base", EdgeKind::Inheritance, None));
        g.edges.push(edge("e2", "s-sys", "s-engine", EdgeKind::Composition, Some("engine [2]")));
        g.edges.push(edge("e3", "s-sys", "s-base", EdgeKind::Association, Some("Link")));
        g.edges.push(edge("e4", "s-base", "s-sys", EdgeKind::Dependency, None));
        let m = render_mermaid(&g, &no_links).unwrap();
        assert!(m.starts_with("classDiagram\n"));
        assert!(m.contains("  %% ref: Sys::s-engine\n  class s_engine[\"S-ENGINE\"] {\n    <<part def>>\n    +mass : Real [kg]\n    +port out : P (out)\n  }\n"), "{m}");
        assert!(m.contains("  s_base <|-- s_engine\n"), "{m}");
        assert!(m.contains("  s_sys *-- s_engine : engine [2]\n"), "{m}");
        assert!(m.contains("  s_sys -- s_base : Link\n"), "{m}");
        assert!(m.contains("  s_base ..> s_sys\n"), "{m}");
        // The compartment is not a class of its own and has no ref line.
        assert!(!m.contains("ref: Sys::s-engine-compartment"));
        assert_eq!(m.matches("%% ref:").count(), 3);
        assert!(!m.contains("click "));
    }

    #[test]
    fn ibd_is_a_flowchart_with_nested_subgraphs_and_port_nodes() {
        let mut g = DiagramGraph::empty(DiagramKind::Ibd, "D", "D", Some("Sys"));
        g.nodes.push(node("sys", NodeKind::Boundary, None));
        let mut main = node("sys-main", NodeKind::Port, Some("sys"));
        main.direction = Some(PortDirection::Out);
        g.nodes.push(main);
        g.nodes.push(node("sys-engine", NodeKind::Block, Some("sys")));
        g.nodes.push(node("sys-engine-out", NodeKind::Port, Some("sys-engine")));
        g.nodes.push(node("sys-motor", NodeKind::Block, Some("sys")));
        g.edges.push(edge("c1", "sys-engine-out", "sys-motor", EdgeKind::Connection, Some("PowerLink")));
        g.edges.push(edge("f1", "sys-engine-out", "sys-motor", EdgeKind::Flow, None));
        g.edges.push(edge("b1", "sys-main", "sys-engine-out", EdgeKind::Binding, None));
        let m = render_mermaid(&g, &no_links).unwrap();
        assert_eq!(
            m,
            "flowchart LR\n  %% ref: Sys::sys\n  subgraph sys[\"SYS\"]\n    %% ref: Sys::sys-main\n    sys_main((SYS-MAIN))\n    %% ref: Sys::sys-engine\n    subgraph sys_engine[\"SYS-ENGINE\"]\n      %% ref: Sys::sys-engine-out\n      sys_engine_out((SYS-ENGINE-OUT))\n    end\n    %% ref: Sys::sys-motor\n    sys_motor[\"SYS-MOTOR\"]\n  end\n\n  sys_engine_out ---|PowerLink| sys_motor\n  sys_engine_out --> sys_motor\n  sys_main -.-|=| sys_engine_out\n"
        );
    }

    #[test]
    fn links_become_click_lines_for_class_and_flowchart_only() {
        let links = |r: &str| (r == "Sys::a").then(|| "https://x.test/a.md?q=\"1\"".to_string());
        let mut g = DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None);
        g.nodes.push(node("a", NodeKind::Block, None));
        g.nodes.push(node("b", NodeKind::Block, None));
        let m = render_mermaid(&g, &links).unwrap();
        assert!(m.ends_with("\n  click a href \"https://x.test/a.md?q=#quot;1#quot;\" _blank\n"), "{m}");
        assert_eq!(m.matches("click ").count(), 1);
        g.kind = DiagramKind::Custom;
        let m = render_mermaid(&g, &links).unwrap();
        assert!(m.starts_with("flowchart TD\n"));
        assert!(m.contains("  click a href "), "{m}");
        g.kind = DiagramKind::StateMachine;
        g.nodes[0].kind = NodeKind::State;
        assert!(!render_mermaid(&g, &links).unwrap().contains("click "));
    }

    #[test]
    fn state_sequence_and_requirement_kinds_map_to_their_grammars() {
        let mut g = DiagramGraph::empty(DiagramKind::StateMachine, "D", "D", None);
        g.nodes.push(node("i", NodeKind::Initial, None));
        g.nodes.push(node("on", NodeKind::State, None));
        g.nodes.push(node("f", NodeKind::Final, None));
        g.edges.push(edge("e-start", "i", "on", EdgeKind::Transition, None));
        g.edges.push(edge("e-stop", "on", "f", EdgeKind::Transition, Some("off")));
        let m = render_mermaid(&g, &no_links).unwrap();
        assert_eq!(m, "stateDiagram-v2\n  %% ref: Sys::on\n  state \"ON\" as on\n\n  [*] --> on\n  on --> [*] : off\n");

        let mut s = DiagramGraph::empty(DiagramKind::Sequence, "D", "D", None);
        s.nodes.push(node("u", NodeKind::Actor, None));
        s.nodes.push(node("fc", NodeKind::Lifeline, None));
        s.edges.push(edge("e-arm", "u", "fc", EdgeKind::Message, None));
        s.edges.push(edge("e-ack", "fc", "u", EdgeKind::Return, Some("ok")));
        let m = render_mermaid(&s, &no_links).unwrap();
        assert_eq!(m, "sequenceDiagram\n  %% ref: Sys::u\n  actor u as U\n  %% ref: Sys::fc\n  participant fc as FC\n\n  u->>fc: arm\n  fc-->>u: ok\n");

        let mut r = DiagramGraph::empty(DiagramKind::Requirement, "D", "D", None);
        let mut req = node("r1", NodeKind::Requirement, None);
        req.stereotype = Some("requirement".into());
        r.nodes.push(req);
        r.nodes.push(node("t1", NodeKind::TestCase, None));
        r.edges.push(edge("e-v", "t1", "r1", EdgeKind::Verify, None));
        let m = render_mermaid(&r, &no_links).unwrap();
        assert!(m.contains("  class r1[\"R1\"] {\n    <<requirement>>\n  }\n"), "{m}");
        assert!(m.contains("  class t1[\"T1\"] {\n    <<test case>>\n  }\n"), "{m}");
        assert!(m.contains("  t1 ..> r1 : «verify»\n"), "{m}");
    }

    #[test]
    fn output_is_deterministic_and_quotes_are_escaped() {
        let mut g = DiagramGraph::empty(DiagramKind::Custom, "D", "D", None);
        let mut n = node("n", NodeKind::Block, None);
        n.label = "say \"hi\"".into();
        g.nodes.push(n);
        let a = render_mermaid(&g, &no_links).unwrap();
        assert_eq!(a, render_mermaid(&g, &no_links).unwrap());
        assert!(a.contains("n[\"say #quot;hi#quot;\"]"), "{a}");
    }
}

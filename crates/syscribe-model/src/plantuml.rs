//! PlantUML writer (`REQ-TRS-VIS-009`, `REQ-TRS-PUML-*`).
//!
//! A pure function of the Diagram IR ([`crate::vis::DiagramGraph`]): the
//! `Diagram` element's `shapes:`/`edges:`/`layout:` frontmatter is never read
//! here — [`crate::vis::build_graph`] is the one manifest parser, and each
//! `render_*` below walks the graph it produced. The output for the demo
//! model's `pumlMode: companion` diagrams is snapshot-tested
//! (`tests/vis_plantuml_snapshot.rs`) so a change to the IR or to this writer
//! is visible as a diff.

use std::collections::HashSet;

use crate::config::PlantumlConfig;
use crate::element::RawElement;
use crate::resolver::Resolver;
use crate::vis::{self, DiagramGraph, DiagramKind, Edge, EdgeKind, Node, NodeKind};

// ── Shared helpers ────────────────────────────────────────────────────────────

fn short_name(qname: &str) -> String {
    qname.rsplit("::").next().unwrap_or(qname).to_string()
}

/// Replace hyphens with underscores so the string is a valid PlantUML identifier.
fn sanitize_id(s: &str) -> String {
    s.replace('-', "_")
}

/// An edge's role label when it carries no explicit `label:`: the short name
/// of the element it refers to, else its id without the conventional `e-`
/// prefix.
fn edge_label(key: &str, eref: Option<&str>) -> String {
    eref.map(short_name)
        .unwrap_or_else(|| key_label(key))
}

/// An edge id without its conventional `e-` prefix (`e-propulsion` → `propulsion`).
fn key_label(key: &str) -> String {
    key.strip_prefix("e-").unwrap_or(key).to_string()
}

/// The `<<stereotype>>` text of a class-diagram node: the IR's stereotype
/// (the resolved element's type, or the type an element-type-named `kind:`
/// implied), else `part` for a plain `block`, else the role name itself.
fn class_stereotype(node: &Node) -> String {
    match node.stereotype.as_deref() {
        Some(s) => s.to_string(),
        None if node.kind == NodeKind::Block => "part".to_string(),
        None => node.kind.as_str().to_string(),
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Generate a PlantUML `.puml` source string from a `Diagram` element.
/// Returns `None` when the element is not a `Diagram` or its `diagramKind`
/// has no PlantUML mapping (`Mermaid`, `PlantUML`, `Allocation`, `UseCase`,
/// `Custom`/absent).
pub fn render_plantuml(
    element: &RawElement,
    elements: &[RawElement],
    cfg: Option<&PlantumlConfig>,
) -> Option<String> {
    let resolver = Resolver::new(elements);
    let (graph, _issues) = vis::build_graph(element, elements, &resolver)?;

    // @startuml identifier / PlantUML output filename stem — must not contain
    // spaces so PlantUML names the .svg predictably.  Derive from the file stem.
    let file_stem: String = std::path::Path::new(&element.file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&graph.name)
        .replace(' ', "_");

    match graph.kind {
        DiagramKind::Bdd => Some(render_bdd(&graph, &file_stem, cfg)),
        DiagramKind::Ibd => Some(render_ibd(&graph, &file_stem, cfg)),
        DiagramKind::StateMachine => Some(render_state_machine(&graph, &file_stem, cfg)),
        DiagramKind::Sequence => Some(render_sequence(&graph, &file_stem, cfg)),
        DiagramKind::Requirement => Some(render_requirement(&graph, &file_stem, cfg)),
        DiagramKind::Allocation | DiagramKind::UseCase | DiagramKind::Action | DiagramKind::Custom => None,
    }
}

/// Return `[[<base_url>/<qref_as_path>.md]]` when `base_url` is configured, else `""`.
/// Absent or empty `base_url` suppresses links (REQ-TRS-PUML-055).
/// `qref_as_path` replaces every `::` with `/` so the URL resolves to the
/// element's file on GitHub or any static hosting.
fn element_url(qref: &str, cfg: Option<&PlantumlConfig>) -> String {
    let base = match cfg {
        None => return String::new(),
        Some(c) => match c.base_url.as_deref() {
            None | Some("") => return String::new(),
            Some(b) => b,
        },
    };
    let path = qref.replace("::", "/");
    format!("[[{}/{}.md]]", base, path)
}

/// Emit the style preamble: `!include`, `!theme`, or built-in SysML skinparams.
fn style_preamble(cfg: Option<&PlantumlConfig>) -> String {
    if let Some(c) = cfg {
        if let Some(ref sf) = c.style_file {
            return format!("!include {}\n", sf.display());
        }
        if let Some(ref theme) = c.theme {
            return format!("!theme {}\n", theme);
        }
    }
    // Built-in SysML professional defaults — kept in sync with .plantuml/sysml.iuml
    SYSML_SKINPARAM.to_string()
}

static SYSML_SKINPARAM: &str = r#"
skinparam defaultFontName "Helvetica Neue, Helvetica, Arial, sans-serif"
skinparam defaultFontSize 11
skinparam backgroundColor WHITE
skinparam shadowing false
skinparam roundCorner 4
skinparam ArrowColor #1F497D
skinparam ArrowThickness 1.5
skinparam ArrowFontSize 10
skinparam ArrowFontColor #2C3E50
skinparam note {
  BackgroundColor #FFFDE7
  BorderColor #F9A825
  FontSize 10
  BorderThickness 1
}
skinparam class {
  BackgroundColor #D4E6F1
  BorderColor #1A4A7A
  BorderThickness 1.5
  HeaderBackgroundColor #1A4A7A
  FontColor #FFFFFF
  FontSize 11
  FontStyle bold
  AttributeFontColor #1C2833
  AttributeFontSize 10
  ArrowColor #1F497D
  ArrowThickness 1.5
  StereotypeFontColor #A8C8E8
  StereotypeFontSize 9
  BackgroundColor<<requirement>> #FDEDEC
  BorderColor<<requirement>> #C0392B
  HeaderBackgroundColor<<requirement>> #C0392B
  FontColor<<requirement>> #FFFFFF
  StereotypeFontColor<<requirement>> #F5B7B1
  BackgroundColor<<requirement def>> #FDEDEC
  BorderColor<<requirement def>> #C0392B
  HeaderBackgroundColor<<requirement def>> #922B21
  FontColor<<requirement def>> #FFFFFF
  BackgroundColor<<test case>> #E9F7EF
  BorderColor<<test case>> #1E8449
  HeaderBackgroundColor<<test case>> #1E8449
  FontColor<<test case>> #FFFFFF
  StereotypeFontColor<<test case>> #A9DFBF
  BackgroundColor<<test case def>> #E9F7EF
  BorderColor<<test case def>> #196F3D
  HeaderBackgroundColor<<test case def>> #196F3D
  FontColor<<test case def>> #FFFFFF
  BackgroundColor<<part>> #D4E6F1
  BorderColor<<part>> #1A4A7A
  HeaderBackgroundColor<<part>> #1A4A7A
  FontColor<<part>> #FFFFFF
  StereotypeFontColor<<part>> #A8C8E8
  BackgroundColor<<part def>> #D4E6F1
  BorderColor<<part def>> #154360
  HeaderBackgroundColor<<part def>> #154360
  FontColor<<part def>> #FFFFFF
  BackgroundColor<<action>> #FEF9E7
  BorderColor<<action>> #B7950B
  HeaderBackgroundColor<<action>> #B7950B
  FontColor<<action>> #FFFFFF
  BackgroundColor<<action def>> #FEF9E7
  BorderColor<<action def>> #9A7D0A
  HeaderBackgroundColor<<action def>> #9A7D0A
  FontColor<<action def>> #FFFFFF
  BackgroundColor<<interface>> #E8F8F5
  BorderColor<<interface>> #148F77
  HeaderBackgroundColor<<interface>> #0E6655
  FontColor<<interface>> #FFFFFF
  BackgroundColor<<interface def>> #E8F8F5
  BorderColor<<interface def>> #0E6655
  HeaderBackgroundColor<<interface def>> #0B5345
  FontColor<<interface def>> #FFFFFF
  BackgroundColor<<allocation>> #F4ECF7
  BorderColor<<allocation>> #7D3C98
  HeaderBackgroundColor<<allocation>> #7D3C98
  FontColor<<allocation>> #FFFFFF
}
skinparam component {
  BackgroundColor #D4E6F1
  BorderColor #1A4A7A
  BorderThickness 1.5
  FontColor #1C2833
  FontSize 11
  FontStyle bold
  ArrowColor #1F497D
  ArrowThickness 1.5
  StereotypeFontColor #4A7A9B
  StereotypeFontSize 9
}
skinparam rectangle {
  BackgroundColor #AED6F1
  BorderColor #1A4A7A
  BorderThickness 2
  FontColor #1A4A7A
  FontSize 12
  FontStyle bold
}
skinparam state {
  BackgroundColor #E8D5F5
  BorderColor #5B2C8E
  BorderThickness 1.5
  FontColor #2C1654
  FontSize 11
  FontStyle bold
  ArrowColor #5B2C8E
  ArrowThickness 1.5
  StartColor #2C1654
  EndColor #2C1654
}
skinparam participant {
  BackgroundColor #D4E6F1
  BorderColor #1A4A7A
  BorderThickness 1.5
  FontColor #1C2833
  FontSize 11
  FontStyle bold
}
skinparam actor {
  BackgroundColor #D4E6F1
  BorderColor #1A4A7A
}
skinparam sequence {
  ArrowColor #1F497D
  ArrowThickness 1.5
  LifeLineBorderColor #1A4A7A
  LifeLineBorderThickness 1
  LifeLineBackgroundColor #EBF5FB
  MessageAlignment left
  DividerBackgroundColor #AED6F1
  DividerBorderColor #1A4A7A
  GroupBackgroundColor #EBF5FB
  GroupBorderColor #1A4A7A
}
"#;

// ── BDD ───────────────────────────────────────────────────────────────────────

fn render_bdd(graph: &DiagramGraph, id: &str, cfg: Option<&PlantumlConfig>) -> String {
    let mut out = String::new();
    out.push_str(&format!("@startuml {}\n", id));
    out.push_str(&style_preamble(cfg));
    out.push_str("hide empty members\n\n");

    for node in &graph.nodes {
        let url = element_url(&node.element_ref, cfg);
        out.push_str(&format!(
            "class \"{}\" as {} <<{}>> {}\n",
            node.label,
            sanitize_id(&node.id),
            class_stereotype(node),
            url
        ));
    }

    out.push('\n');

    for e in &graph.edges {
        // For BDD edges the ref often points to the owning element rather than
        // the member, so the edge id (e.g. "e-propulsion" → "propulsion") is
        // the most reliable role label.
        let label = key_label(&e.id);
        let connector = match e.kind {
            EdgeKind::Composition => "*--",
            EdgeKind::Dependency => "..>",
            EdgeKind::Inheritance => "--|>",
            _ => "-->",
        };
        out.push_str(&format!(
            "{} {} {} : {}\n",
            sanitize_id(&e.source),
            connector,
            sanitize_id(&e.target),
            label
        ));
    }

    out.push_str("\n@enduml\n");
    out
}

// ── IBD ───────────────────────────────────────────────────────────────────────

fn render_ibd_container(
    node: &Node,
    graph: &DiagramGraph,
    container_ids: &HashSet<&str>,
    cfg: Option<&PlantumlConfig>,
    indent: usize,
    out: &mut String,
) {
    let pad = "  ".repeat(indent);
    let url = element_url(&node.element_ref, cfg);
    out.push_str(&format!("{}rectangle \"{}\" as {} {} {{\n", pad, node.label, sanitize_id(&node.id), url));
    for child in graph.children_of(&node.id) {
        if child.kind == NodeKind::Port {
            continue;
        }
        if container_ids.contains(child.id.as_str()) {
            render_ibd_container(child, graph, container_ids, cfg, indent + 1, out);
        } else {
            let curl = element_url(&child.element_ref, cfg);
            out.push_str(&format!("{}  component \"{}\" as {} {}\n", pad, child.label, sanitize_id(&child.id), curl));
        }
    }
    out.push_str(&format!("{}}}\n", pad));
}

fn render_ibd(graph: &DiagramGraph, id: &str, cfg: Option<&PlantumlConfig>) -> String {
    // Resolve a node id through its parent to the block that owns it: a port
    // is drawn on its parent, so an edge between ports becomes an edge between
    // their blocks. A port with no `parent:` falls back to the block whose
    // element reference is the port reference's owner prefix.
    let resolve_to_block = |id: &str| -> String {
        let node = graph.node(id);
        if let Some(port) = node.filter(|n| n.kind == NodeKind::Port) {
            if let Some(parent) = port.parent.as_deref() {
                return parent.to_string();
            }
            if let Some(i) = port.element_ref.rfind("::") {
                let prefix = &port.element_ref[..i];
                if !prefix.is_empty() {
                    if let Some(owner) = graph.nodes.iter().find(|n| n.kind == NodeKind::Block && n.element_ref == prefix) {
                        return owner.id.clone();
                    }
                }
            }
        }
        id.to_string()
    };

    // Nodes that some other node names as `parent:` are containers. Detected
    // structurally so explicit `kind: boundary` is not required.
    let container_ids: HashSet<&str> = graph.nodes.iter().filter_map(|n| n.parent.as_deref()).collect();

    let mut out = String::new();
    out.push_str(&format!("@startuml {}\n", id));
    out.push_str(&style_preamble(cfg));
    out.push('\n');

    // Top-level containers: have children and no parent themselves.
    for n in graph.roots() {
        if n.kind == NodeKind::Port || !container_ids.contains(n.id.as_str()) {
            continue;
        }
        render_ibd_container(n, graph, &container_ids, cfg, 0, &mut out);
    }

    // Standalone components: no parent, no children, not a port.
    for n in graph.roots() {
        if n.kind != NodeKind::Port && !container_ids.contains(n.id.as_str()) {
            let url = element_url(&n.element_ref, cfg);
            out.push_str(&format!("component \"{}\" as {} {}\n", n.label, sanitize_id(&n.id), url));
        }
    }

    out.push('\n');

    // Edges: resolve ports → parent block, skip self-connections
    for e in &graph.edges {
        let src = resolve_to_block(&e.source);
        let tgt = resolve_to_block(&e.target);
        if src == tgt {
            continue;
        }
        let connector = match e.kind {
            EdgeKind::Binding => "..>",
            _ => "-->",
        };
        out.push_str(&format!("{} {} {} : {}\n", sanitize_id(&src), connector, sanitize_id(&tgt), e.kind.as_str()));
    }

    out.push_str("\n@enduml\n");
    out
}

// ── StateMachine ──────────────────────────────────────────────────────────────

fn render_state_machine(graph: &DiagramGraph, id: &str, cfg: Option<&PlantumlConfig>) -> String {
    let initial_ids: HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Initial)
        .map(|n| n.id.as_str())
        .collect();

    let mut out = String::new();
    out.push_str(&format!("@startuml {}\n", id));
    out.push_str(&style_preamble(cfg));
    out.push('\n');

    for n in graph.nodes.iter().filter(|n| n.kind == NodeKind::State) {
        let url = element_url(&n.element_ref, cfg);
        out.push_str(&format!("state \"{}\" as {} {}\n", n.label, sanitize_id(&n.id), url));
    }

    out.push('\n');

    for e in &graph.edges {
        // Prefer explicit label:; fall back to the id stem.
        let label = e.label.clone().unwrap_or_else(|| key_label(&e.id));
        if initial_ids.contains(e.source.as_str()) {
            if label.is_empty() {
                out.push_str(&format!("[*] --> {}\n", sanitize_id(&e.target)));
            } else {
                out.push_str(&format!("[*] --> {} : {}\n", sanitize_id(&e.target), label));
            }
        } else {
            out.push_str(&format!(
                "{} --> {} : {}\n",
                sanitize_id(&e.source),
                sanitize_id(&e.target),
                label
            ));
        }
    }

    out.push_str("\n@enduml\n");
    out
}

// ── Sequence ──────────────────────────────────────────────────────────────────

fn render_sequence(graph: &DiagramGraph, id: &str, cfg: Option<&PlantumlConfig>) -> String {
    let mut out = String::new();
    out.push_str(&format!("@startuml {}\n", id));
    out.push_str(&style_preamble(cfg));
    out.push('\n');

    for n in &graph.nodes {
        let url = element_url(&n.element_ref, cfg);
        match n.kind {
            NodeKind::Actor => out.push_str(&format!("actor \"{}\" as {} {}\n", n.label, sanitize_id(&n.id), url)),
            NodeKind::Lifeline => out.push_str(&format!("participant \"{}\" as {} {}\n", n.label, sanitize_id(&n.id), url)),
            _ => {} // activation, fragment — skipped (REQ-TRS-PUML-023)
        }
    }

    out.push('\n');

    for e in &graph.edges {
        let label = e.label.clone().unwrap_or_else(|| edge_label(&e.id, e.element_ref.as_deref()));
        let arrow = if e.kind == EdgeKind::Return { "-->" } else { "->" };
        out.push_str(&format!(
            "{} {} {} : {}\n",
            sanitize_id(&e.source),
            arrow,
            sanitize_id(&e.target),
            label
        ));
    }

    out.push_str("\n@enduml\n");
    out
}

// ── Requirement diagram ───────────────────────────────────────────────────────

fn requirement_connector(e: &Edge) -> (&'static str, &'static str) {
    match e.kind {
        EdgeKind::Derive => ("..>", "derivedFrom"),
        EdgeKind::Verify => ("..>", "verifies"),
        EdgeKind::Allocation => ("..>", "allocated to"),
        EdgeKind::Satisfy => ("-->", "satisfies"),
        other => ("-->", other.as_str()),
    }
}

fn render_requirement(graph: &DiagramGraph, id: &str, cfg: Option<&PlantumlConfig>) -> String {
    let mut out = String::new();
    out.push_str(&format!("@startuml {}\n", id));
    out.push_str(&style_preamble(cfg));
    out.push_str("hide empty members\n\n");

    for n in &graph.nodes {
        let url = element_url(&n.element_ref, cfg);
        out.push_str(&format!(
            "class \"{}\" as {} <<{}>> {}\n",
            n.label,
            sanitize_id(&n.id),
            class_stereotype(n),
            url
        ));
    }

    out.push('\n');

    for e in &graph.edges {
        let (connector, label) = requirement_connector(e);
        out.push_str(&format!(
            "{} {} {} : {}\n",
            sanitize_id(&e.source),
            connector,
            sanitize_id(&e.target),
            label
        ));
    }

    out.push_str("\n@enduml\n");
    out
}

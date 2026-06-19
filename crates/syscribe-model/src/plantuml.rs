use std::collections::{HashMap, HashSet};

use crate::config::PlantumlConfig;
use crate::element::RawElement;

// ── Shared helpers ────────────────────────────────────────────────────────────

fn lookup_name(qref: &str, elements: &[RawElement]) -> String {
    elements
        .iter()
        .find(|e| e.qualified_name == qref)
        .and_then(|e| e.frontmatter.name.clone())
        .unwrap_or_else(|| short_name(qref))
}

fn short_name(qname: &str) -> String {
    qname.rsplit("::").next().unwrap_or(qname).to_string()
}

/// Replace hyphens with underscores so the string is a valid PlantUML identifier.
fn sanitize_id(s: &str) -> String {
    s.replace('-', "_")
}

fn edge_label(key: &str, eref: Option<&str>) -> String {
    eref.map(short_name)
        .unwrap_or_else(|| key.strip_prefix("e-").unwrap_or(key).to_string())
}

// ── YAML parsing ──────────────────────────────────────────────────────────────

struct Shape {
    key: String,
    qref: String,
    kind: String,
    parent: Option<String>,
}

struct Edge {
    key: String,
    qref: Option<String>,
    source: String,
    target: String,
    kind: String,
}

fn parse_shapes(val: &serde_yaml::Value) -> Vec<Shape> {
    let mut out = Vec::new();
    let Some(map) = val.as_mapping() else { return out };
    for (k, v) in map {
        let key = k.as_str().unwrap_or("").to_string();
        let qref = v.get("ref").and_then(|x| x.as_str()).unwrap_or("").to_string();
        // Normalise to lowercase so renderers use simple equality checks regardless
        // of whether the diagram uses "block"/"port" or "Part"/"Port" etc.
        let kind = v.get("kind").and_then(|x| x.as_str()).unwrap_or("").to_lowercase();
        let parent = v.get("parent").and_then(|x| x.as_str()).map(str::to_string);
        out.push(Shape { key, qref, kind, parent });
    }
    out
}

fn parse_edges(val: &serde_yaml::Value) -> Vec<Edge> {
    let mut out = Vec::new();
    let Some(map) = val.as_mapping() else { return out };
    for (k, v) in map {
        let key = k.as_str().unwrap_or("").to_string();
        let qref = v.get("ref").and_then(|x| x.as_str()).map(str::to_string);
        let source = v.get("source").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let target = v.get("target").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let kind = v.get("kind").and_then(|x| x.as_str()).unwrap_or("").to_string();
        out.push(Edge { key, qref, source, target, kind });
    }
    out
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Generate a PlantUML `.puml` source string from a `Diagram` element.
/// Returns `None` when the `diagramKind` has no PlantUML mapping (e.g. `Mermaid`).
pub fn render_plantuml(
    element: &RawElement,
    elements: &[RawElement],
    cfg: Option<&PlantumlConfig>,
) -> Option<String> {
    let kind = element.frontmatter.diagram_kind.as_deref()?;

    // Display label used inside the diagram (may contain spaces/punctuation).
    let name = element
        .frontmatter
        .name
        .as_deref()
        .unwrap_or_else(|| element.qualified_name.rsplit("::").next().unwrap_or(&element.qualified_name));

    // @startuml identifier / PlantUML output filename stem — must not contain
    // spaces so PlantUML names the .svg predictably.  Derive from the file stem.
    let file_stem: String = std::path::Path::new(&element.file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name)
        .replace(' ', "_");

    match kind {
        "BDD" => Some(render_bdd(element, elements, name, &file_stem, cfg)),
        "IBD" => Some(render_ibd(element, elements, name, &file_stem, cfg)),
        "StateMachine" => Some(render_state_machine(element, elements, name, &file_stem, cfg)),
        "Sequence" => Some(render_sequence(element, elements, name, &file_stem, cfg)),
        "Requirement" => Some(render_requirement(element, elements, name, &file_stem, cfg)),
        _ => None,
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

/// Emit the style preamble: `!include`, `!theme`, or built-in skinparams.
fn style_preamble(cfg: Option<&PlantumlConfig>, diagram_kind: &str) -> String {
    if let Some(c) = cfg {
        if let Some(ref sf) = c.style_file {
            return format!("!include {}\n", sf.display());
        }
        if let Some(ref theme) = c.theme {
            return format!("!theme {}\n", theme);
        }
    }
    // Built-in defaults vary by diagram kind
    match diagram_kind {
        "Requirement" => {
            "skinparam class {\n  BackgroundColor #f9f7ff\n  BorderColor #4a0a6e\n}\n".to_string()
        }
        "IBD" => {
            "skinparam component {\n  BackgroundColor #f5f5fa\n  BorderColor #3a3a4a\n}\n"
                .to_string()
        }
        "StateMachine" => {
            "skinparam state {\n  BackgroundColor #f5f5fa\n  BorderColor #3a3a4a\n}\n".to_string()
        }
        "Sequence" => {
            "skinparam participant {\n  BackgroundColor #f5f5fa\n  BorderColor #3a3a4a\n}\n"
                .to_string()
        }
        _ => {
            "skinparam class {\n  BackgroundColor #f5f5fa\n  BorderColor #3a3a4a\n}\n".to_string()
        }
    }
}

// ── BDD ───────────────────────────────────────────────────────────────────────

fn render_bdd(element: &RawElement, elements: &[RawElement], _name: &str, id: &str, cfg: Option<&PlantumlConfig>) -> String {
    let shapes = element.frontmatter.shapes.as_ref().map(|v| parse_shapes(v)).unwrap_or_default();
    let edges = element.frontmatter.edges.as_ref().map(|v| parse_edges(v)).unwrap_or_default();

    let mut out = String::new();
    out.push_str(&format!("@startuml {}\n", id));
    out.push_str(&style_preamble(cfg, "BDD"));
    out.push_str("hide empty members\n\n");

    for s in &shapes {
        let display = lookup_name(&s.qref, elements);
        let stereo = match s.kind.as_str() {
            "partdef" => "part def",
            "part" | "block" => "part",
            other => other,
        };
        let url = element_url(&s.qref, cfg);
        out.push_str(&format!("class \"{}\" as {} <<{}>> {}\n", display, sanitize_id(&s.key), stereo, url));
    }

    out.push('\n');

    for e in &edges {
        // For BDD edges the ref often points to the owning element rather than
        // the member, so the edge key (e.g. "e-propulsion" → "propulsion") is
        // the most reliable role label.
        let label = e.key.strip_prefix("e-").unwrap_or(&e.key).to_string();
        let connector = match e.kind.as_str() {
            "composition" => "*--",
            "usage" => "..>",
            "generalization" | "specialization" | "inheritance" => "--|>",
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

fn render_ibd(element: &RawElement, elements: &[RawElement], _name: &str, id: &str, cfg: Option<&PlantumlConfig>) -> String {
    let shapes = element.frontmatter.shapes.as_ref().map(|v| parse_shapes(v)).unwrap_or_default();
    let edges = element.frontmatter.edges.as_ref().map(|v| parse_edges(v)).unwrap_or_default();

    // Map shape key → parent key (for ports resolving to their parent block)
    let parent_map: HashMap<&str, &str> = shapes
        .iter()
        .filter_map(|s| s.parent.as_deref().map(|p| (s.key.as_str(), p)))
        .collect();

    // Resolve a shape key through its parent chain to the nearest block/boundary.
    // If the port has no explicit `parent:`, fall back to finding a block/part whose
    // qref is a proper prefix of the port's qref (i.e. the port's owning element).
    let resolve_to_block = |id: &str| -> String {
        let shape = shapes.iter().find(|s| s.key == id);
        let kind = shape.map(|s| s.kind.as_str()).unwrap_or("");
        if kind == "port" {
            if let Some(parent_key) = parent_map.get(id) {
                return parent_key.to_string();
            }
            // Fallback: find a block whose qref is a prefix of this port's qref
            if let Some(port_qref) = shape.map(|s| s.qref.as_str()) {
                let prefix = match port_qref.rfind("::") {
                    Some(i) => &port_qref[..i],
                    None => "",
                };
                if !prefix.is_empty() {
                    if let Some(owner) = shapes.iter().find(|s| {
                        (s.kind == "block" || s.kind == "part") && s.qref == prefix
                    }) {
                        return owner.key.clone();
                    }
                }
            }
        }
        id.to_string()
    };

    // Group block/part shapes by their parent boundary key
    let mut blocks_by_boundary: HashMap<&str, Vec<&Shape>> = HashMap::new();
    for s in &shapes {
        if s.kind == "block" || s.kind == "part" {
            if let Some(p) = s.parent.as_deref() {
                blocks_by_boundary.entry(p).or_default().push(s);
            }
        }
    }

    let mut out = String::new();
    out.push_str(&format!("@startuml {}\n", id));
    out.push_str(&style_preamble(cfg, "IBD"));
    out.push('\n');

    // Boundaries with their nested blocks
    for s in &shapes {
        if s.kind != "boundary" {
            continue;
        }
        let bname = lookup_name(&s.qref, elements);
        let url = element_url(&s.qref, cfg);
        out.push_str(&format!("rectangle \"{}\" as {} {} {{\n", bname, sanitize_id(&s.key), url));
        if let Some(children) = blocks_by_boundary.get(s.key.as_str()) {
            for c in children {
                let cname = lookup_name(&c.qref, elements);
                let curl = element_url(&c.qref, cfg);
                out.push_str(&format!("  component \"{}\" as {} {}\n", cname, sanitize_id(&c.key), curl));
            }
        }
        out.push_str("}\n");
    }

    // Top-level blocks (no parent)
    for s in &shapes {
        if (s.kind == "block" || s.kind == "part") && s.parent.is_none() {
            let cname = lookup_name(&s.qref, elements);
            let url = element_url(&s.qref, cfg);
            out.push_str(&format!("component \"{}\" as {} {}\n", cname, sanitize_id(&s.key), url));
        }
    }

    out.push('\n');

    // Edges: resolve ports → parent block, skip self-connections
    for e in &edges {
        let src = resolve_to_block(&e.source);
        let tgt = resolve_to_block(&e.target);
        if src == tgt {
            continue;
        }
        let connector = match e.kind.as_str() {
            "binding" => "..>",
            _ => "-->",
        };
        out.push_str(&format!("{} {} {} : {}\n", sanitize_id(&src), connector, sanitize_id(&tgt), &e.kind));
    }

    out.push_str("\n@enduml\n");
    out
}

// ── StateMachine ──────────────────────────────────────────────────────────────

fn render_state_machine(element: &RawElement, elements: &[RawElement], _name: &str, id: &str, cfg: Option<&PlantumlConfig>) -> String {
    let shapes = element.frontmatter.shapes.as_ref().map(|v| parse_shapes(v)).unwrap_or_default();
    let edges = element.frontmatter.edges.as_ref().map(|v| parse_edges(v)).unwrap_or_default();

    let initial_keys: HashSet<&str> = shapes
        .iter()
        .filter(|s| s.kind == "initial")
        .map(|s| s.key.as_str())
        .collect();

    let mut out = String::new();
    out.push_str(&format!("@startuml {}\n", id));
    out.push_str(&style_preamble(cfg, "StateMachine"));
    out.push('\n');

    for s in &shapes {
        if s.kind == "state" {
            let sname = lookup_name(&s.qref, elements);
            let url = element_url(&s.qref, cfg);
            out.push_str(&format!("state \"{}\" as {} {}\n", sname, sanitize_id(&s.key), url));
        }
    }

    out.push('\n');

    for e in &edges {
        let label = e.key.strip_prefix("e-").unwrap_or(&e.key);
        if initial_keys.contains(e.source.as_str()) {
            out.push_str(&format!("[*] --> {} : {}\n", sanitize_id(&e.target), label));
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

fn render_sequence(element: &RawElement, elements: &[RawElement], _name: &str, id: &str, cfg: Option<&PlantumlConfig>) -> String {
    let shapes = element.frontmatter.shapes.as_ref().map(|v| parse_shapes(v)).unwrap_or_default();
    let edges = element.frontmatter.edges.as_ref().map(|v| parse_edges(v)).unwrap_or_default();

    let mut out = String::new();
    out.push_str(&format!("@startuml {}\n", id));
    out.push_str(&style_preamble(cfg, "Sequence"));
    out.push('\n');

    for s in &shapes {
        let pname = lookup_name(&s.qref, elements);
        let url = element_url(&s.qref, cfg);
        match s.kind.as_str() {
            "actor" => out.push_str(&format!("actor \"{}\" as {} {}\n", pname, sanitize_id(&s.key), url)),
            "lifeline" => out.push_str(&format!("participant \"{}\" as {} {}\n", pname, sanitize_id(&s.key), url)),
            _ => {} // activation, fragment — skipped (REQ-TRS-PUML-023)
        }
    }

    out.push('\n');

    for e in &edges {
        let label = edge_label(&e.key, e.qref.as_deref());
        let arrow = if e.kind == "return" { "-->" } else { "->" };
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

fn render_requirement(element: &RawElement, elements: &[RawElement], _name: &str, id: &str, cfg: Option<&PlantumlConfig>) -> String {
    let shapes = element.frontmatter.shapes.as_ref().map(|v| parse_shapes(v)).unwrap_or_default();
    let edges = element.frontmatter.edges.as_ref().map(|v| parse_edges(v)).unwrap_or_default();

    let mut out = String::new();
    out.push_str(&format!("@startuml {}\n", id));
    out.push_str(&style_preamble(cfg, "Requirement"));
    out.push_str("hide empty members\n\n");

    for s in &shapes {
        let sname = lookup_name(&s.qref, elements);
        let stereo = match s.kind.as_str() {
            "requirement" => "requirement",
            "requirementdef" => "requirement def",
            "testcase" => "test case",
            "testcasedef" => "test case def",
            "partdef" | "itemdef" => "part def",
            "part" | "item" | "block" => "part",
            "actiondef" => "action def",
            "action" => "action",
            other => other,
        };
        let url = element_url(&s.qref, cfg);
        out.push_str(&format!("class \"{}\" as {} <<{}>> {}\n", sname, sanitize_id(&s.key), stereo, url));
    }

    out.push('\n');

    for e in &edges {
        let (connector, label) = match e.kind.as_str() {
            "derivedFrom" => ("..>", "derivedFrom"),
            "verifies" => ("..>", "verifies"),
            "allocatedTo" => ("..>", "allocated to"),
            other => ("-->", other),
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

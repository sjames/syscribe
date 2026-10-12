//! Flow kinds in diagrams (GH #335): the `[diagram]` table of `.syscribe.toml`, which restyles the flow kinds and
//! places ports by what they carry, and [`apply`], which puts it on a built graph.
//!
//! ```toml
//! [diagram.styles.power]          # each key replaces the built-in value of that flow kind
//! stroke = "#b35900"
//! width = 3
//! dash = ""                       # "" is a solid line
//!
//! [diagram.portSides]             # optional rule by flow kind; a port's own `side:` wins
//! power = "direction"             # in west, out east (today's rule)
//! signal = "south"
//! ```
//!
//! Named styles (GH #338) are selected per diagram by `style: <name>`:
//!
//! ```toml
//! [diagram.style.focus]           # every named style starts from `default`, when that is defined
//! context = "fade"                # what lies outside a view's `include:`: "fade" (default) or "hide"
//!
//! [diagram.style.safety]
//! extends = "focus"               # a chain of styles, nearest last; a cycle is W419
//! [diagram.style.safety.flows.protection]
//! width = 4                       # the keys of [diagram.styles.<kind>], over the project's
//! ```
//!
//! Malformed entries are dropped and reported as `W419`. Without the table every diagram draws as before.

use std::collections::BTreeMap;
use std::path::Path;

use super::derive::{features_of, FeatureRole};
use super::ir::{DiagramGraph, FlowKind, FlowStyle, NodeKind, Side};
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

/// Where a rule puts the ports of a flow kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortSideRule {
    /// The direction rule: in west, out east, inout south.
    Direction,
    Fixed(Side),
}

/// What a view draws of what lies outside its `include:` (GH #338).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Context {
    #[default]
    Fade,
    Hide,
}

/// One `[diagram.style.<name>]` table (GH #338).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NamedStyle {
    pub extends: Option<String>,
    pub context: Option<Context>,
    pub flows: BTreeMap<FlowKind, FlowStyle>,
}

/// A named style with its `extends` chain (and `default`) folded in.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedStyle {
    pub context: Context,
    pub flows: BTreeMap<FlowKind, FlowStyle>,
}

/// The `[diagram]` table.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagramConfig {
    pub styles: BTreeMap<FlowKind, FlowStyle>,
    pub port_sides: BTreeMap<FlowKind, PortSideRule>,
    /// `[diagram.style.<name>]` (GH #338).
    pub named: BTreeMap<String, NamedStyle>,
    /// The `W419` messages.
    pub problems: Vec<String>,
}

/// `#rgb`, `#rrggbb` or a colour name: what every writer (SVG, DOT, Mermaid, PlantUML) accepts unescaped.
fn is_color(s: &str) -> bool {
    match s.strip_prefix('#') {
        Some(hex) => matches!(hex.len(), 3 | 6) && hex.chars().all(|c| c.is_ascii_hexdigit()),
        None => !s.is_empty() && s.chars().all(|c| c.is_ascii_alphabetic()),
    }
}

/// A `stroke-dasharray`: non-negative numbers separated by commas or spaces, or `""` (solid).
fn is_dash(s: &str) -> bool {
    s.split([',', ' ']).filter(|t| !t.is_empty()).all(|t| t.parse::<f64>().is_ok_and(|v| v >= 0.0))
}

fn kinds() -> String {
    FlowKind::ALL.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
}

impl DiagramConfig {
    pub fn load(model_root: &Path) -> Self {
        match std::fs::read_to_string(model_root.join(".syscribe.toml")) {
            Ok(text) => Self::from_toml_str(&text),
            Err(_) => Self::default(),
        }
    }

    pub fn from_toml_str(text: &str) -> Self {
        let mut c = Self::default();
        let Ok(root) = text.parse::<toml::Table>() else { return c };
        let Some(d) = root.get("diagram") else { return c };
        let Some(d) = d.as_table() else {
            c.problems.push("[diagram] must be a table — ignored".into());
            return c;
        };
        for (k, v) in d {
            match k.as_str() {
                "styles" => {
                    let mut flows = BTreeMap::new();
                    c.flows_from(v, "diagram.styles", &mut flows);
                    c.styles = flows;
                }
                "portSides" | "port_sides" => c.sides_from(v),
                "style" => c.named_from(v),
                other => c.problems.push(format!("[diagram] has an unknown key '{other}' (known: styles, portSides, style) — ignored")),
            }
        }
        c.check_chains();
        c
    }

    fn named_from(&mut self, v: &toml::Value) {
        let Some(t) = v.as_table() else {
            self.problems.push("[diagram.style] must be a table of named styles — ignored".into());
            return;
        };
        for (name, v) in t {
            let Some(e) = v.as_table() else {
                self.problems.push(format!("[diagram.style.{name}] must be a table — ignored"));
                continue;
            };
            let mut st = NamedStyle::default();
            for (key, val) in e {
                match (key.as_str(), val) {
                    ("extends", toml::Value::String(s)) => st.extends = Some(s.clone()),
                    ("context", toml::Value::String(s)) if s == "fade" => st.context = Some(Context::Fade),
                    ("context", toml::Value::String(s)) if s == "hide" => st.context = Some(Context::Hide),
                    ("context", _) => self.problems.push(format!("[diagram.style.{name}] context must be \"fade\" or \"hide\" — ignored")),
                    ("flows", v) => {
                        let mut flows = BTreeMap::new();
                        self.flows_from(v, &format!("diagram.style.{name}.flows"), &mut flows);
                        st.flows = flows;
                    }
                    ("extends", _) => self.problems.push(format!("[diagram.style.{name}] extends must be a style name — ignored")),
                    _ => self.problems.push(format!("[diagram.style.{name}] has an unknown key '{key}' (known: extends, context, flows) — ignored")),
                }
            }
            self.named.insert(name.clone(), st);
        }
    }

    /// `extends` naming no style, or a cycle, is `W419`; the link is dropped.
    fn check_chains(&mut self) {
        let names: Vec<String> = self.named.keys().cloned().collect();
        for name in names {
            let Some(parent) = self.named[&name].extends.clone() else { continue };
            // the built-in `default` may be extended even when the project defines none (it is then empty)
            if parent == "default" && !self.named.contains_key("default") {
                self.named.get_mut(&name).unwrap().extends = None;
                continue;
            }
            if !self.named.contains_key(&parent) {
                self.problems.push(format!("[diagram.style.{name}] extends '{parent}', which is not a style — ignored"));
                self.named.get_mut(&name).unwrap().extends = None;
                continue;
            }
            // only a style on a cycle is blamed: one that merely leads into a cycle elsewhere keeps its link
            let mut seen = vec![name.clone()];
            let mut cur = Some(parent);
            while let Some(c) = cur {
                if c == name {
                    self.problems.push(format!("[diagram.style.{name}] extends a chain that comes back to itself — the link is ignored"));
                    self.named.get_mut(&name).unwrap().extends = None;
                    break;
                }
                if seen.contains(&c) {
                    break;
                }
                seen.push(c.clone());
                cur = self.named.get(&c).and_then(|s| s.extends.clone());
            }
        }
    }

    /// Whether `style:` can name `name`: a defined style, or the built-in `default`.
    pub fn has_style(&self, name: &str) -> bool {
        name == "default" || self.named.contains_key(name)
    }

    /// The style a diagram draws with: `default` (when defined), then the chain of `name` from its root, nearest
    /// last; the flow looks start from the project's `[diagram.styles]`.
    pub fn resolve(&self, name: Option<&str>) -> ResolvedStyle {
        let mut chain: Vec<&NamedStyle> = Vec::new();
        let mut cur = name.filter(|n| *n != "default").and_then(|n| self.named.get(n));
        while let Some(st) = cur {
            if chain.iter().any(|c| std::ptr::eq(*c, st)) {
                break;
            }
            chain.push(st);
            cur = st.extends.as_deref().filter(|n| *n != "default").and_then(|n| self.named.get(n));
        }
        chain.extend(self.named.get("default"));
        let mut out = ResolvedStyle { context: Context::Fade, flows: self.styles.clone() };
        for st in chain.iter().rev() {
            if let Some(c) = st.context {
                out.context = c;
            }
            for (k, f) in &st.flows {
                let e = out.flows.entry(*k).or_default();
                if f.stroke.is_some() {
                    e.stroke = f.stroke.clone();
                }
                if f.width.is_some() {
                    e.width = f.width;
                }
                if f.dash.is_some() {
                    e.dash = f.dash.clone();
                }
            }
        }
        out
    }

    fn flows_from(&mut self, v: &toml::Value, at: &str, out: &mut BTreeMap<FlowKind, FlowStyle>) {
        let Some(t) = v.as_table() else {
            self.problems.push(format!("[{at}] must be a table of flow kinds — ignored"));
            return;
        };
        for (k, v) in t {
            let Some(kind) = FlowKind::parse(k) else {
                self.problems.push(format!("[{at}.{k}]: '{k}' is not a flow kind (the kinds are: {}) — ignored", kinds()));
                continue;
            };
            let Some(e) = v.as_table() else {
                self.problems.push(format!("[{at}.{k}] must be a table — ignored"));
                continue;
            };
            let mut st = FlowStyle::default();
            for (key, val) in e {
                match (key.as_str(), val) {
                    ("stroke" | "color", toml::Value::String(s)) if is_color(s) => st.stroke = Some(s.clone()),
                    ("stroke" | "color", toml::Value::String(s)) => {
                        self.problems.push(format!("[{at}.{k}] {key} '{s}' is not a colour (#rgb, #rrggbb or a colour name) — ignored"))
                    }
                    ("dash", toml::Value::String(s)) if !is_dash(s) => {
                        self.problems.push(format!("[{at}.{k}] dash '{s}' is not a dash pattern (numbers such as \"6,3\", or \"\" for solid) — ignored"))
                    }
                    ("width", toml::Value::Integer(i)) if *i > 0 => st.width = Some(*i as f64),
                    ("width", toml::Value::Float(f)) if *f > 0.0 => st.width = Some(*f),
                    ("dash", toml::Value::String(s)) => st.dash = Some(s.clone()),
                    ("stroke" | "color" | "width" | "dash", _) => {
                        self.problems.push(format!("[{at}.{k}] {key} has the wrong type (stroke and dash are strings, width a positive number) — ignored"))
                    }
                    _ => self.problems.push(format!("[{at}.{k}] has an unknown key '{key}' (known: stroke, width, dash) — ignored")),
                }
            }
            out.insert(kind, st);
        }
    }

    fn sides_from(&mut self, v: &toml::Value) {
        let Some(t) = v.as_table() else {
            self.problems.push("[diagram.portSides] must be a table of flow kinds — ignored".into());
            return;
        };
        for (k, v) in t {
            let Some(kind) = FlowKind::parse(k) else {
                self.problems.push(format!("[diagram.portSides] '{k}' is not a flow kind (the kinds are: {}) — ignored", kinds()));
                continue;
            };
            let rule = match v.as_str() {
                Some("direction") => PortSideRule::Direction,
                Some(s) => match Side::parse(s) {
                    Some(side) => PortSideRule::Fixed(side),
                    None => {
                        self.problems.push(format!("[diagram.portSides] {k} = '{s}' is not a side (north, east, south, west or direction) — ignored"));
                        continue;
                    }
                },
                None => {
                    self.problems.push(format!("[diagram.portSides] {k} must be a string — ignored"));
                    continue;
                }
            };
            self.port_sides.insert(kind, rule);
        }
    }
}

/// Put the project's `[diagram]` table on a built graph: its styles for the writers, and, under a port-side rule, the
/// side of each port that has none of its own. A port's flow kind is its own (`flowKind:` on the port or its
/// `PortDef`), else that of the first edge that joins it.
pub fn apply(graph: &mut DiagramGraph, cfg: &DiagramConfig) {
    // the diagram's named style (GH #338): its flow looks, and whether what lies outside the view is drawn
    let style = cfg.resolve(graph.style.as_deref());
    graph.flow_styles = style.flows;
    if style.context == Context::Hide {
        hide_context(graph);
    }
    if cfg.port_sides.is_empty() {
        return;
    }
    let mut sides: Vec<(usize, Side)> = Vec::new();
    for (i, n) in graph.nodes.iter().enumerate() {
        if n.kind != NodeKind::Port || n.side.is_some() {
            continue;
        }
        let flow = n.flow.or_else(|| graph.edges.iter().find(|e| e.source == n.id || e.target == n.id).and_then(|e| e.flow));
        if let Some(PortSideRule::Fixed(side)) = flow.and_then(|f| cfg.port_sides.get(&f)) {
            sides.push((i, *side));
        }
    }
    for (i, side) in sides {
        graph.nodes[i].side = Some(side);
    }
}

/// Drop the faded nodes (with everything inside them) and the faded edges, and the edges left without an end.
pub fn hide_context(graph: &mut DiagramGraph) {
    let mut gone: std::collections::HashSet<String> = graph.nodes.iter().filter(|n| n.faded).map(|n| n.id.clone()).collect();
    loop {
        let more: Vec<String> =
            graph.nodes.iter().filter(|n| !gone.contains(&n.id) && n.parent.as_ref().is_some_and(|p| gone.contains(p))).map(|n| n.id.clone()).collect();
        if more.is_empty() {
            break;
        }
        gone.extend(more);
    }
    graph.nodes.retain(|n| !gone.contains(&n.id));
    graph.edges.retain(|e| !e.faded && !gone.contains(&e.source) && !gone.contains(&e.target));
}

/// `W419` findings on the elements (GH #335): a `flowKind:` or `side:` that names no flow kind or side, or that
/// sits where nothing reads it. Read are: `flowKind:` on a `PortDef` or `Port`, on a port feature, on a
/// `connections:`/`flowConnections:` entry and on a Diagram `edges:` entry; `side:` on a port feature. Every value
/// flagged is ignored, so the edge or port draws as if it were absent. Returns `(file, message)` pairs.
pub fn element_problems(elements: &[RawElement], resolver: &Resolver) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let sides = "north, south, east, west";
    let bad_kind = |k: &str| format!("flowKind '{k}' is not a flow kind (the kinds are: {}) — ignored", kinds());
    for e in elements {
        let fm = &e.frontmatter;
        let who = fm.name.clone().unwrap_or_else(|| e.qualified_name.clone());
        let mut push = |msg: String| out.push((e.file_path.clone(), format!("'{who}': {msg}")));
        if let Some(k) = fm.flow_kind.as_deref() {
            if FlowKind::parse(k).is_none() {
                push(bad_kind(k));
            } else if !matches!(fm.element_type, Some(ElementType::PortDef) | Some(ElementType::Port)) {
                push(format!("flowKind is read only on a PortDef, a Port, a port feature or a connection — ignored on a {}", fm.element_type.as_ref().map(|t| t.name()).unwrap_or("element")));
            }
        }
        let get = |m: &serde_yaml::Mapping, k: &str| m.get(serde_yaml::Value::String(k.into())).and_then(|v| v.as_str()).map(str::to_string);

        // features: only a port reads `flowKind:`/`side:`
        let ports: Vec<String> = features_of(e, elements, resolver).into_iter().filter(|f| f.role == FeatureRole::Port).map(|f| f.name).collect();
        for v in fm.features.iter().flatten() {
            let serde_yaml::Value::Mapping(m) = v else { continue };
            let name = get(m, "name").unwrap_or_default();
            let at = format!("features entry '{name}'");
            let is_port = ports.contains(&name);
            if let Some(k) = get(m, "flowKind") {
                if FlowKind::parse(&k).is_none() {
                    push(format!("{at}: {}", bad_kind(&k)));
                } else if !is_port {
                    push(format!("{at}: flowKind is read only on a port — ignored"));
                }
            }
            if let Some(sd) = get(m, "side") {
                if Side::parse(&sd).is_none() {
                    push(format!("{at}: side '{sd}' is not a side ({sides}) — ignored"));
                } else if !is_port {
                    push(format!("{at}: side is read only on a port — ignored"));
                }
            }
        }

        // connection lists: a connection or flow carries a kind; a binding or succession carries nothing
        for (field, list, carries) in [
            ("connections", &fm.connections, true),
            ("flowConnections", &fm.flow_connections, true),
            ("bindingConnections", &fm.binding_connections, false),
            ("successionConnections", &fm.succession_connections, false),
        ] {
            for v in list.iter().flatten() {
                let serde_yaml::Value::Mapping(m) = v else { continue };
                let ends = ["from", "to", "left", "right", "after", "before"].iter().filter_map(|k| get(m, k)).collect::<Vec<_>>();
                let at = match get(m, "name") {
                    Some(n) => format!("{field} entry '{n}'"),
                    None if ends.len() >= 2 => format!("{field} entry '{} → {}'", ends[0], ends[1]),
                    None => format!("a {field} entry"),
                };
                if let Some(k) = get(m, "flowKind") {
                    if FlowKind::parse(&k).is_none() {
                        push(format!("{at}: {}", bad_kind(&k)));
                    } else if !carries {
                        push(format!("{at}: flowKind is read only on connections and flowConnections — ignored"));
                    }
                }
                if get(m, "side").is_some() {
                    push(format!("{at}: side is read only on a port feature — ignored"));
                }
            }
        }

        // a manifest diagram's `edges:` map
        if let Some(serde_yaml::Value::Mapping(edges)) = fm.edges.as_ref() {
            for (id, v) in edges {
                let serde_yaml::Value::Mapping(m) = v else { continue };
                if let Some(k) = get(m, "flowKind") {
                    if FlowKind::parse(&k).is_none() {
                        push(format!("edges entry '{}': {}", id.as_str().unwrap_or("?"), bad_kind(&k)));
                    }
                }
            }
        }
    }
    out
}

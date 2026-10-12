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
//! Malformed entries are dropped and reported as `W419`. Without the table every diagram draws as before.

use std::collections::BTreeMap;
use std::path::Path;

use super::ir::{DiagramGraph, FlowKind, FlowStyle, NodeKind, Side};

/// Where a rule puts the ports of a flow kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortSideRule {
    /// The direction rule: in west, out east, inout south.
    Direction,
    Fixed(Side),
}

/// The `[diagram]` table.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagramConfig {
    pub styles: BTreeMap<FlowKind, FlowStyle>,
    pub port_sides: BTreeMap<FlowKind, PortSideRule>,
    /// The `W419` messages.
    pub problems: Vec<String>,
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
                "styles" => c.styles_from(v),
                "portSides" | "port_sides" => c.sides_from(v),
                other => c.problems.push(format!("[diagram] has an unknown key '{other}' (known: styles, portSides) — ignored")),
            }
        }
        c
    }

    fn styles_from(&mut self, v: &toml::Value) {
        let Some(t) = v.as_table() else {
            self.problems.push("[diagram.styles] must be a table of flow kinds — ignored".into());
            return;
        };
        for (k, v) in t {
            let Some(kind) = FlowKind::parse(k) else {
                self.problems.push(format!("[diagram.styles.{k}]: '{k}' is not a flow kind (the kinds are: {}) — ignored", kinds()));
                continue;
            };
            let Some(e) = v.as_table() else {
                self.problems.push(format!("[diagram.styles.{k}] must be a table — ignored"));
                continue;
            };
            let mut st = FlowStyle::default();
            for (key, val) in e {
                match (key.as_str(), val) {
                    ("stroke" | "color", toml::Value::String(s)) => st.stroke = Some(s.clone()),
                    ("width", toml::Value::Integer(i)) if *i > 0 => st.width = Some(*i as f64),
                    ("width", toml::Value::Float(f)) if *f > 0.0 => st.width = Some(*f),
                    ("dash", toml::Value::String(s)) => st.dash = Some(s.clone()),
                    ("stroke" | "color" | "width" | "dash", _) => {
                        self.problems.push(format!("[diagram.styles.{k}] {key} has the wrong type (stroke and dash are strings, width a positive number) — ignored"))
                    }
                    _ => self.problems.push(format!("[diagram.styles.{k}] has an unknown key '{key}' (known: stroke, width, dash) — ignored")),
                }
            }
            self.styles.insert(kind, st);
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
    graph.flow_styles = cfg.styles.clone();
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

/// `W419` findings on the elements (GH #335): a `flowKind:` or `side:` that names no flow kind or side, on an
/// element, a `features:` entry or a `connections:` entry. The value is ignored, so the edge or port draws as if
/// it were absent. Returns `(file, message)` pairs.
pub fn element_problems(elements: &[crate::element::RawElement]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let sides = "north, south, east, west";
    for e in elements {
        let fm = &e.frontmatter;
        let who = fm.name.clone().unwrap_or_else(|| e.qualified_name.clone());
        if let Some(k) = fm.flow_kind.as_deref() {
            if FlowKind::parse(k).is_none() {
                out.push((e.file_path.clone(), format!("'{who}': flowKind '{k}' is not a flow kind (the kinds are: {}) — ignored", kinds())));
            }
        }
        let entries = |list: &Option<Vec<serde_yaml::Value>>| list.clone().unwrap_or_default();
        for (field, list) in [("features", entries(&fm.features)), ("connections", entries(&fm.connections))] {
            for v in &list {
                let serde_yaml::Value::Mapping(m) = v else { continue };
                let get = |k: &str| m.get(serde_yaml::Value::String(k.into())).and_then(|v| v.as_str()).map(str::to_string);
                let label = get("name").or_else(|| match (get("from"), get("to")) {
                    (Some(f), Some(t)) => Some(format!("{f} → {t}")),
                    _ => None,
                });
                let at = label.map(|l| format!("{field} entry '{l}'")).unwrap_or_else(|| format!("a {field} entry"));
                if let Some(k) = get("flowKind") {
                    if FlowKind::parse(&k).is_none() {
                        out.push((e.file_path.clone(), format!("'{who}': {at}: flowKind '{k}' is not a flow kind (the kinds are: {}) — ignored", kinds())));
                    }
                }
                if field == "features" {
                    if let Some(s) = get("side") {
                        if Side::parse(&s).is_none() {
                            out.push((e.file_path.clone(), format!("'{who}': {at}: side '{s}' is not a side ({sides}) — ignored")));
                        }
                    }
                }
            }
        }
    }
    out
}

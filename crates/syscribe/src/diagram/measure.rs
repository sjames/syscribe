use std::collections::HashMap;

use serde::Serialize;
use syscribe_model::element::RawElement;

use crate::diagram::layout::{
    build_element_node, load_metrics, render_element, ViewConfig,
};
use crate::diagram::layout::types::PortAnchor;

#[derive(Serialize)]
struct MeasureEntry {
    qname: String,
    width: f64,
    height: f64,
    effective_view: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    effective_include: HashMap<String, Vec<String>>,
    port_anchors: Vec<PortAnchor>,
    peers: Vec<PeerEntry>,
}

#[derive(Serialize)]
struct PeerEntry {
    port: String,
    peer_qname: String,
    peer_port: String,
}

pub fn cmd_diagram_measure(elements: &[RawElement], qnames_arg: &str, view: ViewConfig) {
    let metrics = load_metrics();

    let qnames: Vec<&str> = qnames_arg.split(',').map(|s| s.trim()).collect();
    let mut results: Vec<MeasureEntry> = Vec::new();

    for qname in &qnames {
        let elem = match elements.iter().find(|e| e.qualified_name == *qname) {
            Some(e) => e,
            None => {
                eprintln!("warn: element '{}' not found", qname);
                continue;
            }
        };

        let node = build_element_node(elem, &view);
        let rendered = render_element(&node, metrics.as_ref());
        let peers = gather_peers(elem, elements);

        results.push(MeasureEntry {
            qname: rendered.qualified_name.clone(),
            width: rendered.width,
            height: rendered.height,
            effective_view: view.preset.label().to_string(),
            effective_include: view.include.to_map(),
            port_anchors: rendered.port_anchors,
            peers,
        });
    }

    match serde_json::to_string_pretty(&results) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("error serializing measure output: {}", e),
    }
}

fn gather_peers(elem: &RawElement, _all: &[RawElement]) -> Vec<PeerEntry> {
    let mut peers = Vec::new();
    if let Some(connections) = &elem.frontmatter.connections {
        for conn in connections {
            if let serde_yaml::Value::Mapping(map) = conn {
                let from = map
                    .get(serde_yaml::Value::String("from".into()))
                    .and_then(|v| v.as_str());
                let to = map
                    .get(serde_yaml::Value::String("to".into()))
                    .and_then(|v| v.as_str());

                if let (Some(from_str), Some(to_str)) = (from, to) {
                    if let (Some((from_elem, from_port)), Some((to_elem, to_port))) =
                        (split_port_ref(from_str), split_port_ref(to_str))
                    {
                        let source_qname = qualify(&elem.qualified_name, from_elem);
                        let target_qname = qualify(&elem.qualified_name, to_elem);
                        peers.push(PeerEntry {
                            port: from_port.to_string(),
                            peer_qname: target_qname,
                            peer_port: to_port.to_string(),
                        });
                        peers.push(PeerEntry {
                            port: to_port.to_string(),
                            peer_qname: source_qname,
                            peer_port: from_port.to_string(),
                        });
                    }
                }
            }
        }
    }
    peers
}

fn split_port_ref(s: &str) -> Option<(&str, &str)> {
    let pos = s.rfind("::")?;
    Some((&s[..pos], &s[pos + 2..]))
}

fn qualify(parent: &str, sub: &str) -> String {
    if sub.contains("::") { sub.to_string() } else { format!("{}::{}", parent, sub) }
}

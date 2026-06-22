use syscribe_model::element::{ElementType, RawElement};
use serde::Serialize;

#[derive(Serialize)]
struct ListEntry<'a> {
    qname: &'a str,
    #[serde(rename = "type")]
    element_type: String,
    name: String,
    port_count: usize,
    feature_count: usize,
    connections_to: Vec<String>,
}

pub fn cmd_diagram_list(elements: &[RawElement], type_filter_raw: Option<&str>, ns_filter: Option<&str>) {
    let type_filter: Option<Vec<String>> = type_filter_raw
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());

    let entries: Vec<ListEntry> = elements
        .iter()
        .filter(|e| {
            let etype = e.frontmatter.element_type.as_ref();
            // Skip packages and diagram elements
            !matches!(
                etype,
                Some(ElementType::Package)
                    | Some(ElementType::LibraryPackage)
                    | Some(ElementType::Namespace)
                    | Some(ElementType::Diagram)
                    | Some(ElementType::Unknown)
                    | None
            )
        })
        .filter(|e| {
            if let Some(ns) = ns_filter {
                e.qualified_name.starts_with(ns)
            } else {
                true
            }
        })
        .filter(|e| {
            if let Some(ref types) = type_filter {
                let label = e
                    .frontmatter
                    .element_type
                    .as_ref()
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_default();
                types.iter().any(|t| t.eq_ignore_ascii_case(&label))
            } else {
                true
            }
        })
        .map(|e| {
            let name = e
                .frontmatter
                .name
                .clone()
                .unwrap_or_else(|| {
                    e.qualified_name
                        .split("::")
                        .last()
                        .unwrap_or(&e.qualified_name)
                        .to_string()
                });

            let port_count = e
                .frontmatter
                .features
                .as_ref()
                .map(|fs| {
                    fs.iter()
                        .filter(|f| {
                            matches!(f, serde_yaml::Value::Mapping(m)
                                if m.contains_key(serde_yaml::Value::String("direction".into())))
                        })
                        .count()
                })
                .unwrap_or(0);

            let feature_count = e
                .frontmatter
                .features
                .as_ref()
                .map(|fs| fs.len() - port_count)
                .unwrap_or(0);

            // Gather connections_to from connections frontmatter
            let connections_to = gather_connected_elements(e);

            let type_label = e
                .frontmatter
                .element_type
                .as_ref()
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|| "Unknown".to_string());

            ListEntry {
                qname: &e.qualified_name,
                element_type: type_label,
                name,
                port_count,
                feature_count,
                connections_to,
            }
        })
        .collect();

    match serde_json::to_string_pretty(&entries) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("error serializing list: {}", e),
    }
}

fn gather_connected_elements(e: &RawElement) -> Vec<String> {
    let mut targets = Vec::new();
    if let Some(connections) = &e.frontmatter.connections {
        for conn in connections {
            if let serde_yaml::Value::Mapping(map) = conn {
                for key in ["to", "target", "from", "source"] {
                    if let Some(serde_yaml::Value::String(v)) =
                        map.get(serde_yaml::Value::String(key.into()))
                    {
                        let element_qname = v.split("::").collect::<Vec<_>>();
                        if element_qname.len() > 1 {
                            let candidate = element_qname[..element_qname.len() - 1].join("::");
                            if !candidate.is_empty() && !targets.contains(&candidate) {
                                targets.push(candidate);
                            }
                        }
                    }
                }
            }
        }
    }
    targets
}

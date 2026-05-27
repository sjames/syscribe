use syscribe_model::element::{ElementType, RawElement};

use super::{
    theme::{
        asil_badge, domain_badge, sil_badge, status_badge, stereotype_label, test_level_badge,
        theme_for,
    },
    types::{
        Badge, Compartment, CompartmentKind, ElementNode, FeatureRow, PortDirection, PortRow,
        ViewConfig, MIN_WIDTH,
    },
};

pub fn build_element_node(element: &RawElement, view: &ViewConfig) -> ElementNode {
    let fm = &element.frontmatter;
    let etype = fm.element_type.clone().unwrap_or(ElementType::Unknown);
    let theme = theme_for(&etype);
    let stereotype = stereotype_label(&etype).map(|s| s.to_string());

    let name = fm
        .name
        .clone()
        .or_else(|| fm.title.clone())
        .unwrap_or_else(|| {
            element
                .qualified_name
                .split("::")
                .last()
                .unwrap_or(&element.qualified_name)
                .to_string()
        });

    let is_abstract = fm.is_abstract.unwrap_or(false);

    // ── Header badges ────────────────────────────────────────────────────────
    let mut header_badges: Vec<Badge> = Vec::new();
    if let Some(id) = &fm.id {
        let is_id = id.contains('-') && id.len() <= 20;
        if is_id {
            header_badges.push(Badge {
                text: id.clone(),
                bg: theme.accent,
                fg: theme.header_fg,
                mono: true,
            });
        }
    }

    let mut compartments = vec![Compartment::Header {
        stereotype,
        name: name.clone(),
        is_abstract,
        badges: header_badges,
    }];

    // ── Status badges (Requirement, TestCase, ADR) ───────────────────────────
    let mut status_badges: Vec<Badge> = Vec::new();

    if let Some(status) = &fm.status {
        let (bg, fg) = status_badge(status);
        status_badges.push(Badge { text: status.clone(), bg, fg, mono: false });
    }

    if let Some(level) = fm.sil_level {
        let (bg, fg) = sil_badge(level);
        status_badges.push(Badge { text: format!("SIL-{}", level), bg, fg, mono: false });
    }

    if let Some(level) = &fm.asil_level {
        let (bg, fg) = asil_badge(level);
        let text = if level.starts_with("ASIL") {
            level.clone()
        } else {
            format!("ASIL-{}", level)
        };
        status_badges.push(Badge { text, bg, fg, mono: false });
    }

    if let Some(domain) = &fm.req_domain {
        let (bg, fg) = domain_badge(domain);
        status_badges.push(Badge { text: domain.clone(), bg, fg, mono: false });
    } else if let Some(domain) = &fm.domain {
        let (bg, fg) = domain_badge(domain);
        status_badges.push(Badge { text: domain.clone(), bg, fg, mono: false });
    }

    if let Some(level) = &fm.test_level {
        let (bg, fg) = test_level_badge(level);
        status_badges.push(Badge { text: level.clone(), bg, fg, mono: false });
    }

    if !status_badges.is_empty() {
        compartments.push(Compartment::StatusRow { badges: status_badges });
    }

    // ── Ports list ───────────────────────────────────────────────────────────
    let mut ports: Vec<PortRow> = Vec::new();

    // Inline features that are ports (have a direction field)
    if let Some(features) = &fm.features {
        for feat in features {
            if let serde_yaml::Value::Mapping(map) = feat {
                let name_val = map.get(&serde_yaml::Value::String("name".into()));
                let dir_val = map.get(&serde_yaml::Value::String("direction".into()));
                let type_val = map.get(&serde_yaml::Value::String("type".into()));

                if let (Some(serde_yaml::Value::String(n)), Some(serde_yaml::Value::String(d))) =
                    (name_val, dir_val)
                {
                    let direction = parse_direction(d);
                    let type_ref = type_val.and_then(|v| v.as_str()).map(|s| s.to_string());

                    // Parse nested sub-ports from an optional `ports:` sub-key
                    let sub_ports = map
                        .get(&serde_yaml::Value::String("ports".into()))
                        .and_then(|v| v.as_sequence())
                        .map(|seq| parse_sub_ports(seq))
                        .unwrap_or_default();

                    ports.push(PortRow {
                        name: n.clone(),
                        type_ref,
                        direction,
                        multiplicity: None,
                        sub_ports,
                    });
                }
            }
        }
    }

    // Element-level direction field (for Port elements themselves)
    if matches!(etype, ElementType::Port | ElementType::PortDef) {
        if let Some(dir) = &fm.direction {
            ports.push(PortRow {
                name: name.clone(),
                type_ref: fm.typed_by.as_ref().and_then(|v| v.as_str()).map(|s| s.to_string()),
                direction: parse_direction(dir),
                multiplicity: fm.multiplicity.clone(),
                sub_ports: vec![],
            });
        }
    }

    if !ports.is_empty() {
        compartments.push(Compartment::PortsList { items: ports });
    }

    // ── Features list ────────────────────────────────────────────────────────
    let mut features: Vec<FeatureRow> = Vec::new();

    if let Some(feat_list) = &fm.features {
        for feat in feat_list {
            if let serde_yaml::Value::Mapping(map) = feat {
                // Skip if it has a direction (already added as port)
                if map.contains_key(&serde_yaml::Value::String("direction".into())) {
                    continue;
                }
                let name_val = map.get(&serde_yaml::Value::String("name".into()));
                let type_val = map.get(&serde_yaml::Value::String("type".into()));

                if let Some(serde_yaml::Value::String(n)) = name_val {
                    let type_ref = type_val
                        .and_then(|v| v.as_str())
                        .unwrap_or("Any")
                        .to_string();
                    let unit = map
                        .get(&serde_yaml::Value::String("unit".into()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let value = map
                        .get(&serde_yaml::Value::String("value".into()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let is_derived = map
                        .get(&serde_yaml::Value::String("isDerived".into()))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let is_constant = map
                        .get(&serde_yaml::Value::String("isConstant".into()))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let mult = map
                        .get(&serde_yaml::Value::String("multiplicity".into()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    features.push(FeatureRow {
                        name: n.clone(),
                        type_ref,
                        multiplicity: mult,
                        unit,
                        value,
                        is_derived,
                        is_constant,
                    });
                }
            }
        }
    }

    if !features.is_empty() {
        compartments.push(Compartment::Features { items: features });
    }

    // ── Doc preview ──────────────────────────────────────────────────────────
    let doc_lines: Vec<String> = element
        .doc
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("```"))
        .take(2)
        .map(|l| {
            let max_chars = 48;
            if l.chars().count() > max_chars {
                format!("{}…", &l.chars().take(max_chars).collect::<String>())
            } else {
                l.to_string()
            }
        })
        .collect();

    // Gherkin preview for TestCase
    if matches!(etype, ElementType::TestCase) {
        let given = extract_gherkin_step(&element.doc, "Given");
        let when = extract_gherkin_step(&element.doc, "When");
        let then = extract_gherkin_step(&element.doc, "Then");
        if given.is_some() || when.is_some() || then.is_some() {
            compartments.push(Compartment::GherkinPreview { given, when, then });
        } else if !doc_lines.is_empty() {
            compartments.push(Compartment::DocPreview { lines: doc_lines });
        }
    } else if !doc_lines.is_empty() {
        compartments.push(Compartment::DocPreview { lines: doc_lines });
    }

    // Apply view filtering: preset visibility + per-item name filters
    let compartments = filter_compartments(compartments, view);

    ElementNode {
        qualified_name: element.qualified_name.clone(),
        element_type_label: format!(
            "{:?}",
            fm.element_type.as_ref().unwrap_or(&ElementType::Unknown)
        ),
        compartments,
        theme,
        min_width: view.min_width.unwrap_or(MIN_WIDTH),
        ibd: view.ibd,
    }
}

fn compartment_kind(c: &Compartment) -> CompartmentKind {
    match c {
        Compartment::Header { .. } => CompartmentKind::Header,
        Compartment::StatusRow { .. } => CompartmentKind::Status,
        Compartment::PortsList { .. } => CompartmentKind::Ports,
        Compartment::Features { .. } => CompartmentKind::Features,
        Compartment::DocPreview { .. } | Compartment::GherkinPreview { .. } => CompartmentKind::Doc,
    }
}

fn filter_compartments(compartments: Vec<Compartment>, view: &ViewConfig) -> Vec<Compartment> {
    compartments
        .into_iter()
        .filter(|c| {
            // In IBD mode, always pass PortsList through so the engine can render border squares,
            // even if the view preset (e.g. "name") would otherwise hide the ports compartment.
            if view.ibd && matches!(c, Compartment::PortsList { .. }) {
                return true;
            }
            view.preset.allows(compartment_kind(c))
        })
        .filter_map(|c| apply_include_filter(c, &view.include))
        .collect()
}

/// Apply per-item name filters within a compartment.
/// Returns `None` if the compartment becomes empty after filtering.
fn apply_include_filter(compartment: Compartment, include: &super::types::IncludeFilter) -> Option<Compartment> {
    match compartment {
        Compartment::PortsList { items } => {
            let filtered: Vec<PortRow> = items
                .into_iter()
                .filter(|p| include.port_allowed(&p.name))
                .collect();
            if filtered.is_empty() { None } else { Some(Compartment::PortsList { items: filtered }) }
        }
        Compartment::Features { items } => {
            let filtered: Vec<FeatureRow> = items
                .into_iter()
                .filter(|f| include.feature_allowed(&f.name))
                .collect();
            if filtered.is_empty() { None } else { Some(Compartment::Features { items: filtered }) }
        }
        other => Some(other),
    }
}

/// Parse a `ports:` sequence into a flat list of `PortRow`s (one level deep).
fn parse_sub_ports(seq: &[serde_yaml::Value]) -> Vec<PortRow> {
    seq.iter()
        .filter_map(|v| {
            let map = v.as_mapping()?;
            let name = map
                .get(&serde_yaml::Value::String("name".into()))?
                .as_str()?
                .to_string();
            let dir_str = map
                .get(&serde_yaml::Value::String("direction".into()))
                .and_then(|v| v.as_str())
                .unwrap_or("undirected");
            let type_ref = map
                .get(&serde_yaml::Value::String("type".into()))
                .or_else(|| map.get(&serde_yaml::Value::String("typedBy".into())))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Some(PortRow {
                name,
                type_ref,
                direction: parse_direction(dir_str),
                multiplicity: None,
                sub_ports: vec![],
            })
        })
        .collect()
}

fn parse_direction(s: &str) -> PortDirection {
    match s.to_lowercase().as_str() {
        "in" => PortDirection::In,
        "out" => PortDirection::Out,
        "inout" | "in_out" | "bidirectional" => PortDirection::InOut,
        _ => PortDirection::Undirected,
    }
}

fn extract_gherkin_step(doc: &str, keyword: &str) -> Option<String> {
    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(keyword) {
            let rest = trimmed[keyword.len()..].trim();
            let truncated = if rest.chars().count() > 44 {
                format!("{}…", rest.chars().take(44).collect::<String>())
            } else {
                rest.to_string()
            };
            return Some(format!("{} {}", keyword, truncated));
        }
    }
    None
}

pub fn feature_row_text(row: &FeatureRow) -> String {
    let prefix = if row.is_derived { "/" } else { "" };
    let short_type = row
        .type_ref
        .split("::")
        .last()
        .unwrap_or(&row.type_ref);
    let unit_str = row.unit.as_deref().map(|u| format!(" [{}]", u)).unwrap_or_default();
    let value_str = if row.is_constant {
        row.value.as_deref().map(|v| format!(" = {}", v)).unwrap_or_default()
    } else {
        String::new()
    };
    let mult_str = row
        .multiplicity
        .as_deref()
        .filter(|m| *m != "1")
        .map(|m| format!("[{}]", m))
        .unwrap_or_default();
    format!("{}{} : {}{}{}{}", prefix, row.name, short_type, mult_str, unit_str, value_str)
}

pub fn port_row_text(row: &PortRow) -> String {
    let arrow = row.direction.arrow();
    let type_str = row
        .type_ref
        .as_deref()
        .map(|t| {
            let short = t.split("::").last().unwrap_or(t);
            format!(" : {}", short)
        })
        .unwrap_or_default();
    format!("{} {}{}", arrow, row.name, type_str)
}


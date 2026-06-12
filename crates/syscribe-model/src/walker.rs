use std::path::{Path, PathBuf};
use anyhow::Result;
use walkdir::WalkDir;
use tracing::{warn, debug};
use crate::element::{ElementType, ParseIssue, RawElement, RawFrontmatter};
use crate::frontmatter::{split_frontmatter, parse_frontmatter};

/// Derive a qualified name from a file path relative to the model root.
/// `_index.md` contributes no name segment (represents the directory itself).
pub fn derive_qname(rel_path: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    let components: Vec<_> = rel_path.components().collect();
    let total = components.len();
    for (i, comp) in components.iter().enumerate() {
        let s = comp.as_os_str().to_string_lossy();
        let is_last = i + 1 == total;
        if is_last {
            // File component
            let stem = if s.ends_with(".md") { &s[..s.len() - 3] } else { &s };
            if stem == "_index" {
                // Don't add a segment — the directory name is already added
            } else {
                parts.push(stem.to_string());
            }
        } else {
            // rel_path is already relative to model_root; all directory
            // components are real namespace segments.
            parts.push(s.to_string());
        }
    }
    parts.join("::")
}

/// Load ignore patterns from `<model_root>/.sysmlignore` (one gitignore-style pattern per line).
fn load_sysmlignore(model_root: &Path) -> Vec<String> {
    let path = model_root.join(".sysmlignore");
    match std::fs::read_to_string(&path) {
        Ok(content) => content
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Return true if `rel` (relative to model root) matches any of the patterns.
/// Supports: exact filename (`README.md`), directory prefix (`Draft/`), simple glob (`*.log`).
fn is_ignored(rel: &Path, patterns: &[String]) -> bool {
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    let filename = rel.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    for pat in patterns {
        if pat.ends_with('/') {
            // Directory pattern: any path whose first component matches
            let dir = pat.trim_end_matches('/');
            if rel_str.starts_with(&format!("{}/", dir)) {
                return true;
            }
        } else if pat.contains('*') {
            // Simple glob on filename only
            let re_src = regex::escape(pat).replace("\\*", ".*");
            if let Ok(re) = regex::Regex::new(&format!("^{}$", re_src)) {
                if re.is_match(&filename) {
                    return true;
                }
            }
        } else {
            // Exact filename or exact relative path
            if filename == *pat || rel_str == *pat {
                return true;
            }
        }
    }
    false
}

/// Walk `model_root`, parse every `.md` file, return `Vec<RawElement>`.
pub fn walk_model(model_root: &Path) -> Result<Vec<RawElement>> {
    let mut elements = Vec::new();
    let ignore_patterns = load_sysmlignore(model_root);

    // Two-pass: collect all paths first, sort so _index.md comes before siblings
    let mut paths: Vec<PathBuf> = WalkDir::new(model_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        .filter(|e| {
            if ignore_patterns.is_empty() { return true; }
            let rel = e.path().strip_prefix(model_root).unwrap_or(e.path());
            !is_ignored(rel, &ignore_patterns)
        })
        .map(|e| e.into_path())
        .collect();

    // Sort: shallower paths first, _index.md before siblings at same depth
    paths.sort_by(|a, b| {
        let da = a.components().count();
        let db = b.components().count();
        da.cmp(&db).then_with(|| {
            let a_is_index = a.file_name().map_or(false, |n| n == "_index.md");
            let b_is_index = b.file_name().map_or(false, |n| n == "_index.md");
            b_is_index.cmp(&a_is_index).then_with(|| a.cmp(b))
        })
    });

    for path in &paths {
        let rel = path.strip_prefix(model_root).unwrap_or(path);
        let qname = derive_qname(rel);
        let file_path = path.display().to_string();

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Cannot read {}: {}", file_path, e);
                continue;
            }
        };

        let (fm_str, body) = split_frontmatter(&content);
        let (frontmatter, parse_issue) = match fm_str {
            None => {
                debug!("No frontmatter in {}", file_path);
                (Default::default(), Some(ParseIssue::NoFrontmatter))
            }
            Some(yaml) => match parse_frontmatter(yaml) {
                Ok(fm) => (fm, None),
                Err(e) => {
                    warn!("Frontmatter parse error in {}: {}", file_path, e);
                    (Default::default(), Some(ParseIssue::YamlError(e.to_string())))
                }
            }
        };

        elements.push(RawElement {
            qualified_name: qname,
            file_path,
            frontmatter,
            doc: body.to_string(),
            parse_issue,
        });
    }

    explode_fmea_entries(&mut elements);
    explode_tara_entries(&mut elements);
    Ok(elements)
}

/// Post-processing pass: for each TARASheet, synthesise DamageScenario, ThreatScenario,
/// CybersecurityGoal, and SecurityControl elements from the four section tables.
/// Each row must have an `id` key; rows without one are skipped.
///
/// Because the YAML keys in each row match the camelCase serde field names already
/// defined in RawFrontmatter, we deserialise directly via serde_yaml::from_value
/// and then override element_type.
fn explode_tara_entries(elements: &mut Vec<RawElement>) {
    let mut synthetic: Vec<RawElement> = Vec::new();

    for sheet in elements.iter() {
        if !matches!(sheet.frontmatter.element_type, Some(ElementType::TARASheet)) {
            continue;
        }

        let sections: &[(&[serde_yaml::Value], ElementType)] = &[
            (
                sheet.frontmatter.damage_table.as_deref().unwrap_or(&[]),
                ElementType::DamageScenario,
            ),
            (
                sheet.frontmatter.threat_table.as_deref().unwrap_or(&[]),
                ElementType::ThreatScenario,
            ),
            (
                sheet.frontmatter.goal_table.as_deref().unwrap_or(&[]),
                ElementType::CybersecurityGoal,
            ),
            (
                sheet.frontmatter.control_table.as_deref().unwrap_or(&[]),
                ElementType::SecurityControl,
            ),
        ];

        for (rows, elem_type) in sections {
            for row_val in *rows {
                // Require an id key to identify the row
                let entry_id = match row_val
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                {
                    Some(id) => id,
                    None => continue,
                };

                // Deserialise the row mapping into RawFrontmatter, then override type
                let mut fm: RawFrontmatter =
                    serde_yaml::from_value(row_val.clone()).unwrap_or_default();
                fm.element_type = Some(elem_type.clone());
                // Inherit sheet status when row has none
                if fm.status.is_none() {
                    fm.status = sheet.frontmatter.status.clone();
                }

                synthetic.push(RawElement {
                    qualified_name: format!("{}::{}", sheet.qualified_name, entry_id),
                    file_path: sheet.file_path.clone(),
                    frontmatter: fm,
                    doc: String::new(),
                    parse_issue: None,
                });
            }
        }
    }

    elements.extend(synthetic);
}

/// Post-processing pass: for each FMEASheet, synthesise a FMEAEntry RawElement
/// for every item in its `entries:` list.  Each entry must have an `id` key;
/// entries without one are silently skipped (the validator will warn).
fn explode_fmea_entries(elements: &mut Vec<RawElement>) {
    let mut synthetic: Vec<RawElement> = Vec::new();

    for sheet in elements.iter() {
        if !matches!(sheet.frontmatter.element_type, Some(ElementType::FMEASheet)) {
            continue;
        }
        let entries = match &sheet.frontmatter.entries {
            Some(v) if !v.is_empty() => v.clone(),
            _ => continue,
        };

        for entry_val in &entries {
            let map = match entry_val {
                serde_yaml::Value::Mapping(m) => m,
                _ => continue,
            };

            // Helpers for extracting typed values from the mapping
            let str_val = |key: &str| -> Option<String> {
                map.get(&serde_yaml::Value::String(key.into()))
                    .and_then(|v| v.as_str())
                    .map(String::from)
            };
            let u8_val = |key: &str| -> Option<u8> {
                map.get(&serde_yaml::Value::String(key.into()))
                    .and_then(|v| v.as_u64())
                    .map(|n| n.min(255) as u8)
            };
            let strings_val = |key: &str| -> Option<Vec<String>> {
                match map.get(&serde_yaml::Value::String(key.into())) {
                    Some(serde_yaml::Value::String(s)) => Some(vec![s.clone()]),
                    Some(serde_yaml::Value::Sequence(seq)) => Some(
                        seq.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
                    ),
                    _ => None,
                }
            };

            let entry_id = match str_val("id") {
                Some(id) => id,
                None => continue,
            };

            let failure_mode = str_val("failureMode");
            let label = failure_mode
                .clone()
                .or_else(|| str_val("name"))
                .unwrap_or_else(|| entry_id.clone());

            let s = u8_val("severity");
            let o = u8_val("occurrence");
            let d = u8_val("detection");
            // Compute RPN if all three components are present; otherwise take explicit value
            let rpn: Option<u32> = match (s, o, d) {
                (Some(sv), Some(oc), Some(dt)) => Some(sv as u32 * oc as u32 * dt as u32),
                _ => map
                    .get(&serde_yaml::Value::String("rpn".into()))
                    .and_then(|v| v.as_u64())
                    .map(|n| n as u32),
            };

            let fm = RawFrontmatter {
                element_type: Some(ElementType::FMEAEntry),
                id: Some(entry_id.clone()),
                name: Some(label),
                status: str_val("status").or_else(|| sheet.frontmatter.status.clone()),
                subject: str_val("ref"),
                failure_mode,
                effect: str_val("effect"),
                cause: str_val("cause"),
                fmea_severity: s,
                occurrence: o,
                detection: d,
                rpn,
                recommended_action: str_val("recommendedAction"),
                satisfies: strings_val("satisfies"),
                ..Default::default()
            };

            synthetic.push(RawElement {
                qualified_name: format!("{}::{}", sheet.qualified_name, entry_id),
                file_path: sheet.file_path.clone(),
                frontmatter: fm,
                doc: String::new(),
                parse_issue: None,
            });
        }
    }

    elements.extend(synthetic);
}

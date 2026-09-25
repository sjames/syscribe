use std::collections::{HashMap, HashSet};
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
            let stem = s.strip_suffix(".md").unwrap_or(&s);
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

/// True when `elem` was synthesized from a multi-element sheet (an FMEA/TARA
/// entry, or — REQ-TRS-FM-005 — a `FeatureModel` sheet's `featureTree:`
/// entry) rather than being that file's own 1:1 element: its qualified name
/// doesn't match what [`derive_qname`] on `rel_path` (its own `file_path`,
/// already made relative to the model root by the caller) would produce,
/// because that file's *real*, path-derived qname belongs to the sheet, not
/// to this entry.
///
/// A single-file-at-a-time write (find an element's `file_path` by qname,
/// then rewrite or remove that whole file — `update_element`, `move_element`,
/// `delete_element` in both `syscribe-server`'s routes and the MCP tools)
/// must refuse when this is true: pointed at a synthesized element it would
/// otherwise silently patch/move/delete the *sheet's* file — wrong field for
/// an update (patches the sheet's own top-level frontmatter, not the buried
/// list entry), and every sibling entry lost for a delete or move (the whole
/// sheet file is removed/relocated).
pub fn is_synthesized(elem: &RawElement, rel_path: &Path) -> bool {
    derive_qname(rel_path) != elem.qualified_name
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
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
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
            let a_is_index = a.file_name().is_some_and(|n| n == "_index.md");
            let b_is_index = b.file_name().is_some_and(|n| n == "_index.md");
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
            derived: Default::default(),
            derive_findings: Vec::new(),
            locale_docs: Default::default(),
        });
    }

    // §3.10 locale documentation variants: fold each `locale:` + `qualifiedName:`
    // file into its target's `locale_docs` before any synthesis pass runs.
    attach_locale_variants(&mut elements);
    explode_fmea_entries(&mut elements);
    explode_tara_entries(&mut elements);
    explode_feature_model_trees(&mut elements);
    // Native SysML v2/KerML submodel scoping (ADR-SYS-SYSMLV2-001): strip stray
    // nested `_index.md`s out of any `sysmlSubmodel: true` package's subtree.
    crate::sysmlv2::apply_sysmlv2_submodels(&mut elements, model_root);
    // Parse and merge `.sysml`/`.kerml` content in each surviving subtree into
    // real RawElements (REQ-TRS-SYSMLV2-002).
    crate::sysmlv2::ingest_sysml_submodels(&mut elements, model_root);
    // Foreign-format ingestion via stdio-subprocess plugins (ADR-SYS-PLUGIN-002).
    crate::plugins::apply_foreign_plugins(&mut elements, model_root);
    // Annotated-source ingestion: in-process comment-marker scan (ADR-SYS-ANNOTATE-001).
    crate::annotations::apply_annotation_scans(&mut elements, model_root);
    // Derive pass: evaluate `derive:` blocks; findings stored in each element's derive_findings.
    crate::derive::derive_pass(&mut elements);
    // Configuration inheritance through `derivedFrom:` (§9.8, GH #137): after
    // every other pass so a plugin-/sheet-synthesized Configuration takes part.
    crate::config_inherit::apply_configuration_inheritance(&mut elements);
    Ok(elements)
}

/// Frontmatter keys a §3.10 locale variant may carry. Anything else would
/// redefine the element's structure, which a variant never does (W051).
const LOCALE_VARIANT_KEYS: &[&str] = &["type", "name", "locale", "qualifiedName"];

/// True when `elem` is a §3.10 locale documentation variant: it carries both
/// `locale:` and a `qualifiedName:` naming the element it documents.
pub fn is_locale_variant(elem: &RawElement) -> bool {
    elem.frontmatter.locale.is_some() && elem.frontmatter.qualified_name.is_some()
}

/// Top-level frontmatter keys of `file` outside [`LOCALE_VARIANT_KEYS`],
/// sorted. Re-reads the file: variants are rare, and the typed
/// `RawFrontmatter` cannot tell an authored key from a defaulted one.
fn locale_variant_extra_keys(file: &str) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(file) else { return Vec::new() };
    let (Some(yaml), _) = split_frontmatter(&content) else { return Vec::new() };
    let Ok(map) = serde_yaml::from_str::<serde_yaml::Mapping>(yaml) else { return Vec::new() };
    let mut keys: Vec<String> = map
        .keys()
        .filter_map(|k| k.as_str())
        .filter(|k| !LOCALE_VARIANT_KEYS.contains(k))
        .map(str::to_string)
        .collect();
    keys.sort();
    keys
}

/// §3.10 locale documentation variants (REQ-TRS-PARSE-010, GH #160).
///
/// A file with both `locale:` and `qualifiedName:` contributes its Markdown
/// body as the `locale` documentation of the file-backed element whose
/// (path-derived) qualified name equals `qualifiedName:`; it does not define
/// an element of its own, so it is removed from `elements`. Findings go to
/// the target's `derive_findings`, filed against the variant's path:
///
/// - `W051` — the target already has documentation for that locale (an
///   earlier variant in walk order, or the element's own `locale:`; the
///   first wins), the variant's `type:` differs from the target's, or the
///   variant declares fields other than [`LOCALE_VARIANT_KEYS`] (ignored).
/// - `E026` — `qualifiedName:` names no element. The variant is then kept as
///   its own element (the pre-#160 behaviour) carrying the error, so it is
///   never silently dropped.
fn attach_locale_variants(elements: &mut Vec<RawElement>) {
    if !elements.iter().any(is_locale_variant) {
        return;
    }
    // Targets: every non-variant element by qualified name (first wins on a
    // duplicate, which is E108 anyway).
    let mut by_qname: HashMap<String, usize> = HashMap::new();
    for (i, e) in elements.iter().enumerate() {
        if !is_locale_variant(e) {
            by_qname.entry(e.qualified_name.clone()).or_insert(i);
        }
    }

    let mut attached: Vec<usize> = Vec::new();
    for i in 0..elements.len() {
        if !is_locale_variant(&elements[i]) {
            continue;
        }
        let variant = &elements[i];
        let file = variant.file_path.clone();
        let locale = variant.frontmatter.locale.clone().unwrap_or_default().trim().to_string();
        let target_q = variant.frontmatter.qualified_name.clone().unwrap_or_default().trim().to_string();

        let Some(&t) = by_qname.get(&target_q) else {
            let msg = format!(
                "locale variant (locale '{}') names `qualifiedName: {}`, which resolves to no element — \
                 its documentation cannot be attached, so the file is treated as its own element (§3.10)",
                locale, target_q
            );
            elements[i].derive_findings.push(("E026".to_string(), file, msg));
            continue;
        };

        let mut findings: Vec<(String, String, String)> = Vec::new();
        let extras = locale_variant_extra_keys(&file);
        if !extras.is_empty() {
            findings.push((
                "W051".to_string(),
                file.clone(),
                format!(
                    "locale variant of '{}' declares {} — a variant contributes documentation only and never \
                     redefines the element's structure; ignored (allowed: type, name, locale, qualifiedName)",
                    target_q,
                    extras.iter().map(|k| format!("'{}'", k)).collect::<Vec<_>>().join(", ")
                ),
            ));
        }
        if let (Some(vt), Some(tt)) =
            (&variant.frontmatter.element_type, &elements[t].frontmatter.element_type)
        {
            if vt != tt {
                findings.push((
                    "W051".to_string(),
                    file.clone(),
                    format!(
                        "locale variant declares `type: {}` but its target '{}' is a {}; the variant's type is ignored",
                        vt.name(),
                        target_q,
                        tt.name()
                    ),
                ));
            }
        }
        let body = variant.doc.clone();
        let target = &mut elements[t];
        let own_locale = target.frontmatter.locale.as_deref().map(str::trim) == Some(locale.as_str());
        if own_locale || target.locale_docs.contains_key(&locale) {
            findings.push((
                "W051".to_string(),
                file.clone(),
                format!(
                    "'{}' already has '{}' documentation ({}); this locale variant is ignored",
                    target_q,
                    locale,
                    if own_locale { "its own body is tagged with that locale" } else { "from an earlier variant file" }
                ),
            ));
        } else {
            target.locale_docs.insert(locale, body);
        }
        target.derive_findings.extend(findings);
        attached.push(i);
    }

    for i in attached.into_iter().rev() {
        elements.remove(i);
    }
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
                    derived: Default::default(),
                    derive_findings: Vec::new(),
                    locale_docs: Default::default(),
                });
            }
        }
    }

    elements.extend(synthetic);
}

/// Post-processing pass: for each FMEASheet, synthesise a FMEAEntry RawElement
/// for every item in its `entries:` list.  Each entry must have an `id` key;
/// entries without one are dropped here and reported by the validator as
/// `E923` on the sheet (GH #132). When S, O and D are all present the computed
/// RPN wins over an explicit `rpn:`; a disagreeing explicit value is `W928`.
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
                map.get(serde_yaml::Value::String(key.into()))
                    .and_then(|v| v.as_str())
                    .map(String::from)
            };
            let u8_val = |key: &str| -> Option<u8> {
                map.get(serde_yaml::Value::String(key.into()))
                    .and_then(|v| v.as_u64())
                    .map(|n| n.min(255) as u8)
            };
            let strings_val = |key: &str| -> Option<Vec<String>> {
                match map.get(serde_yaml::Value::String(key.into())) {
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

            let s = u8_val("fmeaSeverity").or_else(|| u8_val("severity"));
            let o = u8_val("occurrence");
            let d = u8_val("detection");
            // Compute RPN if all three components are present; otherwise take explicit value
            let rpn: Option<u32> = match (s, o, d) {
                (Some(sv), Some(oc), Some(dt)) => Some(sv as u32 * oc as u32 * dt as u32),
                _ => map
                    .get(serde_yaml::Value::String("rpn".into()))
                    .and_then(|v| v.as_u64())
                    .map(|n| n as u32),
            };

            const KNOWN_FMEA_ENTRY_KEYS: &[&str] = &[
                "id", "ref", "name", "failureMode", "status", "effect", "cause",
                "fmeaSeverity", "severity", "occurrence", "detection", "rpn",
                "recommendedAction", "satisfies", "ftaRef",
            ];
            let unknown_fmea_keys: Vec<String> = map
                .keys()
                .filter_map(|k| k.as_str())
                .filter(|k| !KNOWN_FMEA_ENTRY_KEYS.contains(k))
                .map(String::from)
                .collect();

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
                fta_ref: str_val("ftaRef"),
                satisfies: strings_val("satisfies"),
                unknown_fmea_keys,
                ..Default::default()
            };

            synthetic.push(RawElement {
                qualified_name: format!("{}::{}", sheet.qualified_name, entry_id),
                file_path: sheet.file_path.clone(),
                frontmatter: fm,
                doc: String::new(),
                parse_issue: None,
                derived: Default::default(),
                derive_findings: Vec::new(),
                locale_docs: Default::default(),
            });
        }
    }

    elements.extend(synthetic);
}


/// Post-processing pass (REQ-TRS-FM-005): for each `type: FeatureModel` sheet,
/// explode its **flat** `featureTree:` list into ordinary `FeatureDef`
/// `RawElement`s — one per entry — so a whole feature model can be authored as
/// one file instead of a directory-per-feature tree. An entry's `name:` is a
/// **dot-separated relative path** from the sheet (e.g. `Platform.CortexM`,
/// not a single basic name); splitting it on `.` and joining with `::` under
/// the sheet's own qname produces exactly the qname a directory-per-feature
/// layout would produce for the same tree shape (`Features::Platform::CortexM`).
/// The synthesized element's own `name:` is rewritten to just the last path
/// segment — the same leaf label a per-file `FeatureDef` would carry — so every
/// downstream consumer (validator, `feature-check`, `matrix`, the web UI) sees
/// the same kind of `FeatureDef` element either way. An ancestor path prefix
/// need not have its own entry, exactly as an ancestor directory need not be a
/// `FeatureDef` today: `feature_model::parent_of` already treats "no FeatureDef
/// at that qname" as "no parent", so this flat form and the multi-file form
/// share that same fallback with no extra logic here.
///
/// An entry with no `name:`, or whose path has an empty segment (leading,
/// trailing, or doubled `.`), cannot be placed in the qname tree; it is dropped
/// and flagged `E231` on the sheet. Two entries that resolve to the same
/// qualified name are flagged `E232`. `featureTree:` declared on any element
/// type other than `FeatureModel` is inert and flagged `W048`.
///
/// A sheet may also declare `crossTreeConstraints:` — a flat list of
/// `{ feature, requires, excludes }` entries, kept separate from the
/// structural `featureTree:` so requires/excludes edges can be reviewed as one
/// section (see `RawFrontmatter::cross_tree_constraints`'s doc comment for the
/// reference-resolution rule). Each entry's resolved `requires`/`excludes` are
/// merged into the matching synthesized `FeatureDef`'s own field; a `feature:`
/// that doesn't resolve to a `FeatureDef` synthesized from *this* sheet is
/// `E233`.
fn explode_feature_model_trees(elements: &mut Vec<RawElement>) {
    let mut synthetic: Vec<RawElement> = Vec::new();
    // (sheet index, code, message) — folded into that sheet's `derive_findings`
    // once the borrow over `elements` for reading is done.
    let mut findings: Vec<(usize, &'static str, String)> = Vec::new();
    // Every qname already in use — pre-existing elements plus whatever this
    // pass has synthesized so far — so cross-sheet collisions are caught too.
    let mut seen_qnames: HashSet<String> =
        elements.iter().map(|e| e.qualified_name.clone()).collect();

    for (idx, sheet) in elements.iter().enumerate() {
        let is_feature_model = sheet.frontmatter.element_type == Some(ElementType::FeatureModel);
        let has_tree = sheet.frontmatter.feature_tree.is_some();
        let has_constraints = sheet.frontmatter.cross_tree_constraints.is_some();
        if !has_tree && !has_constraints {
            continue;
        }

        if !is_feature_model {
            let field = match (has_tree, has_constraints) {
                (true, true) => "'featureTree:'/'crossTreeConstraints:' are",
                (true, false) => "'featureTree:' is",
                (false, true) => "'crossTreeConstraints:' is",
                (false, false) => unreachable!("guarded by the has_tree/has_constraints check above"),
            };
            findings.push((
                idx,
                "W048",
                format!("{} only recognized on a 'type: FeatureModel' element; ignored here", field),
            ));
            continue;
        }

        // This sheet's own entries land at `synthetic[sheet_start..]` — that
        // range is what `crossTreeConstraints:` below is allowed to attach to.
        let sheet_start = synthetic.len();
        if let Some(tree) = &sheet.frontmatter.feature_tree {
            for entry in tree {
                explode_feature_entry(
                    entry,
                    &sheet.qualified_name,
                    &sheet.file_path,
                    idx,
                    &mut seen_qnames,
                    &mut synthetic,
                    &mut findings,
                );
            }
        }

        if let Some(constraints) = &sheet.frontmatter.cross_tree_constraints {
            let by_qname: HashMap<String, usize> = synthetic[sheet_start..]
                .iter()
                .enumerate()
                .map(|(i, e)| (e.qualified_name.clone(), sheet_start + i))
                .collect();
            for c in constraints {
                apply_cross_tree_constraint(c, &sheet.qualified_name, idx, &by_qname, &mut synthetic, &mut findings);
            }
        }
    }

    for (idx, code, message) in findings {
        let file = elements[idx].file_path.clone();
        elements[idx].derive_findings.push((code.to_string(), file, message));
    }
    elements.extend(synthetic);
}

/// Join a sheet's qname with a relative suffix without introducing a leading
/// `::` when the sheet sits at the model root (qname `""` — a `FeatureModel`
/// `_index.md` placed directly at the model root, no wrapping directory).
fn join_under_sheet(sheet_qname: &str, suffix: &str) -> String {
    if sheet_qname.is_empty() {
        suffix.to_string()
    } else {
        format!("{}::{}", sheet_qname, suffix)
    }
}

/// Resolve one `featureTree:`/`crossTreeConstraints:` reference string:
/// containing `::` → already an absolute qname; starting with `FEAT` → a
/// stable id; otherwise → a dot-separated path relative to `sheet_qname`.
/// Returns `None` for a relative path with an empty segment.
fn resolve_feature_ref(raw: &str, sheet_qname: &str) -> Option<String> {
    if raw.contains("::") || raw.starts_with("FEAT") {
        return Some(raw.to_string());
    }
    let segments: Vec<&str> = raw.split('.').collect();
    if raw.is_empty() || segments.iter().any(|s| s.is_empty()) {
        return None;
    }
    Some(join_under_sheet(sheet_qname, &segments.join("::")))
}

/// Explode one flat `featureTree:` entry into a synthetic `FeatureDef`
/// `RawElement`, appending to `synthetic`/`findings`.
fn explode_feature_entry(
    entry: &serde_yaml::Value,
    sheet_qname: &str,
    sheet_file: &str,
    sheet_idx: usize,
    seen_qnames: &mut HashSet<String>,
    synthetic: &mut Vec<RawElement>,
    findings: &mut Vec<(usize, &'static str, String)>,
) {
    let serde_yaml::Value::Mapping(map) = entry else {
        findings.push((
            sheet_idx,
            "E231",
            format!("a featureTree: entry under '{}' is not a mapping — skipped", sheet_qname),
        ));
        return;
    };
    let mut map = map.clone();
    let doc = map
        .remove("doc")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let raw_name = map.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let Some(raw_name) = raw_name else {
        findings.push((
            sheet_idx,
            "E231",
            format!(
                "a featureTree: entry under '{}' has no 'name:' — cannot be placed in the qualified-name tree",
                sheet_qname
            ),
        ));
        return;
    };

    let segments: Vec<&str> = raw_name.split('.').collect();
    if segments.iter().any(|s| s.is_empty()) {
        findings.push((
            sheet_idx,
            "E231",
            format!(
                "featureTree: entry name '{}' under '{}' has an empty path segment (leading, trailing, or doubled '.')",
                raw_name, sheet_qname
            ),
        ));
        return;
    }

    let qname = join_under_sheet(sheet_qname, &segments.join("::"));
    let leaf_name = *segments.last().unwrap();

    if !seen_qnames.insert(qname.clone()) {
        findings.push((
            sheet_idx,
            "E232",
            format!("featureTree: entry '{}' collides with an existing element of the same qualified name", qname),
        ));
        return;
    }

    // REQ-TRS-FM-006: `id:` is optional on a featureTree: entry — unlike a
    // plain per-file FeatureDef, where E201 still requires one — because
    // retyping one per entry is where a flat, potentially large list stings
    // most. When absent (missing, `null`, or empty string; anything else,
    // e.g. a non-string value, is left alone and handled by the normal
    // deserialize path below), derive one from the entry's own dotted path:
    // segments uppercased, non-[A-Z0-9] characters stripped (so a basic-name
    // underscore is dropped, not preserved), joined with `-`, prefixed
    // `FEAT-`. The result is assigned exactly as if hand-authored, so E006
    // (grammar) and E101 (duplicate) already apply with no new code needed.
    let id_absent = match map.get("id") {
        None | Some(serde_yaml::Value::Null) => true,
        Some(serde_yaml::Value::String(s)) => s.is_empty(),
        Some(_) => false,
    };
    if id_absent {
        let seg_id = |s: &str| -> String {
            s.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_uppercase()
        };
        let derived_id = format!("FEAT-{}", segments.iter().map(|s| seg_id(s)).collect::<Vec<_>>().join("-"));
        map.insert("id".into(), derived_id.into());
    }

    map.insert("name".into(), leaf_name.into());
    // A type-mismatched field anywhere in the entry (e.g. `mandatory: "yes"`)
    // fails the *whole* deserialize — `unwrap_or_default()` would silently
    // discard every other field (id, groupKind, requires, ...) with no
    // diagnostic beyond a confusing downstream `E201`. Surface it exactly as
    // a real malformed `.md` file would: `parse_issue: YamlError` → the
    // existing generic `E002`, naming this entry's qname and the concrete
    // deserialize error, and skip synthesizing a FeatureDef we can't trust.
    match serde_yaml::from_value::<RawFrontmatter>(serde_yaml::Value::Mapping(map)) {
        Ok(mut fm) => {
            fm.element_type = Some(ElementType::FeatureDef);
            synthetic.push(RawElement {
                qualified_name: qname,
                file_path: sheet_file.to_string(),
                frontmatter: fm,
                doc,
                parse_issue: None,
                derived: Default::default(),
                derive_findings: Vec::new(),
                locale_docs: Default::default(),
            });
        }
        Err(e) => {
            synthetic.push(RawElement {
                qualified_name: qname,
                file_path: sheet_file.to_string(),
                frontmatter: RawFrontmatter::default(),
                doc,
                parse_issue: Some(ParseIssue::YamlError(format!("featureTree: entry '{}': {}", raw_name, e))),
                derived: Default::default(),
                derive_findings: Vec::new(),
                locale_docs: Default::default(),
            });
        }
    }
}

/// Merge one `crossTreeConstraints:` entry's `requires`/`excludes` into the
/// matching synthesized `FeatureDef`'s own field, or flag `E233` when its
/// `feature:` doesn't resolve within this sheet.
fn apply_cross_tree_constraint(
    c: &serde_yaml::Value,
    sheet_qname: &str,
    sheet_idx: usize,
    by_qname: &HashMap<String, usize>,
    synthetic: &mut [RawElement],
    findings: &mut Vec<(usize, &'static str, String)>,
) {
    let serde_yaml::Value::Mapping(map) = c else {
        findings.push((
            sheet_idx,
            "E233",
            format!("a crossTreeConstraints: entry under '{}' is not a mapping — skipped", sheet_qname),
        ));
        return;
    };

    let strings_of = |key: &str| -> Vec<String> {
        match map.get(key) {
            Some(serde_yaml::Value::String(s)) => vec![s.clone()],
            Some(serde_yaml::Value::Sequence(seq)) => {
                seq.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            }
            _ => Vec::new(),
        }
    };

    let Some(raw_feature) = map.get("feature").and_then(|v| v.as_str()) else {
        findings.push((
            sheet_idx,
            "E233",
            format!("a crossTreeConstraints: entry under '{}' has no 'feature:'", sheet_qname),
        ));
        return;
    };
    let Some(feature_qname) = resolve_feature_ref(raw_feature, sheet_qname) else {
        findings.push((
            sheet_idx,
            "E233",
            format!("crossTreeConstraints: 'feature: {}' under '{}' has an empty path segment", raw_feature, sheet_qname),
        ));
        return;
    };
    let Some(&target_idx) = by_qname.get(&feature_qname) else {
        findings.push((
            sheet_idx,
            "E233",
            format!(
                "crossTreeConstraints: feature '{}' does not resolve to a FeatureDef synthesized from this sheet's own featureTree:",
                feature_qname
            ),
        ));
        return;
    };

    let mut requires: Vec<String> = Vec::new();
    for r in strings_of("requires") {
        match resolve_feature_ref(&r, sheet_qname) {
            Some(resolved) => requires.push(resolved),
            None => findings.push((
                sheet_idx,
                "E233",
                format!("crossTreeConstraints: 'requires: {}' under '{}' has an empty path segment", r, sheet_qname),
            )),
        }
    }
    let mut excludes: Vec<String> = Vec::new();
    for x in strings_of("excludes") {
        match resolve_feature_ref(&x, sheet_qname) {
            Some(resolved) => excludes.push(resolved),
            None => findings.push((
                sheet_idx,
                "E233",
                format!("crossTreeConstraints: 'excludes: {}' under '{}' has an empty path segment", x, sheet_qname),
            )),
        }
    }

    let target = &mut synthetic[target_idx];
    if !requires.is_empty() {
        let list = target.frontmatter.requires.get_or_insert_with(Vec::new);
        list.extend(requires.into_iter().map(serde_yaml::Value::String));
    }
    if !excludes.is_empty() {
        target.frontmatter.excludes.get_or_insert_with(Vec::new).extend(excludes);
    }
}

//! Accepted-findings baselines for `validate --new-since` / `--baseline-file`
//! (GH #431, `REQ-TRS-VALBASE-001`).
//!
//! A finding is identified by its code, the stable identity of the element it
//! is raised on (`id:` when set, else the qualified name) and a hash of its
//! message — never by file path or line, so moving or renaming a file does not
//! turn its existing findings into new ones. Findings are matched as a
//! multiset: a second copy of an accepted finding is new.

use crate::element::RawElement;
use crate::validator::{Finding, Severity};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// One accepted finding in a baseline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub code: String,
    /// Stable id or qualified name of the element; the file when no element owns it.
    pub element: String,
    pub message_hash: String,
    /// Kept for human review of the baseline file; not part of the match key.
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineFile {
    pub schema_version: String,
    pub findings: Vec<Entry>,
}

fn message_hash(msg: &str) -> String {
    blake3::hash(msg.as_bytes()).to_hex()[..16].to_string()
}

fn ident(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn severity_word(s: &Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

/// Describe `findings` as baseline entries, resolving each finding's file to its owning element.
pub fn entries_for(findings: &[&Finding], elements: &[RawElement]) -> Vec<Entry> {
    let by_file: HashMap<&str, &RawElement> = elements.iter().map(|e| (e.file_path.as_str(), e)).collect();
    findings
        .iter()
        .map(|f| Entry {
            code: f.code.to_string(),
            element: by_file.get(f.file.as_str()).map(|e| ident(e)).unwrap_or_else(|| f.file.clone()),
            message_hash: message_hash(&f.message),
            message: f.message.clone(),
            severity: severity_word(&f.severity).to_string(),
        })
        .collect()
}

pub fn to_json(entries: &[Entry]) -> String {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| (&a.code, &a.element, &a.message_hash).cmp(&(&b.code, &b.element, &b.message_hash)));
    serde_json::to_string_pretty(&BaselineFile { schema_version: "1.0".into(), findings: sorted }).unwrap_or_default() + "\n"
}

pub fn from_json(text: &str) -> Result<Vec<Entry>, String> {
    let f: BaselineFile = serde_json::from_str(text).map_err(|e| format!("not a findings baseline: {e}"))?;
    if f.schema_version != "1.0" {
        return Err(format!("unsupported findings-baseline schemaVersion `{}`", f.schema_version));
    }
    Ok(f.findings)
}

/// For each of `findings`, `true` when it is absent from `baseline`.
///
/// A finding with no exact baseline match is still treated as pre-existing when
/// a leftover baseline entry has the same code and message but names an element
/// that no longer exists — the element was renamed or moved, not the finding
/// introduced.
pub fn classify(findings: &[&Finding], elements: &[RawElement], baseline: &[Entry]) -> Vec<bool> {
    let current = entries_for(findings, elements);
    let live: HashSet<String> = elements.iter().map(ident).collect();

    let mut exact: HashMap<(&str, &str, &str), usize> = HashMap::new();
    for b in baseline {
        *exact.entry((b.code.as_str(), b.element.as_str(), b.message_hash.as_str())).or_insert(0) += 1;
    }
    let mut is_new: Vec<bool> = vec![true; current.len()];
    for (i, c) in current.iter().enumerate() {
        if let Some(n) = exact.get_mut(&(c.code.as_str(), c.element.as_str(), c.message_hash.as_str())) {
            if *n > 0 {
                *n -= 1;
                is_new[i] = false;
            }
        }
    }
    // Leftover baseline entries whose element vanished: candidates for a rename/move match.
    let mut orphaned: HashMap<(&str, &str), usize> = HashMap::new();
    for ((code, element, hash), n) in &exact {
        if *n > 0 && !live.contains(*element) {
            *orphaned.entry((*code, *hash)).or_insert(0) += *n;
        }
    }
    for (i, c) in current.iter().enumerate() {
        if is_new[i] {
            if let Some(n) = orphaned.get_mut(&(c.code.as_str(), c.message_hash.as_str())) {
                if *n > 0 {
                    *n -= 1;
                    is_new[i] = false;
                }
            }
        }
    }
    is_new
}

/// Where the accepted findings come from.
#[derive(Debug, Clone)]
pub enum Source {
    /// The findings the model had at this git revision.
    Rev(String),
    /// A JSON list saved with `--write-baseline`.
    File(std::path::PathBuf),
}

/// For each of `current` (the findings of the model in `config`), whether it is new
/// relative to `source`. `validate_old` runs the same validation the caller ran for
/// `current` (plain or projected) on the model at the revision.
pub fn mark_new(
    source: &Source,
    current: &[Finding],
    elements: &[RawElement],
    config: &crate::config::ValidateConfig,
    validate_old: &dyn Fn(&[RawElement], &crate::config::ValidateConfig) -> Vec<Finding>,
) -> Result<Vec<bool>, String> {
    let refs: Vec<&Finding> = current.iter().collect();
    let baseline = match source {
        Source::File(p) => {
            let text = std::fs::read_to_string(p).map_err(|e| format!("cannot read {}: {e}", p.display()))?;
            from_json(&text).map_err(|e| format!("{}: {e}", p.display()))?
        }
        Source::Rev(rev) => {
            let root = config.model_root.clone().ok_or("the model root is unknown")?;
            let old = crate::gitrev::materialize(&root, rev)?;
            let old_elems = crate::walker::walk_model(&old.root)
                .map_err(|e| format!("cannot load the model at `{rev}`: {e}"))?;
            let mut old_cfg = crate::config::ValidateConfig::with_model_root(&old.root);
            old_cfg.magicgrid = config.magicgrid;
            let found = validate_old(&old_elems, &old_cfg);
            // `with_model_root` installs process-wide vocabularies (id prefixes, link types,
            // binding times); put the current model's back before anything else validates.
            let _ = crate::config::ValidateConfig::with_model_root(&root);
            let found_refs: Vec<&Finding> = found.iter().collect();
            entries_for(&found_refs, &old_elems)
        }
    };
    Ok(classify(&refs, elements, &baseline))
}

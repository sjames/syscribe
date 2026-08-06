//! Native SysML v2/KerML submodel ingestion (`ADR-SYS-SYSMLV2-001`,
//! `REQ-TRS-SYSMLV2-*`).
//!
//! A package `_index.md` may declare `sysmlSubmodel: true`. Every
//! `.sysml`/`.kerml` file anywhere in that directory's subtree — however
//! nested — is parsed in-process via the `sysml-v2-parser` crate instead of
//! Markdown+YAML frontmatter; the package's own `_index.md` remains a normal
//! native element. This is a dedicated, always-on native subsystem — not a
//! `[plugins.<alias>]` engine variant (see the ADR's sub-decision 1): there is
//! no sandbox, no config, no alias, and it runs from its own call site in
//! [`crate::walker::walk_model`], not through `plugins::apply_foreign_plugins`.
//!
//! A model with no `sysmlSubmodel: true` package is completely unaffected
//! (`REQ-TRS-SYSMLV2-000`).
//!
//! `.sysml`/`.kerml` files are never collected by [`crate::walker::walk_model`]
//! in the first place (it only walks `.md` files), so they are already
//! invisible to the graph with no special handling needed here — this module
//! does not yet synthesize any `RawElement`s from them (`REQ-TRS-SYSMLV2-001`
//! scopes that out; a later requirement covers real ingestion). What this
//! module *does* handle is the one thing native `.md` walking gets wrong
//! inside a marked subtree: a stray nested `_index.md` would otherwise be
//! parsed as an ordinary package. Hand-authored non-index `.md` element files
//! inside the subtree are left completely alone — they keep participating in
//! the namespace exactly as they would outside a `sysmlSubmodel` package.

use std::path::{Path, PathBuf};

use crate::element::RawElement;

/// Apply `sysmlSubmodel: true` subtree scoping to `elements` in place.
///
/// For every package `_index.md` declaring `sysmlSubmodel: true`, any other
/// `_index.md` found anywhere in that package's directory subtree is removed
/// from the graph (it is not a package — nested subdirectories inside a
/// `sysmlSubmodel` subtree carry no namespace meaning of their own) and
/// replaced with a `W540` finding attached to the declaring package.
pub fn apply_sysmlv2_submodels(elements: &mut Vec<RawElement>, _model_root: &Path) {
    // Every sysmlSubmodel package is anchored at its own `_index.md`.
    let anchors: Vec<(usize, PathBuf, String)> = elements
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            if e.frontmatter.sysml_submodel != Some(true) || !e.file_path.ends_with("_index.md") {
                return None;
            }
            let dir = Path::new(&e.file_path)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default();
            Some((i, dir, e.file_path.clone()))
        })
        .collect();

    if anchors.is_empty() {
        return;
    }

    // Find every stray `_index.md` inside a marked subtree (anywhere, however
    // nested), owned by its deepest-matching marked ancestor.
    let mut strays: Vec<(usize, String)> = Vec::new(); // (owner anchor index, stray file path)
    for elem in elements.iter() {
        if !elem.file_path.ends_with("_index.md") {
            continue;
        }
        if anchors.iter().any(|(_, _, anchor_path)| anchor_path == &elem.file_path) {
            continue; // the package's own anchor — always a normal native element
        }
        let owner = anchors
            .iter()
            .filter(|(_, dir, _)| under_dir(&elem.file_path, dir))
            .max_by_key(|(_, dir, _)| dir.components().count());
        if let Some((owner_idx, _, _)) = owner {
            strays.push((*owner_idx, elem.file_path.clone()));
        }
    }

    for (owner_idx, stray_path) in &strays {
        push_finding(
            &mut elements[*owner_idx],
            "W540",
            stray_path,
            &format!(
                "'{stray_path}' ignored — inside a sysmlSubmodel subtree (nested _index.md files carry no namespace meaning there)"
            ),
        );
    }

    if !strays.is_empty() {
        let stray_paths: Vec<&str> = strays.iter().map(|(_, p)| p.as_str()).collect();
        elements.retain(|e| !stray_paths.contains(&e.file_path.as_str()));
    }
}

/// True if `file_path` lies inside `dir` (component-wise, not a string prefix).
/// Callers exclude the anchor's own `_index.md` separately.
fn under_dir(file_path: &str, dir: &Path) -> bool {
    if dir.as_os_str().is_empty() {
        return false;
    }
    Path::new(file_path).starts_with(dir)
}

fn push_finding(elem: &mut RawElement, code: &str, file: &str, message: &str) {
    elem.derive_findings
        .push((code.to_string(), file.to_string(), message.to_string()));
}

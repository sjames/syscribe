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

use crate::derive::finding;
use crate::element::RawElement;

/// Apply `sysmlSubmodel: true` subtree scoping to `elements` in place.
///
/// For every package `_index.md` declaring `sysmlSubmodel: true`, any other
/// `_index.md` found anywhere in that package's directory subtree is removed
/// from the graph (it is not a package — nested subdirectories inside a
/// `sysmlSubmodel` subtree carry no namespace meaning of their own) and
/// replaced with a `W540` finding attached to the declaring package.
///
/// Anchors are confirmed shallowest-first: a `_index.md` that *itself*
/// declares `sysmlSubmodel: true` but sits inside an already-confirmed
/// anchor's subtree does not get to start its own subtree — it is just
/// another stray, exactly like a plain nested `_index.md` would be. Otherwise
/// a `sysmlSubmodel: true` package nested inside another one would escape
/// exclusion entirely (the inner anchor "claims" itself before the outer
/// anchor's sweep ever sees it).
pub fn apply_sysmlv2_submodels(elements: &mut Vec<RawElement>, _model_root: &Path) {
    // Every `_index.md`, marked or not, is a candidate — sorted shallowest
    // (fewest path components) first so outer packages are confirmed before
    // any package nested inside them is considered.
    let mut candidates: Vec<(usize, PathBuf, String, bool)> = elements
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            if !e.file_path.ends_with("_index.md") {
                return None;
            }
            let dir = Path::new(&e.file_path)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default();
            Some((i, dir, e.file_path.clone(), e.frontmatter.sysml_submodel == Some(true)))
        })
        .collect();
    candidates.sort_by_key(|(_, dir, _, _)| dir.components().count());

    let mut confirmed: Vec<(usize, PathBuf)> = Vec::new(); // (owner idx, dir)
    let mut strays: Vec<(usize, String)> = Vec::new(); // (owner idx, stray file path)

    for (idx, dir, file_path, is_marked) in &candidates {
        let owner = confirmed
            .iter()
            .filter(|(_, anchor_dir)| under_dir(file_path, anchor_dir))
            .max_by_key(|(_, anchor_dir)| anchor_dir.components().count());

        match owner {
            Some((owner_idx, _)) => {
                // Inside an already-confirmed subtree: always a stray, even if
                // it declares `sysmlSubmodel: true` itself — it never gets to
                // start its own subtree.
                strays.push((*owner_idx, file_path.clone()));
            }
            None if *is_marked => {
                // Not inside anything already confirmed, and marked: a new anchor.
                confirmed.push((*idx, dir.clone()));
            }
            None => {
                // An ordinary, unmarked package outside every marked subtree.
            }
        }
    }

    if strays.is_empty() {
        return;
    }

    for (owner_idx, stray_path) in &strays {
        elements[*owner_idx].derive_findings.push(finding(
            "W540",
            stray_path,
            &format!(
                "'{stray_path}' ignored — inside a sysmlSubmodel subtree (nested _index.md files carry no namespace meaning there)"
            ),
        ));
    }

    let stray_paths: Vec<&str> = strays.iter().map(|(_, p)| p.as_str()).collect();
    elements.retain(|e| !stray_paths.contains(&e.file_path.as_str()));
}

/// True if `file_path` lies inside `dir` (component-wise, not a string
/// prefix). Every `_index.md` lives in its own distinct directory, so this
/// only ever matches a *different* file's path against a confirmed anchor's
/// directory — never an anchor against its own directory.
fn under_dir(file_path: &str, dir: &Path) -> bool {
    if dir.as_os_str().is_empty() {
        return false;
    }
    Path::new(file_path).starts_with(dir)
}

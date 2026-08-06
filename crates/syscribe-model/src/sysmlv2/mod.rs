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
//! in the first place (it only walks `.md` files) — this module handles two
//! things `.md` walking gets wrong or misses entirely inside a marked
//! subtree: a stray nested `_index.md` would otherwise be parsed as an
//! ordinary package ([`apply_sysmlv2_submodels`]), and the `.sysml`/`.kerml`
//! content itself needs its own parse-and-merge pass to become real
//! `RawElement`s ([`ingest_sysml_submodels`], `REQ-TRS-SYSMLV2-002`). Hand-
//! authored non-index `.md` element files inside the subtree are left
//! completely alone — they keep participating in the namespace exactly as
//! they would outside a `sysmlSubmodel` package.

pub mod ingest;

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

/// Parse and merge every `.sysml`/`.kerml` file in each `sysmlSubmodel: true`
/// subtree into the graph as ordinary `RawElement`s (`REQ-TRS-SYSMLV2-002`).
///
/// Must run after [`apply_sysmlv2_submodels`] so any stray nested `_index.md`
/// anchors have already been stripped out — this pass only needs to find the
/// surviving, confirmed anchors, with no re-derivation of the stray/shallowest-
/// first logic that lives there.
pub fn ingest_sysml_submodels(elements: &mut Vec<RawElement>, _model_root: &Path) {
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
            Some((i, dir, e.qualified_name.clone()))
        })
        .collect();

    if anchors.is_empty() {
        return;
    }

    let mut synthetic = Vec::new();
    for (idx, dir, pkg_qname) in anchors {
        synthetic.extend(ingest::ingest_subtree(&mut elements[idx], &pkg_qname, &dir));
    }
    elements.extend(synthetic);
}

/// The qualified names of every `RawElement` synthesized by SysMLv2 ingestion
/// (`REQ-TRS-SYSMLV2-002`), for validator policy decisions that must be
/// gated on actual SysMLv2 origin rather than element kind alone
/// (`REQ-TRS-SYSMLV2-004`, `Resolver::is_verify_target`).
///
/// `RawElement` deliberately carries no origin field — see
/// [`ingest_sysml_submodels`]'s module doc and `REQ-TRS-SYSMLV2-002`'s
/// rationale for why SysMLv2-synthesized and hand-authored elements must stay
/// indistinguishable once merged into the graph. This function is the
/// side-channel provenance set that lets *validator policy* (not the graph
/// itself) still ask "did this specific one come from SysMLv2 ingestion?" —
/// the same shape of answer `crate::config::LoadedRepo::qnames` gives
/// multi-repo composition for "is this qname known to a peer repo?", without
/// touching `RawElement`.
///
/// Unlike `LoadedRepo` (whose peer elements aren't in `elements` at all, so
/// its qname index has to come from an independent walk of the peer's model
/// root at config-load time), every SysMLv2-synthesized element already *is*
/// in `elements` by the time `validate_with_config` runs — so rather than
/// threading a second return value out of [`ingest::ingest_subtree`] through
/// [`crate::walker::walk_model`]'s signature (and every one of its ~20
/// existing callers across the CLI/MCP/LSP/web server, exactly the
/// per-call-site wiring risk `ADR-SYS-PLUGIN-001` avoided for the WASM-plugin
/// merge), this derives the set fresh from `elements` itself: every
/// SysMLv2-synthesized `RawElement`'s `file_path` is, by construction, the
/// actual `.sysml`/`.kerml` source file it came from (`push_synth` in
/// `ingest.rs` never sets it to anything else) — a real, already-recorded
/// fact about the element, not a heuristic. O(n) in the element count; cheap
/// enough to call once per `validate_with_config` run.
pub fn synthesized_qnames(elements: &[RawElement]) -> std::collections::HashSet<String> {
    elements
        .iter()
        .filter(|e| e.file_path.ends_with(".sysml") || e.file_path.ends_with(".kerml"))
        .map(|e| e.qualified_name.clone())
        .collect()
}

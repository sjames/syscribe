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
    let first_synth = elements.len();
    elements.extend(synthetic);
    resolve_allocation_endpoints(elements, first_synth);
}

/// `REQ-TRS-SYSMLV2-029` (GH #144) — resolve the raw `allocate <source> to
/// <target>` endpoint text [`ingest`] lifted onto every synthesized
/// `Allocation` (`elements[first_synth..]`) into real qualified names, so the
/// ingested allocation feeds the §12.9 unified allocation set
/// (`validator::allocation_edges_tagged`) with no validator change.
///
/// Unlike `connect` lifting (`REQ-TRS-SYSMLV2-013`, a purely local AST
/// lookahead done inside `ingest`), this runs after the merge, over the
/// complete element list — allocation endpoints routinely cross packages
/// (`allocate Logical::ctrl to Physical::ecu`) and name features a usage
/// inherits from its type (`allocate sys.ctl to board.mcu`), neither of which
/// a single-file view can see. Per endpoint:
///
/// - a stable id (`REQ-*`, …) is left as-is (ids are global);
/// - the head (the text before the first `.`, possibly `::`-qualified)
///   resolves innermost scope first, from the allocation's owning namespace
///   outward to the model root — the same order `Resolver::resolve_scoped_ref`
///   uses for ingested `typedBy:`/`supertype:`;
/// - each further `.` segment resolves as a feature of the element reached so
///   far: declared directly on it (`<cur>::<seg>`), or inherited through its
///   `typedBy:`/`supertype:` chain (cycle-safe);
/// - a chain whose tail fails is truncated to its deepest resolved prefix and
///   the allocation carries a `W542` finding (the `connect` truncation code);
/// - a head that resolves nowhere is kept verbatim with `.` rewritten to `::`,
///   so `E502`/`E503` report it by the name the author wrote.
fn resolve_allocation_endpoints(elements: &mut [RawElement], first_synth: usize) {
    use std::collections::HashMap;

    let needs_work = elements[first_synth..].iter().any(|e| {
        matches!(e.frontmatter.element_type, Some(crate::element::ElementType::Allocation))
            && (e.frontmatter.allocated_from.is_some() || e.frontmatter.allocated_to.is_some())
    });
    if !needs_work {
        return;
    }

    // qname -> (typedBy, supertype) as plain strings, for the feature walk.
    let index: HashMap<String, (Option<String>, Option<String>)> = elements
        .iter()
        .map(|e| {
            let s = |v: &Option<serde_yaml::Value>| v.as_ref().and_then(|v| v.as_str()).map(str::to_string);
            (e.qualified_name.clone(), (s(&e.frontmatter.typed_by), s(&e.frontmatter.supertype)))
        })
        .collect();

    for elem in &mut elements[first_synth..] {
        if !matches!(elem.frontmatter.element_type, Some(crate::element::ElementType::Allocation)) {
            continue;
        }
        let scope = parent_scope(&elem.qualified_name).to_string();
        let mut truncations = Vec::new();
        for field in [&mut elem.frontmatter.allocated_from, &mut elem.frontmatter.allocated_to] {
            for entry in field.iter_mut().flatten() {
                let (resolved, trunc) = resolve_endpoint(&index, &scope, entry);
                truncations.extend(trunc);
                *entry = resolved;
            }
        }
        for msg in truncations {
            elem.derive_findings.push(finding("W542", &elem.file_path, &msg));
        }
    }
}

type EndpointIndex = std::collections::HashMap<String, (Option<String>, Option<String>)>;

/// The enclosing namespace of `qname` (`""` for a top-level name).
fn parent_scope(qname: &str) -> &str {
    qname.rfind("::").map_or("", |i| &qname[..i])
}

/// Resolve `r` innermost-scope-first from `scope` outward to the model root.
fn lookup_scoped(index: &EndpointIndex, scope: &str, r: &str) -> Option<String> {
    let mut scope = scope;
    loop {
        let candidate = if scope.is_empty() { r.to_string() } else { format!("{scope}::{r}") };
        if index.contains_key(&candidate) {
            return Some(candidate);
        }
        if scope.is_empty() {
            return None;
        }
        scope = parent_scope(scope);
    }
}

/// Resolve feature `seg` of the element `owner`: declared directly on it, or
/// inherited through its `typedBy:`/`supertype:` chain (each type reference
/// resolved from the referencing element's own enclosing scope). `seen`
/// guards against specialization/typing cycles.
fn lookup_feature(
    index: &EndpointIndex,
    owner: &str,
    seg: &str,
    seen: &mut std::collections::HashSet<String>,
) -> Option<String> {
    if !seen.insert(owner.to_string()) {
        return None;
    }
    let direct = format!("{owner}::{seg}");
    if index.contains_key(&direct) {
        return Some(direct);
    }
    let (typed_by, supertype) = index.get(owner)?;
    [typed_by, supertype].into_iter().flatten().find_map(|t| {
        let ty = lookup_scoped(index, parent_scope(owner), t)?;
        lookup_feature(index, &ty, seg, seen)
    })
}

/// Resolve one raw allocate-endpoint text; see [`resolve_allocation_endpoints`].
/// Returns the text to store and an optional `W542` truncation message.
fn resolve_endpoint(index: &EndpointIndex, scope: &str, raw: &str) -> (String, Option<String>) {
    if crate::resolver::is_stable_id(raw) {
        return (raw.to_string(), None);
    }
    let mut segments = raw.split('.');
    let head = segments.next().unwrap_or(raw);
    let Some(mut cur) = lookup_scoped(index, scope, head) else {
        return (raw.replace('.', "::"), None);
    };
    for seg in segments {
        match lookup_feature(index, &cur, seg, &mut Default::default()) {
            Some(next) => cur = next,
            None => {
                let message = format!(
                    "allocate endpoint '{raw}' has no feature '{seg}' on '{cur}' (neither declared on it \
                     nor inherited through its typedBy/supertype chain) -- truncated to the edge \
                     endpoint '{cur}' (see REQ-TRS-SYSMLV2-029)"
                );
                return (cur, Some(message));
            }
        }
    }
    (cur, None)
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
/// per-call-site wiring risk `ADR-SYS-PLUGIN-002` avoids for the
/// stdio-subprocess plugin merge), this derives the set fresh from `elements` itself: every
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

//! Suspect-link detection (§SuspectLinks; REQ-TRS-SUS-LINKS-000..007).
//!
//! A **suspect link** is a trace link whose target changed after the link was
//! last reviewed. Review captures a **baseline** — a BLAKE3 hash of a canonical
//! *projection* of the target — stored on the source in `traceBaselines`
//! (REQ-TRS-SUS-LINKS-001). Validation recomputes the projection hash and
//! compares; a mismatch is *suspect* and surfaces as warning `W090`
//! (REQ-TRS-SUS-LINKS-004).
//!
//! The projection (REQ-TRS-SUS-LINKS-002) hashes the markdown body plus the
//! normative frontmatter fields, **excluding** editorial/presentation fields so
//! cosmetic edits do not produce suspect storms. v1 uses a single default
//! projection for every element type.

use std::collections::{BTreeSet, HashSet};

use crate::element::{RawElement, RawFrontmatter};
use crate::resolver::Resolver;

/// Frontmatter fields excluded from the normative projection (v1 default,
/// REQ-TRS-SUS-LINKS-002). These are editorial/presentation or self-referential:
/// changing only these must **not** flip a link suspect. Keys are the serialized
/// (camelCase) frontmatter names.
const EXCLUDED_KEYS: &[&str] = &[
    "name",           // human label, not normative meaning
    "displayOrder",   // §3.16 presentation ordinal
    "extRef",         // external-tool pointer
    "title",          // removed label field (parsed only for E025)
    "traceBaselines", // the baselines themselves
    "layout",         // diagram layout coordinates
    "shapes",         // diagram geometry
    "edges",          // diagram geometry
    "svgFile",        // rendered-artifact pointer
    "pumlFile",       // rendered-artifact pointer
];

/// State of a single trace link relative to its stored baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkState {
    /// Baselined and the target's current projection hash matches.
    Ok,
    /// Baselined but the target's current projection hash differs → W090.
    Suspect,
    /// No baseline stored — silent during validation (REQ-TRS-SUS-LINKS-004),
    /// surfaced only on demand by `suspect list`.
    Unbaselined,
    /// Baselined, but the target does not resolve. Not reported as W090
    /// (REQ-TRS-SUS-LINKS-004 — "missing" ≠ "changed"); the existing
    /// unresolved-cross-reference handling owns this case.
    Unresolved,
}

/// One resolved-or-attempted trace link with its baseline state.
#[derive(Debug, Clone)]
pub struct SuspectLink {
    /// Source element's qualified name.
    pub source_qname: String,
    /// Source element's stable id, when it has one.
    pub source_id: Option<String>,
    /// Source element's file path (used as the finding location).
    pub source_file: String,
    /// The link kind (`verifies`, `derivedFrom`, …).
    pub kind: &'static str,
    /// The target identifier exactly as authored on the link — the `traceBaselines` key.
    pub target_ref: String,
    /// The stored baseline hash, if any.
    pub stored: Option<String>,
    /// The target's current projection hash, if the target resolves.
    pub current: Option<String>,
    /// Classification of this link.
    pub state: LinkState,
}

impl SuspectLink {
    /// A stable, human-facing label for the source (stable id if present, else qname).
    pub fn source_label(&self) -> &str {
        self.source_id.as_deref().unwrap_or(&self.source_qname)
    }
}

/// Compute the algorithm-prefixed content baseline of an element's normative
/// projection (REQ-TRS-SUS-LINKS-002): `blake3:<hex>`.
///
/// The projection is: the included frontmatter fields serialized to canonical
/// JSON (sorted keys, `null`s stripped) followed by the line-ending-normalized
/// markdown body. Two elements whose projections are byte-identical after
/// canonicalization hash identically; a change confined to an excluded field
/// does not change the hash.
pub fn projection_hash(elem: &RawElement) -> String {
    let mut fm_json = serde_json::to_value(&elem.frontmatter).unwrap_or(serde_json::Value::Null);
    if let serde_json::Value::Object(ref mut map) = fm_json {
        for k in EXCLUDED_KEYS {
            map.remove(*k);
        }
    }
    let mut canonical = String::new();
    write_canonical(&fm_json, &mut canonical);

    let body = normalize_body(&elem.doc);

    let mut hasher = blake3::Hasher::new();
    hasher.update(canonical.as_bytes());
    hasher.update(b"\n\x1e\n"); // record separator between frontmatter and body
    hasher.update(body.as_bytes());
    format!("blake3:{}", hasher.finalize().to_hex())
}

/// Normalize the markdown body so cosmetic reformatting does not change the hash:
/// CRLF (and lone CR) → LF, and leading/trailing whitespace trimmed. Trimming both
/// ends makes a CRLF-reformatted body — whose frontmatter split can leave differently
/// interleaved leading newlines than the LF form — canonicalize identically.
fn normalize_body(doc: &str) -> String {
    doc.replace("\r\n", "\n").replace('\r', "\n").trim().to_string()
}

/// Deterministic, `null`-stripped JSON serialization: object keys are emitted in
/// sorted order and `null` members are dropped. Independent of `serde_json`'s
/// map-ordering feature, so the digest is stable across builds.
fn write_canonical(v: &serde_json::Value, out: &mut String) {
    match v {
        serde_json::Value::Null => out.push_str("null"),
        serde_json::Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        serde_json::Value::Number(n) => out.push_str(&n.to_string()),
        serde_json::Value::String(s) => {
            out.push_str(&serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into()))
        }
        serde_json::Value::Array(arr) => {
            out.push('[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        serde_json::Value::Object(map) => {
            let keys: BTreeSet<&String> = map
                .iter()
                .filter(|(_, val)| !val.is_null())
                .map(|(k, _)| k)
                .collect();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(k).unwrap_or_else(|_| "\"\"".into()));
                out.push(':');
                write_canonical(&map[*k], out);
            }
            out.push('}');
        }
    }
}

/// Every trace link declared on a source frontmatter, as `(kind, target_ref)`
/// pairs (REQ-TRS-SUS-LINKS-003). Covers all link kinds; the target ref is the
/// string exactly as authored (the `traceBaselines` key). Deterministic order.
pub fn trace_links(fm: &RawFrontmatter) -> Vec<(&'static str, String)> {
    let mut out: Vec<(&'static str, String)> = Vec::new();

    let mut push_list = |kind: &'static str, list: &Option<Vec<String>>| {
        if let Some(v) = list {
            for r in v {
                out.push((kind, r.clone()));
            }
        }
    };

    push_list("verifies", &fm.verifies);
    push_list("derivedFrom", &fm.derived_from);
    push_list("satisfies", &fm.satisfies);
    push_list("satisfiedBy", &fm.satisfied_by);
    push_list("refines", &fm.refines);
    push_list("implementedBy", &fm.implemented_by);
    push_list("subsets", &fm.subsets);
    push_list("hazardRef", &fm.hazard_ref);
    push_list("mitigatedBy", &fm.mitigated_by);
    push_list("supports", &fm.supports);
    push_list("evidence", &fm.evidence);
    push_list("confirms", &fm.confirms);

    // Structural links carried as `serde_yaml::Value` (string or list).
    for r in yaml_refs(&fm.supertype) {
        out.push(("supertype", r));
    }
    for r in yaml_refs(&fm.redefines) {
        out.push(("redefines", r));
    }
    for r in yaml_refs(&fm.typed_by) {
        out.push(("typedBy", r));
    }

    // Scalar single-target links.
    if let Some(adr) = &fm.breakdown_adr {
        out.push(("breakdownAdr", adr.clone()));
    }

    out
}

/// Resolve a trace-link target (or a user-supplied source/target on the CLI) to
/// an element. Uses the standard resolver first; if that misses — e.g. a stable
/// id whose segments are shorter than the strict pattern allows, so it is indexed
/// only by its path-derived qualified name — it falls back to an exact declared
/// `id` match, then to a unique trailing qualified-name segment. The fallbacks are
/// only consulted on a strict miss and require a *unique* candidate, so they never
/// change which element an already-resolvable reference points to.
pub fn resolve_target<'a>(
    elements: &'a [RawElement],
    resolver: &Resolver,
    r: &str,
) -> Option<&'a RawElement> {
    if let Some(e) = resolver.resolve_ref(elements, r) {
        return Some(e);
    }
    let by_id: Vec<&RawElement> = elements
        .iter()
        .filter(|e| e.frontmatter.id.as_deref() == Some(r))
        .collect();
    if by_id.len() == 1 {
        return Some(by_id[0]);
    }
    let by_seg: Vec<&RawElement> = elements
        .iter()
        .filter(|e| e.qualified_name.rsplit("::").next() == Some(r))
        .collect();
    if by_seg.len() == 1 {
        return Some(by_seg[0]);
    }
    None
}

/// Extract reference strings from an optional YAML value that is either a scalar
/// string or a sequence of strings.
fn yaml_refs(v: &Option<serde_yaml::Value>) -> Vec<String> {
    match v {
        Some(serde_yaml::Value::String(s)) => vec![s.clone()],
        Some(serde_yaml::Value::Sequence(seq)) => {
            seq.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()
        }
        _ => Vec::new(),
    }
}

/// Scan every trace link in the model and classify it against its stored baseline
/// (REQ-TRS-SUS-LINKS-004). Where the same target is referenced by more than one
/// link kind from the same source, only the first (deterministically ordered)
/// occurrence is kept, since the baseline is a property of the target alone
/// (REQ-TRS-SUS-LINKS-003). Output is sorted for stable, diffable reporting.
pub fn scan(elements: &[RawElement], resolver: &Resolver) -> Vec<SuspectLink> {
    let mut out: Vec<SuspectLink> = Vec::new();

    for src in elements {
        let fm = &src.frontmatter;
        let mut seen: HashSet<String> = HashSet::new();

        for (kind, target_ref) in trace_links(fm) {
            // One baseline entry per target regardless of link kind.
            if !seen.insert(target_ref.clone()) {
                continue;
            }
            let stored = fm
                .trace_baselines
                .as_ref()
                .and_then(|m| m.get(&target_ref))
                .cloned();

            let (current, state) = match resolve_target(elements, resolver, &target_ref) {
                None => (
                    None,
                    if stored.is_some() { LinkState::Unresolved } else { LinkState::Unbaselined },
                ),
                Some(tgt) => {
                    let current = projection_hash(tgt);
                    let state = match &stored {
                        None => LinkState::Unbaselined,
                        Some(s) if *s == current => LinkState::Ok,
                        Some(_) => LinkState::Suspect,
                    };
                    (Some(current), state)
                }
            };

            out.push(SuspectLink {
                source_qname: src.qualified_name.clone(),
                source_id: fm.id.clone(),
                source_file: src.file_path.clone(),
                kind,
                target_ref,
                stored,
                current,
                state,
            });
        }
    }

    out.sort_by(|a, b| {
        a.source_label()
            .cmp(b.source_label())
            .then_with(|| a.source_qname.cmp(&b.source_qname))
            .then_with(|| a.target_ref.cmp(&b.target_ref))
            .then_with(|| a.kind.cmp(b.kind))
    });
    out
}

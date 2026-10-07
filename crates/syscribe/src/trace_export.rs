//! `syscribe trace-export` and the MCP `trace_export` tool (ADR-SYS-TREX-001,
//! REQ-TRS-TREX-001..004): one read-only JSON document listing every
//! requirement — native `Requirement` and SysML `RequirementDef`/`Requirement` —
//! with its derivation, satisfaction, verification, refinements and a computed
//! coverage block, optionally projected onto a configuration and emitted in a
//! declared sort order.
//!
//! The lists are the validator's reverse indices (`derivedChildren`,
//! `verifiedBy`, `refinedBy`) and the same `satisfies:` scan `W300` counts, read
//! through the link-type reporting view so a `coverage = true` type extending a
//! built-in link contributes (REQ-TRS-LINKTYPE-006). `coverage` mirrors `W300`
//! (leaf satisfied by at least one element), `W002` (verified by at least one
//! `active` TestCase) and `W305` (integration-verified by an `active` L3/L4/L5
//! TestCase); a retired TestCase is listed but never counts.
//!
//! One computation serves the CLI and MCP: [`build_document`] returns the typed
//! document, [`to_json`] renders it with the field order `REQ-TRS-TREX-001`
//! fixes (serde struct order — a `serde_json::Value` object would re-sort keys).

use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;
use syscribe_model::config::ValidateConfig;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::projection::{self, SelectionOutcome};
use syscribe_model::resolver::Resolver;
use syscribe_model::validator;
use syscribe_model::variability;

use crate::query::{tc_verdict, TcVerdict};

/// Document schema version (`"version": 1`); new fields are additive.
pub const VERSION: u64 = 1;

/// The valid `--sort` values, in the order the usage error names them.
pub const SORT_VALUES: &[&str] = &["directory", "asc", "desc"];

/// Order of the top-level `requirements` list and of every nested reference
/// list (REQ-TRS-TREX-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// The walker's file order — the order `export` and `ls` use.
    #[default]
    Directory,
    /// Ascending by full qualified name.
    Asc,
    /// Descending by full qualified name.
    Desc,
}

impl Sort {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "directory" => Some(Sort::Directory),
            "asc" => Some(Sort::Asc),
            "desc" => Some(Sort::Desc),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Sort::Directory => "directory",
            Sort::Asc => "asc",
            Sort::Desc => "desc",
        }
    }
}

// ── Document schema (REQ-TRS-TREX-001) ──────────────────────────────────────

/// The whole export. Field order is normative.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceDocument {
    pub version: u64,
    pub model_root: String,
    pub config: Option<ConfigRecord>,
    pub sort: &'static str,
    pub requirements: Vec<RequirementEntry>,
    pub summary: Summary,
}

/// The configuration the document was projected onto (REQ-TRS-TREX-002).
/// `id`/`qname` are null for an ad-hoc feature set; `name` then carries the
/// argument as written.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRecord {
    pub id: Option<String>,
    pub qname: Option<String>,
    pub name: Option<String>,
    pub active_features: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementEntry {
    pub qname: String,
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub element_type: String,
    pub status: Option<String>,
    pub req_class: Option<String>,
    pub req_domain: Option<String>,
    pub file: String,
    pub derived_from: Vec<Link<Ref>>,
    pub derived_children: Vec<Link<Ref>>,
    pub breakdown_adr: Option<Link<AdrRef>>,
    pub satisfied_by: Vec<Link<SatisfierRef>>,
    pub verified_by: Vec<Link<VerifierRef>>,
    pub refined_by: Vec<Link<Ref>>,
    pub coverage: Coverage,
}

/// A reference that resolved, or one that did not — `{ "qname": "<as
/// written>", "unresolved": true }` — never dropped.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Link<T> {
    Resolved(T),
    Unresolved(Unresolved),
}

#[derive(Debug, Serialize)]
pub struct Unresolved {
    pub qname: String,
    pub unresolved: bool,
}

#[derive(Debug, Serialize)]
pub struct Ref {
    pub qname: String,
    pub id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdrRef {
    pub qname: String,
    pub id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SatisfierRef {
    pub qname: String,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub element_type: String,
    pub domain: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifierRef {
    pub qname: String,
    pub id: Option<String>,
    pub test_level: Option<String>,
    pub status: Option<String>,
    /// `pass` / `fail` / `unknown` from the results sidecar, null without one.
    pub verdict: Option<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Coverage {
    pub leaf: bool,
    pub satisfied: bool,
    pub verified: bool,
    pub integration_verified: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub requirements: usize,
    pub leaves: usize,
    pub satisfied: usize,
    pub verified: usize,
    pub integration_verified: usize,
}

// ── Building ────────────────────────────────────────────────────────────────

/// A list item with the keys the three sort orders need; stripped after sorting.
struct Keyed<T> {
    /// Position in the walker order (`usize::MAX` for an unresolved reference,
    /// which sorts after every resolved one, in authored order).
    pos: usize,
    qname: String,
    item: T,
}

fn sorted<T>(mut items: Vec<Keyed<T>>, sort: Sort) -> Vec<T> {
    match sort {
        Sort::Directory => items.sort_by_key(|k| k.pos),
        Sort::Asc => items.sort_by(|a, b| a.qname.cmp(&b.qname)),
        Sort::Desc => items.sort_by(|a, b| b.qname.cmp(&a.qname)),
    }
    items.into_iter().map(|k| k.item).collect()
}

/// Where a reference lands.
enum Target<'a> {
    /// Resolves to an element of the (projected) view.
    Found(&'a RawElement),
    /// Resolves in the full model but is inactive in the configuration — omitted
    /// from every list (REQ-TRS-TREX-002).
    Inactive,
    /// Resolves nowhere — reported as `unresolved: true`.
    Dangling,
}

struct Ctx<'a> {
    full: &'a [RawElement],
    full_resolver: Resolver,
    view: &'a [RawElement],
    view_resolver: Resolver,
    /// Walker position of every element of the view, by qualified name.
    pos: HashMap<&'a str, usize>,
}

impl<'a> Ctx<'a> {
    fn lookup(&self, r: &str) -> Target<'a> {
        if let Some(e) = self.view_resolver.resolve_ref(self.view, r) {
            return Target::Found(e);
        }
        if self.full_resolver.resolve_ref(self.full, r).is_some() {
            return Target::Inactive;
        }
        Target::Dangling
    }

    fn pos_of(&self, e: &RawElement) -> usize {
        self.pos.get(e.qualified_name.as_str()).copied().unwrap_or(usize::MAX)
    }

    /// Resolve every reference of `refs` into a keyed list, mapping a found
    /// element through `make`; inactive targets are dropped, dangling ones kept.
    fn links<T, F>(&self, refs: &[String], make: F) -> Vec<Keyed<Link<T>>>
    where
        F: Fn(&'a RawElement) -> T,
    {
        let mut out = Vec::new();
        for r in refs {
            match self.lookup(r) {
                Target::Found(e) => out.push(Keyed {
                    pos: self.pos_of(e),
                    qname: e.qualified_name.clone(),
                    item: Link::Resolved(make(e)),
                }),
                Target::Inactive => {}
                Target::Dangling => out.push(Keyed {
                    pos: usize::MAX,
                    qname: r.clone(),
                    item: Link::Unresolved(Unresolved { qname: r.clone(), unresolved: true }),
                }),
            }
        }
        out
    }
}

fn is_requirement(e: &RawElement) -> bool {
    matches!(
        e.frontmatter.element_type,
        Some(ElementType::Requirement) | Some(ElementType::RequirementDef)
    )
}

fn type_name(e: &RawElement) -> String {
    e.frontmatter
        .element_type
        .as_ref()
        .map(|t| t.name().to_string())
        .unwrap_or_else(|| "Other".to_string())
}

/// The element's path relative to the model root (as the walker joined it).
fn relative_file(model_root: &Path, file_path: &str) -> String {
    Path::new(file_path)
        .strip_prefix(model_root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| file_path.to_string())
}

fn simple_ref(e: &RawElement) -> Ref {
    Ref { qname: e.qualified_name.clone(), id: e.frontmatter.id.clone() }
}

/// The reverse-index key of a target: its stable id when present, else its
/// qualified name (the validator's convention for `verifiedBy`/`refinedBy`).
fn index_key(e: &RawElement) -> &str {
    e.frontmatter.id.as_deref().unwrap_or(e.qualified_name.as_str())
}

/// The active feature qualified names of a selection, canonicalised
/// (`FEAT-*` ids rewritten to qnames) and sorted.
fn active_features(elements: &[RawElement], sel: &projection::Selection) -> Vec<String> {
    let alias = variability::feature_id_to_qname(elements);
    variability::canon_selection(sel, &alias)
        .into_iter()
        .filter(|(_, on)| *on)
        .map(|(q, _)| q)
        .collect()
}

fn stored_configuration<'a>(elements: &'a [RawElement], arg: &str) -> Option<&'a RawElement> {
    elements.iter().find(|e| {
        e.frontmatter.element_type.as_ref() == Some(&ElementType::Configuration)
            && (e.frontmatter.id.as_deref() == Some(arg) || e.qualified_name == arg)
    })
}

fn config_record(elements: &[RawElement], arg: &str, sel: &projection::Selection) -> ConfigRecord {
    match stored_configuration(elements, arg) {
        Some(cfg) => ConfigRecord {
            id: cfg.frontmatter.id.clone(),
            qname: Some(cfg.qualified_name.clone()),
            name: cfg.frontmatter.name.clone(),
            active_features: active_features(elements, sel),
        },
        None => ConfigRecord {
            id: None,
            qname: None,
            name: Some(arg.to_string()),
            active_features: active_features(elements, sel),
        },
    }
}

/// Build the trace document over `elements` (the walker's full model, in its
/// order), optionally projected onto `config` exactly as `export --config` does
/// (REQ-TRS-TREX-002). `Err` carries the projection engine's usage-error message
/// for an unresolvable or invalid configuration; the caller decides how to
/// report it (stderr + exit 1 on the CLI, a tool error over MCP).
pub fn build_document(
    model_root: &Path,
    elements: &[RawElement],
    vcfg: &ValidateConfig,
    config: Option<&str>,
    sort: Sort,
) -> Result<TraceDocument, String> {
    // ── Projection ────────────────────────────────────────────────────────
    let (view, config_record): (Vec<RawElement>, Option<ConfigRecord>) = match config {
        None => (elements.to_vec(), None),
        Some(c) => match projection::resolve_config_flag(elements, c) {
            SelectionOutcome::Error(m) => return Err(m),
            SelectionOutcome::Dormant => {
                // A stored Configuration in a model with no feature model: the lens
                // is inert (nothing is filtered) but the configuration is recorded.
                let sel = stored_configuration(elements, c)
                    .map(|cfg| cfg.frontmatter.feature_selections())
                    .unwrap_or_default();
                (elements.to_vec(), Some(config_record(elements, c, &sel)))
            }
            SelectionOutcome::Resolved(sel) => {
                (projection::project(elements, &sel), Some(config_record(elements, c, &sel)))
            }
        },
    };

    // ── Indices over the projected view ───────────────────────────────────
    let ctx = Ctx {
        full: elements,
        full_resolver: Resolver::new(elements),
        view: &view,
        view_resolver: Resolver::new(&view),
        pos: view.iter().enumerate().map(|(i, e)| (e.qualified_name.as_str(), i)).collect(),
    };
    let val = validator::validate_with_config(&view, vcfg);
    // REQ-TRS-LINKTYPE-006 — the reporting view: a `coverage = true` link that
    // extends derivedFrom/satisfies counts as that base link. Same element order
    // as `view`, so `ctx.view_resolver`'s indices stay valid.
    let cov = syscribe_model::link_types::coverage_view(&view, &vcfg.link_types);
    let results = vcfg.results.as_ref();

    // satisfiedBy[target qname] = satisfying element qnames, in walker order —
    // the same scan `W300` counts (every element's `satisfies:`, any type).
    let mut satisfied_by: HashMap<&str, Vec<&RawElement>> = HashMap::new();
    for (i, src) in cov.iter().enumerate() {
        let Some(sat) = src.frontmatter.satisfies.as_ref() else { continue };
        // The authored element (same index) is what the lists cite.
        let authored = &view[i];
        for r in sat {
            if let Some(target) = ctx.view_resolver.resolve_ref(&view, r) {
                let list = satisfied_by.entry(target.qualified_name.as_str()).or_default();
                if !list.iter().any(|e| e.qualified_name == authored.qualified_name) {
                    list.push(authored);
                }
            }
        }
    }

    // ── Requirements ──────────────────────────────────────────────────────
    let mut entries: Vec<Keyed<RequirementEntry>> = Vec::new();
    for (i, elem) in view.iter().enumerate() {
        if !is_requirement(elem) {
            continue;
        }
        let fm = &elem.frontmatter;
        let cov_fm = cov.get(i).map(|e| &e.frontmatter).unwrap_or(fm);
        let key = index_key(elem);

        let derived_from = sorted(ctx.links(cov_fm.derived_from.as_deref().unwrap_or(&[]), simple_ref), sort);
        let children_ids: Vec<String> = fm
            .id
            .as_deref()
            .and_then(|id| val.derived_children.get(id))
            .cloned()
            .unwrap_or_default();
        let derived_children = sorted(ctx.links(&children_ids, simple_ref), sort);
        let breakdown_adr = fm.breakdown_adr.as_ref().and_then(|r| {
            ctx.links(std::slice::from_ref(r), |adr| AdrRef {
                qname: adr.qualified_name.clone(),
                id: adr.frontmatter.id.clone(),
                status: adr.frontmatter.status.clone(),
            })
            .into_iter()
            .next()
            .map(|k| k.item)
        });

        let satisfiers: Vec<Keyed<Link<SatisfierRef>>> = satisfied_by
            .get(elem.qualified_name.as_str())
            .map(|srcs| {
                srcs.iter()
                    .map(|s| Keyed {
                        pos: ctx.pos_of(s),
                        qname: s.qualified_name.clone(),
                        item: Link::Resolved(SatisfierRef {
                            qname: s.qualified_name.clone(),
                            id: s.frontmatter.id.clone(),
                            element_type: type_name(s),
                            domain: s.frontmatter.domain.clone(),
                        }),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let satisfied_by_list = sorted(satisfiers, sort);

        let verifier_ids: Vec<String> = val.verified_by.get(key).cloned().unwrap_or_default();
        let mut verified = false;
        let mut integration_verified = false;
        for tc_id in &verifier_ids {
            if let Target::Found(tc) = ctx.lookup(tc_id) {
                if tc.frontmatter.status.as_deref() == Some("active") {
                    verified = true;
                    if tc
                        .frontmatter
                        .test_level
                        .as_deref()
                        .is_some_and(|lvl| matches!(lvl, "L3" | "L4" | "L5"))
                    {
                        integration_verified = true;
                    }
                }
            }
        }
        let verified_by = sorted(
            ctx.links(&verifier_ids, |tc| VerifierRef {
                qname: tc.qualified_name.clone(),
                id: tc.frontmatter.id.clone(),
                test_level: tc.frontmatter.test_level.clone(),
                status: tc.frontmatter.status.clone(),
                verdict: results.map(|_| match tc_verdict(tc, results) {
                    TcVerdict::Pass => "pass",
                    TcVerdict::Fail => "fail",
                    TcVerdict::Unknown => "unknown",
                }),
            }),
            sort,
        );

        let refiner_refs: Vec<String> = val.refined_by.get(key).cloned().unwrap_or_default();
        let refined_by = sorted(ctx.links(&refiner_refs, simple_ref), sort);

        let leaf = derived_children.is_empty();
        let coverage = Coverage {
            leaf,
            satisfied: !satisfied_by_list.is_empty(),
            verified,
            integration_verified,
        };

        entries.push(Keyed {
            pos: i,
            qname: elem.qualified_name.clone(),
            item: RequirementEntry {
                qname: elem.qualified_name.clone(),
                id: fm.id.clone(),
                name: fm.name.clone(),
                element_type: type_name(elem),
                status: fm.status.clone(),
                req_class: fm.req_class.clone(),
                req_domain: fm.req_domain.clone(),
                file: relative_file(model_root, &elem.file_path),
                derived_from,
                derived_children,
                breakdown_adr,
                satisfied_by: satisfied_by_list,
                verified_by,
                refined_by,
                coverage,
            },
        });
    }
    let requirements = sorted(entries, sort);

    let summary = Summary {
        requirements: requirements.len(),
        leaves: requirements.iter().filter(|r| r.coverage.leaf).count(),
        satisfied: requirements.iter().filter(|r| r.coverage.satisfied).count(),
        verified: requirements.iter().filter(|r| r.coverage.verified).count(),
        integration_verified: requirements.iter().filter(|r| r.coverage.integration_verified).count(),
    };

    Ok(TraceDocument {
        version: VERSION,
        model_root: model_root.to_string_lossy().into_owned(),
        config: config_record,
        sort: sort.name(),
        requirements,
        summary,
    })
}

/// Pretty-printed JSON in the normative field order, no trailing newline.
pub fn to_json(doc: &TraceDocument) -> String {
    serde_json::to_string_pretty(doc).unwrap_or_else(|_| "{}".to_string())
}

// ── CLI ─────────────────────────────────────────────────────────────────────

/// `syscribe -m <root> trace-export [--config <C>] [--sort <order>] [--out <file>]`
/// (REQ-TRS-TREX-004). `rest` is everything after the command name; unknown
/// options were already rejected by `cliargs::check_known_options`.
pub fn cmd_trace_export(model_root: &Path, elements: &[RawElement], vcfg: &ValidateConfig, rest: &[String]) {
    let sort_name =
        crate::cliargs::or_exit(crate::cliargs::enum_value("trace-export", rest, "--sort", SORT_VALUES, "directory"));
    let sort = Sort::parse(sort_name).unwrap_or_default();
    let config = rest.windows(2).find(|w| w[0] == "--config").map(|w| w[1].as_str());
    let out = rest.windows(2).find(|w| w[0] == "--out").map(|w| w[1].as_str());

    let doc = match build_document(model_root, elements, vcfg, config, sort) {
        Ok(d) => d,
        Err(m) => {
            // Same wording and exit as `export --config` (REQ-TRS-TREX-002).
            eprintln!("{m}");
            std::process::exit(1);
        }
    };
    let text = to_json(&doc);
    match out {
        None => println!("{text}"),
        Some(path) => {
            let path = Path::new(path);
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("Error: cannot create '{}': {e}", parent.display());
                    std::process::exit(1);
                }
            }
            if let Err(e) = std::fs::write(path, format!("{text}\n")) {
                eprintln!("Error: cannot write '{}': {e}", path.display());
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_values_round_trip() {
        for v in SORT_VALUES {
            assert_eq!(Sort::parse(v).unwrap().name(), *v);
        }
        assert!(Sort::parse("random").is_none());
        assert_eq!(Sort::default(), Sort::Directory);
    }

    #[test]
    fn unresolved_links_sort_after_resolved_in_directory_order() {
        let items = vec![
            Keyed { pos: usize::MAX, qname: "Zed".into(), item: "dangling" },
            Keyed { pos: 7, qname: "B::b".into(), item: "b" },
            Keyed { pos: 2, qname: "C::c".into(), item: "c" },
        ];
        assert_eq!(sorted(items, Sort::Directory), vec!["c", "b", "dangling"]);
        let items = vec![
            Keyed { pos: 7, qname: "B::b".into(), item: "b" },
            Keyed { pos: 2, qname: "C::c".into(), item: "c" },
        ];
        assert_eq!(sorted(items, Sort::Desc), vec!["c", "b"]);
    }

    #[test]
    fn document_field_order_is_normative() {
        let doc = TraceDocument {
            version: VERSION,
            model_root: "m".into(),
            config: None,
            sort: "directory",
            requirements: vec![RequirementEntry {
                qname: "R::X".into(),
                id: Some("REQ-X-001".into()),
                name: None,
                element_type: "Requirement".into(),
                status: None,
                req_class: None,
                req_domain: None,
                file: "R/REQ-X-001.md".into(),
                derived_from: vec![Link::Unresolved(Unresolved { qname: "REQ-NOPE-001".into(), unresolved: true })],
                derived_children: vec![],
                breakdown_adr: None,
                satisfied_by: vec![],
                verified_by: vec![],
                refined_by: vec![],
                coverage: Coverage { leaf: true, satisfied: false, verified: false, integration_verified: false },
            }],
            summary: Summary { requirements: 1, leaves: 1, satisfied: 0, verified: 0, integration_verified: 0 },
        };
        let text = to_json(&doc);
        let keys: Vec<&str> = ["\"version\"", "\"modelRoot\"", "\"config\"", "\"sort\"", "\"requirements\"", "\"summary\""].into();
        let positions: Vec<usize> = keys.iter().map(|k| text.find(k).unwrap()).collect();
        assert!(positions.windows(2).all(|w| w[0] < w[1]), "{text}");
        let entry_keys = [
            "\"qname\"", "\"id\"", "\"name\"", "\"type\"", "\"status\"", "\"reqClass\"", "\"reqDomain\"", "\"file\"",
            "\"derivedFrom\"", "\"derivedChildren\"", "\"breakdownAdr\"", "\"satisfiedBy\"", "\"verifiedBy\"",
            "\"refinedBy\"", "\"coverage\"",
        ];
        let positions: Vec<usize> = entry_keys.iter().map(|k| text.find(k).unwrap()).collect();
        assert!(positions.windows(2).all(|w| w[0] < w[1]), "{text}");
        assert!(text.contains("\"unresolved\": true"), "{text}");
    }
}

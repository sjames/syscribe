//! `syscribe topics` — distinctive per-package keywords via TF-IDF
//! (REQ-TRS-SEARCH-002).
//!
//! Treats each package (the containing namespace) as one document formed from its
//! elements' normative text, and reports each package's top terms by TF-IDF — the
//! vocabulary that distinguishes it from the rest of the corpus. Deterministic and
//! offline (ADR-SYS-SCAN-002); reuses the shared `textstats` TF-IDF core.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use syscribe_model::element::RawElement;

use crate::query::type_label;
use crate::textstats::Corpus;

/// The containing package of an element: its qualified name minus the last segment,
/// or `(root)` for an element directly under the model root.
pub fn parent_package(e: &RawElement) -> String {
    match e.qualified_name.rsplit_once("::") {
        Some((head, _)) => head.to_string(),
        None => "(root)".to_string(),
    }
}

/// Compute the topics document `{ packages: { <pkg>: [ {term, score} … ] } }` over
/// `view` (already projection-applied). `type_filter` selects the element type
/// (default `Requirement`).
pub fn compute_topics(view: &[RawElement], type_filter: Option<&str>, top: usize) -> Value {
    let want = type_filter.unwrap_or("Requirement");
    // Group the selected elements by package, in deterministic (sorted) order.
    let mut by_pkg: BTreeMap<String, Vec<&RawElement>> = BTreeMap::new();
    for e in view {
        if e.frontmatter.element_type.as_ref().map(type_label) == Some(want) {
            by_pkg.entry(parent_package(e)).or_default().push(e);
        }
    }
    let pkgs: Vec<String> = by_pkg.keys().cloned().collect();
    let texts: Vec<String> = pkgs
        .iter()
        .map(|p| {
            by_pkg[p].iter().map(|e| crate::textstats::element_text(e)).collect::<Vec<_>>().join("\n")
        })
        .collect();
    let corpus = Corpus::build(&texts);

    let mut packages = serde_json::Map::new();
    for (i, pkg) in pkgs.iter().enumerate() {
        let terms: Vec<Value> = corpus
            .top_terms(i, top)
            .into_iter()
            .map(|(term, score)| json!({ "term": term, "score": (score * 1000.0).round() / 1000.0 }))
            .collect();
        packages.insert(pkg.clone(), Value::Array(terms));
    }
    json!({ "packages": Value::Object(packages) })
}

/// Resolve the optional `--config` lens then compute the topics document. Shared by
/// the CLI command and the MCP `topics` tool.
pub fn topics_document(
    elements: &[RawElement],
    type_filter: Option<&str>,
    top: usize,
    config: Option<&str>,
) -> Result<Value, String> {
    use syscribe_model::projection::{project, resolve_selection, SelectionOutcome};
    let projected: Option<Vec<RawElement>> = match config {
        None => None,
        Some(c) => match resolve_selection(elements, c) {
            SelectionOutcome::Dormant => None,
            SelectionOutcome::Resolved(sel) => Some(project(elements, &sel)),
            SelectionOutcome::Error(m) => return Err(m),
        },
    };
    let view: &[RawElement] = projected.as_deref().unwrap_or(elements);
    Ok(compute_topics(view, type_filter, top))
}

/// The `topics` command. Returns the exit code (0 ok, 1 on an unresolvable `--config`).
pub fn cmd_topics(
    elements: &[RawElement],
    type_filter: Option<&str>,
    top: usize,
    config: Option<&str>,
    json: bool,
) -> i32 {
    let doc = match topics_document(elements, type_filter, top, config) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("Error: {msg}");
            return 1;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
    } else {
        let packages = doc["packages"].as_object().unwrap();
        if packages.is_empty() {
            println!("No elements to analyse.");
        }
        for (pkg, terms) in packages {
            println!("## {pkg}");
            for t in terms.as_array().unwrap() {
                println!("  {} ({})", t["term"].as_str().unwrap_or(""), t["score"]);
            }
        }
    }
    0
}

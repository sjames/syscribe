//! `syscribe search-text` — ranked full-text search over normative text
//! (REQ-TRS-SEARCH-001).
//!
//! Builds an in-memory inverted index over the tokenised element bodies (+ the
//! `name` label) and scores with Okapi BM25, returning the best matches ordered by
//! relevance with a snippet marking the first hit. No external search engine and no
//! persisted index (ADR-SYS-SCAN-001): the index is built in memory from the parsed
//! model. Complements the identifier-fuzzy `search` tool, which is unchanged.

use std::collections::HashMap;

use serde_json::{json, Value};
use syscribe_model::element::RawElement;

use crate::query::type_label;

// Standard Okapi BM25 parameters.
const K1: f64 = 1.2;
const B: f64 = 0.75;
/// Half-width (in characters) of the snippet window around the first hit.
const SNIPPET_HALF: usize = 80;

/// Lowercase alphanumeric-run tokeniser.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

/// The searchable text of an element: its display name plus its Markdown body.
fn doc_text(e: &RawElement) -> String {
    match &e.frontmatter.name {
        Some(n) => format!("{n}\n{}", e.doc),
        None => e.doc.clone(),
    }
}

struct Index<'a> {
    docs: Vec<&'a RawElement>,
    /// term → postings list of (doc index, term frequency).
    postings: HashMap<String, Vec<(usize, u32)>>,
    /// token count per doc (index-aligned with `docs`).
    lengths: Vec<f64>,
    avg_len: f64,
}

impl<'a> Index<'a> {
    /// Build the inverted index over the given elements (already scoped/projected).
    fn build(elements: &'a [RawElement]) -> Index<'a> {
        let mut docs = Vec::new();
        let mut postings: HashMap<String, Vec<(usize, u32)>> = HashMap::new();
        let mut lengths = Vec::new();
        for e in elements {
            let toks = tokenize(&doc_text(e));
            if toks.is_empty() {
                continue; // nothing to index (no body, no name)
            }
            let idx = docs.len();
            docs.push(e);
            lengths.push(toks.len() as f64);
            let mut tf: HashMap<String, u32> = HashMap::new();
            for t in toks {
                *tf.entry(t).or_insert(0) += 1;
            }
            for (term, freq) in tf {
                postings.entry(term).or_default().push((idx, freq));
            }
        }
        let avg_len = if lengths.is_empty() {
            0.0
        } else {
            lengths.iter().sum::<f64>() / lengths.len() as f64
        };
        Index { docs, postings, lengths, avg_len }
    }

    /// BM25-score every doc matching any query term; return (doc index, score)
    /// pairs ordered by descending score.
    fn score(&self, query_terms: &[String]) -> Vec<(usize, f64)> {
        let n = self.docs.len() as f64;
        let mut scores: HashMap<usize, f64> = HashMap::new();
        // Distinct query terms (repeats do not multiply the idf weight).
        let mut seen = std::collections::HashSet::new();
        for term in query_terms {
            if !seen.insert(term) {
                continue;
            }
            let Some(posting) = self.postings.get(term) else { continue };
            let df = posting.len() as f64;
            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
            for &(doc, freq) in posting {
                let tf = freq as f64;
                let denom = tf + K1 * (1.0 - B + B * self.lengths[doc] / self.avg_len);
                *scores.entry(doc).or_insert(0.0) += idf * (tf * (K1 + 1.0)) / denom;
            }
        }
        let mut ranked: Vec<(usize, f64)> = scores.into_iter().collect();
        // Descending score, tie-broken by qualified name so the ordering is stable
        // and independent of the input element order (CLI walk vs MCP store load).
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| self.docs[a.0].qualified_name.cmp(&self.docs[b.0].qualified_name))
        });
        ranked
    }
}

/// A bounded window of the element body around the first query-term hit, with the
/// matched terms marked `**like this**`. Falls back to the leading window when no
/// term is found in the body (it may have matched only the name).
fn snippet(e: &RawElement, query_terms: &[String]) -> String {
    let body = doc_text(e);
    let body_lc = body.to_lowercase();
    let chars: Vec<char> = body.chars().collect();
    // First hit position (char index) of any query term.
    let first = query_terms
        .iter()
        .filter_map(|t| body_lc.find(t.as_str()).map(|byte| body_lc[..byte].chars().count()))
        .min()
        .unwrap_or(0);
    let start = first.saturating_sub(SNIPPET_HALF);
    let end = (first + SNIPPET_HALF).min(chars.len());
    let mut window: String = chars[start..end].iter().collect();
    window = window.split_whitespace().collect::<Vec<_>>().join(" ");
    // Mark matched terms (case-insensitive, whole-token best-effort).
    for t in query_terms {
        window = mark_term(&window, t);
    }
    let mut out = String::new();
    if start > 0 {
        out.push('…');
    }
    out.push_str(&window);
    if end < chars.len() {
        out.push('…');
    }
    out
}

/// Wrap case-insensitive occurrences of `term` in `**…**`, preserving original case.
fn mark_term(text: &str, term: &str) -> String {
    if term.is_empty() {
        return text.to_string();
    }
    let text_lc = text.to_lowercase();
    let mut out = String::new();
    let mut i = 0;
    let bytes_ok = |s: &str, at: usize| s.is_char_boundary(at);
    while i < text.len() {
        if bytes_ok(&text_lc, i) {
            if let Some(rel) = text_lc[i..].find(term) {
                let hit = i + rel;
                if bytes_ok(text, hit) && bytes_ok(text, hit + term.len()) {
                    out.push_str(&text[i..hit]);
                    out.push_str("**");
                    out.push_str(&text[hit..hit + term.len()]);
                    out.push_str("**");
                    i = hit + term.len();
                    continue;
                }
            }
        }
        // no further match: append the rest and stop.
        out.push_str(&text[i..]);
        break;
    }
    out
}

/// Compute the ranked search document `{ total, results }` over `elements` (already
/// projection-applied). `Err` on an empty query. `type_filter`/`status_filter`
/// restrict the indexed set before scoring.
pub fn search_document(
    elements: &[RawElement],
    query: &str,
    type_filter: Option<&str>,
    status_filter: Option<&str>,
    limit: usize,
) -> Result<Value, String> {
    let terms = tokenize(query);
    if terms.is_empty() {
        return Err("search-text requires a non-empty query".to_string());
    }
    // Scope the indexed set by type/status before building the index.
    let scoped: Vec<RawElement> = elements
        .iter()
        .filter(|e| {
            type_filter.is_none_or(|t| e.frontmatter.element_type.as_ref().map(type_label) == Some(t))
                && status_filter.is_none_or(|s| e.frontmatter.status.as_deref() == Some(s))
        })
        .cloned()
        .collect();
    let index = Index::build(&scoped);
    let ranked = index.score(&terms);
    let total = ranked.len();
    let results: Vec<Value> = ranked
        .iter()
        .take(limit)
        .map(|&(doc, score)| {
            let e = index.docs[doc];
            json!({
                "id": e.frontmatter.id,
                "qname": e.qualified_name,
                "type": e.frontmatter.element_type.as_ref().map(type_label),
                "score": (score * 1000.0).round() / 1000.0,
                "snippet": snippet(e, &terms),
            })
        })
        .collect();
    Ok(json!({ "total": total, "results": results }))
}

/// The `search-text` command. Resolves the optional `--config` lens, runs the
/// ranked search, prints it (text or `--json`), and returns the exit code (0 ok, 1
/// on an empty query or unresolvable `--config`).
pub fn cmd_search_text(
    elements: &[RawElement],
    query: &str,
    type_filter: Option<&str>,
    status_filter: Option<&str>,
    config: Option<&str>,
    limit: usize,
    json: bool,
) -> i32 {
    use syscribe_model::projection::{project, resolve_selection, SelectionOutcome};
    let projected: Option<Vec<RawElement>> = match config {
        None => None,
        Some(c) => match resolve_selection(elements, c) {
            SelectionOutcome::Dormant => None,
            SelectionOutcome::Resolved(sel) => Some(project(elements, &sel)),
            SelectionOutcome::Error(m) => {
                eprintln!("Error: {m}");
                return 1;
            }
        },
    };
    let view: &[RawElement] = projected.as_deref().unwrap_or(elements);
    let doc = match search_document(view, query, type_filter, status_filter, limit) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("Error: {msg}");
            return 1;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
    } else {
        let results = doc["results"].as_array().unwrap();
        if results.is_empty() {
            println!("No matches.");
        }
        for r in results {
            let id = r["id"].as_str().unwrap_or_else(|| r["qname"].as_str().unwrap_or("?"));
            println!("{}  (score {})", id, r["score"]);
            println!("    {}", r["snippet"].as_str().unwrap_or(""));
        }
    }
    0
}

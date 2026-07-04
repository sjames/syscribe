//! `syscribe clusters` — topical clustering via TF-IDF cosine k-means
//! (REQ-TRS-SEARCH-003).
//!
//! Vectorises each element as its TF-IDF vector and groups by cosine similarity
//! using k-means, surfacing cross-package themes. Deterministic and offline
//! (ADR-SYS-SCAN-002): bag-of-words TF-IDF (no neural embeddings), and a
//! farthest-first centroid initialisation seeded from a fixed (qname-sorted)
//! element order — no random seed — so runs are reproducible.

use std::collections::HashMap;

use serde_json::{json, Value};
use syscribe_model::element::RawElement;

use crate::query::type_label;
use crate::textstats::{add_into, cosine, top_terms_of, Corpus};

const MAX_ITERS: usize = 20;
/// Number of centroid terms used as a cluster label.
const LABEL_TERMS: usize = 5;

fn disp_id(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Cosine distance (1 − similarity) of two L2-normalised sparse vectors.
fn distance(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    1.0 - cosine(a, b)
}

/// Normalise a summed sparse vector in place (L2).
fn normalize(v: &mut HashMap<String, f64>) {
    let norm = v.values().map(|w| w * w).sum::<f64>().sqrt();
    if norm > 0.0 {
        for w in v.values_mut() {
            *w /= norm;
        }
    }
}

/// Deterministic farthest-first initialisation: start from doc 0 (qname-sorted),
/// then repeatedly add the doc whose minimum distance to the chosen centroids is
/// greatest (ties broken by the earlier doc, i.e. lower qname).
fn init_centroids(vecs: &[HashMap<String, f64>], k: usize) -> Vec<usize> {
    let mut chosen = vec![0usize];
    while chosen.len() < k {
        let mut best_doc = 0usize;
        let mut best_min = -1.0f64;
        for (i, v) in vecs.iter().enumerate() {
            if chosen.contains(&i) {
                continue;
            }
            let min_d = chosen
                .iter()
                .map(|&c| distance(v, &vecs[c]))
                .fold(f64::INFINITY, f64::min);
            if min_d > best_min {
                best_min = min_d;
                best_doc = i;
            }
        }
        if chosen.contains(&best_doc) {
            break; // all remaining docs already chosen (degenerate)
        }
        chosen.push(best_doc);
    }
    chosen
}

/// Compute the clusters document `{ k, clusters: [ { label, size, members } … ] }`
/// over `view` (already projection-applied). `k` is clamped to the element count.
pub fn compute_clusters(view: &[RawElement], type_filter: Option<&str>, k: usize) -> Value {
    let want = type_filter.unwrap_or("Requirement");
    // Deterministic element order: sort by qualified name so init is stable
    // independent of the input (CLI walk vs MCP store) order.
    let mut elems: Vec<&RawElement> = view
        .iter()
        .filter(|e| e.frontmatter.element_type.as_ref().map(type_label) == Some(want))
        .collect();
    elems.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));

    let n = elems.len();
    let k_eff = k.min(n).max(if n == 0 { 0 } else { 1 });
    if k_eff == 0 {
        return json!({ "k": 0, "clusters": [] });
    }

    let texts: Vec<String> = elems.iter().map(|e| crate::textstats::element_text(e)).collect();
    let corpus = Corpus::build(&texts);
    let vecs: Vec<HashMap<String, f64>> = (0..n).map(|i| corpus.tfidf_vec(i)).collect();

    // Initialise centroids (as copies of the chosen docs' vectors).
    let seed = init_centroids(&vecs, k_eff);
    let mut centroids: Vec<HashMap<String, f64>> = seed.iter().map(|&i| vecs[i].clone()).collect();
    let mut assign = vec![0usize; n];

    for _ in 0..MAX_ITERS {
        // Assignment: nearest centroid by cosine (ties → lower cluster index).
        let mut changed = false;
        for (i, v) in vecs.iter().enumerate() {
            let mut best_c = 0usize;
            let mut best_sim = f64::NEG_INFINITY;
            for (c, cen) in centroids.iter().enumerate() {
                let sim = cosine(v, cen);
                if sim > best_sim {
                    best_sim = sim;
                    best_c = c;
                }
            }
            if assign[i] != best_c {
                assign[i] = best_c;
                changed = true;
            }
        }
        // Update: centroid = normalised mean of its members. An empty cluster keeps
        // its previous centroid (stable).
        let mut new_centroids: Vec<HashMap<String, f64>> = vec![HashMap::new(); k_eff];
        let mut counts = vec![0usize; k_eff];
        for (i, &c) in assign.iter().enumerate() {
            add_into(&mut new_centroids[c], &vecs[i]);
            counts[c] += 1;
        }
        for c in 0..k_eff {
            if counts[c] == 0 {
                new_centroids[c] = centroids[c].clone();
            } else {
                normalize(&mut new_centroids[c]);
            }
        }
        centroids = new_centroids;
        if !changed {
            break;
        }
    }

    // Emit clusters in index order; members sorted by qname (already, via elems order).
    let clusters: Vec<Value> = (0..k_eff)
        .map(|c| {
            let members: Vec<String> = (0..n)
                .filter(|&i| assign[i] == c)
                .map(|i| disp_id(elems[i]))
                .collect();
            let label: Vec<String> =
                top_terms_of(&centroids[c], LABEL_TERMS).into_iter().map(|(t, _)| t).collect();
            json!({ "label": label, "size": members.len(), "members": members })
        })
        .collect();

    json!({ "k": k_eff, "clusters": clusters })
}

/// Resolve the optional `--config` lens then compute the clusters document. Shared
/// by the CLI command and the MCP `clusters` tool. `Err` when `k < 1`.
pub fn clusters_document(
    elements: &[RawElement],
    type_filter: Option<&str>,
    k: usize,
    config: Option<&str>,
) -> Result<Value, String> {
    if k < 1 {
        return Err("--k must be at least 1".to_string());
    }
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
    Ok(compute_clusters(view, type_filter, k))
}

/// The `clusters` command. Returns the exit code (0 ok, 1 on `--k < 1` or an
/// unresolvable `--config`).
pub fn cmd_clusters(
    elements: &[RawElement],
    type_filter: Option<&str>,
    k: usize,
    config: Option<&str>,
    json: bool,
) -> i32 {
    let doc = match clusters_document(elements, type_filter, k, config) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("Error: {msg}");
            return 1;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
    } else {
        for (i, c) in doc["clusters"].as_array().unwrap().iter().enumerate() {
            let label = c["label"].as_array().unwrap().iter().filter_map(|t| t.as_str()).collect::<Vec<_>>().join(", ");
            println!("## Cluster {i} [{}] ({} members)", label, c["size"]);
            for m in c["members"].as_array().unwrap() {
                println!("  {}", m.as_str().unwrap_or(""));
            }
        }
    }
    0
}

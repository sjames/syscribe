//! `syscribe summarize` — deterministic hierarchical content digest with a
//! content-addressed cache (REQ-TRS-OUT-023).
//!
//! Bottom-up per-package rollup: for each package (namespace) node — a requirement
//! count, a status split (`stats`), an "about" term line (`topics` TF-IDF), and the
//! one-line extract (`digest`) of a bounded set of representative requirements —
//! nested through the package hierarchy. Extractive and offline (ADR-SYS-SCAN-002):
//! the tool emits a structured extract, not prose. Each node's own fields are cached
//! under `.syscribe/cache/summaries.json` keyed by a content hash of its subtree, so
//! an unchanged subtree is served from cache (incremental).

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde_json::{json, Value};
use syscribe_model::element::{ElementType, RawElement};

use crate::textstats::Corpus;

/// Requirement `status:` values reported, in lifecycle order (matches `audit`).
const STATUS_ORDER: [&str; 5] = ["draft", "review", "approved", "implemented", "verified"];
/// Max representative requirements listed directly at a node.
const REPRESENTATIVE: usize = 5;
/// Top "about" terms per node.
const ABOUT_TERMS: usize = 8;

fn is_requirement(e: &RawElement) -> bool {
    e.frontmatter.element_type.as_ref() == Some(&ElementType::Requirement)
}

fn disp_id(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Package path segments of an element (its qualified name minus the last segment).
/// An element directly under the model root has an empty path.
fn pkg_path(e: &RawElement) -> Vec<String> {
    let mut segs: Vec<String> = e.qualified_name.split("::").map(str::to_string).collect();
    segs.pop(); // drop the element's own identity segment
    segs
}

/// The first non-empty body line, whitespace-collapsed and length-bounded (mirrors
/// the `digest` one-line rule).
fn one_line(doc: &str) -> String {
    let line = doc.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    let collapsed = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() > 160 {
        format!("{}…", collapsed.chars().take(160).collect::<String>())
    } else {
        collapsed
    }
}

fn status_split(reqs: &[&RawElement]) -> Value {
    let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
    let mut other = 0u32;
    for r in reqs {
        match r.frontmatter.status.as_deref() {
            Some(s) if STATUS_ORDER.contains(&s) => *counts.entry(STATUS_ORDER.iter().find(|x| **x == s).unwrap()).or_insert(0) += 1,
            _ => other += 1,
        }
    }
    let mut m = serde_json::Map::new();
    for s in STATUS_ORDER {
        if let Some(c) = counts.get(s) {
            m.insert(s.to_string(), json!(c));
        }
    }
    if other > 0 {
        m.insert("other".to_string(), json!(other));
    }
    Value::Object(m)
}

/// A stable content hash of a subtree's requirements (their ids and bodies), for the
/// cache key. Non-cryptographic — a collision at worst forces a recompute.
fn subtree_hash(reqs: &[&RawElement]) -> String {
    // Sort by qname so the hash is order-independent.
    let mut items: Vec<(&str, &str)> =
        reqs.iter().map(|r| (r.qualified_name.as_str(), r.doc.as_str())).collect();
    items.sort();
    let mut h = DefaultHasher::new();
    for (q, d) in items {
        q.hash(&mut h);
        d.hash(&mut h);
    }
    format!("{:016x}", h.finish())
}

/// Bump when the summarisation LOGIC changes (e.g. tokeniser, term selection) so a
/// stale cache — whose content hash still matches — is invalidated across versions.
const CACHE_VERSION: u32 = 1;

/// The persisted per-package cache: `{ version, entries: { qname → { hash, own } } }`
/// where `own` is a node's computed fields excluding its children.
struct Cache {
    map: serde_json::Map<String, Value>,
    dirty: bool,
}

impl Cache {
    fn load(path: &Path, bypass: bool) -> Cache {
        if bypass {
            return Cache { map: serde_json::Map::new(), dirty: true };
        }
        let doc = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok());
        // Invalidate on a version mismatch (a logic change the content hash can't see).
        let map = doc
            .filter(|v| v.get("version").and_then(|n| n.as_u64()) == Some(CACHE_VERSION as u64))
            .and_then(|v| v.get("entries").and_then(|e| e.as_object().cloned()))
            .unwrap_or_default();
        Cache { map, dirty: false }
    }

    /// The cached `own` fields for `qname` if its stored hash matches.
    fn get(&self, qname: &str, hash: &str) -> Option<Value> {
        let e = self.map.get(qname)?;
        if e.get("hash").and_then(|h| h.as_str()) == Some(hash) {
            e.get("own").cloned()
        } else {
            None
        }
    }

    fn put(&mut self, qname: &str, hash: &str, own: Value) {
        self.map.insert(qname.to_string(), json!({ "hash": hash, "own": own }));
        self.dirty = true;
    }

    fn save(&self, path: &Path) {
        if !self.dirty {
            return;
        }
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let doc = json!({ "version": CACHE_VERSION, "entries": Value::Object(self.map.clone()) });
        if let Ok(s) = serde_json::to_string(&doc) {
            let _ = std::fs::write(path, s);
        }
    }
}

/// Compute a node's OWN fields (count, statusSplit, terms, representative) — the
/// unit that is cached. `subtree` are all requirements at or below this node;
/// `direct` are those exactly in this package.
fn compute_own(subtree: &[&RawElement], direct: &[&RawElement]) -> Value {
    // "about" terms: top TF-IDF of the subtree's requirement texts, treating each
    // requirement as a document so distinctive vocabulary rises.
    let texts: Vec<String> = subtree.iter().map(|e| crate::textstats::element_text(e)).collect();
    let corpus = Corpus::build(&texts);
    let mut acc: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for i in 0..corpus.len() {
        crate::textstats::add_into(&mut acc, &corpus.tfidf_vec(i));
    }
    let terms: Vec<String> =
        crate::textstats::top_terms_of(&acc, ABOUT_TERMS).into_iter().map(|(t, _)| t).collect();

    // Representative: the one-line extract of up to N direct requirements (qname order).
    let mut direct_sorted: Vec<&&RawElement> = direct.iter().collect();
    direct_sorted.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
    let representative: Vec<Value> = direct_sorted
        .iter()
        .take(REPRESENTATIVE)
        .map(|e| json!({ "id": disp_id(e), "text": one_line(&e.doc) }))
        .collect();

    json!({
        "count": subtree.len(),
        "statusSplit": status_split(subtree),
        "terms": terms,
        "representative": representative,
    })
}

/// Recursively build the summary node for package `path`. `reqs` is the full
/// requirement set; `depth` bounds how deep children are emitted (`None` = no bound).
fn build_node(
    path: &[String],
    reqs: &[&RawElement],
    depth: Option<usize>,
    cache: &mut Cache,
) -> Value {
    let qname = if path.is_empty() { "(root)".to_string() } else { path.join("::") };
    let subtree: Vec<&RawElement> = reqs
        .iter()
        .filter(|r| {
            let p = pkg_path(r);
            p.len() >= path.len() && p[..path.len()] == *path
        })
        .copied()
        .collect();
    let direct: Vec<&RawElement> =
        subtree.iter().filter(|r| pkg_path(r) == *path).copied().collect();

    // Own fields: cache by subtree content hash.
    let hash = subtree_hash(&subtree);
    let own = match cache.get(&qname, &hash) {
        Some(cached) => cached,
        None => {
            let computed = compute_own(&subtree, &direct);
            cache.put(&qname, &hash, computed.clone());
            computed
        }
    };

    // Children: the distinct next segments below this path (deterministic order).
    let mut child_segs: Vec<String> = subtree
        .iter()
        .filter_map(|r| {
            let p = pkg_path(r);
            (p.len() > path.len()).then(|| p[path.len()].clone())
        })
        .collect();
    child_segs.sort();
    child_segs.dedup();

    let children: Vec<Value> = match depth {
        Some(0) => Vec::new(),
        _ => child_segs
            .iter()
            .map(|seg| {
                let mut child_path = path.to_vec();
                child_path.push(seg.clone());
                build_node(&child_path, reqs, depth.map(|d| d - 1), cache)
            })
            .collect(),
    };

    let mut node = own.as_object().cloned().unwrap_or_default();
    node.insert("qname".to_string(), json!(qname));
    node.insert("children".to_string(), Value::Array(children));
    Value::Object(node)
}

/// Compute the summary document over `view`, using/refreshing the cache at
/// `model_root/.syscribe/cache/summaries.json`. `scope` restricts to a subtree.
/// `Err` on an unresolvable `--scope`.
pub fn compute_summary(
    view: &[RawElement],
    model_root: &Path,
    scope: Option<&str>,
    depth: Option<usize>,
    no_cache: bool,
) -> Result<Value, String> {
    let reqs: Vec<&RawElement> = view.iter().filter(|e| is_requirement(e)).collect();

    // Resolve the root path from --scope (a package qname), if given.
    let root_path: Vec<String> = match scope {
        None => Vec::new(),
        Some(s) => {
            let segs: Vec<String> = s.split("::").map(str::to_string).collect();
            let exists = reqs.iter().any(|r| {
                let p = pkg_path(r);
                p.len() >= segs.len() && p[..segs.len()] == segs[..]
            });
            if !exists {
                return Err(format!("--scope '{s}' matches no package"));
            }
            segs
        }
    };

    let cache_path = model_root.join(".syscribe").join("cache").join("summaries.json");
    let mut cache = Cache::load(&cache_path, no_cache);
    let doc = build_node(&root_path, &reqs, depth, &mut cache);
    cache.save(&cache_path);
    Ok(doc)
}

/// Resolve the optional `--config` lens then compute the summary. Shared by the CLI
/// command and the MCP `summarize` tool.
pub fn summarize_document(
    elements: &[RawElement],
    model_root: &Path,
    scope: Option<&str>,
    depth: Option<usize>,
    no_cache: bool,
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
    compute_summary(view, model_root, scope, depth, no_cache)
}

/// The `summarize` command. Returns the exit code (0 ok, 1 on an unresolvable
/// `--scope`/`--config`).
#[allow(clippy::too_many_arguments)]
pub fn cmd_summarize(
    elements: &[RawElement],
    model_root: &Path,
    scope: Option<&str>,
    depth: Option<usize>,
    no_cache: bool,
    config: Option<&str>,
    json: bool,
) -> i32 {
    let doc = match summarize_document(elements, model_root, scope, depth, no_cache, config) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("Error: {msg}");
            return 1;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
    } else {
        print_node(&doc, 0);
    }
    0
}

fn print_node(node: &Value, indent: usize) {
    let pad = "  ".repeat(indent);
    let qname = node["qname"].as_str().unwrap_or("?");
    let count = node["count"].as_u64().unwrap_or(0);
    let terms = node["terms"].as_array().map(|a| {
        a.iter().filter_map(|t| t.as_str()).collect::<Vec<_>>().join(", ")
    }).unwrap_or_default();
    println!("{pad}{qname}  ({count} reqs)  about: {terms}");
    if let Some(reps) = node["representative"].as_array() {
        for r in reps {
            println!("{pad}  - {}: {}", r["id"].as_str().unwrap_or("?"), r["text"].as_str().unwrap_or(""));
        }
    }
    if let Some(children) = node["children"].as_array() {
        for c in children {
            print_node(c, indent + 1);
        }
    }
}

//! `syscribe digest` — token-budgeted, one-line-per-requirement bulk view
//! (REQ-TRS-OUT-022).
//!
//! The "dump the slice" companion to `stats` (REQ-TRS-OUT-021): after an LLM
//! narrows with the `stats` facets, `digest` streams the matching native
//! `Requirement`s as ~30-token rows. Distinct from `export`, which dumps the full
//! element (frontmatter + computed indices). Reuses the `stats`/`projection`
//! scoping (`--where`/`--status`/`--tag`/`--config`) and the coverage notion of
//! `verified`. One document producer backs both the CLI and the MCP `digest` tool.

use serde_json::{json, Value};
use syscribe_model::element::{ElementType, RawElement};

use crate::query::{self, CustomWhere};

/// Max length of the one-line `text` summary before truncation.
const TEXT_MAX: usize = 200;

/// Scoping / paging options for a `digest` run.
#[derive(Default)]
pub struct DigestOptions<'a> {
    pub wheres: &'a [CustomWhere],
    pub status: Option<&'a str>,
    pub tag: Option<&'a str>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

fn is_type(e: &RawElement, t: ElementType) -> bool {
    e.frontmatter.element_type.as_ref() == Some(&t)
}

/// Cross-reference identity keys an inbound `verifies:` may use.
fn keys(e: &RawElement) -> Vec<String> {
    let mut k = vec![e.qualified_name.clone()];
    if let Some(id) = &e.frontmatter.id {
        k.push(id.clone());
    }
    k
}

fn passes_filters(e: &RawElement, opts: &DigestOptions) -> bool {
    if let Some(s) = opts.status {
        if e.frontmatter.status.as_deref() != Some(s) {
            return false;
        }
    }
    if let Some(t) = opts.tag {
        if !e.frontmatter.tags.iter().flatten().any(|x| x == t) {
            return false;
        }
    }
    for pred in opts.wheres {
        if !query::custom_or_extra_matches(e, pred) {
            return false;
        }
    }
    true
}

/// The first non-empty body line, whitespace-collapsed and length-bounded.
fn one_line(doc: &str) -> String {
    let line = doc.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    let collapsed = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() > TEXT_MAX {
        let truncated: String = collapsed.chars().take(TEXT_MAX).collect();
        format!("{truncated}…")
    } else {
        collapsed
    }
}

fn digest_row(e: &RawElement, verified: bool) -> Value {
    let fm = &e.frontmatter;
    let mut obj = serde_json::Map::new();
    obj.insert("id".into(), json!(fm.id.clone().unwrap_or_else(|| e.qualified_name.clone())));
    if let Some(name) = &fm.name {
        obj.insert("name".into(), json!(name));
    }
    obj.insert("status".into(), json!(fm.status));
    obj.insert("reqDomain".into(), json!(fm.req_domain));
    if let Some(sil) = fm.sil_level {
        obj.insert("sil".into(), json!(sil));
    }
    if let Some(asil) = &fm.asil_level {
        obj.insert("asil".into(), json!(asil));
    }
    obj.insert("text".into(), json!(one_line(&e.doc)));
    obj.insert("verified".into(), json!(verified));
    Value::Object(obj)
}

/// Compute the digest document `{ total, offset, rows }` over `view` (already
/// projection-applied, if any).
pub fn compute_digest(view: &[RawElement], opts: &DigestOptions) -> Value {
    // Verification: a requirement is `verified` when an active (non-draft) TestCase
    // in the view targets it (reuses the coverage notion, not a new computation).
    let active_tc_targets: Vec<Vec<String>> = view
        .iter()
        .filter(|e| is_type(e, ElementType::TestCase))
        .filter(|e| e.frontmatter.status.as_deref() != Some("draft"))
        .map(|e| e.frontmatter.verifies.clone().unwrap_or_default())
        .collect();
    let is_verified = |r: &RawElement| -> bool {
        let rkeys = keys(r);
        active_tc_targets.iter().any(|ver| ver.iter().any(|v| rkeys.iter().any(|k| k == v)))
    };

    let matched: Vec<&RawElement> = view
        .iter()
        .filter(|e| is_type(e, ElementType::Requirement) && passes_filters(e, opts))
        .collect();
    let total = matched.len();
    let offset = opts.offset.unwrap_or(0);
    let rows: Vec<Value> = matched
        .iter()
        .skip(offset)
        .take(opts.limit.unwrap_or(usize::MAX))
        .map(|e| digest_row(e, is_verified(e)))
        .collect();

    json!({ "total": total, "offset": offset, "rows": rows })
}

/// Resolve the optional `--config` lens then compute the digest document. Shared by
/// the CLI command and the MCP `digest` tool.
pub fn digest_document(
    elements: &[RawElement],
    config: Option<&str>,
    opts: &DigestOptions,
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
    Ok(compute_digest(view, opts))
}

/// The `digest` command. Default output is NDJSON (one compact row per line);
/// `--json` emits the `{ total, offset, rows }` document. Returns the exit code
/// (0 ok, 1 on an unresolvable `--config`).
pub fn cmd_digest(
    elements: &[RawElement],
    config: Option<&str>,
    opts: &DigestOptions,
    json: bool,
) -> i32 {
    let doc = match digest_document(elements, config, opts) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("Error: {msg}");
            return 1;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
    } else {
        for row in doc["rows"].as_array().unwrap() {
            println!("{}", serde_json::to_string(row).unwrap());
        }
    }
    0
}

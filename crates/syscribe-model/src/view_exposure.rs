//! Which model elements a `View`/`ViewDef` exposes (GH #205).
//!
//! `expose:` entries are a qualified name or import pattern (`Pkg`, `Pkg::*`,
//! `Pkg::**`), a `{target|ref, isRecursive, filter}` map; the view's own
//! `filterCondition:` (and each entry's `filter:`) narrows the result. A filter
//! understands `@MetadataName` (the element carries that metadata application),
//! `not`, `and`, `or`; any other clause is reported in `notes` and treated as true.
//! Exposure is inherited through `typedBy:`/`supertype:` ViewDefs.

use std::collections::HashSet;

use crate::element::{metadata_applications, RawElement};
use crate::resolver::Resolver;

#[derive(Debug, Default)]
pub struct Exposure<'a> {
    /// Exposed elements, in qualified-name order, without duplicates.
    pub items: Vec<&'a RawElement>,
    /// `expose:` targets that resolved to nothing.
    pub unresolved: Vec<String>,
    /// Filter clauses that could not be evaluated.
    pub notes: Vec<String>,
}

/// One parsed `expose:` entry.
pub struct ExposeEntry {
    pub target: String,
    pub recursive: bool,
    pub filter: Option<String>,
}

pub fn expose_entries(view: &RawElement) -> Vec<ExposeEntry> {
    view.frontmatter
        .expose
        .iter()
        .flatten()
        .filter_map(|v| match v {
            serde_yaml::Value::String(s) => {
                Some(ExposeEntry { target: s.clone(), recursive: false, filter: None })
            }
            serde_yaml::Value::Mapping(m) => {
                let g = |k: &str| m.get(serde_yaml::Value::from(k));
                let target = g("target").or_else(|| g("ref")).and_then(|v| v.as_str())?.to_string();
                Some(ExposeEntry {
                    target,
                    recursive: g("isRecursive").and_then(|v| v.as_bool()).unwrap_or(false),
                    filter: g("filter").and_then(|v| v.as_str()).map(str::to_string),
                })
            }
            _ => None,
        })
        .collect()
}

/// Split `Pkg::*` / `Pkg::**` into (`Pkg`, wildcard?, recursive?).
fn split_pattern(t: &str) -> (&str, bool, bool) {
    let t = t.trim();
    if let Some(p) = t.strip_suffix("::**") {
        (p, true, true)
    } else if let Some(p) = t.strip_suffix("::*") {
        (p, true, false)
    } else {
        (t, false, false)
    }
}

/// Resolve the package/element an expose target names, relative to `view`.
pub fn resolve_target<'a>(
    elements: &'a [RawElement],
    resolver: &Resolver,
    view: &RawElement,
    target: &str,
) -> Option<&'a RawElement> {
    let (base, _, _) = split_pattern(target);
    if base.is_empty() {
        return resolver.get(elements, "");
    }
    resolver
        .resolve_scoped_ref(elements, &view.qualified_name, base)
        .or_else(|| resolver.resolve_ref(elements, base))
}

fn has_metadata(e: &RawElement, name: &str) -> bool {
    metadata_applications(&e.frontmatter.metadata)
        .iter()
        .any(|a| a.def == name || a.def.rsplit("::").next() == Some(name))
}

/// Evaluate a filter expression against `e`. Unknown clauses push a note and pass.
fn eval_filter(e: &RawElement, expr: &str, notes: &mut Vec<String>) -> bool {
    expr.split(" or ").any(|disj| {
        disj.split(" and ").all(|clause| {
            let c = clause.trim();
            let (neg, c) = match c.strip_prefix("not ") {
                Some(r) => (true, r.trim()),
                None => (false, c),
            };
            let v = match c.strip_prefix('@') {
                Some(m) if !m.contains(|ch: char| ch.is_whitespace() || "<>=!()".contains(ch)) => {
                    has_metadata(e, m)
                }
                _ => {
                    let n = format!("filter clause '{c}' is not evaluated (only `@Metadata`, `not`, `and`, `or`)");
                    if !notes.contains(&n) {
                        notes.push(n);
                    }
                    true
                }
            };
            v != neg
        })
    })
}

/// The view definitions whose exposure a view inherits: itself, then its
/// `typedBy:`/`supertype:` chain (cycle-safe).
fn view_chain<'a>(elements: &'a [RawElement], resolver: &Resolver, view: &'a RawElement) -> Vec<&'a RawElement> {
    let mut out = vec![view];
    let mut seen: HashSet<&str> = HashSet::from([view.qualified_name.as_str()]);
    let mut i = 0;
    while i < out.len() {
        let cur = out[i];
        for v in [cur.frontmatter.typed_by.as_ref(), cur.frontmatter.supertype.as_ref()].into_iter().flatten() {
            let names: Vec<&str> = match v {
                serde_yaml::Value::String(s) => vec![s.as_str()],
                serde_yaml::Value::Sequence(s) => s.iter().filter_map(|x| x.as_str()).collect(),
                _ => vec![],
            };
            for n in names {
                if let Some(t) = resolver.resolve_scoped_ref(elements, &cur.qualified_name, n) {
                    if seen.insert(t.qualified_name.as_str()) {
                        out.push(t);
                    }
                }
            }
        }
        i += 1;
    }
    out
}

pub fn exposure<'a>(elements: &'a [RawElement], resolver: &Resolver, view: &'a RawElement) -> Exposure<'a> {
    let mut ex = Exposure::default();
    let mut seen: HashSet<&str> = HashSet::new();
    let chain = view_chain(elements, resolver, view);
    let global_filters: Vec<&str> =
        chain.iter().filter_map(|v| v.frontmatter.filter_condition.as_deref()).collect();
    for v in &chain {
        for entry in expose_entries(v) {
            let Some(base) = resolve_target(elements, resolver, v, &entry.target) else {
                ex.unresolved.push(entry.target.clone());
                continue;
            };
            let (_, wildcard, deep_pat) = split_pattern(&entry.target);
            let prefix = if base.qualified_name.is_empty() { String::new() } else { format!("{}::", base.qualified_name) };
            let recursive = deep_pat || entry.recursive;
            for e in elements {
                let q = e.qualified_name.as_str();
                let inside = if wildcard || recursive {
                    q.strip_prefix(prefix.as_str())
                        .is_some_and(|rest| !rest.is_empty() && (recursive || !rest.contains("::")))
                } else {
                    q == base.qualified_name
                };
                if !inside || e.file_path.is_empty() {
                    continue;
                }
                let pass = entry
                    .filter
                    .iter()
                    .map(String::as_str)
                    .chain(global_filters.iter().copied())
                    .all(|f| eval_filter(e, f, &mut ex.notes));
                if pass && seen.insert(q) {
                    ex.items.push(e);
                }
            }
        }
    }
    ex.items.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
    ex
}

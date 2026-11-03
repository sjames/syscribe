//! Element-level difference between two loaded models (GH #433,
//! `REQ-TRS-MODELDIFF-001`).
//!
//! Elements are paired by stable `id:` (else qualified name), so a rename or move
//! of an id-identified element is one `Moved` change rather than a removal plus an
//! addition. Equality is judged on the parsed frontmatter and normalised body (the
//! same projection `suspect`/`baseline` hash), so reformatting YAML or re-wrapping
//! whitespace produces no change.

use crate::baseline::element_type_name;
use crate::element::RawElement;
use crate::resolver::Resolver;
use crate::suspect::element_hash;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Frontmatter fields that are links to other elements.
const LINK_FIELDS: &[&str] = &[
    "satisfies", "verifies", "derivedFrom", "achieves", "refines", "allocatedTo", "allocatedFrom",
    "implementedBy", "parent", "blockedBy", "breakdownAdr", "supertype", "typedBy", "subsets",
    "redefines", "supersedes", "subConfigurations", "appliesWhen", "links",
];
/// Link fields whose values are element references (checked for dangling targets).
/// `implementedBy` (paths) and `appliesWhen` (an expression) are links but not refs.
const REF_FIELDS: &[&str] = &[
    "satisfies", "verifies", "derivedFrom", "achieves", "refines", "allocatedTo", "blockedBy",
    "parent", "breakdownAdr", "supertype", "typedBy", "supersedes", "subConfigurations", "links",
];
const EVIDENCE: &str = "evidence";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Added,
    Removed,
    Moved,
    Modified,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct LinkRef {
    pub field: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub id: String,
    pub qualified_name: String,
    #[serde(rename = "type")]
    pub element_type: String,
    pub kind: ChangeKind,
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moved_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<(Option<String>, Option<String>)>,
    pub links_added: Vec<LinkRef>,
    pub links_removed: Vec<LinkRef>,
    pub evidence_added: Vec<String>,
    pub evidence_removed: Vec<String>,
    /// Other frontmatter fields whose value changed.
    pub fields_changed: Vec<String>,
    pub body_changed: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RefChange {
    pub element: String,
    pub field: String,
    pub target: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSet {
    pub changes: Vec<Change>,
    pub newly_broken: Vec<RefChange>,
    pub newly_resolved: Vec<RefChange>,
}

/// Which elements take part: both models are filtered identically.
#[derive(Debug, Default, Clone)]
pub struct Filter {
    /// Element type names (`Requirement`, …); empty = all.
    pub types: Vec<String>,
    /// Qualified-name prefix (a package); `None` = whole model.
    pub package: Option<String>,
}

impl Filter {
    fn keeps(&self, e: &RawElement) -> bool {
        if !self.types.is_empty() {
            let t = e.frontmatter.element_type.as_ref().map(element_type_name).unwrap_or_default();
            if !self.types.iter().any(|x| x.eq_ignore_ascii_case(&t)) {
                return false;
            }
        }
        match &self.package {
            Some(p) => e.qualified_name == *p || e.qualified_name.starts_with(&format!("{p}::")),
            None => true,
        }
    }
}

fn ident(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn fm_map(e: &RawElement) -> BTreeMap<String, Value> {
    match serde_json::to_value(&e.frontmatter) {
        Ok(Value::Object(m)) => m.into_iter().filter(|(_, v)| !v.is_null()).collect(),
        _ => BTreeMap::new(),
    }
}

fn strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| strings(x, out)),
        Value::Object(o) => o.values().for_each(|x| strings(x, out)),
        _ => {}
    }
}

/// `(field, target)` pairs for the given link fields of `e`; the user-link map is flattened
/// to `links.<type>`.
fn link_refs(m: &BTreeMap<String, Value>, fields: &[&str]) -> BTreeSet<LinkRef> {
    let mut out = BTreeSet::new();
    for f in fields {
        let Some(v) = m.get(*f) else { continue };
        if *f == "links" {
            if let Value::Object(o) = v {
                for (ty, val) in o {
                    let mut t = Vec::new();
                    strings(val, &mut t);
                    out.extend(t.into_iter().map(|target| LinkRef { field: format!("links.{ty}"), target }));
                }
            }
            continue;
        }
        let mut t = Vec::new();
        strings(v, &mut t);
        out.extend(t.into_iter().map(|target| LinkRef { field: (*f).to_string(), target }));
    }
    out
}

/// One printable token per evidence entry: its `ref`/`path`, in authored order of keys.
fn evidence_tokens(m: &BTreeMap<String, Value>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some(Value::Array(a)) = m.get(EVIDENCE) {
        for e in a {
            let tok = match e {
                Value::Object(o) => o
                    .get("ref")
                    .or_else(|| o.get("path"))
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| e.to_string()),
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            out.insert(tok);
        }
    }
    out
}

fn status_of(m: &BTreeMap<String, Value>) -> Option<String> {
    m.get("status").and_then(|v| v.as_str()).map(String::from)
}

fn norm_body(e: &RawElement) -> String {
    e.doc.replace("\r\n", "\n").replace('\r', "\n").trim().to_string()
}

fn base_change(e: &RawElement, kind: ChangeKind) -> Change {
    Change {
        id: ident(e),
        qualified_name: e.qualified_name.clone(),
        element_type: e.frontmatter.element_type.as_ref().map(element_type_name).unwrap_or_default(),
        kind,
        file: e.file_path.clone(),
        moved_from: None,
        status: None,
        links_added: Vec::new(),
        links_removed: Vec::new(),
        evidence_added: Vec::new(),
        evidence_removed: Vec::new(),
        fields_changed: Vec::new(),
        body_changed: false,
        summary: String::new(),
    }
}

fn describe_links(prefix: &str, links: &[LinkRef], parts: &mut Vec<String>) {
    if !links.is_empty() {
        let l: Vec<String> = links.iter().map(|r| format!("{} {}", r.field, r.target)).collect();
        parts.push(format!("{prefix} {}", l.join(", ")));
    }
}

fn finish(mut c: Change) -> Change {
    let mut parts = Vec::new();
    match c.kind {
        ChangeKind::Added => parts.push("added".to_string()),
        ChangeKind::Removed => parts.push("removed".to_string()),
        ChangeKind::Moved => parts.push(format!("moved from {}", c.moved_from.clone().unwrap_or_default())),
        ChangeKind::Modified => {}
    }
    if let Some((a, b)) = &c.status {
        parts.push(format!(
            "status {} → {}",
            a.as_deref().unwrap_or("(none)"),
            b.as_deref().unwrap_or("(none)")
        ));
    }
    describe_links("+", &c.links_added, &mut parts);
    describe_links("-", &c.links_removed, &mut parts);
    if !c.evidence_added.is_empty() {
        parts.push(format!("+evidence {}", c.evidence_added.join(", ")));
    }
    if !c.evidence_removed.is_empty() {
        parts.push(format!("-evidence {}", c.evidence_removed.join(", ")));
    }
    if !c.fields_changed.is_empty() {
        parts.push(format!("fields {}", c.fields_changed.join(", ")));
    }
    if c.body_changed {
        parts.push("body changed".to_string());
    }
    c.summary = parts.join("; ");
    c
}

/// Compare one element present on both sides; `None` when nothing normative differs.
fn compare(old: &RawElement, new: &RawElement) -> Option<Change> {
    if element_hash(old, &[]) == element_hash(new, &[]) && old.qualified_name == new.qualified_name {
        return None;
    }
    let (om, nm) = (fm_map(old), fm_map(new));
    let mut c = base_change(new, ChangeKind::Modified);
    if old.qualified_name != new.qualified_name {
        c.kind = ChangeKind::Moved;
        c.moved_from = Some(old.qualified_name.clone());
    }
    let (os, ns) = (status_of(&om), status_of(&nm));
    if os != ns {
        c.status = Some((os, ns));
    }
    let (ol, nl) = (link_refs(&om, LINK_FIELDS), link_refs(&nm, LINK_FIELDS));
    c.links_added = nl.difference(&ol).cloned().collect();
    c.links_removed = ol.difference(&nl).cloned().collect();
    let (oe, ne) = (evidence_tokens(&om), evidence_tokens(&nm));
    c.evidence_added = ne.difference(&oe).cloned().collect();
    c.evidence_removed = oe.difference(&ne).cloned().collect();

    let keys: BTreeSet<&String> = om.keys().chain(nm.keys()).collect();
    for k in keys {
        if k == "status" || k == EVIDENCE || LINK_FIELDS.contains(&k.as_str()) {
            continue;
        }
        if om.get(k) != nm.get(k) {
            c.fields_changed.push(k.clone());
        }
    }
    // A link/evidence field whose entries are unchanged but reordered/re-nested is not a change;
    // one whose scalar form differs without a set difference (e.g. string → list) isn't either.
    c.body_changed = norm_body(old) != norm_body(new);
    let empty = c.status.is_none()
        && c.links_added.is_empty()
        && c.links_removed.is_empty()
        && c.evidence_added.is_empty()
        && c.evidence_removed.is_empty()
        && c.fields_changed.is_empty()
        && !c.body_changed
        && c.kind == ChangeKind::Modified;
    if empty {
        return None;
    }
    Some(finish(c))
}

/// Dangling reference targets of every element in `elems`, keyed `(element, field, target)`.
fn dangling(elems: &[&RawElement], all: &[RawElement]) -> BTreeSet<RefChange> {
    let resolver = Resolver::new(all);
    let mut out = BTreeSet::new();
    for e in elems {
        let m = fm_map(e);
        for r in link_refs(&m, REF_FIELDS) {
            if resolver.resolve_ref(all, &r.target).is_none() {
                out.insert(RefChange { element: ident(e), field: r.field, target: r.target });
            }
        }
    }
    out
}

/// Compare `old` with `new`.
pub fn diff(old: &[RawElement], new: &[RawElement], filter: &Filter) -> ChangeSet {
    let old_f: Vec<&RawElement> = old.iter().filter(|e| filter.keeps(e)).collect();
    let new_f: Vec<&RawElement> = new.iter().filter(|e| filter.keeps(e)).collect();
    let old_by: HashMap<String, &RawElement> = old_f.iter().map(|e| (ident(e), *e)).collect();
    let new_by: HashMap<String, &RawElement> = new_f.iter().map(|e| (ident(e), *e)).collect();

    let mut changes: Vec<Change> = Vec::new();
    let mut removed: Vec<&RawElement> = Vec::new();
    let mut added: Vec<&RawElement> = Vec::new();
    for e in &new_f {
        match old_by.get(&ident(e)) {
            Some(o) => changes.extend(compare(o, e)),
            None => added.push(e),
        }
    }
    for e in &old_f {
        if !new_by.contains_key(&ident(e)) {
            removed.push(e);
        }
    }
    // An element with no stable id that moved keeps its content: pair a removal and an
    // addition whose content hash is identical and unique on both sides.
    let hash_of = |e: &RawElement| element_hash(e, &["name"]);
    let mut by_hash_removed: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, e) in removed.iter().enumerate() {
        by_hash_removed.entry(hash_of(e)).or_default().push(i);
    }
    let mut by_hash_added: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, e) in added.iter().enumerate() {
        by_hash_added.entry(hash_of(e)).or_default().push(i);
    }
    let mut paired_r = BTreeSet::new();
    let mut paired_a = BTreeSet::new();
    for (h, ri) in &by_hash_removed {
        if let (1, Some(ai)) = (ri.len(), by_hash_added.get(h)) {
            if ai.len() == 1 {
                let (o, n) = (removed[ri[0]], added[ai[0]]);
                if o.frontmatter.id.is_none() && n.frontmatter.id.is_none() {
                    let mut c = base_change(n, ChangeKind::Moved);
                    c.moved_from = Some(o.qualified_name.clone());
                    changes.push(finish(c));
                    paired_r.insert(ri[0]);
                    paired_a.insert(ai[0]);
                }
            }
        }
    }
    for (i, e) in added.iter().enumerate() {
        if !paired_a.contains(&i) {
            let mut c = base_change(e, ChangeKind::Added);
            let m = fm_map(e);
            c.status = status_of(&m).map(|s| (None, Some(s)));
            c.links_added = link_refs(&m, LINK_FIELDS).into_iter().collect();
            c.evidence_added = evidence_tokens(&m).into_iter().collect();
            changes.push(finish(c));
        }
    }
    for (i, e) in removed.iter().enumerate() {
        if !paired_r.contains(&i) {
            changes.push(finish(base_change(e, ChangeKind::Removed)));
        }
    }
    changes.sort_by(|a, b| (&a.qualified_name, &a.id).cmp(&(&b.qualified_name, &b.id)));

    // References: judged against each side's complete element set, reported for the filtered scope.
    // A reference that vanished with its removed owner was not "resolved".
    let od: BTreeSet<RefChange> = dangling(&old_f, old).into_iter().filter(|r| new_by.contains_key(&r.element)).collect();
    let nd = dangling(&new_f, new);
    ChangeSet {
        changes,
        newly_broken: nd.difference(&od).cloned().collect(),
        newly_resolved: od.difference(&nd).cloned().collect(),
    }
}

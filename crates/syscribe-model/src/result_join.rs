//! Joining result records to the TestCase they are about, and noticing a result that is older than a change to
//! its test or to what the test verifies (REQ-TRS-RESJOIN-001).
//!
//! A record names its test by something a test tool can know: a TestCase id, a qualified name or an `extRef`
//! value (unique per model, `W028`). Joining stores the canonical id, so every reader finds the record, and the
//! hash of the TestCase and of each requirement it verifies as they were at that moment, so a later edit is
//! visible.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::result_records::ResultRecord;
use crate::suspect::projection_hash;

/// What a join made of a set of records.
pub struct Joined {
    /// Records attached to a TestCase, with its canonical id and the hashes recorded.
    pub matched: Vec<ResultRecord>,
    /// Records whose key names no element at all.
    pub unmatched: Vec<ResultRecord>,
}

/// The canonical reference of a TestCase: its stable id, else its qualified name.
pub fn canonical(tc: &RawElement) -> String {
    tc.frontmatter.id.clone().unwrap_or_else(|| tc.qualified_name.clone())
}

struct Index<'a> {
    by_id: HashMap<&'a str, Vec<&'a RawElement>>,
    by_qname: HashMap<&'a str, Vec<&'a RawElement>>,
    by_ext: HashMap<&'a str, Vec<&'a RawElement>>,
}

impl<'a> Index<'a> {
    fn new(elems: &'a [RawElement]) -> Self {
        let mut ix = Index { by_id: HashMap::new(), by_qname: HashMap::new(), by_ext: HashMap::new() };
        for e in elems {
            if let Some(id) = e.frontmatter.id.as_deref() {
                ix.by_id.entry(id).or_default().push(e);
            }
            ix.by_qname.entry(e.qualified_name.as_str()).or_default().push(e);
            for x in e.frontmatter.ext_ref.iter().flatten() {
                ix.by_ext.entry(x.as_str()).or_default().push(e);
            }
        }
        ix
    }

    /// Resolve `key` in the documented order: id, qualified name, `extRef`. `Ok(None)` when nothing has that
    /// key; an error when the key is ambiguous or names something that is not a TestCase.
    fn resolve(&self, key: &str) -> Result<Option<&'a RawElement>, String> {
        for (stage, map) in [("id", &self.by_id), ("qualified name", &self.by_qname), ("extRef", &self.by_ext)] {
            let Some(hits) = map.get(key) else { continue };
            if hits.len() > 1 {
                let names: Vec<&str> = hits.iter().map(|e| e.qualified_name.as_str()).collect();
                return Err(format!("'{key}' is ambiguous as an {stage}: it names {}", names.join(", ")));
            }
            let e = hits[0];
            if e.frontmatter.element_type != Some(ElementType::TestCase) {
                let kind = e.frontmatter.element_type.as_ref().map_or("element", |t| t.name());
                return Err(format!("'{key}' names {}, a {kind}, not a TestCase", e.qualified_name));
            }
            return Ok(Some(e));
        }
        Ok(None)
    }
}

/// Join `records` to the TestCases of the model. A key that is ambiguous, or names something that is not a TestCase,
/// is an error naming every such record, so a report that cannot be attributed is never half-stored.
pub fn join(records: Vec<ResultRecord>, elems: &[RawElement]) -> Result<Joined, String> {
    let ix = Index::new(elems);
    let resolver = Resolver::new(elems);
    let mut out = Joined { matched: Vec::new(), unmatched: Vec::new() };
    let mut problems: Vec<String> = Vec::new();
    let mut hashes: HashMap<String, (String, BTreeMap<String, String>)> = HashMap::new();
    for mut r in records {
        match ix.resolve(&r.test_case) {
            Err(e) => problems.push(format!("record '{}': {e}", r.test_case)),
            Ok(None) => out.unmatched.push(r),
            Ok(Some(tc)) => {
                let id = canonical(tc);
                let (tc_hash, reqs) = hashes.entry(id.clone()).or_insert_with(|| {
                    let mut reqs = BTreeMap::new();
                    for t in tc.frontmatter.verifies.iter().flatten() {
                        if let Some(req) = resolver.resolve_ref(elems, t) {
                            reqs.insert(canonical(req), projection_hash(req));
                        }
                    }
                    (projection_hash(tc), reqs)
                });
                if let Some(u) = r.provenance.as_mut().and_then(|p| p.unit.as_mut()) {
                    if let Some(of) = u.of.clone() {
                        match resolver.resolve_ref(elems, &of) {
                            Some(e) if matches!(e.frontmatter.element_type, Some(ElementType::PartDef | ElementType::Part)) => u.of = Some(e.qualified_name.clone()),
                            Some(e) => {
                                problems.push(format!("record '{}': unit.of '{of}' names {}, which is not a part definition", r.test_case, e.qualified_name));
                                continue;
                            }
                            None => {
                                problems.push(format!("record '{}': unit.of '{of}' does not resolve to any model element", r.test_case));
                                continue;
                            }
                        }
                    }
                }
                r.test_case = id;
                r.tc_hash = Some(tc_hash.clone());
                r.req_hashes = reqs.clone();
                out.matched.push(r);
            }
        }
    }
    if !problems.is_empty() {
        return Err(format!("records: {}", problems.join("; ")));
    }
    Ok(out)
}

/// Why the results of `tc` are older than the model: the TestCase, or a requirement it verifies, changed (or
/// vanished) since the records were ingested. Empty when nothing changed or no record carries hashes.
pub fn stale_reasons<'r>(tc: &RawElement, records: impl IntoIterator<Item = &'r ResultRecord>, elems: &[RawElement], resolver: &Resolver) -> Vec<String> {
    let now = projection_hash(tc);
    let mut reasons: BTreeSet<String> = BTreeSet::new();
    for r in records {
        if r.tc_hash.as_deref().is_some_and(|h| h != now) {
            reasons.insert(format!("the TestCase {} changed", canonical(tc)));
        }
        for (req, recorded) in &r.req_hashes {
            match resolver.resolve_ref(elems, req) {
                Some(e) if projection_hash(e) == *recorded => {}
                Some(_) => {
                    reasons.insert(format!("requirement {req} changed"));
                }
                None => {
                    reasons.insert(format!("requirement {req} no longer exists"));
                }
            }
        }
    }
    reasons.into_iter().collect()
}

/// A lenient lookup of a TestCase by a key a report may carry (a tag, for example): an id, a qualified name or an
/// `extRef`. A key that names nothing, something that is not a TestCase, or is ambiguous gives `None`, since a tag
/// such as `REQ-1` or `smoke` is not an error.
pub fn testcase_lookup<'a>(elems: &'a [RawElement]) -> impl Fn(&str) -> Option<&'a RawElement> {
    let ix = Index::new(elems);
    move |key| ix.resolve(key).ok().flatten()
}

// ── environments (REQ-TRS-RESENV-001) ────────────────────────────────────────

/// The id of the TestEnvironment that `r` (an id or a qualified name) names, or `None` if it names none.
pub fn environment_id(elems: &[RawElement], resolver: &Resolver, r: &str) -> Option<String> {
    let e = resolver.resolve_ref(elems, r)?;
    (e.frontmatter.element_type == Some(ElementType::TestEnvironment)).then(|| canonical(e))
}

/// The environments an element runs on (`runsOn:`), as canonical ids. References that name no TestEnvironment
/// are left out (`E894` reports them).
pub fn runs_on_ids(el: &RawElement, elems: &[RawElement], resolver: &Resolver) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for r in el.frontmatter.runs_on.iter().flatten() {
        if let Some(id) = environment_id(elems, resolver, r) {
            if !out.contains(&id) {
                out.push(id);
            }
        }
    }
    out
}

/// The TestEnvironments that cannot vouch for a result: retired, or past their calibration. A result recorded on
/// one does not count as a pass for a plan that judges its environments.
pub fn unhealthy_environments(elems: &[RawElement]) -> std::collections::HashSet<String> {
    elems
        .iter()
        .filter(|e| e.frontmatter.element_type == Some(ElementType::TestEnvironment))
        .filter(|e| e.frontmatter.status.as_deref() == Some("retired") || e.frontmatter.calibration_status.as_deref() == Some("expired"))
        .map(canonical)
        .collect()
}

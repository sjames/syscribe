//! The Traceability (hazard-to-test) generator (GH #223).
//!
//! Subject: a `SafetyGoal`, a `HazardousEvent`, a `Requirement` (its lineage:
//! ancestors, descendants and the goals they trace to) or a package (every goal
//! and hazardous event under it). The picture is the V-model's safety spine,
//! `HazardousEvent ← SafetyGoal ← Requirement ← … ← TestCase`, with each
//! goal's `FaultTree` (`topEvent:`) and supporting `Argument`s (`supports:`)
//! alongside. Edges are drawn the way the links are held (OSLC, §12.1: the
//! derived / verifying artifact points upstream) and the layout reverses them,
//! so the hazards read first, left to right.
//!
//! Colour and overlay (`Node.mark`): a test case shows its verdict (`unknown`
//! in a derived diagram — there is no results sidecar — and the real verdict
//! when [`graph_with_verdicts`] is given one); a requirement shows its
//! lifecycle status and `ASIL`; a goal or hazard shows its `ASIL`. The tone is
//! the roll-up of what is below the node: a failing test or a **gap** is
//! `bad`, an unverified or draft branch `warn`, a fully passing one `ok`.
//! Gaps are red badges that name the finding the validator raises:
//! `W002`/`W003` (a leaf with no active test case), `W305` (a parent with no
//! active L3–L5 integration test), `W300` (an approved leaf nothing
//! satisfies), `no requirement` under a goal, `no goal` on a hazardous event.

use std::collections::{HashMap, HashSet};

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::safety_case::Verdict;

use super::super::ir::{DiagramGraph, DiagramKind, EdgeKind, NodeKind, Tone};
use super::super::manifest::Issue;
use super::analysis::{add_edge, add_node, display_id, keys_of, mark, node_for, roll, unresolved_node};
use super::requirement::{is_under, keeps_keys, unmatched_key_issues};
use super::{w418, Filters};

fn is_package(t: &ElementType) -> bool {
    matches!(t, ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace)
}

/// Lookup tables built once per diagram.
struct Index<'a> {
    elements: &'a [RawElement],
    resolver: &'a Resolver,
    /// requirement qname -> the requirements it is `derivedFrom`.
    parents: HashMap<String, Vec<&'a RawElement>>,
    /// requirement qname -> the requirements `derivedFrom` it.
    children: HashMap<String, Vec<&'a RawElement>>,
    /// goal qname -> requirements with `derivedFromSafetyGoal` naming it.
    reqs_for_goal: HashMap<String, Vec<&'a RawElement>>,
    /// requirement qname -> test cases that `verifies` it.
    tests_for: HashMap<String, Vec<&'a RawElement>>,
    /// requirement qname -> number of elements that `satisfies` it.
    satisfiers: HashMap<String, usize>,
    /// goal qname -> fault trees naming it as `topEvent`.
    trees_for: HashMap<String, Vec<&'a RawElement>>,
    /// goal qname -> arguments whose `supports` names it.
    args_for: HashMap<String, Vec<&'a RawElement>>,
    /// hazardous event qname -> goals listing it.
    goals_of_he: HashMap<String, Vec<&'a RawElement>>,
}

impl<'a> Index<'a> {
    fn new(elements: &'a [RawElement], resolver: &'a Resolver) -> Self {
        let mut ix = Index {
            elements,
            resolver,
            parents: HashMap::new(),
            children: HashMap::new(),
            reqs_for_goal: HashMap::new(),
            tests_for: HashMap::new(),
            satisfiers: HashMap::new(),
            trees_for: HashMap::new(),
            args_for: HashMap::new(),
            goals_of_he: HashMap::new(),
        };
        let q = |r: &str| resolver.resolve_ref(elements, r).map(|t| t.qualified_name.clone());
        for e in elements {
            for s in e.frontmatter.satisfies.as_deref().unwrap_or(&[]) {
                if let Some(t) = q(s) {
                    *ix.satisfiers.entry(t).or_default() += 1;
                }
            }
            if Resolver::is_native_requirement(e) {
                for d in e.frontmatter.derived_from.as_deref().unwrap_or(&[]) {
                    if let Some(p) = resolver.resolve_ref(elements, d).filter(|p| Resolver::is_native_requirement(p)) {
                        ix.parents.entry(e.qualified_name.clone()).or_default().push(p);
                        ix.children.entry(p.qualified_name.clone()).or_default().push(e);
                    }
                }
                if let Some(g) = e.frontmatter.derived_from_safety_goal.as_deref().and_then(q) {
                    ix.reqs_for_goal.entry(g).or_default().push(e);
                }
            } else if Resolver::is_native_testcase(e) {
                for v in e.frontmatter.verifies.as_deref().unwrap_or(&[]) {
                    if let Some(t) = q(v) {
                        ix.tests_for.entry(t).or_default().push(e);
                    }
                }
            } else if matches!(e.frontmatter.element_type, Some(ElementType::FaultTree)) {
                if let Some(g) = e.frontmatter.top_event.as_deref().and_then(q) {
                    ix.trees_for.entry(g).or_default().push(e);
                }
            } else if Resolver::is_argument(e) {
                for s in e.frontmatter.supports.as_deref().unwrap_or(&[]) {
                    if let Some(g) = q(s) {
                        ix.args_for.entry(g).or_default().push(e);
                    }
                }
            } else if Resolver::is_safety_goal(e) {
                for h in e.frontmatter.hazardous_events.as_deref().unwrap_or(&[]) {
                    if let Some(h) = q(h) {
                        ix.goals_of_he.entry(h).or_default().push(e);
                    }
                }
            }
        }
        ix
    }

    fn goals_of_requirement(&self, r: &RawElement) -> Option<&'a RawElement> {
        let g = r.frontmatter.derived_from_safety_goal.as_deref()?;
        self.resolver.resolve_ref(self.elements, g).filter(|g| Resolver::is_safety_goal(g))
    }

    /// Requirements reachable from `r` through `next`, `r` excluded, cycle-safe.
    fn closure(&self, r: &'a RawElement, next: &HashMap<String, Vec<&'a RawElement>>) -> Vec<&'a RawElement> {
        let mut seen: HashSet<&str> = HashSet::new();
        seen.insert(r.qualified_name.as_str());
        let mut out = Vec::new();
        let mut stack = vec![r];
        while let Some(c) = stack.pop() {
            for n in next.get(&c.qualified_name).into_iter().flatten() {
                if seen.insert(n.qualified_name.as_str()) {
                    out.push(*n);
                    stack.push(n);
                }
            }
        }
        out
    }
}

fn is_active(tc: &RawElement) -> bool {
    tc.frontmatter.status.as_deref() == Some("active")
}

fn verdict_tone(v: Verdict) -> Tone {
    match v {
        Verdict::Pass => Tone::Ok,
        Verdict::Fail => Tone::Bad,
        Verdict::Unknown => Tone::Warn,
    }
}

/// The gap badges of a requirement — the findings `W002`/`W003`/`W305`/`W300`.
fn requirement_gaps(ix: &Index, r: &RawElement) -> Vec<String> {
    let status = r.frontmatter.status.as_deref().unwrap_or("");
    let tests = ix.tests_for.get(&r.qualified_name).map(Vec::as_slice).unwrap_or(&[]);
    let active: Vec<&&RawElement> = tests.iter().filter(|t| is_active(t)).collect();
    let is_parent = ix.children.get(&r.qualified_name).is_some_and(|c| !c.is_empty());
    let mut gaps = Vec::new();
    if !is_parent {
        match status {
            "approved" | "implemented" if active.is_empty() => gaps.push("W002 no test".to_string()),
            "verified" if active.is_empty() => gaps.push("W003 no test".to_string()),
            _ => {}
        }
        if matches!(status, "approved" | "implemented") && ix.satisfiers.get(&r.qualified_name).copied().unwrap_or(0) == 0 {
            gaps.push("W300 unsatisfied".to_string());
        }
    } else if matches!(status, "approved" | "implemented" | "verified")
        && !active.iter().any(|t| matches!(t.frontmatter.test_level.as_deref(), Some("L3" | "L4" | "L5")))
    {
        gaps.push("W305 no integration test".to_string());
    }
    gaps
}

struct Tones<'a, 'v> {
    ix: &'a Index<'a>,
    verdict_of: &'v dyn Fn(&RawElement) -> Verdict,
    memo: HashMap<String, Tone>,
}

impl<'a> Tones<'a, '_> {
    /// The roll-up tone of a requirement: its own gaps, its tests' verdicts and
    /// everything derived from it.
    fn requirement(&mut self, r: &'a RawElement, depth: usize) -> Tone {
        if let Some(t) = self.memo.get(&r.qualified_name) {
            return *t;
        }
        if depth > 64 {
            return Tone::Neutral;
        }
        // Guard cycles: provisional value while recursing.
        self.memo.insert(r.qualified_name.clone(), Tone::Neutral);
        let mut parts: Vec<Tone> = Vec::new();
        if !requirement_gaps(self.ix, r).is_empty() {
            parts.push(Tone::Bad);
        }
        let tests = self.ix.tests_for.get(&r.qualified_name).cloned().unwrap_or_default();
        let active: Vec<&RawElement> = tests.into_iter().filter(|t| is_active(t)).collect();
        for t in &active {
            parts.push(verdict_tone((self.verdict_of)(t)));
        }
        if active.is_empty() {
            parts.push(if r.frontmatter.status.as_deref() == Some("draft") { Tone::Neutral } else { Tone::Warn });
        }
        for c in self.ix.children.get(&r.qualified_name).cloned().unwrap_or_default() {
            parts.push(self.requirement(c, depth + 1));
        }
        let t = roll(parts);
        self.memo.insert(r.qualified_name.clone(), t);
        t
    }

    fn goal(&mut self, g: &'a RawElement) -> (Tone, bool) {
        let reqs = self.ix.reqs_for_goal.get(&g.qualified_name).cloned().unwrap_or_default();
        let none = reqs.is_empty();
        let mut parts: Vec<Tone> = reqs.into_iter().map(|r| self.requirement(r, 0)).collect();
        if none {
            parts.push(if g.frontmatter.status.as_deref() == Some("draft") { Tone::Warn } else { Tone::Bad });
        }
        (roll(parts), none)
    }
}

fn asil_text(fm: &crate::element::RawFrontmatter) -> Option<String> {
    fm.asil_level.as_deref().filter(|a| !a.is_empty()).map(|a| format!("ASIL {a}"))
}

fn hazard_asil(h: &RawElement) -> Option<String> {
    let fm = &h.frontmatter;
    asil_text(fm).or_else(|| {
        crate::asil::derive(fm.severity.as_deref()?, fm.exposure.as_deref()?, fm.controllability.as_deref()?).map(|a| if a == "QM" { "QM".to_string() } else { format!("ASIL {a}") })
    })
}

struct Emitter<'a, 'g, 'v> {
    graph: &'g mut DiagramGraph,
    ix: &'a Index<'a>,
    filters: &'a Filters,
    tones: Tones<'a, 'v>,
    candidates: Vec<Vec<String>>,
    /// Restrict requirements to this set (a requirement subject's lineage).
    only: Option<HashSet<String>>,
    reqs_done: HashSet<String>,
}

impl<'a> Emitter<'a, '_, '_> {
    fn allowed(&mut self, e: &RawElement) -> bool {
        self.candidates.push(keys_of(e));
        keeps_keys(self.filters, &keys_of(e))
    }

    fn requirement(&mut self, r: &'a RawElement, upstream: &str, held_by: &str) -> Option<String> {
        if self.only.as_ref().is_some_and(|o| !o.contains(&r.qualified_name)) || !self.allowed(r) {
            return None;
        }
        let id = crate::vis::ir::derived_shape_id(&r.qualified_name);
        add_edge(self.graph, EdgeKind::Derive, &id, upstream, Some(r.qualified_name.clone()), None);
        let _ = held_by;
        if !self.reqs_done.insert(r.qualified_name.clone()) {
            return Some(id);
        }
        let tone = self.tones.requirement(r, 0);
        let gaps = requirement_gaps(self.ix, r);
        let status = r.frontmatter.status.clone().unwrap_or_else(|| "no status".to_string());
        let mut node = node_for(r, NodeKind::Requirement, Some(mark(status, asil_text(&r.frontmatter), tone, gaps)));
        node.stereotype = r.frontmatter.id.clone();
        add_node(self.graph, node);
        for t in self.ix.tests_for.get(&r.qualified_name).cloned().unwrap_or_default() {
            if !self.allowed(t) {
                continue;
            }
            let v = (self.tones.verdict_of)(t);
            let active = is_active(t);
            let tc_tone = if active { verdict_tone(v) } else { Tone::Neutral };
            let level = t.frontmatter.test_level.clone().unwrap_or_default();
            let value = [level, t.frontmatter.status.clone().unwrap_or_default()].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ");
            let badges = if active { Vec::new() } else { vec!["inactive".to_string()] };
            let mut tn = node_for(t, NodeKind::TestCase, Some(mark(v.as_str(), (!value.is_empty()).then_some(value), tc_tone, badges)));
            tn.stereotype = t.frontmatter.id.clone();
            let tid = add_node(self.graph, tn);
            add_edge(self.graph, EdgeKind::Verify, &tid, &id, Some(t.qualified_name.clone()), None);
        }
        for c in self.ix.children.get(&r.qualified_name).cloned().unwrap_or_default() {
            self.requirement(c, &id, &r.qualified_name);
        }
        Some(id)
    }

    fn goal(&mut self, g: &'a RawElement, subject_q: &str) {
        let (tone, no_req) = self.tones.goal(g);
        let mut badges = Vec::new();
        if no_req {
            badges.push("no requirement".to_string());
        }
        let status = g.frontmatter.status.clone().unwrap_or_else(|| "no status".to_string());
        let gid = add_node(self.graph, node_for(g, NodeKind::Goal, Some(mark(status, asil_text(&g.frontmatter), tone, badges))));
        let _ = subject_q;
        // The hazardous events the goal answers.
        for h in g.frontmatter.hazardous_events.as_deref().unwrap_or(&[]) {
            match self.ix.resolver.resolve_ref(self.ix.elements, h) {
                Some(he) => {
                    if !self.allowed(he) {
                        continue;
                    }
                    let hid = self.hazard(he, tone);
                    add_edge(self.graph, EdgeKind::Trace, &gid, &hid, Some(g.qualified_name.clone()), None);
                }
                None => {
                    let hid = add_node(self.graph, unresolved_node(h, NodeKind::Block));
                    add_edge(self.graph, EdgeKind::Trace, &gid, &hid, Some(g.qualified_name.clone()), None);
                }
            }
        }
        for ft in self.ix.trees_for.get(&g.qualified_name).cloned().unwrap_or_default() {
            if !self.allowed(ft) {
                continue;
            }
            let status = ft.frontmatter.status.clone().unwrap_or_else(|| "no status".to_string());
            let t = if status == "draft" { Tone::Warn } else { Tone::Neutral };
            let fid = add_node(self.graph, node_for(ft, NodeKind::Block, Some(mark(status, Some("fault tree".to_string()), t, Vec::new()))));
            add_edge(self.graph, EdgeKind::Trace, &fid, &gid, Some(ft.qualified_name.clone()), None);
        }
        for a in self.ix.args_for.get(&g.qualified_name).cloned().unwrap_or_default() {
            if !self.allowed(a) {
                continue;
            }
            let status = a.frontmatter.status.clone().unwrap_or_else(|| "no status".to_string());
            let t = if status == "draft" { Tone::Warn } else { Tone::Neutral };
            let value = a.frontmatter.argument_type.clone().unwrap_or_else(|| "claim".to_string());
            let aid = add_node(self.graph, node_for(a, NodeKind::Goal, Some(mark(status, Some(value), t, Vec::new()))));
            add_edge(self.graph, EdgeKind::Trace, &aid, &gid, Some(a.qualified_name.clone()), None);
        }
        for r in self.ix.reqs_for_goal.get(&g.qualified_name).cloned().unwrap_or_default() {
            self.requirement(r, &gid, &g.qualified_name);
        }
    }

    /// A hazardous event node; `goal_tone` colours it by what answers it.
    fn hazard(&mut self, he: &'a RawElement, goal_tone: Tone) -> String {
        let goals = self.ix.goals_of_he.get(&he.qualified_name).map(Vec::len).unwrap_or(0);
        let mut badges = Vec::new();
        let mut tone = goal_tone;
        if goals == 0 {
            badges.push("no goal".to_string());
            tone = Tone::Bad;
        }
        let status = he.frontmatter.status.clone().unwrap_or_else(|| "no status".to_string());
        add_node(self.graph, node_for(he, NodeKind::Block, Some(mark(status, hazard_asil(he), tone, badges))))
    }
}

/// Generate with the test verdicts from `verdict_of` (a derived diagram has no
/// results sidecar and uses `unknown`; the CLI passes the ingested ones).
pub fn generate_with(
    graph: &mut DiagramGraph,
    subject: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    filters: &Filters,
    issues: &mut Vec<Issue>,
    verdict_of: &dyn Fn(&RawElement) -> Verdict,
) {
    let Some(st) = subject.frontmatter.element_type.as_ref() else { return };
    let is_goal = Resolver::is_safety_goal(subject);
    let is_hazard = Resolver::is_hazardous_event(subject);
    let is_req = matches!(st, ElementType::Requirement);
    if !(is_goal || is_hazard || is_req || is_package(st)) {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — a Traceability diagram subject must be a SafetyGoal, a HazardousEvent, a Requirement or a Package",
            subject.qualified_name,
            st.name()
        )));
        return;
    }
    let ix = Index::new(elements, resolver);
    let sq = subject.qualified_name.as_str();

    let mut goals: Vec<&RawElement> = Vec::new();
    let mut lone_hazards: Vec<&RawElement> = Vec::new();
    let mut only: Option<HashSet<String>> = None;
    if is_goal {
        goals.push(subject);
    } else if is_hazard {
        goals.extend(ix.goals_of_he.get(sq).cloned().unwrap_or_default());
        if goals.is_empty() {
            lone_hazards.push(subject);
        }
    } else if is_req {
        let ups = ix.closure(subject, &ix.parents);
        let downs = ix.closure(subject, &ix.children);
        let mut lineage: HashSet<String> = HashSet::new();
        lineage.insert(subject.qualified_name.clone());
        for r in ups.iter().chain(downs.iter()) {
            lineage.insert(r.qualified_name.clone());
        }
        for r in std::iter::once(&subject).chain(ups.iter()) {
            if let Some(g) = ix.goals_of_requirement(r) {
                if !goals.iter().any(|x| x.qualified_name == g.qualified_name) {
                    goals.push(g);
                }
            }
        }
        only = Some(lineage);
    } else {
        for e in elements.iter().filter(|e| is_under(&e.qualified_name, sq)) {
            if Resolver::is_safety_goal(e) {
                goals.push(e);
            } else if Resolver::is_hazardous_event(e) && ix.goals_of_he.get(&e.qualified_name).is_none_or(|g| g.is_empty()) {
                lone_hazards.push(e);
            }
        }
    }
    if goals.is_empty() && lone_hazards.is_empty() {
        issues.push(w418(format!(
            "`subject` '{}' reaches no SafetyGoal or HazardousEvent — nothing to draw",
            subject.qualified_name
        )));
        return;
    }
    let mut em = Emitter {
        graph,
        ix: &ix,
        filters,
        tones: Tones { ix: &ix, verdict_of, memo: HashMap::new() },
        candidates: Vec::new(),
        only,
        reqs_done: HashSet::new(),
    };
    for g in &goals {
        em.goal(g, sq);
    }
    for h in lone_hazards {
        em.hazard(h, Tone::Bad);
    }
    // Only the elements around the subject can be filtered by name.
    let candidates = std::mem::take(&mut em.candidates);
    unmatched_key_issues(filters, &candidates, issues);
}

pub fn generate(
    graph: &mut DiagramGraph,
    subject: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    filters: &Filters,
    issues: &mut Vec<Issue>,
) {
    generate_with(graph, subject, elements, resolver, filters, issues, &|_| Verdict::Unknown);
}

/// The traceability diagram of `subject` with real test verdicts, without a
/// `Diagram` element behind it (CLI reports that ingest a results sidecar).
pub fn graph_with_verdicts(
    subject: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    verdict_of: &dyn Fn(&RawElement) -> Verdict,
) -> (DiagramGraph, Vec<Issue>) {
    let name = subject.frontmatter.name.clone().unwrap_or_else(|| display_id(subject));
    let mut graph = DiagramGraph::empty(DiagramKind::Traceability, &subject.qualified_name, &name, Some(&subject.qualified_name));
    graph.derived = true;
    let mut issues = Vec::new();
    generate_with(&mut graph, subject, elements, resolver, &Filters::default(), &mut issues, verdict_of);
    (graph, issues)
}

/// Whether a requirement traces up to a `SafetyGoal` (`derivedFromSafetyGoal:`
/// on it or on an ancestor through `derivedFrom:`), the condition for it to have
/// a hazard-to-test picture at all.
pub fn traces_to_goal(req: &RawElement, elements: &[RawElement], resolver: &Resolver) -> bool {
    let mut seen: HashSet<String> = HashSet::new();
    let mut stack = vec![req];
    while let Some(r) = stack.pop() {
        if !seen.insert(r.qualified_name.clone()) {
            continue;
        }
        if r.frontmatter.derived_from_safety_goal.as_deref().and_then(|g| resolver.resolve_ref(elements, g)).is_some_and(Resolver::is_safety_goal) {
            return true;
        }
        for d in r.frontmatter.derived_from.as_deref().unwrap_or(&[]) {
            if let Some(p) = resolver.resolve_ref(elements, d).filter(|p| Resolver::is_native_requirement(p)) {
                stack.push(p);
            }
        }
    }
    false
}

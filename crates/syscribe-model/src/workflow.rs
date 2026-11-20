//! Picking the next piece of work and judging whether an item may be called done (GH #485,
//! `REQ-TRS-NEXT-001`, `REQ-TRS-DONE-001`): the harness-independent half of the agent loop. A
//! harness — Claude Code, Codex, an editor agent — asks `syscribe next` what to work on and
//! `syscribe done <PI>` whether it may stop, instead of each re-deriving both from the model.
//!
//! This module holds the pure selection logic over the elements; the command layer adds the checks
//! that need the CLI crate (the stereotype gates, `trace-code`, the agent policy).

use std::collections::HashSet;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

/// A PlanningItem an agent may take.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub id: String,
    pub qualified_name: String,
    pub name: String,
    pub status: String,
    pub achieves: Vec<String>,
    /// `in_progress` and unclaimed: work that was started and left, so it is offered first.
    pub resumable: bool,
    /// Things to know before starting: open questions an agent may answer, overlap with active work.
    pub hazards: Vec<String>,
}

/// A PlanningItem that is not offered, and why.
#[derive(Debug, Clone, PartialEq)]
pub struct Skipped {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NextReport {
    /// Offered work, best first: resumable items, then by id.
    pub candidates: Vec<Candidate>,
    pub skipped: Vec<Skipped>,
}

pub fn is_planning_item(e: &RawElement) -> bool {
    e.frontmatter.element_type == Some(ElementType::PlanningItem)
}

pub fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Whether `blocker` no longer holds an item back: a PlanningItem that is `done`, an ADR that is
/// `accepted`, a Requirement that is `implemented`/`verified`, anything else carrying a closing status.
pub fn blocker_met(e: &RawElement) -> bool {
    let status = e.frontmatter.status.as_deref().unwrap_or("");
    match e.frontmatter.element_type {
        Some(ElementType::PlanningItem) => status == "done",
        Some(ElementType::ADR) => status == "accepted",
        Some(ElementType::Requirement) => matches!(status, "implemented" | "verified"),
        _ => matches!(status, "done" | "accepted" | "verified" | "released" | "active"),
    }
}

/// The `blockedBy:` entries of `item` that still hold it back, as `ref (why)`.
pub fn unmet_blockers(elements: &[RawElement], resolver: &Resolver, item: &RawElement) -> Vec<String> {
    let mut out = Vec::new();
    for b in item.frontmatter.blocked_by.iter().flatten() {
        match resolver.resolve_ref(elements, b) {
            None => out.push(format!("{b} (unresolved)")),
            Some(t) if !blocker_met(t) => out.push(format!("{b} ({})", t.frontmatter.status.as_deref().unwrap_or("no status"))),
            Some(_) => {}
        }
    }
    out
}

/// The direct child PlanningItems of `item`.
pub fn children<'a>(elements: &'a [RawElement], item: &RawElement) -> Vec<&'a RawElement> {
    let keys: HashSet<&str> = item.frontmatter.id.iter().map(String::as_str).chain([item.qualified_name.as_str()]).collect();
    elements.iter().filter(|e| is_planning_item(e) && e.frontmatter.parent.as_deref().is_some_and(|p| keys.contains(p))).collect()
}

fn is_claimed(e: &RawElement) -> Option<&str> {
    e.frontmatter.claimed_by.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

fn is_active(e: &RawElement) -> bool {
    is_planning_item(e) && (e.frontmatter.status.as_deref() == Some("in_progress") || is_claimed(e).is_some())
}

/// The open questions of `item`, as `(reference, answerable by an agent)`.
fn open_questions(item: &RawElement, default_human: bool) -> Vec<(String, bool)> {
    crate::questions::parse(item)
        .0
        .into_iter()
        .filter(|q| q.status == "open")
        .map(|q| (q.reference(), q.effective_answer_by(default_human) == "any"))
        .collect()
}

/// Every PlanningItem sorted into offered work and skipped, with the reason for each skip.
/// `default_human` is `[agents] human_only_answers` (a question's own `answerBy:` wins).
pub fn next(elements: &[RawElement], default_human: bool) -> NextReport {
    let resolver = Resolver::new(elements);
    let active: Vec<&RawElement> = elements.iter().filter(|e| is_active(e)).collect();
    let mut report = NextReport::default();
    for item in elements.iter().filter(|e| is_planning_item(e)) {
        let id = label(item);
        let status = item.frontmatter.status.clone().unwrap_or_default();
        let mut skip = |reason: String| report.skipped.push(Skipped { id: id.clone(), reason });
        if status == "done" {
            continue;
        }
        if status != "todo" && status != "in_progress" {
            skip(format!("status {status}"));
            continue;
        }
        if let Some(by) = is_claimed(item) {
            skip(format!("claimed by {by}"));
            continue;
        }
        let kids = children(elements, item);
        if !kids.is_empty() {
            let open = kids.iter().filter(|k| k.frontmatter.status.as_deref() != Some("done")).count();
            skip(format!("composite: work its {open} open step(s)"));
            continue;
        }
        let blockers = unmet_blockers(elements, &resolver, item);
        if !blockers.is_empty() {
            skip(format!("blocked by {}", blockers.join(", ")));
            continue;
        }
        let questions = open_questions(item, default_human);
        let for_person: Vec<&str> = questions.iter().filter(|(_, any)| !any).map(|(r, _)| r.as_str()).collect();
        if !for_person.is_empty() {
            skip(format!("waiting on a person: {}", for_person.join(", ")));
            continue;
        }
        let mut hazards: Vec<String> = questions.iter().map(|(r, _)| format!("open question {r} (an agent may answer it)")).collect();
        let mine: HashSet<&str> = item.frontmatter.achieves.iter().flatten().map(String::as_str).collect();
        for other in active.iter().filter(|o| o.qualified_name != item.qualified_name) {
            let shared: Vec<&str> = other.frontmatter.achieves.iter().flatten().map(String::as_str).filter(|a| mine.contains(a)).collect();
            if !shared.is_empty() {
                hazards.push(format!("overlaps active {} by achieves {} (W311)", label(other), shared.join(", ")));
            }
        }
        report.candidates.push(Candidate {
            id,
            qualified_name: item.qualified_name.clone(),
            name: item.frontmatter.name.clone().unwrap_or_default(),
            resumable: status == "in_progress",
            status,
            achieves: item.frontmatter.achieves.clone().unwrap_or_default(),
            hazards,
        });
    }
    report.candidates.sort_by(|a, b| b.resumable.cmp(&a.resumable).then_with(|| a.id.cmp(&b.id)));
    report.skipped.sort_by(|a, b| a.id.cmp(&b.id));
    report
}

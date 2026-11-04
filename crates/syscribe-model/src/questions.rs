//! Inline `PlanningItem` questions (GH #436, `ADR-SYS-PLANITEM-001` addendum,
//! `REQ-TRS-PIQ-001`): the decisions awaiting an answer, kept on the item they block.
//!
//! A question is `{id, text, recommended?, status, answer?, answeredBy?, answeredAt?}`. It is
//! private to its item (`PI-X#Q1` in messages), never a cross-reference target, and detected
//! structurally — from the field, never from prose.

use crate::element::{ElementType, RawElement};
use crate::validator::{Finding, Severity};
use serde::Serialize;
use serde_yaml::Value;
use std::collections::HashSet;

pub const STATUSES: &[&str] = &["open", "answered", "deferred"];

/// One well-formed question and the item that owns it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    /// The owning item's stable id (else its qualified name).
    pub item: String,
    pub item_qualified_name: String,
    pub id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answered_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answered_at: Option<String>,
}

impl Question {
    /// `PI-X#Q1`.
    pub fn reference(&self) -> String {
        format!("{}#{}", self.item, self.id)
    }
}

fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// A YAML scalar as text; mappings, sequences and null are not text.
fn scalar(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn field(m: &serde_yaml::Mapping, key: &str) -> Option<String> {
    m.get(Value::String(key.into())).and_then(scalar).filter(|s| !s.trim().is_empty())
}

/// `Q<digits>`.
pub fn is_question_id(s: &str) -> bool {
    s.len() >= 2 && s.starts_with('Q') && s[1..].bytes().all(|b| b.is_ascii_digit())
}

/// The questions of one element and the problems found in them: `(code, message)`.
/// Only a `PlanningItem` has questions; anything else yields nothing.
pub fn parse(e: &RawElement) -> (Vec<Question>, Vec<(&'static str, String)>) {
    let mut qs = Vec::new();
    let mut problems: Vec<(&'static str, String)> = Vec::new();
    if e.frontmatter.element_type != Some(ElementType::PlanningItem) {
        return (qs, problems);
    }
    let Some(raw) = &e.frontmatter.questions else { return (qs, problems) };
    let item = label(e);
    let Value::Sequence(entries) = raw else {
        problems.push(("E724", format!("`questions:` on {item} must be a list of mappings")));
        return (qs, problems);
    };
    let mut seen: HashSet<String> = HashSet::new();
    for (i, entry) in entries.iter().enumerate() {
        let n = i + 1;
        let Value::Mapping(m) = entry else {
            problems.push(("E724", format!("question #{n} on {item} is not a mapping")));
            continue;
        };
        let Some(id) = field(m, "id") else {
            problems.push(("E724", format!("question #{n} on {item} has no `id`")));
            continue;
        };
        if !is_question_id(&id) {
            problems.push(("E724", format!("question id `{id}` on {item} must look like `Q1`")));
            continue;
        }
        let Some(text) = field(m, "text") else {
            problems.push(("E724", format!("question {item}#{id} has no `text`")));
            continue;
        };
        if !seen.insert(id.clone()) {
            problems.push(("E725", format!("question id `{id}` is used more than once on {item}")));
            continue;
        }
        let status = match field(m, "status") {
            Some(s) if STATUSES.contains(&s.as_str()) => s,
            Some(s) => {
                problems.push(("E726", format!("question {item}#{id} has status `{s}`; expected {}", STATUSES.join(" | "))));
                continue;
            }
            None => {
                problems.push(("E726", format!("question {item}#{id} has no `status`; expected {}", STATUSES.join(" | "))));
                continue;
            }
        };
        let answer = field(m, "answer");
        if status == "answered" && answer.is_none() {
            problems.push(("E727", format!("question {item}#{id} is `answered` but has no `answer`")));
        }
        qs.push(Question {
            item: item.clone(),
            item_qualified_name: e.qualified_name.clone(),
            id,
            text,
            recommended: field(m, "recommended"),
            status,
            answer,
            answered_by: field(m, "answeredBy"),
            answered_at: field(m, "answeredAt"),
        });
    }
    (qs, problems)
}

/// Every well-formed question in the model, in element then entry order.
pub fn collect(elements: &[RawElement]) -> Vec<Question> {
    elements.iter().flat_map(|e| parse(e).0).collect()
}

/// `E724`–`E727` for malformed entries, `W317` for an item at `status: done` with an `open` question,
/// `W318` for a `questions:` field on an element that is not a `PlanningItem` (it is ignored there).
pub fn findings(elements: &[RawElement]) -> Vec<Finding> {
    let mut out = Vec::new();
    for e in elements {
        if e.frontmatter.questions.is_some() && e.frontmatter.element_type != Some(ElementType::PlanningItem) {
            out.push(Finding {
                code: "W318",
                file: e.file_path.clone(),
                message: format!(
                    "`questions:` on {} is ignored — only a PlanningItem has questions; move them to the PlanningItem that owns the decision, or remove the field",
                    label(e)
                ),
                severity: Severity::Warning,
            });
            continue;
        }
        let (qs, problems) = parse(e);
        for (code, message) in problems {
            out.push(Finding { code, file: e.file_path.clone(), message, severity: Severity::Error });
        }
        if e.frontmatter.status.as_deref() == Some("done") {
            let open: Vec<&Question> = qs.iter().filter(|q| q.status == "open").collect();
            if !open.is_empty() {
                out.push(Finding {
                    code: "W317",
                    file: e.file_path.clone(),
                    message: format!(
                        "{} is `done` but has {} open question(s): {} — answer them (`set {} question.answer <Qn> text=…`) or defer them (`question.defer <Qn>`)",
                        label(e),
                        open.len(),
                        open.iter().map(|q| q.reference()).collect::<Vec<_>>().join(", "),
                        label(e)
                    ),
                    severity: Severity::Warning,
                });
            }
        }
    }
    out
}

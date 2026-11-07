//! Inline `PlanningItem` questions (GH #436, `ADR-SYS-PLANITEM-001` addendum,
//! `REQ-TRS-PIQ-001`): the decisions awaiting an answer, kept on the item they block.
//!
//! A question is `{id, text, recommended?, status, answer?, answeredBy?, answeredAt?, answerBy?}`. It is
//! private to its item (`PI-X#Q1` in messages), never a cross-reference target, and detected
//! structurally — from the field, never from prose.

use crate::element::{ElementType, RawElement};
use crate::validator::{Finding, Severity};
use serde::Serialize;
use serde_yaml::Value;
use std::collections::HashSet;

pub const STATUSES: &[&str] = &["open", "answered", "deferred"];
/// Who may answer a question (GH #457, `REQ-TRS-AGENTPOL-004`): `human` reserves it to a person.
pub const ANSWER_BY: &[&str] = &["human", "any"];

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
    /// `answerBy:` as declared (`human` | `any`); absent means the project default
    /// (`[agents] human_only_answers`). An unrecognised value is reported (`E724`), kept here, and
    /// counts as `human`, so a typo never opens a question to agents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer_by: Option<String>,
}

impl Question {
    /// `PI-X#Q1`.
    pub fn reference(&self) -> String {
        format!("{}#{}", self.item, self.id)
    }

    /// Only a person answers this question: its own `answerBy:` says so (or is unrecognised), or it says
    /// nothing and the project default (`[agents] human_only_answers`) reserves every question.
    pub fn reserved(&self, default_human: bool) -> bool {
        match self.answer_by.as_deref() {
            Some("any") => false,
            Some(_) => true,
            None => default_human,
        }
    }

    /// `"human"` or `"any"`: who may answer it, once the project default is applied.
    pub fn effective_answer_by(&self, default_human: bool) -> &'static str {
        if self.reserved(default_human) { "human" } else { "any" }
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
    if e.frontmatter.element_type != Some(ElementType::PlanningItem) {
        return (Vec::new(), Vec::new());
    }
    match &e.frontmatter.questions {
        Some(raw) => parse_value(&label(e), &e.qualified_name, raw),
        None => (Vec::new(), Vec::new()),
    }
}

/// The questions in a `.md` file's text, when it is a PlanningItem whose frontmatter parses. The owning
/// item is named by its `id` (the qualified name is not known from the text alone and is left empty).
pub fn parse_content(content: &str) -> Vec<Question> {
    let (Some(yaml), _) = crate::frontmatter::split_frontmatter(content) else { return Vec::new() };
    let Ok(fm) = crate::frontmatter::parse_frontmatter(yaml) else { return Vec::new() };
    if fm.element_type != Some(ElementType::PlanningItem) {
        return Vec::new();
    }
    match &fm.questions {
        Some(raw) => parse_value(fm.id.as_deref().unwrap_or(""), "", raw).0,
        None => Vec::new(),
    }
}

/// The questions of one item's `questions:` value; `item` is its stable id (else qualified name).
pub fn parse_value(item: &str, qualified_name: &str, raw: &Value) -> (Vec<Question>, Vec<(&'static str, String)>) {
    let mut qs = Vec::new();
    let mut problems: Vec<(&'static str, String)> = Vec::new();
    let item = item.to_string();
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
        let answer_by = field(m, "answerBy");
        if let Some(a) = answer_by.as_deref().filter(|a| !ANSWER_BY.contains(a)) {
            problems.push((
                "E724",
                format!("question {item}#{id} has answerBy `{a}`; expected {} (until fixed it counts as `human`)", ANSWER_BY.join(" | ")),
            ));
        }
        qs.push(Question {
            item: item.clone(),
            item_qualified_name: qualified_name.to_string(),
            id,
            text,
            recommended: field(m, "recommended"),
            status,
            answer,
            answered_by: field(m, "answeredBy"),
            answered_at: field(m, "answeredAt"),
            answer_by,
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

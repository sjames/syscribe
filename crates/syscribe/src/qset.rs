//! `set <PI> question.add | question.answer | question.defer` (GH #436, `REQ-TRS-PIQ-003`).
//!
//! [`plan`] is pure: it decides what the question list becomes and which entry lines change, and is
//! shared by the CLI (which patches the file line-wise, keeping comments and order) and the MCP
//! `question` tool (which hands the new list to the guarded-write engine).

use serde_json::{json, Value};
use std::path::Path;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::frontmatter::{splice_frontmatter, split_frontmatter};
use syscribe_model::questions::is_question_id;

use crate::set::{append_to_list_field, line_scalar, preview_or_write, yaml_to_json};

/// One edit to an item's questions.
#[derive(Debug, Clone)]
pub enum Op {
    Add { text: String, recommended: Option<String> },
    Answer { id: String, text: String, by: Option<String> },
    Defer { id: String },
}

/// What an [`Op`] does to a given item.
#[derive(Debug)]
pub struct Plan {
    /// The question the edit concerns (`Q3`).
    pub id: String,
    /// The whole list after the edit — the fallback, and what the MCP tool writes.
    pub full_list: Vec<Value>,
    /// For `Add`: the lines of the new entry (without the leading `- `).
    pub new_entry: Option<Vec<String>>,
    /// For `Answer`/`Defer`: the keys of the existing entry to set, in order.
    pub sets: Vec<(String, String)>,
    /// A one-line summary for the user.
    pub summary: String,
}

fn existing_list(e: &RawElement) -> Result<Vec<Value>, String> {
    match e.frontmatter.questions.as_ref().map(yaml_to_json) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(a)) => Ok(a),
        Some(_) => Err(format!(
            "`questions:` on {} is not a list; fix that (E724) before editing questions",
            e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name)
        )),
    }
}

fn entry_id(v: &Value) -> Option<&str> {
    v.get("id").and_then(|x| x.as_str())
}

pub fn plan(e: &RawElement, op: &Op, now: &str) -> Result<Plan, String> {
    let item = e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name).to_string();
    if e.frontmatter.element_type != Some(ElementType::PlanningItem) {
        return Err(format!("{item} is not a PlanningItem — only a PlanningItem has questions"));
    }
    let mut list = existing_list(e)?;
    let known: Vec<String> = list.iter().filter_map(|v| entry_id(v).map(String::from)).collect();
    let locate = |id: &str| -> Result<usize, String> {
        if !is_question_id(id) {
            return Err(format!("`{id}` is not a question id (expected `Q1`, `Q2`, …)"));
        }
        list.iter().position(|v| entry_id(v) == Some(id)).ok_or_else(|| {
            format!(
                "{item} has no question {id}{}",
                if known.is_empty() { " (it has none)".to_string() } else { format!(" (it has {})", known.join(", ")) }
            )
        })
    };
    match op {
        Op::Add { text, recommended } => {
            if text.trim().is_empty() {
                return Err("question.add needs a non-empty text=<question>".into());
            }
            let next = known
                .iter()
                .filter_map(|k| k.strip_prefix('Q').and_then(|n| n.parse::<u64>().ok()))
                .max()
                .unwrap_or(0)
                + 1;
            let id = format!("Q{next}");
            let mut entry = serde_json::Map::new();
            let mut lines = vec![format!("id: {id}"), format!("text: {}", line_scalar(text))];
            entry.insert("id".into(), json!(id));
            entry.insert("text".into(), json!(text));
            if let Some(r) = recommended.as_ref().filter(|r| !r.trim().is_empty()) {
                lines.push(format!("recommended: {}", line_scalar(r)));
                entry.insert("recommended".into(), json!(r));
            }
            lines.push("status: open".into());
            entry.insert("status".into(), json!("open"));
            list.push(Value::Object(entry));
            Ok(Plan { summary: format!("Added question {item}#{id}"), id, full_list: list, new_entry: Some(lines), sets: vec![] })
        }
        Op::Answer { id, text, by } => {
            if text.trim().is_empty() {
                return Err("question.answer needs a non-empty text=<answer>".into());
            }
            let at = locate(id)?;
            let mut sets: Vec<(String, String)> = vec![
                ("status".into(), "answered".into()),
                ("answer".into(), text.clone()),
                ("answeredAt".into(), now.to_string()),
            ];
            if let Some(b) = by.as_ref().filter(|b| !b.trim().is_empty()) {
                sets.push(("answeredBy".into(), b.clone()));
            }
            apply_sets(&mut list[at], &sets);
            Ok(Plan { summary: format!("Answered question {item}#{id}"), id: id.clone(), full_list: list, new_entry: None, sets })
        }
        Op::Defer { id } => {
            let at = locate(id)?;
            let sets = vec![("status".to_string(), "deferred".to_string())];
            apply_sets(&mut list[at], &sets);
            Ok(Plan { summary: format!("Deferred question {item}#{id}"), id: id.clone(), full_list: list, new_entry: None, sets })
        }
    }
}

fn apply_sets(entry: &mut Value, sets: &[(String, String)]) {
    if let Value::Object(m) = entry {
        for (k, v) in sets {
            m.insert(k.clone(), json!(v));
        }
    }
}

/// Indentation of `l` in columns.
fn indent_of(l: &str) -> usize {
    l.len() - l.trim_start_matches([' ', '\t']).len()
}

/// Edit entry `qid` of the block list `questions:` in the frontmatter `yaml`, touching only that
/// entry's lines. `None` when the layout isn't one this can edit with confidence (a flow list, a
/// multi-line value on a key being replaced, an entry it cannot find) — the caller then
/// re-serialises. Every line outside the entry is byte-identical.
pub(crate) fn patch_entry(yaml: &str, qid: &str, sets: &[(String, String)]) -> Option<String> {
    let lines: Vec<&str> = yaml.lines().collect();
    let k = lines.iter().position(|l| l.strip_prefix("questions:").is_some_and(|r| r.trim().is_empty() || r.trim().starts_with('#')))?;
    let mut end = k + 1;
    while end < lines.len() {
        let l = lines[end];
        if l.trim().is_empty() || l.starts_with([' ', '\t', '-', '#']) {
            end += 1;
        } else {
            break;
        }
    }
    while end > k + 1 && (lines[end - 1].trim().is_empty() || lines[end - 1].starts_with('#')) {
        end -= 1;
    }
    // Entry starts: lines whose first non-space token is `-`, at the list's own indentation.
    let first_dash = (k + 1..end).find(|&i| {
        let t = lines[i].trim_start();
        t.starts_with("- ") || t == "-"
    })?;
    let dash_ind = indent_of(lines[first_dash]);
    let starts: Vec<usize> = (first_dash..end)
        .filter(|&i| {
            let t = lines[i].trim_start();
            indent_of(lines[i]) == dash_ind && (t.starts_with("- ") || t == "-")
        })
        .collect();
    let id_matches = |l: &str| -> bool {
        let body = l.trim_start().strip_prefix("- ").unwrap_or(l.trim_start());
        body.strip_prefix("id:").is_some_and(|r| r.split('#').next().unwrap_or("").trim().trim_matches(['"', '\'']) == qid)
    };
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    let (s, e) = starts.iter().enumerate().find_map(|(n, &s)| {
        let e = starts.get(n + 1).copied().unwrap_or(end);
        (s..e).any(|i| id_matches(lines[i])).then_some((s, e))
    })?;
    // The entry's last meaningful line (trailing blanks/comments inside the range stay put).
    let mut last = e - 1;
    while last > s && (lines[last].trim().is_empty() || lines[last].trim_start().starts_with('#')) {
        last -= 1;
    }
    let key_ind = dash_ind + 2;
    let mut inserts: Vec<String> = Vec::new();
    for (key, value) in sets {
        let found = (s..e).find(|&i| {
            let body = if i == s { lines[i].trim_start().strip_prefix("- ").unwrap_or("") } else { lines[i].trim_start() };
            let col_ok = i == s || indent_of(lines[i]) == key_ind;
            col_ok && body.strip_prefix(key.as_str()).is_some_and(|r| r.starts_with(':'))
        });
        match found {
            Some(i) => {
                let t = lines[i].trim_start();
                let (prefix, body) = if i == s {
                    (format!("{}- ", " ".repeat(dash_ind)), t.strip_prefix("- ").unwrap_or(t))
                } else {
                    (" ".repeat(key_ind), t)
                };
                let inline = body[key.len() + 1..].trim();
                // A block scalar or a continuation line makes a one-line replacement unsafe.
                if inline.is_empty() || inline.starts_with(['|', '>']) {
                    return None;
                }
                if lines.get(i + 1).is_some_and(|n| i + 1 < e && indent_of(n) > key_ind && !n.trim().is_empty()) {
                    return None;
                }
                out[i] = format!("{prefix}{key}: {}", line_scalar(value));
            }
            None => inserts.push(format!("{}{key}: {}", " ".repeat(key_ind), line_scalar(value))),
        }
    }
    if !inserts.is_empty() {
        out.splice(last + 1..last + 1, inserts);
    }
    let result = out.join("\n");
    // Trust but verify: the patched entry must read back with exactly the values asked for.
    let v: serde_yaml::Value = serde_yaml::from_str(&result).ok()?;
    let entry = v.get("questions")?.as_sequence()?.iter().find(|q| q.get("id").and_then(|x| x.as_str()) == Some(qid))?;
    for (key, value) in sets {
        if entry.get(key.as_str()).and_then(|x| x.as_str()) != Some(value.as_str()) {
            return None;
        }
    }
    // Same number of entries as before: nothing was merged or lost.
    let before = serde_yaml::from_str::<serde_yaml::Value>(yaml).ok()?;
    if before.get("questions")?.as_sequence()?.len() != v.get("questions")?.as_sequence()?.len() {
        return None;
    }
    Some(result)
}

/// The file content after applying `plan` to `content` (the file at `file_path`): a line-wise edit that
/// keeps everything else byte-identical, else a re-serialisation of the frontmatter with the new list.
pub fn edit_content(content: &str, file_path: &str, plan: &Plan) -> Result<String, String> {
    if let Some(lines) = &plan.new_entry {
        return Ok(append_to_list_field(content, file_path, "questions", &[lines.clone()], json!(plan.full_list)));
    }
    let (yaml, _) = split_frontmatter(content);
    if let Some((y, n)) = yaml.and_then(|y| patch_entry(y, &plan.id, &plan.sets).map(|n| (y, n))) {
        return Ok(splice_frontmatter(content, y, &n));
    }
    // A layout we cannot patch line-wise: re-serialise the frontmatter with the new list.
    let mut fields = serde_json::Map::new();
    fields.insert("questions".into(), json!(plan.full_list));
    syscribe_model::mutate::apply_update_fields(content, Some(&Value::Object(fields)), None)
        .map_err(|m| format!("cannot patch {file_path}: {m}"))
}

/// Run `op` on `elem` and write (or, with `dry_run`, preview) the result. Exits 1 on refusal.
pub fn run(model_root: &Path, elem: &RawElement, op: &Op, dry_run: bool) {
    let plan = match plan(elem, op, &crate::claim::iso8601_now()) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("Refusing: {m}");
            std::process::exit(1);
        }
    };
    let content = std::fs::read_to_string(&elem.file_path).unwrap_or_default();
    let new_content = match edit_content(&content, &elem.file_path, &plan) {
        Ok(c) => c,
        Err(m) => {
            eprintln!("{m}");
            std::process::exit(1);
        }
    };
    if preview_or_write(model_root, elem, &new_content, dry_run) {
        println!("{}", plan.summary);
    }
}

/// Parse `question.*` arguments (`op` is the operation token, `rest` what follows it).
pub fn parse_op(op: &str, rest: &[&str]) -> Result<Op, String> {
    let mut text = None;
    let mut recommended = None;
    let mut by = None;
    let mut id = None;
    for a in rest {
        if let Some(v) = a.strip_prefix("text=") {
            text = Some(v.to_string());
        } else if let Some(v) = a.strip_prefix("recommended=") {
            recommended = Some(v.to_string());
        } else if let Some(v) = a.strip_prefix("by=") {
            by = Some(v.to_string());
        } else if !a.contains('=') && id.is_none() {
            id = Some((*a).to_string());
        } else {
            return Err(format!("{op}: unrecognized argument '{a}'"));
        }
    }
    match op {
        "question.add" => {
            if id.is_some() || by.is_some() {
                return Err("question.add takes text=<question> and optionally recommended=<proposal>".into());
            }
            Ok(Op::Add { text: text.ok_or("question.add needs text=<question>")?, recommended })
        }
        "question.answer" => {
            if recommended.is_some() {
                return Err("question.answer takes <Qn> text=<answer> [by=<actor>]".into());
            }
            Ok(Op::Answer { id: id.ok_or("question.answer needs the question id, e.g. Q1")?, text: text.ok_or("question.answer needs text=<answer>")?, by })
        }
        "question.defer" => {
            if text.is_some() || recommended.is_some() || by.is_some() {
                return Err("question.defer takes only <Qn>".into());
            }
            Ok(Op::Defer { id: id.ok_or("question.defer needs the question id, e.g. Q1")? })
        }
        other => Err(format!("unknown question operation '{other}' (question.add, question.answer, question.defer)")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sets(p: &[(&str, &str)]) -> Vec<(String, String)> {
        p.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    const FM: &str = "id: PI-001\nstatus: todo\n# decisions\nquestions:\n  # the first\n  - id: Q1\n    text: Which UART?   # ask hw\n    status: open\n  - id: Q2\n    text: Baud rate\n    recommended: 115200\n    status: open\nachieves:\n  - REQ-001";

    #[test]
    fn answering_edits_only_its_entry_and_keeps_comments() {
        let out = patch_entry(FM, "Q1", &sets(&[("status", "answered"), ("answer", "UART2"), ("answeredAt", "2026-11-03T10:00:00Z")])).unwrap();
        assert!(out.contains("  # the first\n  - id: Q1\n    text: Which UART?   # ask hw\n    status: answered\n    answer: UART2\n    answeredAt: 2026-11-03T10:00:00Z\n  - id: Q2"), "{out}");
        assert!(out.starts_with("id: PI-001\nstatus: todo\n# decisions\nquestions:") && out.ends_with("achieves:\n  - REQ-001"));
        assert!(out.contains("    recommended: 115200\n    status: open\n"), "Q2 untouched:\n{out}");
    }

    #[test]
    fn the_dash_line_key_and_the_last_entry_are_handled() {
        let fm = "questions:\n- id: Q1\n  text: a\n  status: open\n- id: Q2\n  text: b\n  status: open\nz: 1";
        let out = patch_entry(fm, "Q2", &sets(&[("status", "deferred")])).unwrap();
        assert_eq!(out, "questions:\n- id: Q1\n  text: a\n  status: open\n- id: Q2\n  text: b\n  status: deferred\nz: 1");
        let out = patch_entry(fm, "Q1", &sets(&[("id", "Q1"), ("status", "deferred")])).unwrap();
        assert!(out.contains("- id: Q1\n  text: a\n  status: deferred\n- id: Q2"), "{out}");
    }

    #[test]
    fn a_layout_it_cannot_edit_safely_falls_back() {
        assert!(patch_entry("questions: [{id: Q1, text: a, status: open}]\nz: 1", "Q1", &sets(&[("status", "deferred")])).is_none());
        assert!(patch_entry(FM, "Q9", &sets(&[("status", "deferred")])).is_none());
        let multi = "questions:\n  - id: Q1\n    text: a\n    status: open\n    answer: |\n      long\n      text";
        assert!(patch_entry(multi, "Q1", &sets(&[("answer", "short")])).is_none());
    }

    #[test]
    fn arguments_parse() {
        assert!(matches!(parse_op("question.add", &["text=Which port?", "recommended=UART2"]), Ok(Op::Add { .. })));
        assert!(matches!(parse_op("question.answer", &["Q1", "text=UART2", "by=alice"]), Ok(Op::Answer { .. })));
        assert!(matches!(parse_op("question.defer", &["Q1"]), Ok(Op::Defer { .. })));
        assert!(parse_op("question.add", &[]).is_err());
        assert!(parse_op("question.answer", &["text=x"]).is_err());
        assert!(parse_op("question.defer", &["Q1", "text=x"]).is_err());
        assert!(parse_op("question.bogus", &[]).is_err());
    }
}

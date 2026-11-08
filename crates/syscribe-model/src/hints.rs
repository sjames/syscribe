//! Workflow hints (GH #439): informational findings that name the dedicated command when a
//! file looks hand-edited in a way a command would have handled. Opt-in (`validate --hints`),
//! never errors, never affect the exit status unless selected with `--deny`.

use crate::element::{ElementType, RawElement};
use crate::validator::{Finding, Severity};
use std::collections::HashSet;

/// Per-type `status:` vocabulary, mirroring the validator's own enum checks
/// (E007 Requirement/TestCase, E604 TestPlan, E304 ADR, E708 PlanningItem). `None`: free text.
/// `set status=` and the hints both read this, so there is one list to keep in step.
pub fn status_vocabulary(et: &ElementType) -> Option<&'static [&'static str]> {
    match et {
        ElementType::Requirement => Some(&["draft", "review", "approved", "implemented", "verified"]),
        ElementType::TestCase => Some(&["draft", "review", "approved", "active", "retired"]),
        ElementType::TestPlan => Some(&["draft", "review", "approved", "active", "retired"]),
        ElementType::ADR => Some(&["proposed", "accepted", "deprecated", "superseded"]),
        ElementType::PlanningItem => Some(&["todo", "in_progress", "blocked", "done"]),
        ElementType::ReviewRecord => Some(&["open", "closed", "waived"]),
        ElementType::ResidualRisk => Some(crate::residual_risk::STATUSES),
        _ => None,
    }
}

fn hint(code: &'static str, file: &str, message: String) -> Finding {
    Finding { code, file: file.to_string(), message, severity: Severity::Info }
}

fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// A string that is within a small edit distance of one of `options`.
fn nearest<'a>(word: &str, options: &[&'a str]) -> Option<&'a str> {
    let w = word.to_lowercase();
    options
        .iter()
        .map(|o| (edit_distance(&w, &o.to_lowercase()), *o))
        .filter(|(d, _)| *d <= (w.len() / 3).max(2))
        .min_by_key(|(d, _)| *d)
        .map(|(_, o)| o)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            cur.push((prev[j] + usize::from(ca != cb)).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// The key an `evidence:` entry is identified by.
fn evidence_key(v: &serde_yaml::Value) -> Option<String> {
    match v {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Mapping(m) => ["ref", "path", "commit"].iter().find_map(|k| {
            m.get(serde_yaml::Value::String((*k).into())).and_then(|x| x.as_str()).map(|s| format!("{k}: {s}"))
        }),
        _ => None,
    }
}

/// A Markdown heading whose text is "Questions" or "Open questions" (any level, any case).
fn has_questions_heading(doc: &str) -> bool {
    doc.lines().any(|l| {
        let t = l.trim_start();
        t.starts_with('#') && {
            let h = t.trim_start_matches('#').trim().trim_end_matches(':').to_lowercase();
            h == "questions" || h == "open questions" || h == "open questions and decisions"
        }
    })
}

/// All hints for `elements`.
pub fn hint_findings(elements: &[RawElement]) -> Vec<Finding> {
    let mut out = Vec::new();
    for e in elements {
        let fm = &e.frontmatter;
        let Some(et) = &fm.element_type else { continue };
        let id = label(e);

        // I910 — `claim` always writes claimedBy and claimedAt together.
        if fm.claimed_by.is_some() && fm.claimed_at.is_none() {
            out.push(hint(
                "I910",
                &e.file_path,
                format!(
                    "`claimedBy` is set on {id} without `claimedAt`, which `claim` always writes with it — it looks hand-edited; use `syscribe claim {id} --by <agent>` and `syscribe release {id}` (never hand-edit claim fields)"
                ),
            ));
        }

        // I911 — the same evidence entry twice.
        if let Some(list) = &fm.evidence {
            let mut seen: HashSet<String> = HashSet::new();
            for v in list {
                if let Some(k) = evidence_key(v) {
                    if !seen.insert(k.clone()) {
                        out.push(hint(
                            "I911",
                            &e.file_path,
                            format!("{id} lists the evidence entry `{k}` more than once; keep one (`syscribe set {id} evidence.add …` appends a new entry)"),
                        ));
                    }
                }
            }
        }

        // I913 — a prose "Questions" section while the structured field is empty (GH #436).
        if *et == ElementType::PlanningItem && has_questions_heading(&e.doc) && crate::questions::parse(e).0.is_empty() {
            out.push(hint(
                "I913",
                &e.file_path,
                format!("{id} has a Questions heading in its body but no `questions:` entries; record each with `syscribe set {id} question.add text=…` so `syscribe questions` can list and track it"),
            ));
        }

        // I912 — a status outside the type's vocabulary: name the nearest valid value and the command.
        if let (Some(status), Some(vocab)) = (&fm.status, status_vocabulary(et)) {
            if !vocab.contains(&status.as_str()) {
                let tail = match nearest(status, vocab) {
                    Some(n) => format!("did you mean `{n}`? apply it with `syscribe set {id} status={n}`"),
                    None => format!("valid values: {}; apply one with `syscribe set {id} status=<value>`", vocab.join(", ")),
                };
                out.push(hint("I912", &e.file_path, format!("status `{status}` on {id} is not in this type's vocabulary — {tail}")));
            }
        }
    }
    out
}

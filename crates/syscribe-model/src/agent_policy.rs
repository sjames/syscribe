//! The project's agent policy (GH #442, `ADR-SYS-AGENTPOLICY-001`): rules a tool-driven change must respect,
//! declared once in the opt-in `[agents]` table of `.syscribe.toml`.
//!
//! ```toml
//! [agents]
//! protected_paths = ["model/Safety/**"]                       # tools may not write these
//! human_only_status = { Requirement = ["approved"], ADR = ["accepted"] }
//! human_only_answers = true                                   # PlanningItem questions are answered by a person
//! require_before_done = ["validate", "evidence"]             # before an agent sets a PlanningItem done
//! require_by = true                                           # every model-writing command names its actor
//! actors = ["agent-*", "*[bot]"]                              # git authors that count as agents in the audit
//! audit_limit = 200                                           # how many recent commits `validate` inspects
//! ```
//!
//! The policy is a guard rail for automated editors, not a security boundary: a person who needs to act
//! edits the file, or runs the command with `--as-human`. [`Policy::check_status`] and
//! [`Policy::check_path`] are the checks the writers call; [`audit`] reads git history for changes that
//! reached the files by another route.

use crate::element::{ElementType, RawElement};
use crate::questions::Question;
use crate::validator::{Finding, Severity};
use std::collections::BTreeMap;
use std::path::Path;

const KNOWN_KEYS: &[&str] = &["protected_paths", "human_only_status", "human_only_answers", "require_before_done", "require_by", "actors", "audit_limit"];
const BEFORE_DONE: &[&str] = &["validate", "evidence"];
const DEFAULT_AUDIT_LIMIT: usize = 200;

#[derive(Debug, Clone, Default)]
pub struct Policy {
    /// Repository-relative globs a tool-driven change may not touch.
    pub protected_paths: Vec<String>,
    /// Element type -> statuses only a person sets.
    pub human_only_status: BTreeMap<String, Vec<String>>,
    /// Every PlanningItem question is answered by a person unless it says `answerBy: any` (GH #457).
    pub human_only_answers: bool,
    /// `validate` and/or `evidence`: required before an agent sets a PlanningItem `done`.
    pub require_before_done: Vec<String>,
    /// Every model-writing command must carry `--by` (or `--as-human`).
    pub require_by: bool,
    /// Globs on a git author that mark a commit as agent-made, for the audit.
    pub actors: Vec<String>,
    pub audit_limit: usize,
    /// Malformed entries, ignored; reported as `W762`.
    pub problems: Vec<String>,
}

/// A rule that stopped a change, and how a person can proceed.
#[derive(Debug, Clone)]
pub struct Refusal {
    pub rule: &'static str,
    pub message: String,
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "refused by [agents] {}: {}", self.rule, self.message)
    }
}

const HOW_A_PERSON_PROCEEDS: &str = "a person can edit the file directly, or run the command with --as-human";

fn string_list(v: &toml::Value, key: &str, problems: &mut Vec<String>) -> Vec<String> {
    match v.as_array() {
        Some(a) => {
            let mut out = Vec::new();
            for item in a {
                match item.as_str().map(str::trim).filter(|s| !s.is_empty()) {
                    Some(s) => out.push(s.to_string()),
                    None => problems.push(format!("[agents] {key}: every entry must be a non-empty string")),
                }
            }
            out
        }
        None => {
            problems.push(format!("[agents] {key} must be a list of strings"));
            Vec::new()
        }
    }
}

/// The `[agents]` policy of `<model_root>/.syscribe.toml`, or `None` when the table is absent (the feature is
/// opt-in). Malformed entries are ignored one by one and listed in [`Policy::problems`].
pub fn load(model_root: &Path) -> Option<Policy> {
    let text = std::fs::read_to_string(model_root.join(".syscribe.toml")).ok()?;
    let doc: toml::Value = toml::from_str(&text).ok()?;
    let t = doc.get("agents")?.as_table()?;
    let mut p = Policy { audit_limit: DEFAULT_AUDIT_LIMIT, ..Default::default() };
    for (k, v) in t {
        match k.as_str() {
            "protected_paths" => p.protected_paths = string_list(v, k, &mut p.problems),
            "actors" => p.actors = string_list(v, k, &mut p.problems),
            "require_before_done" => {
                for r in string_list(v, k, &mut p.problems) {
                    if BEFORE_DONE.contains(&r.as_str()) {
                        p.require_before_done.push(r);
                    } else {
                        p.problems.push(format!("[agents] require_before_done: unknown requirement `{r}` (expected {})", BEFORE_DONE.join(", ")));
                    }
                }
            }
            "require_by" => match v.as_bool() {
                Some(b) => p.require_by = b,
                None => p.problems.push("[agents] require_by must be true or false".to_string()),
            },
            "human_only_answers" => match v.as_bool() {
                Some(b) => p.human_only_answers = b,
                None => p.problems.push("[agents] human_only_answers must be true or false".to_string()),
            },
            "audit_limit" => match v.as_integer().filter(|n| *n > 0) {
                Some(n) => p.audit_limit = n as usize,
                None => p.problems.push("[agents] audit_limit must be a positive integer".to_string()),
            },
            "human_only_status" => match v.as_table() {
                Some(tbl) => {
                    for (ty, statuses) in tbl {
                        // `ElementType` has a catch-all for names it does not know; a typo'd type would
                        // otherwise be accepted and protect nothing.
                        let et = serde_json::from_value::<ElementType>(serde_json::Value::String(ty.clone())).unwrap_or(ElementType::Unknown);
                        if et == ElementType::Unknown {
                            p.problems.push(format!("[agents] human_only_status: `{ty}` is not an element type"));
                            continue;
                        }
                        let mut ok = Vec::new();
                        for s in string_list(statuses, &format!("human_only_status.{ty}"), &mut p.problems) {
                            // A status the type does not have could never match: almost certainly a typo.
                            match crate::hints::status_vocabulary(&et) {
                                Some(vocab) if !vocab.contains(&s.as_str()) => p.problems.push(format!(
                                    "[agents] human_only_status.{ty}: `{s}` is not a {ty} status (expected {})",
                                    vocab.join(", ")
                                )),
                                _ => ok.push(s),
                            }
                        }
                        if !ok.is_empty() {
                            p.human_only_status.insert(ty.clone(), ok);
                        }
                    }
                }
                None => p.problems.push("[agents] human_only_status must be a table of element type -> statuses".to_string()),
            },
            other => p.problems.push(format!("[agents] unknown key `{other}` (known: {})", KNOWN_KEYS.join(", "))),
        }
    }
    Some(p)
}

impl Policy {
    /// The `protected_paths` glob that matches any spelling in `candidates`, if one does.
    pub fn protecting_pattern(&self, candidates: &[String]) -> Option<&str> {
        self.protected_paths
            .iter()
            .find(|pat| candidates.iter().any(|c| crate::annotations::glob_match(pat, c.trim_start_matches("./"))))
            .map(String::as_str)
    }

    /// A change by an agent may not touch `path`. `candidates` are the same path spelled relative to the
    /// repository and to the model root; any of them matching a protected glob refuses it.
    pub fn check_path(&self, candidates: &[String]) -> Result<(), Refusal> {
        match self.protecting_pattern(candidates) {
            Some(pat) => Err(Refusal {
                rule: "protected_paths",
                message: format!("`{}` is protected (`{pat}`); {HOW_A_PERSON_PROCEEDS}.", candidates.first().map(String::as_str).unwrap_or("")),
            }),
            None => Ok(()),
        }
    }

    /// An agent may not set `element_type`'s status to `status`.
    pub fn check_status(&self, element_type: &str, status: &str) -> Result<(), Refusal> {
        match self.human_only_status.get(element_type) {
            Some(list) if list.iter().any(|s| s == status) => Err(Refusal {
                rule: "human_only_status",
                message: format!("a {element_type} is set to `{status}` by a person; {HOW_A_PERSON_PROCEEDS}."),
            }),
            _ => Ok(()),
        }
    }

    /// The refusal for `require_by`: a model-writing command named no actor.
    pub fn actor_required() -> Refusal {
        Refusal {
            rule: "require_by",
            message: "this project requires every model-writing command to name its actor: pass --by <actor>, or --as-human if you are a person.".to_string(),
        }
    }

    /// An agent may not create a PlanningItem that is already `done` without the evidence `require_before_done`
    /// asks for — otherwise `create` would be a way round the rule that `set status=done` is held to.
    pub fn check_created_done(&self, element_type: &str, status: &str, has_evidence: bool) -> Result<(), Refusal> {
        if self.require_before_done.is_empty() || element_type != "PlanningItem" || status != "done" || has_evidence {
            return Ok(());
        }
        Err(Refusal {
            rule: "require_before_done",
            message: "an item is created `done` only with resolving `evidence`, as it is marked done only with it (an item with none raises E719).".to_string(),
        })
    }

    /// `require_before_done`: setting a PlanningItem `done` needs the listed conditions to hold. Judged on a
    /// candidate copy: `evidence` needs the item to raise no `E719`; `validate` needs the change to introduce
    /// no error anywhere.
    pub fn check_before_done(&self, elements: &[RawElement], config: &crate::config::ValidateConfig, qname: &str, new_status: &str) -> Result<(), Refusal> {
        if new_status != "done" || self.require_before_done.is_empty() {
            return Ok(());
        }
        let Some(target) = elements.iter().find(|e| e.qualified_name == qname) else { return Ok(()) };
        if target.frontmatter.element_type != Some(ElementType::PlanningItem) {
            return Ok(());
        }
        let mut candidate: Vec<RawElement> = elements.to_vec();
        if let Some(c) = candidate.iter_mut().find(|e| e.qualified_name == qname) {
            c.frontmatter.status = Some(new_status.to_string());
        }
        let after = crate::validator::validate_with_config(&candidate, config).findings;
        if self.require_before_done.iter().any(|r| r == "evidence") {
            if let Some(f) = after.iter().find(|f| f.code == "E719" && f.file == target.file_path) {
                return Err(Refusal {
                    rule: "require_before_done",
                    message: format!("`evidence` is required before an item is done, and {} has none (E719: {}).", label(target), f.message),
                });
            }
        }
        if self.require_before_done.iter().any(|r| r == "validate") {
            let before = crate::validator::validate_with_config(elements, config).findings;
            let known: std::collections::HashSet<(&str, &str, &str)> = before.iter().map(|f| (f.code, f.file.as_str(), f.message.as_str())).collect();
            let new: Vec<&Finding> = after.iter().filter(|f| f.severity == Severity::Error && !known.contains(&(f.code, f.file.as_str(), f.message.as_str()))).collect();
            if !new.is_empty() {
                return Err(Refusal {
                    rule: "require_before_done",
                    message: format!(
                        "`validate` must pass before an item is done, and this change introduces {} error(s): {}.",
                        new.len(),
                        new.iter().take(3).map(|f| format!("{} {}", f.code, f.file)).collect::<Vec<_>>().join("; ")
                    ),
                });
            }
        }
        Ok(())
    }

    /// The rules in plain words, for `agent-guide`, the MCP instructions and `syscribe policy`.
    pub fn describe(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.require_by {
            out.push("Every model-writing command must name its actor with `--by <actor>` (a person says `--as-human`).".to_string());
        }
        if !self.protected_paths.is_empty() {
            out.push(format!("Tools must not write protected paths: {}.", self.protected_paths.iter().map(|p| format!("`{p}`")).collect::<Vec<_>>().join(", ")));
        }
        for (ty, statuses) in &self.human_only_status {
            out.push(format!("Only a person sets a {ty} to {}.", statuses.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join(" or ")));
        }
        if self.human_only_answers {
            out.push("Only a person answers a PlanningItem question, unless the question says `answerBy: any`: record what you need decided with `question.add`, never `question.answer` it yourself (`questions --json` shows `answerBy` per question).".to_string());
        }
        if !self.require_before_done.is_empty() {
            out.push(format!("Before setting a PlanningItem `done`: {}.", self.require_before_done.iter().map(|r| match r.as_str() {
                "validate" => "the change must introduce no validation error",
                _ => "it must have resolving evidence",
            }).collect::<Vec<_>>().join(" and ")));
        }
        // Unless every change must name its actor, an agent that forgets `--by` is simply not bound.
        if !out.is_empty() && !self.require_by {
            out.push("These rules bind a change that names its actor: always pass `--by <you>` (the project can set `require_by` to make that mandatory).".to_string());
        }
        out
    }
}

fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

// ── Questions reserved to a person (GH #457, `REQ-TRS-AGENTPOL-004`) ─────────────────────────────────────

/// Does the project reserve every question to a person by default (`[agents] human_only_answers = true`)?
/// A question's own `answerBy:` always wins over this default.
pub fn default_human(policy: Option<&Policy>) -> bool {
    policy.is_some_and(|p| p.human_only_answers)
}

/// The fields that record an answer, as one comparable value.
fn answer_state(q: &Question) -> (bool, Option<&str>, Option<&str>, Option<&str>) {
    (q.status == "answered", q.answer.as_deref(), q.answered_by.as_deref(), q.answered_at.as_deref())
}

/// An agent's change from `before` to `after` may not answer a reserved question: set its `answer`,
/// `answeredBy` or `answeredAt`, or move it to `answered`. Nor may it lift a reservation (turn a reserved
/// question into one anybody may answer) — otherwise one write could unreserve and answer. Questions are
/// matched by their item's stable id and their own id, so a move of the item is not mistaken for a new
/// answer. Reopening or deleting a question, or editing its text, is not answering it.
pub fn check_answer_change(before: &[Question], after: &[Question], default_human: bool) -> Result<(), Refusal> {
    for a in after {
        let b = before.iter().find(|b| b.item == a.item && b.id == a.id);
        let was_reserved = b.is_some_and(|b| b.reserved(default_human));
        if !(was_reserved || a.reserved(default_human)) {
            continue;
        }
        let why = match a.answer_by.as_deref().or(b.and_then(|b| b.answer_by.as_deref())) {
            Some(v) => format!("`answerBy: {v}`"),
            None => "[agents] human_only_answers".to_string(),
        };
        if was_reserved && !a.reserved(default_human) {
            return Err(Refusal {
                rule: "human_only_answers",
                message: format!("{} is answered by a person ({why}), and an agent may not lift that reservation; {HOW_A_PERSON_PROCEEDS}.", a.reference()),
            });
        }
        let now = answer_state(a);
        let then = b.map(answer_state).unwrap_or((false, None, None, None));
        let answers = (now.0 && !then.0) || (now != then && (now.1.is_some() || now.2.is_some() || now.3.is_some()));
        if answers {
            return Err(Refusal {
                rule: "human_only_answers",
                message: format!(
                    "{} is answered by a person ({why}); an agent may record what it needs decided but not the answer — {HOW_A_PERSON_PROCEEDS}.",
                    a.reference()
                ),
            });
        }
    }
    Ok(())
}

/// [`check_answer_change`] on the text of one file before (`None`: a new file) and after a write. Anything
/// that is not a PlanningItem passes.
pub fn check_answer_content(policy: Option<&Policy>, old: Option<&str>, new: &str) -> Result<(), Refusal> {
    let after = crate::questions::parse_content(new);
    if after.is_empty() {
        return Ok(());
    }
    let before = old.map(crate::questions::parse_content).unwrap_or_default();
    check_answer_change(&before, &after, default_human(policy))
}

/// Who wrote a question's current answer, from the git history of the item's file (#443: git is the change
/// log). `agent` follows the audit's rule: a `Syscribe-Actor` trailer without `Syscribe-As: human`, or an
/// author matching `[agents] actors`.
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnswerProvenance {
    /// The `Syscribe-Actor`, else the commit author.
    pub by: String,
    pub agent: bool,
    /// The commit's author date (ISO 8601).
    pub at: String,
    /// The short commit id.
    pub commit: String,
}

const RS: char = '\u{1e}';
const US: char = '\u{1f}';

/// For every answered question selected by `want`, the commit that last changed its answer — keyed by
/// (item, question id). A question is left out when that cannot be told: the model is not in git, the
/// answer in the working tree differs from the last commit (the latest write is not recorded yet), or no
/// commit within `audit_limit` changed it.
pub fn answer_provenance(
    model_root: &Path,
    elements: &[RawElement],
    policy: Option<&Policy>,
    want: impl Fn(&Question) -> bool,
) -> std::collections::HashMap<(String, String), AnswerProvenance> {
    let mut out = std::collections::HashMap::new();
    let per_item: Vec<(&RawElement, Vec<Question>)> = elements
        .iter()
        .filter(|e| e.frontmatter.element_type == Some(ElementType::PlanningItem))
        .map(|e| (e, crate::questions::parse(e).0.into_iter().filter(|q| q.answer.is_some() && want(q)).collect::<Vec<_>>()))
        .filter(|(_, qs)| !qs.is_empty())
        .collect();
    if per_item.is_empty() {
        return out;
    }
    let Ok(abs) = std::fs::canonicalize(model_root) else { return out };
    let Some(root) = crate::config::detect_git_root(&abs) else { return out };
    let limit = policy.map(|p| p.audit_limit).unwrap_or(DEFAULT_AUDIT_LIMIT).to_string();
    let actors: &[String] = policy.map(|p| p.actors.as_slice()).unwrap_or(&[]);
    for (e, qs) in per_item {
        let file = std::fs::canonicalize(&e.file_path).unwrap_or_else(|_| Path::new(&e.file_path).to_path_buf());
        let Ok(rel) = file.strip_prefix(&root).map(|r| r.to_string_lossy().replace('\\', "/")) else { continue };
        let at = |rev: &str| git(&root, &["show", &format!("{rev}:{rel}")]).map(|c| crate::questions::parse_content(&c)).unwrap_or_default();
        let head = at("HEAD");
        let find = |list: &[Question], id: &str| list.iter().find(|q| q.id == id).map(|q| answer_state(q)).map(|(a, b, c, d)| (a, b.map(String::from), c.map(String::from), d.map(String::from)));
        // Only answers already committed can be attributed.
        let mut pending: Vec<&Question> = qs.iter().filter(|q| find(&head, &q.id) == find(std::slice::from_ref(*q), &q.id)).collect();
        if pending.is_empty() {
            continue;
        }
        let Some(log) = git(&root, &["log", &format!("--format={RS}%H{US}%h{US}%an{US}%aI{US}%B{US}"), "-n", &limit, "--", &rel]) else { continue };
        for rec in log.split(RS).filter(|r| !r.trim().is_empty()) {
            if pending.is_empty() {
                break;
            }
            let f: Vec<&str> = rec.split(US).collect();
            if f.len() < 5 {
                continue;
            }
            let (new, old) = (at(f[0]), at(&format!("{}^", f[0])));
            pending.retain(|q| {
                if find(&new, &q.id) == find(&old, &q.id) {
                    return true;
                }
                let human = trailer(f[4], "Syscribe-As").as_deref() == Some("human");
                let actor = trailer(f[4], "Syscribe-Actor");
                let agent = !human && (actor.is_some() || actors.iter().any(|g| crate::annotations::glob_match(g, f[2])));
                out.insert(
                    (q.item.clone(), q.id.clone()),
                    AnswerProvenance { by: actor.unwrap_or_else(|| f[2].to_string()), agent, at: f[3].to_string(), commit: f[1].to_string() },
                );
                false
            });
        }
    }
    out
}

/// `W323`: a question reserved to a person carries an answer an agent wrote — its `answeredBy` matches
/// `[agents] actors`, or the git history shows the commit that last changed the answer was an agent's. Only
/// reserved questions are looked at (so nothing runs without a policy or an `answerBy: human`); an answer
/// whose writer cannot be told raises nothing.
pub fn answer_audit(model_root: &Path, elements: &[RawElement], policy: Option<&Policy>) -> Vec<Finding> {
    let d = default_human(policy);
    let actors: &[String] = policy.map(|p| p.actors.as_slice()).unwrap_or(&[]);
    let mut out = Vec::new();
    let reserved = |q: &Question| q.reserved(d);
    let mut by_field: std::collections::HashSet<(String, String)> = Default::default();
    for e in elements {
        for q in crate::questions::parse(e).0.into_iter().filter(|q| q.answer.is_some() && reserved(q)) {
            if let Some(who) = q.answered_by.as_deref().filter(|w| actors.iter().any(|g| crate::annotations::glob_match(g, w))) {
                out.push(warning(
                    "W323",
                    &e.file_path,
                    format!(
                        "{} is answered by a person, but `answeredBy: {who}` is an agent ([agents] actors) — have a person confirm or replace the answer (`set {} question.answer {} text=… --as-human`)",
                        q.reference(),
                        q.item,
                        q.id
                    ),
                ));
                by_field.insert((q.item.clone(), q.id.clone()));
            }
        }
    }
    let prov = answer_provenance(model_root, elements, policy, |q| reserved(q));
    for e in elements {
        for q in crate::questions::parse(e).0 {
            let key = (q.item.clone(), q.id.clone());
            if by_field.contains(&key) {
                continue;
            }
            if let Some(p) = prov.get(&key).filter(|p| p.agent) {
                out.push(warning(
                    "W323",
                    &e.file_path,
                    format!(
                        "{} is answered by a person, but its answer was written by an agent ({} in commit {}) — have a person confirm or replace the answer (`set {} question.answer {} text=… --as-human`)",
                        q.reference(),
                        p.by,
                        p.commit,
                        q.item,
                        q.id
                    ),
                ));
            }
        }
    }
    out
}

fn trailer(body: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}: ");
    body.lines().rev().find_map(|l| l.strip_prefix(&prefix)).map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let o = std::process::Command::new("git").arg("-C").arg(root).args(args).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).into_owned())
}

/// For each (commit, file) in the recent history, the values of the `status:` lines its diff adds.
fn added_statuses(root: &Path, scope: &str, limit: &str) -> std::collections::HashMap<(String, String), Vec<String>> {
    let mut map: std::collections::HashMap<(String, String), Vec<String>> = Default::default();
    let Some(patch) = git(root, &["log", "-p", "-U0", "--no-color", "--no-renames", "--format=\u{1e}%H", "-n", limit, "--", scope]) else { return map };
    for rec in patch.split('\u{1e}').filter(|r| !r.trim().is_empty()) {
        let mut lines = rec.lines();
        let Some(hash) = lines.next().map(str::trim) else { continue };
        let mut file: Option<&str> = None;
        for l in lines {
            if let Some(p) = l.strip_prefix("+++ b/") {
                file = Some(p.trim());
            } else if let (Some(v), Some(f)) = (l.strip_prefix("+status:"), file) {
                let v = v.split('#').next().unwrap_or("").trim().trim_matches(['"', '\'']).to_string();
                map.entry((hash.to_string(), f.to_string())).or_default().push(v);
            }
        }
    }
    map
}

fn warning(code: &'static str, file: &str, message: String) -> Finding {
    Finding { code, file: file.to_string(), message, severity: Severity::Warning }
}

/// `W762` for each malformed `[agents]` entry, and — from the recent git history of the model — `W760` where an
/// agent-made commit changed a protected path and `W761` where one set a human-only status. A commit is
/// agent-made when it carries a `Syscribe-Actor` trailer (and no `Syscribe-As: human`), or its author matches
/// `actors`. These are the violations that reached the files by a route the writers' own checks did not see.
pub fn audit(model_root: &Path, policy: &Policy) -> Vec<Finding> {
    let mut out: Vec<Finding> = policy.problems.iter().map(|m| warning("W762", ".syscribe.toml", m.clone())).collect();
    if policy.protected_paths.is_empty() && policy.human_only_status.is_empty() {
        return out;
    }
    let Ok(abs) = std::fs::canonicalize(model_root) else { return out };
    let Some(root) = crate::config::detect_git_root(&abs) else { return out };
    let rel = abs.strip_prefix(&root).unwrap_or(Path::new("")).to_string_lossy().replace('\\', "/");
    let limit = policy.audit_limit.to_string();
    let model_prefix = if rel.is_empty() { String::new() } else { format!("{rel}/") };
    let scope = if rel.is_empty() { ".".to_string() } else { rel };
    let Some(log) = git(
        &root,
        &["log", "--format=\u{1e}%H\u{1f}%an\u{1f}%B\u{1f}", "--name-status", "--no-renames", "-n", &limit, "--", &scope],
    ) else {
        return out;
    };
    // A file's content is only worth reading when the commit's own diff adds a `status:` line carrying a
    // status somebody has reserved for a person: one `git log -p` finds those, instead of reading every file.
    let reserved: std::collections::BTreeSet<&str> = policy.human_only_status.values().flatten().map(String::as_str).collect();
    let added = if reserved.is_empty() { Default::default() } else { added_statuses(&root, &scope, &limit) };
    for rec in log.split('\u{1e}').filter(|r| !r.trim().is_empty()) {
        let f: Vec<&str> = rec.split('\u{1f}').collect();
        if f.len() < 4 {
            continue;
        }
        let (full, author, body, files) = (f[0], f[1], f[2], f[3]);
        let hash = &full[..full.len().min(7)];
        let human = trailer(body, "Syscribe-As").as_deref() == Some("human");
        let actor = trailer(body, "Syscribe-Actor");
        let by_author = policy.actors.iter().any(|g| crate::annotations::glob_match(g, author));
        if human || !(actor.is_some() || by_author) {
            continue;
        }
        let who = actor.unwrap_or_else(|| author.to_string());
        for line in files.lines().filter(|l| !l.trim().is_empty()) {
            let mut parts = line.split('\t');
            let (Some(status), Some(path)) = (parts.next(), parts.next()) else { continue };
            // The policy may spell a protected path relative to the repository or to the model root.
            let spellings = [path.to_string(), path.strip_prefix(&model_prefix).unwrap_or(path).to_string()];
            if let Some(pat) = policy.protecting_pattern(&spellings) {
                out.push(warning(
                    "W760",
                    path,
                    format!("{path} was changed in commit {hash} by {who}, although [agents] protected_paths protects it (`{pat}`)."),
                ));
            }
            let sets_reserved = added.get(&(full.to_string(), path.to_string())).is_some_and(|v| v.iter().any(|s| reserved.contains(s.as_str())));
            if status != "D" && path.ends_with(".md") && sets_reserved {
                let new = git(&root, &["show", &format!("{full}:{path}")]);
                let old = if status == "A" { None } else { git(&root, &["show", &format!("{full}^:{path}")]) };
                if let Some(c) = crate::modeldiff::change_between(path, old.as_deref(), new.as_deref()) {
                    if let Some((_, Some(to))) = &c.status {
                        if policy.check_status(&c.element_type, to).is_err() {
                            out.push(warning("W761", path, format!("{path}: a {} was set to `{to}` in commit {hash} by {who}; that status is for a person to set ([agents] human_only_status).", c.element_type)));
                        }
                    }
                }
            }
        }
    }
    out.dedup_by(|a, b| a.code == b.code && a.file == b.file && a.message == b.message);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(toml: &str) -> Policy {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("syscribe-agent-policy-{}-{}-{n}", std::process::id(), toml.len()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".syscribe.toml"), toml).unwrap();
        let p = load(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        p.expect("an [agents] table")
    }

    #[test]
    fn no_table_means_no_policy() {
        let dir = std::env::temp_dir().join(format!("syscribe-agent-policy-none-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".syscribe.toml"), "[ids]\nmax_digits = 4\n").unwrap();
        assert!(load(&dir).is_none());
        assert!(load(&dir.join("missing")).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_full_policy_loads() {
        let p = policy("[agents]\nprotected_paths = [\"model/Safety/**\"]\nhuman_only_status = { Requirement = [\"approved\"], ADR = [\"accepted\"] }\nrequire_before_done = [\"validate\", \"evidence\"]\nrequire_by = true\nactors = [\"agent-*\"]\naudit_limit = 50\n");
        assert!(p.problems.is_empty(), "{:?}", p.problems);
        assert_eq!((p.require_by, p.audit_limit), (true, 50));
        assert_eq!(p.human_only_status["Requirement"], ["approved"]);
        assert_eq!(p.require_before_done, ["validate", "evidence"]);
    }

    #[test]
    fn malformed_entries_are_ignored_one_by_one_and_reported() {
        let p = policy("[agents]\nprotected_paths = \"not a list\"\nhuman_only_status = { Requirement = [\"aproved\"], NotAType = [\"x\"], ADR = [\"accepted\"] }\nrequire_before_done = [\"tests\"]\nrequire_by = \"yes\"\naudit_limit = 0\nbogus = 1\n");
        assert_eq!(p.problems.len(), 7, "{:#?}", p.problems);
        assert!(p.problems.iter().any(|m| m.contains("`aproved` is not a Requirement status")), "a typo'd status is caught");
        assert!(p.problems.iter().any(|m| m.contains("`NotAType` is not an element type")));
        assert!(p.problems.iter().any(|m| m.contains("unknown key `bogus`")));
        // The good entry survives; the bad ones do nothing.
        assert_eq!(p.human_only_status["ADR"], ["accepted"]);
        assert!(p.protected_paths.is_empty() && p.require_before_done.is_empty() && !p.require_by);
        assert_eq!(p.audit_limit, DEFAULT_AUDIT_LIMIT);
    }

    #[test]
    fn protected_paths_match_in_either_spelling() {
        let p = policy("[agents]\nprotected_paths = [\"model/Safety/**\", \"**/secret.md\"]\n");
        assert!(p.check_path(&["model/Safety/Hazards/HE-1.md".into()]).is_err());
        assert!(p.check_path(&["model/Requirements/REQ-1.md".into(), "Requirements/REQ-1.md".into()]).is_ok());
        assert!(p.check_path(&["model/Other/secret.md".into()]).is_err(), "globstar");
        let e = p.check_path(&["model/Safety/X.md".into()]).unwrap_err();
        assert_eq!(e.rule, "protected_paths");
        assert!(e.to_string().contains("--as-human"), "it says how a person proceeds: {e}");
    }

    #[test]
    fn human_only_status_is_per_type() {
        let p = policy("[agents]\nhuman_only_status = { Requirement = [\"approved\"] }\n");
        assert!(p.check_status("Requirement", "approved").is_err());
        assert!(p.check_status("Requirement", "draft").is_ok());
        assert!(p.check_status("TestCase", "approved").is_ok(), "another type is unaffected");
    }

    fn pi(questions: &str) -> String {
        format!("---\nid: PI-1\ntype: PlanningItem\nname: P\nstatus: todo\nquestions:\n{questions}---\n\nx\n")
    }
    const OPEN: &str = "  - id: Q1\n    text: q\n    status: open\n";
    const OPEN_HUMAN: &str = "  - id: Q1\n    text: q\n    status: open\n    answerBy: human\n";
    const OPEN_ANY: &str = "  - id: Q1\n    text: q\n    status: open\n    answerBy: any\n";
    const ANSWERED: &str = "  - id: Q1\n    text: q\n    status: answered\n    answer: a\n";

    #[test]
    fn human_only_answers_loads_and_is_described() {
        let p = policy("[agents]\nhuman_only_answers = true\n");
        assert!(p.problems.is_empty() && p.human_only_answers);
        assert!(p.describe().join("\n").contains("answerBy: any"));
        let bad = policy("[agents]\nhuman_only_answers = \"yes\"\n");
        assert!(!bad.human_only_answers && bad.problems.iter().any(|m| m.contains("human_only_answers must be true or false")));
    }

    #[test]
    fn a_reserved_question_cannot_be_answered_by_a_change() {
        let on = policy("[agents]\nhuman_only_answers = true\n");
        // The policy default reserves every question...
        let r = check_answer_content(Some(&on), Some(&pi(OPEN)), &pi(ANSWERED)).unwrap_err();
        assert_eq!(r.rule, "human_only_answers");
        assert!(r.message.contains("PI-1#Q1"), "{r}");
        // ...an explicit `answerBy: any` wins over it...
        assert!(check_answer_content(Some(&on), Some(&pi(OPEN_ANY)), &pi(&format!("{ANSWERED}    answerBy: any\n"))).is_ok());
        // ...and `answerBy: human` reserves a question with no policy at all.
        assert!(check_answer_content(None, Some(&pi(OPEN_HUMAN)), &pi(&format!("{ANSWERED}    answerBy: human\n"))).is_err());
        assert!(check_answer_content(None, Some(&pi(OPEN)), &pi(ANSWERED)).is_ok(), "nothing is reserved by default");
        // A new file that arrives answered is an answer too; editing the text is not.
        assert!(check_answer_content(Some(&on), None, &pi(ANSWERED)).is_err());
        assert!(check_answer_content(Some(&on), Some(&pi(OPEN)), &pi("  - id: Q1\n    text: better q\n    status: open\n")).is_ok());
        // Nor may a change lift the reservation and answer in one go, or just lift it.
        assert!(check_answer_content(None, Some(&pi(OPEN_HUMAN)), &pi(&format!("{ANSWERED}    answerBy: any\n"))).is_err());
        assert!(check_answer_content(None, Some(&pi(OPEN_HUMAN)), &pi(OPEN_ANY)).unwrap_err().message.contains("lift"));
        // Reopening an answered question is not answering it.
        assert!(check_answer_content(Some(&on), Some(&pi(ANSWERED)), &pi(OPEN)).is_ok());
        // Not a PlanningItem: nothing to check.
        assert!(check_answer_content(Some(&on), None, "---\ntype: PartDef\nname: A\n---\n").is_ok());
    }

    #[test]
    fn the_rules_are_described_in_plain_words() {
        let p = policy("[agents]\nprotected_paths = [\"model/Safety/**\"]\nhuman_only_status = { Requirement = [\"approved\"] }\nrequire_by = true\nrequire_before_done = [\"evidence\"]\n");
        let d = p.describe().join("\n");
        assert!(d.contains("--by") && d.contains("`model/Safety/**`") && d.contains("Requirement to `approved`") && d.contains("resolving evidence"), "{d}");
    }
}

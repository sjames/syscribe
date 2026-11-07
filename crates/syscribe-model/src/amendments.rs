//! Amendment records on accepted ADRs and confirmed requirements (GH #454, `REQ-TRS-AMEND-001..004`).
//!
//! An `ADR` (and optionally a `Requirement`) may carry `amendments:`, a list of
//! `{date, by, reason, ref?, kind}` mappings recording each change made after the element was
//! accepted / confirmed:
//!
//! - `date` — ISO date `YYYY-MM-DD` the change was made;
//! - `by` — who made it (a username or a name);
//! - `reason` — why;
//! - `ref` — optional: a question (`PI-X#Qn`), a requirement item (`REQ-X#slug`), an element id
//!   (a `ReviewRecord`, …) or a git commit (7–40 hex);
//! - `kind` — `clarification` (the meaning is unchanged) or `scope-change` (it is not).
//!
//! Checks:
//!
//! - `E330` — the list or an entry is malformed, or a `ref` resolves to nothing.
//! - `W325` — a `scope-change` without an owner reference: no `ref`, or a `ref` that is not an
//!   answered question, a `ReviewRecord`, or a commit made by a person (see [`owner_reference`]).
//! - `W324` — behind `[traceability] require_amendments = true`: an accepted ADR or confirmed
//!   requirement whose normative content changed since the commit that set that status, with no
//!   amendment dated on or after the change (see [`change_findings`]).
//!
//! `amendments:` itself is excluded from the suspect projection (`crate::suspect`), so recording an
//! amendment never flips a link suspect; the edit it records does.

use std::collections::HashMap;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Serialize;
use serde_yaml::Value;

use crate::commit_evidence::CommitCheck;
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::validator::{Finding, Severity};

/// The amendment kinds.
pub const KINDS: &[&str] = &["clarification", "scope-change"];
/// The keys an amendment entry may carry.
pub const KEYS: &[&str] = &["date", "by", "reason", "ref", "kind"];
/// How many commits of one file's history `W324` reads, at most.
pub const HISTORY_LIMIT: usize = 200;

/// One well-formed amendment.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Amendment {
    pub date: String,
    pub by: String,
    pub reason: String,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub kind: String,
}

/// Whether elements of this type may carry `amendments:`.
pub fn hosts(et: Option<&ElementType>) -> bool {
    matches!(et, Some(ElementType::ADR | ElementType::Requirement))
}

/// The statuses at which an element of this type is accepted / confirmed, so an edit needs an
/// amendment (`W324`).
pub fn guarded_statuses(et: Option<&ElementType>) -> &'static [&'static str] {
    match et {
        Some(ElementType::ADR) => &["accepted"],
        Some(ElementType::Requirement) => &["approved", "implemented", "verified"],
        _ => &[],
    }
}

fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn scalar(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// `YYYY-MM-DD`, a real calendar date.
pub fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    if !(b.len() == 10 && b.iter().enumerate().all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() })) {
        return false;
    }
    let (y, m, d): (u32, u32, u32) = (s[0..4].parse().unwrap_or(0), s[5..7].parse().unwrap_or(0), s[8..10].parse().unwrap_or(0));
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let dim = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    d >= 1 && d <= dim
}

/// The amendments in one `amendments:` value and the problems found (`E330` messages). `owner` is
/// the element's id (else qualified name), used in messages.
pub fn parse_value(owner: &str, raw: &Value) -> (Vec<Amendment>, Vec<String>) {
    let mut out = Vec::new();
    let mut problems = Vec::new();
    let entries = match raw {
        Value::Null => return (out, problems),
        Value::Sequence(s) => s,
        _ => {
            problems.push(format!(
                "`amendments:` on {owner} must be a list of mappings `{{date, by, reason, kind, ref?}}`"
            ));
            return (out, problems);
        }
    };
    for (i, entry) in entries.iter().enumerate() {
        let n = i + 1;
        let Value::Mapping(m) = entry else {
            problems.push(format!("`amendments:` entry {n} on {owner} is not a mapping — write `- date: YYYY-MM-DD` with by, reason and kind"));
            continue;
        };
        let get = |k: &str| m.get(Value::String(k.into())).and_then(scalar).filter(|s| !s.is_empty());
        let mut bad = false;
        for k in m.keys() {
            let key = k.as_str().unwrap_or("");
            if !KEYS.contains(&key) {
                problems.push(format!("`amendments:` entry {n} on {owner} has unknown key `{key}` (expected {})", KEYS.join(", ")));
                bad = true;
            }
        }
        let mut missing = Vec::new();
        for k in ["date", "by", "reason", "kind"] {
            if get(k).is_none() {
                missing.push(k);
            }
        }
        if !missing.is_empty() {
            problems.push(format!("`amendments:` entry {n} on {owner} is missing {}", missing.iter().map(|k| format!("`{k}`")).collect::<Vec<_>>().join(", ")));
            bad = true;
        }
        let date = get("date").unwrap_or_default();
        if !date.is_empty() && !is_iso_date(&date) {
            problems.push(format!("`amendments:` entry {n} on {owner} has date `{date}`, not an ISO date (YYYY-MM-DD)"));
            bad = true;
        }
        let kind = get("kind").unwrap_or_default();
        if !kind.is_empty() && !KINDS.contains(&kind.as_str()) {
            problems.push(format!("`amendments:` entry {n} on {owner} has kind `{kind}` (expected {})", KINDS.join(" or ")));
            bad = true;
        }
        if bad {
            continue;
        }
        out.push(Amendment {
            date,
            by: get("by").unwrap_or_default(),
            reason: get("reason").unwrap_or_default(),
            reference: get("ref"),
            kind,
        });
    }
    (out, problems)
}

/// The amendments of one element and the problems in them; nothing for a type that does not host
/// them (`W047` covers those).
pub fn parse(e: &RawElement) -> (Vec<Amendment>, Vec<String>) {
    if !hosts(e.frontmatter.element_type.as_ref()) {
        return (Vec::new(), Vec::new());
    }
    match &e.frontmatter.amendments {
        Some(raw) => parse_value(&label(e), raw),
        None => (Vec::new(), Vec::new()),
    }
}

/// What an amendment's `ref` names.
#[derive(Debug, Clone)]
pub enum RefTarget<'a> {
    /// A PlanningItem question (`PI-X#Qn`).
    Question(crate::questions::Question),
    /// A requirement item (`REQ-X#slug`, `REQ-X#5`).
    Item(&'a RawElement),
    /// A model element.
    Element(&'a RawElement),
    /// A git commit: `(full sha, subject)`, or `None` when it is well formed but git could not be asked.
    Commit(Option<(String, String)>),
}

/// Whether `r` could be a commit abbreviation rather than an element id.
fn looks_like_commit(r: &str) -> bool {
    crate::commit_evidence::check_shape(r).is_ok()
}

/// Resolve an amendment `ref`: an item or question reference, an element (id or qualified name), or a
/// commit of the model's repository. `Err` is why it resolves to nothing.
pub fn resolve_ref<'a>(elements: &'a [RawElement], resolver: &Resolver, commits: &CommitCheck, r: &str) -> Result<RefTarget<'a>, String> {
    let r = r.trim();
    if let Some((elem, part)) = crate::req_items::parse_ref(r) {
        let Some(t) = crate::req_items::resolve_elem(elements, resolver, &elem) else {
            return Err(format!("`{r}` names no element `{elem}`"));
        };
        if !crate::req_items::applies(t, &part) {
            return Err(format!("`{r}`: {elem} has no {}", if matches!(part, crate::req_items::Part::Question(_)) { "questions" } else { "numbered items" }));
        }
        return match crate::req_items::lookup(t, &part)? {
            crate::req_items::Target::Question(q) => Ok(RefTarget::Question(q)),
            crate::req_items::Target::Item(_) => Ok(RefTarget::Item(t)),
        };
    }
    if let Some(t) = crate::req_items::resolve_elem(elements, resolver, r) {
        return Ok(RefTarget::Element(t));
    }
    if looks_like_commit(r) {
        return match commits.state(r) {
            crate::commit_evidence::State::Unchecked => Ok(RefTarget::Commit(None)),
            crate::commit_evidence::State::Resolved(crate::commit_evidence::Lookup::Found { sha, subject }) => Ok(RefTarget::Commit(Some((sha, subject)))),
            _ => Err(commits.problem(r).map(|p| p.replace("PlanningItem evidence `commit`", "commit")).unwrap_or_else(|| format!("`{r}` is not a commit"))),
        };
    }
    Err(format!("`{r}` resolves to no question, element or commit"))
}

/// A commit's author and full message, from the repository containing `top`.
fn commit_author_and_body(top: &Path, sha: &str) -> Option<(String, String)> {
    let o = Command::new("git").arg("-C").arg(top).args(["log", "-1", "--format=%an%x1f%B", sha]).stderr(Stdio::null()).output().ok()?;
    if !o.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&o.stdout).into_owned();
    let (a, b) = s.split_once('\u{1f}')?;
    Some((a.trim().to_string(), b.to_string()))
}

fn trailer(body: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}: ");
    body.lines().rev().find_map(|l| l.strip_prefix(&prefix)).map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Whether a resolved `ref` is an owner reference for a `scope-change` (`W325`); `Err` says why not.
///
/// An owner reference is one of:
/// - an **answered** PlanningItem question (`status: answered`, with an answer);
/// - a **`ReviewRecord`** element;
/// - a **commit made by a person**: its message carries `Syscribe-As: human`, or it has no
///   `Syscribe-Actor` trailer and its author matches none of `[agents] actors` (the audit's rule,
///   GH #442/#443). A well-formed commit that cannot be checked (no git) is accepted.
pub fn owner_reference(target: &RefTarget<'_>, commits: &CommitCheck, actors: &[String]) -> Result<(), String> {
    match target {
        RefTarget::Question(q) => {
            if q.status == "answered" && q.answer.is_some() {
                Ok(())
            } else {
                Err(format!("question {} is {}, not answered", q.reference(), q.status))
            }
        }
        RefTarget::Element(e) if e.frontmatter.element_type == Some(ElementType::ReviewRecord) => Ok(()),
        RefTarget::Element(e) => Err(format!(
            "{} is a {}, not a ReviewRecord",
            label(e),
            e.frontmatter.element_type.as_ref().map(|t| t.name()).unwrap_or("untyped element")
        )),
        RefTarget::Item(e) => Err(format!("a requirement item of {} records no owner decision", label(e))),
        RefTarget::Commit(None) => Ok(()),
        RefTarget::Commit(Some((sha, _))) => {
            let Some(repo) = commits.repo.as_ref() else { return Ok(()) };
            let Some((author, body)) = commit_author_and_body(repo.top(), sha) else { return Ok(()) };
            let human = trailer(&body, "Syscribe-As").as_deref() == Some("human");
            let actor = trailer(&body, "Syscribe-Actor");
            let by_author = actors.iter().any(|g| crate::annotations::glob_match(g, &author));
            if human || !(actor.is_some() || by_author) {
                Ok(())
            } else {
                Err(format!(
                    "commit {} was made by an agent ({})",
                    &sha[..sha.len().min(10)],
                    actor.unwrap_or(author)
                ))
            }
        }
    }
}

fn finding(code: &'static str, file: &str, message: String, severity: Severity) -> Finding {
    Finding { code, file: file.to_string(), message, severity }
}

/// `E330` (malformed `amendments:`, or a `ref` that resolves to nothing) and `W325` (a
/// `scope-change` without an owner reference). `actors` is `[agents] actors`.
pub fn findings(elements: &[RawElement], resolver: &Resolver, model_root: Option<&Path>, actors: &[String]) -> Vec<Finding> {
    let parsed: Vec<(&RawElement, Vec<Amendment>, Vec<String>)> = elements
        .iter()
        .filter(|e| e.frontmatter.amendments.is_some())
        .map(|e| {
            let (a, p) = parse(e);
            (e, a, p)
        })
        .collect();
    if parsed.is_empty() {
        return Vec::new();
    }
    // Every commit-shaped ref that names no element, resolved in one batch.
    let commit_entries: Vec<Value> = parsed
        .iter()
        .flat_map(|(_, a, _)| a.iter())
        .filter_map(|a| a.reference.as_deref())
        .filter(|r| looks_like_commit(r) && crate::req_items::resolve_elem(elements, resolver, r).is_none())
        .map(|r| Value::Mapping(serde_yaml::Mapping::from_iter([(Value::String("commit".into()), Value::String(r.to_string()))])))
        .collect();
    let commits = CommitCheck::new(model_root, commit_entries.iter());
    let mut out = Vec::new();
    for (e, list, problems) in &parsed {
        for p in problems {
            out.push(finding("E330", &e.file_path, format!("{p} — see `syscribe help {}` for the amendment shape", e.frontmatter.element_type.as_ref().map(|t| t.name()).unwrap_or("")), Severity::Error));
        }
        let owner = label(e);
        for a in list {
            let resolved = match a.reference.as_deref() {
                Some(r) => match resolve_ref(elements, resolver, &commits, r) {
                    Ok(t) => Some(t),
                    Err(why) => {
                        out.push(finding(
                            "E330",
                            &e.file_path,
                            format!("amendment of {} on {owner}: `ref` {why} — name an existing question (`PI-X#Qn`), element id or commit", a.date),
                            Severity::Error,
                        ));
                        continue;
                    }
                },
                None => None,
            };
            if a.kind != "scope-change" {
                continue;
            }
            let why = match &resolved {
                None => Some("it has no `ref`".to_string()),
                Some(t) => owner_reference(t, &commits, actors).err(),
            };
            if let Some(why) = why {
                out.push(finding(
                    "W325",
                    &e.file_path,
                    format!(
                        "scope-change amendment of {} on {owner} has no owner reference: {why} — set `ref:` to the answered question (`PI-X#Qn`), the ReviewRecord, or the commit in which the owner approved the change",
                        a.date
                    ),
                    Severity::Warning,
                ));
            }
        }
    }
    out
}

// ── W324: edits to an accepted / confirmed element without an amendment ──────────────────────

fn git_bytes(top: &Path, args: &[&str], stdin: Option<String>) -> Option<Vec<u8>> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(top).args(args).stdout(Stdio::piped()).stderr(Stdio::null());
    cmd.stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() });
    let mut child = cmd.spawn().ok()?;
    let writer = stdin.and_then(|input| {
        let mut pipe = child.stdin.take()?;
        Some(std::thread::spawn(move || {
            let _ = pipe.write_all(input.as_bytes());
        }))
    });
    let mut out = Vec::new();
    child.stdout.take()?.read_to_end(&mut out).ok()?;
    if let Some(w) = writer {
        let _ = w.join();
    }
    child.wait().ok()?.success().then_some(out)
}

/// One committed version of an element's file.
#[derive(Debug, Clone)]
struct Version {
    sha: String,
    /// Author date, `YYYY-MM-DD`.
    date: String,
    status: Option<String>,
    projection: Option<String>,
}

/// `(status, normative projection hash)` of a file's text, read as the element it defines.
fn project(content: &str, qualified_name: &str, file_path: &str) -> Option<(Option<String>, String)> {
    let (Some(yaml), body) = crate::frontmatter::split_frontmatter(content) else { return None };
    let fm = crate::frontmatter::parse_frontmatter(yaml).ok()?;
    let e = RawElement {
        qualified_name: qualified_name.to_string(),
        file_path: file_path.to_string(),
        frontmatter: fm,
        doc: body.to_string(),
        parse_issue: None,
        derived: Default::default(),
        derive_findings: Vec::new(),
        locale_docs: Default::default(),
        about_notes: Vec::new(),
    };
    Some((e.frontmatter.status.clone(), crate::suspect::projection_hash(&e)))
}

/// The newest-first history of one file (following renames), at most `limit` commits, each read
/// back through a single `git cat-file --batch`.
fn history(top: &Path, rel: &str, limit: usize, qualified_name: &str) -> Vec<Version> {
    let n = limit.to_string();
    let Some(log) = git_bytes(top, &["log", "--follow", "--format=%x1e%H%x1f%as", "--name-only", "-n", &n, "--", rel], None) else {
        return Vec::new();
    };
    let log = String::from_utf8_lossy(&log).into_owned();
    let mut commits: Vec<(String, String, String)> = Vec::new(); // (sha, date, path at that commit)
    for rec in log.split('\u{1e}').filter(|r| !r.trim().is_empty()) {
        let mut lines = rec.lines();
        let Some(head) = lines.next() else { continue };
        let Some((sha, date)) = head.split_once('\u{1f}') else { continue };
        let path = lines.map(str::trim).find(|l| !l.is_empty()).unwrap_or(rel).to_string();
        commits.push((sha.trim().to_string(), date.trim().to_string(), path));
    }
    if commits.is_empty() {
        return Vec::new();
    }
    let input: String = commits.iter().map(|(sha, _, path)| format!("{sha}:{path}\n")).collect();
    let Some(blob) = git_bytes(top, &["cat-file", "--batch"], Some(input)) else { return Vec::new() };
    let mut out = Vec::new();
    let mut pos = 0usize;
    for (sha, date, path) in commits {
        let Some(nl) = blob[pos..].iter().position(|b| *b == b'\n') else { break };
        let header = String::from_utf8_lossy(&blob[pos..pos + nl]).into_owned();
        pos += nl + 1;
        let mut parts = header.split_whitespace();
        let (_oid, kind, size) = (parts.next(), parts.next(), parts.next().and_then(|s| s.parse::<usize>().ok()));
        let (status, projection) = match (kind, size) {
            (Some("blob"), Some(size)) if pos + size <= blob.len() => {
                let content = String::from_utf8_lossy(&blob[pos..pos + size]).into_owned();
                pos += size + 1; // the content and its trailing newline
                match project(&content, qualified_name, &path) {
                    Some((s, p)) => (s, Some(p)),
                    None => (None, None),
                }
            }
            (Some(_), Some(size)) if pos + size <= blob.len() => {
                pos += size + 1;
                (None, None)
            }
            _ => (None, None), // `<name> missing`
        };
        out.push(Version { sha, date, status, projection });
    }
    out
}

/// When the normative content of an element last changed after it reached a guarded status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unrecorded {
    /// The commit that set the guarded status (the oldest commit of the newest unbroken run of
    /// commits at a guarded status).
    pub status_commit: String,
    pub status_date: String,
    /// The commit of the latest normative change, `None` for an uncommitted working-tree edit.
    pub change_commit: Option<String>,
    /// Its date (`YYYY-MM-DD`; today for a working-tree edit).
    pub change_date: String,
}

/// The latest normative change of `e` since it reached a guarded status, from git: `None` when there
/// is none, when git or the repository is missing, or when the status was set only in the working tree.
pub fn latest_change(top: &Path, e: &RawElement, limit: usize, today: &str) -> Option<Unrecorded> {
    let guarded = guarded_statuses(e.frontmatter.element_type.as_ref());
    let is_guarded = |s: &Option<String>| s.as_deref().is_some_and(|s| guarded.contains(&s));
    let file = std::fs::canonicalize(&e.file_path).unwrap_or_else(|_| PathBuf::from(&e.file_path));
    let rel = file.strip_prefix(top).ok()?.to_string_lossy().replace('\\', "/");
    let content = std::fs::read_to_string(&file).ok()?;
    let (wt_status, wt_projection) = project(&content, &e.qualified_name, &e.file_path)?;
    if !is_guarded(&wt_status) {
        return None;
    }
    let versions = history(top, &rel, limit, &e.qualified_name);
    let run = versions.iter().take_while(|v| is_guarded(&v.status)).count();
    if run == 0 {
        return None; // the status was set in the working tree: nothing has changed since
    }
    let status = &versions[run - 1];
    let unrecorded = |change_commit: Option<String>, change_date: String| Unrecorded {
        status_commit: status.sha.clone(),
        status_date: status.date.clone(),
        change_commit,
        change_date,
    };
    if versions[0].projection.as_deref() != Some(wt_projection.as_str()) {
        return Some(unrecorded(None, today.to_string()));
    }
    for i in 0..run - 1 {
        if versions[i].projection != versions[i + 1].projection {
            return Some(unrecorded(Some(versions[i].sha.clone()), versions[i].date.clone()));
        }
    }
    None
}

/// `W324`: every accepted ADR / confirmed Requirement whose normative content (body and normative
/// frontmatter — the suspect projection, which leaves out `status`, `amendments` and editorial
/// fields) changed since the commit that set that status, with no well-formed `amendments:` entry
/// dated on or after the latest such change. Uncommitted edits count, dated today. Without git, or
/// outside a repository, nothing is reported. At most `limit` commits of each file are read.
pub fn change_findings(elements: &[RawElement], model_root: &Path, limit: usize, today: &str) -> Vec<Finding> {
    let candidates: Vec<&RawElement> = elements
        .iter()
        .filter(|e| {
            let g = guarded_statuses(e.frontmatter.element_type.as_ref());
            !g.is_empty() && e.frontmatter.status.as_deref().is_some_and(|s| g.contains(&s))
        })
        .collect();
    if candidates.is_empty() {
        return Vec::new();
    }
    let Some(repo) = crate::commit_evidence::Repo::discover(model_root) else { return Vec::new() };
    let mut out = Vec::new();
    let mut cache: HashMap<String, Option<Unrecorded>> = HashMap::new();
    for e in candidates {
        let Some(u) = cache.entry(e.file_path.clone()).or_insert_with(|| latest_change(repo.top(), e, limit, today)).clone() else { continue };
        let (amendments, _) = parse(e);
        if amendments.iter().any(|a| a.date.as_str() >= u.change_date.as_str()) {
            continue;
        }
        let id = label(e);
        let status = e.frontmatter.status.clone().unwrap_or_default();
        let when = match &u.change_commit {
            Some(c) => format!("in commit {} on {}", &c[..c.len().min(10)], u.change_date),
            None => "in an uncommitted working-tree edit".to_string(),
        };
        out.push(finding(
            "W324",
            &e.file_path,
            format!(
                "{id} is `{status}` (since commit {} on {}) but its normative content changed {when} with no `amendments:` entry dated {} or later — record the change: `syscribe set {id} amendment.add kind=clarification|scope-change reason=\"…\" [ref=…] --by <you>`",
                &u.status_commit[..u.status_commit.len().min(10)],
                u.status_date,
                u.change_date,
            ),
            Severity::Warning,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yaml(s: &str) -> Value {
        serde_yaml::from_str(s).unwrap()
    }

    #[test]
    fn a_well_formed_list_parses() {
        let (a, p) = parse_value("ADR-1", &yaml("- date: 2026-03-01\n  by: alice\n  reason: covers B too\n  kind: scope-change\n  ref: PI-1#Q2\n"));
        assert!(p.is_empty(), "{p:?}");
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].reference.as_deref(), Some("PI-1#Q2"));
        assert_eq!(a[0].kind, "scope-change");
    }

    #[test]
    fn malformed_entries_are_reported() {
        let (_, p) = parse_value("ADR-1", &yaml("date: 2026-03-01\n"));
        assert!(p[0].contains("must be a list"), "{p:?}");
        let (a, p) = parse_value(
            "ADR-1",
            &yaml("- just text\n- date: 2026-02-30\n  by: a\n  reason: r\n  kind: clarification\n- date: 2026-03-01\n  by: a\n  reason: r\n  kind: tweak\n- by: a\n  extra: 1\n"),
        );
        assert!(a.is_empty());
        let all = p.join("\n");
        assert!(all.contains("entry 1") && all.contains("not a mapping"), "{all}");
        assert!(all.contains("not an ISO date"), "{all}");
        assert!(all.contains("kind `tweak`"), "{all}");
        assert!(all.contains("unknown key `extra`") && all.contains("missing `date`, `reason`, `kind`"), "{all}");
    }

    #[test]
    fn iso_dates() {
        assert!(is_iso_date("2024-02-29"));
        assert!(!is_iso_date("2023-02-29"));
        assert!(!is_iso_date("2026-3-01"));
        assert!(!is_iso_date("2026-13-01"));
    }
}

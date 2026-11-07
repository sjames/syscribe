//! `log [<element>]` — the history of the model as git records it (GH #443).
//!
//! There is no separate log file. A writing command run with `--commit` leaves a commit whose author is
//! the actor (`--by`) and whose message carries `Syscribe-Command` / `Syscribe-Actor` / `Syscribe-Reason`
//! trailers; this reads that history back. For one element it also shows what each commit changed in it
//! (status moves, links, evidence). `--unattributed` lists the commits that touched the model without
//! those trailers — the hand edits.

use serde::Serialize;
use std::path::Path;
use syscribe_model::config::detect_git_root;
use syscribe_model::element::RawElement;
use syscribe_model::modeldiff;
use syscribe_model::resolver::Resolver;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub commit: String,
    pub date: String,
    pub author: String,
    pub subject: String,
    /// `Syscribe-Actor`, when the change was attributed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Made with a syscribe command that left its trailers (so not a hand edit).
    pub attributed: bool,
    pub files: Vec<String>,
    /// For `log <element>`: what this commit changed in the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<String>,
}

const RS: char = '\u{1e}';
const US: char = '\u{1f}';

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let o = std::process::Command::new("git").arg("-C").arg(root).args(args).output().map_err(|e| format!("cannot run git: {e}"))?;
    if !o.status.success() {
        return Err(format!("git {}: {}", args.first().unwrap_or(&""), String::from_utf8_lossy(&o.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&o.stdout).into_owned())
}

fn trailer(body: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}: ");
    body.lines().rev().find_map(|l| l.strip_prefix(&prefix)).map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Commits touching `paths` (newest first), parsed from `git log`.
fn commits(root: &Path, paths: &[String], follow: bool, since: Option<&str>, limit: usize) -> Result<Vec<Entry>, String> {
    let fmt = format!("--format={RS}%h{US}%an{US}%aI{US}%s{US}%B{US}");
    let mut args: Vec<String> = vec!["log".into(), fmt, "--name-only".into(), format!("--max-count={limit}")];
    if follow {
        args.push("--follow".into());
    }
    if let Some(s) = since {
        args.push(format!("--since={s}"));
    }
    args.push("--".into());
    args.extend(paths.iter().cloned());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    Ok(parse_entries(&git(root, &refs)?))
}

/// Entries from `git log` output in the `RS`/`US` format above.
fn parse_entries(out: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    for rec in out.split(RS).filter(|r| !r.trim().is_empty()) {
        let f: Vec<&str> = rec.split(US).collect();
        if f.len() < 6 {
            continue;
        }
        let body = f[4];
        let command = trailer(body, "Syscribe-Command");
        entries.push(Entry {
            commit: f[0].to_string(),
            date: f[2].to_string(),
            author: f[1].to_string(),
            subject: f[3].to_string(),
            actor: trailer(body, "Syscribe-Actor"),
            reason: trailer(body, "Syscribe-Reason"),
            attributed: command.is_some(),
            command,
            files: f[5].lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect(),
            change: None,
        });
    }
    entries
}

fn show_file(root: &Path, rev: &str, path: &str) -> Option<String> {
    git(root, &["show", &format!("{rev}:{path}")]).ok()
}

/// Model files with uncommitted changes (modified, staged or untracked), git-root-relative.
fn uncommitted(root: &Path, model_rel: &str) -> Vec<String> {
    let mut args = vec!["status", "--porcelain=v1", "-z", "--untracked-files=all"];
    if !model_rel.is_empty() {
        args.push("--");
        args.push(model_rel);
    }
    git(root, &args).map(|o| o.split('\0').filter(|l| l.len() > 3).map(|l| l[3..].to_string()).collect()).unwrap_or_default()
}

/// The data for `log`, shared with the MCP `log` tool.
pub struct Query<'a> {
    pub element: Option<&'a str>,
    pub by: Option<&'a str>,
    pub since: Option<&'a str>,
    pub unattributed: bool,
    /// `--item <PI>`: only the commits that PlanningItem names as `evidence: - commit:` (GH #455).
    pub item: Option<&'a str>,
    pub limit: usize,
}

pub struct Log {
    pub scope: String,
    pub entries: Vec<Entry>,
    pub uncommitted: Vec<String>,
}

/// `log --item <PI>` (GH #455): the commits a PlanningItem names as `evidence: - commit:`, in
/// authored order. A hash that does not resolve is reported in the scope line, not as an error.
fn item_commits(root: &Path, elements: &[RawElement], item: &str) -> Result<(String, Vec<Entry>), String> {
    let e = Resolver::new(elements).resolve_ref(elements, item).ok_or_else(|| format!("element not found: {item}"))?;
    if e.frontmatter.element_type != Some(syscribe_model::element::ElementType::PlanningItem) {
        return Err(format!("{item} is not a PlanningItem"));
    }
    let label = e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone());
    let raws: Vec<String> = e.frontmatter.evidence.iter().flatten().filter_map(syscribe_model::commit_evidence::commit_value).collect();
    let check = syscribe_model::commit_evidence::CommitCheck::new(Some(root), e.frontmatter.evidence.iter().flatten());
    let mut fulls: Vec<String> = Vec::new();
    let mut unresolved: Vec<String> = Vec::new();
    for r in &raws {
        match check.found(r) {
            Some((sha, _)) if !fulls.contains(&sha) => fulls.push(sha),
            Some(_) => {}
            None => unresolved.push(r.clone()),
        }
    }
    let mut scope = format!("the commits {label} names as evidence");
    if !unresolved.is_empty() {
        scope.push_str(&format!(" ({} not found in this repository: {})", unresolved.len(), unresolved.join(", ")));
    }
    if fulls.is_empty() {
        return Ok((scope, Vec::new()));
    }
    let fmt = format!("--format={RS}%h{US}%an{US}%aI{US}%s{US}%B{US}");
    let mut args: Vec<String> = vec!["log".into(), "--no-walk=unsorted".into(), fmt, "--name-only".into(), "-m".into(), "--first-parent".into()];
    args.extend(fulls);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    Ok((scope, parse_entries(&git(root, &refs)?)))
}

pub fn collect(model_root: &Path, elements: &[RawElement], q: &Query) -> Result<Log, String> {
    let abs = std::fs::canonicalize(model_root).map_err(|e| format!("{}: {e}", model_root.display()))?;
    let root = detect_git_root(&abs).ok_or_else(|| format!("{} is not inside a git repository, so there is no history", model_root.display()))?;
    if let Some(item) = q.item {
        let (scope, mut entries) = item_commits(&root, elements, item)?;
        if let Some(by) = q.by {
            entries.retain(|e| e.author == by || e.actor.as_deref() == Some(by));
        }
        entries.truncate(q.limit);
        return Ok(Log { scope, entries, uncommitted: Vec::new() });
    }
    let model_rel = abs.strip_prefix(&root).unwrap_or(Path::new("")).to_string_lossy().replace('\\', "/");
    let (paths, follow, scope, file_rel) = match q.element {
        None => (vec![if model_rel.is_empty() { ".".to_string() } else { model_rel.clone() }], false, format!("the model ({})", if model_rel.is_empty() { "." } else { &model_rel }), None),
        Some(r) => {
            let e = Resolver::new(elements).resolve_ref(elements, r).ok_or_else(|| format!("element not found: {r}"))?.clone();
            let rel = crate::impactfiles::to_git_rel(&root, Path::new(&e.file_path)).ok_or_else(|| format!("{r} is not inside the repository"))?;
            (vec![rel.clone()], true, e.frontmatter.id.clone().unwrap_or(e.qualified_name.clone()), Some(rel))
        }
    };
    // Filters apply after the fact, so ask git for more than will be shown.
    let ask = if q.by.is_some() || q.unattributed { q.limit.saturating_mul(10).max(200) } else { q.limit };
    let mut entries = commits(&root, &paths, follow, q.since, ask)?;
    if q.unattributed {
        entries.retain(|e| !e.attributed);
    }
    if let Some(by) = q.by {
        entries.retain(|e| e.author == by || e.actor.as_deref() == Some(by));
    }
    entries.truncate(q.limit);
    if let Some(rel) = &file_rel {
        for e in &mut entries {
            // A rename shows up as the file's new path in `files`; read each side from the right place.
            let path_now = e.files.first().cloned().unwrap_or_else(|| rel.clone());
            let new = show_file(&root, &e.commit, &path_now);
            let old = show_file(&root, &format!("{}^", e.commit), &path_now);
            e.change = modeldiff::change_between(&path_now, old.as_deref(), new.as_deref()).map(|c| c.summary).filter(|s| !s.is_empty());
        }
    }
    let dirty = uncommitted(&root, &model_rel);
    let uncommitted = match &file_rel {
        Some(rel) => dirty.into_iter().filter(|p| p == rel).collect(),
        None => dirty,
    };
    Ok(Log { scope, entries, uncommitted })
}

/// `args` are the tokens after `log`. Returns the exit code.
pub fn cmd_log(model_root: &Path, elements: &[RawElement], args: &[String]) -> i32 {
    let value = |flag: &str| args.windows(2).find(|w| w[0] == flag).map(|w| w[1].as_str());
    let json = args.iter().any(|a| a == "--json");
    let limit = match crate::cliargs::usize_value("log", args, "--limit") {
        Ok(l) => l.unwrap_or(50),
        Err(m) => {
            eprintln!("{m}");
            return 1;
        }
    };
    let mut skip = false;
    let element = args.iter().find(|a| {
        if skip {
            skip = false;
            return false;
        }
        if matches!(a.as_str(), "--by" | "--since" | "--limit" | "--item") {
            skip = true;
            return false;
        }
        !a.starts_with("--")
    });
    let q = Query { element: element.map(String::as_str), by: value("--by"), since: value("--since"), unattributed: args.iter().any(|a| a == "--unattributed"), item: value("--item"), limit };
    let log = match collect(model_root, elements, &q) {
        Ok(l) => l,
        Err(m) => {
            eprintln!("log: {m}");
            return 1;
        }
    };
    if json {
        let doc = serde_json::json!({ "schemaVersion": "1.0", "scope": log.scope, "uncommitted": log.uncommitted, "entries": log.entries });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return 0;
    }
    println!("# History of {}\n", log.scope);
    if !log.uncommitted.is_empty() {
        println!("Uncommitted changes in {} file(s) are not in the history yet: {}\n", log.uncommitted.len(), log.uncommitted.join(", "));
    }
    if log.entries.is_empty() {
        println!("No matching commits.");
        return 0;
    }
    for e in &log.entries {
        let who = e.actor.as_deref().map(|a| format!("{a} (as {})", e.author)).filter(|_| e.actor.as_deref() != Some(e.author.as_str())).unwrap_or_else(|| e.author.clone());
        let day = e.date.split('T').next().unwrap_or(&e.date);
        println!("{}  {}  {}  {}{}", e.commit, day, who, e.subject, if e.attributed { "" } else { "   [not made by syscribe]" });
        if let Some(c) = &e.change {
            println!("    {c}");
        } else if q.element.is_none() {
            println!("    {} file(s)", e.files.len());
        }
        if let Some(r) = &e.reason {
            println!("    reason: {r}");
        }
    }
    0
}

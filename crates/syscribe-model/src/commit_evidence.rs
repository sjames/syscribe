//! PlanningItem `evidence: - commit: <sha>` entries, resolved against git (GH #455,
//! `REQ-TRS-PICOMMIT-001`).
//!
//! A commit entry names the commit that delivered the work. `validate` checks it against the git
//! repository that contains the model (`E728`), counts a resolving one as evidence for `E719`, and
//! — behind `[planning]` — checks that it touches a path the item names (`W320`) and that a done
//! leaf has one (`W321`). `steps`, `context`, `set … evidence.add commit=` and `log --item` share
//! this module.
//!
//! Git is asked as little as possible: every distinct hash of one run is resolved by a single
//! `git cat-file --batch-check` and its subject read by a single `git log --no-walk`, and the
//! changed paths (only when `W320` is on) by one more `git log`. A resolved commit is immutable, so
//! it is also cached for the life of the process (a long-running MCP server or LSP); an unknown or
//! ambiguous hash is asked again next time, since a later commit or fetch can change the answer.
//! When git is missing or the model is not inside a repository, nothing is checked.

use std::collections::HashMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};

/// The shortest abbreviation accepted.
pub const MIN_LEN: usize = 7;
/// A full SHA-1 object name.
pub const MAX_LEN: usize = 40;

/// What one commit hash resolved to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup {
    /// A commit: its full object name and subject line.
    Found { sha: String, subject: String },
    /// No object with this name or prefix.
    Unknown,
    /// The abbreviation matches more than one object.
    Ambiguous,
    /// The name resolves, but to an object that is not a commit (its type).
    NotACommit(String),
}

/// The `commit:` value of an evidence entry as text, if the entry has one. A number (an
/// all-digit hash YAML read as an integer) is turned back into its digits.
pub fn commit_value(entry: &serde_yaml::Value) -> Option<String> {
    let v = entry.as_mapping()?.get(serde_yaml::Value::String("commit".into()))?;
    Some(match v {
        serde_yaml::Value::String(s) => s.trim().to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Null => String::new(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
    })
}

/// Check the shape of a hash: 7–40 hexadecimal characters. Returns the lower-cased hash, or why
/// it is malformed.
pub fn check_shape(raw: &str) -> Result<String, String> {
    let s = raw.trim();
    if s.is_empty() {
        return Err("it is empty".into());
    }
    if !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("it is not hexadecimal".into());
    }
    if s.len() < MIN_LEN {
        return Err(format!("it has {} characters, fewer than {MIN_LEN}", s.len()));
    }
    if s.len() > MAX_LEN {
        return Err(format!("it has {} characters, more than {MAX_LEN}", s.len()));
    }
    Ok(s.to_ascii_lowercase())
}

/// The git repository a model lives in.
#[derive(Debug, Clone)]
pub struct Repo {
    top: PathBuf,
}

type FoundCache = Mutex<HashMap<(PathBuf, String), (String, String)>>;
type PathCache = Mutex<HashMap<(PathBuf, String), Vec<String>>>;

fn found_cache() -> &'static FoundCache {
    static C: OnceLock<FoundCache> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

fn path_cache() -> &'static PathCache {
    static C: OnceLock<PathCache> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

fn git(top: &Path, args: &[&str], stdin: Option<&str>) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(top).args(args).stdout(Stdio::piped()).stderr(Stdio::null());
    cmd.stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() });
    let mut child = cmd.spawn().ok()?;
    if let Some(input) = stdin {
        // Written before reading: the input is small (one line per hash).
        let mut pipe = child.stdin.take()?;
        pipe.write_all(input.as_bytes()).ok()?;
    }
    let out = child.wait_with_output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

impl Repo {
    /// The repository containing `dir`, or `None` when git cannot be run or `dir` is in no repository.
    pub fn discover(dir: &Path) -> Option<Repo> {
        let top = git(dir, &["rev-parse", "--show-toplevel"], None)?;
        let top = PathBuf::from(top.trim());
        let top = std::fs::canonicalize(&top).unwrap_or(top);
        Some(Repo { top })
    }

    /// The working-tree root.
    pub fn top(&self) -> &Path {
        &self.top
    }

    /// Resolve every hash in `hashes` (already shape-checked, lower case). One `cat-file` for all
    /// of them and one `log` for the subjects of the commits not yet cached.
    pub fn resolve(&self, hashes: &[String]) -> HashMap<String, Lookup> {
        let mut out: HashMap<String, Lookup> = HashMap::new();
        let mut ask: Vec<&str> = Vec::new();
        {
            let cache = found_cache().lock().unwrap_or_else(|e| e.into_inner());
            for h in hashes {
                if out.contains_key(h) || ask.contains(&h.as_str()) {
                    continue;
                }
                match cache.get(&(self.top.clone(), h.clone())) {
                    Some((sha, subject)) => {
                        out.insert(h.clone(), Lookup::Found { sha: sha.clone(), subject: subject.clone() });
                    }
                    None => ask.push(h),
                }
            }
        }
        if ask.is_empty() {
            return out;
        }
        let input: String = ask.iter().map(|h| format!("{h}\n")).collect();
        let Some(listing) = git(&self.top, &["cat-file", "--batch-check=%(objectname) %(objecttype)"], Some(&input)) else {
            // git failed as a whole: nothing can be said, so nothing is reported.
            return out;
        };
        let mut commits: Vec<(String, String)> = Vec::new(); // (asked, full)
        for (h, line) in ask.iter().zip(listing.lines()) {
            let mut parts = line.split_whitespace();
            let (a, b) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
            let lookup = match b {
                "missing" => Lookup::Unknown,
                "ambiguous" => Lookup::Ambiguous,
                "commit" => {
                    commits.push((h.to_string(), a.to_string()));
                    continue;
                }
                "" => Lookup::Unknown,
                other => Lookup::NotACommit(other.to_string()),
            };
            out.insert(h.to_string(), lookup);
        }
        if !commits.is_empty() {
            let mut args: Vec<String> = vec!["log".into(), "--no-walk=unsorted".into(), "--format=%H%x1f%s".into()];
            args.extend(commits.iter().map(|(_, full)| full.clone()));
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            let subjects: HashMap<String, String> = git(&self.top, &refs, None)
                .unwrap_or_default()
                .lines()
                .filter_map(|l| l.split_once('\u{1f}').map(|(h, s)| (h.to_string(), s.to_string())))
                .collect();
            let mut cache = found_cache().lock().unwrap_or_else(|e| e.into_inner());
            for (h, full) in commits {
                let subject = subjects.get(&full).cloned().unwrap_or_default();
                cache.insert((self.top.clone(), h.clone()), (full.clone(), subject.clone()));
                out.insert(h, Lookup::Found { sha: full, subject });
            }
        }
        out
    }

    /// The paths each commit (full object names) changed, as absolute paths under [`Self::top`].
    /// A merge is compared with its first parent; a root commit lists every file it added.
    pub fn changed_paths(&self, full: &[String]) -> HashMap<String, Vec<PathBuf>> {
        let mut out: HashMap<String, Vec<String>> = HashMap::new();
        let mut ask: Vec<&str> = Vec::new();
        {
            let cache = path_cache().lock().unwrap_or_else(|e| e.into_inner());
            for f in full {
                match cache.get(&(self.top.clone(), f.clone())) {
                    Some(p) => {
                        out.insert(f.clone(), p.clone());
                    }
                    None if !ask.contains(&f.as_str()) => ask.push(f),
                    None => {}
                }
            }
        }
        if !ask.is_empty() {
            let mut args: Vec<&str> = vec!["log", "--no-walk=unsorted", "-m", "--first-parent", "--name-only", "--format=%x1e%H"];
            args.extend(ask.iter().copied());
            if let Some(text) = git(&self.top, &args, None) {
                let mut cache = path_cache().lock().unwrap_or_else(|e| e.into_inner());
                for rec in text.split('\u{1e}').filter(|r| !r.trim().is_empty()) {
                    let mut lines = rec.lines();
                    let Some(sha) = lines.next().map(str::trim) else { continue };
                    let paths: Vec<String> = lines.map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect();
                    cache.insert((self.top.clone(), sha.to_string()), paths.clone());
                    out.insert(sha.to_string(), paths);
                }
            }
        }
        out.into_iter().map(|(k, v)| (k, v.into_iter().map(|p| self.top.join(p)).collect())).collect()
    }
}

/// The commit entries of a set of evidence lists, checked in one batch.
#[derive(Debug, Default)]
pub struct CommitCheck {
    /// `None` when git is missing or the model is in no repository: nothing is checked.
    pub repo: Option<Repo>,
    lookups: HashMap<String, Lookup>,
}

/// The state of one `commit:` value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Malformed(String),
    /// Well formed, but not checked (no git, or no repository).
    Unchecked,
    Resolved(Lookup),
}

impl CommitCheck {
    /// Resolve every well-formed commit value in `entries` against the repository containing `dir`.
    pub fn new<'a>(dir: Option<&Path>, entries: impl IntoIterator<Item = &'a serde_yaml::Value>) -> Self {
        let hashes: Vec<String> = entries.into_iter().filter_map(commit_value).filter_map(|v| check_shape(&v).ok()).collect();
        if hashes.is_empty() {
            return CommitCheck::default();
        }
        let Some(repo) = dir.and_then(Repo::discover) else { return CommitCheck::default() };
        let lookups = repo.resolve(&hashes);
        CommitCheck { repo: Some(repo), lookups }
    }

    /// The state of one raw `commit:` value.
    pub fn state(&self, raw: &str) -> State {
        let h = match check_shape(raw) {
            Ok(h) => h,
            Err(why) => return State::Malformed(why),
        };
        if self.repo.is_none() {
            return State::Unchecked;
        }
        // A hash git could not be asked about (git failed mid-run) is left unchecked too.
        self.lookups.get(&h).cloned().map(State::Resolved).unwrap_or(State::Unchecked)
    }

    /// Whether a raw value counts as resolving evidence: it resolves to a commit, or it is well
    /// formed and git could not be asked.
    pub fn resolves(&self, raw: &str) -> bool {
        matches!(self.state(raw), State::Unchecked | State::Resolved(Lookup::Found { .. }))
    }

    /// The `E728` message for a raw value, or `None` when it is fine (or cannot be checked).
    pub fn problem(&self, raw: &str) -> Option<String> {
        let top = self.repo.as_ref().map(|r| r.top().display().to_string()).unwrap_or_default();
        match self.state(raw) {
            State::Malformed(why) => Some(format!(
                "PlanningItem evidence `commit` '{raw}' is malformed: {why} (expected {MIN_LEN}–{MAX_LEN} hexadecimal characters; quote an all-digit hash)"
            )),
            State::Unchecked | State::Resolved(Lookup::Found { .. }) => None,
            State::Resolved(Lookup::Unknown) => Some(format!("PlanningItem evidence `commit` '{raw}' is unknown: no commit of the repository at {top} has that name (a typo, or rebased or not fetched?)")),
            State::Resolved(Lookup::Ambiguous) => Some(format!("PlanningItem evidence `commit` '{raw}' is ambiguous: it abbreviates more than one object — give more characters")),
            State::Resolved(Lookup::NotACommit(t)) => Some(format!("PlanningItem evidence `commit` '{raw}' names a {t}, not a commit")),
        }
    }

    /// The commit a raw value resolved to, if any.
    pub fn found(&self, raw: &str) -> Option<(String, String)> {
        match self.state(raw) {
            State::Resolved(Lookup::Found { sha, subject }) => Some((sha, subject)),
            _ => None,
        }
    }
}

/// A JSON description of one commit entry for the read views (`steps`, `context`):
/// `{commit, sha?, subject?, status}`, `status` one of `resolved`, `unknown`, `ambiguous`,
/// `not-a-commit`, `malformed`, `unchecked`.
pub fn describe(check: &CommitCheck, raw: &str) -> serde_json::Value {
    let mut m = serde_json::Map::new();
    m.insert("commit".into(), serde_json::Value::String(raw.to_string()));
    let status = match check.state(raw) {
        State::Malformed(_) => "malformed",
        State::Unchecked => "unchecked",
        State::Resolved(Lookup::Found { sha, subject }) => {
            m.insert("sha".into(), serde_json::Value::String(sha));
            m.insert("subject".into(), serde_json::Value::String(subject));
            "resolved"
        }
        State::Resolved(Lookup::Unknown) => "unknown",
        State::Resolved(Lookup::Ambiguous) => "ambiguous",
        State::Resolved(Lookup::NotACommit(_)) => "not-a-commit",
    };
    m.insert("status".into(), serde_json::Value::String(status.into()));
    serde_json::Value::Object(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_is_seven_to_forty_hex() {
        assert_eq!(check_shape("ABCDEF1").unwrap(), "abcdef1");
        assert!(check_shape("abcdef").unwrap_err().contains("fewer than 7"));
        assert!(check_shape(&"a".repeat(41)).unwrap_err().contains("more than 40"));
        assert!(check_shape("abcdefg").unwrap_err().contains("not hexadecimal"));
        assert!(check_shape("").is_err());
    }

    #[test]
    fn an_all_digit_hash_read_as_a_number_keeps_its_digits() {
        let v: serde_yaml::Value = serde_yaml::from_str("commit: 1234567").unwrap();
        assert_eq!(commit_value(&v).as_deref(), Some("1234567"));
        let v: serde_yaml::Value = serde_yaml::from_str("ref: TC-1").unwrap();
        assert_eq!(commit_value(&v), None);
    }

    #[test]
    fn without_a_repository_a_well_formed_hash_is_unchecked_and_counts() {
        let c = CommitCheck::default();
        assert_eq!(c.state("abcdef1"), State::Unchecked);
        assert!(c.resolves("abcdef1"));
        assert!(c.problem("abcdef1").is_none());
        assert!(!c.resolves("xyz"));
        assert!(c.problem("xyz").unwrap().contains("malformed"));
    }
}

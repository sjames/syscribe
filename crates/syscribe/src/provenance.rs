//! Provenance for tool changes, stored in git (GH #443, #444).
//!
//! Git is the only record. A writing command may be given `--by <actor>` and `--reason <text>`; with
//! `--commit` (or `[provenance] commit = true` in `.syscribe.toml`) the files it changed are committed
//! — only those files, so another agent's pending edits are never swept in — with the actor as the
//! commit author and `Syscribe-*` trailers carrying the command, actor and reason. `syscribe log`
//! reads that history back. `--expect-hash` makes an edit based on a stale read fail instead of
//! overwriting a concurrent change.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use syscribe_model::config::detect_git_root;

#[derive(Debug, Clone, Default)]
pub struct Globals {
    /// `--json` was given: writers that report through [`say!`] emit one result envelope instead of text.
    pub json: bool,
    pub dry_run: bool,
    pub commit: bool,
    pub by: Option<String>,
    pub reason: Option<String>,
    pub expect_hash: Option<String>,
    /// The command (`claim`) gives `--by` its own, free-form meaning, so it is not validated as an actor id
    /// unless it is also used as a commit author.
    pub opaque_by: bool,
    /// A global flag was given without its value; reported by [`init`] as a usage error.
    pub error: Option<String>,
}

static GLOBALS: OnceLock<Globals> = OnceLock::new();

pub fn get() -> &'static Globals {
    GLOBALS.get_or_init(Globals::default)
}

/// Install the globals. An invalid actor id is a usage error.
pub fn init(g: Globals) -> Result<(), String> {
    if let Some(e) = &g.error {
        return Err(e.clone());
    }
    if let Some(by) = &g.by {
        // `claim --by` is an opaque claimant id; it only has to be a safe author name when it also commits.
        if (!g.opaque_by || g.commit) && !valid_actor(by) {
            return Err(format!("--by '{by}' is not a valid actor id (letters, digits, '.', '_' and '-', at most 64)"));
        }
    }
    let _ = GLOBALS.set(g);
    Ok(())
}

/// What a writing command said and changed, kept so `--json` can report it as one envelope (GH #444).
#[derive(Default)]
struct Capture {
    used: bool,
    messages: Vec<String>,
    diff: String,
}

static CAPTURE: std::sync::Mutex<Capture> = std::sync::Mutex::new(Capture { used: false, messages: Vec::new(), diff: String::new() });

/// True when writers should not print text because `--json` asked for the envelope.
pub fn json_out() -> bool {
    get().json
}

#[doc(hidden)]
pub fn say_impl(line: String) {
    if let Ok(mut c) = CAPTURE.lock() {
        c.used = true;
        c.messages.push(line.clone());
    }
    if !json_out() {
        println!("{line}");
    }
}

/// A unified diff a writer produced: shown as text, or carried in the `--json` envelope.
pub fn record_diff(diff: &str) {
    if let Ok(mut c) = CAPTURE.lock() {
        c.used = true;
        c.diff.push_str(diff);
    }
    if !json_out() {
        print!("{diff}");
    }
}

/// Like `println!`, for the status lines of a writing command: suppressed under `--json`, which reports
/// them in the envelope instead.
#[macro_export]
macro_rules! say {
    ($($t:tt)*) => { $crate::provenance::say_impl(format!($($t)*)) };
}

/// An actor id usable as a git author name and in a trailer.
pub fn valid_actor(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Pull the global flags out of `args`. `--commit`, `--reason` and `--expect-hash` are always removed;
/// `--by` is removed unless the command is `claim` (where it already names the claimant) or `log` (where it
/// filters by actor); `--dry-run` is always left where commands that read it themselves look for it — after
/// the command; if it was given before the command it is moved to the end. A value-taking flag with no value
/// is recorded in [`Globals::error`].
pub fn extract(args: Vec<String>) -> (Vec<String>, Globals) {
    let mut g = Globals::default();
    let mut rest: Vec<String> = Vec::new();
    let mut seen_command = false;
    g.json = args.iter().any(|a| a == "--json");
    let mut it = args.into_iter();
    // `--by` keeps its own meaning for these commands, so the flag and its value stay in place.
    let keeps_by = |rest: &[String]| matches!(rest.first().map(String::as_str), Some("claim" | "log"));
    while let Some(a) = it.next() {
        // The value of `--flag value`, or of `--flag=value`; an error if there is none.
        let mut value = |flag: &str, inline: Option<&str>, g: &mut Globals| -> Option<String> {
            let v = inline.map(String::from).or_else(|| it.next().filter(|v| !v.starts_with("--")));
            if v.is_none() && g.error.is_none() {
                g.error = Some(format!("{flag} expects a value"));
            }
            v
        };
        match a.as_str() {
            "--commit" => g.commit = true,
            "--dry-run" => {
                g.dry_run = true;
                if seen_command {
                    rest.push(a);
                }
            }
            "--reason" => g.reason = value("--reason", None, &mut g),
            "--expect-hash" => g.expect_hash = value("--expect-hash", None, &mut g),
            "--by" => {
                let v = value("--by", None, &mut g);
                if keeps_by(&rest) {
                    rest.push(a);
                    if let Some(v) = &v {
                        rest.push(v.clone());
                    }
                }
                g.by = v;
            }
            s if s.starts_with("--reason=") => g.reason = value("--reason", Some(&s["--reason=".len()..]), &mut g),
            s if s.starts_with("--expect-hash=") => g.expect_hash = value("--expect-hash", Some(&s["--expect-hash=".len()..]), &mut g),
            s if s.starts_with("--by=") => {
                g.by = value("--by", Some(&s["--by=".len()..]), &mut g);
                if keeps_by(&rest) {
                    rest.push(a);
                }
            }
            _ => {
                if !seen_command && !a.starts_with('-') {
                    seen_command = true;
                }
                rest.push(a);
            }
        }
    }
    g.opaque_by = rest.first().map(String::as_str) == Some("claim");
    if g.dry_run && !rest.iter().any(|a| a == "--dry-run") {
        rest.push("--dry-run".to_string());
    }
    (rest, g)
}

/// The commands whose writes honour `--expect-hash`. Anywhere else the flag would be silently ignored — a
/// false assurance — so it is a usage error instead.
const EXPECT_HASH_COMMANDS: &[&str] = &["set", "claim", "release", "move", "applies-when", "scaffold-gherkin"];

/// A usage error if `--expect-hash` was given to a command that does not check it.
pub fn check_supported(command: Option<&str>) -> Result<(), String> {
    if get().expect_hash.is_some() && !command.is_some_and(|c| EXPECT_HASH_COMMANDS.contains(&c)) {
        return Err(format!(
            "--expect-hash is not supported by `{}`; it applies to {}",
            command.unwrap_or("(none)"),
            EXPECT_HASH_COMMANDS.join(", ")
        ));
    }
    Ok(())
}

/// Refuse (exit 1) if `--expect-hash` was given and the file at `path` no longer has that content —
/// the caller's view of it is stale.
pub fn check_expect_hash(path: &Path) {
    let Some(want) = &get().expect_hash else { return };
    let have = syscribe_model::fsio::file_hash(path).unwrap_or_else(|| "(unreadable)".to_string());
    if &have != want {
        eprintln!(
            "Refusing: {} has changed since it was read (expected {want}, found {have}). Re-read it and retry.",
            path.display()
        );
        std::process::exit(1);
    }
}

pub(crate) fn config_commit(model_root: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(model_root.join(".syscribe.toml")) else { return false };
    toml::from_str::<toml::Value>(&text)
        .ok()
        .and_then(|v| v.get("provenance")?.get("commit")?.as_bool())
        .unwrap_or(false)
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let o = std::process::Command::new("git").arg("-C").arg(root).args(args).output().map_err(|e| format!("cannot run git: {e}"))?;
    if !o.status.success() {
        let msg = String::from_utf8_lossy(&o.stderr);
        let out = String::from_utf8_lossy(&o.stdout);
        return Err(format!("git {}: {}{}", args.first().unwrap_or(&""), msg.trim(), if msg.trim().is_empty() { out.trim() } else { "" }));
    }
    Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// The commit message for a change: subject, optional reason, and the `Syscribe-*` trailers.
pub fn commit_message(argv: &[String], g: &Globals) -> (String, Option<String>, String) {
    let line = argv.join(" ");
    let mut subject = format!("syscribe: {line}");
    if subject.chars().count() > 72 {
        subject = format!("{}…", subject.chars().take(71).collect::<String>().trim_end());
    }
    let one_line = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut trailers = vec![format!("Syscribe-Command: {}", one_line(&line))];
    if let Some(by) = &g.by {
        trailers.push(format!("Syscribe-Actor: {by}"));
    }
    if let Some(r) = &g.reason {
        trailers.push(format!("Syscribe-Reason: {}", one_line(r)));
    }
    (subject, g.reason.clone().filter(|r| !r.trim().is_empty()), trailers.join("\n"))
}

/// Commit exactly `paths` (those inside a git repository and not ignored). Returns the short commit id, or
/// `None` when there was nothing to commit — a path rewritten with identical content is a successful no-op,
/// not a failed commit.
pub(crate) fn commit_paths(paths: &[PathBuf], argv: &[String], g: &Globals) -> Result<Option<String>, String> {
    let Some(first) = paths.iter().find_map(|p| p.parent().filter(|d| d.exists()).map(Path::to_path_buf)) else { return Ok(None) };
    let Some(root) = detect_git_root(&first) else { return Ok(None) };
    let mut rel: Vec<String> = Vec::new();
    for p in paths {
        let Ok(r) = crate::impactfiles::lenient_path(p).strip_prefix(&root).map(|r| r.to_string_lossy().replace('\\', "/")) else { continue };
        // An ignored path (a generated output) is not part of the model's history.
        if git(&root, &["check-ignore", "-q", "--", &r]).is_ok() {
            continue;
        }
        rel.push(r);
    }
    if rel.is_empty() {
        return Ok(None);
    }
    let mut add = vec!["add", "--"];
    add.extend(rel.iter().map(String::as_str));
    git(&root, &add)?;
    // Nothing differs from HEAD for these paths (identical content rewritten): nothing to commit.
    let mut quiet = vec!["diff", "--cached", "--quiet", "--"];
    quiet.extend(rel.iter().map(String::as_str));
    if git(&root, &quiet).is_ok() {
        return Ok(None);
    }
    let (subject, reason, trailers) = commit_message(argv, g);
    let mut args: Vec<String> = vec!["commit".into(), "-q".into(), "-m".into(), subject];
    if let Some(r) = reason {
        args.push("-m".into());
        args.push(r);
    }
    args.push("-m".into());
    args.push(trailers);
    if let Some(by) = &g.by {
        args.push(format!("--author={by} <{by}@syscribe.invalid>"));
    }
    args.push("--".into());
    args.extend(rel);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    git(&root, &refs)?;
    git(&root, &["rev-parse", "--short", "HEAD"]).map(Some)
}

/// Where the running command is, so any of its exits can finish it ([`exit`]).
static CONTEXT: OnceLock<(PathBuf, Vec<String>)> = OnceLock::new();
static FINISHED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Remember the model root and command line for [`exit`].
pub fn set_context(model_root: &Path, argv: &[String]) {
    let _ = CONTEXT.set((model_root.to_path_buf(), argv.to_vec()));
}

/// Exit with `code`, first finishing the command (commit, dry-run summary, JSON envelope) when it succeeded.
/// For writers that end the process themselves instead of returning to `main`.
pub fn exit(code: i32) -> ! {
    if code == 0 {
        if let Some((root, argv)) = CONTEXT.get() {
            finish(root, argv);
        }
    }
    std::process::exit(code)
}

/// Called once, at the end of a successful command. Under `--dry-run` it says how many files were left
/// untouched; when committing is on it commits what the command changed; and under `--json`, for a writer that
/// reported through [`say!`], it prints the result envelope. Exits 1 if the commit fails — the files stay
/// changed, and the message says so.
pub fn finish(model_root: &Path, argv: &[String]) {
    if FINISHED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let g = get();
    let mut commit: Option<String> = None;
    if g.dry_run {
        let n = syscribe_model::fsio::would_write().len();
        if n > 0 {
            eprintln!("dry run: {n} file(s) not written.");
        }
    } else if g.commit || config_commit(model_root) {
        let paths = syscribe_model::fsio::written();
        if !paths.is_empty() {
            match commit_paths(&paths, argv, g) {
                Ok(Some(sha)) => {
                    if g.json {
                        eprintln!("Committed {sha}.");
                    } else {
                        println!("Committed {sha}.");
                    }
                    commit = Some(sha);
                }
                Ok(None) => {}
                Err(m) => {
                    eprintln!("The change was written, but committing it failed: {m}");
                    std::process::exit(1);
                }
            }
        }
    }
    if g.json {
        emit_envelope(argv, commit);
    }
}

/// `{"schemaVersion","ok","dryRun","command","wrote","wouldWrite","commit","messages","diff"}`.
fn emit_envelope(argv: &[String], commit: Option<String>) {
    let Ok(c) = CAPTURE.lock() else { return };
    if !c.used {
        return;
    }
    let g = get();
    let show = |ps: Vec<PathBuf>| -> Vec<String> { ps.iter().map(|p| p.to_string_lossy().into_owned()).collect() };
    let doc = serde_json::json!({
        "schemaVersion": "1.0",
        "ok": true,
        "dryRun": g.dry_run,
        "command": argv.join(" "),
        "wrote": show(syscribe_model::fsio::written()),
        "wouldWrite": show(syscribe_model::fsio::would_write()),
        "commit": commit,
        "messages": c.messages,
        "diff": c.diff,
    });
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn globals_are_removed_and_recorded() {
        let (rest, g) = extract(s(&["set", "REQ-X", "status=approved", "--by", "agent-1", "--reason", "evidence TC-3 passed", "--commit", "--expect-hash", "blake3:ab"]));
        assert_eq!(rest, ["set", "REQ-X", "status=approved"]);
        assert_eq!((g.by.as_deref(), g.reason.as_deref(), g.commit, g.expect_hash.as_deref()), (Some("agent-1"), Some("evidence TC-3 passed"), true, Some("blake3:ab")));
    }

    #[test]
    fn log_keeps_by_because_it_filters_by_actor() {
        let (rest, _g) = extract(s(&["log", "REQ-X", "--by", "agent-1"]));
        assert_eq!(rest, ["log", "REQ-X", "--by", "agent-1"]);
    }

    #[test]
    fn claim_keeps_by_because_it_already_means_the_claimant() {
        let (rest, g) = extract(s(&["claim", "PI-1", "--by", "agent-1"]));
        assert_eq!(rest, ["claim", "PI-1", "--by", "agent-1"]);
        assert_eq!(g.by.as_deref(), Some("agent-1"));
        let (rest, _g) = extract(s(&["claim", "PI-1", "--by=agent-2"]));
        assert_eq!(rest, ["claim", "PI-1", "--by=agent-2"]);
    }

    #[test]
    fn dry_run_is_always_visible_after_the_command() {
        let (rest, g) = extract(s(&["set", "X", "status=done", "--dry-run"]));
        assert_eq!(rest, ["set", "X", "status=done", "--dry-run"]);
        assert!(g.dry_run);
        let (rest, g) = extract(s(&["--dry-run", "set", "X"]));
        assert_eq!(rest, ["set", "X", "--dry-run"], "commands look for the flag after their name");
        assert!(g.dry_run);
    }

    #[test]
    fn equals_forms_work() {
        let (rest, g) = extract(s(&["move", "A", "B", "--reason=tidy", "--by=bob"]));
        assert_eq!(rest, ["move", "A", "B"]);
        assert_eq!((g.reason.as_deref(), g.by.as_deref()), (Some("tidy"), Some("bob")));
    }

    #[test]
    fn actors_are_restricted_to_safe_characters() {
        assert!(valid_actor("agent-1") && valid_actor("a.b_c"));
        assert!(!valid_actor("") && !valid_actor("a b") && !valid_actor("a<b>") && !valid_actor(&"x".repeat(65)));
    }

    #[test]
    fn the_commit_message_carries_trailers_and_the_reason() {
        let g = Globals { by: Some("agent-1".into()), reason: Some("evidence\nTC-3 passed".into()), ..Default::default() };
        let (subject, reason, trailers) = commit_message(&s(&["set", "REQ-X", "status=approved"]), &g);
        assert_eq!(subject, "syscribe: set REQ-X status=approved");
        assert_eq!(reason.as_deref(), Some("evidence\nTC-3 passed"));
        assert_eq!(trailers, "Syscribe-Command: set REQ-X status=approved\nSyscribe-Actor: agent-1\nSyscribe-Reason: evidence TC-3 passed");
        let (_s, none, t) = commit_message(&s(&["claim", "PI-1"]), &Globals::default());
        assert!(none.is_none() && t == "Syscribe-Command: claim PI-1", "no actor trailer when none was given: {t}");
    }
}

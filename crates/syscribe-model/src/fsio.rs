//! The write layer for commands that change files (GH #444).
//!
//! Every user-facing write goes through here so one switch — the global `--dry-run` — can turn the
//! whole process read-only: a write is recorded as "would write" and nothing touches the disk. The
//! paths that really were written are remembered, so a command that opted in (`--commit`) can commit
//! exactly the files it changed. The MCP server's guarded-write engine, which plans in a candidate
//! copy, deliberately does not use this layer.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

type Guard = Box<dyn Fn(&Path) -> Result<(), String> + Send + Sync>;
static GUARD: std::sync::OnceLock<Guard> = std::sync::OnceLock::new();

/// Install a check consulted before every write or removal; an `Err` refuses it (an `io::Error` of kind
/// `PermissionDenied` carrying the message). The agent policy uses it so protected paths cannot be written by
/// any command, including ones that have no check of their own. Set once, by the CLI.
pub fn set_guard(g: impl Fn(&Path) -> Result<(), String> + Send + Sync + 'static) {
    let _ = GUARD.set(Box::new(g));
}

fn guarded(path: &Path) -> std::io::Result<()> {
    match GUARD.get() {
        Some(g) => g(path).map_err(|m| std::io::Error::new(std::io::ErrorKind::PermissionDenied, m)),
        None => Ok(()),
    }
}

type ContentGuard = Box<dyn Fn(&Path, &[u8]) -> Result<(), String> + Send + Sync>;
static CONTENT_GUARD: std::sync::OnceLock<ContentGuard> = std::sync::OnceLock::new();

/// Install a check that also sees what a write would put in the file (the old content is still on disk when
/// it runs). The agent policy uses it so no command can answer a question reserved to a person (GH #457).
/// Set once, by the CLI.
pub fn set_content_guard(g: impl Fn(&Path, &[u8]) -> Result<(), String> + Send + Sync + 'static) {
    let _ = CONTENT_GUARD.set(Box::new(g));
}

fn content_guarded(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    match CONTENT_GUARD.get() {
        Some(g) => g(path, contents).map_err(|m| std::io::Error::new(std::io::ErrorKind::PermissionDenied, m)),
        None => Ok(()),
    }
}

static DRY_RUN: AtomicBool = AtomicBool::new(false);
static RECORD: AtomicBool = AtomicBool::new(false);
static WRITTEN: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
static WOULD_WRITE: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

pub fn set_dry_run(on: bool) {
    DRY_RUN.store(on, Ordering::SeqCst);
}

pub fn is_dry_run() -> bool {
    DRY_RUN.load(Ordering::SeqCst)
}

/// Start remembering the paths written (or declined). The CLI turns this on so a command can commit exactly
/// what it changed; a long-lived server never does, so it accumulates nothing.
pub fn enable_recording() {
    RECORD.store(true, Ordering::SeqCst);
}

/// Remember `p` in `list` when recording is on; true if it was not there yet.
fn push(list: &Mutex<Vec<PathBuf>>, p: &Path) -> bool {
    if !RECORD.load(Ordering::SeqCst) {
        return true;
    }
    let Ok(mut l) = list.lock() else { return true };
    if l.iter().any(|x| x == p) {
        return false;
    }
    l.push(p.to_path_buf());
    true
}

/// Paths this process really wrote or removed, in order, without duplicates.
pub fn written() -> Vec<PathBuf> {
    WRITTEN.lock().map(|l| l.clone()).unwrap_or_default()
}

/// Record paths changed by code that does its own writing (the model crate's guarded `move`), so a command
/// that opted into `--commit` still commits exactly what it changed.
pub fn note_written(paths: impl IntoIterator<Item = PathBuf>) {
    if !is_dry_run() {
        for p in paths {
            push(&WRITTEN, &p);
        }
    }
}

/// Paths a dry run declined to write.
pub fn would_write() -> Vec<PathBuf> {
    WOULD_WRITE.lock().map(|l| l.clone()).unwrap_or_default()
}

fn declined(path: &Path, what: &str) {
    // Said once per path: a command that updates one file many times is not reported many times.
    if push(&WOULD_WRITE, path) {
        eprintln!("dry run: would {what} {}", path.display());
    }
}

/// `std::fs::write`, or a recorded no-op under `--dry-run`.
pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    let path = path.as_ref();
    // Refused even in a dry run: a dry run must report what a real run would be refused.
    guarded(path)?;
    content_guarded(path, contents.as_ref())?;
    if is_dry_run() {
        declined(path, "write");
        return Ok(());
    }
    std::fs::write(path, contents)?;
    push(&WRITTEN, path);
    Ok(())
}

/// Would a write to `path` be refused? The same check [`write`] makes, so a command that writes several files
/// can refuse before it has written any of them.
pub fn check(path: impl AsRef<Path>) -> std::io::Result<()> {
    guarded(path.as_ref())
}

/// [`write`], but never leaves a truncated file: the contents go to a file beside `path` and are renamed over it. The
/// guard and `--dry-run` apply to `path` itself, and it is `path` that is recorded as written.
pub fn write_atomic(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    let path = path.as_ref();
    guarded(path)?;
    content_guarded(path, contents.as_ref())?;
    if is_dry_run() {
        declined(path, "write");
        return Ok(());
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, contents)?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    push(&WRITTEN, path);
    Ok(())
}

/// `std::fs::create_dir_all`, or a no-op under `--dry-run`.
pub fn create_dir_all(path: impl AsRef<Path>) -> std::io::Result<()> {
    guarded(path.as_ref())?;
    if is_dry_run() {
        return Ok(());
    }
    std::fs::create_dir_all(path)
}

/// `std::fs::remove_file`, or a recorded no-op under `--dry-run`.
pub fn remove_file(path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref();
    guarded(path)?;
    if is_dry_run() {
        declined(path, "remove");
        return Ok(());
    }
    std::fs::remove_file(path)?;
    push(&WRITTEN, path);
    Ok(())
}

/// The word for a completed write in a status message: "Wrote", or "Would write" under `--dry-run`,
/// so a command never claims to have written what a dry run declined to write.
pub fn wrote() -> &'static str {
    if is_dry_run() { "Would write" } else { "Wrote" }
}

/// `done` normally, `would` under `--dry-run` ("Installed" / "Would install").
pub fn verb(done: &'static str, would: &'static str) -> &'static str {
    if is_dry_run() { would } else { done }
}

/// Under `--dry-run`, say once (on stderr) that the messages which follow describe what would happen — for
/// commands whose status text is too involved to reword.
pub fn dry_banner() {
    if is_dry_run() {
        eprintln!("dry run: nothing is written — the messages below describe what would happen.");
    }
}

/// A stable content hash of a file's bytes, as `--expect-hash` and `show` use it.
pub fn content_hash(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

/// The content hash of the file at `path`, or `None` if it cannot be read.
pub fn file_hash(path: &Path) -> Option<String> {
    std::fs::read(path).ok().map(|b| content_hash(&b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_and_prefixed() {
        assert_eq!(content_hash(b"abc"), content_hash(b"abc"));
        assert_ne!(content_hash(b"abc"), content_hash(b"abd"));
        assert!(content_hash(b"").starts_with("blake3:"));
    }
}

//! Subprocess spawn/timeout/kill mechanics for stdio plugins (`ADR-SYS-PLUGIN-002`).
//!
//! No new Cargo dependency: `std::process::Command` plus a hand-rolled
//! wait-with-timeout poll loop (`std::process::Child::wait` has no timeout
//! variant). Stdout/stderr are drained on background threads concurrently
//! with the poll loop, to avoid the classic pipe-buffer deadlock — a plugin
//! blocked writing to a full stdout pipe while nobody is reading it, while
//! the parent is blocked waiting for exit before it starts reading.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::config::PluginEntry;
use super::envelope::PluginRequest;

/// Poll interval while waiting for the child to exit or its timeout to elapse.
const POLL_INTERVAL: Duration = Duration::from_millis(25);

#[derive(Debug)]
pub enum PluginError {
    /// `command` could not be resolved — not found on `PATH`, or (given as a
    /// path) does not exist / is not executable.
    NotFound,
    /// The process failed to spawn at all.
    Spawn(String),
    /// Killed after exceeding `timeout_ms`.
    Timeout,
    /// The process exited with a non-zero (or signal-terminated) status.
    NonZeroExit { code: Option<i32>, stderr_tail: String },
    /// Some other I/O failure talking to the child.
    Io(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "command not found on PATH or not executable"),
            Self::Spawn(msg) => write!(f, "failed to start: {msg}"),
            Self::Timeout => write!(f, "killed after exceeding timeout_ms"),
            Self::NonZeroExit { code, stderr_tail } => write!(
                f,
                "exited with status {}: {}",
                code.map(|c| c.to_string()).unwrap_or_else(|| "signal".to_string()),
                stderr_tail
            ),
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

/// Resolve `command` to an executable path: a name containing a path
/// separator is resolved relative to `model_root` (absolute used verbatim);
/// otherwise it is searched for on `PATH`. Returns `None` when nothing
/// executable is found.
fn resolve_command(command: &str, model_root: &Path) -> Option<PathBuf> {
    if command.contains('/') || command.contains(std::path::MAIN_SEPARATOR) {
        let p = Path::new(command);
        let resolved = if p.is_absolute() { p.to_path_buf() } else { model_root.join(p) };
        return is_executable(&resolved).then_some(resolved);
    }
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join(command))
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p)
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(p: &Path) -> bool {
    p.is_file()
}

/// Truncate `s` to at most `max_bytes` bytes, keeping the tail and never
/// splitting a UTF-8 character.
fn tail(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut start = s.len() - max_bytes;
    while start < s.len() && !s.is_char_boundary(start) {
        start += 1;
    }
    s[start..].to_string()
}

/// Spawn the plugin, write `req` to its stdin, and return its raw stdout text
/// on a clean (exit-0) run. Callers that want the parsed envelope should feed
/// the result into [`super::envelope::convert`]; `--dry-run` prints it as-is.
pub fn invoke_raw(entry: &PluginEntry, req: &PluginRequest, model_root: &Path) -> Result<String, PluginError> {
    let cmd_path = resolve_command(&entry.command, model_root).ok_or(PluginError::NotFound)?;

    let mut child = Command::new(&cmd_path)
        .args(&entry.args)
        .current_dir(model_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| PluginError::Spawn(e.to_string()))?;

    let payload = serde_json::to_vec(req).map_err(|e| PluginError::Io(e.to_string()))?;
    if let Some(mut stdin) = child.stdin.take() {
        // The request is small (a handful of paths) — well under any OS pipe
        // buffer, so a blocking write here cannot deadlock against the child
        // not yet reading. Dropping `stdin` at the end of this block signals EOF.
        let _ = stdin.write_all(&payload);
    }

    // Drain stdout/stderr concurrently with the poll loop below so a chatty
    // plugin can't deadlock against a full pipe while we're only watching
    // `try_wait()`.
    let (out_tx, out_rx) = mpsc::channel();
    if let Some(mut out) = child.stdout.take() {
        std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = Vec::new();
            let _ = out.read_to_end(&mut buf);
            let _ = out_tx.send(buf);
        });
    }
    let (err_tx, err_rx) = mpsc::channel();
    if let Some(mut err) = child.stderr.take() {
        std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = Vec::new();
            let _ = err.read_to_end(&mut buf);
            let _ = err_tx.send(buf);
        });
    }

    let timeout = Duration::from_millis(entry.timeout_ms);
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(POLL_INTERVAL);
            }
            Err(e) => return Err(PluginError::Io(e.to_string())),
        }
    };

    // On a timeout, killing the direct child does not guarantee its own
    // children (if any) release the inherited stdout/stderr pipes — a reader
    // thread could then block indefinitely on an orphaned grandchild. Return
    // immediately without waiting on the reader channels; there is no output
    // to report for a killed run anyway. (Known limitation, same posture as
    // "no sandbox": a plugin that deliberately forks a surviving orphan can
    // outlive its own kill — out of scope here, same as every other
    // full-OS-access trust trade-off this design already accepts.)
    let Some(status) = status else {
        return Err(PluginError::Timeout);
    };

    // The reader threads finish shortly after a normal exit — give them a
    // generous but bounded window rather than blocking forever.
    let stdout_buf = out_rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default();
    let stderr_buf = err_rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default();

    if !status.success() {
        let stderr_tail = tail(&String::from_utf8_lossy(&stderr_buf), 2000);
        return Err(PluginError::NonZeroExit { code: status.code(), stderr_tail });
    }

    Ok(String::from_utf8_lossy(&stdout_buf).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(command: &str, args: &[&str], timeout_ms: u64) -> PluginEntry {
        PluginEntry {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            timeout_ms,
        }
    }

    fn req() -> PluginRequest {
        PluginRequest {
            protocol_version: 1,
            alias: "toydsl".to_string(),
            package_qname: "Legacy::ToyDsl".to_string(),
            package_dir: "/tmp/does-not-matter".to_string(),
            model_root: "/tmp/does-not-matter".to_string(),
        }
    }

    #[test]
    fn well_formed_run_round_trips() {
        let e = entry("/bin/sh", &["-c", "cat >/dev/null; echo '{\"elements\":[]}'"], 5000);
        let out = invoke_raw(&e, &req(), Path::new(".")).unwrap();
        assert!(out.contains("\"elements\""));
    }

    #[test]
    fn nonexistent_command_is_not_found() {
        let e = entry("this-binary-does-not-exist-anywhere", &[], 5000);
        assert!(matches!(invoke_raw(&e, &req(), Path::new(".")), Err(PluginError::NotFound)));
    }

    #[test]
    fn nonzero_exit_is_reported_with_stderr_tail() {
        let e = entry("/bin/sh", &["-c", "cat >/dev/null; echo boom 1>&2; exit 3"], 5000);
        match invoke_raw(&e, &req(), Path::new(".")) {
            Err(PluginError::NonZeroExit { code, stderr_tail }) => {
                assert_eq!(code, Some(3));
                assert!(stderr_tail.contains("boom"));
            }
            other => panic!("expected NonZeroExit, got {other:?}"),
        }
    }

    #[test]
    fn timeout_kills_the_child_rather_than_waiting_it_out() {
        let e = entry("/bin/sh", &["-c", "cat >/dev/null; sleep 5; echo late"], 200);
        let start = Instant::now();
        let result = invoke_raw(&e, &req(), Path::new("."));
        let elapsed = start.elapsed();
        assert!(matches!(result, Err(PluginError::Timeout)));
        assert!(elapsed < Duration::from_secs(2), "expected a fast kill, took {elapsed:?}");
    }

    #[test]
    fn malformed_stdout_is_still_returned_raw_for_the_caller_to_reject() {
        let e = entry("/bin/sh", &["-c", "cat >/dev/null; echo 'not json'"], 5000);
        let out = invoke_raw(&e, &req(), Path::new(".")).unwrap();
        assert_eq!(out.trim(), "not json");
    }
}

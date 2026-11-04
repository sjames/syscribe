//! Materialise the model as it stood at a git revision (GH #431, #433).
//!
//! The model directory at `<rev>` is exported with `git archive` into a private
//! temporary directory, which the ordinary walker and validator can then load
//! unchanged. Nothing in the working tree or the index is touched. The export is
//! removed when the [`RevModel`] is dropped.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// A model exported from a git revision into a temporary directory.
pub struct RevModel {
    /// The model root inside the export; pass it to `walk_model`.
    pub root: PathBuf,
    tmp: PathBuf,
}

impl Drop for RevModel {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.tmp);
    }
}

/// Export the model rooted at `model_root` as of `rev`.
///
/// Fails with a readable message when `model_root` is not inside a git
/// repository, `rev` does not resolve, or the model directory did not exist at
/// `rev` (a model introduced after `rev` has no "before").
pub fn materialize(model_root: &Path, rev: &str) -> Result<RevModel, String> {
    if rev.starts_with('-') {
        return Err(format!("`{rev}` is not a git revision"));
    }
    let abs = std::fs::canonicalize(model_root).map_err(|e| format!("{}: {e}", model_root.display()))?;
    let git_root = crate::config::detect_git_root(&abs)
        .ok_or_else(|| format!("{} is not inside a git repository", model_root.display()))?;
    let rel = abs.strip_prefix(&git_root).unwrap_or(Path::new("")).to_string_lossy().replace('\\', "/");

    let commit = crate::config::git_output(&git_root, &["rev-parse", "--verify", "--quiet", &format!("{rev}^{{commit}}")])
        .ok_or_else(|| format!("`{rev}` does not name a commit in this repository"))?;
    let tree_ish = if rel.is_empty() { commit.clone() } else { format!("{commit}:{rel}") };
    if crate::config::git_output(&git_root, &["cat-file", "-t", &tree_ish]).as_deref() != Some("tree") {
        return Err(format!("the model directory `{rel}` does not exist at `{rev}`"));
    }

    static N: AtomicU64 = AtomicU64::new(0);
    let tmp = std::env::temp_dir().join(format!("syscribe-rev-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&tmp).map_err(|e| format!("cannot create {}: {e}", tmp.display()))?;
    // Constructed now so a failure below still cleans up.
    let out = RevModel { root: tmp.clone(), tmp: tmp.clone() };

    let mut archive = Command::new("git")
        .arg("-C")
        .arg(&git_root)
        .args(["archive", "--format=tar", &tree_ish])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run git: {e}"))?;
    let extract = Command::new("tar")
        .arg("-x")
        .arg("-C")
        .arg(&tmp)
        .stdin(archive.stdout.take().ok_or("git archive produced no output")?)
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("cannot run tar: {e}"))?;
    let status = archive.wait_with_output().map_err(|e| format!("git archive: {e}"))?;
    if !status.status.success() {
        return Err(format!("git archive failed: {}", String::from_utf8_lossy(&status.stderr).trim()));
    }
    if !extract.status.success() {
        return Err(format!("tar failed: {}", String::from_utf8_lossy(&extract.stderr).trim()));
    }
    Ok(out)
}

/// Export the model directory as it is in the git *index* (what a commit would record), into a
/// temporary directory. Used by `check-staged` when the working tree differs from the index, so the
/// check judges the content that would actually be committed.
pub fn materialize_index(model_root: &Path) -> Result<RevModel, String> {
    let abs = std::fs::canonicalize(model_root).map_err(|e| format!("{}: {e}", model_root.display()))?;
    let git_root = crate::config::detect_git_root(&abs)
        .ok_or_else(|| format!("{} is not inside a git repository", model_root.display()))?;
    let rel = abs.strip_prefix(&git_root).unwrap_or(Path::new("")).to_string_lossy().replace('\\', "/");

    let mut ls = Command::new("git");
    ls.arg("-C").arg(&git_root).args(["ls-files", "-z"]);
    if !rel.is_empty() {
        ls.arg("--").arg(&rel);
    }
    let listed = ls.output().map_err(|e| format!("cannot run git: {e}"))?;
    if !listed.status.success() {
        return Err(format!("git ls-files failed: {}", String::from_utf8_lossy(&listed.stderr).trim()));
    }

    static N: AtomicU64 = AtomicU64::new(0);
    let tmp = std::env::temp_dir().join(format!("syscribe-index-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&tmp).map_err(|e| format!("cannot create {}: {e}", tmp.display()))?;
    let root = if rel.is_empty() { tmp.clone() } else { tmp.join(&rel) };
    let out = RevModel { root, tmp: tmp.clone() };

    let mut prefix = tmp.to_string_lossy().into_owned();
    if !prefix.ends_with('/') {
        prefix.push('/');
    }
    let mut co = Command::new("git")
        .arg("-C")
        .arg(&git_root)
        .args(["checkout-index", "-z", "--stdin", "-f"])
        .arg(format!("--prefix={prefix}"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run git: {e}"))?;
    {
        use std::io::Write;
        let mut stdin = co.stdin.take().ok_or("git checkout-index has no stdin")?;
        stdin.write_all(&listed.stdout).map_err(|e| format!("git checkout-index: {e}"))?;
    }
    let done = co.wait_with_output().map_err(|e| format!("git checkout-index: {e}"))?;
    if !done.status.success() {
        return Err(format!("git checkout-index failed: {}", String::from_utf8_lossy(&done.stderr).trim()));
    }
    Ok(out)
}

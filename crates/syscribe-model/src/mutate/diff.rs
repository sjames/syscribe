//! A small line-based unified-diff helper (no external crate). Used to preview
//! what a write tool would change. Diffs are computed by comparing the real model
//! tree against the candidate (post-edit) temp copy, so the same code serves
//! single-file edits, multi-file moves, deletes, and batch `apply_changes`.

use std::collections::BTreeSet;
use std::path::Path;

/// One element of an LCS edit script over lines.
enum Edit<'a> {
    Eq(&'a str),
    Del(&'a str),
    Ins(&'a str),
}

/// Longest-common-subsequence edit script between two line slices.
fn lcs_edits<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<Edit<'a>> {
    let (n, m) = (a.len(), b.len());
    // dp[i][j] = LCS length of a[i..], b[j..].
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if a[i] == b[j] {
            out.push(Edit::Eq(a[i]));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            out.push(Edit::Del(a[i]));
            i += 1;
        } else {
            out.push(Edit::Ins(b[j]));
            j += 1;
        }
    }
    while i < n {
        out.push(Edit::Del(a[i]));
        i += 1;
    }
    while j < m {
        out.push(Edit::Ins(b[j]));
        j += 1;
    }
    out
}

/// Unified diff for a single file path. `old`/`new` are the file contents (or
/// `None` when the file is absent — creation has `old=None`, deletion `new=None`).
/// Returns `""` when nothing changed.
pub fn file_unified_diff(path: &str, old: Option<&str>, new: Option<&str>) -> String {
    if old == new {
        return String::new();
    }
    let old_lines: Vec<&str> = old.map(|s| s.lines().collect()).unwrap_or_default();
    let new_lines: Vec<&str> = new.map(|s| s.lines().collect()).unwrap_or_default();
    if old_lines == new_lines {
        return String::new();
    }

    let old_label = if old.is_some() {
        format!("a/{path}")
    } else {
        "/dev/null".to_string()
    };
    let new_label = if new.is_some() {
        format!("b/{path}")
    } else {
        "/dev/null".to_string()
    };
    let old_start = if old_lines.is_empty() { 0 } else { 1 };
    let new_start = if new_lines.is_empty() { 0 } else { 1 };

    let mut out = String::new();
    out.push_str(&format!("--- {old_label}\n+++ {new_label}\n"));
    out.push_str(&format!(
        "@@ -{},{} +{},{} @@\n",
        old_start,
        old_lines.len(),
        new_start,
        new_lines.len()
    ));
    for e in lcs_edits(&old_lines, &new_lines) {
        match e {
            Edit::Eq(s) => {
                out.push(' ');
                out.push_str(s);
                out.push('\n');
            }
            Edit::Del(s) => {
                out.push('-');
                out.push_str(s);
                out.push('\n');
            }
            Edit::Ins(s) => {
                out.push('+');
                out.push_str(s);
                out.push('\n');
            }
        }
    }
    out
}

/// Collect model-root-relative paths of all regular files under `root`.
fn collect_rel(root: &Path, out: &mut BTreeSet<String>) {
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Ok(rel) = entry.path().strip_prefix(root) {
            out.insert(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

/// Concatenated unified diff of every file that differs between two model trees
/// (`old_root` = current, `new_root` = candidate). Per-file hunks carry standard
/// `--- `/`+++ ` headers. Returns `""` when the trees are identical.
pub fn tree_unified_diff(old_root: &Path, new_root: &Path) -> String {
    let mut rels: BTreeSet<String> = BTreeSet::new();
    collect_rel(old_root, &mut rels);
    collect_rel(new_root, &mut rels);

    let mut out = String::new();
    for rel in rels {
        let old = std::fs::read_to_string(old_root.join(&rel)).ok();
        let new = std::fs::read_to_string(new_root.join(&rel)).ok();
        if old == new {
            continue;
        }
        out.push_str(&file_unified_diff(&rel, old.as_deref(), new.as_deref()));
    }
    out
}

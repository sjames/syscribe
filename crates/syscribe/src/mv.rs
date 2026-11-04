//! `syscribe move <source> <dest>` — CLI entry point over the shared
//! `syscribe_model::mutate` move engine (relocates an element or package and
//! rewrites every qualified-name reference to it, atomically).
//!
//! The planning/rewriting/rollback logic itself lives in
//! `syscribe_model::mutate::mv` so the MCP server's `move_element` tool and any
//! future `syscribe-server` caller can share it; this module is now just the CLI
//! wrapper (argument plumbing + human-readable stdout/stderr reporting).

use std::path::Path;

use syscribe_model::element::RawElement;
use syscribe_model::mutate::MoveReport;
use syscribe_model::resolver::Resolver;

/// True when `q` is a syntactically valid qualified name (`Seg(::Seg)*`).
pub use syscribe_model::mutate::valid_move_dest;

/// Plan and (unless `dry_run`) apply a move of `source_key` to `dest`, rewriting
/// every qualified-name reference. Thin wrapper over
/// `syscribe_model::mutate::move_element` preserving the historical
/// `Result<_, String>` shape for existing call sites.
pub fn move_element(
    model_root: &Path,
    elements: &[RawElement],
    resolver: &Resolver,
    source_key: &str,
    dest: &str,
    dry_run: bool,
) -> Result<MoveReport, String> {
    syscribe_model::mutate::move_element(model_root, elements, resolver, source_key, dest, dry_run)
        .map_err(|e| e.to_string())
}

/// `move` subcommand entry point.
pub fn cmd_move(
    model_root: &Path,
    elements: &[RawElement],
    resolver: &Resolver,
    source_key: &str,
    dest: &str,
    dry_run: bool,
) {
    if let Some(src) = resolver.resolve_ref(elements, source_key) {
        crate::provenance::check_expect_hash(std::path::Path::new(&src.file_path));
    }
    // A move rewrites every file that refers to the element; an agent policy has to see all of them before any
    // is touched, so plan it first.
    if crate::policy_gate::agent_active() {
        if let Ok(plan) = move_element(model_root, elements, resolver, source_key, dest, true) {
            let mut files = vec![plan.from_path.clone(), plan.to_path.clone()];
            files.extend(plan.rewritten_files.iter().cloned());
            crate::policy_gate::check_paths(&files);
        }
    }
    let report = match move_element(model_root, elements, resolver, source_key, dest, dry_run) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    if !dry_run {
        // The model crate did the writing; tell the write layer which files changed.
        syscribe_model::fsio::note_written(
            [report.from_path.clone(), report.to_path.clone()].into_iter().chain(report.rewritten_files.iter().cloned()),
        );
    }
    let kind = if report.is_package { "package" } else { "element" };
    if dry_run {
        crate::say!("[dry-run] move {kind} {} -> {}", report.from, report.to);
        crate::say!(
            "[dry-run]   relocate {} -> {}",
            report.from_path.display(),
            report.to_path.display()
        );
        if report.rewritten_files.is_empty() {
            crate::say!("[dry-run]   no reference updates needed");
        } else {
            for p in &report.rewritten_files {
                crate::say!("[dry-run]   update references in {}", p.display());
            }
        }
        return;
    }

    crate::say!("Moved {kind} {} -> {}", report.from, report.to);
    crate::say!("  {} -> {}", report.from_path.display(), report.to_path.display());
    crate::say!("  updated references in {} file(s)", report.rewritten_files.len());
}

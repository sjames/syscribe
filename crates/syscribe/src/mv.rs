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
pub use syscribe_model::mutate::valid_qname;

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
    let report = match move_element(model_root, elements, resolver, source_key, dest, dry_run) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let kind = if report.is_package { "package" } else { "element" };
    if dry_run {
        println!("[dry-run] move {kind} {} -> {}", report.from, report.to);
        println!(
            "[dry-run]   relocate {} -> {}",
            report.from_path.display(),
            report.to_path.display()
        );
        if report.rewritten_files.is_empty() {
            println!("[dry-run]   no reference updates needed");
        } else {
            for p in &report.rewritten_files {
                println!("[dry-run]   update references in {}", p.display());
            }
        }
        return;
    }

    println!("Moved {kind} {} -> {}", report.from, report.to);
    println!("  {} -> {}", report.from_path.display(), report.to_path.display());
    println!("  updated references in {} file(s)", report.rewritten_files.len());
}

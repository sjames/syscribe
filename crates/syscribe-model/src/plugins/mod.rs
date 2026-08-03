//! WASM plugin host for foreign-format ingestion (ADR-SYS-PLUGIN-001,
//! REQ-TRS-PLUGIN-*).
//!
//! A package `_index.md` declaring `foreignFormat: <alias>` hands its subtree
//! to the plugin named by `[plugins.<alias>]` in `.syscribe.toml`. The plugin
//! only *parses* — read-only ingestion, decided alongside this feature: the
//! foreign folder stays authoritative and is edited by its own native tooling,
//! never by Syscribe's mutate/diagram-editor write paths.
//!
//! Runs inside [`crate::walker::walk_model`] itself (not bolted onto each of
//! its ~20 callers across the CLI/MCP/LSP/web server) so every consumer picks
//! up foreign elements for free, with no per-call-site wiring risk.

pub mod config;
pub mod envelope;
#[cfg(feature = "wasm-plugins")]
mod runtime;

use std::path::Path;

use crate::element::RawElement;
use envelope::ConvertOutcome;

/// Run every configured foreign-format plugin over `elements`, replacing each
/// foreign package's native placeholder content with the plugin's output.
///
/// No-op when `[plugins]` is not configured (REQ-TRS-PLUGIN-000 — inert by
/// default, so a model with no foreign packages is completely unaffected).
pub fn apply_foreign_plugins(elements: &mut Vec<RawElement>, model_root: &Path) {
    let plugins = config::load_plugins(model_root);

    // Only an `_index.md` (a package) can declare `foreignFormat:` — REQ-TRS-PLUGIN-001.
    // A `foreignFormat:` on any other file is ignored here (not a package, so it
    // has no subtree to own); it's still visible in that element's own
    // frontmatter, so a model author who put it in the wrong place isn't left
    // with zero signal, just no special handling.
    let foreign_packages: Vec<(usize, String, String)> = elements
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            let alias = e.frontmatter.foreign_format.as_ref()?;
            if !e.file_path.ends_with("_index.md") {
                return None;
            }
            Some((i, e.qualified_name.clone(), alias.clone()))
        })
        .collect();

    if foreign_packages.is_empty() {
        return;
    }

    let mut synthetic: Vec<RawElement> = Vec::new();
    let mut owned_dirs: Vec<(String, String)> = Vec::new(); // (dir, package's own _index.md path)

    for (idx, pkg_qname, alias) in foreign_packages {
        let pkg_file_path = elements[idx].file_path.clone();
        let pkg_dir = Path::new(&pkg_file_path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();

        let Some(entry) = plugins.get(&alias) else {
            push_finding(
                &mut elements[idx],
                "E532",
                &format!(
                    "foreignFormat '{alias}' has no matching [plugins.{alias}] entry in .syscribe.toml"
                ),
            );
            continue;
        };

        let wasm_path = entry.wasm_path(model_root);
        if !wasm_path.exists() {
            push_finding(
                &mut elements[idx],
                "E530",
                &format!(
                    "[plugins.{alias}].wasm '{}' does not exist on disk",
                    wasm_path.display()
                ),
            );
            continue;
        }

        owned_dirs.push((pkg_dir.display().to_string(), pkg_file_path.clone()));

        let raw = match invoke_plugin(&wasm_path, entry, &pkg_dir, model_root, &alias) {
            Ok(raw) => raw,
            Err(msg) => {
                push_finding(&mut elements[idx], "W530", &msg);
                continue;
            }
        };
        let envelope: envelope::ElementsEnvelope = match serde_json::from_str(&raw) {
            Ok(env) => env,
            Err(err) => {
                push_finding(
                    &mut elements[idx],
                    "W532",
                    &format!("plugin '{alias}' returned malformed envelope JSON: {err}"),
                );
                continue;
            }
        };

        if !envelope.diagnostics.is_empty() {
            let n = envelope.diagnostics.len();
            let preview: Vec<String> = envelope
                .diagnostics
                .iter()
                .take(3)
                .map(|d| format!("{}: {}", d.severity, d.message))
                .collect();
            push_finding(
                &mut elements[idx],
                "W532",
                &format!(
                    "plugin '{alias}' reported {n} diagnostic(s): {}",
                    preview.join("; ")
                ),
            );
        }

        for env_elem in envelope.elements {
            let raw_qname = env_elem.qname.clone();
            match envelope::convert_element(&pkg_qname, &pkg_file_path, env_elem) {
                ConvertOutcome::Ok(elem) => synthetic.push(elem),
                ConvertOutcome::BadFrontmatter(msg) => push_finding(
                    &mut elements[idx],
                    "W533",
                    &format!("element '{raw_qname}' from plugin '{alias}' dropped: {msg}"),
                ),
                ConvertOutcome::UnknownType(t) => push_finding(
                    &mut elements[idx],
                    "W534",
                    &format!(
                        "element '{raw_qname}' from plugin '{alias}' has unrecognised type '{t}', dropped"
                    ),
                ),
            }
        }
    }

    // Strip native elements under each foreign package's directory — the whole
    // subtree is plugin-owned — except the package's own `_index.md` anchor.
    if !owned_dirs.is_empty() {
        elements.retain(|e| {
            if owned_dirs.iter().any(|(_, anchor)| anchor == &e.file_path) {
                return true;
            }
            !owned_dirs
                .iter()
                .any(|(dir, _)| under_dir(&e.file_path, dir))
        });
    }

    elements.extend(synthetic);
}

fn under_dir(file_path: &str, dir: &str) -> bool {
    if dir.is_empty() {
        return false;
    }
    Path::new(file_path).starts_with(Path::new(dir))
}

fn push_finding(elem: &mut RawElement, code: &str, message: &str) {
    elem.derive_findings
        .push((code.to_string(), elem.file_path.clone(), message.to_string()));
}

#[cfg(feature = "wasm-plugins")]
fn invoke_plugin(
    wasm_path: &Path,
    entry: &config::PluginEntry,
    pkg_dir: &Path,
    model_root: &Path,
    alias: &str,
) -> Result<String, String> {
    runtime::run(wasm_path, entry, pkg_dir, model_root, alias, true)
        .map_err(|e| format!("plugin '{alias}': {e}"))
}

/// `syscribe plugins run <alias> --dry-run` — invoke one configured plugin and
/// return its raw envelope JSON without merging it into any graph. `elements`
/// only needs to contain the `_index.md` declaring `foreignFormat: <alias>`
/// (a full `walk_model` result works fine too).
pub fn dry_run(alias: &str, model_root: &Path, elements: &[RawElement]) -> Result<String, String> {
    let plugins = config::load_plugins(model_root);
    let entry = plugins
        .get(alias)
        .ok_or_else(|| format!("no [plugins.{alias}] entry in .syscribe.toml"))?;

    let wasm_path = entry.wasm_path(model_root);
    if !wasm_path.exists() {
        return Err(format!(
            "[plugins.{alias}].wasm '{}' does not exist on disk",
            wasm_path.display()
        ));
    }

    let pkg = elements
        .iter()
        .find(|e| e.frontmatter.foreign_format.as_deref() == Some(alias))
        .ok_or_else(|| format!("no package in the model declares foreignFormat: {alias}"))?;
    let pkg_dir = Path::new(&pkg.file_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();

    dry_run_invoke(&wasm_path, entry, &pkg_dir, model_root, alias)
}

/// `use_cache: false` — a plugin author running `--dry-run` always gets a
/// guaranteed-live invocation, never a cached result (and never populates
/// the cache either).
#[cfg(feature = "wasm-plugins")]
fn dry_run_invoke(
    wasm_path: &Path,
    entry: &config::PluginEntry,
    pkg_dir: &Path,
    model_root: &Path,
    alias: &str,
) -> Result<String, String> {
    runtime::run(wasm_path, entry, pkg_dir, model_root, alias, false)
}

#[cfg(not(feature = "wasm-plugins"))]
fn dry_run_invoke(
    _wasm_path: &Path,
    _entry: &config::PluginEntry,
    _pkg_dir: &Path,
    _model_root: &Path,
    _alias: &str,
) -> Result<String, String> {
    Err("this build of syscribe was compiled without the `wasm-plugins` feature".to_string())
}

#[cfg(not(feature = "wasm-plugins"))]
fn invoke_plugin(
    _wasm_path: &Path,
    _entry: &config::PluginEntry,
    _pkg_dir: &Path,
    _model_root: &Path,
    alias: &str,
) -> Result<String, String> {
    Err(format!(
        "plugin '{alias}' is configured but this build of syscribe-model was compiled without the `wasm-plugins` feature"
    ))
}

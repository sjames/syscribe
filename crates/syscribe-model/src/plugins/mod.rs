//! stdio-subprocess plugin host for foreign-format ingestion (`ADR-SYS-PLUGIN-002`).
//!
//! A package `_index.md` declaring `foreignFormat: <alias>` hands its subtree
//! to the plugin process named by `[plugins.<alias>]` in `.syscribe.toml`. The
//! plugin only *parses* — read-only ingestion: the foreign folder stays
//! authoritative and is edited by its own native tooling, never by Syscribe's
//! mutate/diagram-editor write paths.
//!
//! Runs inside [`crate::walker::walk_model`] itself (not bolted onto each of
//! its many callers across the CLI/MCP/LSP/web server) so every consumer picks
//! up foreign elements for free, with no per-call-site wiring risk.
//!
//! A model with no `[plugins]` configured, or no `foreignFormat:` package, is
//! completely unaffected.

pub mod config;
pub mod envelope;
pub mod runtime;

use std::path::{Path, PathBuf};

use crate::derive::finding;
use crate::element::RawElement;
use envelope::PluginRequest;

/// A resolved `foreignFormat:`-marked package, ready to be handed to a plugin.
pub struct PluginPackage {
    pub alias: String,
    pub dir: PathBuf,
    pub qname: String,
    /// The owning `_index.md`'s path — every synthesized element's `file_path`.
    pub index_path: String,
}

/// Run every configured foreign-format plugin over `elements`, replacing each
/// foreign package's native placeholder content with the plugin's output.
///
/// No-op when `[plugins]` is not configured and no package declares
/// `foreignFormat:` (inert by default).
pub fn apply_foreign_plugins(elements: &mut Vec<RawElement>, model_root: &Path) {
    // Only an `_index.md` (a package) can declare `foreignFormat:` — a marker
    // anywhere else is silently ignored here (it is still visible in that
    // element's own frontmatter, so a model author who put it in the wrong
    // place isn't left with zero signal, just no special handling — same
    // posture `sysmlSubmodel:` already takes).
    //
    // Anchors are confirmed shallowest-first (mirrors
    // `sysmlv2::apply_sysmlv2_submodels`): a `foreignFormat:`-marked package
    // nested inside an already-claimed subtree does not get to start its own
    // subtree — it's absorbed into the outer package like everything else in
    // there, rather than escaping exclusion by claiming itself first.
    let mut candidates: Vec<(usize, PathBuf, String, Option<String>)> = elements
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            if !e.file_path.ends_with("_index.md") {
                return None;
            }
            let dir = Path::new(&e.file_path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
            Some((i, dir, e.file_path.clone(), e.frontmatter.foreign_format.clone()))
        })
        .collect();
    candidates.sort_by_key(|(_, dir, _, _)| dir.components().count());

    let mut confirmed: Vec<(usize, PluginPackage)> = Vec::new();
    for (idx, dir, file_path, alias) in candidates {
        if confirmed.iter().any(|(_, p)| under_dir(&file_path, &p.dir)) {
            continue; // inside an already-claimed subtree: absorbed, not a new anchor
        }
        if let Some(alias) = alias {
            confirmed.push((
                idx,
                PluginPackage {
                    alias,
                    dir,
                    qname: elements[idx].qualified_name.clone(),
                    index_path: file_path,
                },
            ));
        }
    }

    if confirmed.is_empty() {
        return;
    }

    let plugins = config::load_plugins(model_root);
    let mut synthetic: Vec<RawElement> = Vec::new();
    let mut owned_dirs: Vec<(PathBuf, String)> = Vec::new(); // (dir, anchor file_path)

    for (owner_idx, pkg) in &confirmed {
        let Some(entry) = plugins.get(&pkg.alias) else {
            elements[*owner_idx].derive_findings.push(finding(
                "E551",
                &pkg.index_path,
                &format!(
                    "foreignFormat '{}' has no matching [plugins.{}] entry in .syscribe.toml",
                    pkg.alias, pkg.alias
                ),
            ));
            continue;
        };

        owned_dirs.push((pkg.dir.clone(), pkg.index_path.clone()));

        let req = build_request(pkg, model_root);

        match runtime::invoke_raw(entry, &req, model_root) {
            Ok(raw_json) => match envelope::convert(&pkg.qname, &pkg.index_path, &raw_json) {
                Ok((elems, findings)) => {
                    synthetic.extend(elems);
                    for (code, msg) in findings {
                        elements[*owner_idx].derive_findings.push(finding(code, &pkg.index_path, &msg));
                    }
                }
                Err(msg) => {
                    elements[*owner_idx]
                        .derive_findings
                        .push(finding("W551", &pkg.index_path, &msg));
                }
            },
            Err(e) => {
                let code = if matches!(e, runtime::PluginError::NotFound) { "E550" } else { "W550" };
                elements[*owner_idx].derive_findings.push(finding(
                    code,
                    &pkg.index_path,
                    &format!("plugin '{}': {e}", pkg.alias),
                ));
            }
        }
    }

    // Strip native elements under each foreign package's directory — the
    // whole subtree is plugin-owned — except the package's own `_index.md` anchor.
    if !owned_dirs.is_empty() {
        elements.retain(|e| {
            if owned_dirs.iter().any(|(_, anchor)| anchor == &e.file_path) {
                return true;
            }
            !owned_dirs.iter().any(|(dir, _)| under_dir(&e.file_path, dir))
        });
    }

    elements.extend(synthetic);
}

/// Build the stdin request for `pkg`, with `packageDir`/`modelRoot` resolved
/// to absolute paths — a plugin's own working directory is not guaranteed to
/// match the caller's, so a relative path in the request would be ambiguous
/// (and silently wrong if the plugin resolves it against its own cwd rather
/// than the caller's).
pub fn build_request(pkg: &PluginPackage, model_root: &Path) -> PluginRequest {
    PluginRequest {
        protocol_version: 1,
        alias: pkg.alias.clone(),
        package_qname: pkg.qname.clone(),
        package_dir: absolute(&pkg.dir).display().to_string(),
        model_root: absolute(model_root).display().to_string(),
    }
}

/// Best-effort absolute form of `p`: canonicalized when it exists on disk
/// (resolving `..`/symlinks), otherwise joined onto the process's current
/// directory as a fallback rather than sent to the plugin unresolved.
fn absolute(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| {
        std::env::current_dir()
            .map(|cwd| cwd.join(p))
            .unwrap_or_else(|_| p.to_path_buf())
    })
}

/// True if `file_path` lies inside `dir` (component-wise, not a string prefix).
fn under_dir(file_path: &str, dir: &Path) -> bool {
    if dir.as_os_str().is_empty() {
        return false;
    }
    Path::new(file_path).starts_with(dir)
}

/// The qualified names of every `RawElement` synthesized by a stdio plugin
/// (`ADR-SYS-PLUGIN-002`), for validator policy decisions that must be gated
/// on actual plugin origin rather than element kind alone
/// (`Resolver::is_verify_target`'s `E104` widening) — mirrors
/// `crate::sysmlv2::synthesized_qnames`'s role and rationale for the SysMLv2
/// submodel, but derives the set differently: a SysMLv2-synthesized element's
/// `file_path` is itself the distinguishing `.sysml`/`.kerml` source file, but
/// a plugin-synthesized element's `file_path` is its owning package's
/// `_index.md` — indistinguishable by `file_path` alone from that anchor
/// itself. Instead: after [`apply_foreign_plugins`] runs, the *only* elements
/// whose qname is nested under a `foreignFormat:`-declaring package's own
/// qname (other than that package itself) are guaranteed plugin-synthesized —
/// every native element that used to live there was stripped. `RawElement`
/// still carries no origin field; this derives the set fresh from `elements`,
/// the same "no second return value threaded through `walk_model`'s many
/// callers" tradeoff SysMLv2's version documents.
pub fn synthesized_qnames(elements: &[RawElement]) -> std::collections::HashSet<String> {
    let foreign_pkg_qnames: Vec<&str> = elements
        .iter()
        .filter(|e| e.frontmatter.foreign_format.is_some())
        .map(|e| e.qualified_name.as_str())
        .collect();
    if foreign_pkg_qnames.is_empty() {
        return std::collections::HashSet::new();
    }
    elements
        .iter()
        .filter(|e| {
            e.frontmatter.foreign_format.is_none()
                && foreign_pkg_qnames
                    .iter()
                    .any(|pkg| e.qualified_name.starts_with(pkg) && e.qualified_name[pkg.len()..].starts_with("::"))
        })
        .map(|e| e.qualified_name.clone())
        .collect()
}

/// Find the package whose `_index.md` declares `foreignFormat: <alias>`,
/// scanning `_index.md` files directly rather than running the full
/// [`crate::walker::walk_model`] pipeline (which would invoke
/// [`apply_foreign_plugins`] itself). Used standalone by
/// `syscribe plugins run <alias> --dry-run`.
pub fn find_alias_package(model_root: &Path, alias: &str) -> Option<PluginPackage> {
    for entry in walkdir::WalkDir::new(model_root).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() || entry.file_name() != "_index.md" {
            continue;
        }
        let path = entry.path();
        let Ok(content) = std::fs::read_to_string(path) else { continue };
        let (fm_str, _body) = crate::frontmatter::split_frontmatter(&content);
        let Some(yaml) = fm_str else { continue };
        let Ok(fm) = crate::frontmatter::parse_frontmatter(yaml) else { continue };
        if fm.foreign_format.as_deref() != Some(alias) {
            continue;
        }
        let rel = path.strip_prefix(model_root).unwrap_or(path);
        let qname = crate::walker::derive_qname(rel);
        let dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        return Some(PluginPackage {
            alias: alias.to_string(),
            dir,
            qname,
            index_path: path.display().to_string(),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ElementType, RawFrontmatter};
    use std::sync::atomic::{AtomicU64, Ordering};

    fn tempdir() -> PathBuf {
        static N: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "syscribe-plugins-mod-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn index_elem(qname: &str, file_path: &str, foreign_format: Option<&str>) -> RawElement {
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: RawFrontmatter {
                element_type: Some(ElementType::Package),
                foreign_format: foreign_format.map(|s| s.to_string()),
                ..Default::default()
            },
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: Vec::new(),
        }
    }

    #[test]
    fn no_foreign_format_marker_is_a_no_op() {
        let mut elements = vec![index_elem("Pkg", "Pkg/_index.md", None)];
        let root = tempdir();
        apply_foreign_plugins(&mut elements, &root);
        assert_eq!(elements.len(), 1);
        assert!(elements[0].derive_findings.is_empty());
    }

    #[test]
    fn missing_plugins_entry_is_e551_and_never_panics() {
        let mut elements = vec![index_elem("Legacy", "Legacy/_index.md", Some("toydsl"))];
        let root = tempdir(); // no .syscribe.toml at all
        apply_foreign_plugins(&mut elements, &root);
        assert_eq!(elements.len(), 1, "anchor stays even with no elements synthesized");
        let codes: Vec<&str> = elements[0].derive_findings.iter().map(|(c, _, _)| c.as_str()).collect();
        assert_eq!(codes, vec!["E551"]);
    }

    #[test]
    fn well_formed_plugin_synthesizes_elements_and_strips_native_subtree() {
        let root = tempdir();
        std::fs::write(
            root.join(".syscribe.toml"),
            "[plugins.toydsl]\ncommand = \"/bin/sh\"\nargs = [\"-c\", \"cat >/dev/null; echo '{\\\"elements\\\":[{\\\"qname\\\":\\\"Widget\\\",\\\"type\\\":\\\"PartDef\\\"}]}'\"]\n",
        )
        .unwrap();

        let mut elements = vec![
            index_elem("Legacy", "Legacy/_index.md", Some("toydsl")),
            // A stray native file under the foreign subtree — must be stripped.
            RawElement {
                qualified_name: "Legacy::Stray".to_string(),
                file_path: "Legacy/Stray.md".to_string(),
                frontmatter: RawFrontmatter::default(),
                doc: String::new(),
                parse_issue: None,
                derived: Default::default(),
                derive_findings: Vec::new(),
            },
        ];
        apply_foreign_plugins(&mut elements, &root);

        let qnames: Vec<&str> = elements.iter().map(|e| e.qualified_name.as_str()).collect();
        assert!(qnames.contains(&"Legacy"), "anchor _index.md survives");
        assert!(!qnames.contains(&"Legacy::Stray"), "native stray under the subtree is stripped");
        assert!(qnames.contains(&"Legacy::Widget"), "plugin-synthesized element is merged in");
    }

    #[test]
    fn nested_foreign_format_marker_inside_a_claimed_subtree_is_absorbed_not_a_new_anchor() {
        let mut elements = vec![
            index_elem("Outer", "Outer/_index.md", Some("toydsl")),
            index_elem("Outer::Inner", "Outer/Inner/_index.md", Some("toydsl")),
        ];
        let root = tempdir(); // no [plugins.toydsl] entry -> E551 on the real anchor only
        apply_foreign_plugins(&mut elements, &root);
        let outer = elements.iter().find(|e| e.qualified_name == "Outer").unwrap();
        assert_eq!(outer.derive_findings.len(), 1);
        assert_eq!(outer.derive_findings[0].0, "E551");
    }
}

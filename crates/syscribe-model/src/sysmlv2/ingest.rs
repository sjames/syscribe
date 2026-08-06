//! Native parsing of `.sysml`/`.kerml` files inside a `sysmlSubmodel: true` subtree
//! into `RawElement`s (`REQ-TRS-SYSMLV2-002`, `REQ-TRS-SYSMLV2-007`).
//!
//! `W541` (parse/read failure) is a **placeholder** code — `REQ-TRS-SYSMLV2-006`
//! formalizes the dedicated error/warning code range for this subsystem later;
//! don't read anything permanent into the exact number yet.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::derive::finding;
use crate::element::{ElementType, RawElement, RawFrontmatter};

/// A SysML v2 `package`, merged across every `.sysml`/`.kerml` file in the
/// subtree that contributes to it by name (`REQ-TRS-SYSMLV2-002`'s multi-file
/// merge: two files each declaring `package Foo { ... }` combine into one
/// `Foo` namespace instead of colliding on qname).
#[derive(Default)]
struct MergedPackage {
    /// The file that first introduced this package name at this nesting
    /// position — used as the synthesized `Package` element's own `file_path`.
    /// Not meaningful beyond "some real contributing file"; a package merged
    /// from several files doesn't have one canonical owner.
    declared_in: Option<String>,
    /// Non-`Package` body elements contributed by any file, paired with the
    /// source file each came from (so a synthesized element's own `file_path`
    /// reflects where it was actually declared, not just the owning package).
    body: Vec<(sysml_v2_parser::PackageBodyElement, String)>,
    /// Nested packages, keyed by name and merged the same way as this level.
    children: BTreeMap<String, MergedPackage>,
}

/// Every `.sysml`/`.kerml` file under `dir`, recursively — however nested, since a
/// `sysmlSubmodel` subtree's directory layout below the marked root carries no
/// namespace meaning of its own (`REQ-TRS-SYSMLV2-001`). Sorted for determinism.
fn find_sysml_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "sysml" || ext == "kerml")
        })
        .map(|e| e.into_path())
        .collect();
    files.sort();
    files
}

/// Parse every `.sysml`/`.kerml` file under `dir`, merge same-named packages
/// across files, and convert the mapped element kinds into `RawElement`s owned
/// by `pkg_qname`. A read or parse failure pushes a `W541` finding onto `owner`
/// (the package's own `_index.md` element) and contributes zero elements from
/// that file — never aborts the rest of the subtree.
pub fn ingest_subtree(owner: &mut RawElement, pkg_qname: &str, dir: &Path) -> Vec<RawElement> {
    let mut merged = MergedPackage::default();
    for path in find_sysml_files(dir) {
        let file_path = path.display().to_string();
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                owner.derive_findings.push(finding(
                    "W541",
                    &file_path,
                    &format!("could not read '{file_path}': {e}"),
                ));
                continue;
            }
        };
        match sysml_v2_parser::parse(&content) {
            Ok(root) => merge_root(&mut merged, root, &file_path),
            Err(e) => {
                owner.derive_findings.push(finding(
                    "W541",
                    &file_path,
                    &format!("SysML v2/KerML parse error in '{file_path}': {e}"),
                ));
            }
        }
    }

    let mut out = Vec::new();
    convert_merged(&merged, pkg_qname, &mut out);
    out
}

/// Merge one file's already-parsed root namespace into `target`. Only
/// `Package` is handled so far — `REQ-TRS-SYSMLV2-007`'s remaining fixed kinds
/// land in later commits; everything else (bare root members, `library
/// package`, `namespace`, imports) is silently invisible for now (parse-broad,
/// map-narrow — same posture the full mapping will keep for constructs outside
/// the fixed set).
fn merge_root(target: &mut MergedPackage, root: sysml_v2_parser::RootNamespace, file_path: &str) {
    for node in root.elements {
        if let sysml_v2_parser::RootElement::Package(pkg_node) = node.value {
            merge_package(target, pkg_node.value, file_path);
        }
    }
}

/// Merge one `package` declaration into `target.children`, combining with
/// whatever a same-named package already contributed (from this file or an
/// earlier one).
fn merge_package(target: &mut MergedPackage, pkg: sysml_v2_parser::Package, file_path: &str) {
    let Some(name) = ident_name(&pkg.identification) else {
        return; // anonymous package: no identity to qname or merge against
    };
    let is_new = !target.children.contains_key(&name);
    let entry = target.children.entry(name).or_default();
    if is_new {
        entry.declared_in = Some(file_path.to_string());
    }
    if let sysml_v2_parser::PackageBody::Brace { elements } = pkg.body {
        merge_package_body(entry, elements, file_path);
    }
}

fn merge_package_body(
    target: &mut MergedPackage,
    elements: Vec<sysml_v2_parser::Node<sysml_v2_parser::PackageBodyElement>>,
    file_path: &str,
) {
    for node in elements {
        match node.value {
            sysml_v2_parser::PackageBodyElement::Package(inner) => {
                merge_package(target, inner.value, file_path);
            }
            other => target.body.push((other, file_path.to_string())),
        }
    }
}

fn ident_name(id: &sysml_v2_parser::Identification) -> Option<String> {
    id.name.clone().or_else(|| id.short_name.clone())
}

/// Walk the merged package tree, emitting `RawElement`s under `qname`.
fn convert_merged(merged: &MergedPackage, qname: &str, out: &mut Vec<RawElement>) {
    for (_elem, _file_path) in &merged.body {
        // Element-kind mapping (Part, Attribute, Port, ...) lands in later commits.
    }
    for (name, child) in &merged.children {
        let child_qname = format!("{qname}::{name}");
        let file_path = child.declared_in.as_deref().unwrap_or(qname);
        out.push(synth(&child_qname, file_path, ElementType::Package, name, None, None));
        convert_merged(child, &child_qname, out);
    }
}

/// Build one synthesized `RawElement`. `supertype` carries a Def's `:>`
/// specialization target; `typed_by` carries a Usage's `:` typing target — the two
/// are kept distinct exactly like hand-authored frontmatter does.
fn synth(
    qname: &str,
    file_path: &str,
    ty: ElementType,
    name: &str,
    supertype: Option<String>,
    typed_by: Option<String>,
) -> RawElement {
    RawElement {
        qualified_name: qname.to_string(),
        file_path: file_path.to_string(),
        frontmatter: RawFrontmatter {
            element_type: Some(ty),
            name: Some(name.to_string()),
            supertype: supertype.map(serde_yaml::Value::String),
            typed_by: typed_by.map(serde_yaml::Value::String),
            ..Default::default()
        },
        doc: String::new(),
        parse_issue: None,
        derived: Default::default(),
        derive_findings: Vec::new(),
    }
}

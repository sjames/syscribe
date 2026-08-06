//! Native parsing of `.sysml`/`.kerml` files inside a `sysmlSubmodel: true` subtree
//! into `RawElement`s (`REQ-TRS-SYSMLV2-002`, `REQ-TRS-SYSMLV2-007`).
//!
//! `W541` (parse/read failure) is a **placeholder** code — `REQ-TRS-SYSMLV2-006`
//! formalizes the dedicated error/warning code range for this subsystem later;
//! don't read anything permanent into the exact number yet.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::derive::finding;
use crate::element::{ElementType, RawElement, RawFrontmatter};

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

/// Parse every `.sysml`/`.kerml` file under `dir` and convert the mapped element
/// kinds into `RawElement`s owned by `pkg_qname`. A read or parse failure pushes a
/// `W541` finding onto `owner` (the package's own `_index.md` element) and
/// contributes zero elements from that file — never aborts the rest of the subtree.
pub fn ingest_subtree(owner: &mut RawElement, pkg_qname: &str, dir: &Path) -> Vec<RawElement> {
    let mut out = Vec::new();
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
            Ok(root) => convert_root(root, pkg_qname, &file_path, &mut out),
            Err(e) => {
                owner.derive_findings.push(finding(
                    "W541",
                    &file_path,
                    &format!("SysML v2/KerML parse error in '{file_path}': {e}"),
                ));
            }
        }
    }
    out
}

/// Walk one file's already-parsed root namespace, emitting mapped elements under
/// `qname`. Only `Package` is handled so far — `REQ-TRS-SYSMLV2-007`'s remaining
/// fixed kinds land in later commits; everything else is silently invisible for now
/// (parse-broad, map-narrow — same posture the full mapping will keep for
/// constructs outside the fixed set).
fn convert_root(
    root: sysml_v2_parser::RootNamespace,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    for node in root.elements {
        if let sysml_v2_parser::RootElement::Package(pkg_node) = node.value {
            convert_package(pkg_node.value, qname, file_path, out);
        }
    }
}

fn convert_package(
    pkg: sysml_v2_parser::Package,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    let Some(name) = ident_name(&pkg.identification) else {
        return; // anonymous package: no identity to qname against, skip
    };
    let child_qname = format!("{qname}::{name}");
    out.push(synth(&child_qname, file_path, ElementType::Package, &name, None, None));
    if let sysml_v2_parser::PackageBody::Brace { elements } = pkg.body {
        for node in elements {
            if let sysml_v2_parser::PackageBodyElement::Package(inner) = node.value {
                convert_package(inner.value, &child_qname, file_path, out);
            }
        }
    }
}

fn ident_name(id: &sysml_v2_parser::Identification) -> Option<String> {
    id.name.clone().or_else(|| id.short_name.clone())
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

//! Read-only inspection of the SysMLv2 submodels in a loaded model
//! (`REQ-TRS-SYSMLV2-031`/`-032`): the shared data behind `syscribe sysml` and
//! the MCP `sysml_submodels` tool.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{json, Value};

use crate::element::RawElement;

/// The `sysml-v2-parser` release this build links (`REQ-TRS-SYSMLV2-053`). A test compares it with
/// the version pinned in `Cargo.toml`, so an upgrade cannot leave the report stale.
pub const PARSER_VERSION: &str = "0.54.0";

/// The parser's own AST schema version (`sysml_v2_parser::PARSE_AST_VERSION`).
pub const PARSER_AST_VERSION: u32 = sysml_v2_parser::PARSE_AST_VERSION;

/// Codes of the SysMLv2 ingestion range surfaced by the report.
const SUBMODEL_CODES: &[&str] = &["W540", "W541", "W542", "W543"];

#[derive(Debug, Clone)]
pub struct SubmodelFile {
    pub path: String,
    /// False when the file could not be read or parsed (`W541`).
    pub parsed: bool,
    pub unmapped: BTreeMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct SubmodelFinding {
    pub code: String,
    pub file: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Submodel {
    pub package: String,
    pub index_file: String,
    pub files: Vec<SubmodelFile>,
    pub elements_by_kind: BTreeMap<String, usize>,
    pub unmapped: BTreeMap<String, usize>,
    pub findings: Vec<SubmodelFinding>,
}

impl Submodel {
    pub fn files_parsed(&self) -> usize {
        self.files.iter().filter(|f| f.parsed).count()
    }
    pub fn element_total(&self) -> usize {
        self.elements_by_kind.values().sum()
    }
    pub fn unmapped_total(&self) -> usize {
        self.unmapped.values().sum()
    }
}

fn is_sysml_path(p: &str) -> bool {
    p.ends_with(".sysml") || p.ends_with(".kerml")
}

/// Gather one [`Submodel`] per `sysmlSubmodel: true` package in `elements`
/// (the walked graph, ingestion already applied). Sorted by package qname.
pub fn submodels(elements: &[RawElement]) -> Vec<Submodel> {
    let mut out = Vec::new();
    for anchor in elements.iter().filter(|e| {
        e.frontmatter.sysml_submodel == Some(true) && e.file_path.ends_with("_index.md")
    }) {
        let dir = Path::new(&anchor.file_path).parent().map(Path::to_path_buf).unwrap_or_default();
        let prefix = format!("{}::", anchor.qualified_name);

        // `REQ-TRS-SYSMLV2-059`: the counts come from the ingestion pass itself (a scratch owner
        // absorbs its W54x findings), so they always equal what `W543` reports.
        let mut scratch = anchor.clone();
        let detail = super::ingest::ingest_subtree_detailed(&mut scratch, &anchor.qualified_name, &dir);
        let mut files = Vec::new();
        let mut unmapped: BTreeMap<String, usize> = BTreeMap::new();
        for path in super::ingest::find_sysml_files(&dir) {
            let shown = path.display().to_string();
            let found = detail.file_counts.iter().find(|(f, _)| *f == shown).map(|(_, c)| c);
            let parsed = found.is_some();
            let counts: BTreeMap<String, usize> = found
                .map(|c| c.iter().map(|(k, n)| (k.to_string(), *n)).collect())
                .unwrap_or_default();
            for (k, n) in &counts {
                *unmapped.entry(k.clone()).or_insert(0) += n;
            }
            files.push(SubmodelFile { path: shown, parsed, unmapped: counts });
        }

        let mut elements_by_kind: BTreeMap<String, usize> = BTreeMap::new();
        let mut findings = Vec::new();
        let owned = std::iter::once(anchor).chain(
            elements
                .iter()
                .filter(|e| is_sysml_path(&e.file_path) && e.qualified_name.starts_with(&prefix)),
        );
        for e in owned {
            if !std::ptr::eq(e, anchor) {
                let kind = e
                    .frontmatter
                    .element_type
                    .as_ref()
                    .map_or("Unknown", |t| t.name())
                    .to_string();
                *elements_by_kind.entry(kind).or_insert(0) += 1;
            }
            for (code, file, message) in &e.derive_findings {
                if SUBMODEL_CODES.contains(&code.as_str()) {
                    findings.push(SubmodelFinding {
                        code: code.clone(),
                        file: file.clone(),
                        message: message.clone(),
                    });
                }
            }
        }
        findings.sort_by(|a, b| (&a.code, &a.file, &a.message).cmp(&(&b.code, &b.file, &b.message)));

        out.push(Submodel {
            package: anchor.qualified_name.clone(),
            index_file: anchor.file_path.clone(),
            files,
            elements_by_kind,
            unmapped,
            findings,
        });
    }
    out.sort_by(|a, b| a.package.cmp(&b.package));
    out
}

/// JSON shape shared by `sysml --json` and the MCP `sysml_submodels` tool.
pub fn submodels_json(elements: &[RawElement]) -> Value {
    let subs = submodels(elements);
    json!({
        "parser": {
            "name": "sysml-v2-parser",
            "version": PARSER_VERSION,
            "astVersion": PARSER_AST_VERSION,
        },
        "submodels": subs.iter().map(|s| json!({
            "package": s.package,
            "indexFile": s.index_file,
            "filesParsed": s.files_parsed(),
            "fileCount": s.files.len(),
            "files": s.files.iter().map(|f| json!({
                "path": f.path,
                "parsed": f.parsed,
                "unmapped": f.unmapped,
            })).collect::<Vec<_>>(),
            "elementTotal": s.element_total(),
            "elementsByKind": s.elements_by_kind,
            "unmappedTotal": s.unmapped_total(),
            "unmapped": s.unmapped,
            "findings": s.findings.iter().map(|f| json!({
                "code": f.code, "file": f.file, "message": f.message,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

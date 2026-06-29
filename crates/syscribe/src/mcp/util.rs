//! Shared JSON-shaping helpers for the MCP tools: element summaries, findings,
//! path normalisation, and JSON→YAML value conversion.

use std::path::Path;

use serde_json::{json, Value};
use syscribe_model::element::RawElement;
use syscribe_model::validator::{Finding, Severity};

use crate::query::type_label;

/// Normalise an absolute file path to a model-root-relative path, so findings
/// produced against a temp copy and against the real model compare equal.
pub fn rel_file(file: &str, root: &Path) -> String {
    let root_s = root.to_string_lossy();
    file.strip_prefix(root_s.as_ref())
        .map(|s| s.trim_start_matches(['/', '\\']).to_string())
        .unwrap_or_else(|| file.to_string())
}

/// Lower-case label for a finding severity (`error` | `warning` | `info`).
pub fn severity_str(s: &Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

/// The token-efficient summary view of an element (no body, no full frontmatter).
pub fn elem_summary(e: &RawElement) -> Value {
    json!({
        "qname": e.qualified_name,
        "id": e.frontmatter.id,
        "type": e.frontmatter.element_type.as_ref().map(type_label),
        "name": e.frontmatter.name,
        "status": e.frontmatter.status,
        "file": e.file_path,
    })
}

/// The detailed view: the summary plus the Markdown body and full frontmatter.
pub fn elem_detail(e: &RawElement) -> Value {
    let mut v = elem_summary(e);
    let obj = v.as_object_mut().expect("summary is an object");
    obj.insert("doc".into(), Value::String(e.doc.clone()));
    obj.insert(
        "frontmatter".into(),
        serde_json::to_value(&e.frontmatter).unwrap_or(Value::Null),
    );
    v
}

/// A single finding as JSON, with its file path normalised relative to `root`.
pub fn finding_json(f: &Finding, root: &Path) -> Value {
    json!({
        "code": f.code,
        "severity": severity_str(&f.severity),
        "file": rel_file(&f.file, root),
        "message": f.message,
    })
}

/// Convert a `serde_json::Value` to a `serde_yaml::Value` for frontmatter writes.
pub fn json_to_yaml(v: &Value) -> serde_yaml::Value {
    serde_yaml::to_value(v).unwrap_or(serde_yaml::Value::Null)
}

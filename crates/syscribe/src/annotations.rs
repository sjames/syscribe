//! Annotated-source comment-marker scan invocation (`ADR-SYS-ANNOTATE-001`).
//! `scan` runs one `annotationFormat:`-marked package's scan in isolation and
//! prints the resulting elements — no merge, no validation. Fast feedback
//! loop for authoring `marker`/`include`/`exclude`, including seeing exactly
//! what a marker block produced (or why it didn't) before trusting it inside
//! a full `validate` run.

use std::path::Path;

use serde::Serialize;
use syscribe_model::annotations::{find_package, scan_package};

#[derive(Serialize)]
struct DryRunElement {
    qname: String,
    #[serde(rename = "type")]
    element_type: String,
    #[serde(rename = "filePath")]
    file_path: String,
    frontmatter: serde_json::Value,
}

#[derive(Serialize)]
struct DryRunFinding {
    code: String,
    location: String,
    message: String,
}

#[derive(Serialize)]
struct DryRunReport {
    elements: Vec<DryRunElement>,
    findings: Vec<DryRunFinding>,
}

/// `annotations scan <selector> --dry-run` — `selector` matches a package's
/// qualified name first, then (first match) its `annotationFormat:` label.
/// `--dry-run` is required rather than implied, mirroring `plugins run
/// <alias> --dry-run`'s reasoning: the call site stays unambiguous about
/// what it does (scan in isolation, print raw, no merge).
pub fn cmd_scan(model_root: &Path, selector: &str) -> i32 {
    let Some(pkg) = find_package(model_root, selector) else {
        eprintln!(
            "Error: no package in the model declares annotationFormat: with qname or label '{selector}'"
        );
        return 1;
    };

    let (elems, findings) = scan_package(&pkg, model_root);
    let report = DryRunReport {
        elements: elems
            .into_iter()
            .map(|e| DryRunElement {
                qname: e.qualified_name,
                element_type: e
                    .frontmatter
                    .element_type
                    .as_ref()
                    .map(|t| format!("{t:?}"))
                    .unwrap_or_default(),
                file_path: e.file_path,
                frontmatter: serde_json::to_value(&e.frontmatter).unwrap_or(serde_json::Value::Null),
            })
            .collect(),
        findings: findings
            .into_iter()
            .map(|(code, location, message)| DryRunFinding {
                code: code.to_string(),
                location,
                message,
            })
            .collect(),
    };

    match serde_json::to_string_pretty(&report) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("failed to serialize scan report: {e}");
            1
        }
    }
}

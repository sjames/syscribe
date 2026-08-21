//! Wire types for the stdio-subprocess plugin protocol (`ADR-SYS-PLUGIN-002`).
//!
//! [`PluginRequest`] is serialized to JSON and written to the plugin's stdin;
//! [`ElementsEnvelope`] is what the plugin writes to stdout in response.
//! Conversion into a [`RawElement`] mirrors the FMEA/TARA row-explosion trick
//! already in `walker.rs`: the frontmatter map deserializes straight into
//! [`RawFrontmatter`] via a JSON→YAML round trip, so a plugin-emitted element
//! flows through the resolver/validator identically to a hand-authored one.

use serde::{Deserialize, Serialize};

use crate::element::{ElementType, RawElement, RawFrontmatter};

/// `(code, message)` findings [`convert`] attaches to the owning package's
/// `_index.md` — self-reported plugin diagnostics fold into one `W551`; a
/// per-element conversion failure is `W552`/`W553`.
pub type ConvertFindings = Vec<(&'static str, String)>;

/// Request written to the plugin's stdin as one JSON object, then stdin is closed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRequest {
    pub protocol_version: u32,
    pub alias: String,
    pub package_qname: String,
    /// Absolute path to the package's directory subtree.
    pub package_dir: String,
    /// Absolute path to the model root.
    pub model_root: String,
}

/// Top-level shape a plugin writes to stdout, as one JSON object.
#[derive(Debug, Default, Deserialize)]
pub struct ElementsEnvelope {
    #[serde(default)]
    pub elements: Vec<EnvelopeElement>,
    /// Parse-time diagnostics the plugin wants surfaced (not a hard failure —
    /// folded into a single `W551` finding on the owning package).
    #[serde(default)]
    pub diagnostics: Vec<EnvelopeDiagnostic>,
}

#[derive(Debug, Deserialize)]
pub struct EnvelopeElement {
    /// Relative qname suffix, joined onto the owning package's qname with `::`.
    pub qname: String,
    /// Must name a recognised [`ElementType`] variant; an unrecognised value
    /// deserializes to `ElementType::Unknown` (`#[serde(other)]`) and is
    /// reported as `W553` rather than failing the whole run.
    #[serde(rename = "type")]
    pub element_type: String,
    #[serde(default)]
    pub doc: Option<String>,
    /// Everything else the plugin wants on the element (`id`, `name`,
    /// `custom_fields`, …) — the same free-form escape hatch native model
    /// authors get, flowing straight into typed [`RawFrontmatter`] slots.
    #[serde(flatten)]
    pub frontmatter: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct EnvelopeDiagnostic {
    pub severity: String,
    pub message: String,
    #[serde(default)]
    #[allow(dead_code)] // not yet surfaced individually — folded into one W551
    pub source_ref: Option<String>,
}

/// Parse `raw_json` (a plugin's stdout) and convert every element into a real
/// [`RawElement`] owned by `pkg_qname`/`pkg_file_path`.
///
/// Returns the synthesized elements plus `(code, message)` findings to attach
/// to the owning package's `_index.md`: self-reported plugin diagnostics fold
/// into one `W551`; a per-element conversion failure is `W552`/`W553` and
/// drops only that element, keeping its siblings. A top-level malformed
/// envelope is the only hard `Err` — everything else degrades gracefully.
pub fn convert(
    pkg_qname: &str,
    pkg_file_path: &str,
    raw_json: &str,
) -> Result<(Vec<RawElement>, ConvertFindings), String> {
    let envelope: ElementsEnvelope =
        serde_json::from_str(raw_json).map_err(|e| format!("malformed envelope JSON: {e}"))?;

    let mut findings: ConvertFindings = Vec::new();
    if !envelope.diagnostics.is_empty() {
        let n = envelope.diagnostics.len();
        let preview: Vec<String> = envelope
            .diagnostics
            .iter()
            .take(3)
            .map(|d| format!("{}: {}", d.severity, d.message))
            .collect();
        findings.push((
            "W551",
            format!("plugin reported {n} diagnostic(s): {}", preview.join("; ")),
        ));
    }

    let mut out = Vec::new();
    for env_elem in envelope.elements {
        let raw_qname = env_elem.qname.clone();
        let mut map = env_elem.frontmatter;
        map.insert(
            "type".to_string(),
            serde_json::Value::String(env_elem.element_type.clone()),
        );

        let yaml_value: serde_yaml::Value =
            match serde_yaml::to_value(serde_json::Value::Object(map)) {
                Ok(v) => v,
                Err(e) => {
                    findings.push(("W552", format!("element '{raw_qname}' dropped: {e}")));
                    continue;
                }
            };
        let fm: RawFrontmatter = match serde_yaml::from_value(yaml_value) {
            Ok(fm) => fm,
            Err(e) => {
                findings.push(("W552", format!("element '{raw_qname}' dropped: {e}")));
                continue;
            }
        };

        if fm.element_type == Some(ElementType::Unknown) {
            findings.push((
                "W553",
                format!(
                    "element '{raw_qname}' has unrecognised type '{}', dropped",
                    env_elem.element_type
                ),
            ));
            continue;
        }

        out.push(RawElement {
            qualified_name: format!("{pkg_qname}::{raw_qname}"),
            file_path: pkg_file_path.to_string(),
            frontmatter: fm,
            doc: env_elem.doc.unwrap_or_default(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: Vec::new(),
        });
    }

    Ok((out, findings))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_formed_envelope_converts() {
        let json = r#"{
            "elements": [
                {"qname": "PressureSensor", "type": "PartDef", "name": "PressureSensor", "doc": "Measures cabin pressure."},
                {"qname": "SamplingRate", "type": "RequirementDef", "id": "REQ-TOY-001"}
            ],
            "diagnostics": []
        }"#;
        let (elems, findings) = convert("Legacy::ToyDsl", "Legacy/ToyDsl/_index.md", json).unwrap();
        assert!(findings.is_empty());
        assert_eq!(elems.len(), 2);
        assert_eq!(elems[0].qualified_name, "Legacy::ToyDsl::PressureSensor");
        assert_eq!(elems[0].file_path, "Legacy/ToyDsl/_index.md");
        assert_eq!(elems[0].frontmatter.element_type, Some(ElementType::PartDef));
        assert_eq!(elems[0].doc, "Measures cabin pressure.");
        assert_eq!(elems[1].frontmatter.id.as_deref(), Some("REQ-TOY-001"));
    }

    #[test]
    fn unrecognised_type_drops_with_w553() {
        let json = r#"{"elements": [{"qname": "Foo", "type": "NotARealType"}]}"#;
        let (elems, findings) = convert("Pkg", "Pkg/_index.md", json).unwrap();
        assert!(elems.is_empty());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].0, "W553");
    }

    #[test]
    fn bad_frontmatter_drops_with_w552_siblings_kept() {
        // `silLevel` is `Option<u8>`; a non-numeric string fails to deserialize.
        let json = r#"{"elements": [
            {"qname": "Bad", "type": "PartDef", "silLevel": "not-a-number"},
            {"qname": "Good", "type": "PartDef"}
        ]}"#;
        let (elems, findings) = convert("Pkg", "Pkg/_index.md", json).unwrap();
        assert_eq!(elems.len(), 1);
        assert_eq!(elems[0].qualified_name, "Pkg::Good");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].0, "W552");
    }

    #[test]
    fn self_reported_diagnostics_fold_into_one_w551() {
        let json = r#"{"elements": [], "diagnostics": [
            {"severity": "warning", "message": "ambiguous token at line 4"},
            {"severity": "error", "message": "unterminated block"}
        ]}"#;
        let (elems, findings) = convert("Pkg", "Pkg/_index.md", json).unwrap();
        assert!(elems.is_empty());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].0, "W551");
        assert!(findings[0].1.contains("2 diagnostic"));
    }

    #[test]
    fn malformed_json_is_a_hard_error() {
        let err = convert("Pkg", "Pkg/_index.md", "not json").unwrap_err();
        assert!(err.contains("malformed envelope JSON"));
    }
}

//! JSON envelope returned by a foreign-format WASM plugin (ADR-SYS-PLUGIN-001,
//! REQ-TRS-PLUGIN-002).
//!
//! Conversion into a [`RawElement`] mirrors the existing TARA/FMEA row-explosion
//! trick in `walker.rs`: the frontmatter map deserializes straight into
//! [`RawFrontmatter`] via a JSON→YAML round trip, so a plugin-emitted element
//! flows through the resolver/validator identically to a hand-authored one.

use crate::element::{ElementType, RawElement, RawFrontmatter};

/// Top-level shape a plugin's `parse` export returns as a JSON string.
#[derive(Debug, Default, serde::Deserialize)]
pub struct ElementsEnvelope {
    #[serde(default)]
    pub elements: Vec<EnvelopeElement>,
    /// Parse-time diagnostics the plugin wants surfaced (not a hard failure —
    /// folded into a single `W532` finding on the owning package).
    #[serde(default)]
    pub diagnostics: Vec<EnvelopeDiagnostic>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EnvelopeElement {
    /// Relative qname suffix, joined onto the owning package's qname with `::`.
    pub qname: String,
    /// Must name a recognised [`ElementType`] variant; an unrecognised value
    /// deserializes to `ElementType::Unknown` (`#[serde(other)]`) and is
    /// reported as `W534` by the caller rather than failing the whole run.
    #[serde(rename = "type")]
    pub element_type: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub doc: String,
    /// Everything else the plugin wants on the element — same free-form
    /// escape hatch native model authors get via `custom_fields`.
    #[serde(flatten)]
    pub frontmatter: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EnvelopeDiagnostic {
    pub severity: String,
    pub message: String,
    #[serde(default)]
    pub source_ref: Option<String>,
}

/// Outcome of converting one [`EnvelopeElement`] into a [`RawElement`].
#[derive(Debug)]
pub enum ConvertOutcome {
    Ok(RawElement),
    /// `W533` — the frontmatter map didn't deserialize into [`RawFrontmatter`].
    BadFrontmatter(String),
    /// `W534` — `type:` isn't a recognised [`ElementType`].
    UnknownType(String),
}

/// Convert one plugin-emitted element into a [`RawElement`] owned by `package`.
///
/// `package_file_path` is the owning package's `_index.md` path — borrowed as
/// the synthetic element's `file_path`, exactly as `explode_tara_entries` and
/// `explode_fmea_entries` already do for their synthesized siblings, so
/// diagnostics and "view source" always point somewhere real on disk.
pub fn convert_element(
    package_qname: &str,
    package_file_path: &str,
    e: EnvelopeElement,
) -> ConvertOutcome {
    let mut map = e.frontmatter;
    map.insert(
        "type".to_string(),
        serde_json::Value::String(e.element_type.clone()),
    );
    if let Some(id) = &e.id {
        map.insert("id".to_string(), serde_json::Value::String(id.clone()));
    }
    if let Some(name) = &e.name {
        map.insert("name".to_string(), serde_json::Value::String(name.clone()));
    }

    let yaml_value: serde_yaml::Value = match serde_yaml::to_value(serde_json::Value::Object(map)) {
        Ok(v) => v,
        Err(err) => return ConvertOutcome::BadFrontmatter(err.to_string()),
    };
    let fm: RawFrontmatter = match serde_yaml::from_value(yaml_value) {
        Ok(fm) => fm,
        Err(err) => return ConvertOutcome::BadFrontmatter(err.to_string()),
    };

    if fm.element_type == Some(ElementType::Unknown) {
        return ConvertOutcome::UnknownType(e.element_type);
    }

    ConvertOutcome::Ok(RawElement {
        qualified_name: format!("{package_qname}::{}", e.qname),
        file_path: package_file_path.to_string(),
        frontmatter: fm,
        doc: e.doc,
        parse_issue: None,
        derived: Default::default(),
        derive_findings: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn elem(element_type: &str, extra: serde_json::Map<String, serde_json::Value>) -> EnvelopeElement {
        EnvelopeElement {
            qname: "Foo".to_string(),
            element_type: element_type.to_string(),
            id: None,
            name: Some("Foo".to_string()),
            doc: "doc".to_string(),
            frontmatter: extra,
        }
    }

    #[test]
    fn recognised_type_converts() {
        match convert_element("Pkg", "Pkg/_index.md", elem("PartDef", Default::default())) {
            ConvertOutcome::Ok(raw) => {
                assert_eq!(raw.qualified_name, "Pkg::Foo");
                assert_eq!(raw.file_path, "Pkg/_index.md");
                assert_eq!(raw.frontmatter.element_type, Some(ElementType::PartDef));
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn unrecognised_type_is_reported() {
        match convert_element("Pkg", "Pkg/_index.md", elem("NotARealType", Default::default())) {
            ConvertOutcome::UnknownType(t) => assert_eq!(t, "NotARealType"),
            _ => panic!("expected UnknownType"),
        }
    }

    #[test]
    fn frontmatter_type_mismatch_is_reported() {
        let mut extra = serde_json::Map::new();
        // `silLevel` is `Option<u8>`; a non-numeric string fails to deserialize.
        extra.insert("silLevel".to_string(), serde_json::Value::String("not-a-number".to_string()));
        match convert_element("Pkg", "Pkg/_index.md", elem("PartDef", extra)) {
            ConvertOutcome::BadFrontmatter(_) => {}
            _ => panic!("expected BadFrontmatter"),
        }
    }

    #[test]
    fn extra_fields_flow_into_typed_frontmatter_slots() {
        let mut extra = serde_json::Map::new();
        extra.insert("status".to_string(), serde_json::Value::String("draft".to_string()));
        match convert_element("Pkg", "Pkg/_index.md", elem("PartDef", extra)) {
            ConvertOutcome::Ok(raw) => assert_eq!(raw.frontmatter.status.as_deref(), Some("draft")),
            other => panic!("expected Ok, got {other:?}"),
        }
    }
}

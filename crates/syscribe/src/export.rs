//! `syscribe model export` — structured, machine-readable dump of the whole
//! model graph (issue #2).
//!
//! Emits every element with its identity, typed frontmatter, and resolved
//! relationships (the reverse indices the validator already computes), so
//! external tools — CI gates, dashboards, LLM agents — can consume the model
//! without re-implementing the parser.

use syscribe_model::config::ValidateConfig;
use syscribe_model::element::RawElement;
use syscribe_model::validator;

/// Stable schema version for consumers to key on. Bump on breaking changes.
pub const SCHEMA_VERSION: &str = "1.0";

/// Recursively drop `null` object members so the export carries only the
/// frontmatter fields that are actually set.
fn strip_nulls(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let cleaned: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .filter(|(_, val)| !val.is_null())
                .map(|(k, val)| (k, strip_nulls(val)))
                .collect();
            serde_json::Value::Object(cleaned)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(strip_nulls).collect())
        }
        other => other,
    }
}

/// Build the JSON object for a single element.
fn element_json(
    elem: &RawElement,
    result: &validator::ValidationResult,
    config: &ValidateConfig,
) -> serde_json::Value {
    let fm = &elem.frontmatter;

    // Typed frontmatter with unset fields removed.
    let frontmatter = strip_nulls(serde_json::to_value(fm).unwrap_or(serde_json::Value::Null));

    // Resolved reverse-index relationships, keyed by the element's stable id.
    let mut computed = serde_json::Map::new();
    if let Some(id) = fm.id.as_deref() {
        if let Some(tcs) = result.verified_by.get(id) {
            computed.insert("verifiedBy".into(), serde_json::json!(tcs));
        }
        if let Some(children) = result.derived_children.get(id) {
            computed.insert("derivedChildren".into(), serde_json::json!(children));
        }
    }
    // REQ-TRS-MG-001/002 — refinedBy / actorIn key by stable id when present,
    // else qualified name (matching the validator's index_key).
    let mg_key = fm.id.as_deref().unwrap_or(elem.qualified_name.as_str());
    if let Some(ucs) = result.refined_by.get(mg_key) {
        computed.insert("refinedBy".into(), serde_json::json!(ucs));
    }
    if let Some(ucs) = result.actor_in.get(mg_key) {
        computed.insert("actorIn".into(), serde_json::json!(ucs));
    }
    // REQ-TRS-MG-008 — mopRefinedBy: the MoPs refining this MoE.
    if let Some(mops) = result.mop_refined_by.get(mg_key) {
        computed.insert("mopRefinedBy".into(), serde_json::json!(mops));
    }
    // REQ-TRS-ALLOC-001 — allocatedFrom: the sources allocated to this target,
    // over both authoring forms. Keyed by stable id else qname.
    if let Some(srcs) = result.allocated_from.get(mg_key) {
        computed.insert("allocatedFrom".into(), serde_json::json!(srcs));
    }

    let mut obj = serde_json::Map::new();
    obj.insert("qname".into(), serde_json::json!(elem.qualified_name));
    obj.insert("file".into(), serde_json::json!(elem.file_path));
    // REQ-TRS-LINK-001/004 — hosted source URL when `[links]` is configured.
    if let Some(url) = config.hosted_url_for(
        &elem.file_path,
        &elem.qualified_name,
        fm.id.as_deref().unwrap_or(""),
    ) {
        obj.insert("url".into(), serde_json::json!(url));
    }
    if let Some(id) = &fm.id {
        obj.insert("id".into(), serde_json::json!(id));
    }
    obj.insert(
        "type".into(),
        serde_json::to_value(&fm.element_type).unwrap_or(serde_json::Value::Null),
    );
    if let Some(name) = &fm.name {
        obj.insert("name".into(), serde_json::json!(name));
    }
    obj.insert("frontmatter".into(), frontmatter);
    if !computed.is_empty() {
        obj.insert("computed".into(), serde_json::Value::Object(computed));
    }
    serde_json::Value::Object(obj)
}

/// `export` subcommand. `ndjson` switches from a single pretty JSON document to
/// newline-delimited JSON (one header line, then one element per line).
pub fn cmd_export(elements: &[RawElement], config: &ValidateConfig, ndjson: bool) {
    let result = validator::validate_with_config(elements, config);

    if ndjson {
        let header = serde_json::json!({
            "schemaVersion": SCHEMA_VERSION,
            "kind": "header",
            "elementCount": elements.len(),
        });
        println!("{}", serde_json::to_string(&header).unwrap());
        for elem in elements {
            println!("{}", serde_json::to_string(&element_json(elem, &result, config)).unwrap());
        }
        return;
    }

    let items: Vec<serde_json::Value> =
        elements.iter().map(|e| element_json(e, &result, config)).collect();
    let doc = serde_json::json!({
        "schemaVersion": SCHEMA_VERSION,
        "elementCount": elements.len(),
        "elements": items,
    });
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}

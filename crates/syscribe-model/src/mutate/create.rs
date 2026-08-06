//! Element-creation planning: build validated new-element file content ready to
//! write, allocating a stable id for id-identified types when the caller doesn't
//! supply one explicitly.

use serde_json::Value;

use crate::element::{ElementType, RawElement};
use crate::resolver::{is_stable_id, STABLE_ID_KINDS};

use super::mv::valid_qname;

/// Failure building a `create` plan.
#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error("an element with this qualified name already exists")]
    AlreadyExists,
    #[error("not a valid basic qualified name")]
    InvalidQname,
}

/// A planned new-element write: where to write it, the file content, and the
/// reported id (auto-allocated for id-identified types, or the explicit one).
pub struct CreatePlan {
    pub rel: String,
    pub content: String,
    pub id: Value,
}

/// The built-in stable-id prefix for an id-identified element type, keyed by the
/// type's exact Rust variant name (identical to [`STABLE_ID_KINDS`]'s type-name
/// column — fieldless enum variants `Debug`-format to their own identifier, so
/// this needs no separate label table).
fn builtin_prefix(et: &ElementType) -> Option<&'static str> {
    let label = format!("{et:?}");
    STABLE_ID_KINDS
        .iter()
        .find(|(ty, _, _)| *ty == label)
        .map(|(_, p, _)| *p)
}

/// Convert a `serde_json::Value` to a `serde_yaml::Value` for frontmatter writes.
fn json_to_yaml(v: &Value) -> serde_yaml::Value {
    serde_yaml::to_value(v).unwrap_or(serde_yaml::Value::Null)
}

/// Allocate the next unused `<prefix>-GEN-{n:03}` stable id not already present
/// among `elements` (and recognised by [`is_stable_id`]).
fn alloc_stable_id(elements: &[RawElement], prefix: &str) -> String {
    let mut n = 1u32;
    loop {
        let cand = format!("{prefix}-GEN-{n:03}");
        let taken = elements
            .iter()
            .any(|e| e.frontmatter.id.as_deref() == Some(cand.as_str()));
        if !taken && is_stable_id(&cand) {
            return cand;
        }
        n += 1;
    }
}

/// Validate inputs and build the file content for a `create` (single tool or a
/// batch op). Returns `Err` on an already-existing or syntactically invalid qname.
pub fn plan_create(
    elements: &[RawElement],
    qname_raw: &str,
    type_name: &str,
    fields: Option<&Value>,
    doc: Option<&str>,
) -> Result<CreatePlan, CreateError> {
    let qname = qname_raw.replace('/', "::");
    if elements.iter().any(|e| e.qualified_name == qname) {
        return Err(CreateError::AlreadyExists);
    }
    if !valid_qname(&qname) {
        return Err(CreateError::InvalidQname);
    }
    let etype: ElementType = serde_yaml::from_value(serde_yaml::Value::String(type_name.to_string()))
        .unwrap_or(ElementType::Unknown);
    let fields_obj = fields.and_then(|v| v.as_object());
    let explicit_id = fields_obj
        .and_then(|o| o.get("id"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let mut allocated_id: Option<String> = None;
    if etype.is_id_identified() && explicit_id.is_none() {
        if let Some(prefix) = builtin_prefix(&etype) {
            allocated_id = Some(alloc_stable_id(elements, prefix));
        }
    }

    let mut map = serde_yaml::Mapping::new();
    map.insert("type".into(), serde_yaml::Value::String(type_name.to_string()));
    if let Some(id) = &allocated_id {
        map.insert("id".into(), serde_yaml::Value::String(id.clone()));
    }
    if let Some(o) = fields_obj {
        for (k, v) in o {
            map.insert(serde_yaml::Value::String(k.clone()), json_to_yaml(v));
        }
    }
    let yaml = serde_yaml::to_string(&serde_yaml::Value::Mapping(map)).unwrap_or_default();
    let content = format!("---\n{yaml}---\n\n{}\n", doc.unwrap_or(""));
    let rel = format!("{}.md", qname.replace("::", "/"));
    let id = allocated_id
        .or(explicit_id)
        .map(Value::String)
        .unwrap_or(Value::Null);
    Ok(CreatePlan { rel, content, id })
}

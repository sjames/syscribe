//! Element-creation planning: build validated new-element file content ready to
//! write, allocating a stable id for id-identified types when the caller doesn't
//! supply one explicitly.
//!
//! File placement follows the element's identity (GH #185):
//!
//! * id-identified types (`Requirement`, `TestCase`, `ADR`, `PlanningItem`, …) are
//!   stored as `<parent>/<id>.md`, qualified name `Parent::<id>`;
//! * `Package` is stored as `<qname>/_index.md`;
//! * every other (name-identified) type is stored as `<qname>.md`.

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
    #[error("not a valid parent package qualified name")]
    InvalidParent,
    #[error("`{0}` is not a valid stable id")]
    InvalidId(String),
    #[error("the qualified name's last segment `{qname_id}` conflicts with fields.id `{field_id}`")]
    IdConflict { qname_id: String, field_id: String },
    #[error("an id-identified element needs a `parent` package or a `qname` ending in its id")]
    MissingTarget,
    #[error("a name-identified element needs a `qname` (parent-only addressing is for id-identified types)")]
    NeedsQname,
}

/// A planned new-element write: where to write it, the file content, and the
/// reported id (auto-allocated for id-identified types, or the explicit one).
pub struct CreatePlan {
    /// Qualified name the new element will have once written.
    pub qname: String,
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
        let taken = elements.iter().any(|e| {
            e.frontmatter.id.as_deref() == Some(cand.as_str())
                || e.qualified_name.rsplit("::").next() == Some(cand.as_str())
        });
        if !taken && is_stable_id(&cand) {
            return cand;
        }
        n += 1;
    }
}

/// Build a plan for a `create` addressed by qualified name. See [`plan_create_in`].
pub fn plan_create(
    elements: &[RawElement],
    qname_raw: &str,
    type_name: &str,
    fields: Option<&Value>,
    doc: Option<&str>,
) -> Result<CreatePlan, CreateError> {
    plan_create_in(elements, None, Some(qname_raw), type_name, fields, doc)
}

/// Validate inputs and build the file content for a `create` (single tool or a
/// batch op). Address the new element with `qname`, `parent`, or both:
///
/// * id-identified type + `parent` (`""` = model root): the id is `fields.id` or
///   auto-allocated; written to `<parent>/<id>.md`.
/// * id-identified type + `qname` whose last segment is a stable id: written to
///   `<qname>.md`; `fields.id`, when present, must equal that segment.
/// * id-identified type + a plain basic `qname`: legacy name-derived `<qname>.md`.
/// * `Package` + `qname`: `<qname>/_index.md`.
/// * other types + `qname`: `<qname>.md`.
pub fn plan_create_in(
    elements: &[RawElement],
    parent_raw: Option<&str>,
    qname_raw: Option<&str>,
    type_name: &str,
    fields: Option<&Value>,
    doc: Option<&str>,
) -> Result<CreatePlan, CreateError> {
    let etype: ElementType = serde_yaml::from_value(serde_yaml::Value::String(type_name.to_string()))
        .unwrap_or(ElementType::Unknown);
    let fields_obj = fields.and_then(|v| v.as_object());
    let explicit_id = fields_obj
        .and_then(|o| o.get("id"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let parent = parent_raw.map(|p| p.replace('/', "::"));
    let qname_in = qname_raw.map(|q| q.replace('/', "::"));
    if let Some(p) = &parent {
        if !p.is_empty() && !valid_qname(p) {
            return Err(CreateError::InvalidParent);
        }
    }

    let qname: String;
    let rel: String;
    let mut id: Option<String> = None;
    let mut allocated = false;

    if etype.is_id_identified() {
        // The (parent, leaf) pair when the leaf is a stable id, else the legacy form.
        let split = |q: &str| -> (String, String) {
            match q.rsplit_once("::") {
                Some((a, b)) => (a.to_string(), b.to_string()),
                None => (String::new(), q.to_string()),
            }
        };
        match (&parent, &qname_in) {
            (None, None) => return Err(CreateError::MissingTarget),
            (Some(p), None) => {
                let the_id = match &explicit_id {
                    Some(i) => i.clone(),
                    None => {
                        let prefix = builtin_prefix(&etype).ok_or(CreateError::MissingTarget)?;
                        allocated = true;
                        alloc_stable_id(elements, prefix)
                    }
                };
                if !is_stable_id(&the_id) {
                    return Err(CreateError::InvalidId(the_id));
                }
                qname = if p.is_empty() { the_id.clone() } else { format!("{p}::{the_id}") };
                id = Some(the_id);
            }
            (_, Some(q)) => {
                let (q_par, leaf) = split(q);
                if is_stable_id(&leaf) {
                    if !q_par.is_empty() && !valid_qname(&q_par) {
                        return Err(CreateError::InvalidQname);
                    }
                    if let Some(p) = &parent {
                        if *p != q_par {
                            return Err(CreateError::InvalidQname);
                        }
                    }
                    if let Some(f) = &explicit_id {
                        if *f != leaf {
                            return Err(CreateError::IdConflict { qname_id: leaf, field_id: f.clone() });
                        }
                    }
                    qname = q.clone();
                    id = Some(leaf);
                } else {
                    // Legacy: a basic-name qname keeps a name-derived file.
                    if parent.is_some() || !valid_qname(q) {
                        return Err(CreateError::InvalidQname);
                    }
                    if let Some(f) = &explicit_id {
                        if !is_stable_id(f) {
                            return Err(CreateError::InvalidId(f.clone()));
                        }
                        id = Some(f.clone());
                    } else if let Some(prefix) = builtin_prefix(&etype) {
                        id = Some(alloc_stable_id(elements, prefix));
                        allocated = true;
                    }
                    qname = q.clone();
                }
            }
        }
        rel = format!("{}.md", qname.replace("::", "/"));
    } else {
        if qname_in.is_none() {
            return Err(if parent.is_some() { CreateError::NeedsQname } else { CreateError::MissingTarget });
        }
        let q = qname_in.unwrap_or_default();
        if !valid_qname(&q) {
            return Err(CreateError::InvalidQname);
        }
        rel = if matches!(etype, ElementType::Package) {
            format!("{}/_index.md", q.replace("::", "/"))
        } else {
            format!("{}.md", q.replace("::", "/"))
        };
        qname = q;
    }

    if elements.iter().any(|e| e.qualified_name == qname) {
        return Err(CreateError::AlreadyExists);
    }
    // A stable id is unique model-wide, wherever it is filed.
    if let Some(i) = &id {
        if elements.iter().any(|e| e.frontmatter.id.as_deref() == Some(i.as_str())) {
            return Err(CreateError::AlreadyExists);
        }
    }

    let mut map = serde_yaml::Mapping::new();
    map.insert("type".into(), serde_yaml::Value::String(type_name.to_string()));
    if let Some(i) = &id {
        // Only written ahead of `fields` when derived; an explicit one comes from `fields`.
        if allocated || explicit_id.is_none() {
            map.insert("id".into(), serde_yaml::Value::String(i.clone()));
        }
    }
    if let Some(o) = fields_obj {
        for (k, v) in o {
            map.insert(serde_yaml::Value::String(k.clone()), json_to_yaml(v));
        }
    }
    let yaml = serde_yaml::to_string(&serde_yaml::Value::Mapping(map)).unwrap_or_default();
    let content = format!("---\n{yaml}---\n\n{}\n", doc.unwrap_or(""));
    Ok(CreatePlan {
        qname,
        rel,
        content,
        id: id.map(Value::String).unwrap_or(Value::Null),
    })
}

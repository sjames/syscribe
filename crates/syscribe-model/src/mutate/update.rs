//! Update-fields application: merge arbitrary frontmatter field edits (and an
//! optional body replacement) into an already-read `.md` file's content.
//!
//! Promoted from `crates/syscribe/src/mcp/mod.rs::apply_update` so both the
//! MCP `update_element` tool and `syscribe-server`'s `PUT /api/elements/{*qname}`
//! route share one implementation instead of each carrying its own copy.
//! Pure string-in/string-out — callers own the file I/O and (for guarded
//! writes) the candidate-copy staging.

use serde_json::Value;

use crate::frontmatter::{patch_frontmatter, PatchFrontmatterError};

/// Convert a `serde_json::Value` to a `serde_yaml::Value` for frontmatter writes.
fn json_to_yaml(v: &Value) -> serde_yaml::Value {
    serde_yaml::to_value(v).unwrap_or(serde_yaml::Value::Null)
}

/// Merge `fields` into `content`'s YAML frontmatter mapping — an explicit JSON
/// `null` value removes the key, any other value is inserted/overwritten —
/// and optionally replace the Markdown body with `doc`. Unknown/untouched
/// keys and the body (when `doc` is `None`) pass through unchanged.
pub fn apply_update_fields(
    content: &str,
    fields: Option<&Value>,
    doc: Option<&str>,
) -> Result<String, PatchFrontmatterError> {
    patch_frontmatter(content, doc, |map| {
        if let Some(Value::Object(o)) = fields {
            for (k, v) in o {
                let key = serde_yaml::Value::String(k.clone());
                if v.is_null() {
                    map.remove(&key);
                } else {
                    map.insert(key, json_to_yaml(v));
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_fields_and_null_removes_key() {
        let content = "---\ntype: PartDef\nname: Widget\nstatus: draft\n---\n\nDoc body.\n";
        let fields = serde_json::json!({"name": "NewName", "status": null, "isAbstract": true});
        let out = apply_update_fields(content, Some(&fields), None).unwrap();
        assert!(out.contains("name: NewName"));
        assert!(!out.contains("status:"));
        assert!(out.contains("isAbstract: true"));
        assert!(out.contains("Doc body."));
    }

    #[test]
    fn replaces_body_when_doc_given() {
        let content = "---\ntype: PartDef\nname: Widget\n---\n\nOld body.\n";
        let out = apply_update_fields(content, None, Some("New body.\n")).unwrap();
        assert!(out.contains("New body."));
        assert!(!out.contains("Old body."));
    }

    #[test]
    fn no_fields_and_no_doc_is_a_no_op_round_trip() {
        let content = "---\ntype: PartDef\nname: Widget\n---\n\nOld body.\n";
        let out = apply_update_fields(content, None, None).unwrap();
        assert!(out.contains("name: Widget"));
        assert!(out.contains("Old body."));
    }
}

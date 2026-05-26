use anyhow::{Context, Result};
use crate::element::RawFrontmatter;

/// Split a `.md` file content into (frontmatter_yaml, markdown_body).
/// Returns (None, full_content) if no YAML front matter block found.
pub fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    let content = content.trim_start_matches('\u{FEFF}'); // strip BOM — still borrows from param
    if !content.starts_with("---") {
        return (None, content);
    }
    // Find closing ---
    let after_open = &content[3..];
    let close = after_open.find("\n---").or_else(|| after_open.find("\r\n---"));
    match close {
        None => (None, content),
        Some(pos) => {
            let yaml = after_open[..pos].trim_start_matches('\n').trim_start_matches('\r');
            let rest_start = pos + 4; // skip "\n---"
            let body = after_open[rest_start..].trim_start_matches('\n').trim_start_matches('\r');
            (Some(yaml), body)
        }
    }
}

/// Parse YAML frontmatter string into `RawFrontmatter`.
pub fn parse_frontmatter(yaml: &str) -> Result<RawFrontmatter> {
    serde_yaml::from_str(yaml).context("Failed to parse YAML frontmatter")
}

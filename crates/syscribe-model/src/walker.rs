use std::path::{Path, PathBuf};
use anyhow::Result;
use walkdir::WalkDir;
use tracing::{warn, debug};
use crate::element::RawElement;
use crate::frontmatter::{split_frontmatter, parse_frontmatter};

/// Derive a qualified name from a file path relative to the model root.
/// `_index.md` contributes no name segment (represents the directory itself).
pub fn derive_qname(rel_path: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    let components: Vec<_> = rel_path.components().collect();
    let total = components.len();
    for (i, comp) in components.iter().enumerate() {
        let s = comp.as_os_str().to_string_lossy();
        let is_last = i + 1 == total;
        if is_last {
            // File component
            let stem = if s.ends_with(".md") { &s[..s.len() - 3] } else { &s };
            if stem == "_index" {
                // Don't add a segment — the directory name is already added
            } else {
                parts.push(stem.to_string());
            }
        } else {
            // rel_path is already relative to model_root; all directory
            // components are real namespace segments.
            parts.push(s.to_string());
        }
    }
    parts.join("::")
}

/// Walk `model_root`, parse every `.md` file, return `Vec<RawElement>`.
pub fn walk_model(model_root: &Path) -> Result<Vec<RawElement>> {
    let mut elements = Vec::new();

    // Two-pass: collect all paths first, sort so _index.md comes before siblings
    let mut paths: Vec<PathBuf> = WalkDir::new(model_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        .map(|e| e.into_path())
        .collect();

    // Sort: shallower paths first, _index.md before siblings at same depth
    paths.sort_by(|a, b| {
        let da = a.components().count();
        let db = b.components().count();
        da.cmp(&db).then_with(|| {
            let a_is_index = a.file_name().map_or(false, |n| n == "_index.md");
            let b_is_index = b.file_name().map_or(false, |n| n == "_index.md");
            b_is_index.cmp(&a_is_index).then_with(|| a.cmp(b))
        })
    });

    for path in &paths {
        let rel = path.strip_prefix(model_root).unwrap_or(path);
        let qname = derive_qname(rel);
        let file_path = path.display().to_string();

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Cannot read {}: {}", file_path, e);
                continue;
            }
        };

        let (fm_str, body) = split_frontmatter(&content);
        let frontmatter = match fm_str {
            None => {
                debug!("No frontmatter in {}", file_path);
                Default::default()
            }
            Some(yaml) => match parse_frontmatter(yaml) {
                Ok(fm) => fm,
                Err(e) => {
                    warn!("Frontmatter parse error in {}: {}", file_path, e);
                    Default::default()
                }
            }
        };

        elements.push(RawElement {
            qualified_name: qname,
            file_path,
            frontmatter,
            doc: body.to_string(),
        });
    }

    Ok(elements)
}

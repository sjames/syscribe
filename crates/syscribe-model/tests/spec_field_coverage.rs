//! Every frontmatter field the parser accepts is documented in both the full
//! specification (`spec/markdown-sysml-format.md`, Appendix A or a field-table
//! row) and the embedded `syscribe spec fields` reference.
//!
//! The field names are read from the four frontmatter structs in `element.rs`, so
//! adding a field without documenting it fails here.

use std::collections::HashSet;
use std::path::Path;

/// Parser state that authors never write.
const INTERNAL: &[&str] = &[
    "appliesWhenInherited", // computed from the directory chain
    "cold",                 // the cold-tier container
    "extra",                // the unknown-key catch-all (W047)
    "inherited",            // computed configuration inheritance
    "placeholderFields",    // computed from {{…}} whole-value placeholders
    "profileEnabled",       // set by the walker from `[metadata] profiles`
    "tier2",
    "tier3",
    "unknownFmeaKeys",
    "view", // the parsed mapping form of `include:`
    "variantRetained", // set on a clone by a staged projection (REQ-TRS-BT-031)
];

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn frontmatter_fields() -> Vec<String> {
    let src = std::fs::read_to_string(root().join("src/element.rs")).unwrap();
    let mut out = HashSet::new();
    for name in ["RawFrontmatter", "ColdFrontmatter", "ColdFrontmatter2", "ColdFrontmatter3"] {
        let start = src.find(&format!("pub struct {name} {{")).unwrap_or_else(|| panic!("no struct {name}"));
        let end = src[start..].find("\n}\n").unwrap() + start;
        let body = &src[start..end];
        let mut attrs = String::new();
        for line in body.lines().skip(1) {
            let t = line.trim();
            if t.starts_with("#[") {
                attrs.push_str(t);
            } else if let Some(rest) = t.strip_prefix("pub ") {
                if let Some((field, _)) = rest.split_once(':') {
                    let renamed = attrs.split("rename = \"").nth(1).and_then(|r| r.split('"').next());
                    out.insert(match renamed {
                        Some(r) => r.to_string(),
                        None => camel(field.trim()),
                    });
                }
                attrs.clear();
            } else if !t.starts_with("///") && !t.is_empty() {
                attrs.clear();
            }
        }
    }
    let mut v: Vec<String> = out.into_iter().filter(|f| !INTERNAL.contains(&f.as_str())).collect();
    v.sort();
    v
}

fn camel(snake: &str) -> String {
    let mut out = String::new();
    let mut up = false;
    for c in snake.chars() {
        if c == '_' {
            up = true;
        } else if up {
            out.extend(c.to_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn snake(camel: &str) -> String {
    camel.chars().fold(String::new(), |mut s, c| {
        if c.is_uppercase() {
            s.push('_');
            s.extend(c.to_lowercase());
        } else {
            s.push(c);
        }
        s
    })
}

fn backticked(s: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut parts = s.split('`');
    parts.next();
    for (i, p) in parts.enumerate() {
        if i % 2 == 0 && !p.is_empty() && p.chars().all(|c| c.is_alphanumeric() || c == '_') {
            out.insert(p.to_string());
        }
    }
    out
}

/// Backticked names in the first cell of every table row.
fn first_cells(doc: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for line in doc.lines().filter(|l| l.starts_with('|')) {
        if let Some(cell) = line.split('|').nth(1) {
            out.extend(backticked(cell));
        }
    }
    out
}

#[test]
fn every_frontmatter_field_is_documented() {
    let fields = frontmatter_fields();
    assert!(fields.len() > 300, "field extraction broke: {}", fields.len());

    let spec = std::fs::read_to_string(root().join("../../spec/markdown-sysml-format.md")).unwrap();
    let a = spec.find("## Appendix A").expect("Appendix A");
    let b = spec.find("## Appendix B").expect("Appendix B");
    let mut in_spec = first_cells(&spec);
    in_spec.extend(backticked(&spec[a..b]));

    let reference = std::fs::read_to_string(root().join("../../prompts/spec/fields.md")).unwrap();
    let in_ref = first_cells(&reference);

    let has = |set: &HashSet<String>, f: &str| set.contains(f) || set.contains(&snake(f));
    let not_in_spec: Vec<&String> = fields.iter().filter(|f| !has(&in_spec, f)).collect();
    let not_in_ref: Vec<&String> = fields.iter().filter(|f| !has(&in_ref, f)).collect();
    assert!(
        not_in_spec.is_empty(),
        "fields with no row or Appendix A entry in spec/markdown-sysml-format.md: {not_in_spec:?}"
    );
    assert!(
        not_in_ref.is_empty(),
        "fields with no row in prompts/spec/fields.md (`syscribe spec fields`): {not_in_ref:?}"
    );
}

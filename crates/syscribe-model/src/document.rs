//! The `Document` element (GH #532, `REQ-TRS-DOCGEN-002`): a document that is built from a template
//! and the model — a design description, a status report, a release note — declared in the model,
//! so `doc build` renders it and `doc build --check` says when it is out of date.

use std::collections::HashMap;

use serde_yaml::Value as Y;

use crate::element::{ElementType, RawElement};

/// The frontmatter keys of a `Document` (held in the `extra` catch-all).
pub const KEYS: &[&str] = &["template", "rendered", "artefacts", "version", "audience", "config"];

/// What a `Document` declares.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Decl {
    /// The template the document is rendered from (a source path: `repo:doc/main.tex.tmpl`).
    pub template: String,
    /// Where the rendered text is written; default: beside the template, its name without the
    /// `.tmpl`/`.j2`/`.jinja` suffix.
    pub rendered: Option<String>,
    /// The artefacts a further tool makes (`outputs:` is a PlantDef field, so the key is `artefacts:`): from the rendered text (a PDF); `--check` reports one
    /// that is missing or older than the rendered text.
    pub artefacts: Vec<String>,
    pub version: Option<String>,
    pub audience: Option<String>,
    /// The Configuration the document is for, when it is for one variant.
    pub config: Option<String>,
}

pub fn is_document(e: &RawElement) -> bool {
    e.frontmatter.element_type == Some(ElementType::Document)
}

/// The suffixes that mark a file as a template; the rendered file is the name without it.
pub const TEMPLATE_SUFFIXES: &[&str] = &[".tmpl", ".j2", ".jinja"];

/// `main.tex.tmpl` -> `main.tex`; a name with no template suffix gets `.out` appended, so the
/// rendered file never overwrites its template.
pub fn rendered_name(template_file: &str) -> String {
    for s in TEMPLATE_SUFFIXES {
        if let Some(stem) = template_file.strip_suffix(s) {
            if !stem.is_empty() {
                return stem.to_string();
            }
        }
    }
    format!("{template_file}.out")
}

/// Parse a Document's declaration. `Err` lists what is malformed.
pub fn parse(extra: &HashMap<String, Y>) -> Result<Decl, Vec<String>> {
    let mut errs = Vec::new();
    let mut d = Decl::default();
    let mut text = |k: &str, required: bool| -> Option<String> {
        match extra.get(k) {
            None => {
                if required {
                    errs.push(format!("a Document needs `{k}:` (the path of the template it is rendered from)"));
                }
                None
            }
            Some(Y::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
            Some(Y::Number(n)) if k == "version" => Some(n.to_string()),
            Some(v) => {
                errs.push(format!("`{k}:` must be a non-empty string, got {}", serde_json::to_string(v).unwrap_or_default()));
                None
            }
        }
    };
    d.template = text("template", true).unwrap_or_default();
    d.rendered = text("rendered", false);
    d.version = text("version", false);
    d.audience = text("audience", false);
    d.config = text("config", false);
    match extra.get("artefacts") {
        None => {}
        Some(Y::String(s)) if !s.trim().is_empty() => d.artefacts.push(s.trim().to_string()),
        Some(Y::Sequence(l)) if l.iter().all(|x| x.as_str().is_some_and(|s| !s.trim().is_empty())) => d.artefacts = l.iter().filter_map(|x| x.as_str()).map(|s| s.trim().to_string()).collect(),
        Some(v) => errs.push(format!("`artefacts:` must be a path or a list of paths, got {}", serde_json::to_string(v).unwrap_or_default())),
    }
    if errs.is_empty() {
        Ok(d)
    } else {
        Err(errs)
    }
}

/// A Document's declaration. `version:` is a typed frontmatter field (a profile package carries one
/// too), so it is read from there.
pub fn of(e: &RawElement) -> Result<Decl, Vec<String>> {
    let mut d = parse(&e.frontmatter.extra)?;
    if d.version.is_none() {
        d.version = e.frontmatter.version.clone();
    }
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_declaration_parses() {
        let extra: HashMap<String, Y> = serde_yaml::from_str("template: repo:doc/main.tex.tmpl\nartefacts: [repo:doc/build/dd.pdf]\nversion: 0.2\naudience: customer\n").unwrap();
        let d = parse(&extra).unwrap();
        assert_eq!((d.template.as_str(), d.artefacts.len(), d.version.as_deref(), d.audience.as_deref()), ("repo:doc/main.tex.tmpl", 1, Some("0.2"), Some("customer")));
        let bad: HashMap<String, Y> = serde_yaml::from_str("artefacts: 3\n").unwrap();
        let e = parse(&bad).unwrap_err().join("\n");
        assert!(e.contains("needs `template:`") && e.contains("`artefacts:` must be"), "{e}");
    }

    #[test]
    fn the_rendered_name_drops_the_template_suffix() {
        assert_eq!(rendered_name("main.tex.tmpl"), "main.tex");
        assert_eq!(rendered_name("notes.md.j2"), "notes.md");
        assert_eq!(rendered_name("main.tex"), "main.tex.out");
    }
}

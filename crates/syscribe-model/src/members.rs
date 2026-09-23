//! Generated package membership (ADR-SYS-PKG-001; REQ-TRS-PKG-001, GH #120).
//!
//! A package's members are defined by the directory tree — an element's
//! qualified-name parent — never by prose in its `_index.md`. This module is the
//! one place that derives them, shared by `show`, the web UI detail panel and
//! `export-html` so every surface lists exactly the same members. It is the same
//! notion `ls` uses: *direct* children only (one more `::` segment), the model
//! root's `_index.md` (empty qualified name) owning every top-level element.

use crate::element::{ElementType, RawElement};

/// The qualified name of `qname`'s parent: everything before the last `::`, or
/// `""` (the model root) for a top-level element. `None` for the root itself.
pub fn parent_qname(qname: &str) -> Option<&str> {
    if qname.is_empty() {
        return None;
    }
    Some(qname.rfind("::").map_or("", |i| &qname[..i]))
}

/// The direct members of the element whose qualified name is `qname`, sorted by
/// qualified name (deterministic). Grandchildren are excluded.
pub fn direct_members<'a>(elements: &'a [RawElement], qname: &str) -> Vec<&'a RawElement> {
    let mut out: Vec<&RawElement> = elements
        .iter()
        .filter(|e| parent_qname(&e.qualified_name) == Some(qname))
        .collect();
    out.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
    out
}

/// Whether `elem` is package-typed (`Package`/`LibraryPackage`/`Namespace`) — a
/// member list is always shown for these, even when empty (explicit empty state).
pub fn is_package(elem: &RawElement) -> bool {
    matches!(
        elem.frontmatter.element_type,
        Some(ElementType::Package) | Some(ElementType::LibraryPackage) | Some(ElementType::Namespace)
    )
}

/// A member's display label: its stable id, else its qualified name.
pub fn member_label(e: &RawElement) -> &str {
    e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn elem(qname: &str, yaml: &str) -> RawElement {
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("{qname}.md"),
            frontmatter: serde_yaml::from_str(yaml).unwrap(),
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: vec![],
        }
    }

    fn model() -> Vec<RawElement> {
        vec![
            elem("", "type: Package\nname: Root\n"),
            elem("Reqs", "type: Package\nname: Reqs\n"),
            elem("Reqs::REQ-B", "type: Requirement\nid: REQ-PK-002\n"),
            elem("Reqs::REQ-A", "type: Requirement\nid: REQ-PK-001\n"),
            elem("Reqs::Sub", "type: Package\nname: Sub\n"),
            elem("Reqs::Sub::REQ-C", "type: Requirement\nid: REQ-PK-010\n"),
            elem("ReqsOther", "type: Package\nname: ReqsOther\n"),
            elem("Empty", "type: Package\nname: Empty\n"),
        ]
    }

    #[test]
    fn direct_members_are_children_only_and_sorted() {
        let m = model();
        let q: Vec<&str> = direct_members(&m, "Reqs").iter().map(|e| e.qualified_name.as_str()).collect();
        assert_eq!(q, vec!["Reqs::REQ-A", "Reqs::REQ-B", "Reqs::Sub"]);
    }

    #[test]
    fn root_owns_top_level_elements_but_not_itself() {
        let m = model();
        let q: Vec<&str> = direct_members(&m, "").iter().map(|e| e.qualified_name.as_str()).collect();
        assert_eq!(q, vec!["Empty", "Reqs", "ReqsOther"]);
    }

    #[test]
    fn empty_package_and_leaf_have_no_members() {
        let m = model();
        assert!(direct_members(&m, "Empty").is_empty());
        assert!(direct_members(&m, "Reqs::REQ-A").is_empty());
        assert!(is_package(&m[7]) && !is_package(&m[2]));
    }

    #[test]
    fn parent_and_label() {
        assert_eq!(parent_qname("A::B::C"), Some("A::B"));
        assert_eq!(parent_qname("A"), Some(""));
        assert_eq!(parent_qname(""), None);
        let m = model();
        assert_eq!(member_label(&m[2]), "REQ-PK-002");
        assert_eq!(member_label(&m[4]), "Reqs::Sub");
    }
}

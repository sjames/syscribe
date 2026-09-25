//! §3.10 `about:` comment files (REQ-TRS-PARSE-011, GH #164), over the
//! qualification fixture `qual/fixtures/TC-TRS-PARSE-011` (read-only).

use std::path::PathBuf;

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../qual/fixtures/TC-TRS-PARSE-011")
        .join(name)
}

#[test]
fn an_about_comment_attaches_to_every_listed_element_and_is_not_an_element() {
    let els = walk_model(&fixture("notes")).expect("walk");
    assert!(
        !els.iter().any(|e| e.qualified_name.ends_with("SafetyNote") || e.qualified_name.ends_with("ColdStartNote")),
        "a comment file became an element"
    );
    for q in ["VehicleSystem::Engine", "VehicleSystem::Transmission"] {
        let e = els.iter().find(|e| e.qualified_name == q).expect(q);
        assert_eq!(e.about_notes.len(), 1, "{q} carries the SafetyNote comment");
        assert_eq!(e.about_notes[0].name, "SafetyNote");
        assert!(e.about_notes[0].body.contains("ISO 26262"));
        assert_eq!(e.about_notes[0].locale, None);
    }
    // A stable-id entry resolves; the comment's locale travels with it.
    let req = els.iter().find(|e| e.frontmatter.id.as_deref() == Some("REQ-VS-001")).expect("REQ-VS-001");
    assert_eq!(req.about_notes.len(), 1);
    assert_eq!(req.about_notes[0].locale.as_deref(), Some("en"));
    let codes: Vec<&str> = validate(&els).findings.iter().map(|f| f.code).collect();
    assert!(!codes.contains(&"E027") && !codes.contains(&"W052"), "clean model raised {codes:?}");
}

#[test]
fn unresolved_entries_and_ignored_fields_are_reported() {
    let els = walk_model(&fixture("bad")).expect("walk");
    let findings = validate(&els).findings;
    let has = |code: &str, file: &str, text: &str| {
        findings.iter().any(|f| f.code == code && f.file.ends_with(file) && f.message.contains(text))
    };
    assert!(has("E027", "PartialNote.md", "VehicleSystem::Ghost"));
    assert!(has("E027", "OrphanNote.md", "VehicleSystem::Nowhere"));
    assert!(has("W052", "StructNote.md", "supertype"));
    assert!(has("W052", "VehicleSystem/_index.md", "about:"));
    // The partial comment still attaches; the unattachable one stays an element.
    let engine = els.iter().find(|e| e.qualified_name == "VehicleSystem::Engine").expect("Engine");
    let names: Vec<&str> = engine.about_notes.iter().map(|n| n.name.as_str()).collect();
    assert_eq!(names, vec!["PartialNote", "StructNote"]);
    assert!(els.iter().any(|e| e.qualified_name == "VehicleSystem::OrphanNote"));
    // The package `_index.md` stays the package.
    assert!(els.iter().any(|e| e.qualified_name == "VehicleSystem"));
}

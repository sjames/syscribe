//! GH #185: id-identified elements are created as `<parent>/<id>.md`, packages as
//! `<dir>/_index.md`, `move` accepts an id destination, and W065 flags a file stem
//! that differs from the id.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;
use syscribe_model::mutate::mv::move_element;
use syscribe_model::mutate::{plan_create, plan_create_in, valid_move_dest, write_confined, CreateError};
use syscribe_model::resolver::Resolver;
use syscribe_model::walker::walk_model;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-cid-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Requirements/_index.md", "---\ntype: Package\nname: Requirements\n---\n");
    r
}

fn req_fields() -> serde_json::Value {
    json!({"name": "R", "status": "draft", "reqDomain": "software", "reqClass": "system"})
}

fn commit(root: &Path, plan: &syscribe_model::mutate::CreatePlan) {
    write_confined(root, &plan.rel, &plan.content).unwrap();
}

#[test]
fn parent_plus_explicit_id_writes_id_file() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let mut f = req_fields();
    f["id"] = json!("REQ-ENG-SAFE-006");
    let p = plan_create_in(&els, Some("Requirements::Safety"), None, "Requirement", Some(&f), None).unwrap();
    assert_eq!(p.rel, "Requirements/Safety/REQ-ENG-SAFE-006.md");
    assert_eq!(p.qname, "Requirements::Safety::REQ-ENG-SAFE-006");
    assert_eq!(p.id, json!("REQ-ENG-SAFE-006"));
    commit(&r, &p);
    let after = walk_model(&r).unwrap();
    let e = after.iter().find(|e| e.qualified_name == p.qname).expect("qname is the id");
    assert_eq!(e.frontmatter.id.as_deref(), Some("REQ-ENG-SAFE-006"));
}

#[test]
fn parent_without_id_auto_allocates_and_names_the_file_after_it() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let p = plan_create_in(&els, Some("Requirements"), None, "Requirement", Some(&req_fields()), None).unwrap();
    let id = p.id.as_str().unwrap().to_string();
    assert!(id.starts_with("REQ-"), "{id}");
    assert_eq!(p.rel, format!("Requirements/{id}.md"));
    assert!(p.content.contains(&format!("id: {id}")));
    commit(&r, &p);
    let els = walk_model(&r).unwrap();
    let p2 = plan_create_in(&els, Some("Requirements"), None, "Requirement", Some(&req_fields()), None).unwrap();
    assert_ne!(p2.id, p.id);
}

#[test]
fn root_parent_is_the_empty_string() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let p = plan_create_in(&els, Some(""), None, "ADR", None, None).unwrap();
    let id = p.id.as_str().unwrap();
    assert_eq!(p.rel, format!("{id}.md"));
    assert_eq!(p.qname, id);
}

#[test]
fn qname_ending_in_a_stable_id_derives_parent_and_writes_the_id() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let p = plan_create(&els, "Requirements::Safety::REQ-ENG-SAFE-006", "Requirement", Some(&req_fields()), None).unwrap();
    assert_eq!(p.rel, "Requirements/Safety/REQ-ENG-SAFE-006.md");
    assert!(p.content.contains("id: REQ-ENG-SAFE-006"), "id written into frontmatter: {}", p.content);
    assert_eq!(p.id, json!("REQ-ENG-SAFE-006"));
}

#[test]
fn qname_id_conflicting_with_fields_id_is_refused() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let mut f = req_fields();
    f["id"] = json!("REQ-OTHER-001");
    let e = plan_create(&els, "Requirements::REQ-ENG-001", "Requirement", Some(&f), None).err().unwrap();
    assert!(matches!(e, CreateError::IdConflict { .. }), "{e}");
}

#[test]
fn invalid_explicit_id_and_bad_parent_are_refused() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let mut f = req_fields();
    f["id"] = json!("not-an-id");
    assert!(matches!(
        plan_create_in(&els, Some("Requirements"), None, "Requirement", Some(&f), None).err().unwrap(),
        CreateError::InvalidId(_)
    ));
    assert!(matches!(
        plan_create_in(&els, Some("../etc"), None, "Requirement", Some(&req_fields()), None).err().unwrap(),
        CreateError::InvalidParent
    ));
    assert!(plan_create(&els, "Requirements::../REQ-XX-001", "Requirement", Some(&req_fields()), None).is_err());
}

#[test]
fn duplicate_id_or_qname_is_refused() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let p = plan_create(&els, "Requirements::REQ-ENG-001", "Requirement", Some(&req_fields()), None).unwrap();
    commit(&r, &p);
    let els = walk_model(&r).unwrap();
    assert!(matches!(
        plan_create(&els, "Requirements::REQ-ENG-001", "Requirement", Some(&req_fields()), None).err().unwrap(),
        CreateError::AlreadyExists
    ));
    let mut f = req_fields();
    f["id"] = json!("REQ-ENG-001");
    assert!(matches!(
        plan_create_in(&els, Some("Requirements::Other"), None, "Requirement", Some(&f), None).err().unwrap(),
        CreateError::AlreadyExists
    ));
}

#[test]
fn legacy_basic_qname_still_works_for_id_identified_types() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let p = plan_create(&els, "Requirements::FaultLogging", "Requirement", Some(&req_fields()), None).unwrap();
    assert_eq!(p.rel, "Requirements/FaultLogging.md");
    assert!(p.id.as_str().unwrap().starts_with("REQ-"));
}

#[test]
fn id_identified_without_target_is_refused() {
    let els = walk_model(&model()).unwrap();
    assert!(matches!(
        plan_create_in(&els, None, None, "Requirement", None, None).err().unwrap(),
        CreateError::MissingTarget
    ));
}

#[test]
fn name_identified_types_keep_qname_form_and_reject_parent_only() {
    let els = walk_model(&model()).unwrap();
    let p = plan_create(&els, "Parts::Sensor", "PartDef", None, None).unwrap();
    assert_eq!(p.rel, "Parts/Sensor.md");
    assert_eq!(p.id, serde_json::Value::Null);
    assert!(plan_create(&els, "Parts::Sensor-1", "PartDef", None, None).is_err(), "hyphen refused for name-identified");
    assert!(plan_create(&els, "Parts::REQ-ENG-001", "PartDef", None, None).is_err(), "an id-shaped stem is not a PartDef name");
    assert!(matches!(
        plan_create_in(&els, Some("Parts"), None, "PartDef", None, None).err().unwrap(),
        CreateError::NeedsQname
    ));
}

#[test]
fn package_is_written_as_index_md_and_validates() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let p = plan_create(&els, "Design", "Package", Some(&json!({"name": "Design"})), None).unwrap();
    assert_eq!(p.rel, "Design/_index.md");
    commit(&r, &p);
    assert!(!r.join("Design.md").exists());
    let after = walk_model(&r).unwrap();
    assert!(after.iter().any(|e| e.qualified_name == "Design"));
    let c = plan_create(&after, "Design::Thing", "PartDef", None, None).unwrap();
    assert_eq!(c.rel, "Design/Thing.md");
    assert!(matches!(plan_create(&after, "Design", "Package", None, None).err().unwrap(), CreateError::AlreadyExists));
}

#[test]
fn move_accepts_an_id_destination_and_rewrites_references() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let a = plan_create(&els, "Requirements::REQ_A", "Requirement", Some(&req_fields()), None).unwrap();
    commit(&r, &a);
    let els = walk_model(&r).unwrap();
    let mut f = req_fields();
    f["derivedFrom"] = json!(["Requirements::REQ_A"]);
    let b = plan_create(&els, "Requirements::REQ-BB-001", "Requirement", Some(&f), None).unwrap();
    commit(&r, &b);

    let els = walk_model(&r).unwrap();
    let resolver = Resolver::new(&els);
    move_element(&r, &els, &resolver, "Requirements::REQ_A", "Requirements::Safety::REQ-AA-001", false).unwrap();
    assert!(r.join("Requirements/Safety/REQ-AA-001.md").is_file());
    assert!(!r.join("Requirements/REQ_A.md").exists());
    let b_text = std::fs::read_to_string(r.join("Requirements/REQ-BB-001.md")).unwrap();
    assert!(b_text.contains("Requirements::Safety::REQ-AA-001"), "{b_text}");
}

#[test]
fn move_refuses_hyphenated_destination_for_names_and_packages() {
    let r = model();
    write(&r, "Parts/Sensor.md", "---\ntype: PartDef\nname: Sensor\n---\n");
    let els = walk_model(&r).unwrap();
    let resolver = Resolver::new(&els);
    assert!(move_element(&r, &els, &resolver, "Parts::Sensor", "Parts::Sens-or", true).is_err());
    assert!(move_element(&r, &els, &resolver, "Requirements", "REQ-XX-001", true).is_err());
}

#[test]
fn valid_move_dest_shapes() {
    assert!(valid_move_dest("A::B"));
    assert!(valid_move_dest("A::REQ-XX-001"));
    assert!(valid_move_dest("REQ-XX-001"));
    assert!(!valid_move_dest("REQ-XX-001::B"), "only the last segment may be an id");
    assert!(!valid_move_dest("A::../REQ-XX-001"));
    assert!(!valid_move_dest("A::b-c"));
    assert!(!valid_move_dest(""));
}

fn w065_with(root: &Path, enabled: bool) -> Vec<String> {
    let toml = root.join(".syscribe.toml");
    if enabled {
        std::fs::write(&toml, "[ids]\ncheck_file_names = true\n").unwrap();
    } else {
        let _ = std::fs::remove_file(&toml);
    }
    let els = walk_model(root).unwrap();
    let cfg = syscribe_model::config::ValidateConfig::with_model_root(root);
    syscribe_model::validator::validate_with_config(&els, &cfg)
        .findings.iter().filter(|f| f.code == "W065").map(|f| f.file.clone()).collect()
}

fn w065(root: &Path) -> Vec<String> {
    w065_with(root, true)
}

#[test]
fn w065_is_off_unless_check_file_names_is_configured() {
    let r = model();
    std::fs::write(r.join("Wrong.md"), "---\ntype: Requirement\nid: REQ-XX-050\nname: x\nstatus: approved\n---\nbody\n").unwrap();
    assert!(w065_with(&r, false).is_empty(), "opt-in: silent by default");
    assert!(!w065_with(&r, true).is_empty(), "fires once enabled");
}

#[test]
fn w065_fires_when_stem_differs_from_id_and_not_when_it_matches() {
    let r = model();
    let approved = |id: &str| {
        format!("---\nid: {id}\ntype: Requirement\nname: R\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\nThe system shall do it.\n")
    };
    write(&r, "Requirements/REQ_RL_005.md", &approved("REQ-RL-005"));
    write(&r, "Requirements/REQ-RL-006.md", &approved("REQ-RL-006"));
    let hits = w065(&r);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].ends_with("REQ_RL_005.md"));
}

#[test]
fn w065_is_draft_suppressed_and_ignores_name_identified_and_packages() {
    let r = model();
    write(
        &r,
        "Requirements/REQ_RL_005.md",
        "---\nid: REQ-RL-005\ntype: Requirement\nname: R\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n",
    );
    write(&r, "Features/Abs.md", "---\ntype: FeatureDef\nid: FEAT-ABS\nname: Abs\n---\n");
    assert!(w065(&r).is_empty(), "{:?}", w065(&r));
}

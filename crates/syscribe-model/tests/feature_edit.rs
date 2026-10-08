//! `REQ-TRS-FMED-004`: semantic feature-model edits against a real model on
//! disk — each operation, its undo, and the validity delta it produces.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

use syscribe_model::feature_edit::{analysis_delta, apply, EditOp};
use syscribe_model::feature_model::analysis_json;
use syscribe_model::feature_tree::feature_tree;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-fedit-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

/// Car (a file with a directory of children beside it) > Engine (alternative: Petrol, Electric),
/// Charger, and a Radio group in the `_index.md` layout; Electric requires Charger; one
/// configuration and one `appliesWhen` that name features.
fn model() -> PathBuf {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    write(&r, "Features/Car.md", "---\ntype: FeatureDef\nid: FEAT-CAR\nname: Car\nmandatory: true\n---\n");
    write(&r, "Features/Car/Engine.md", "---\ntype: FeatureDef\nid: FEAT-ENGINE\nname: Engine\nmandatory: true\ngroupKind: alternative\n---\n");
    write(&r, "Features/Car/Engine/Petrol.md", "---\ntype: FeatureDef\nid: FEAT-PETROL\nname: Petrol\n---\n");
    write(&r, "Features/Car/Engine/Electric.md", "---\ntype: FeatureDef\nid: FEAT-ELECTRIC\nname: Electric\nrequires: [FEAT-CHARGER]\n---\n");
    write(&r, "Features/Car/Charger.md", "---\ntype: FeatureDef\nid: FEAT-CHARGER\nname: Charger\n---\n");
    write(&r, "Features/Car/Radio/_index.md", "---\ntype: FeatureDef\nid: FEAT-RADIO\nname: Radio\ngroupKind: or\n---\n");
    write(&r, "Features/Car/Radio/FM.md", "---\ntype: FeatureDef\nid: FEAT-FM\nname: FM\n---\n");
    write(&r, "Configurations/_index.md", "---\ntype: Package\nname: Configurations\n---\n");
    write(
        &r,
        "Configurations/CONF-ONE-001.md",
        "---\ntype: Configuration\nid: CONF-ONE-001\nname: One\nstatus: draft\nfeatureModel: Features\nfeatures:\n  Features::Car: true\n  Features::Car::Engine: true\n  Features::Car::Engine::Petrol: true\n  Features::Car::Engine::Electric: false\n  Features::Car::Charger: false\n  Features::Car::Radio: false\n  Features::Car::Radio::FM: false\n---\n",
    );
    write(&r, "Parts/Charger_Unit.md", "---\ntype: PartDef\nname: Charger_Unit\nappliesWhen: Features::Car::Charger\n---\n");
    r
}

fn qnames(root: &Path) -> Vec<String> {
    feature_tree(&walk_model(root).unwrap()).into_iter().map(|f| f.qname).collect()
}

fn op(json: Value) -> EditOp {
    serde_json::from_value(json).unwrap()
}

fn snapshot(root: &Path) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| (e.path().strip_prefix(root).unwrap().to_string_lossy().to_string(), std::fs::read_to_string(e.path()).unwrap()))
        .collect();
    out.sort();
    out
}

#[test]
fn add_creates_a_feature_file_with_a_fresh_id_and_undo_deletes_it() {
    let r = model();
    let before = snapshot(&r);
    let out = apply(&r, &op(serde_json::json!({"op": "add", "parent": "Features::Car", "name": "Sunroof", "mandatory": false}))).unwrap();
    assert_eq!(out.feature.as_deref(), Some("Features::Car::Sunroof"));
    let text = read(&r, "Features/Car/Sunroof.md");
    assert!(text.contains("type: FeatureDef") && text.contains("id: FEAT-SUNROOF") && text.contains("name: Sunroof"), "{text}");
    assert!(!text.contains("mandatory"), "optional is the default: {text}");
    assert!(qnames(&r).contains(&"Features::Car::Sunroof".to_string()));
    apply(&r, &out.undo).unwrap();
    assert_eq!(snapshot(&r), before, "undo puts the model back byte for byte");
}

#[test]
fn add_with_a_group_kind_and_membership_and_by_parent_id() {
    let r = model();
    apply(&r, &op(serde_json::json!({"op": "add", "parent": "FEAT-CHARGER", "name": "Fast", "groupKind": "alternative", "mandatory": true}))).unwrap();
    let text = read(&r, "Features/Car/Charger/Fast.md");
    assert!(text.contains("mandatory: true"), "{text}");
    let tree = feature_tree(&walk_model(&r).unwrap());
    let fast = tree.iter().find(|f| f.name == "Fast").unwrap();
    assert!(fast.mandatory);
    assert_eq!(fast.parent.as_deref(), Some("Features::Car::Charger"));
}

#[test]
fn a_new_root_goes_beside_the_existing_roots_and_ids_never_collide() {
    let r = model();
    apply(&r, &op(serde_json::json!({"op": "add", "name": "Bike"}))).unwrap();
    assert!(r.join("Features/Bike.md").exists());
    // A second feature whose id would collide gets a suffix.
    apply(&r, &op(serde_json::json!({"op": "add", "parent": "Features::Car", "name": "Bike_"}))).unwrap();
    let ids: Vec<String> = walk_model(&r).unwrap().iter().filter_map(|e| e.frontmatter.id.clone()).filter(|i| i.starts_with("FEAT-BIKE")).collect();
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1], "{ids:?}");
}

#[test]
fn add_refuses_a_duplicate_a_bad_name_an_unknown_parent_and_a_bad_group() {
    let r = model();
    let err = |v: Value| apply(&r, &op(v)).unwrap_err();
    assert!(err(serde_json::json!({"op": "add", "parent": "Features::Car", "name": "Engine"})).contains("already exists"));
    assert!(err(serde_json::json!({"op": "add", "parent": "Features::Car", "name": "Bad Name"})).contains("not a valid feature name"));
    assert!(err(serde_json::json!({"op": "add", "parent": "Features::Car", "name": "A::B"})).contains("not a valid feature name"));
    assert!(err(serde_json::json!({"op": "add", "parent": "Nowhere", "name": "X"})).contains("not a feature"));
    assert!(err(serde_json::json!({"op": "add", "parent": "Features::Car", "name": "Y", "groupKind": "xor"})).contains("not optional, alternative or or"));
}

#[test]
fn set_group_changes_the_key_and_undo_restores_the_file() {
    let r = model();
    let before = read(&r, "Features/Car/Engine.md");
    let out = apply(&r, &op(serde_json::json!({"op": "setGroup", "feature": "Features::Car::Engine", "groupKind": "or"}))).unwrap();
    assert!(read(&r, "Features/Car/Engine.md").contains("groupKind: or"));
    apply(&r, &out.undo).unwrap();
    assert_eq!(read(&r, "Features/Car/Engine.md"), before);
    apply(&r, &op(serde_json::json!({"op": "setGroup", "feature": "FEAT-ENGINE", "groupKind": "optional"}))).unwrap();
    assert!(!read(&r, "Features/Car/Engine.md").contains("groupKind"), "optional is the default and is not written");
    assert!(apply(&r, &op(serde_json::json!({"op": "setGroup", "feature": "FEAT-ENGINE", "groupKind": "nope"}))).is_err());
}

#[test]
fn set_mandatory_toggles_the_flag_and_drops_the_legacy_shorthand() {
    let r = model();
    write(&r, "Features/Car/Legacy.md", "---\ntype: FeatureDef\nid: FEAT-LEGACY\nname: Legacy\ngroupKind: mandatory\n---\n");
    apply(&r, &op(serde_json::json!({"op": "setMandatory", "feature": "Features::Car::Legacy", "mandatory": false}))).unwrap();
    let t = read(&r, "Features/Car/Legacy.md");
    assert!(!t.contains("groupKind") && !t.contains("mandatory"), "{t}");
    let tree = feature_tree(&walk_model(&r).unwrap());
    assert!(!tree.iter().find(|f| f.name == "Legacy").unwrap().mandatory);
    apply(&r, &op(serde_json::json!({"op": "setMandatory", "feature": "Features::Car::Charger", "mandatory": true}))).unwrap();
    assert!(read(&r, "Features/Car/Charger.md").contains("mandatory: true"));
}

#[test]
fn constraints_are_added_by_stable_id_refused_when_repeated_and_removed() {
    let r = model();
    let out = apply(&r, &op(serde_json::json!({"op": "addConstraint", "feature": "Features::Car::Engine::Petrol", "kind": "excludes", "target": "Features::Car::Engine::Electric"}))).unwrap();
    assert!(read(&r, "Features/Car/Engine/Petrol.md").contains("FEAT-ELECTRIC"), "spelled by id so a rename leaves it intact");
    let again = apply(&r, &op(serde_json::json!({"op": "addConstraint", "feature": "FEAT-PETROL", "kind": "excludes", "target": "FEAT-ELECTRIC"}))).unwrap_err();
    assert!(again.contains("already"), "{again}");
    assert!(apply(&r, &op(serde_json::json!({"op": "addConstraint", "feature": "FEAT-PETROL", "kind": "excludes", "target": "FEAT-PETROL"}))).unwrap_err().contains("itself"));
    assert!(apply(&r, &op(serde_json::json!({"op": "addConstraint", "feature": "FEAT-PETROL", "kind": "implies", "target": "FEAT-CHARGER"}))).is_err());
    apply(&r, &out.undo).unwrap();
    assert!(!read(&r, "Features/Car/Engine/Petrol.md").contains("excludes"));
    // Remove the existing requires, spelled in the file as an id, by qualified name.
    apply(&r, &op(serde_json::json!({"op": "removeConstraint", "feature": "FEAT-ELECTRIC", "kind": "requires", "target": "Features::Car::Charger"}))).unwrap();
    assert!(!read(&r, "Features/Car/Engine/Electric.md").contains("requires"), "{}", read(&r, "Features/Car/Engine/Electric.md"));
    assert!(apply(&r, &op(serde_json::json!({"op": "removeConstraint", "feature": "FEAT-ELECTRIC", "kind": "requires", "target": "FEAT-CHARGER"}))).unwrap_err().contains("does not"));
}

#[test]
fn remove_a_leaf_cleans_constraints_and_configuration_keys_and_undo_restores_everything() {
    let r = model();
    let before = snapshot(&r);
    let out = apply(&r, &op(serde_json::json!({"op": "remove", "feature": "Features::Car::Charger"}))).unwrap();
    assert!(!r.join("Features/Car/Charger.md").exists());
    assert!(!read(&r, "Features/Car/Engine/Electric.md").contains("requires"), "the requires: that named it goes with it");
    assert!(!read(&r, "Configurations/CONF-ONE-001.md").contains("Charger"), "and so does the configuration's choice");
    assert!(!qnames(&r).contains(&"Features::Car::Charger".to_string()));
    apply(&r, &out.undo).unwrap();
    assert_eq!(snapshot(&r), before, "undo restores the feature, the constraint and the choice");
}

#[test]
fn remove_a_feature_with_children_needs_the_subtree_flag() {
    let r = model();
    let before = snapshot(&r);
    let err = apply(&r, &op(serde_json::json!({"op": "remove", "feature": "Features::Car::Engine"}))).unwrap_err();
    assert!(err.contains("2 feature(s) below it"), "{err}");
    assert_eq!(snapshot(&r), before, "a refusal changes nothing");
    let out = apply(&r, &op(serde_json::json!({"op": "remove", "feature": "FEAT-ENGINE", "subtree": true}))).unwrap();
    let left = qnames(&r);
    assert!(!left.iter().any(|q| q.contains("Engine")), "{left:?}");
    assert!(!read(&r, "Configurations/CONF-ONE-001.md").contains("Engine"));
    apply(&r, &out.undo).unwrap();
    assert_eq!(snapshot(&r), before);
}

#[test]
fn rename_changes_the_qualified_name_rewrites_references_and_the_label_and_undo_renames_back() {
    let r = model();
    let out = apply(&r, &op(serde_json::json!({"op": "rename", "feature": "Features::Car::Charger", "name": "Plug"}))).unwrap();
    assert_eq!(out.feature.as_deref(), Some("Features::Car::Plug"));
    assert!(r.join("Features/Car/Plug.md").exists() && !r.join("Features/Car/Charger.md").exists());
    assert!(read(&r, "Features/Car/Plug.md").contains("name: Plug"), "the label follows the file name");
    assert!(read(&r, "Configurations/CONF-ONE-001.md").contains("Features::Car::Plug"), "configuration keys are rewritten");
    assert!(read(&r, "Parts/Charger_Unit.md").contains("Features::Car::Plug"), "appliesWhen is rewritten");
    assert!(read(&r, "Features/Car/Engine/Electric.md").contains("FEAT-CHARGER"), "an id reference needs no rewrite");
    apply(&r, &out.undo).unwrap();
    assert!(r.join("Features/Car/Charger.md").exists());
    assert!(read(&r, "Configurations/CONF-ONE-001.md").contains("Features::Car::Charger"));
}

#[test]
fn renaming_a_feature_that_is_a_file_with_a_directory_of_children_moves_both() {
    let r = model();
    apply(&r, &op(serde_json::json!({"op": "rename", "feature": "Features::Car", "name": "Vehicle"}))).unwrap();
    assert!(r.join("Features/Vehicle.md").exists(), "the file");
    assert!(r.join("Features/Vehicle/Engine.md").exists(), "and its children");
    assert!(!r.join("Features/Car").exists() && !r.join("Features/Car.md").exists());
    let q = qnames(&r);
    assert!(q.contains(&"Features::Vehicle::Engine::Petrol".to_string()), "{q:?}");
    assert!(read(&r, "Configurations/CONF-ONE-001.md").contains("Features::Vehicle::Engine::Petrol"));
}

#[test]
fn renaming_a_group_in_the_index_layout_moves_its_directory() {
    let r = model();
    apply(&r, &op(serde_json::json!({"op": "rename", "feature": "Features::Car::Radio", "name": "Audio"}))).unwrap();
    assert!(r.join("Features/Car/Audio/_index.md").exists() && r.join("Features/Car/Audio/FM.md").exists());
    assert!(read(&r, "Features/Car/Audio/_index.md").contains("name: Audio"));
}

#[test]
fn move_reparents_a_feature_with_its_subtree_and_undo_moves_it_back_refusing_a_cycle() {
    let r = model();
    let out = apply(&r, &op(serde_json::json!({"op": "move", "feature": "Features::Car::Charger", "newParent": "Features::Car::Radio"}))).unwrap();
    assert_eq!(out.feature.as_deref(), Some("Features::Car::Radio::Charger"));
    let tree = feature_tree(&walk_model(&r).unwrap());
    assert_eq!(tree.iter().find(|f| f.name == "Charger").unwrap().parent.as_deref(), Some("Features::Car::Radio"));
    assert!(read(&r, "Configurations/CONF-ONE-001.md").contains("Features::Car::Radio::Charger"));
    apply(&r, &out.undo).unwrap();
    assert!(r.join("Features/Car/Charger.md").exists());
    let err = apply(&r, &op(serde_json::json!({"op": "move", "feature": "Features::Car::Engine", "newParent": "Features::Car::Engine::Petrol"}))).unwrap_err();
    assert!(err.contains("inside its own subtree"), "{err}");
    let same = apply(&r, &op(serde_json::json!({"op": "move", "feature": "Features::Car::Charger", "newParent": "Features::Car"}))).unwrap_err();
    assert!(same.contains("already there"), "{same}");
}

#[test]
fn a_root_can_be_made_a_child_and_a_child_a_root() {
    let r = model();
    apply(&r, &op(serde_json::json!({"op": "add", "name": "Bike"}))).unwrap();
    apply(&r, &op(serde_json::json!({"op": "move", "feature": "Features::Bike", "newParent": "Features::Car"}))).unwrap();
    assert!(r.join("Features/Car/Bike.md").exists());
    apply(&r, &op(serde_json::json!({"op": "move", "feature": "Features::Car::Bike"}))).unwrap();
    assert!(r.join("Features/Bike.md").exists(), "a move without a parent makes it a root beside the others");
}

/// A sheet: Car > Engine (alternative: Petrol, Electric), Charger, one entry with an explicit id and the
/// rest with derived ids; Electric requires Charger through `crossTreeConstraints:` (relative paths);
/// a configuration, a binding and an `appliesWhen:` that name sheet features.
fn sheet_model() -> PathBuf {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &r,
        "Features.md",
        "---\ntype: FeatureModel\nname: Features\nfeatureTree:\n  - name: Car\n    id: FEAT-CAR\n    mandatory: true\n  - name: Car.Engine\n    mandatory: true\n    groupKind: alternative\n  - name: Car.Engine.Petrol\n  - name: Car.Engine.Electric\n    parameters:\n      - { name: kw, type: ScalarValues::Real, range: \"50..=300\" }\n  - name: Car.Charger\ncrossTreeConstraints:\n  - feature: Car.Engine.Electric\n    requires: [Car.Charger]\n---\n",
    );
    write(
        &r,
        "Configurations/CONF-S-001.md",
        "---\ntype: Configuration\nid: CONF-S-001\nname: S\nstatus: draft\nfeatureModel: Features\nfeatures:\n  Features::Car: true\n  Features::Car::Engine: true\n  Features::Car::Engine::Petrol: true\n  Features::Car::Engine::Electric: false\n  Features::Car::Charger: false\nparameterBindings:\n  Features::Car::Engine::Electric.kw: 120\n---\n",
    );
    write(&r, "Parts/Plug.md", "---\ntype: PartDef\nname: Plug\nappliesWhen: Features::Car::Charger\n---\n");
    r
}

#[test]
fn a_sheet_entry_takes_group_and_membership_edits() {
    let r = sheet_model();
    let before = snapshot(&r);
    let out = apply(&r, &op(serde_json::json!({"op": "setGroup", "feature": "Features::Car::Engine", "groupKind": "or"}))).unwrap();
    let second = apply(&r, &op(serde_json::json!({"op": "setMandatory", "feature": "Features::Car::Charger", "mandatory": true}))).unwrap();
    let tree = feature_tree(&walk_model(&r).unwrap());
    assert_eq!(tree.iter().find(|f| f.name == "Engine").unwrap().group_kind.as_str(), "or");
    assert!(tree.iter().find(|f| f.name == "Charger").unwrap().mandatory);
    // The undo stack is last-in-first-out: undoing the first edit under the second would be refused.
    assert!(apply(&r, &out.undo).unwrap_err().contains("changed since the edit"));
    apply(&r, &second.undo).unwrap();
    apply(&r, &out.undo).unwrap();
    assert_eq!(tree_of(&r, "Engine"), "alternative");
    let _ = before;
}

fn tree_of(r: &Path, name: &str) -> String {
    feature_tree(&walk_model(r).unwrap()).iter().find(|f| f.name == name).unwrap().group_kind.as_str().to_string()
}

#[test]
fn renaming_a_sheet_entry_renames_its_subtree_keeps_ids_and_rewrites_every_reference() {
    let r = sheet_model();
    let before = snapshot(&r);
    let out = apply(&r, &op(serde_json::json!({"op": "rename", "feature": "Features::Car::Engine", "name": "Motor"}))).unwrap();
    assert_eq!(out.feature.as_deref(), Some("Features::Car::Motor"));
    let sheet = read(&r, "Features.md");
    assert!(sheet.contains("Car.Motor") && sheet.contains("Car.Motor.Petrol") && sheet.contains("Car.Motor.Electric"), "{sheet}");
    assert!(!sheet.contains("Car.Engine"), "{sheet}");
    assert!(sheet.contains("FEAT-CAR-ENGINE"), "the derived id is written down so it survives the rename: {sheet}");
    let tree = feature_tree(&walk_model(&r).unwrap());
    let motor = tree.iter().find(|f| f.name == "Motor").unwrap();
    assert_eq!(motor.qname, "Features::Car::Motor");
    assert_eq!(motor.id.as_deref(), Some("FEAT-CAR-ENGINE"));
    assert_eq!(tree.iter().find(|f| f.name == "Electric").unwrap().parent.as_deref(), Some("Features::Car::Motor"));
    assert!(read(&r, "Configurations/CONF-S-001.md").contains("Features::Car::Motor::Petrol"));
    assert!(read(&r, "Configurations/CONF-S-001.md").contains("Features::Car::Motor::Electric.kw"), "bindings follow too");
    assert!(tree.iter().find(|f| f.name == "Electric").unwrap().requires.contains(&"Features::Car::Charger".to_string()), "the constraint survives");
    apply(&r, &out.undo).unwrap();
    assert_eq!(snapshot(&r), before, "undo is byte for byte");
}

#[test]
fn a_sheet_entry_moves_within_its_sheet_with_its_subtree_and_constraints_that_name_it_by_path() {
    let r = sheet_model();
    let out = apply(&r, &op(serde_json::json!({"op": "move", "feature": "Features::Car::Charger", "newParent": "Features::Car::Engine"}))).unwrap();
    assert_eq!(out.feature.as_deref(), Some("Features::Car::Engine::Charger"));
    let sheet = read(&r, "Features.md");
    assert!(sheet.contains("Car.Engine.Charger"), "{sheet}");
    assert!(sheet.contains("Car.Engine.Charger") && !sheet.contains("- Car.Charger"), "the relative path in crossTreeConstraints followed: {sheet}");
    let tree = feature_tree(&walk_model(&r).unwrap());
    assert_eq!(tree.iter().find(|f| f.name == "Charger").unwrap().parent.as_deref(), Some("Features::Car::Engine"));
    assert!(tree.iter().find(|f| f.name == "Electric").unwrap().requires.contains(&"Features::Car::Engine::Charger".to_string()));
    assert!(read(&r, "Parts/Plug.md").contains("Features::Car::Engine::Charger"));
    // And back out to a root of the sheet.
    apply(&r, &op(serde_json::json!({"op": "move", "feature": "Features::Car::Engine::Charger"}))).unwrap();
    assert!(read(&r, "Features.md").contains("name: Charger"));
    assert!(walk_model(&r).unwrap().iter().any(|e| e.qualified_name == "Features::Charger"));
}

#[test]
fn a_sheet_entry_cannot_leave_its_sheet_or_collide_and_says_why() {
    let r = sheet_model();
    write(&r, "Other/Wheels.md", "---\ntype: FeatureDef\nid: FEAT-WHEELS\nname: Wheels\n---\n");
    let err = apply(&r, &op(serde_json::json!({"op": "move", "feature": "Features::Car::Charger", "newParent": "Other::Wheels"}))).unwrap_err();
    assert!(err.contains("within that sheet"), "{err}");
    let err = apply(&r, &op(serde_json::json!({"op": "rename", "feature": "Features::Car::Charger", "name": "Engine"}))).unwrap_err();
    assert!(err.contains("already exists"), "{err}");
    // A file-based child of a sheet entry blocks renaming that entry: the files would be left behind.
    write(&r, "Features/Car/Engine/Hybrid.md", "---\ntype: FeatureDef\nid: FEAT-HYBRID\nname: Hybrid\n---\n");
    let err = apply(&r, &op(serde_json::json!({"op": "rename", "feature": "Features::Car::Engine", "name": "Motor"}))).unwrap_err();
    assert!(err.contains("not an entry") && err.contains("Hybrid"), "{err}");
}

#[test]
fn sheet_constraints_are_added_inline_and_removed_from_the_cross_tree_list() {
    let r = sheet_model();
    let before = snapshot(&r);
    let added = apply(&r, &op(serde_json::json!({"op": "addConstraint", "feature": "Features::Car::Engine::Petrol", "kind": "excludes", "target": "Features::Car::Engine::Electric"}))).unwrap();
    let tree = feature_tree(&walk_model(&r).unwrap());
    assert!(tree.iter().find(|f| f.name == "Petrol").unwrap().excludes.contains(&"Features::Car::Engine::Electric".to_string()));
    let dup = apply(&r, &op(serde_json::json!({"op": "addConstraint", "feature": "Features::Car::Engine::Electric", "kind": "requires", "target": "Features::Car::Charger"}))).unwrap_err();
    assert!(dup.contains("already"), "the cross-tree list's constraint counts: {dup}");
    apply(&r, &added.undo).unwrap();
    assert_eq!(snapshot(&r), before);
    let removed = apply(&r, &op(serde_json::json!({"op": "removeConstraint", "feature": "Features::Car::Engine::Electric", "kind": "requires", "target": "Features::Car::Charger"}))).unwrap();
    assert!(!read(&r, "Features.md").contains("crossTreeConstraints"), "the emptied list is dropped: {}", read(&r, "Features.md"));
    let tree = feature_tree(&walk_model(&r).unwrap());
    assert!(tree.iter().find(|f| f.name == "Electric").unwrap().requires.is_empty());
    apply(&r, &removed.undo).unwrap();
    assert_eq!(snapshot(&r), before);
}

#[test]
fn removing_sheet_entries_takes_the_subtree_the_cross_tree_constraint_and_the_choices() {
    let r = sheet_model();
    let before = snapshot(&r);
    let err = apply(&r, &op(serde_json::json!({"op": "remove", "feature": "Features::Car::Engine"}))).unwrap_err();
    assert!(err.contains("2 feature(s) below it"), "{err}");
    let out = apply(&r, &op(serde_json::json!({"op": "remove", "feature": "Features::Car::Engine", "subtree": true}))).unwrap();
    let names: Vec<String> = feature_tree(&walk_model(&r).unwrap()).into_iter().map(|f| f.qname).collect();
    assert_eq!(names, vec!["Features::Car", "Features::Car::Charger"]);
    let sheet = read(&r, "Features.md");
    assert!(!sheet.contains("Engine") && !sheet.contains("crossTreeConstraints"), "{sheet}");
    let conf = read(&r, "Configurations/CONF-S-001.md");
    assert!(!conf.contains("Engine") && !conf.contains("parameterBindings") || !conf.contains("Electric"), "{conf}");
    apply(&r, &out.undo).unwrap();
    assert_eq!(snapshot(&r), before, "undo restores the sheet and the configuration");
    // A single entry, and a constraint owned by it goes with it.
    apply(&r, &op(serde_json::json!({"op": "remove", "feature": "Features::Car::Charger"}))).unwrap();
    assert!(!read(&r, "Features.md").contains("crossTreeConstraints"), "{}", read(&r, "Features.md"));
}

#[test]
fn parameters_are_added_replaced_and_removed_with_their_bindings_in_either_layout() {
    // Sheet entry: Electric already has `kw`, bound to 120 by the configuration.
    let r = sheet_model();
    let before = snapshot(&r);
    let out = apply(&r, &op(serde_json::json!({"op": "setParameter", "feature": "Features::Car::Engine::Electric", "parameter": {"name": "kw", "type": "ScalarValues::Real", "range": "50..=400", "isRequired": true}}))).unwrap();
    let sheet = read(&r, "Features.md");
    assert!(sheet.contains("50..=400") && !sheet.contains("50..=300") && sheet.matches("name: kw").count() == 1, "replaced, not duplicated: {sheet}");
    let second = apply(&r, &op(serde_json::json!({"op": "setParameter", "feature": "Features::Car::Engine::Electric", "parameter": {"name": "volts", "type": "ScalarValues::Integer", "default": 400}}))).unwrap();
    assert_eq!(walk_model(&r).unwrap().iter().find(|e| e.qualified_name == "Features::Car::Engine::Electric").unwrap().frontmatter.parameters.as_ref().unwrap().len(), 2);
    apply(&r, &second.undo).unwrap();
    apply(&r, &out.undo).unwrap();
    // A parameter some configuration binds cannot be removed: the binding goes first.
    let snap = snapshot(&r);
    let err = apply(&r, &op(serde_json::json!({"op": "removeParameter", "feature": "Features::Car::Engine::Electric", "name": "kw"}))).unwrap_err();
    assert!(err.contains("bound by CONF-S-001") && err.contains("removeBinding"), "{err}");
    assert_eq!(snapshot(&r), snap, "a refusal changes nothing");
    let unbind = apply(&r, &op(serde_json::json!({"op": "removeBinding", "configuration": "CONF-S-001", "feature": "FEAT-CAR-ENGINE-ELECTRIC", "name": "kw"}))).unwrap();
    assert!(!read(&r, "Configurations/CONF-S-001.md").contains("parameterBindings"), "{}", read(&r, "Configurations/CONF-S-001.md"));
    assert!(apply(&r, &op(serde_json::json!({"op": "removeBinding", "configuration": "CONF-S-001", "feature": "Features::Car::Engine::Electric", "name": "kw"}))).unwrap_err().contains("binds no parameters"));
    let rm = apply(&r, &op(serde_json::json!({"op": "removeParameter", "feature": "Features::Car::Engine::Electric", "name": "kw"}))).unwrap();
    assert!(!read(&r, "Features.md").contains("name: kw"));
    apply(&r, &rm.undo).unwrap();
    apply(&r, &unbind.undo).unwrap();
    assert!(read(&r, "Configurations/CONF-S-001.md").contains("Electric.kw: 120"));
    assert!(apply(&r, &op(serde_json::json!({"op": "removeBinding", "configuration": "Nope", "feature": "Features::Car", "name": "kw"}))).unwrap_err().contains("not a Configuration"));
    let _ = before;
    // Per-file layout.
    let r = model();
    apply(&r, &op(serde_json::json!({"op": "setParameter", "feature": "FEAT-CHARGER", "parameter": {"name": "amps", "type": "ScalarValues::Real", "unit": "A"}}))).unwrap();
    assert!(read(&r, "Features/Car/Charger.md").contains("name: amps"));
    let err = apply(&r, &op(serde_json::json!({"op": "removeParameter", "feature": "FEAT-CHARGER", "name": "nope"}))).unwrap_err();
    assert!(err.contains("no parameter 'nope'"), "{err}");
    let err = apply(&r, &op(serde_json::json!({"op": "setParameter", "feature": "FEAT-CHARGER", "parameter": {"name": "bad name"}}))).unwrap_err();
    assert!(err.contains("not a valid parameter name"), "{err}");
    let err = apply(&r, &op(serde_json::json!({"op": "setParameter", "feature": "FEAT-CHARGER", "parameter": "amps"}))).unwrap_err();
    assert!(err.contains("object"), "{err}");
}

#[test]
fn removing_a_feature_also_removes_the_bindings_of_its_parameters() {
    let r = sheet_model();
    apply(&r, &op(serde_json::json!({"op": "remove", "feature": "Features::Car::Engine::Electric"}))).unwrap();
    assert!(!read(&r, "Configurations/CONF-S-001.md").contains("kw"), "{}", read(&r, "Configurations/CONF-S-001.md"));
}

/// Drop the `Engine` entry from every configuration, as the editor's user must before marking it abstract.
fn unname_engine(r: &Path) {
    for entry in std::fs::read_dir(r.join("Configurations")).unwrap().flatten() {
        let text = std::fs::read_to_string(entry.path()).unwrap();
        let kept: Vec<&str> = text.lines().filter(|l| !l.trim_start().starts_with("Features::Car::Engine:")).collect();
        std::fs::write(entry.path(), kept.join("\n") + "\n").unwrap();
    }
}

#[test]
fn a_feature_a_configuration_names_cannot_be_made_abstract_until_the_entry_is_removed() {
    let r = model();
    let before = snapshot(&r);
    let e = apply(&r, &op(serde_json::json!({"op": "setAbstract", "feature": "Features::Car::Engine", "isAbstract": true}))).unwrap_err();
    assert!(e.contains("CONF-ONE-001") && e.contains("E238"), "{e}");
    assert_eq!(snapshot(&r), before, "a refusal writes nothing");
    unname_engine(&r);
    apply(&r, &op(serde_json::json!({"op": "setAbstract", "feature": "Features::Car::Engine", "isAbstract": true}))).unwrap();
}

#[test]
fn a_feature_is_marked_abstract_and_unmarked_in_either_layout_and_stays_selectable() {
    let r = model();
    unname_engine(&r);
    let before = snapshot(&r);
    let out = apply(&r, &op(serde_json::json!({"op": "setAbstract", "feature": "Features::Car::Engine", "isAbstract": true}))).unwrap();
    assert!(read(&r, "Features/Car/Engine.md").contains("isAbstract: true"));
    let tree = feature_tree(&walk_model(&r).unwrap());
    assert!(tree.iter().find(|f| f.name == "Engine").unwrap().is_abstract);
    // Abstract changes the picture, not the product space: the analysis is the same.
    let a = analysis_json(&walk_model(&r).unwrap());
    assert_eq!(a["void"], false);
    assert_eq!(a["features"]["Features::Car::Engine"]["state"], "core", "an abstract mandatory feature is still core: {a}");
    apply(&r, &out.undo).unwrap();
    assert_eq!(snapshot(&r), before);
    apply(&r, &op(serde_json::json!({"op": "setAbstract", "feature": "FEAT-ENGINE", "isAbstract": false}))).unwrap();
    assert!(!read(&r, "Features/Car/Engine.md").contains("isAbstract"), "false is the default and is not written");
    // A sheet entry.
    let r = sheet_model();
    unname_engine(&r);
    let out = apply(&r, &op(serde_json::json!({"op": "setAbstract", "feature": "Features::Car::Engine", "isAbstract": true}))).unwrap();
    assert!(feature_tree(&walk_model(&r).unwrap()).iter().find(|f| f.name == "Engine").unwrap().is_abstract);
    apply(&r, &out.undo).unwrap();
    assert!(!feature_tree(&walk_model(&r).unwrap()).iter().find(|f| f.name == "Engine").unwrap().is_abstract);
}

#[test]
fn the_validity_delta_names_what_an_edit_made_dead_and_what_undoing_it_fixed() {
    let r = model();
    let before = analysis_json(&walk_model(&r).unwrap());
    // Petrol excludes Engine's only other choice and Engine is mandatory: excluding Electric AND
    // making Petrol exclude Engine would void; use a clearer one: Charger excludes Electric makes Electric dead.
    let out = apply(&r, &op(serde_json::json!({"op": "addConstraint", "feature": "FEAT-CHARGER", "kind": "excludes", "target": "FEAT-ELECTRIC"}))).unwrap();
    let after = analysis_json(&walk_model(&r).unwrap());
    let d = analysis_delta(&before, &after);
    assert_eq!(d["worsens"], true, "{d}");
    assert_eq!(d["newDead"], serde_json::json!(["Features::Car::Engine::Electric"]), "{d}");
    assert_eq!(d["becameVoid"], false);
    apply(&r, &out.undo).unwrap();
    let healed = analysis_delta(&after, &analysis_json(&walk_model(&r).unwrap()));
    assert_eq!(healed["worsens"], false);
    assert_eq!(healed["resolvedDead"], serde_json::json!(["Features::Car::Engine::Electric"]));
}

#[test]
fn the_validity_delta_reports_a_model_made_void_and_an_invalidated_configuration() {
    let r = model();
    let before = analysis_json(&walk_model(&r).unwrap());
    // Car is mandatory and so is Engine; make Car exclude Engine: void.
    apply(&r, &op(serde_json::json!({"op": "addConstraint", "feature": "FEAT-CAR", "kind": "excludes", "target": "FEAT-ENGINE"}))).unwrap();
    let after = analysis_json(&walk_model(&r).unwrap());
    let d = analysis_delta(&before, &after);
    assert_eq!(d["becameVoid"], true, "{d}");
    assert_eq!(d["worsens"], true);
    assert!(d["newDead"].as_array().unwrap().is_empty(), "a void model is reported once, not as every feature dead");
    assert!(!d["conflicts"].as_array().unwrap().is_empty());

    let r = model();
    let before = analysis_json(&walk_model(&r).unwrap());
    assert_eq!(before["invalidConfigurations"], serde_json::json!([]), "the fixture configuration starts valid");
    // Making Radio mandatory invalidates a configuration that leaves it out.
    apply(&r, &op(serde_json::json!({"op": "setMandatory", "feature": "Features::Car::Radio", "mandatory": true}))).unwrap();
    let d = analysis_delta(&before, &analysis_json(&walk_model(&r).unwrap()));
    assert_eq!(d["newInvalidConfigurations"], serde_json::json!(["CONF-ONE-001"]), "{d}");
    assert_eq!(d["worsens"], true);
}

#[test]
fn an_edit_that_changes_nothing_for_validity_does_not_worsen_it() {
    let r = model();
    let before = analysis_json(&walk_model(&r).unwrap());
    apply(&r, &op(serde_json::json!({"op": "add", "parent": "Features::Car", "name": "Sunroof"}))).unwrap();
    let d = analysis_delta(&before, &analysis_json(&walk_model(&r).unwrap()));
    assert_eq!(d["worsens"], false, "{d}");
    assert_eq!(d["countsAfter"]["features"], d["countsBefore"]["features"].as_u64().unwrap() + 1);
}

#[test]
fn an_undo_is_refused_when_the_file_was_edited_elsewhere_in_between() {
    let r = model();
    let out = apply(&r, &op(serde_json::json!({"op": "setMandatory", "feature": "Features::Car::Radio", "mandatory": true}))).unwrap();
    let file = r.join("Features/Car/Radio/_index.md");
    let edited = format!("{}\n# a note somebody typed meanwhile\n", std::fs::read_to_string(&file).unwrap());
    std::fs::write(&file, &edited).unwrap();
    let e = apply(&r, &out.undo).unwrap_err();
    assert!(e.contains("changed since the edit"), "{e}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), edited, "the other change survives a refused undo");
    // Untouched, the same undo applies, and its own undo (a redo) is conditional in turn.
    std::fs::write(&file, std::fs::read_to_string(&file).unwrap().replace("\n# a note somebody typed meanwhile\n", "")).unwrap();
    let redo = apply(&r, &out.undo).unwrap().undo;
    std::fs::write(&file, "---\ntype: FeatureDef\nid: FEAT-RADIO\nname: Radio\n---\nchanged\n").unwrap();
    assert!(apply(&r, &redo).unwrap_err().contains("changed since the edit"));
}

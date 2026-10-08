//! `move_element` on a plain `Name.md` that has a `Name/` directory of children beside it must
//! relocate both, and leave nothing under the old name.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::mutate::mv::move_element;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-mvsib-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Engine.md", "---\ntype: PartDef\nname: Engine\n---\n");
    write(&r, "Engine/Piston.md", "---\ntype: PartDef\nname: Piston\n---\n");
    write(&r, "Car.md", "---\ntype: PartDef\nname: Car\nsupertype: Engine\n---\n");
    r
}

#[test]
fn a_file_with_a_directory_beside_it_moves_as_one() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let resolver = Resolver::new(&els);
    move_element(&r, &els, &resolver, "Engine", "Motor", false).unwrap();
    assert!(r.join("Motor.md").is_file(), "the element's own file moved");
    assert!(r.join("Motor/Piston.md").is_file(), "and its children");
    assert!(!r.join("Engine.md").exists() && !r.join("Engine").exists(), "nothing is left under the old name");
    let after = walk_model(&r).unwrap();
    assert!(!validate(&after).findings.iter().any(|f| f.code == "E110"), "the rewritten supertype resolves");
    assert!(std::fs::read_to_string(r.join("Car.md")).unwrap().contains("supertype: Motor"));
}

#[test]
fn a_dry_run_moves_nothing() {
    let r = model();
    let els = walk_model(&r).unwrap();
    let resolver = Resolver::new(&els);
    move_element(&r, &els, &resolver, "Engine", "Motor", true).unwrap();
    assert!(r.join("Engine.md").is_file() && r.join("Engine/Piston.md").is_file());
}

#[test]
fn a_move_rewrites_a_qualified_name_inside_an_applieswhen_expression() {
    let r = model();
    write(&r, "F/_index.md", "---\ntype: Package\nname: F\n---\n");
    write(&r, "F/Alpha.md", "---\ntype: FeatureDef\nid: FEAT-ALPHA\nname: Alpha\n---\n");
    write(&r, "F/B.md", "---\ntype: FeatureDef\nid: FEAT-BEE\nname: B\n---\n");
    write(&r, "Gated.md", "---\ntype: PartDef\nname: Gated\nappliesWhen: \"F::Alpha and not F::B\"\n---\n");
    let els = walk_model(&r).unwrap();
    let resolver = Resolver::new(&els);
    move_element(&r, &els, &resolver, "F::Alpha", "F::Beta", false).unwrap();
    let gated = std::fs::read_to_string(r.join("Gated.md")).unwrap();
    assert!(gated.contains("F::Beta and not F::B"), "{gated}");
    assert!(!validate(&walk_model(&r).unwrap()).findings.iter().any(|f| f.code == "E209"));
}

#[test]
fn a_label_that_equals_the_moved_name_is_not_rewritten() {
    let r = model();
    write(&r, "Car.md", "---\ntype: PartDef\nname: Car\nsupertype: Engine\nfeatures:\n  - name: Engine\n    typedBy: Engine\n---\n");
    let els = walk_model(&r).unwrap();
    let resolver = Resolver::new(&els);
    move_element(&r, &els, &resolver, "Engine", "Motor", false).unwrap();
    let car = std::fs::read_to_string(r.join("Car.md")).unwrap();
    assert!(car.contains("supertype: Motor") && car.contains("typedBy: Motor"), "{car}");
    assert!(car.contains("- name: Engine"), "the usage's own name stays: {car}");
}

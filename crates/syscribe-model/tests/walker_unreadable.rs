//! An unreadable directory in the model is an error, not a silent gap.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use syscribe_model::walker::walk_model;

#[test]
fn unreadable_subdirectory_fails_the_walk() {
    if unsafe { libc::geteuid() } == 0 {
        return; // permission bits are ignored for root
    }
    let root = std::env::temp_dir().join(format!("syscribe-unreadable-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("Sub")).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\n---\n").unwrap();
    std::fs::write(root.join("Sub/A.md"), "---\ntype: PartDef\n---\nx\n").unwrap();
    std::fs::set_permissions(root.join("Sub"), std::fs::Permissions::from_mode(0o000)).unwrap();
    let r = walk_model(&root);
    std::fs::set_permissions(root.join("Sub"), std::fs::Permissions::from_mode(0o755)).unwrap();
    let ok = walk_model(&root).unwrap();
    let _ = std::fs::remove_dir_all(&root);
    assert!(r.is_err(), "an unreadable directory must fail the walk");
    assert!(ok.iter().any(|e| e.qualified_name == "Sub::A"));
}

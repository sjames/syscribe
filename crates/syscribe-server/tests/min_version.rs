//! GH #496 (`REQ-TRS-MINVER-001`): `syscribe-server` refuses a model that needs a newer syscribe.

use std::process::Command;

#[test]
fn the_server_refuses_a_model_that_needs_a_newer_syscribe() {
    let root = std::env::temp_dir().join(format!("syscribe-server-mv-{}", std::process::id())).join("model");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join(".syscribe.toml"), "[syscribe]\nminVersion = \"999.0.0\"\n").unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe-server")).arg("-m").arg(&root).output().unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(err.contains("999.0.0") && err.contains(&format!("this is syscribe {}", env!("CARGO_PKG_VERSION"))), "{err}");
}

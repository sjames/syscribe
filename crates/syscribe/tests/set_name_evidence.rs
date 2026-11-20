//! GH #502 (`REQ-TRS-SETNAME-001`): `set <id> name=` and `evidence.add path=repo:…`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// A repository whose model lives in `<repo>/model/`, with a source file outside the model at `<repo>/src/real.rs`.
fn repo() -> (PathBuf, PathBuf) {
    static N: AtomicU64 = AtomicU64::new(0);
    let base = std::env::temp_dir().join(format!("syscribe-setname-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let model = base.join("model");
    let w = |rel: &str, c: &str| {
        let p = base.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("model/.syscribe.toml", "repo_root = \"..\"\n");
    w("src/real.rs", "fn main() {}\n");
    w("model/Docs/guide.txt", "Guide.\n");
    w("model/PI-SN-001.md", "---\nid: PI-SN-001\ntype: PlanningItem\nname: \"Update the heater\"\nstatus: todo\nitemType: task\nachieves: [REQ-SN-001]\n---\n\nWork.\n");
    w("model/REQ-SN-001.md", "---\nid: REQ-SN-001\ntype: Requirement\nname: \"Heater requirement\"\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nSee the item Update the heater for the plan.\n");
    w("model/Parts/Heater.md", "---\ntype: PartDef\nname: Heater\n---\n\nA heater.\n");
    (base, model)
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

#[test]
fn name_is_changed_in_place_on_an_id_identified_element() {
    let (_, m) = repo();
    let (out, code) = run(&m, &["set", "PI-SN-001", "name=Update the heater firmware: phase 2", "--as-human"]);
    assert_eq!(code, 0, "{out}");
    let f = read(&m, "PI-SN-001.md");
    assert!(f.contains("name: \"Update the heater firmware: phase 2\"") || f.contains("name: 'Update the heater firmware: phase 2'"), "{f}");
    assert!(f.contains("status: todo") && f.contains("achieves: [REQ-SN-001]") && f.contains("Work."), "nothing else changes: {f}");
    // The name is read back from the file, not just written to it.
    let (out, _) = run(&m, &["list", "PlanningItem", "--json"]);
    assert!(out.contains("Update the heater firmware: phase 2"), "{out}");
    assert_eq!(run(&m, &["validate"]).1, 0);
}

#[test]
fn renaming_to_the_same_name_is_a_no_op_and_dry_run_writes_nothing() {
    let (_, m) = repo();
    let before = read(&m, "PI-SN-001.md");
    let (out, code) = run(&m, &["set", "PI-SN-001", "name=Update the heater", "--as-human"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("nothing to do"), "{out}");
    let (out, code) = run(&m, &["set", "PI-SN-001", "name=Something else", "--dry-run", "--as-human"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("Something else"), "the diff is shown: {out}");
    assert_eq!(read(&m, "PI-SN-001.md"), before);
}

#[test]
fn a_name_that_is_an_identity_segment_is_a_rename_and_is_refused() {
    let (_, m) = repo();
    let before = read(&m, "Parts/Heater.md");
    let (out, code) = run(&m, &["set", "Parts::Heater", "name=Warmer", "--as-human"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("syscribe move") && out.contains("identity"), "{out}");
    assert_eq!(read(&m, "Parts/Heater.md"), before, "nothing is written");
}

#[test]
fn an_empty_or_multi_line_name_is_refused() {
    let (_, m) = repo();
    let before = read(&m, "PI-SN-001.md");
    for bad in ["name=", "name=   ", "name=two\nlines"] {
        let (out, code) = run(&m, &["set", "PI-SN-001", bad, "--as-human"]);
        assert_eq!(code, 1, "{bad:?}: {out}");
    }
    assert_eq!(read(&m, "PI-SN-001.md"), before);
}

#[test]
fn the_old_name_quoted_in_prose_is_pointed_out() {
    let (_, m) = repo();
    let (out, code) = run(&m, &["set", "PI-SN-001", "name=Rewrite the plan", "--as-human"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("Update the heater") && out.contains("REQ-SN-001") && out.contains("quoted"), "{out}");
}

#[test]
fn an_unknown_scalar_field_still_lists_what_is_supported_including_name() {
    let (_, m) = repo();
    let (out, code) = run(&m, &["set", "PI-SN-001", "nmae=x", "--as-human"]);
    assert_eq!(code, 1);
    assert!(out.contains("Scalar fields: status, name,") && out.contains("Did you mean 'name'"), "{out}");
}

#[test]
fn evidence_path_accepts_a_repository_relative_path() {
    let (_, m) = repo();
    let (out, code) = run(&m, &["set", "PI-SN-001", "evidence.add", "path=repo:src/real.rs", "--as-human"]);
    assert_eq!(code, 0, "{out}");
    assert!(read(&m, "PI-SN-001.md").contains("repo:src/real.rs"));
    // `validate` resolves the same entry the same way.
    let (out, _) = run(&m, &["validate"]);
    assert!(!out.contains("E717") && !out.contains("E716"), "{out}");
}

#[test]
fn evidence_path_keeps_the_other_forms_and_names_what_it_looked_for() {
    let (base, m) = repo();
    // A bare path and `model:` are model-relative; an absolute path and a URI are accepted as before.
    for (n, p) in [("a", "path=Docs/guide.txt".to_string()), ("b", "path=model:Docs/guide.txt".to_string()), ("c", format!("path={}", base.join("src/real.rs").display())), ("d", "path=https://example.com/r.html".to_string())] {
        let (out, code) = run(&m, &["set", "PI-SN-001", "evidence.add", &p, "--dry-run", "--as-human"]);
        assert_eq!(code, 0, "{n} {p}: {out}");
    }
    // `src/real.rs` exists, but outside the model: a bare path is not repository-relative.
    let (out, code) = run(&m, &["set", "PI-SN-001", "evidence.add", "path=src/real.rs", "--as-human"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("looked for") && out.contains("model/src/real.rs") && out.contains("repo:<path>"), "{out}");
    // A missing file is refused through `repo:` too, naming the repository location.
    let (out, code) = run(&m, &["set", "PI-SN-001", "evidence.add", "path=repo:src/missing.rs", "--as-human"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("looked for") && out.contains("src/missing.rs"), "{out}");
    assert!(!read(&m, "PI-SN-001.md").contains("missing.rs"), "nothing written");
}

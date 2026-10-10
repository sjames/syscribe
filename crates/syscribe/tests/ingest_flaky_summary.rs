//! TC-TRS-JUNIT-001 scenario "ingest summary" (GH #259).

use std::process::Command;

#[test]
fn ingest_prints_the_flaky_count_once_per_test() {
    let root = std::env::temp_dir().join(format!("syscribe-ingflaky-{}", std::process::id())).join("model");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    let xml = root.join("r.xml");
    std::fs::write(
        &xml,
        r#"<testsuite><testcase classname="c.T" name="a"/><testcase classname="c.T" name="b"><flakyFailure/></testcase><testcase classname="c.T" name="c"><failure/></testcase></testsuite>"#,
    )
    .unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(&root)
        .args(["ingest-results", "--format", "junit"])
        .arg(&xml)
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("1 pass, 1 fail, 1 flaky, 0 ignored"), "{out}");
}

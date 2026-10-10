//! TC-TRS-W015SPLIT-001 / GH #252.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(leaf3_tested: bool) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-w015-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/Opt.md", "---\ntype: FeatureDef\nid: FEAT-OPT-001\nname: Opt\ngroupKind: optional\n---\n\nO.\n");
    w("Configurations/CONF-A-001.md", "---\ntype: Configuration\nid: CONF-A-001\nname: A\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: true\n---\n\nA.\n");
    let req = |id: &str, parent: Option<&str>| {
        let d = parent.map(|p| format!("derivedFrom: [{p}]\n")).unwrap_or_default();
        format!("---\nid: {id}\ntype: Requirement\nname: {id}\nstatus: approved\nreqDomain: software\nreqClass: system\n{d}---\n\nShall.\n")
    };
    let tc = |id: &str, v: &str| format!("---\nid: {id}\ntype: TestCase\nname: {id}\nstatus: active\ntestLevel: L2\nverifies: [{v}]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    w("R/REQ-WS-001.md", &req("REQ-WS-001", None));
    w("R/REQ-WS-002.md", &req("REQ-WS-002", Some("REQ-WS-001")));
    w("R/REQ-WS-003.md", &req("REQ-WS-003", Some("REQ-WS-001")));
    w("R/REQ-WS-004.md", &req("REQ-WS-004", None));
    w("R/TC-WS-002.md", &tc("TC-WS-002", "REQ-WS-002"));
    if leaf3_tested {
        w("R/TC-WS-003.md", &tc("TC-WS-003", "REQ-WS-003"));
    }
    d
}

fn w015(d: &Path) -> Vec<String> {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d).arg("validate").output().unwrap();
    String::from_utf8_lossy(&o.stdout).lines().filter(|l| l.contains("W015")).map(String::from).collect()
}

#[test]
fn a_parent_with_verified_leaves_is_covered_through_its_children_only() {
    let lines = w015(&model(true));
    let parent = lines.iter().find(|l| l.contains("REQ-WS-001")).unwrap_or_else(|| panic!("{lines:?}"));
    assert!(parent.contains("covered through its children only") && parent.contains("direct test"), "{parent}");
}

#[test]
fn a_parent_with_an_unverified_leaf_and_a_plain_leaf() {
    let lines = w015(&model(false));
    let parent = lines.iter().find(|l| l.contains("REQ-WS-001")).unwrap();
    assert!(parent.contains("not every leaf below it is verified"), "{parent}");
    let leaf = lines.iter().find(|l| l.contains("REQ-WS-004")).unwrap();
    assert!(!leaf.contains("leaf") && !leaf.contains("children"), "a leaf's message is unchanged: {leaf}");
    assert!(lines.iter().any(|l| l.contains("REQ-WS-003")));
}

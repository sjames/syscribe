//! TC-TRS-SCBACK-001 / GH #247.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-scback-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("SG-SC-001.md", "---\ntype: SafetyGoal\nid: SG-SC-001\nname: g\nstatus: approved\nasilLevel: B\nsafeState: s\n---\n\nG.\n");
    for id in ["ARG-SC-001", "ARG-SC-002"] {
        w(
            &format!("{id}.md"),
            &format!("---\ntype: Argument\nid: {id}\nname: claim {id}\nstatus: approved\nargumentType: claim\nsupports: SG-SC-001\nevidence: [REQ-SC-001]\n---\n\nC.\n"),
        );
    }
    w("REQ-SC-001.md", "---\ntype: Requirement\nid: REQ-SC-001\nname: r\nstatus: approved\nreqDomain: software\nasilLevel: B\n---\n\nShall.\n");
    w(
        "TC-SC-001.md",
        "---\ntype: TestCase\nid: TC-SC-001\nname: t\nstatus: active\ntestLevel: L3\nverifies: [REQ-SC-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n",
    );
    r
}

fn run(root: &Path, args: &[&str]) -> String {
    String::from_utf8_lossy(&Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap().stdout).into_owned()
}

#[test]
fn a_repeated_subtree_is_printed_once_and_back_referenced() {
    let out = run(&model(), &["safety-case", "SG-SC-001"]);
    let req_lines: Vec<&str> = out.lines().filter(|l| l.contains("REQ-SC-001 —")).collect();
    assert_eq!(req_lines.len(), 2, "{out}");
    assert_eq!(req_lines.iter().filter(|l| l.contains("(see above)")).count(), 1, "{out}");
    assert_eq!(out.lines().filter(|l| l.contains("TC-SC-001")).count(), 1, "the test is under the first expansion only: {out}");
}

#[test]
fn json_still_has_the_subtree_under_both_arguments() {
    let out = run(&model(), &["safety-case", "SG-SC-001", "--json"]);
    assert_eq!(out.matches("\"TC-SC-001\"").count(), 2, "{out}");
}

#[test]
fn the_expanded_set_resets_for_each_goal() {
    // A second goal citing the same requirement expands it again (the set is per goal).
    let r = model();
    std::fs::write(r.join("SG-SC-002.md"), "---\ntype: SafetyGoal\nid: SG-SC-002\nname: g2\nstatus: approved\nasilLevel: B\nsafeState: s\n---\n\nG.\n").unwrap();
    std::fs::write(
        r.join("ARG-SC-003.md"),
        "---\ntype: Argument\nid: ARG-SC-003\nname: claim 3\nstatus: approved\nargumentType: claim\nsupports: SG-SC-002\nevidence: [REQ-SC-001]\n---\n\nC.\n",
    )
    .unwrap();
    let out = run(&r, &["safety-case"]);
    // SG-SC-001 expands the test once; SG-SC-002 expands it again: two TC lines in total.
    assert_eq!(out.lines().filter(|l| l.contains("TC-SC-001")).count(), 2, "{out}");
}

#[test]
fn a_repeated_argument_subtree_is_back_referenced_too() {
    // ARG-SC-004 is cited by two parents; the second citation is collapsed.
    let r = model();
    std::fs::write(
        r.join("ARG-SC-004.md"),
        "---\ntype: Argument\nid: ARG-SC-004\nname: shared strategy\nstatus: approved\nargumentType: strategy\nevidence: [REQ-SC-001]\n---\n\nS.\n",
    )
    .unwrap();
    for id in ["ARG-SC-001", "ARG-SC-002"] {
        std::fs::write(
            r.join(format!("{id}.md")),
            format!("---\ntype: Argument\nid: {id}\nname: claim {id}\nstatus: approved\nargumentType: claim\nsupports: SG-SC-001\nevidence: [ARG-SC-004]\n---\n\nC.\n"),
        )
        .unwrap();
    }
    let out = run(&r, &["safety-case", "SG-SC-001"]);
    let lines: Vec<&str> = out.lines().filter(|l| l.contains("ARG-SC-004 —")).collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert_eq!(lines.iter().filter(|l| l.contains("(see above)")).count(), 1, "{out}");
}

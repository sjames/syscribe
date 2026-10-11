//! GH #314: a TestPlan bound to configurations is a plan for those products only, and its `selection:` sweeps in only
//! test cases that run in at least one of them.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-tpp-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&r).unwrap();
    let w = |n: &str, c: &str| std::fs::write(r.join(n), c).unwrap();
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features.md", "---\ntype: FeatureModel\nname: Features\nfeatureTree:\n  - {name: Region, id: FEAT-TPP-REGION, mandatory: true, groupKind: alternative}\n  - {name: Region.US, id: FEAT-TPP-US}\n  - {name: Region.IN, id: FEAT-TPP-IN}\n---\n");
    for (cid, us, inn) in [("CONF-TPP-US-001", true, false), ("CONF-TPP-IN-001", false, true)] {
        w(&format!("{cid}.md"), &format!("---\ntype: Configuration\nid: {cid}\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Region: true\n  Features::Region::US: {us}\n  Features::Region::IN: {inn}\n---\n"));
    }
    w("REQ-TPP-001.md", "---\ntype: Requirement\nid: REQ-TPP-001\nname: r\nstatus: approved\nreqDomain: software\n---\n\nThe system shall.\n");
    for (n, when) in [(1, "appliesWhen: Features::Region::US\n"), (2, "appliesWhen: Features::Region::IN\n"), (3, "")] {
        w(&format!("TC-TPP-{n:03}.md"), &format!("---\ntype: TestCase\nid: TC-TPP-{n:03}\nname: t\nstatus: active\ntestLevel: L3\nverifies: [REQ-TPP-001]\n{when}---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n"));
    }
    for r_ in ["US", "IN"] {
        w(&format!("TP-TPP-{r_}-001.md"), &format!("---\ntype: TestPlan\nid: TP-TPP-{r_}-001\nname: p\nstatus: approved\nscope: certification\nconfigurations: [CONF-TPP-{r_}-001]\nselection:\n  testLevels: [L3]\n---\n\nPlan.\n"));
    }
    r
}

fn validate(root: &PathBuf, args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).arg("validate").args(args).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

#[test]
fn a_plan_for_another_product_is_not_part_of_this_projection() {
    let root = model();
    let t = validate(&root, &["--config", "CONF-TPP-US-001"]);
    for code in ["E606", "W019", "W611"] {
        assert!(!t.contains(code), "{code} in the US projection: {t}");
    }
    let t = validate(&root, &["--config", "CONF-TPP-IN-001"]);
    for code in ["E606", "W019", "W611"] {
        assert!(!t.contains(code), "{code} in the India projection: {t}");
    }
    assert!(validate(&root, &["--all-configs"]).contains("2 pass"), "both products validate");
}

#[test]
fn selection_does_not_sweep_in_test_cases_that_run_in_no_bound_product() {
    let t = validate(&model(), &[]);
    assert!(!t.contains("W611"), "the US plan does not sweep in the India-only test case: {t}");
}

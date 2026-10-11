//! GH #312: an element that is part of the product but satisfies a requirement the product does not have is not an escaping
//! reference; other traceability links to an inactive target still are.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-sg-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&r).unwrap();
    let w = |n: &str, c: &str| std::fs::write(r.join(n), c).unwrap();
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features.md", "---\ntype: FeatureModel\nname: Features\nfeatureTree:\n  - {name: Opt, id: FEAT-SG-OPT}\n---\n");
    w("CONF-SG-ON-001.md", "---\ntype: Configuration\nid: CONF-SG-ON-001\nname: on\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: true\n---\n");
    w("CONF-SG-OFF-001.md", "---\ntype: Configuration\nid: CONF-SG-OFF-001\nname: off\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: false\n---\n");
    w("REQ-SG-001.md", "---\ntype: Requirement\nid: REQ-SG-001\nname: r\nstatus: approved\nreqDomain: software\nappliesWhen: Features::Opt\n---\n\nThe system shall.\n");
    w("Platform.md", "---\ntype: PartDef\nname: Platform\ndomain: software\nsatisfies: [REQ-SG-001]\n---\nPlatform.\n");
    w("Use.md", "---\ntype: Part\nname: Use\ntypedBy: Platform\n---\nUse.\n");
    w("TC-SG-001.md", "---\ntype: TestCase\nid: TC-SG-001\nname: t\nstatus: active\ntestLevel: L3\nverifies: [REQ-SG-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n");
    r
}

fn lens(root: &PathBuf, conf: &str) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(["validate", "--config", conf]).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

#[test]
fn satisfying_an_inactive_requirement_is_not_an_escape_but_verifying_one_is() {
    let root = model();
    let t = lens(&root, "CONF-SG-OFF-001");
    assert!(!t.contains("Platform.md"), "the platform element does not escape: {t}");
    assert!(t.contains("TC-SG-001.md") && t.contains("W019"), "an ungated test case verifying an inactive requirement still escapes: {t}");
    assert!(!lens(&root, "CONF-SG-ON-001").contains("W019"), "nothing escapes when the feature is on");
}

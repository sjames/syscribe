//! GH #493 (`REQ-TRS-PIGATE-001`): under `--config`, a PlanningItem reference to an element that
//! exists but is gated out of the Configuration is not dangling.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct M {
    root: PathBuf,
}

impl M {
    fn new(parent_gate: &str) -> M {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-pg-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
        std::fs::create_dir_all(&root).unwrap();
        let m = M { root };
        m.w("_index.md", "---\ntype: Package\nname: Root\n---\n");
        m.w("Features/Layout.md", "---\ntype: FeatureDef\nid: FEAT-LAYOUT\nname: Layout\n---\n");
        m.w("Features/Ab.md", "---\ntype: FeatureDef\nid: FEAT-AB\nname: Ab\nrequires: [Features::Layout]\n---\n");
        m.w("Configs/CONF-LAYOUT-001.md", "---\ntype: Configuration\nid: CONF-LAYOUT-001\nname: Layout only\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Layout: true\n  Features::Ab: false\n---\n");
        m.w("Configs/CONF-AB-001.md", "---\ntype: Configuration\nid: CONF-AB-001\nname: Both\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Layout: true\n  Features::Ab: true\n---\n");
        m.w("Req/REQ-DEMO-LAY-001.md", "---\ntype: Requirement\nid: REQ-DEMO-LAY-001\nname: Layout\nstatus: draft\nreqDomain: software\nreqClass: system\nappliesWhen: Features::Layout\n---\nLayout.\n");
        m.w("Req/REQ-DEMO-AB-001.md", "---\ntype: Requirement\nid: REQ-DEMO-AB-001\nname: Ab\nstatus: draft\nreqDomain: software\nreqClass: system\nappliesWhen: Features::Ab\n---\nAb.\n");
        m.w("Plan/PI-DEMO-001.md", &format!("---\ntype: PlanningItem\nid: PI-DEMO-001\nname: Update\nstatus: todo\nitemType: feature\nappliesWhen: {parent_gate}\nachieves: [REQ-DEMO-LAY-001, REQ-DEMO-AB-001]\n---\n"));
        m.w("Plan/PI-DEMO-S1-001.md", "---\ntype: PlanningItem\nid: PI-DEMO-S1-001\nname: Layout step\nstatus: todo\nitemType: task\nparent: PI-DEMO-001\nappliesWhen: Features::Layout\n---\n");
        m
    }
    fn w(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn run(&self, args: &[&str]) -> (String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&self.root).args(args).output().unwrap();
        (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
    }
}

#[test]
fn a_reference_to_an_element_gated_out_of_the_configuration_is_not_dangling() {
    let m = M::new("Features::Layout");
    let (out, code) = m.run(&["validate", "--config", "CONF-LAYOUT-001"]);
    assert!(!out.contains("E714") && !out.contains("E710"), "{out}");
    assert_eq!(code, 0, "{out}");
    let (out, code) = m.run(&["validate", "--all-configs"]);
    assert_eq!(code, 0, "{out}");
}

#[test]
fn a_parent_gated_out_leaves_the_step_valid() {
    // The step serves Layout-only Configurations; its parent exists only with Ab.
    let m = M::new("Features::Ab");
    let (out, code) = m.run(&["validate", "--config", "CONF-LAYOUT-001"]);
    assert!(!out.contains("E710"), "{out}");
    assert_eq!(code, 0, "{out}");
}

#[test]
fn a_reference_that_exists_nowhere_is_still_an_error() {
    let m = M::new("Features::Layout");
    m.w("Plan/PI-DEMO-001.md", "---\ntype: PlanningItem\nid: PI-DEMO-001\nname: Update\nstatus: todo\nitemType: feature\nappliesWhen: Features::Layout\nachieves: [REQ-DEMO-LAY-001, REQ-NOPE-001]\nblockedBy: [REQ-NOPE-002]\n---\n");
    m.w("Plan/PI-DEMO-S1-001.md", "---\ntype: PlanningItem\nid: PI-DEMO-S1-001\nname: Layout step\nstatus: todo\nitemType: task\nparent: PI-NOPE-003\nappliesWhen: Features::Layout\n---\n");
    for args in [&["validate"][..], &["validate", "--config", "CONF-LAYOUT-001"][..]] {
        let (out, code) = m.run(args);
        assert!(out.contains("E714") && out.contains("E720") && out.contains("E710") && code != 0, "{args:?}: {out}");
    }
}

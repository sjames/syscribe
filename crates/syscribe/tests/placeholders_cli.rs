//! TC-TRS-PHOLD-001 end to end: the `--config` lens substitutes placeholders (GH #265).

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-phcli-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/Display.md", "---\ntype: FeatureDef\nid: FEAT-PC-100\nname: Display\ngroupKind: optional\nparameters:\n  - {name: sizeInch, type: ScalarValues::Real, unit: in}\n---\n\nD.\n");
    for (id, v) in [("CONF-PC-ALPHA-001", "8"), ("CONF-PC-BRAVO-001", "12.3")] {
        w(
            &format!("Configs/{id}.md"),
            &format!("---\nid: {id}\ntype: Configuration\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Display: true\nparameterBindings:\n  Features::Display.sizeInch: {v}\n---\n\nC.\n"),
        );
    }
    w(
        "Reqs/REQ-PC-001.md",
        "---\nid: REQ-PC-001\ntype: Requirement\nname: \"Display is {{Features::Display.sizeInch|unit}}\"\nstatus: draft\nreqDomain: software\nappliesWhen: Features::Display\n---\n\nShall.\n",
    );
    r
}

fn trace(root: &std::path::Path, conf: Option<&str>) -> String {
    let mut c = Command::new(env!("CARGO_BIN_EXE_syscribe"));
    c.arg("-m").arg(root).args(["trace", "REQ-PC-001"]);
    if let Some(conf) = conf {
        c.args(["--config", conf]);
    }
    String::from_utf8_lossy(&c.output().unwrap().stdout).into_owned()
}

#[test]
fn the_config_lens_shows_each_configurations_value() {
    let r = model();
    assert!(trace(&r, Some("CONF-PC-ALPHA-001")).contains("Display is 8 in"), "{}", trace(&r, Some("CONF-PC-ALPHA-001")));
    assert!(trace(&r, Some("CONF-PC-BRAVO-001")).contains("Display is 12.3 in"), "{}", trace(&r, Some("CONF-PC-BRAVO-001")));
}

#[test]
fn without_a_config_the_placeholder_stays_symbolic() {
    let r = model();
    assert!(trace(&r, None).contains("{{Features::Display.sizeInch|unit}}"), "{}", trace(&r, None));
}

#[test]
fn validate_all_configs_passes_for_a_well_formed_model() {
    let r = model();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&r).args(["validate", "--all-configs"]).output().unwrap();
    assert!(o.status.success(), "{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
}

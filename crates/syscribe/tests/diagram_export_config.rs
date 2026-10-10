//! TC-TRS-BDDCFG-001 / GH #244.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-bddcfg-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/CloudSync.md", "---\ntype: FeatureDef\nid: FEAT-CLOUD-SYNC\nname: CloudSync\ngroupKind: optional\n---\n\nCloud.\n");
    w("System/_index.md", "---\ntype: Package\nname: System\n---\n");
    w("System/Base.md", "---\ntype: PartDef\nname: Base\nasilLevel: B\nresponsibility: SafetyTeam\n---\n\nBase block.\n");
    w("System/Cloud.md", "---\ntype: PartDef\nname: Cloud\nappliesWhen: FEAT-CLOUD-SYNC\n---\n\nCloud block.\n");
    w("Diagrams/_index.md", "---\ntype: Package\nname: Diagrams\n---\n");
    w("Diagrams/D.md", "---\ntype: Diagram\nname: D\ndiagramKind: BDD\nsubject: System\n---\n\nDerived.\n");
    w("Configurations/CONF-ON-001.md", "---\ntype: Configuration\nid: CONF-ON-001\nname: On\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::CloudSync: true\n---\n\nOn.\n");
    w("Configurations/CONF-OFF-001.md", "---\ntype: Configuration\nid: CONF-OFF-001\nname: Off\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::CloudSync: false\n---\n\nOff.\n");
    d
}

fn run(d: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

#[test]
fn a_gated_block_follows_the_configuration() {
    let d = model();
    let (all, c) = run(&d, &["diagram", "export", "Diagrams::D", "--format", "mermaid"]);
    assert_eq!(c, 0, "{all}");
    assert!(all.contains("Cloud") && all.contains("Base"), "{all}");
    let (on, c) = run(&d, &["diagram", "export", "Diagrams::D", "--format", "mermaid", "--config", "CONF-ON-001"]);
    assert_eq!(c, 0, "{on}");
    assert!(on.contains("Cloud") && on.contains("Base"), "{on}");
    let (off, c) = run(&d, &["diagram", "export", "Diagrams::D", "--format", "mermaid", "--config", "CONF-OFF-001"]);
    assert_eq!(c, 0, "{off}");
    assert!(off.contains("Base") && !off.contains("Cloud"), "{off}");
}

#[test]
fn an_unknown_configuration_is_an_error() {
    let d = model();
    let (o, c) = run(&d, &["diagram", "export", "Diagrams::D", "--config", "CONF-NOPE-001"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("CONF-NOPE-001"), "{o}");
}

#[test]
fn blocks_show_the_asil_banner_and_responsibility() {
    let d = model();
    let (svg, c) = run(&d, &["diagram", "export", "Diagrams::D", "--format", "svg"]);
    assert_eq!(c, 0, "{svg}");
    assert!(svg.contains("ASIL B"), "{svg}");
    assert!(svg.contains("responsibility: SafetyTeam"), "{svg}");
}

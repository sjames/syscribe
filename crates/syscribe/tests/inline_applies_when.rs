//! GH #448: `appliesWhen:` on an inline `features:` / `connections:` entry gates it in a `--config` projection,
//! in `diff`, and in the multi-config diagram, like a child element would.

use std::path::{Path, PathBuf};
use std::process::Command;

fn model(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("syscribe-inlineaw-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let w = |rel: &str, c: &str| {
        let p = d.join("model").join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("Features/_index.md", "---\ntype: FeatureModel\nname: Features\nfeatureTree:\n  - {name: Mode, mandatory: true, groupKind: alternative, isAbstract: true}\n  - {name: Mode.Alpha, groupKind: optional}\n  - {name: Mode.Beta, groupKind: optional}\n---\n");
    for (n, a, b) in [("ALPHA", "true", "false"), ("BETA", "false", "true")] {
        w(&format!("Configurations/CONF-{n}-001.md"), &format!("---\ntype: Configuration\nid: CONF-{n}-001\nname: \"V {n}\"\nstatus: draft\nfeatureModel: Features\nfeatures:\n  Features::Mode::Alpha: {a}\n  Features::Mode::Beta: {b}\n---\n"));
    }
    w("Arch/P.md", "---\ntype: PortDef\nname: P\n---\n");
    w("Arch/Sys.md", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: p, type: Port, typedBy: Arch::P, direction: in}\n---\n");
    w("Arch/Ext.md", "---\ntype: PartDef\nname: Ext\nfeatures:\n  - {name: q, type: Port, typedBy: Arch::P, direction: out}\n---\n");
    w("Arch/Ctx.md", "---\ntype: PartDef\nname: Ctx\nfeatures:\n  - {name: sys, type: Part, typedBy: Arch::Sys}\n  - {name: onlyB, type: Part, typedBy: Arch::Ext, role: external, appliesWhen: Features::Mode::Beta}\nconnections:\n  - {from: onlyB.q, to: sys.p, appliesWhen: Features::Mode::Beta}\n---\n");
    w("Arch/CtxIbd.md", "---\ntype: Diagram\nname: CtxIbd\ndiagramKind: IBD\nsubject: Arch::Ctx\n---\n");
    d.join("model")
}

fn run(m: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(m).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).to_string(), o.status.code().unwrap_or(-1))
}

#[test]
fn the_gated_usage_is_in_one_variant_only() {
    let m = model("diagram");
    let svg = |c: &str| {
        let out = m.parent().unwrap().join(format!("{c}.svg"));
        let (_, code) = run(&m, &["diagram", "export", "Arch::CtxIbd", "--format", "svg", "--config", c, "--out", out.to_str().unwrap()]);
        assert_eq!(code, 0, "export {c}");
        std::fs::read_to_string(out).unwrap()
    };
    assert!(!svg("CONF-ALPHA-001").contains("onlyB"), "absent from Alpha");
    assert!(svg("CONF-BETA-001").contains("onlyB"), "present in Beta");
    let _ = std::fs::remove_dir_all(m.parent().unwrap());
}

#[test]
fn diff_reports_the_inline_usage_and_connection() {
    let m = model("diff");
    let (out, code) = run(&m, &["diff", "--config", "CONF-ALPHA-001", "--config", "CONF-BETA-001", "--json"]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let u = &v["usages"][0];
    assert_eq!(u["element"], "Arch::Ctx");
    assert_eq!(u["onlyInA"], serde_json::json!([]));
    assert_eq!(u["onlyInB"], serde_json::json!(["connections:onlyB.q -> sys.p", "features:onlyB"]));
    let (text, _) = run(&m, &["diff", "--config", "CONF-ALPHA-001", "--config", "CONF-BETA-001"]);
    assert!(text.contains("Inline usages and connections that differ (1)"), "{text}");
    // No difference, no section.
    let (same, _) = run(&m, &["diff", "--config", "CONF-BETA-001", "--config", "CONF-BETA-001", "--json"]);
    assert!(serde_json::from_str::<serde_json::Value>(&same).unwrap().get("usages").is_none());
    let _ = std::fs::remove_dir_all(m.parent().unwrap());
}

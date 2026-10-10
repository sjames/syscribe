//! TC-TRS-AUDITCLS-001 / GH #252 (audit part).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(toml: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-audcls-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    if !toml.is_empty() {
        w(".syscribe.toml", toml);
    }
    let req = |id: &str, class: &str, parent: Option<&str>| {
        let d = parent.map(|p| format!("derivedFrom: [{p}]\n")).unwrap_or_default();
        format!("---\nid: {id}\ntype: Requirement\nname: {id}\nstatus: approved\nreqDomain: software\nreqClass: {class}\n{d}---\n\nShall.\n")
    };
    let tc = |id: &str, v: &str| format!("---\nid: {id}\ntype: TestCase\nname: {id}\nstatus: active\ntestLevel: L3\nverifies: [{v}]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    w("R/REQ-AC-001.md", &req("REQ-AC-001", "stakeholder", None));
    w("R/REQ-AC-002.md", &req("REQ-AC-002", "system", Some("REQ-AC-001")));
    w("R/REQ-AC-003.md", &req("REQ-AC-003", "system", Some("REQ-AC-001")));
    w("R/REQ-AC-004.md", &req("REQ-AC-004", "system", None));
    w("R/TC-AC-002.md", &tc("TC-AC-002", "REQ-AC-002"));
    w("R/TC-AC-003.md", &tc("TC-AC-003", "REQ-AC-003"));
    d
}

fn run(d: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

#[test]
fn counts_and_percentages_per_class() {
    let d = model("");
    let (j, _) = run(&d, &["audit", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    let c = &v["coverageByClass"];
    // system: 002 and 003 verified leaves (complete), 004 uncovered (none)
    assert_eq!((c["system"]["complete"].as_u64(), c["system"]["none"].as_u64(), c["system"]["partial"].as_u64()), (Some(2), Some(1), Some(0)), "{c}");
    assert_eq!(c["system"]["percentComplete"].as_f64(), Some(66.7), "rounded like the other coverage figures: {c}");
    // stakeholder parent: all leaves verified but no direct test → partial under the default 'both'
    assert_eq!(c["stakeholder"]["partial"], 1, "{c}");
    let (t, _) = run(&d, &["audit"]);
    assert!(t.contains("Coverage by Requirement Class") && t.contains("stakeholder") && t.contains("system"), "{t}");
}

#[test]
fn the_policy_applies_and_an_invalid_table_is_reported() {
    let toml = "[[coverage.rule]]\nreqClass = \"stakeholder\"\nparent_rule = \"rollup\"\n";
    let (j, _) = run(&model(toml), &["audit", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["coverageByClass"]["stakeholder"]["complete"], 1, "{j}");
    let (j, _) = run(&model("[coverage]\ndefault = \"x\"\n"), &["audit", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert!(v["coverageByClass"].is_null() && v["coverageByClassError"].as_str().is_some_and(|e| e.contains("default")), "{j}");
}

#[test]
fn config_evaluates_the_selected_variant_alone() {
    let d = model("");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/Opt.md", "---\ntype: FeatureDef\nid: FEAT-OPT-001\nname: Opt\ngroupKind: optional\n---\n\nO.\n");
    for (id, v) in [("CONF-A-001", "true"), ("CONF-B-001", "false")] {
        w(&format!("Configurations/{id}.md"), &format!("---\ntype: Configuration\nid: {id}\nname: {id}\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: {v}\n---\n\nC.\n"));
    }
    // REQ-AC-004 is verified only by a test that exists when Opt is selected
    w("R/TC-AC-004.md", "---\nid: TC-AC-004\ntype: TestCase\nname: t4\nstatus: active\ntestLevel: L3\nverifies: [REQ-AC-004]\nappliesWhen: FEAT-OPT-001\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    let (j, _) = run(&d, &["audit", "--json", "--config", "CONF-A-001"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["coverageByClass"]["system"]["complete"], 3, "variant A verifies all system requirements: {j}");
}

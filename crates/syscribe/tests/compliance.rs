//! TC-TRS-COMPLY-001 / GH #239.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(toml: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-comply-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    if !toml.is_empty() {
        w(".syscribe.toml", toml);
    }
    w("Safety/_index.md", "---\ntype: Package\nname: Safety\n---\n");
    w("Safety/HE-CP-001.md", "---\nid: HE-CP-001\ntype: HazardousEvent\nname: loss of display\nstatus: approved\n---\n\nHazard.\n");
    w("Safety/SG-CP-001.md", "---\nid: SG-CP-001\ntype: SafetyGoal\nname: show speed\nstatus: draft\nasilLevel: B\n---\n\nGoal.\n");
    w("Safety/SG-CP-002.md", "---\nid: SG-CP-002\ntype: SafetyGoal\nname: show warnings\nstatus: approved\nasilLevel: B\n---\n\nGoal.\n");
    w("Requirements/_index.md", "---\ntype: Package\nname: Requirements\n---\n");
    w("Requirements/REQ-CP-001.md", "---\nid: REQ-CP-001\ntype: Requirement\nname: r\nstatus: approved\nreqDomain: software\nreqClass: software\ntags: [sw]\n---\n\nShall.\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/Opt.md", "---\ntype: FeatureDef\nid: FEAT-OPT-001\nname: Opt\ngroupKind: optional\n---\n\nO.\n");
    w("Requirements/REQ-CP-002.md", "---\nid: REQ-CP-002\ntype: Requirement\nname: r2\nstatus: approved\nreqDomain: software\nreqClass: software\ntags: [sw]\nappliesWhen: FEAT-OPT-001\n---\n\nShall.\n");
    w("Configurations/CONF-OFF-001.md", "---\ntype: Configuration\nid: CONF-OFF-001\nname: Off\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: false\n---\n\nOff.\n");
    d
}

fn run(d: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn items(j: &str) -> serde_json::Value {
    serde_json::from_str::<serde_json::Value>(j).unwrap()
}

#[test]
fn the_builtin_iso26262_mapping_reports_complete_partial_and_missing() {
    let d = model("");
    let (o, c) = run(&d, &["compliance", "--standard", "iso26262", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v = items(&o);
    let find = |wp: &str| v["items"].as_array().unwrap().iter().find(|i| i["workProduct"].as_str().unwrap().contains(wp)).unwrap_or_else(|| panic!("{wp}: {o}")).clone();
    let hara = find("Hazard");
    assert_eq!((hara["present"].as_u64(), hara["approved"].as_u64(), hara["status"].as_str()), (Some(1), Some(1), Some("complete")));
    let sg = find("Safety goal");
    assert_eq!((sg["present"].as_u64(), sg["approved"].as_u64(), sg["status"].as_str()), (Some(2), Some(1), Some("partial")));
    assert_eq!(find("FMEA")["status"], "missing");
    assert!(v["summary"]["missing"].as_u64().unwrap() >= 1);
    let (t, _) = run(&d, &["compliance", "--standard", "iso26262"]);
    assert!(t.contains("partial") && t.contains("missing") && t.contains("complete"), "{t}");
}

#[test]
fn a_standards_table_replaces_the_items_and_the_lens_applies() {
    let toml = "[[standards.aspice.item]]\nprocess = \"SWE.1\"\nworkProduct = \"Software requirements\"\ntype = \"Requirement\"\nreqClass = \"software\"\ntag = [\"sw\"]\n[[standards.aspice.item]]\nprocess = \"SWE.4\"\nworkProduct = \"Unit tests\"\ntype = \"TestCase\"\ntestLevel = \"L1\"\n";
    let d = model(toml);
    let v = items(&run(&d, &["compliance", "--standard", "aspice", "--json"]).0);
    let its = v["items"].as_array().unwrap();
    assert_eq!(its.len(), 2);
    assert_eq!((its[0]["process"].as_str(), its[0]["present"].as_u64()), (Some("SWE.1"), Some(2)));
    assert_eq!(its[1]["status"], "missing");
    let v = items(&run(&d, &["compliance", "--standard", "aspice", "--json", "--config", "CONF-OFF-001"]).0);
    assert_eq!(v["items"][0]["present"], 1, "REQ-CP-002 is gated off in the variant");
    assert_eq!(run(&d, &["compliance", "--standard", "aspice", "--fail-on-missing"]).1, 1);
    assert_eq!(run(&d, &["compliance", "--standard", "iso26262"]).1, 0);
}

#[test]
fn errors_exit_one() {
    let d = model("");
    let (o, c) = run(&d, &["compliance", "--standard", "nope"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("aspice") && o.contains("iso26262") && o.contains("iso21434"), "{o}");
    assert_eq!(run(&d, &["compliance"]).1, 1);
    assert_eq!(run(&model("[standards.x]\nitem = 3\n"), &["compliance", "--standard", "x"]).1, 1);
    assert_eq!(run(&model("[[standards.x.item]]\nprocess = \"P\"\n"), &["compliance", "--standard", "x"]).1, 1);
}

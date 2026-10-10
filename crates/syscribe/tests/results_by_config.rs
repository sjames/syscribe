//! TC-TRS-CFGRES-001 / GH #258 (per-configuration verdicts).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-cfgres-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let r = d.join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/Opt.md", "---\ntype: FeatureDef\nid: FEAT-OPT-001\nname: Opt\ngroupKind: optional\n---\n\nO.\n");
    for (id, v) in [("CONF-A-001", "true"), ("CONF-B-001", "false")] {
        w(&format!("Configurations/{id}.md"), &format!("---\ntype: Configuration\nid: {id}\nname: {id}\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: {v}\n---\n\nC.\n"));
    }
    w("REQ-PC-001.md", "---\nid: REQ-PC-001\ntype: Requirement\nname: r\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\nShall.\n");
    w("TC-PC-001.md", "---\nid: TC-PC-001\ntype: TestCase\nname: t\nstatus: active\ntestLevel: L3\nverifies: [REQ-PC-001]\ntestFunctions:\n  - function: \"t_one\"\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    d
}

fn junit(d: &Path, name: &str, verdict: &str) -> String {
    let body = if verdict == "pass" { "<testcase classname=\"C\" name=\"t_one\"/>".to_string() } else { "<testcase classname=\"C\" name=\"t_one\"><failure message=\"boom\"/></testcase>".to_string() };
    let p = d.join(name);
    std::fs::write(&p, format!("<testsuite>{body}</testsuite>")).unwrap();
    p.to_string_lossy().into_owned()
}

fn run(d: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d.join("model")).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn sidecar(d: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(d.join("model/.syscribe/results.json")).unwrap()).unwrap()
}

fn ingest(d: &Path, cfg: Option<&str>, verdict: &str, extra: &[&str]) -> (String, i32) {
    let f = junit(d, &format!("{}-{verdict}.xml", cfg.unwrap_or("global")), verdict);
    let mut a = vec!["ingest-results", "--format", "junit"];
    if let Some(c) = cfg {
        a.extend(["--config", c]);
    }
    a.extend(extra);
    a.push(&f);
    run(d, &a)
}

#[test]
fn config_ingest_is_stored_per_configuration_and_leaves_global_alone() {
    let d = model();
    let (o, c) = ingest(&d, Some("CONF-A-001"), "fail", &[]);
    assert_eq!(c, 0, "{o}");
    let s = sidecar(&d);
    assert_eq!(s["by_config"]["CONF-A-001"]["by_leaf"]["t_one"], "fail", "{s}");
    assert!(s["by_leaf"].as_object().is_none_or(|m| m.is_empty()), "global untouched: {s}");
    // a second ingest for the same configuration replaces its section; another configuration is independent
    ingest(&d, Some("CONF-A-001"), "pass", &[]);
    ingest(&d, Some("CONF-B-001"), "fail", &[]);
    let s = sidecar(&d);
    assert_eq!(s["by_config"]["CONF-A-001"]["by_leaf"]["t_one"], "pass");
    assert_eq!(s["by_config"]["CONF-B-001"]["by_leaf"]["t_one"], "fail");
    // a global ingest does not disturb the configuration sections
    ingest(&d, None, "pass", &[]);
    let s = sidecar(&d);
    assert_eq!(s["by_leaf"]["t_one"], "pass");
    assert_eq!(s["by_config"]["CONF-B-001"]["by_leaf"]["t_one"], "fail", "{s}");
}

#[test]
fn an_unknown_configuration_exits_one_and_writes_nothing() {
    let d = model();
    let (o, c) = ingest(&d, Some("CONF-NOPE"), "fail", &[]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("CONF-NOPE"), "{o}");
    assert!(!d.join("model/.syscribe/results.json").exists());
}

fn cell(d: &Path, conf: &str) -> String {
    let (o, c) = run(d, &["matrix", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let row = v["rows"].as_array().unwrap().iter().find(|r| r["id"] == "REQ-PC-001").unwrap();
    row["cells"][conf].as_str().unwrap_or_else(|| panic!("{conf} cell in {o}")).to_string()
}

#[test]
fn each_matrix_column_shows_its_own_verdict_and_falls_back_to_the_global_one() {
    let d = model();
    ingest(&d, Some("CONF-A-001"), "fail", &[]);
    ingest(&d, Some("CONF-B-001"), "pass", &[]);
    assert_eq!(cell(&d, "CONF-A-001"), "failing");
    assert_eq!(cell(&d, "CONF-B-001"), "passing");
    // a global pass applies to a configuration without its own verdict, never over one that has it
    let d2 = model();
    ingest(&d2, Some("CONF-A-001"), "fail", &[]);
    ingest(&d2, None, "pass", &[]);
    assert_eq!(cell(&d2, "CONF-A-001"), "failing", "its own verdict wins");
    assert_eq!(cell(&d2, "CONF-B-001"), "passing", "no own verdict: global");
}

#[test]
fn run_history_keeps_configuration_sections_and_diff_names_the_configuration() {
    let d = model();
    ingest(&d, Some("CONF-A-001"), "pass", &["--run", "R1"]);
    ingest(&d, Some("CONF-A-001"), "fail", &["--run", "R2"]);
    let (o, c) = run(&d, &["results", "diff", "R1", "R2", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let reg: Vec<&str> = v["regressions"].as_array().unwrap().iter().map(|x| x["test"].as_str().unwrap()).collect();
    assert_eq!(reg, vec!["t_one @ CONF-A-001"], "{o}");
    // --results-as-of restores the configuration section of the retained run
    let (o, _) = run(&d, &["--results-as-of", "R1", "matrix", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let row = v["rows"].as_array().unwrap().iter().find(|r| r["id"] == "REQ-PC-001").unwrap();
    assert_eq!(row["cells"]["CONF-A-001"], "passing", "{o}");
}

#[test]
fn an_old_sidecar_without_by_config_still_reads() {
    let d = model();
    std::fs::create_dir_all(d.join("model/.syscribe")).unwrap();
    std::fs::write(d.join("model/.syscribe/results.json"), r#"{"schema_version":"1.0","format":"junit","source":"x","ingested_at_unix":1,"count":1,"by_leaf":{"t_one":"pass"}}"#).unwrap();
    assert_eq!(cell(&d, "CONF-A-001"), "passing");
}

#[test]
fn config_lens_commands_judge_the_evidence_of_that_configuration() {
    let d = model();
    ingest(&d, Some("CONF-A-001"), "fail", &[]);
    ingest(&d, Some("CONF-B-001"), "pass", &[]);
    let (a, _) = run(&d, &["trace", "REQ-PC-001", "--config", "CONF-A-001"]);
    let (b, _) = run(&d, &["trace", "REQ-PC-001", "--config", "CONF-B-001"]);
    assert!(a.to_lowercase().contains("fail"), "A sees the failing run: {a}");
    assert!(!b.to_lowercase().contains("fail") && b.to_lowercase().contains("pass"), "B sees the passing run: {b}");
    // coverage tree and matrix --rollup under the lens
    let (ta, _) = run(&d, &["coverage", "tree", "REQ-PC-001", "--config", "CONF-A-001", "--json"]);
    let (tb, _) = run(&d, &["coverage", "tree", "REQ-PC-001", "--config", "CONF-B-001", "--json"]);
    let (ja, jb): (serde_json::Value, serde_json::Value) = (serde_json::from_str(&ta).unwrap(), serde_json::from_str(&tb).unwrap());
    assert_ne!(ja, jb, "the tree reads different evidence per configuration");
    // without a lens only the global sections apply: no global verdict at all
    let (g, _) = run(&d, &["trace", "REQ-PC-001"]);
    assert!(!g.to_lowercase().contains("fail"), "{g}");
}

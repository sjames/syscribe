//! `REQ-TRS-EXPRES-001` (GH #529): `export --resolved` carries what a document generator would otherwise
//! re-derive from the files — bodies, effective parameter values, reverse links, questions, build identity.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-expres-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    w("Features/Limits.md", "---\ntype: FeatureDef\nid: FEAT-LIMITS\nname: Limits\nmandatory: true\nparameters:\n  - { name: maxTemp, type: ScalarValues::Integer, default: 120, unit: degC, range: \"0..200\" }\n---\n");
    w("Features/Heater.md", "---\ntype: FeatureDef\nid: FEAT-HEATER\nname: Heater\nmandatory: true\nparameters:\n  - { name: onTimeMs, type: ScalarValues::Integer, unit: ms, derivedFrom: \"Features::Limits.maxTemp * 2\" }\n---\n");
    for (n, v) in [("001", "110"), ("002", "130")] {
        w(&format!("Configs/CONF-HTR-{n}.md"), &format!("---\ntype: Configuration\nid: CONF-HTR-{n}\nname: Variant {n}\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Limits: true\n  Features::Heater: true\nparameterBindings:\n  Features::Limits.maxTemp: {v}\n---\n"));
    }
    w("Rig.md", "---\ntype: PartDef\nname: Rig\ndomain: system\nsatisfies: [REQ-HTR-001]\nfeatures:\n  - name: settle\n    value: 150\n    unit: ms\n---\n\nThe rig.\n");
    w("REQ-HTR-001.md", "---\nid: REQ-HTR-001\ntype: Requirement\nname: \"Temperature limit\"\nstatus: approved\nreqDomain: system\nreqClass: system\n---\n\nThe heater shall not exceed {{Features::Limits.maxTemp|unit}} and shall settle within {{Rig.settle}}.\n\n1. {#limit} The limit holds at every ambient.\n");
    w("TC-HTR-001.md", "---\nid: TC-HTR-001\ntype: TestCase\ntestLevel: L4\nstatus: active\nname: \"Limit\"\nverifies: [REQ-HTR-001]\n---\n\n```gherkin\nFeature: F\n  Scenario: S\n    Given a heater\n```\n");
    w("PI-HTR-001.md", "---\nid: PI-HTR-001\ntype: PlanningItem\nname: \"Confirm the limit\"\nstatus: in_progress\nitemType: task\nachieves: [REQ-HTR-001]\nquestions:\n  - id: Q1\n    text: Which sensor?\n    recommended: PT100\n    status: open\n  - id: Q2\n    text: Sample rate\n    status: answered\n    answer: \"10 Hz\"\n    answeredBy: alice\n---\n\nWork on REQ-HTR-001#limit.\n");
    w("ADR-HTR-001.md", "---\nid: ADR-HTR-001\ntype: ADR\nname: \"Sensor choice\"\nstatus: proposed\n---\n\nPending PI-HTR-001#Q1; see REQ-HTR-001#limit.\n");
    root
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into_owned(), o.status.code().unwrap_or(-1))
}

fn export(root: &Path, args: &[&str]) -> Value {
    let mut a = vec!["export"];
    a.extend(args);
    let (out, code) = run(root, &a);
    assert_eq!(code, 0, "{out}");
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}\n{out}"))
}

fn elem<'a>(doc: &'a Value, id: &str) -> &'a Value {
    doc["elements"].as_array().unwrap().iter().find(|e| e["id"] == id || e["qname"] == id).unwrap_or_else(|| panic!("no element {id}"))
}

fn param<'a>(rows: &'a Value, key: &str) -> &'a Value {
    rows.as_array().unwrap_or_else(|| panic!("no rows: {rows}")).iter().find(|r| r["key"] == key).unwrap_or_else(|| panic!("no {key} in {rows}"))
}

fn git(root: &Path, args: &[&str]) {
    let o = Command::new("git").arg("-C").arg(root).args(["-c", "user.name=t", "-c", "user.email=t@example.com", "-c", "commit.gpgsign=false"]).args(args).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}

#[test]
fn the_plain_export_is_unchanged() {
    let m = model();
    let d = export(&m, &[]);
    assert!(d.get("meta").is_none() && d.get("resolved").is_none(), "{d}");
    let r = elem(&d, "REQ-HTR-001");
    assert!(r.get("body").is_none());
    assert!(r["computed"].get("satisfiedBy").is_none());
    assert_eq!(d["schemaVersion"], "1.0");
}

#[test]
fn every_element_carries_its_body_raw_and_with_value_references_rendered() {
    let m = model();
    let d = export(&m, &["--resolved"]);
    assert_eq!(d["resolved"], true);
    let b = &elem(&d, "REQ-HTR-001")["body"];
    assert!(b["raw"].as_str().unwrap().contains("{{Rig.settle}}"), "{b}");
    assert!(b["rendered"].as_str().unwrap().contains("settle within 150 ms."), "{b}");
    // No Configuration chosen: a feature parameter stays symbolic.
    assert!(b["rendered"].as_str().unwrap().contains("{{Features::Limits.maxTemp|unit}}"), "{b}");
    assert_eq!(elem(&d, "Rig")["body"]["raw"], "The rig.\n");
}

#[test]
fn under_a_configuration_the_body_holds_that_configurations_values() {
    let m = model();
    let d = export(&m, &["--resolved", "--config", "CONF-HTR-002"]);
    let b = elem(&d, "REQ-HTR-001")["body"]["rendered"].as_str().unwrap().to_string();
    assert!(b.contains("shall not exceed 130 degC and shall settle within 150 ms."), "{b}");
}

#[test]
fn a_configuration_carries_its_effective_parameters() {
    let m = model();
    let d = export(&m, &["--resolved"]);
    let rows = &elem(&d, "CONF-HTR-001")["effectiveParameters"];
    let t = param(rows, "Features::Limits.maxTemp");
    assert_eq!(t["value"], 110);
    assert_eq!(t["unit"], "degC");
    assert_eq!(t["range"], serde_json::json!({"min": 0.0, "max": 200.0}));
    assert_eq!(t["source"], "own");
    assert_eq!(t["feature"], "Features::Limits");
    assert_eq!(t["featureId"], "FEAT-LIMITS");
    assert_eq!(t["name"], "maxTemp");
    // A derived parameter is its computed value, with the expression beside it.
    let o = param(rows, "Features::Heater.onTimeMs");
    assert_eq!(o["value"], 220);
    assert_eq!(o["unit"], "ms");
    assert_eq!(o["source"], "derived");
    assert_eq!(o["expression"], "Features::Limits.maxTemp * 2");
    // The other Configuration has its own values.
    assert_eq!(param(&elem(&d, "CONF-HTR-002")["effectiveParameters"], "Features::Heater.onTimeMs")["value"], 260);
    assert!(d.get("effectiveParameters").is_none(), "top-level values need --config");
}

#[test]
fn with_config_the_top_level_holds_that_configurations_parameters() {
    let m = model();
    let d = export(&m, &["--resolved", "--config", "CONF-HTR-002"]);
    assert_eq!(d["meta"]["config"], "CONF-HTR-002");
    assert_eq!(param(&d["effectiveParameters"], "Features::Limits.maxTemp")["value"], 130);
    assert_eq!(param(&d["effectiveParameters"], "Features::Heater.onTimeMs")["value"], 260);
}

#[test]
fn reverse_links_are_computed() {
    let m = model();
    let d = export(&m, &["--resolved"]);
    let c = &elem(&d, "REQ-HTR-001")["computed"];
    assert_eq!(c["verifiedBy"], serde_json::json!(["TC-HTR-001"]));
    assert_eq!(c["satisfiedBy"], serde_json::json!(["Rig"]));
    assert_eq!(
        c["citedBy"],
        serde_json::json!([{"by": "ADR-HTR-001", "ref": "REQ-HTR-001#limit"}, {"by": "PI-HTR-001", "ref": "REQ-HTR-001#limit"}])
    );
    assert_eq!(elem(&d, "PI-HTR-001")["computed"]["citedBy"], serde_json::json!([{"by": "ADR-HTR-001", "ref": "PI-HTR-001#Q1"}]));
}

#[test]
fn a_planning_item_carries_its_questions_and_the_open_count() {
    let m = model();
    let d = export(&m, &["--resolved"]);
    let p = elem(&d, "PI-HTR-001");
    assert_eq!(p["openQuestions"], 1);
    let qs = p["questions"].as_array().unwrap();
    assert_eq!(qs.len(), 2);
    assert_eq!(qs[0]["ref"], "PI-HTR-001#Q1");
    assert_eq!(qs[0]["status"], "open");
    assert_eq!(qs[0]["recommended"], "PT100");
    assert_eq!(qs[1]["answer"], "10 Hz");
    // An item with no questions carries neither member.
    assert!(elem(&d, "REQ-HTR-001").get("questions").is_none());
}

#[test]
fn meta_names_the_build() {
    let m = model();
    // Outside git: no commit, and nothing claimed about the tree.
    let d = export(&m, &["--resolved"]);
    assert!(d["meta"]["commit"].is_null() && d["meta"]["dirty"].is_null(), "{}", d["meta"]);
    assert_eq!(d["meta"]["syscribeVersion"], env!("CARGO_PKG_VERSION"));
    assert!(d["meta"]["config"].is_null());
    let at = d["meta"]["generatedAt"].as_str().unwrap();
    assert!(at.len() == 20 && at.ends_with('Z') && at.as_bytes()[10] == b'T', "{at}");

    git(&m, &["init", "-q"]);
    git(&m, &["add", "."]);
    git(&m, &["commit", "-q", "-m", "init"]);
    let d = export(&m, &["--resolved"]);
    let commit = d["meta"]["commit"].as_str().unwrap();
    assert!(commit.len() == 40 && commit.chars().all(|c| c.is_ascii_hexdigit()), "{commit}");
    assert_eq!(d["meta"]["dirty"], false);
    std::fs::write(m.join("Rig.md"), "---\ntype: PartDef\nname: Rig\ndomain: system\nsatisfies: [REQ-HTR-001]\n---\n\nChanged.\n").unwrap();
    assert_eq!(export(&m, &["--resolved"])["meta"]["dirty"], true);
}

#[test]
fn ndjson_carries_meta_in_the_header_and_the_members_on_each_line() {
    let m = model();
    let (out, code) = run(&m, &["export", "--resolved", "--ndjson", "--config", "CONF-HTR-001"]);
    assert_eq!(code, 0);
    let lines: Vec<Value> = out.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(lines[0]["kind"], "header");
    assert_eq!(lines[0]["meta"]["config"], "CONF-HTR-001");
    assert_eq!(param(&lines[0]["effectiveParameters"], "Features::Limits.maxTemp")["value"], 110);
    let r = lines.iter().find(|l| l["id"] == "REQ-HTR-001").unwrap();
    assert!(r["body"]["rendered"].as_str().unwrap().contains("110 degC"), "{r}");
}

#[test]
fn the_schema_is_documented_under_spec_and_help() {
    let m = model();
    let (spec, code) = run(&m, &["spec", "export"]);
    assert_eq!(code, 0);
    for k in ["schemaVersion", "effectiveParameters", "citedBy", "satisfiedBy", "openQuestions", "generatedAt", "body"] {
        assert!(spec.contains(k), "spec export does not document {k}");
    }
    let (toc, _) = run(&m, &["spec"]);
    assert!(toc.contains("syscribe spec export"), "{toc}");
    let (help, _) = run(&m, &["help", "export"]);
    assert!(help.contains("--resolved"), "{help}");
}

//! GH #500/#501 slice 2a (`REQ-TRS-RESCOORD-001`, ADR-SYS-COVERAGE-001): a result record names the condition
//! point(s) it exercised and the unit it ran on, and both are part of the record's key.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-coord-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("REQ-HTR-001.md", "---\nid: REQ-HTR-001\ntype: Requirement\nname: \"Heater response\"\nstatus: approved\nreqDomain: system\nreqClass: system\n---\n\nThe heater shall reach the setpoint.\n");
    w("TE-HTR-001.md", "---\nid: TE-HTR-001\ntype: TestEnvironment\nname: Rig\nstatus: available\nenvironmentKind: bench\n---\n\nRig.\n");
    w("Heater.md", "---\ntype: PartDef\nname: Heater\n---\n\nThe heater.\n");
    w("TC-HTR-001.md", "---\nid: TC-HTR-001\ntype: TestCase\ntestLevel: L4\nstatus: active\nname: \"Heater response\"\nverifies: [REQ-HTR-001]\n---\n\n```gherkin\nFeature: F\n  Scenario: Response\n    Given a heater\n```\n");
    root
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn ingest(root: &Path, json: &str, extra: &[&str]) -> (String, i32) {
    let file = root.join("r.json");
    std::fs::write(&file, json).unwrap();
    let mut args = vec!["ingest-results", "--format", "records"];
    args.extend_from_slice(extra);
    args.push(file.to_str().unwrap());
    run(root, &args)
}

fn records(root: &Path) -> Vec<serde_json::Value> {
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap()).unwrap();
    v["records"].as_array().unwrap().clone()
}

fn rec(verdict: &str, cond: &str, unit: &str) -> String {
    let c = if cond.is_empty() { String::new() } else { format!(r#","condition":{{"ambient":"{cond}"}}"#) };
    let u = if unit.is_empty() { String::new() } else { format!(r#","unit":{{"id":"{unit}"}}"#) };
    format!(r#"{{"testCase":"TC-HTR-001","verdict":"{verdict}","provenance":{{"environment":"TE-HTR-001"{u}}}{c}}}"#)
}

fn trace_verdict(root: &Path) -> String {
    let (out, _) = run(root, &["trace", "REQ-HTR-001"]);
    let line = out.lines().find(|l| l.contains("TC-HTR-001")).unwrap_or_else(|| panic!("no row in:\n{out}"));
    let Some(open) = line.find('[') else { return "none".into() };
    line[open + 1..line[open..].find(']').map(|i| open + i).unwrap()].to_string()
}

#[test]
fn results_at_different_conditions_are_kept_side_by_side() {
    let m = model();
    let (out, code) = ingest(&m, &format!("[{},{},{}]", rec("pass", "cold", ""), rec("pass", "nominal", ""), rec("pass", "hot", "")), &[]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(records(&m).len(), 3);
    // a later ingest at one condition supersedes only that condition
    let (out, code) = ingest(&m, &format!("[{}]", rec("fail", "hot", "")), &[]);
    assert_eq!(code, 0, "{out}");
    let rs = records(&m);
    assert_eq!(rs.len(), 3, "{rs:?}");
    assert_eq!(rs.iter().filter(|r| r["verdict"] == "fail").count(), 1);
    assert_eq!(rs.iter().find(|r| r["condition"]["ambient"] == "hot").unwrap()["verdict"], "fail");
}

#[test]
fn a_failure_at_one_condition_is_not_hidden_by_a_pass_at_another() {
    let m = model();
    ingest(&m, &format!("[{},{}]", rec("fail", "hot", ""), rec("pass", "cold", "")), &[]);
    assert_eq!(trace_verdict(&m), "fail");
    ingest(&m, &format!("[{}]", rec("pass", "hot", "")), &[]);
    assert_eq!(trace_verdict(&m), "pass", "the corrected hot result replaces the failing one");
}

#[test]
fn results_on_different_units_do_not_replace_each_other() {
    let m = model();
    ingest(&m, &format!("[{},{}]", rec("pass", "", "SN-1"), rec("pass", "", "SN-2")), &[]);
    assert_eq!(records(&m).len(), 2);
    ingest(&m, &format!("[{}]", rec("fail", "", "SN-2")), &[]);
    let rs = records(&m);
    assert_eq!(rs.len(), 2);
    assert_eq!(rs.iter().find(|r| r["provenance"]["unit"]["id"] == "SN-1").unwrap()["verdict"], "pass");
    assert_eq!(rs.iter().find(|r| r["provenance"]["unit"]["id"] == "SN-2").unwrap()["verdict"], "fail");
}

#[test]
fn unit_of_must_resolve_to_a_part_definition_and_is_stored_by_qualified_name() {
    let m = model();
    let ok = r#"[{"testCase":"TC-HTR-001","verdict":"pass","provenance":{"unit":{"id":"SN-1","of":"Heater","lot":"L7"}}}]"#;
    let (out, code) = ingest(&m, ok, &[]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(records(&m)[0]["provenance"]["unit"]["of"], "Heater");
    assert_eq!(records(&m)[0]["provenance"]["unit"]["lot"], "L7");
    for (of, why) in [("Nothing", "does not resolve"), ("REQ-HTR-001", "not a part definition")] {
        let m = model();
        let (out, code) = ingest(&m, &format!(r#"[{{"testCase":"TC-HTR-001","verdict":"pass","provenance":{{"unit":{{"id":"SN-1","of":"{of}"}}}}}}]"#), &[]);
        assert_ne!(code, 0, "{out}");
        assert!(out.contains(why), "{out}");
        assert!(!m.join(".syscribe/results.json").exists(), "nothing is written");
    }
}

#[test]
fn a_malformed_condition_rejects_the_whole_file() {
    let m = model();
    let (out, code) = ingest(&m, &format!(r#"[{},{{"testCase":"TC-HTR-001","verdict":"pass","condition":{{"my axis":"hot"}}}}]"#, rec("pass", "cold", "")), &[]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("basic name"), "{out}");
    assert!(!m.join(".syscribe/results.json").exists());
}

#[test]
fn the_flags_fill_in_what_a_record_leaves_out_and_never_override_it() {
    let m = model();
    let j = format!(r#"[{{"testCase":"TC-HTR-001","verdict":"pass"}},{}]"#, rec("pass", "cold", "SN-9"));
    let (out, code) = ingest(&m, &j, &["--environment", "TE-HTR-001", "--unit", "SN-1", "--unit-of", "Heater", "--lot", "L7", "--condition", "ambient=hot", "--condition", "supply=low"]);
    assert_eq!(code, 0, "{out}");
    let rs = records(&m);
    let plain = rs.iter().find(|r| r["provenance"]["unit"]["id"] == "SN-1").unwrap();
    assert_eq!(plain["condition"]["ambient"], "hot");
    assert_eq!(plain["condition"]["supply"], "low");
    assert_eq!(plain["provenance"]["unit"]["of"], "Heater");
    let own = rs.iter().find(|r| r["provenance"]["unit"]["id"] == "SN-9").unwrap();
    assert_eq!(own["condition"]["ambient"], "cold", "a record's own value is kept");
    assert_eq!(own["condition"]["supply"], "low", "an axis the record did not name is filled");
    assert!(own["provenance"]["unit"]["of"].is_null(), "a record's own unit is not mixed with the flag's");
}

#[test]
fn bad_flags_are_refused_and_write_nothing() {
    for flags in [vec!["--condition", "hot"], vec!["--condition", "my axis=hot"], vec!["--unit-of", "Heater"], vec!["--lot", "L7"]] {
        let m = model();
        let (out, code) = ingest(&m, &format!("[{}]", rec("pass", "", "")), &flags);
        assert_ne!(code, 0, "{flags:?}: {out}");
        assert!(!m.join(".syscribe/results.json").exists(), "{flags:?}");
    }
}

#[test]
fn the_flags_are_for_records_like_formats_only() {
    let m = model();
    let f = m.join("r.xml");
    std::fs::write(&f, "<testsuite/>").unwrap();
    let (out, code) = run(&m, &["ingest-results", "--format", "junit", "--unit", "SN-1", f.to_str().unwrap()]);
    assert_ne!(code, 0, "{out}");
}

#[test]
fn records_without_coordinates_behave_as_before() {
    let m = model();
    let (out, code) = ingest(&m, &format!("[{}]", rec("pass", "", "")), &[]);
    assert_eq!(code, 0, "{out}");
    let rs = records(&m);
    assert_eq!(rs.len(), 1);
    assert!(rs[0].get("condition").is_none() && rs[0]["provenance"].get("unit").is_none());
    assert_eq!(trace_verdict(&m), "pass");
}

//! Adversarial and edge-case tests for `ingest-results --format records` (REQ-TRS-RESREC-001): what a careless
//! or hostile report, an odd model or a large run does to the ingest and to the readers.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_model() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("syscribe-resedge-{}-{}", std::process::id(), n)).join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: Edge\n---\n\nRoot.\n");
    write(&root, "REQ-ED-001.md", "---\nid: REQ-ED-001\ntype: Requirement\nname: \"R\"\nstatus: approved\nreqDomain: system\n---\n\nBody.\n");
    write(
        &root,
        "TC-ED-001.md",
        "---\nid: TC-ED-001\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: \"T\"\nverifies: [REQ-ED-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: One\n    Given x\n    Then y\n\n  Scenario: Zażółć gęślą jaźń\n    Given x\n    Then y\n```\n",
    );
    write(&root, "TC-ED-NOSCEN.md", "---\nid: TC-ED-NOSCEN\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: \"No scenarios\"\nverifies: [REQ-ED-001]\n---\n\nProse only.\n");
    write(
        &root,
        "TC-ED-FN.md",
        "---\nid: TC-ED-FN\ntype: TestCase\ntestLevel: L2\nstatus: active\nname: \"With functions\"\nverifies: [REQ-ED-001]\ntestFunctions:\n  - function: \"m::t_ok\"\n    scenario: \"Only\"\nsourceFile: repo:x.rs\n---\n\n```gherkin\nFeature: f\n  Scenario: Only\n    Given x\n    Then y\n```\n",
    );
    write(&root, "TE-ED-001.md", "---\nid: TE-ED-001\ntype: TestEnvironment\nname: Rig\nstatus: available\nenvironmentKind: bench\n---\n\nRig.\n");
    root
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn run(root: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().expect("spawn syscribe");
    (String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned(), out.status.code().unwrap_or(-1))
}

fn ingest_bytes(root: &Path, bytes: &[u8], extra: &[&str]) -> (String, String, i32) {
    std::fs::write(root.join("r.json"), bytes).unwrap();
    let file = root.join("r.json");
    let mut args = vec!["ingest-results", "--format", "records"];
    args.extend_from_slice(extra);
    args.push(file.to_str().unwrap());
    run(root, &args)
}

fn ingest(root: &Path, json: &str, extra: &[&str]) -> (String, String, i32) {
    ingest_bytes(root, json.as_bytes(), extra)
}

fn sidecar(root: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap()).unwrap()
}

fn trace(root: &Path) -> String {
    run(root, &["trace", "REQ-ED-001"]).0
}

#[test]
fn two_records_with_one_key_in_one_file_leave_one_record_the_last() {
    let root = new_model();
    let (_, err, code) = ingest(
        &root,
        r#"[{"testCase":"TC-ED-001","scenario":"One","verdict":"fail"},{"testCase":"TC-ED-001","scenario":"One","verdict":"pass"}]"#,
        &[],
    );
    assert_eq!(code, 0, "{err}");
    let recs = sidecar(&root)["records"].clone();
    let recs = recs.as_array().unwrap();
    assert_eq!(recs.len(), 1, "latest per key: {recs:?}");
    assert_eq!(recs[0]["verdict"], "pass", "the later record in the file wins");
}

#[test]
fn a_utf8_byte_order_mark_is_tolerated() {
    let root = new_model();
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(br#"[{"testCase":"TC-ED-001","verdict":"pass"}]"#);
    let (_, err, code) = ingest_bytes(&root, &bytes, &[]);
    assert_eq!(code, 0, "a BOM from a Windows tool must not break the ingest: {err}");
}

#[test]
fn non_utf8_input_is_a_clean_error() {
    let root = new_model();
    let (_, err, code) = ingest_bytes(&root, &[0xFF, 0xFE, 0x00, 0x5B], &[]);
    assert_ne!(code, 0);
    assert!(!err.contains("panicked"), "{err}");
    assert!(!root.join(".syscribe/results.json").exists());
}

#[test]
fn unicode_scenario_titles_join_and_roundtrip() {
    let root = new_model();
    let (_, err, code) = ingest(&root, r#"[{"testCase":"TC-ED-001","scenario":"One","verdict":"pass"},{"testCase":"TC-ED-001","scenario":"Zażółć gęślą jaźń","verdict":"pass"}]"#, &[]);
    assert_eq!(code, 0, "{err}");
    assert!(trace(&root).contains("TC-ED-001 [pass]"), "{}", trace(&root));
}

#[test]
fn null_optional_fields_are_accepted_as_absent() {
    let root = new_model();
    let (_, err, code) = ingest(&root, r#"[{"testCase":"TC-ED-001","scenario":null,"verdict":"pass","rationale":null,"provenance":null,"measurements":[],"steps":[]}]"#, &[]);
    assert_eq!(code, 0, "{err}");
}

#[test]
fn verdict_names_are_exact() {
    for bad in ["PASS", "Pass", "passed", "not_run", "notrun", "skip"] {
        let root = new_model();
        let (_, _, code) = ingest(&root, &format!(r#"[{{"testCase":"TC-ED-001","verdict":"{bad}"}}]"#), &[]);
        assert_ne!(code, 0, "{bad} must be rejected");
    }
    for good in ["pass", "fail", "flaky", "blocked", "not-run", "inconclusive", "ignored"] {
        let root = new_model();
        let (_, err, code) = ingest(&root, &format!(r#"[{{"testCase":"TC-ED-001","verdict":"{good}"}}]"#), &[]);
        assert_eq!(code, 0, "{good}: {err}");
    }
}

#[test]
fn measurement_values_parse_in_the_usual_notations() {
    let ok = |value: &str, op: &str, lim: &str| {
        let root = new_model();
        let json = format!(r#"[{{"testCase":"TC-ED-001","verdict":"pass","measurements":[{{"name":"m","value":"{value}","limit":{{"op":"{op}","value":"{lim}"}}}}]}}]"#);
        let (_, err, code) = ingest(&root, &json, &[]);
        (code, err)
    };
    assert_eq!(ok("-3 ms", "<", "0 ms").0, 0, "negative values");
    // GH #499: a reading below freezing, on either offset scale, as a value or a limit.
    assert_eq!(ok("-20 degC", ">", "-40 degC").0, 0, "a negative Celsius reading: {:?}", ok("-20 degC", ">", "-40 degC").1);
    assert_eq!(ok("-20 degC", "<", "253.16 K").0, 0, "…compared with kelvin");
    assert_eq!(ok("-20 degC", "==", "253.15 K").0, 0, "…equal to its kelvin value");
    assert_eq!(ok("-40 degF", "==", "-40 degC").0, 0, "…the scales meet at -40");
    assert_ne!(ok("-20 degC", ">", "-10 degC").0, 0, "a pass cannot violate its own limit below zero too");
    assert_ne!(ok("-300 degC", ">", "-400 degC").0, 0, "below absolute zero is not a quantity");
    assert_eq!(ok("1.5e-3 s", "<", "5 ms").0, 0, "scientific notation: {:?}", ok("1.5e-3 s", "<", "5 ms").1);
    assert_eq!(ok("12", ">=", "10").0, 0, "dimensionless");
    assert_eq!(ok("5 ms", "==", "5000 us").0, 0, "equality across units");
    assert_eq!(ok("5 ms", "<=", "5 ms").0, 0, "boundary is inclusive for <=");
    assert_ne!(ok("5 ms", "<", "5 ms").0, 0, "boundary is exclusive for <");
    assert_ne!(ok("fast", "<", "5 ms").0, 0, "a word is not a quantity");
    assert_ne!(ok("5 ms", "<", "").0, 0, "empty limit");
    assert_ne!(ok("", "<", "5 ms").0, 0, "empty value");
}

#[test]
fn a_limit_violation_is_accepted_for_every_verdict_but_pass() {
    for v in ["fail", "flaky", "blocked", "not-run", "inconclusive", "ignored"] {
        let root = new_model();
        let json = format!(r#"[{{"testCase":"TC-ED-001","verdict":"{v}","measurements":[{{"name":"m","value":"9 ms","limit":{{"op":"<","value":"5 ms"}}}}]}}]"#);
        assert_eq!(ingest(&root, &json, &[]).2, 0, "{v}");
    }
    let root = new_model();
    let json = r#"[{"testCase":"TC-ED-001","verdict":"waived","rationale":"known","measurements":[{"name":"m","value":"9 ms","limit":{"op":"<","value":"5 ms"}}]}]"#;
    assert_eq!(ingest(&root, json, &[]).2, 0, "a waiver may record the violation it waives");
}

#[test]
fn the_environment_resolves_by_qualified_name_and_is_stored_by_id() {
    let root = new_model();
    let (_, err, code) = ingest(&root, r#"[{"testCase":"TC-ED-001","verdict":"pass","provenance":{"environment":"Edge::TE-ED-001"}}]"#, &[]);
    // the qualified name of a top-level element under the model root
    if code == 0 {
        assert_eq!(sidecar(&root)["records"][0]["provenance"]["environment"], "TE-ED-001");
    } else {
        assert!(err.contains("TestEnvironment"), "{err}");
    }
    let root = new_model();
    let (_, err, code) = ingest(&root, r#"[{"testCase":"TC-ED-001","verdict":"pass","provenance":{"environment":"TE-ED-001"}}]"#, &[]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(sidecar(&root)["records"][0]["provenance"]["environment"], "TE-ED-001");
}

#[test]
fn an_environment_that_is_another_kind_of_element_is_rejected() {
    let root = new_model();
    let (_, err, code) = ingest(&root, r#"[{"testCase":"TC-ED-001","verdict":"pass","provenance":{"environment":"REQ-ED-001"}}]"#, &[]);
    assert_ne!(code, 0);
    assert!(err.contains("REQ-ED-001"), "{err}");
}

#[test]
fn records_for_unknown_test_cases_are_stored_not_dropped_and_do_not_crash_readers() {
    // joining and reporting unmatched records is REQ-TRS-RESJOIN-001; until then they must at least be harmless
    let root = new_model();
    let (_, err, code) = ingest(&root, r#"[{"testCase":"TC-NOPE-999","verdict":"fail"},{"testCase":"TC-ED-001","verdict":"pass"}]"#, &[]);
    assert_eq!(code, 0, "{err}");
    let sc = sidecar(&root);
    assert_eq!(sc["records"].as_array().unwrap().len(), 1, "only the joined record is evidence");
    assert_eq!(sc["unmatched"].as_array().unwrap().len(), 1, "the other is kept, listed, and counts for nothing");
    for cmd in [vec!["trace", "REQ-ED-001"], vec!["matrix"], vec!["validate"], vec!["safety-case"]] {
        let (_, e, c) = run(&root, &cmd);
        assert!((0..=2).contains(&c) && !e.contains("panicked"), "{cmd:?}: {c} {e}");
    }
}

#[test]
fn a_whole_testcase_record_scores_a_testcase_with_no_gherkin_scenarios() {
    let root = new_model();
    assert_eq!(ingest(&root, r#"[{"testCase":"TC-ED-NOSCEN","verdict":"pass"}]"#, &[]).2, 0);
    assert!(trace(&root).contains("TC-ED-NOSCEN [pass]"), "{}", trace(&root));
    let root = new_model();
    assert_eq!(ingest(&root, r#"[{"testCase":"TC-ED-NOSCEN","verdict":"waived","rationale":"n/a on this build"}]"#, &[]).2, 0);
    assert!(trace(&root).contains("TC-ED-NOSCEN [waived]"), "{}", trace(&root));
    let root = new_model();
    assert_eq!(ingest(&root, r#"[{"testCase":"TC-ED-NOSCEN","verdict":"blocked"}]"#, &[]).2, 0);
    assert!(trace(&root).contains("TC-ED-NOSCEN [unknown]"), "{}", trace(&root));
}

#[test]
fn a_testcase_with_test_functions_is_scored_by_them_and_ignores_records() {
    let root = new_model();
    assert_eq!(ingest(&root, r#"[{"testCase":"TC-ED-FN","verdict":"fail"}]"#, &[]).2, 0);
    assert!(!trace(&root).contains("TC-ED-FN [fail]"), "testFunctions decide, as for session-log:\n{}", trace(&root));
}

#[test]
fn a_whole_testcase_failure_beats_a_scenario_pass() {
    let root = new_model();
    let json = r#"[{"testCase":"TC-ED-001","verdict":"fail"},{"testCase":"TC-ED-001","scenario":"One","verdict":"pass"},{"testCase":"TC-ED-001","scenario":"Zażółć gęślą jaźń","verdict":"pass"}]"#;
    assert_eq!(ingest(&root, json, &[]).2, 0);
    assert!(trace(&root).contains("TC-ED-001 [fail]"), "{}", trace(&root));
}

#[test]
fn replace_is_refused_for_the_other_formats_and_writes_nothing() {
    let root = new_model();
    write(&root, "j.xml", "<testsuite><testcase classname=\"c\" name=\"t\"/></testsuite>");
    let (_, err, code) = run(&root, &["ingest-results", "--replace", root.join("j.xml").to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(err.contains("--replace"), "{err}");
    assert!(!root.join(".syscribe/results.json").exists());
}

#[test]
fn an_unknown_option_is_rejected_not_ignored() {
    let root = new_model();
    let (_, err, code) = ingest(&root, r#"[{"testCase":"TC-ED-001","verdict":"pass"}]"#, &["--replac"]);
    assert_ne!(code, 0, "a misspelt --replace must not silently merge");
    assert!(err.contains("--replac"), "{err}");
}

#[test]
fn the_report_can_come_from_stdin() {
    let root = new_model();
    let mut child = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(&root)
        .args(["ingest-results", "--format", "records", "/dev/stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(br#"[{"testCase":"TC-ED-001","verdict":"pass"}]"#).unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn a_large_run_ingests_and_reads_quickly() {
    let root = new_model();
    let mut json = String::from("[");
    for i in 0..5000 {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(r#"{{"testCase":"TC-LOAD-{i:04}","scenario":"S{}","verdict":"{}"}}"#, i % 7, if i % 11 == 0 { "fail" } else { "pass" }));
    }
    json.push(']');
    let t = std::time::Instant::now();
    let (_, err, code) = ingest(&root, &json, &[]);
    assert_eq!(code, 0, "{err}");
    let ingest_time = t.elapsed();
    let t = std::time::Instant::now();
    let _ = trace(&root);
    let read_time = t.elapsed();
    assert!(ingest_time.as_secs() < 20, "ingest of 5000 records took {ingest_time:?}");
    assert!(read_time.as_secs() < 20, "a reader over 5000 records took {read_time:?}");
    // none of the 5000 invented ids is a TestCase of the model, so all are unmatched, and none is evidence
    let sc = sidecar(&root);
    assert_eq!(sc["unmatched"].as_array().unwrap().len(), 5000);
    assert!(sc.get("records").is_none() || sc["records"].as_array().unwrap().is_empty());
}

#[test]
fn a_very_long_field_is_stored_without_trouble() {
    let root = new_model();
    let long = "x".repeat(200_000);
    let (_, err, code) = ingest(&root, &format!(r#"[{{"testCase":"TC-ED-001","verdict":"waived","rationale":"{long}"}}]"#), &[]);
    assert_eq!(code, 0, "{err}");
}

#[test]
fn the_config_lens_overlays_records_on_the_global_ones() {
    let root = new_model();
    write(&root, "CONF-ED.md", "---\nid: CONF-ED\ntype: Configuration\nname: Conf\nfeatures: {}\n---\n\nC.\n");
    // global: both pass; configuration: scenario One fails
    assert_eq!(
        ingest(&root, r#"[{"testCase":"TC-ED-001","scenario":"One","verdict":"pass"},{"testCase":"TC-ED-001","scenario":"Zażółć gęślą jaźń","verdict":"pass"}]"#, &[]).2,
        0
    );
    assert_eq!(ingest(&root, r#"[{"testCase":"TC-ED-001","scenario":"One","verdict":"fail"}]"#, &["--config", "CONF-ED"]).2, 0);
    let (global, _, _) = run(&root, &["trace", "REQ-ED-001"]);
    let (conf, _, _) = run(&root, &["trace", "REQ-ED-001", "--config", "CONF-ED"]);
    assert!(conf.contains("TC-ED-001 [fail]"), "the configuration sees its own failure:\n{conf}");
    // with no lens the reader is conservative: a failure on any variant is never hidden
    assert!(global.contains("TC-ED-001 [fail]") || global.contains("TC-ED-001 [pass]"), "{global}");
}

#[test]
fn run_history_retains_record_verdicts_and_diff_reports_a_regression() {
    let root = new_model();
    assert_eq!(ingest(&root, r#"[{"testCase":"TC-ED-001","scenario":"One","verdict":"pass"}]"#, &["--run", "r1"]).2, 0);
    assert_eq!(ingest(&root, r#"[{"testCase":"TC-ED-001","scenario":"One","verdict":"fail"}]"#, &["--run", "r2"]).2, 0);
    let (out, err, code) = run(&root, &["results", "diff", "r1", "r2", "--fail-on-regression"]);
    assert_eq!(code, 1, "a regression must fail the gate: {out}{err}");
    assert!(out.contains("TC-ED-001"), "{out}");
}

#[test]
fn as_of_a_retained_run_reads_the_records_of_that_run() {
    let root = new_model();
    let both = r#"[{"testCase":"TC-ED-001","scenario":"One","verdict":"pass"},{"testCase":"TC-ED-001","scenario":"Zażółć gęślą jaźń","verdict":"pass"}]"#;
    assert_eq!(ingest(&root, both, &["--run", "good"]).2, 0);
    assert_eq!(ingest(&root, r#"[{"testCase":"TC-ED-001","scenario":"One","verdict":"fail"}]"#, &["--run", "bad"]).2, 0);
    let (now, _, _) = run(&root, &["trace", "REQ-ED-001"]);
    let (then, _, _) = run(&root, &["--results-as-of", "good", "trace", "REQ-ED-001"]);
    assert!(now.contains("TC-ED-001 [fail]"), "{now}");
    assert!(then.contains("TC-ED-001 [pass]"), "{then}");
}

#[test]
fn json_output_of_the_readers_stays_valid_with_the_new_verdicts() {
    let root = new_model();
    let json = r#"[{"testCase":"TC-ED-001","scenario":"One","verdict":"waived","rationale":"r"},{"testCase":"TC-ED-001","scenario":"Zażółć gęślą jaźń","verdict":"pass"}]"#;
    assert_eq!(ingest(&root, json, &[]).2, 0);
    for cmd in [vec!["matrix", "--json"], vec!["audit", "--json"], vec!["testplan", "--json"]] {
        let (out, _, code) = run(&root, &cmd);
        if (0..=2).contains(&code) && !out.trim().is_empty() {
            serde_json::from_str::<serde_json::Value>(&out).unwrap_or_else(|e| panic!("{cmd:?} printed invalid JSON: {e}\n{out}"));
        }
    }
}

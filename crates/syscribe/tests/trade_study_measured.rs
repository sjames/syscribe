//! TC-TRS-MOEMEAS-001 (GH #467): a MoE can take its value from the measurements recorded for a Configuration, and a
//! recorded failure knocks the Configuration out of the ranking whatever its design values predict.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static N: AtomicU64 = AtomicU64::new(0);

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn run(root: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

fn moe(extra: &str) -> String {
    format!("---\ntype: CalculationDef\nname: CloseTimeMoE\nreturnType: ScalarValues::Real\nexpression: \"moe = closeMs\"\ncustom_fields:\n  mg_cell: B4\n  mg_moe: true\n  mg_moe_measures: REQ-TS-001\n  mg_moe_unit: ms\n  mg_moe_direction: minimize\n  mg_moe_threshold: 2000\n  mg_moe_objective: 1000\n{extra}---\n\nClose time.\n")
}

/// Two variants. By design B is faster (1500 ms vs 1650 ms); by measurement B is slower and breaks its limit.
fn model(moe_extra: &str, ingest: bool) -> PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-moemeas-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    write(&root, "_index.md", "---\ntype: Package\nname: Ts\n---\n\nFixture.\n");
    write(&root, "Features/_index.md", "---\ntype: Package\nname: Features\n---\n\nFeatures.\n");
    write(&root, "Features/Opt.md", "---\ntype: FeatureDef\nid: FEAT-TS-OPT\nname: Opt\nmandatory: false\n---\n\nOptional.\n");
    write(&root, "REQ-TS-001.md", "---\nid: REQ-TS-001\ntype: Requirement\nname: Close\nstatus: approved\nreqDomain: system\n---\n\nShall close.\n");
    write(&root, "TC-TS-001.md", "---\nid: TC-TS-001\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: Close time\nverifies: [REQ-TS-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: Close\n    Given x\n    Then y\n```\n");
    write(&root, ".syscribe.toml", "[profiles.magicgrid]\nmagicgrid = true\n");
    write(&root, "Moe/CloseTimeMoE.md", &moe(moe_extra));
    for (id, ms) in [("CONF-TS-A", 1650), ("CONF-TS-B", 1500)] {
        write(&root, &format!("Configurations/{id}.md"), &format!("---\ntype: Configuration\nid: {id}\nname: \"{id}\"\nstatus: approved\nfeatureModel: Features\nparameterBindings:\n  closeMs: {ms}\n---\n\nVariant.\n"));
    }
    if ingest {
        for (id, v, verdict, limit) in [("CONF-TS-A", "1650 ms", "pass", "2 s"), ("CONF-TS-B", "2300 ms", "fail", "2 s")] {
            let json = format!("[{{\"testCase\": \"TC-TS-001\", \"scenario\": \"Close\", \"verdict\": \"{verdict}\", \"measurements\": [{{\"name\": \"closeTime\", \"value\": \"{v}\", \"limit\": {{\"op\": \"<\", \"value\": \"{limit}\"}}}}]}}]");
            write(&root, &format!("{id}.json"), &json);
            let f = root.join(format!("{id}.json"));
            let (_, e, c) = run(&root, &["ingest-results", "--format", "records", "--config", id, f.to_str().unwrap()]);
            assert_eq!(c, 0, "{e}");
        }
    }
    root
}

fn study(root: &Path) -> Value {
    let (o, e, c) = run(root, &["trade-study", "--json"]);
    assert_eq!(c, 0, "{e}");
    serde_json::from_str(&o).unwrap()
}

fn rollup(v: &Value, conf: &str) -> Value {
    v["rollup"].as_array().unwrap().iter().find(|r| r["configuration"] == conf).unwrap().clone()
}

fn cell(v: &Value, conf: &str) -> Value {
    v["cells"].as_array().unwrap().iter().find(|c| c["configuration"] == conf).unwrap().clone()
}

#[test]
fn without_a_measurement_the_prediction_decides_as_before() {
    let root = model("", true);
    let v = study(&root);
    assert_eq!(rollup(&v, "CONF-TS-B")["winner"], true, "{v}");
    assert!(cell(&v, "CONF-TS-B")["measured"].is_null());
}

#[test]
fn a_recorded_failure_knocks_out_the_variant_the_design_values_favour() {
    // The issue: B looks best by design, but its recorded close time breaks its limit and its test failed.
    let root = model("  mg_moe_source: measurement\n  mg_moe_measurement: closeTime\n  mg_moe_aggregate: max\n", true);
    let v = study(&root);
    assert_eq!((rollup(&v, "CONF-TS-A")["winner"].clone(), rollup(&v, "CONF-TS-A")["fail"].clone()), (true.into(), false.into()), "{v}");
    assert_eq!((rollup(&v, "CONF-TS-B")["winner"].clone(), rollup(&v, "CONF-TS-B")["fail"].clone()), (false.into(), true.into()), "{v}");
    let (a, b) = (cell(&v, "CONF-TS-A"), cell(&v, "CONF-TS-B"));
    // Scored on the recorded value, the predicted one beside it.
    assert_eq!((a["value"].as_f64(), a["predicted"].as_f64(), a["measured"]["value"].as_f64()), (Some(1650.0), Some(1650.0), Some(1650.0)));
    assert_eq!((b["value"].as_f64(), b["predicted"].as_f64()), (Some(2300.0), Some(1500.0)));
    assert_eq!(b["thresholdViolation"], true);
    let why = b["violationReasons"].to_string();
    assert!(why.contains("worse than the threshold") && why.contains("fail verdict") && why.contains("own limit"), "{why}");
    // The evidence behind the cell: aggregate, record count, records and the ingest they came from.
    assert_eq!((b["measured"]["aggregate"].clone(), b["measured"]["recordCount"].clone(), b["measured"]["unit"].clone()), ("max".into(), 1.into(), "ms".into()));
    assert_eq!(b["measured"]["records"][0]["testCase"], "TC-TS-001");
    assert_eq!((b["measured"]["evidence"]["origin"].clone(), b["measured"]["evidence"]["format"].clone()), ("sidecar".into(), "records".into()));
    assert_eq!(v["moes"][0]["source"], "measurement");

    let (text, _, _) = run(&root, &["trade-study"]);
    assert!(text.contains("[pred 1500]") && text.contains("WINNER") && text.contains("FAIL"), "{text}");
    assert!(text.contains("Knock-outs:") && text.contains("CONF-TS-B / CloseTimeMoE"), "{text}");
}

#[test]
fn a_recorded_failure_knocks_out_even_when_the_prediction_scores() {
    // The default source stays the prediction, but recorded evidence still disqualifies, and is shown beside it.
    let root = model("  mg_moe_measurement: closeTime\n", true);
    let v = study(&root);
    assert_eq!(rollup(&v, "CONF-TS-B")["fail"], true, "{v}");
    assert_eq!(rollup(&v, "CONF-TS-B")["winner"], false);
    assert_eq!((cell(&v, "CONF-TS-B")["value"].as_f64(), cell(&v, "CONF-TS-B")["measured"]["value"].as_f64()), (Some(1500.0), Some(2300.0)));
    let (text, _, _) = run(&root, &["trade-study"]);
    assert!(text.contains("[meas 2300]"), "{text}");
}

#[test]
fn a_variant_with_no_recorded_measurement_is_n_a_and_never_wins_by_default() {
    let root = model("  mg_moe_source: measurement\n  mg_moe_measurement: closeTime\n", false);
    let v = study(&root);
    for c in ["CONF-TS-A", "CONF-TS-B"] {
        let x = cell(&v, c);
        assert_eq!((x["evaluable"].clone(), x["value"].clone(), x["measured"].clone()), (false.into(), Value::Null, Value::Null), "{x}");
    }
    // No evidence at all is not a win: nothing is marked WINNER.
    assert!(v["rollup"].as_array().unwrap().iter().all(|r| r["winner"] == false), "{v}");
    let (text, _, _) = run(&root, &["trade-study"]);
    assert!(text.contains("n/a") && !text.contains("WINNER"), "{text}");
}

#[test]
fn the_new_moe_fields_are_validated_by_mg033() {
    let mg033 = |extra: &str| {
        let root = model(extra, false);
        let (o, _, _) = run(&root, &["validate", "--profile", "magicgrid", "--json"]);
        let v: Value = serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"));
        v.to_string()
    };
    assert!(!mg033("  mg_moe_source: measurement\n  mg_moe_measurement: closeTime\n  mg_moe_aggregate: p95\n").contains("MG033"), "a valid measured MoE is clean");
    assert!(mg033("  mg_moe_aggregate: median\n  mg_moe_measurement: closeTime\n").contains("mg_moe_aggregate must be one of mean, max, p95, last"));
    assert!(mg033("  mg_moe_source: sensor\n").contains("mg_moe_source must be 'expression' or 'measurement'"));
    assert!(mg033("  mg_moe_source: measurement\n").contains("names no recorded measurement"));
    assert!(mg033("  mg_moe_measurement: closeTime\n  mg_moe_unit: parsecs-per-fortnight\n").contains("is not a unit a recorded measurement can be converted to"));
}

// ── records from files outside the model: --results ─────────────────────────────────────────────────────────────

fn rec(tc: &str, verdict: &str, value: &str) -> String {
    format!("{{\"testCase\": \"{tc}\", \"scenario\": \"Close\", \"verdict\": \"{verdict}\", \"measurements\": [{{\"name\": \"closeTime\", \"value\": \"{value}\", \"limit\": {{\"op\": \"<\", \"value\": \"2 s\"}}}}]}}")
}

/// A directory of runs outside the model: `<dir>/<CONF id>.json`.
fn run_dir(root: &Path, name: &str, a: Option<String>, b: Option<String>) -> PathBuf {
    let d = root.parent().unwrap().join(name);
    for (id, r) in [("CONF-TS-A", a), ("CONF-TS-B", b)] {
        if let Some(r) = r {
            write(&d, &format!("{id}.json"), &format!("[{r}]"));
        }
    }
    d
}

const SCORED_BY_MEASUREMENT: &str = "  mg_moe_source: measurement\n  mg_moe_measurement: closeTime\n  mg_moe_aggregate: max\n";

#[test]
fn a_results_directory_is_read_without_ingesting_anything() {
    let root = model(SCORED_BY_MEASUREMENT, false);
    let d = run_dir(&root, "run1", Some(rec("TC-TS-001", "pass", "1650 ms")), Some(rec("TC-TS-001", "fail", "2300 ms")));
    let (o, e, c) = run(&root, &["trade-study", "--json", "--results", d.to_str().unwrap()]);
    assert_eq!(c, 0, "{e}");
    let v: Value = serde_json::from_str(&o).unwrap();
    assert_eq!((rollup(&v, "CONF-TS-A")["winner"].clone(), rollup(&v, "CONF-TS-B")["fail"].clone()), (true.into(), true.into()), "{v}");
    assert!(!root.join(".syscribe/results.json").exists(), "reading a run writes nothing");
    // The evidence is named exactly: the file and a hash of its contents.
    let ev = &cell(&v, "CONF-TS-A")["measured"]["evidence"];
    assert_eq!((ev["origin"].clone(), ev["count"].clone(), ev["unmatched"].clone()), ("external".into(), 1.into(), 0.into()), "{ev}");
    assert!(ev["path"].as_str().unwrap().ends_with("run1/CONF-TS-A.json") && ev["hash"].as_str().unwrap().starts_with("blake3:"), "{ev}");
    assert_eq!(v["evidence"].as_array().unwrap().len(), 2);
    let (text, _, _) = run(&root, &["trade-study", "--results", d.to_str().unwrap()]);
    assert!(text.contains("Recorded evidence (--results):") && text.contains("CONF-TS-A <- ") && text.contains("blake3:"), "{text}");
}

#[test]
fn a_results_run_replaces_the_sidecar_and_a_missing_file_is_no_evidence_not_the_sidecars() {
    // The sidecar says B fails; the named run says B passes, and has no file for A at all.
    let root = model(SCORED_BY_MEASUREMENT, true);
    let d = run_dir(&root, "run2", None, Some(rec("TC-TS-001", "pass", "1500 ms")));
    let v: Value = serde_json::from_str(&run(&root, &["trade-study", "--json", "--results", d.to_str().unwrap()]).0).unwrap();
    assert_eq!((rollup(&v, "CONF-TS-B")["winner"].clone(), rollup(&v, "CONF-TS-B")["fail"].clone()), (true.into(), false.into()), "the run replaces the sidecar: {v}");
    let a = cell(&v, "CONF-TS-A");
    assert_eq!((a["evaluable"].clone(), a["measured"].clone()), (false.into(), Value::Null), "no file for A means no evidence, not the sidecar's 1650: {a}");
    let (text, _, _) = run(&root, &["trade-study", "--results", d.to_str().unwrap()]);
    assert!(text.contains("CONF-TS-A — no file in the directory"), "{text}");
}

#[test]
fn one_configuration_can_be_named_and_wins_over_the_directory_and_the_sidecar_still_serves_the_rest() {
    let root = model(SCORED_BY_MEASUREMENT, true);
    // Only B is named: B reads the file; A is not covered, so it still reads the sidecar.
    let f = root.parent().unwrap().join("b.json");
    write(&root, "../b.json", &format!("[{}]", rec("TC-TS-001", "pass", "1500 ms")));
    let spec = format!("CONF-TS-B={}", f.display());
    let v: Value = serde_json::from_str(&run(&root, &["trade-study", "--json", "--results", &spec]).0).unwrap();
    assert_eq!(cell(&v, "CONF-TS-B")["measured"]["evidence"]["origin"], "external");
    assert_eq!((cell(&v, "CONF-TS-A")["measured"]["evidence"]["origin"].clone(), cell(&v, "CONF-TS-A")["measured"]["value"].as_f64()), ("sidecar".into(), Some(1650.0)));
    // Named, it wins over a directory that also holds a file for B.
    let d = run_dir(&root, "run3", Some(rec("TC-TS-001", "pass", "1650 ms")), Some(rec("TC-TS-001", "fail", "2300 ms")));
    let v: Value = serde_json::from_str(&run(&root, &["trade-study", "--json", "--results", d.to_str().unwrap(), "--results", &spec]).0).unwrap();
    assert_eq!(cell(&v, "CONF-TS-B")["measured"]["value"].as_f64(), Some(1500.0), "{v}");
}

#[test]
fn a_record_naming_no_test_case_is_ignored_and_counted() {
    let root = model(SCORED_BY_MEASUREMENT, false);
    let both = format!("{}, {}", rec("TC-TS-001", "pass", "1650 ms"), rec("TC-NOPE-999", "pass", "1 ms"));
    let d = root.parent().unwrap().join("run4");
    write(&d, "CONF-TS-A.json", &format!("[{both}]"));
    let v: Value = serde_json::from_str(&run(&root, &["trade-study", "--json", "--results", d.to_str().unwrap()]).0).unwrap();
    let ev = &cell(&v, "CONF-TS-A")["measured"]["evidence"];
    assert_eq!((ev["count"].clone(), ev["unmatched"].clone()), (2.into(), 1.into()), "{ev}");
    assert_eq!(cell(&v, "CONF-TS-A")["measured"]["recordCount"], 1, "only the record that names a TestCase counts");
}

#[test]
fn bad_results_arguments_are_refused_naming_the_problem() {
    let root = model(SCORED_BY_MEASUREMENT, false);
    let d = run_dir(&root, "run5", Some(rec("TC-TS-001", "pass", "1650 ms")), None);
    let f = d.join("CONF-TS-A.json");
    let fail = |args: &[&str], needle: &str| {
        let mut a = vec!["trade-study"];
        a.extend_from_slice(args);
        let (o, e, c) = run(&root, &a);
        assert_eq!(c, 1, "{args:?}: {o}{e}");
        assert!(e.contains(needle), "{args:?}: {e}");
    };
    fail(&["--results", f.to_str().unwrap()], "not a directory");
    fail(&["--results", "CONF-NOPE=x.json"], "does not name a compared Configuration");
    fail(&["--results", root.to_str().unwrap()], "holds no <Configuration id>.json");
    let bad = root.parent().unwrap().join("bad");
    write(&bad, "CONF-TS-A.json", "[{\"testCase\": \"TC-TS-001\", \"verdict\": \"maybe\"}]");
    fail(&["--results", bad.to_str().unwrap()], "record 0");
    let spec = format!("CONF-TS-A={}", f.display());
    fail(&["--results", &spec, "--results", &spec], "twice");
    // A repeated flag with no value is a usage error like any other option.
    fail(&["--results"], "--results");
}

// ── a retained run, by id: --run (GH #471) ──────────────────────────────────────────────────────────────────────

/// Ingest `records` for one Configuration under run `run`.
fn ingest_run(root: &Path, conf: &str, run: &str, records: &str) {
    let name = format!("{conf}-{run}.json");
    write(root, &format!("../{name}"), &format!("[{records}]"));
    let f = root.parent().unwrap().join(&name);
    let (_, e, c) = run_cli(root, &["ingest-results", "--format", "records", "--config", conf, "--run", run, f.to_str().unwrap()]);
    assert_eq!(c, 0, "{e}");
}

fn run_cli(root: &Path, args: &[&str]) -> (String, String, i32) {
    run(root, args)
}

fn by_run(root: &Path, run_id: &str) -> Value {
    let (o, e, c) = run(root, &["trade-study", "--json", "--run", run_id]);
    assert_eq!(c, 0, "{e}");
    serde_json::from_str(&o).unwrap()
}

/// R1: B fails (2300 ms, over its limit). R2, later: B is fixed (1500 ms). The sidecar ends up holding R2 only.
fn two_runs() -> PathBuf {
    let root = model(SCORED_BY_MEASUREMENT, false);
    ingest_run(&root, "CONF-TS-A", "R1", &rec("TC-TS-001", "pass", "1650 ms"));
    ingest_run(&root, "CONF-TS-B", "R1", &rec("TC-TS-001", "fail", "2300 ms"));
    ingest_run(&root, "CONF-TS-A", "R2", &rec("TC-TS-001", "pass", "1650 ms"));
    ingest_run(&root, "CONF-TS-B", "R2", &rec("TC-TS-001", "pass", "1500 ms"));
    root
}

#[test]
fn an_earlier_run_is_ranked_by_id_after_a_later_ingest_overwrote_the_sidecar() {
    let root = two_runs();
    // Latest (the sidecar): R2, where B passes and wins on its better measurement.
    let latest: Value = serde_json::from_str(&run(&root, &["trade-study", "--json"]).0).unwrap();
    assert_eq!(rollup(&latest, "CONF-TS-B")["winner"], true, "{latest}");
    // R1 by id: B failed then, so it is knocked out and A wins, though the sidecar no longer holds R1.
    let r1 = by_run(&root, "R1");
    assert_eq!((rollup(&r1, "CONF-TS-A")["winner"].clone(), rollup(&r1, "CONF-TS-B")["fail"].clone()), (true.into(), true.into()), "{r1}");
    assert_eq!(cell(&r1, "CONF-TS-B")["measured"]["value"].as_f64(), Some(2300.0));
    // R2 by id agrees with the latest.
    assert_eq!(rollup(&by_run(&root, "R2"), "CONF-TS-B")["winner"], true);
    // The run id is in the evidence, and in the text.
    let ev = &cell(&r1, "CONF-TS-B")["measured"]["evidence"];
    assert_eq!((ev["origin"].clone(), ev["run"].clone(), ev["count"].clone()), ("run".into(), "R1".into(), 1.into()), "{ev}");
    assert_eq!(ev["stale"], serde_json::json!([]));
    let (text, _, _) = run(&root, &["trade-study", "--run", "R1"]);
    assert!(text.contains("<- run R1 (1 record(s))"), "{text}");
}

#[test]
fn re_ingesting_a_run_supersedes_its_own_records() {
    let root = two_runs();
    ingest_run(&root, "CONF-TS-B", "R1", &rec("TC-TS-001", "pass", "1400 ms"));
    let r1 = by_run(&root, "R1");
    assert_eq!(cell(&r1, "CONF-TS-B")["measured"]["value"].as_f64(), Some(1400.0), "{r1}");
    assert_eq!(cell(&r1, "CONF-TS-B")["measured"]["recordCount"], 1, "replaced, not appended");
}

#[test]
fn an_unknown_run_or_one_without_records_is_refused_naming_what_to_do() {
    let root = two_runs();
    let (_, e, c) = run(&root, &["trade-study", "--run", "R9"]);
    assert_eq!(c, 1);
    assert!(e.contains("no run 'R9'") && e.contains("R1") && e.contains("R2"), "{e}");
    // A function-level run keeps verdicts, not records: there is nothing to rank.
    write(&root, "../r.xml", "<testsuite name=\"s\"><testcase classname=\"c\" name=\"t_one\"/></testsuite>");
    let (_, e, c) = run(&root, &["ingest-results", "--format", "junit", "--run", "R3", root.parent().unwrap().join("r.xml").to_str().unwrap()]);
    assert_eq!(c, 0, "{e}");
    let (_, e, c) = run(&root, &["trade-study", "--run", "R3"]);
    assert_eq!(c, 1);
    assert!(e.contains("holds no records for any compared Configuration") && e.contains("--config"), "{e}");
    // No history at all.
    let (_, e, c) = run(&model(SCORED_BY_MEASUREMENT, false), &["trade-study", "--run", "R1"]);
    assert_eq!(c, 1);
    assert!(e.contains("none is") && e.contains("ingest-results --run"), "{e}");
}

#[test]
fn run_and_a_results_directory_are_alternatives_but_one_named_file_overrides_the_run() {
    let root = two_runs();
    let d = run_dir(&root, "run6", Some(rec("TC-TS-001", "pass", "1650 ms")), None);
    let (_, e, c) = run(&root, &["trade-study", "--run", "R1", "--results", d.to_str().unwrap()]);
    assert_eq!(c, 1);
    assert!(e.contains("--run and a --results directory both cover every compared Configuration"), "{e}");
    // A named file wins for that Configuration; the run covers the other.
    write(&root, "../b-fixed.json", &format!("[{}]", rec("TC-TS-001", "pass", "1500 ms")));
    let spec = format!("CONF-TS-B={}", root.parent().unwrap().join("b-fixed.json").display());
    let v: Value = serde_json::from_str(&run(&root, &["trade-study", "--json", "--run", "R1", "--results", &spec]).0).unwrap();
    assert_eq!(cell(&v, "CONF-TS-B")["measured"]["evidence"]["origin"], "external");
    assert_eq!((cell(&v, "CONF-TS-A")["measured"]["evidence"]["origin"].clone(), cell(&v, "CONF-TS-A")["measured"]["evidence"]["run"].clone()), ("run".into(), "R1".into()));
    assert_eq!(rollup(&v, "CONF-TS-B")["fail"], false, "the file's passing B replaces R1's failing one: {v}");
}

#[test]
fn a_run_read_after_the_model_changed_says_so() {
    let root = two_runs();
    // The TestCase these records were stamped against is edited afterwards.
    let tc = root.join("TC-TS-001.md");
    let text = std::fs::read_to_string(&tc).unwrap().replace("Then y", "Then y and z");
    std::fs::write(&tc, text).unwrap();
    let v = by_run(&root, "R1");
    let stale = cell(&v, "CONF-TS-A")["measured"]["evidence"]["stale"].to_string();
    assert!(stale.contains("TC-TS-001") && stale.contains("changed"), "{stale}");
    let (text, _, _) = run(&root, &["trade-study", "--run", "R1"]);
    assert!(text.contains("STALE") && text.contains("TC-TS-001"), "{text}");
    // It is information, not a knock-out: the ranking is unchanged.
    assert_eq!(rollup(&v, "CONF-TS-A")["winner"], true);
}

#[test]
fn a_run_flag_without_a_value_is_a_usage_error() {
    let root = two_runs();
    let (_, e, c) = run(&root, &["trade-study", "--run"]);
    assert_eq!(c, 1);
    assert!(e.contains("--run"), "{e}");
}

#[test]
fn run_and_results_are_refused_when_the_model_has_tradestudy_elements() {
    // TradeStudy elements take precedence over the MoE study, and neither flag applies to them.
    let root = two_runs();
    write(&root, "Studies/TRD-TS-001.md", "---\ntype: TradeStudy\nid: TRD-TS-001\nname: A study\nstatus: draft\n---\n\nStudy.\n");
    for flag in [["--run", "R1"], ["--results", "x"]] {
        let (_, e, c) = run(&root, &["trade-study", flag[0], flag[1]]);
        assert_eq!(c, 1, "{flag:?}: {e}");
        assert!(e.contains("not to TradeStudy elements"), "{flag:?}: {e}");
    }
}

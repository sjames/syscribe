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
    assert_eq!(b["measured"]["ingest"]["format"], "records");
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

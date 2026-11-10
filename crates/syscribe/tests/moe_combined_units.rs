//! GH #476/#477/#478: MoE key checks (MG034), MoEs over several measurements and over other MoEs, and Celsius.

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

fn moe_file(name: &str, custom: &str) -> String {
    format!("---\ntype: CalculationDef\nname: {name}\nreturnType: ScalarValues::Real\nexpression: \"moe = x\"\ncustom_fields:\n  mg_cell: B4\n  mg_moe: true\n  mg_moe_measures: REQ-TS-001\n{custom}---\n\nMoE.\n")
}

fn model(moes: &[(&str, &str)], ingest: bool) -> PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-moecomb-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    write(&root, "_index.md", "---\ntype: Package\nname: Ts\n---\n\nFixture.\n");
    write(&root, "Features/_index.md", "---\ntype: Package\nname: Features\n---\n\nFeatures.\n");
    write(&root, "Features/Opt.md", "---\ntype: FeatureDef\nid: FEAT-TS-OPT\nname: Opt\nmandatory: false\n---\n\nOptional.\n");
    write(&root, "REQ-TS-001.md", "---\nid: REQ-TS-001\ntype: Requirement\nname: Close\nstatus: approved\nreqDomain: system\n---\n\nShall close.\n");
    write(&root, "TC-TS-001.md", "---\nid: TC-TS-001\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: Close time\nverifies: [REQ-TS-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: Close\n    Given x\n    Then y\n```\n");
    write(&root, ".syscribe.toml", "[profiles.magicgrid]\nmagicgrid = true\n");
    for (n, c) in moes {
        write(&root, &format!("Moe/{n}.md"), &moe_file(n, c));
    }
    for id in ["CONF-TS-A", "CONF-TS-B"] {
        write(&root, &format!("Configurations/{id}.md"), &format!("---\ntype: Configuration\nid: {id}\nname: \"{id}\"\nstatus: approved\nfeatureModel: Features\n---\n\nVariant.\n"));
    }
    if ingest {
        // A: fast open, slow close.  B: the reverse.
        for (id, open, close) in [("CONF-TS-A", "100 ms", "300 ms"), ("CONF-TS-B", "200 ms", "100 ms")] {
            let json = format!("[{{\"testCase\": \"TC-TS-001\", \"scenario\": \"Close\", \"verdict\": \"pass\", \"measurements\": [{{\"name\": \"openTime\", \"value\": \"{open}\"}}, {{\"name\": \"closeTime\", \"value\": \"{close}\"}}]}}]");
            write(&root, &format!("{id}.json"), &json);
            let f = root.join(format!("{id}.json"));
            let (_, e, c) = run(&root, &["ingest-results", "--format", "records", "--config", id, f.to_str().unwrap()]);
            assert_eq!(c, 0, "{e}");
        }
    }
    root
}

fn cell(v: &Value, moe: &str, conf: &str) -> Value {
    v["cells"].as_array().unwrap().iter().find(|c| c["moe"] == moe && c["configuration"] == conf).unwrap().clone()
}

#[test]
fn a_moe_expression_reads_several_recorded_measurements() {
    let root = model(&[("Resp", "  mg_moe_source: measurement\n  mg_moe_variables:\n    closeT: {measurement: closeTime, unit: ms}\n    openT: {measurement: openTime, unit: ms}\n  mg_moe_direction: minimize\n  mg_moe_threshold: 400\n  mg_moe_objective: 100\n  mg_moe_weight: 1\n")], true);
    let mut p = std::fs::read_to_string(root.join("Moe/Resp.md")).unwrap();
    p = p.replace("moe = x", "moe = 0.6 * closeT + 0.4 * openT");
    std::fs::write(root.join("Moe/Resp.md"), p).unwrap();
    let (o, e, c) = run(&root, &["trade-study", "--json"]);
    assert_eq!(c, 0, "{e}");
    let v: Value = serde_json::from_str(&o).unwrap();
    assert_eq!(cell(&v, "Resp", "CONF-TS-A")["value"].as_f64(), Some(220.0), "{v}");
    assert_eq!(cell(&v, "Resp", "CONF-TS-B")["value"].as_f64(), Some(140.0), "{v}");
}

#[test]
fn a_moe_built_from_other_moes_rolls_up_and_propagates_knock_outs() {
    let leaf = |m: &str, th: u32| format!("  mg_moe_source: measurement\n  mg_moe_measurement: {m}\n  mg_moe_unit: ms\n  mg_moe_direction: minimize\n  mg_moe_threshold: {th}\n  mg_moe_objective: 100\n  mg_moe_weight: 0.5\n");
    let root = model(&[
        ("Open", &leaf("openTime", 150)),
        ("Close", &leaf("closeTime", 400)),
        ("Overall", "  mg_moe_unit: fraction\n  mg_moe_inputs:\n    o: Moe::Open\n    c: Moe::Close\n  mg_moe_direction: maximize\n  mg_moe_threshold: 0\n  mg_moe_objective: 1\n  mg_moe_weight: 1\n"),
    ], true);
    let mut p = std::fs::read_to_string(root.join("Moe/Overall.md")).unwrap();
    p = p.replace("moe = x", "moe = (o + c) / 2");
    std::fs::write(root.join("Moe/Overall.md"), p).unwrap();
    let (o, e, c) = run(&root, &["trade-study", "--json"]);
    assert_eq!(c, 0, "{e}");
    let v: Value = serde_json::from_str(&o).unwrap();
    // A: open 100 (score 1), close 300 (score 1/3) -> 0.666..; B: open 200 breaks threshold 150.
    let a = cell(&v, "Overall", "CONF-TS-A")["value"].as_f64().unwrap();
    assert!((a - (1.0 + 1.0 / 3.0) / 2.0).abs() < 1e-9, "{v}");
    let b = cell(&v, "Overall", "CONF-TS-B");
    assert_eq!(b["thresholdViolation"], true, "{b}");
    assert!(b["violationReasons"].to_string().contains("input Open"), "{b}");
    // Leaves are still reported; only the top MoE carries weight in the rollup.
    assert!(cell(&v, "Open", "CONF-TS-A")["value"].is_number());
    let r = v["rollup"].as_array().unwrap().iter().find(|r| r["configuration"] == "CONF-TS-A").unwrap().clone();
    assert!((r["weightedTotal"].as_f64().unwrap() - a).abs() < 1e-9, "{r}");
}

#[test]
fn mg034_flags_a_misspelt_key_a_missing_key_and_bad_weights() {
    let root = model(&[("M", "  mg_moe_unit: ms\n  mg_moe_direction: minimize\n  mg_moe_treshold: 150\n  mg_moe_objective: 50\n  mg_moe_weight: 0.4\n")], false);
    let (o, _, _) = run(&root, &["validate", "--profile", "magicgrid", "--json"]);
    let text = o.to_string();
    assert!(text.contains("MG034") && text.contains("unknown key 'mg_moe_treshold'") && text.contains("did you mean 'mg_moe_threshold'"), "{text}");
    assert!(text.contains("has no mg_moe_threshold"), "{text}");
    assert!(text.contains("weights sum to 0.4"), "{text}");
}

#[test]
fn celsius_and_fahrenheit_are_absolute_temperatures() {
    let root = model(&[], false);
    let (o, _, c) = run(&root, &["eval", "23.4 degC", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v: Value = serde_json::from_str(&o).unwrap();
    assert!((v["value"].as_f64().unwrap() - 296.55).abs() < 1e-9, "{v}");
    let (o, _, _) = run(&root, &["eval", "20 degC - 10 degC", "--json"]);
    let v: Value = serde_json::from_str(&o).unwrap();
    assert!((v["value"].as_f64().unwrap() - 10.0).abs() < 1e-9, "a difference is in kelvin: {v}");
    let (o, _, _) = run(&root, &["eval", "212 degF", "--unit", "degC", "--json"]);
    let v: Value = serde_json::from_str(&o).unwrap();
    assert!((v["value"].as_f64().unwrap() - 100.0).abs() < 1e-9, "{v}");
}

#[test]
fn mg034_spares_an_input_moe_its_weight_and_checks_inputs_and_variables() {
    let leaf = "  mg_moe_unit: ms\n  mg_moe_direction: minimize\n  mg_moe_threshold: 400\n  mg_moe_objective: 100\n";
    let root = model(&[
        ("Leaf", leaf),
        ("Top", "  mg_moe_inputs:\n    l: Moe::Leaf\n    gone: Moe::Missing\n  mg_moe_direction: maximize\n  mg_moe_threshold: 0\n  mg_moe_objective: 1\n  mg_moe_weight: 1\n"),
    ], false);
    let p = std::fs::read_to_string(root.join("Moe/Top.md")).unwrap().replace("moe = x", "moe = l + typo");
    std::fs::write(root.join("Moe/Top.md"), p).unwrap();
    let (o, _, _) = run(&root, &["validate", "--profile", "magicgrid", "--json"]);
    // The input MoE is not asked for a weight, nor counted in the weight sum.
    assert!(!o.contains("has no mg_moe_weight"), "{o}");
    assert!(!o.contains("weights sum"), "{o}");
    assert!(o.contains("'gone: Moe::Missing' does not resolve to a MoE"), "{o}");
    assert!(o.contains("expression uses 'typo'"), "{o}");
    assert!(!o.contains("expression uses 'l'"), "{o}");
}

#[test]
fn mg034_reports_a_cycle_of_inputs() {
    let c = |o: &str| format!("  mg_moe_inputs:\n    v: Moe::{o}\n  mg_moe_direction: maximize\n  mg_moe_threshold: 0\n  mg_moe_objective: 1\n  mg_moe_weight: 1\n");
    let root = model(&[("P", &c("Q")), ("Q", &c("P"))], false);
    let (o, _, _) = run(&root, &["validate", "--profile", "magicgrid", "--json"]);
    assert!(o.contains("is its own input through mg_moe_inputs"), "{o}");
}

#[test]
fn temperature_arithmetic_separates_readings_from_differences() {
    let root = model(&[], false);
    let ev = |e: &str| {
        let (o, err, c) = run(&root, &["eval", e, "--json"]);
        if c == 0 { Ok(serde_json::from_str::<Value>(&o).unwrap()["value"].as_f64().unwrap()) } else { Err(err) }
    };
    let near = |r: Result<f64, String>, x: f64| assert!((r.clone().unwrap() - x).abs() < 1e-9, "{r:?} != {x}");
    near(ev("20 degC + 5 delta_degC"), 298.15); // reading + difference is a reading
    near(ev("5 delta_degC + 20 degC"), 298.15);
    near(ev("68 degF - 20 degC"), 0.0);
    near(ev("2 * 5 delta_degC"), 10.0); // differences scale
    near(ev("300 K - 290 K"), 10.0);
    near(ev("20 degC + 9 delta_degF"), 298.15); // 9 degF difference = 5 K
    for bad in ["20 degC + 10 degC", "20 degC + 5 K", "5 delta_K - 20 degC", "2 * 20 degC", "20 degC / 2", "-(5 degC)", "sqrt(20 degC)", "20 degC > 5 delta_K", "min(20 degC, 5 delta_K)"] {
        let e = ev(bad).expect_err(bad);
        assert!(e.contains("dimension error"), "{bad}: {e}");
    }
    // Conversion: a reading and a difference read differently in degC.
    let conv = |e: &str, u: &str| {
        let (o, err, c) = run(&root, &["eval", e, "--unit", u, "--json"]);
        if c == 0 { Ok(serde_json::from_str::<Value>(&o).unwrap()["value"].as_f64().unwrap()) } else { Err(err) }
    };
    near(conv("20 degC - 10 degC", "delta_degC"), 10.0);
    near(conv("20 degC", "degF"), 68.0);
    near(conv("300 K", "degC"), 26.85);
    assert!(conv("20 degC - 10 degC", "degC").unwrap_err().contains("not an absolute reading"));
    assert!(conv("20 degC", "delta_degC").unwrap_err().contains("not a difference"));
}

//! GH #222 — `[cyber]` configuration of ISO/SAE 21434 risk and CAL determination.
//!
//! Black-box: each test builds a tiny model in a fresh temp dir and drives the
//! `syscribe` binary (`cyber-risk`, `validate`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_model(toml: Option<&str>) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir()
        .join(format!("syscribe-cyber-{}-{}", std::process::id(), n))
        .join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: CyberTest\n---\n\nRoot.\n");
    if let Some(t) = toml {
        write(&root, ".syscribe.toml", t);
    }
    root
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn syscribe");
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.code().unwrap_or(-1))
}

fn ds(root: &Path, id: &str, extra: &str) {
    write(
        root,
        &format!("{id}.md"),
        &format!("---\nid: {id}\ntype: DamageScenario\nname: D {id}\nstatus: draft\n{extra}---\n\nb\n"),
    );
}

fn ts(root: &Path, id: &str, ds_id: &str, extra: &str) {
    write(
        root,
        &format!("{id}.md"),
        &format!(
            "---\nid: {id}\ntype: ThreatScenario\nname: T {id}\nstatus: draft\ndamageScenarios: [{ds_id}]\n{extra}---\n\nb\n"
        ),
    );
}

/// `cyber-risk --json` row for `id`.
fn row(root: &Path, id: &str) -> serde_json::Value {
    let (out, _) = run(root, &["cyber-risk", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    v.as_array().unwrap().iter().find(|r| r["id"] == id).cloned().unwrap_or_else(|| panic!("no {id} in {out}"))
}

fn has_code(root: &Path, code: &str) -> bool {
    let (out, _) = run(root, &["validate"]);
    out.contains(code)
}

fn basic_pair(root: &Path, sev: &str, feas: &str, vector: &str) {
    ds(root, "DS-CY-001", &format!("damageSeverity: {sev}\n"));
    ts(root, "TS-CY-001", "DS-CY-001", &format!("attackFeasibility: {feas}\nattackVector: {vector}\n"));
}

#[test]
fn default_is_the_rank_sum_and_json_shape_is_unchanged() {
    let root = new_model(None);
    // negligible impact + high feasibility rates medium today (0 + 3 = 3).
    basic_pair(&root, "negligible", "high", "network");
    let r = row(&root, "TS-CY-001");
    assert_eq!(r["risk"], "medium");
    let keys: Vec<&str> = r.as_object().unwrap().keys().map(|s| s.as_str()).collect();
    for k in ["id", "severity", "feasibility", "risk", "treatment", "addressed", "flag"] {
        assert!(keys.contains(&k), "{keys:?}");
    }
    assert!(!keys.contains(&"method") && !keys.contains(&"expectedCal"), "{keys:?}");
    let (text, _) = run(&root, &["cyber-risk"]);
    assert!(text.contains("| Threat | Severity | Feasibility | Risk | Treatment | Addressed | Flag |"), "{text}");
    assert!(!text.contains("Risk configuration"), "{text}");
}

#[test]
fn empty_or_irrelevant_cyber_table_changes_nothing() {
    let plain = new_model(None);
    let empty = new_model(Some("[cyber]\nmethod = \"simple\"\n"));
    for r in [&plain, &empty] {
        basic_pair(r, "severe", "medium", "local");
    }
    let (a, _) = run(&plain, &["cyber-risk"]);
    let (b, _) = run(&empty, &["cyber-risk"]);
    assert_eq!(a, b);
    let (a, ca) = run(&plain, &["validate"]);
    let (b, cb) = run(&empty, &["validate"]);
    assert_eq!(ca, cb);
    // Same findings modulo the temp-dir path.
    let strip = |s: &str, r: &Path| s.replace(r.parent().unwrap().to_str().unwrap(), "<R>");
    assert_eq!(strip(&a, &plain), strip(&b, &empty));
}

#[test]
fn annex_method_uses_the_example_risk_matrix_and_labels_it() {
    let root = new_model(Some("[cyber]\nmethod = \"annex\"\n"));
    basic_pair(&root, "negligible", "high", "network");
    let r = row(&root, "TS-CY-001");
    // Annex example matrix: negligible impact is risk value 1 whatever the feasibility.
    assert_eq!(r["riskValue"], 1);
    assert_eq!(r["risk"], "low");
    assert_eq!(r["method"], "annex");
    let (text, _) = run(&root, &["cyber-risk"]);
    assert!(text.contains("EXAMPLE") && text.contains("not normative"), "{text}");
}

#[test]
fn annex_cal_comes_from_impact_and_attack_vector() {
    let root = new_model(Some("[cyber]\nmethod = \"annex\"\n"));
    basic_pair(&root, "severe", "high", "network");
    assert_eq!(row(&root, "TS-CY-001")["expectedCal"], "CAL4");
    let root = new_model(Some("[cyber]\nmethod = \"annex\"\n"));
    basic_pair(&root, "severe", "high", "physical");
    assert_eq!(row(&root, "TS-CY-001")["expectedCal"], "CAL2");
}

#[test]
fn risk_matrix_cells_override_per_cell_over_the_base_method() {
    let root = new_model(Some("[cyber.risk_matrix.negligible]\nhigh = \"critical\"\n"));
    basic_pair(&root, "negligible", "high", "network");
    assert_eq!(row(&root, "TS-CY-001")["risk"], "critical");
    // an untouched cell keeps the rank-sum default (moderate + low = 2 -> medium)
    let root2 = new_model(Some("[cyber.risk_matrix.negligible]\nhigh = \"critical\"\n"));
    basic_pair(&root2, "moderate", "low", "network");
    assert_eq!(row(&root2, "TS-CY-001")["risk"], "medium");
}

#[test]
fn integer_cells_map_through_risk_levels() {
    let root = new_model(Some(
        "[cyber.risk_matrix.severe]\nhigh = 5\n[cyber.risk_levels]\n\"5\" = \"high\"\n",
    ));
    basic_pair(&root, "severe", "high", "local");
    let r = row(&root, "TS-CY-001");
    assert_eq!(r["risk"], "high");
    assert_eq!(r["riskValue"], 5);
}

#[test]
fn cal_table_and_cal_by_risk_overrides_drive_w032() {
    // Goal declares CAL1 for a severe/high network threat; default risk->CAL needs CAL4.
    let model = |toml: Option<&str>| {
        let root = new_model(toml);
        basic_pair(&root, "severe", "high", "network");
        write(
            &root,
            "CSG-CY-001.md",
            "---\nid: CSG-CY-001\ntype: CybersecurityGoal\nname: G\nstatus: draft\nthreatScenarios: [TS-CY-001]\ncalLevel: CAL1\nsecurityProperty: integrity\n---\n\nb\n",
        );
        root
    };
    assert!(has_code(&model(None), "W032"));
    // cal_table says severe x network = CAL1 -> goal is consistent, W032 gone.
    let relaxed = model(Some("[cyber.cal_table.severe]\nnetwork = \"CAL1\"\n"));
    assert!(!has_code(&relaxed, "W032"));
    // cal_by_risk alone can relax too.
    let by_risk = model(Some("[cyber.cal_by_risk]\ncritical = \"CAL1\"\n"));
    assert!(!has_code(&by_risk, "W032"));
}

const AP_FIELDS: &str = "elapsedTime: up_to_1_week\nexpertise: layman\nknowledge: public\nwindowOfOpportunity: unlimited\nequipment: standard\n";

#[test]
fn attack_potential_factors_derive_feasibility() {
    let root = new_model(None);
    ds(&root, "DS-CY-001", "damageSeverity: major\n");
    // 1+0+0+0+0 = 1 -> high feasibility under the example thresholds
    ts(&root, "TS-CY-001", "DS-CY-001", AP_FIELDS);
    let r = row(&root, "TS-CY-001");
    assert_eq!(r["feasibility"], "high");
    assert_eq!(r["risk"], "critical"); // major(2)+high(3)=5
}

#[test]
fn attack_potential_thresholds_and_tables_are_configurable() {
    let root = new_model(Some(
        "[cyber.attack_potential.thresholds]\nhigh_max = 0\nmedium_max = 2\nlow_max = 3\n",
    ));
    ds(&root, "DS-CY-001", "damageSeverity: major\n");
    ts(&root, "TS-CY-001", "DS-CY-001", AP_FIELDS); // 1 point -> medium with these thresholds
    assert_eq!(row(&root, "TS-CY-001")["feasibility"], "medium");

    let root = new_model(Some("[cyber.attack_potential.expertise]\nlayman = 30\nguru = 31\n"));
    ds(&root, "DS-CY-001", "damageSeverity: major\n");
    ts(&root, "TS-CY-001", "DS-CY-001", AP_FIELDS); // 1 + 30 = 31 -> very_low
    assert_eq!(row(&root, "TS-CY-001")["feasibility"], "very_low");
}

#[test]
fn integer_points_are_accepted_and_bad_factors_raise_e641() {
    let root = new_model(None);
    ds(&root, "DS-CY-001", "damageSeverity: major\n");
    ts(&root, "TS-CY-001", "DS-CY-001", "elapsedTime: 30\nexpertise: 0\nknowledge: 0\nwindowOfOpportunity: 0\nequipment: 0\n");
    assert_eq!(row(&root, "TS-CY-001")["feasibility"], "very_low");
    assert!(!has_code(&root, "E641"));

    let bad = new_model(None);
    ds(&bad, "DS-CY-001", "damageSeverity: major\n");
    ts(&bad, "TS-CY-001", "DS-CY-001", &AP_FIELDS.replace("layman", "wizard"));
    assert!(has_code(&bad, "E641"));
}

#[test]
fn incomplete_factors_raise_w641_and_are_ignored() {
    let root = new_model(None);
    ds(&root, "DS-CY-001", "damageSeverity: major\n");
    ts(&root, "TS-CY-001", "DS-CY-001", "elapsedTime: up_to_1_day\nexpertise: layman\n");
    assert!(has_code(&root, "W641"));
    assert_eq!(row(&root, "TS-CY-001")["risk"], "unknown");
}

#[test]
fn declared_feasibility_wins_and_a_mismatch_is_w642() {
    let root = new_model(None);
    ds(&root, "DS-CY-001", "damageSeverity: major\n");
    ts(&root, "TS-CY-001", "DS-CY-001", &format!("attackFeasibility: low\n{AP_FIELDS}"));
    let r = row(&root, "TS-CY-001");
    assert_eq!(r["feasibility"], "low");
    assert!(has_code(&root, "W642"));
    let ok = new_model(None);
    ds(&ok, "DS-CY-001", "damageSeverity: major\n");
    ts(&ok, "TS-CY-001", "DS-CY-001", &format!("attackFeasibility: high\n{AP_FIELDS}"));
    assert!(!has_code(&ok, "W642"));
}

#[test]
fn attack_step_factors_feed_the_attack_tree_rollup() {
    let root = new_model(None);
    ds(&root, "DS-CY-001", "damageSeverity: major\n");
    ts(&root, "TS-CY-001", "DS-CY-001", "attackFeasibility: high\n");
    write(&root, "AT-CY-001.md", "---\nid: AT-CY-001\ntype: AttackTree\nname: AT\nstatus: draft\nthreatRef: TS-CY-001\n---\n\nb\n");
    write(
        &root,
        "AT-CY-001/ATS-CY-001.md",
        &format!("---\nid: ATS-CY-001\ntype: AttackStep\nname: S\n{AP_FIELDS}---\n\nb\n"),
    );
    // single step -> tree feasibility high == declared high: no W035.
    assert!(!has_code(&root, "W035"));
    let root2 = new_model(None);
    ds(&root2, "DS-CY-001", "damageSeverity: major\n");
    ts(&root2, "TS-CY-001", "DS-CY-001", "attackFeasibility: low\n");
    write(&root2, "AT-CY-001.md", "---\nid: AT-CY-001\ntype: AttackTree\nname: AT\nstatus: draft\nthreatRef: TS-CY-001\n---\n\nb\n");
    write(
        &root2,
        "AT-CY-001/ATS-CY-001.md",
        &format!("---\nid: ATS-CY-001\ntype: AttackStep\nname: S\n{AP_FIELDS}---\n\nb\n"),
    );
    assert!(has_code(&root2, "W035"));
}

#[test]
fn per_category_impact_overall_is_the_max() {
    let root = new_model(None);
    ds(&root, "DS-CY-001", "damageSeverity: negligible\nfinancialImpact: severe\nprivacyImpact: moderate\n");
    ts(&root, "TS-CY-001", "DS-CY-001", "attackFeasibility: low\nattackVector: local\n");
    let r = row(&root, "TS-CY-001");
    assert_eq!(r["severity"], "severe");
    assert_eq!(r["risk"], "high"); // severe(3)+low(1)
    assert!(!has_code(&root, "E640"));
    // category ratings alone (no damageSeverity) are enough
    let only = new_model(None);
    ds(&only, "DS-CY-001", "safetyImpact: major\n");
    ts(&only, "TS-CY-001", "DS-CY-001", "attackFeasibility: low\n");
    assert_eq!(row(&only, "TS-CY-001")["severity"], "major");
}

#[test]
fn bad_category_value_is_e640() {
    let root = new_model(None);
    ds(&root, "DS-CY-001", "damageSeverity: major\nsafetyImpact: huge\n");
    assert!(has_code(&root, "E640"));
}

#[test]
fn malformed_cyber_entries_warn_w640_and_fall_back() {
    let root = new_model(Some(
        "[cyber]\nmethod = \"bogus\"\nfrobnicate = 1\n[cyber.risk_matrix.negligible]\nhigh = \"purple\"\nweird = \"low\"\n[cyber.cal_table.severe]\nnetwork = \"CAL9\"\n[cyber.attack_potential.thresholds]\nhigh_max = 30\nmedium_max = 20\nlow_max = 10\n",
    ));
    basic_pair(&root, "negligible", "high", "network");
    // everything fell back to defaults: rank-sum medium, no extra columns
    let r = row(&root, "TS-CY-001");
    assert_eq!(r["risk"], "medium");
    assert!(r.get("method").is_none(), "{r}");
    let (out, code) = run(&root, &["validate"]);
    assert!(out.matches("W640").count() >= 6, "{out}");
    assert_eq!(code, 0, "W640 is a warning: {out}");
}

#[test]
fn good_siblings_of_a_bad_entry_still_apply() {
    let root = new_model(Some(
        "[cyber.risk_matrix.negligible]\nhigh = \"purple\"\nlow = \"critical\"\n",
    ));
    basic_pair(&root, "negligible", "low", "network");
    assert_eq!(row(&root, "TS-CY-001")["risk"], "critical");
    assert!(has_code(&root, "W640"));
}

#[test]
fn w031_follows_the_configured_matrix() {
    // default: moderate + medium = 3 -> medium, no W031. Override to high -> W031.
    let plain = new_model(None);
    basic_pair(&plain, "moderate", "medium", "local");
    assert!(!has_code(&plain, "W031"));
    let cfgd = new_model(Some("[cyber.risk_matrix.moderate]\nmedium = \"high\"\n"));
    basic_pair(&cfgd, "moderate", "medium", "local");
    assert!(has_code(&cfgd, "W031"));
}

//! TC-TRS-PARAM-001 / GH #274: `syscribe constraints check`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct M {
    root: PathBuf,
}

impl M {
    fn new() -> M {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-cc-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
        std::fs::create_dir_all(&root).unwrap();
        let m = M { root };
        m.w("_index.md", "---\ntype: Package\nname: Root\n---\n");
        m
    }
    fn w(&self, rel: &str, c: &str) -> &M {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
        self
    }
    fn run(&self, args: &[&str]) -> (String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&self.root).args(args).output().unwrap();
        (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
    }
    fn check(&self, extra: &[&str]) -> (serde_json::Value, i32) {
        let mut a = vec!["constraints", "check", "--json"];
        a.extend(extra);
        let (o, c) = self.run(&a);
        (serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}")), c)
    }
}

fn status<'a>(v: &'a serde_json::Value, id_part: &str) -> &'a str {
    v["results"].as_array().unwrap().iter().find(|r| r["id"].as_str().unwrap().contains(id_part)).unwrap_or_else(|| panic!("{id_part} in {v}"))["status"].as_str().unwrap()
}

const MASS_DEF: &str = "---\ntype: ConstraintDef\nname: MassMargin\nfeatures:\n  - {name: totalMass, typedBy: ScalarValues::Real, direction: in}\n  - {name: maxMass, typedBy: ScalarValues::Real, direction: in, value: \"5.0\", valueKind: default-bound}\nexpression: \"totalMass <= maxMass\"\n---\n\nMass limit.\n";

fn vehicle(mass: Option<&str>) -> String {
    let value = mass.map(|m| format!("    value: \"{m}\"\n")).unwrap_or_default();
    format!("---\ntype: PartDef\nname: Vehicle\nfeatures:\n  - name: massKg\n    typedBy: ScalarValues::Real\n    unit: SI::kg\n{value}---\n\nV.\n")
}

fn usage(def: &str, right: &str) -> String {
    format!("---\ntype: Constraint\nname: Checks\nfeatures:\n  - name: checkMass\n    type: Constraint\n    typedBy: {def}\n    bindingConnections:\n      - left: totalMass\n        right: {right}\n---\n\nU.\n")
}

fn base(mass: Option<&str>) -> M {
    let m = M::new();
    m.w("Defs/MassMargin.md", MASS_DEF).w("Vehicle.md", &vehicle(mass)).w("Checks.md", &usage("Defs::MassMargin", "Vehicle::massKg"));
    m
}

#[test]
fn a_satisfied_constraint_passes_with_its_bindings_reported() {
    let (v, c) = base(Some("4.2")).check(&[]);
    assert_eq!(c, 0, "{v}");
    assert_eq!(status(&v, "checkMass"), "pass");
    let r = &v["results"][0];
    assert_eq!(r["expression"], "totalMass <= maxMass");
    assert_eq!(r["constraint"], "Defs::MassMargin");
    assert_eq!(r["bindings"]["totalMass"], "4.2", "the number as written: ScalarValues drops the unit");
    assert_eq!(r["bindings"]["maxMass"], "5", "the default");
    assert_eq!(v["summary"]["pass"], 1);
}

#[test]
fn a_violated_constraint_fails_with_exit_two() {
    let (v, c) = base(Some("6.1")).check(&[]);
    assert_eq!(c, 2);
    assert_eq!(status(&v, "checkMass"), "fail");
    assert_eq!(v["summary"]["fail"], 1);
}

#[test]
fn a_missing_input_is_unresolved_not_a_failure_unless_strict() {
    let m = base(None);
    let (v, c) = m.check(&[]);
    assert_eq!(c, 0, "{v}");
    assert_eq!(status(&v, "checkMass"), "unresolved");
    assert_eq!(v["results"][0]["unresolved"][0], "Vehicle::massKg");
    assert_eq!(m.check(&["--strict"]).1, 2);
}

#[test]
fn a_bad_expression_is_an_error_row_not_a_crash() {
    let m = base(Some("4"));
    m.w("Defs/MassMargin.md", &MASS_DEF.replace("totalMass <= maxMass", "totalMass <="));
    let (v, c) = m.check(&[]);
    assert_eq!(c, 2);
    assert_eq!(status(&v, "checkMass"), "error");
    assert!(v["results"][0]["message"].as_str().unwrap().to_lowercase().contains("parse"), "{v}");
}

#[test]
fn bindings_may_be_expressions_over_several_attributes() {
    let m = base(Some("2"));
    m.w("Checks.md", &usage("Defs::MassMargin", "Vehicle::massKg * 2 + 1.5 kg"));
    let (v, c) = m.check(&[]);
    assert_eq!(c, 2, "5.5 > 5 fails");
    assert_eq!(v["results"][0]["bindings"]["totalMass"], "5.5", "{v}");
    assert_eq!(status(&v, "checkMass"), "fail", "5.5 > 5");
}

#[test]
fn scalar_parameters_drop_units_and_quantity_parameters_are_dimension_checked() {
    // the UAV battery pattern: a J-valued attribute against a unitless default
    let m = M::new();
    m.w("Defs/Energy.md", "---\ntype: ConstraintDef\nname: Energy\nfeatures:\n  - {name: cap, typedBy: ScalarValues::Real, direction: in}\n  - {name: minCap, typedBy: ScalarValues::Real, direction: in, value: \"50.0\", valueKind: default-bound}\nexpression: \"cap >= minCap\"\n---\n\nE.\n");
    m.w("Battery.md", "---\ntype: PartDef\nname: Battery\nfeatures:\n  - {name: cap, typedBy: ScalarValues::Real, unit: SI::J, value: \"133.56\"}\n---\n\nB.\n");
    m.w("Checks.md", "---\ntype: Constraint\nname: Checks\nfeatures:\n  - name: c\n    type: Constraint\n    typedBy: Defs::Energy\n    bindingConnections:\n      - {left: cap, right: Battery::cap}\n---\n\nU.\n");
    let (v, c) = m.check(&[]);
    assert_eq!((c, status(&v, "Checks")), (0, "pass"), "{v}");
    // ISQ-typed parameters see dimensions: mass in kg against a limit in kg passes, against a time fails with a dimension error
    let q = M::new();
    q.w("Defs/Mass.md", "---\ntype: ConstraintDef\nname: Mass\nfeatures:\n  - {name: m, typedBy: ISQ::MassValue, direction: in}\n  - {name: limit, typedBy: ISQ::MassValue, direction: in, unit: SI::kg, value: \"5.0\", valueKind: default-bound}\nexpression: \"m <= limit\"\n---\n\nM.\n");
    q.w("Vehicle.md", &vehicle(Some("4.2")));
    q.w("Checks.md", &usage("Defs::Mass", "Vehicle::massKg").replace("left: totalMass", "left: m"));
    let (v, c) = q.check(&[]);
    assert_eq!((c, status(&v, "checkMass")), (0, "pass"), "{v}");
    q.w("Time.md", "---\ntype: PartDef\nname: Time\nfeatures:\n  - {name: t, typedBy: ScalarValues::Real, unit: SI::s, value: \"3\"}\n---\n\nT.\n");
    q.w("Checks.md", &usage("Defs::Mass", "Time::t").replace("left: totalMass", "left: m"));
    let (v, c) = q.check(&[]);
    assert_eq!(c, 2, "{v}");
    assert_eq!(status(&v, "checkMass"), "error");
    assert!(v["results"][0]["message"].as_str().unwrap().to_lowercase().contains("dimension"), "{v}");
}

#[test]
fn requires_clauses_are_checked_and_negation_inverts() {
    let m = base(Some("4"));
    m.w("REQ-CC-001.md", "---\nid: REQ-CC-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\nrequires:\n  - expression: \"Vehicle::massKg <= 5 kg\"\n  - expression: \"Vehicle::massKg > 9 kg\"\n  - expression: \"Vehicle::massKg <= 5\"\nassume:\n  - expression: \"Vehicle::massKg > 0 kg\"\n---\n\nShall.\n");
    let (v, c) = m.check(&[]);
    assert_eq!(c, 2);
    let rows: Vec<_> = v["results"].as_array().unwrap().iter().filter(|r| r["id"].as_str().unwrap().contains("REQ-CC-001")).collect();
    assert_eq!(rows.len(), 4, "{v}");
    let dim = rows.iter().find(|r| r["expression"] == "Vehicle::massKg <= 5").unwrap();
    assert_eq!(dim["status"], "error", "a mass against a bare number is a dimension error: {v}");
    assert!(dim["message"].as_str().unwrap().to_lowercase().contains("dimension"));
    assert!(rows.iter().any(|r| r["kind"] == "requires" && r["status"] == "pass"));
    assert!(rows.iter().any(|r| r["kind"] == "requires" && r["status"] == "fail"));
    assert!(rows.iter().any(|r| r["kind"] == "assume" && r["status"] == "pass"));
    // isNegated
    let n = base(Some("6"));
    n.w("Checks.md", &usage("Defs::MassMargin", "Vehicle::massKg").replace("    type: Constraint\n", "    type: Constraint\n    isNegated: true\n"));
    let (v, c) = n.check(&[]);
    assert_eq!((c, status(&v, "checkMass")), (0, "pass"), "6 > 5, negated: holds. {v}");
}

fn configured() -> M {
    let m = M::new();
    m.w("Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    m.w("Features/Battery.md", "---\ntype: FeatureDef\nid: FEAT-CC-100\nname: Battery\ngroupKind: optional\nparameters:\n  - {name: capacity, type: ScalarValues::Real}\n---\n\nB.\n");
    for (id, v) in [("CONF-CC-SMALL-001", "40"), ("CONF-CC-LARGE-001", "90")] {
        m.w(&format!("Configs/{id}.md"), &format!("---\nid: {id}\ntype: Configuration\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Battery: true\nparameterBindings:\n  Features::Battery.capacity: {v}\n---\n\nC.\n"));
    }
    m.w("Defs/Energy.md", "---\ntype: ConstraintDef\nname: Energy\nfeatures:\n  - {name: cap, typedBy: ScalarValues::Real, direction: in}\n  - {name: minCap, typedBy: ScalarValues::Real, direction: in, value: \"50.0\", valueKind: default-bound}\nexpression: \"cap >= minCap\"\n---\n\nE.\n");
    m.w("Checks.md", "---\ntype: Constraint\nname: Checks\nfeatures:\n  - name: c\n    type: Constraint\n    typedBy: Defs::Energy\n    bindingConnections:\n      - {left: cap, right: \"Features::Battery.capacity\"}\n---\n\nU.\n");
    m
}

#[test]
fn a_configurations_parameter_bindings_feed_the_constraints() {
    let m = configured();
    let (v, c) = m.check(&["--config", "CONF-CC-LARGE-001"]);
    assert_eq!((c, status(&v, "Checks")), (0, "pass"), "{v}");
    let (v, c) = m.check(&["--config", "CONF-CC-SMALL-001"]);
    assert_eq!((c, status(&v, "Checks")), (2, "fail"), "{v}");
    let (v, c) = m.check(&[]);
    assert_eq!((c, status(&v, "Checks")), (0, "unresolved"), "no configuration, no binding: {v}");
}

#[test]
fn all_configs_tabulates_every_configuration() {
    let m = configured();
    let (v, c) = m.check(&["--all-configs"]);
    assert_eq!(c, 2, "{v}");
    let cfgs = v["configurations"].as_array().unwrap();
    assert_eq!(cfgs.len(), 2);
    let sum = |id: &str| cfgs.iter().find(|x| x["id"] == id).unwrap()["summary"].clone();
    assert_eq!(sum("CONF-CC-LARGE-001")["pass"], 1);
    assert_eq!(sum("CONF-CC-SMALL-001")["fail"], 1);
    let (t, _) = m.run(&["constraints", "check", "--all-configs"]);
    assert!(t.contains("CONF-CC-LARGE-001") && t.contains("CONF-CC-SMALL-001") && t.contains("pass") && t.contains("fail"), "{t}");
}

#[test]
fn set_overrides_everything_and_usage_errors_exit_one() {
    let m = base(Some("4.2"));
    let (v, c) = m.check(&["--set", "Vehicle::massKg=9"]);
    assert_eq!((c, status(&v, "checkMass")), (2, "fail"), "{v}");
    let (o, c) = m.run(&["constraints", "check", "--config", "CONF-NOPE"]);
    assert_eq!(c, 1, "{o}");
    let (o, c) = m.run(&["constraints", "check", "--bogus"]);
    assert_eq!(c, 1, "{o}");
    let (o, c) = m.run(&["constraints", "check", "--set", "novalue"]);
    assert_eq!(c, 1, "{o}");
    let (o, c) = m.run(&["constraints"]);
    assert_eq!(c, 1, "{o}");
}

#[test]
fn a_model_without_constraints_reports_nothing_to_check() {
    let m = M::new();
    let (v, c) = m.check(&[]);
    assert_eq!(c, 0);
    assert_eq!(v["results"].as_array().unwrap().len(), 0);
}

const RANGE_CALC: &str = "---\ntype: CalculationDef\nname: Range\nparameters:\n  - {name: energy, typedBy: ISQ::EnergyValue, direction: in}\n  - {name: power, typedBy: ScalarValues::Real, direction: in, unit: SI::W, value: \"50\"}\nreturnType: ScalarValues::Real\nbody: \"energy / power\"\n---\n\nEndurance.\n";

fn calc(m: &M, args: &[&str]) -> (serde_json::Value, i32) {
    let mut a = vec!["constraints", "calc", "--json"];
    a.extend(args);
    let (o, c) = m.run(&a);
    (serde_json::from_str(&o).unwrap_or_else(|_| serde_json::json!({"raw": o})), c)
}

#[test]
fn calc_evaluates_a_calculation_body_with_defaults_and_overrides() {
    let m = M::new();
    m.w("Defs/Range.md", RANGE_CALC);
    let (v, c) = calc(&m, &["Defs::Range", "--set", "energy=100Wh"]);
    assert_eq!(c, 0, "{v}");
    assert!((v["value"].as_f64().unwrap() - 7200.0).abs() < 1e-6, "100 Wh / 50 W = 2 h = 7200 s: {v}");
    assert_eq!(v["unit"], "s");
    assert_eq!(v["calculation"], "Defs::Range");
    let (v, _) = calc(&m, &["Defs::Range", "--set", "energy=100Wh", "--set", "power=100W"]);
    assert!((v["value"].as_f64().unwrap() - 3600.0).abs() < 1e-6, "{v}");
}

#[test]
fn calc_bodies_can_read_attributes_and_report_missing_inputs() {
    let m = M::new();
    m.w("Defs/Twice.md", "---\ntype: CalculationDef\nname: Twice\nbody: \"Vehicle::massKg * 2\"\n---\n\nT.\n");
    m.w("Vehicle.md", &vehicle(Some("4")));
    let (v, c) = calc(&m, &["Defs::Twice"]);
    assert_eq!((c, v["value"].as_f64()), (0, Some(8.0)), "{v}");
    let n = M::new();
    n.w("Defs/Range.md", RANGE_CALC);
    let (v, c) = calc(&n, &["Defs::Range"]);
    assert_eq!(c, 2, "energy has no value: {v}");
    assert!(v.to_string().contains("energy"), "{v}");
}

#[test]
fn calc_refuses_what_is_not_an_evaluable_calculation() {
    let m = M::new();
    m.w("Defs/MassMargin.md", MASS_DEF);
    m.w("Defs/Opaque.md", "---\ntype: CalculationDef\nname: Opaque\nbody: \"return x;\"\nbodyLanguage: alf\n---\n\nO.\n");
    assert_eq!(calc(&m, &["Defs::MassMargin"]).1, 1, "a ConstraintDef is not a calculation");
    assert_eq!(calc(&m, &["Defs::Opaque"]).1, 1, "alf is opaque");
    assert_eq!(calc(&m, &["Defs::Nope"]).1, 1);
    assert_eq!(calc(&m, &[]).1, 1);
}

fn sweep(m: &M, args: &[&str]) -> (serde_json::Value, i32) {
    let mut a = vec!["constraints", "sweep", "--json"];
    a.extend(args);
    let (o, c) = m.run(&a);
    (serde_json::from_str(&o).unwrap_or_else(|_| serde_json::json!({"raw": o})), c)
}

#[test]
fn sweep_evaluates_the_checks_over_the_grid() {
    let m = base(Some("4"));
    let (v, c) = sweep(&m, &["--param", "Vehicle::massKg=3..7:1"]);
    assert_eq!(c, 0, "{v}");
    let pts = v["points"].as_array().unwrap();
    assert_eq!(pts.len(), 5, "{v}");
    let passes: Vec<i64> = pts.iter().map(|p| p["summary"]["pass"].as_i64().unwrap()).collect();
    assert_eq!(passes, vec![1, 1, 1, 0, 0], "5 is within the limit, 6 and 7 are not: {v}");
    assert_eq!(pts[0]["values"]["Vehicle::massKg"], "3");
    let (v, _) = sweep(&m, &["--param", "Vehicle::massKg=3kg..7kg:2kg", "--show", "Vehicle::massKg * 2"]);
    let pts = v["points"].as_array().unwrap();
    assert_eq!(pts.len(), 3);
    assert_eq!(pts[1]["show"], "10 kg", "{v}");
}

#[test]
fn sweep_takes_lists_and_a_cartesian_product() {
    let m = base(Some("4"));
    let (v, _) = sweep(&m, &["--param", "Vehicle::massKg=1,5,9"]);
    assert_eq!(v["points"].as_array().unwrap().len(), 3);
    m.w("Limit.md", "---\ntype: PartDef\nname: Limit\nfeatures:\n  - {name: v, typedBy: ScalarValues::Real, value: \"5\"}\n---\n\nL.\n");
    let (v, _) = sweep(&m, &["--param", "Vehicle::massKg=1,2", "--param", "Limit::v=10,20,30"]);
    assert_eq!(v["points"].as_array().unwrap().len(), 6);
}

#[test]
fn sweep_rejects_bad_ranges_and_huge_grids() {
    let m = base(Some("4"));
    for bad in ["Vehicle::massKg=3..7", "Vehicle::massKg=3..7:0", "Vehicle::massKg=7..3:1", "Vehicle::massKg=3kg..7s:1kg", "nonsense", "Vehicle::massKg=1..100000:1"] {
        let (v, c) = sweep(&m, &["--param", bad]);
        assert_eq!(c, 1, "{bad}: {v}");
    }
    assert_eq!(sweep(&m, &[]).1, 1, "at least one --param");
}

#[allow(dead_code)]
fn unused(_: &Path) {}

//! GH #489–#492: function calls in `parameterConstraints`, derived parameters, calculation
//! composition and `buildVar` on a `CalculationDef` (`REQ-TRS-PCFUNC-001`, `REQ-TRS-DERIVED-001`,
//! `REQ-TRS-CALCCOMP-001`, `REQ-TRS-CALCBUILD-001`).

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct M {
    root: PathBuf,
}

impl M {
    fn new(constraints: &str, extra_params: &str) -> M {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-dc-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
        std::fs::create_dir_all(&root).unwrap();
        let m = M { root };
        m.w("_index.md", "---\ntype: Package\nname: Root\n---\n");
        m.w(
            "Features/Disk.md",
            &format!(
                "---\ntype: FeatureDef\nid: FEAT-DISK\nname: Disk\nmandatory: true\nparameters:\n  - {{ name: slotMiB, type: ScalarValues::Integer, default: 1000, buildVar: SLOT_MIB }}\n  - {{ name: superMiB, type: ScalarValues::Integer, derivedFrom: \"Features::Disk.slotMiB * 2\", buildVar: SUPER_MIB }}\n{extra_params}---\n"
            ),
        );
        m.w("Features/_index.md", &format!("---\ntype: Package\nname: Features\nparameterConstraints:\n{constraints}---\n"));
        m.w("Configs/CONF-DEMO-001.md", "---\ntype: Configuration\nid: CONF-DEMO-001\nname: Demo\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Disk: true\n---\n");
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
}

const CONSTRAINTS: &str = "  - { id: PC-PLAIN, expression: \"Features::Disk.slotMiB >= 3000\" }\n  - { id: PC-FUNC, expression: \"max(Features::Disk.slotMiB, 1) >= 3000\" }\n";

/// #489: an expression with a function call is judged, not left undetermined.
#[test]
fn a_function_call_in_a_constraint_is_evaluated_and_fails() {
    let m = M::new(CONSTRAINTS, "");
    let (out, code) = m.run(&["feature-check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("'PC-PLAIN'") && out.contains("'PC-FUNC'"), "{out}");
    let (out, _) = m.run(&["residual", "--config", "CONF-DEMO-001"]);
    assert!(out.contains("PC-FUNC") && !out.contains("undetermined"), "{out}");
    let (out, code) = m.run(&["validate"]);
    assert!(out.contains("PC-FUNC") && code != 0, "{out}");
}

#[test]
fn a_true_function_constraint_passes_and_an_unevaluable_one_is_an_error() {
    let ok = M::new("  - { id: PC-OK, expression: \"ceil(Features::Disk.slotMiB / 3) <= 400\" }\n", "");
    let (out, code) = ok.run(&["feature-check"]);
    assert_eq!(code, 0, "{out}");
    // Every parameter has a value but the expression has no truth value: an error, never a silent pass.
    let bad = M::new("  - { id: PC-BAD, expression: \"max(Features::Disk.slotMiB, 1)\" }\n", "");
    let (out, code) = bad.run(&["feature-check"]);
    assert!(out.contains("E250") && code != 0, "{out}");
}

/// #490: a derived parameter is computed and used everywhere a bound value is.
#[test]
fn a_derived_parameter_is_computed_exported_and_constrained() {
    let m = M::new("  - { id: PC-DERIVED, expression: \"Features::Disk.superMiB <= 1500\" }\n", "");
    let (out, _) = m.run(&["show", "CONF-DEMO-001"]);
    assert!(out.contains("| Features::Disk.superMiB | 2000 | derived:"), "{out}");
    let (out, code) = m.run(&["feature-check"]);
    assert!(out.contains("'PC-DERIVED'") && code == 1, "{out}");
    let m = M::new("  - { id: PC-DERIVED, expression: \"Features::Disk.superMiB <= 2500\" }\n", "");
    let (out, code) = m.run(&["build-config", "--config", "CONF-DEMO-001", "--format", "env"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("export SLOT_MIB=1000") && out.contains("export SUPER_MIB=2000"), "{out}");
}

#[test]
fn a_derived_value_is_range_and_integer_checked_and_a_cycle_is_an_error() {
    let m = M::new("", "  - { name: thirdMiB, type: ScalarValues::Integer, derivedFrom: \"Features::Disk.slotMiB / 3\", range: \"0..400\" }\n  - { name: bigMiB, type: ScalarValues::Real, derivedFrom: \"Features::Disk.slotMiB * 2\", range: \"0..1500\" }\n");
    let (out, code) = m.run(&["feature-check"]);
    assert!(out.contains("E249") && out.contains("thirdMiB") && out.contains("not a whole number"), "{out}");
    assert!(out.contains("bigMiB") && out.contains("outside range"), "{out}");
    assert_ne!(code, 0);
    let m = M::new("", "  - { name: a, type: ScalarValues::Integer, derivedFrom: \"Features::Disk.b + 1\" }\n  - { name: b, type: ScalarValues::Integer, derivedFrom: \"Features::Disk.a + 1\" }\n");
    let (out, _) = m.run(&["feature-check"]);
    assert!(out.contains("E207"), "{out}");
}

/// #491/#492: calculations compose, are exported through buildVar and are not "unused".
fn calc_model() -> M {
    let m = M::new("", "");
    m.w("Calc/SuperMiB.md", "---\ntype: CalculationDef\nname: SuperMiB\nbody: \"Features::Disk.slotMiB * 2\"\nbuildVar: CALC_SUPER\nintegral: true\n---\n");
    m.w("Calc/TotalMiB.md", "---\ntype: CalculationDef\nname: TotalMiB\nbody: \"Calc::SuperMiB + 1\"\nbuildVar: CALC_TOTAL\n---\n");
    m.w("Calc/Scaled.md", "---\ntype: CalculationDef\nname: Scaled\nparameters:\n  - { name: k, typedBy: ScalarValues::Integer }\nbody: \"Features::Disk.slotMiB * k\"\n---\n");
    m.w("Calc/UsesCall.md", "---\ntype: CalculationDef\nname: UsesCall\nbody: \"Calc::Scaled(k = 3) + Calc::SuperMiB\"\nbuildVar: CALC_CALL\n---\n");
    m
}

#[test]
fn a_calculation_names_and_calls_other_calculations() {
    let m = calc_model();
    let (out, code) = m.run(&["constraints", "calc", "Calc::TotalMiB", "--config", "CONF-DEMO-001"]);
    assert_eq!((out.trim(), code), ("2001", 0));
    let (out, code) = m.run(&["constraints", "calc", "Calc::UsesCall", "--config", "CONF-DEMO-001"]);
    assert_eq!((out.trim(), code), ("5000", 0), "{out}");
    let (out, _) = m.run(&["validate"]);
    assert!(!out.contains("W007"), "a composed or exported calculation is used: {out}");
}

#[test]
fn a_calculation_cycle_and_a_bad_argument_are_errors() {
    let m = calc_model();
    m.w("Calc/CycA.md", "---\ntype: CalculationDef\nname: CycA\nbody: \"Calc::CycB + 1\"\n---\n");
    m.w("Calc/CycB.md", "---\ntype: CalculationDef\nname: CycB\nbody: \"Calc::CycA + 1\"\n---\n");
    let (out, code) = m.run(&["constraints", "calc", "Calc::CycA", "--config", "CONF-DEMO-001"]);
    assert!(code != 0 && out.contains("circular calculation"), "{out}");
    m.w("Calc/BadArg.md", "---\ntype: CalculationDef\nname: BadArg\nbody: \"Calc::Scaled(nope = 1)\"\n---\n");
    let (out, code) = m.run(&["constraints", "calc", "Calc::BadArg", "--config", "CONF-DEMO-001"]);
    assert!(code != 0 && out.contains("no parameter 'nope'"), "{out}");
}

#[test]
fn build_config_exports_calculation_results_in_every_format() {
    let m = calc_model();
    let (out, code) = m.run(&["build-config", "--config", "CONF-DEMO-001", "--format", "env"]);
    assert_eq!(code, 0, "{out}");
    for want in ["export CALC_SUPER=2000", "export CALC_TOTAL=2001", "export CALC_CALL=5000"] {
        assert!(out.contains(want), "{want} in {out}");
    }
    let (out, _) = m.run(&["build-config", "--config", "CONF-DEMO-001", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["vars"]["CALC_TOTAL"], 2001);
    for f in ["makefile", "cmake", "c-header", "kconfig"] {
        let (out, code) = m.run(&["build-config", "--config", "CONF-DEMO-001", "--format", f]);
        assert!(code == 0 && out.contains("CALC_TOTAL"), "{f}: {out}");
    }
}

#[test]
fn build_config_unit_integral_failure_and_conflict() {
    let m = calc_model();
    m.w("Calc/Mass.md", "---\ntype: CalculationDef\nname: Mass\nbody: \"2 kg\"\nunit: g\nbuildVar: CALC_UNIT\n---\n");
    m.w("Calc/Frac.md", "---\ntype: CalculationDef\nname: Frac\nbody: \"Features::Disk.slotMiB / 3\"\nintegral: true\nbuildVar: CALC_FRAC\n---\n");
    let (out, code) = m.run(&["build-config", "--config", "CONF-DEMO-001", "--format", "env", "--no-validate"]);
    assert!(out.contains("E251") && out.contains("CALC_FRAC") && code != 0, "an integral calculation with a fractional result: {out}");
    assert!(!out.contains("CALC_UNIT"), "a failing build never prints variables: {out}");
    m.w("Calc/Frac.md", "---\ntype: CalculationDef\nname: Frac\nbody: \"Features::Disk.slotMiB / 4\"\nintegral: true\nbuildVar: CALC_FRAC\n---\n");
    let (out, code) = m.run(&["build-config", "--config", "CONF-DEMO-001", "--format", "env", "--no-validate"]);
    assert!(code == 0 && out.contains("export CALC_UNIT=2000") && out.contains("export CALC_FRAC=250"), "the result in the declared unit: {out}");
    m.w("Calc/Frac.md", "---\ntype: CalculationDef\nname: Frac\nbody: \"Features::Disk.slotMiB / 4\"\nintegral: true\nbuildVar: CALC_SUPER\n---\n");
    let (out, code) = m.run(&["build-config", "--config", "CONF-DEMO-001", "--format", "env", "--no-validate"]);
    assert!(out.contains("E050") && out.contains("CALC_SUPER") && code != 0, "two exports of one variable: {out}");
}

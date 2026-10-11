//! TC-TRS-FMI-001 / REQ-TRS-FMI-001 (GH #291): the FMI backend against the vendored Reference FMUs
//! (BSD-2, see tests/fmus/LICENSE.txt). Run with `cargo test` in this crate.

use std::path::PathBuf;

use syscribe_fmi::{parse_model_description, FmiBackend};
use syscribe_model::expr::Value;
use syscribe_model::ir::lower;
use syscribe_model::plant::{Causality, CoSim, Plant, PlantBackend, VarType};
use syscribe_model::sm_run::{Options, Run};
use syscribe_model::units::Dim;
use syscribe_model::walker::walk_model;

fn fmu(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fmus").join(name)
}

fn sha_of(name: &str) -> String {
    let sums = std::fs::read_to_string(fmu("SHA256SUMS")).unwrap();
    sums.lines().find(|l| l.ends_with(name)).unwrap().split_whitespace().next().unwrap().to_string()
}

const LENGTH: Dim = Dim([1, 0, 0, 0, 0, 0, 0]);

#[test]
fn inspect_reads_version_units_causality_and_the_hash_for_fmi_2_and_3() {
    for (file, version) in [("BouncingBall-2.0.fmu", "2.0"), ("BouncingBall-3.0.fmu", "3.0")] {
        let info = FmiBackend.inspect(&fmu(file)).unwrap();
        assert_eq!(info.fmi_version, version);
        assert!(info.supports_cosimulation);
        assert_eq!(info.sha256, sha_of(file), "the sha256 pin matches the checksum file");
        let h = info.var("h").unwrap();
        assert_eq!((h.causality, h.ty, h.dim), (Causality::Output, VarType::Real, Some(LENGTH)), "{file}");
        assert!((h.factor - 1.0).abs() < 1e-12);
        assert_eq!(info.var("v").unwrap().dim, Some(Dim([1, 0, -1, 0, 0, 0, 0])), "m/s from the BaseUnit exponents");
        let g = info.var("g").unwrap();
        assert_eq!((g.causality, g.dim), (Causality::Parameter, Some(Dim([1, 0, -2, 0, 0, 0, 0]))));
        assert_eq!(info.var("e").unwrap().dim, None, "no unit, no dimension");
        assert!(!info.guid.is_empty());
    }
}

#[test]
fn feedthrough_exposes_real_integer_and_boolean_inputs_and_outputs() {
    let info = FmiBackend.inspect(&fmu("Feedthrough-3.0.fmu")).unwrap();
    assert_eq!(info.var("Float64_continuous_input").unwrap().causality, Causality::Input);
    assert_eq!(info.var("Int32_output").unwrap().ty, VarType::Integer);
    assert_eq!(info.var("Boolean_input").unwrap().ty, VarType::Boolean);
    assert_eq!(info.var("Float64_fixed_parameter").unwrap().causality, Causality::Parameter);
}

#[test]
fn the_reference_trajectory_is_reproduced_for_both_fmi_versions() {
    for file in ["BouncingBall-2.0.fmu", "BouncingBall-3.0.fmu"] {
        let mut p = FmiBackend.instantiate(&fmu(file), &[]).unwrap();
        let (mut t, dt) = (0.0, 0.01);
        let mut first = None;
        while t < 1.0 {
            p.step(t, dt).unwrap();
            t += dt;
            let h = p.get("h").unwrap();
            if first.is_none() && h < 0.1 && t > 0.1 {
                first = Some((t, h, p.get("v").unwrap()));
            }
        }
        let (t1, h1, v1) = first.expect("the ball falls below 0.1 m");
        assert!((t1 - 0.43).abs() < 1e-9 && (h1 - 0.09517465).abs() < 1e-7 && (v1 + 4.2183).abs() < 1e-3, "{file}: {t1} {h1} {v1}");
    }
}

#[test]
fn parameters_are_applied_before_the_run() {
    let peak_after_bounce = |e: f64| {
        let mut p = FmiBackend.instantiate(&fmu("BouncingBall-2.0.fmu"), &[("e".to_string(), e)]).unwrap();
        let (mut t, mut peak) = (0.0, 0.0f64);
        while t < 1.2 {
            p.step(t, 0.01).unwrap();
            t += 0.01;
            if t > 0.47 {
                peak = peak.max(p.get("h").unwrap());
            }
        }
        peak
    };
    let (hi, lo) = (peak_after_bounce(0.7), peak_after_bounce(0.5));
    assert!((hi - 0.49).abs() < 0.04 && (lo - 0.25).abs() < 0.04, "peaks {hi} {lo} (≈ e² × 1 m)");
}

#[test]
fn types_cross_the_boundary_through_feedthrough() {
    let mut p = FmiBackend.instantiate(&fmu("Feedthrough-3.0.fmu"), &[]).unwrap();
    p.set("Float64_continuous_input", 2.5).unwrap();
    p.set("Int32_input", 7.0).unwrap();
    p.set("Boolean_input", 1.0).unwrap();
    p.step(0.0, 0.1).unwrap();
    assert_eq!(p.get("Float64_continuous_output").unwrap(), 2.5);
    assert_eq!(p.get("Int32_output").unwrap(), 7.0);
    assert_eq!(p.get("Boolean_output").unwrap(), 1.0);
}

#[test]
fn provenance_identifies_the_fmu_the_backend_and_the_platform() {
    let p = FmiBackend.instantiate(&fmu("BouncingBall-3.0.fmu"), &[]).unwrap();
    let j = p.provenance();
    assert_eq!(j["kind"], "fmi");
    assert_eq!(j["fmiVersion"], "3.0");
    assert_eq!(j["mode"], "cosimulation");
    assert_eq!(j["sha256"], sha_of("BouncingBall-3.0.fmu"));
    assert!(j["backend"].as_str().unwrap().contains("fmi crate"));
    assert!(j["platform"].as_str().unwrap().contains(std::env::consts::OS));
}

#[test]
fn bad_inputs_are_errors_not_panics() {
    let dir = std::env::temp_dir().join(format!("syscribe-fmi-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("junk.fmu"), "not a zip").unwrap();
    assert!(FmiBackend.inspect(&dir.join("junk.fmu")).is_err());
    assert!(FmiBackend.inspect(&dir.join("missing.fmu")).is_err());
    assert!(FmiBackend.instantiate(&fmu("BouncingBall-2.0.fmu"), &[("nope".to_string(), 1.0)]).is_err());
    let mut p = FmiBackend.instantiate(&fmu("BouncingBall-2.0.fmu"), &[]).unwrap();
    assert!(p.get("nope").is_err());
}

#[test]
fn variable_level_units_and_unit_fallbacks_are_read() {
    // fmi-schema 0.8 drops `unit` on a ScalarVariable's Real; the backend reads it
    let xml = r#"<?xml version="1.0"?>
<fmiModelDescription fmiVersion="2.0" modelName="X" guid="{g}">
  <CoSimulation modelIdentifier="X"/>
  <UnitDefinitions>
    <Unit name="kN"><BaseUnit kg="1" m="1" s="-2" factor="1000"/></Unit>
    <Unit name="degC"><BaseUnit K="1" offset="273.15"/></Unit>
  </UnitDefinitions>
  <ModelVariables>
    <ScalarVariable name="f" valueReference="1" causality="output"><Real unit="kN"/></ScalarVariable>
    <ScalarVariable name="t" valueReference="2" causality="output"><Real unit="degC"/></ScalarVariable>
    <ScalarVariable name="m" valueReference="3" causality="input"><Real unit="kg"/></ScalarVariable>
  </ModelVariables>
</fmiModelDescription>"#;
    let md = parse_model_description(xml).unwrap();
    let v = |n: &str| md.variables.iter().find(|x| x.name == n).unwrap().info.clone();
    assert_eq!((v("f").dim, v("f").factor), (Some(Dim([1, 1, -2, 0, 0, 0, 0])), 1000.0));
    assert_eq!((v("t").dim, v("t").offset), (Some(Dim([0, 0, 0, 0, 1, 0, 0])), 273.15));
    assert_eq!(v("m").dim, Some(Dim([0, 1, 0, 0, 0, 0, 0])), "a unit with no definition falls back to syscribe's own unit table");
    assert!(parse_model_description("<nothing/>").is_err());
}

#[test]
fn a_state_machine_reacts_to_the_real_bouncing_ball() {
    for file in ["BouncingBall-2.0.fmu", "BouncingBall-3.0.fmu"] {
        let dir = std::env::temp_dir().join(format!("syscribe-fmi-sm-{}-{file}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
        std::fs::write(
            dir.join("M.md"),
            "---\ntype: StateDef\nname: M\nvariables:\n  - {name: height, value: \"0\", unit: \"SI::m\"}\nsubStates:\n  - name: Falling\n    isInitial: true\n    transitions:\n      - {target: Bounced, accept: {when: \"height < 0.1 m\"}}\n  - name: Bounced\n---\n",
        )
        .unwrap();
        let m = lower(&walk_model(&dir).unwrap(), "M").unwrap();
        let plant = FmiBackend.instantiate(&fmu(file), &[]).unwrap();
        let cosim = CoSim::new(vec![("ball".to_string(), plant)], vec![("ball.h".to_string(), "height".to_string())], 0.01, &m).unwrap();
        let mut r = Run::with_plants(&m, Options::default(), cosim);
        assert_eq!(r.var("height"), Some(Value::Num(syscribe_model::expr::Quantity { v: 1.0, dim: LENGTH })), "initial output read before the first settle");
        r.advance(1.0);
        assert_eq!(r.active(), ["Bounced"], "{file}");
        let at = r.trace().iter().find(|s| s.cause.starts_with("change")).unwrap().time;
        assert!((at - 0.43).abs() < 1e-9, "{file}: fired at {at}");
        assert_eq!(r.plant_provenance()[0]["plant"]["fmiVersion"], &file[13..16]);
    }
}

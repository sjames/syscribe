//! TC-TRS-PLANT-001 / REQ-TRS-PLANT-001 (GH #340): in-process continuous plants — a `PlantDef` written in the expression
//! language, or a library plant — run by the composite engine through the same seam as an FMU, with no FMI backend.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::config::ValidateConfig;
use syscribe_model::expr::Value;
use syscribe_model::sm_run::Options;
use syscribe_model::system_run::{System, SystemModel};
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

const CTL: &str = "---
type: StateDef
name: CtlM
variables:
  - {name: u, value: \"1\"}
  - {name: y, value: \"0\"}
subStates:
  - {name: Run, isInitial: true}
---
";

fn lag(extra: &str) -> String {
    format!(
        "---
type: PlantDef
name: Lag
parameters:
  - {{name: k, value: \"1\"}}
  - {{name: tau, value: \"200 ms\"}}
inputs:
  - {{name: u}}
outputs:
  - {{name: y}}
states:
  - {{name: x, value: \"0\"}}
equations:
  - \"der(x) = (k * u - x) / tau\"
  - \"y := x\"
{extra}---
A first-order lag.
"
    )
}

/// A model with the controller, `PlantP` (whose frontmatter is `plant_part`), `Sys` (controller + plant at `step`) and
/// the extra files given.
fn world(plant_part: &str, step: Option<&str>, files: &[(&str, String)]) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-plantdef-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("CtlM.md"), CTL).unwrap();
    std::fs::write(root.join("CtlP.md"), "---\ntype: PartDef\nname: CtlP\nexhibitsStates: [CtlM]\n---\nController.\n").unwrap();
    std::fs::write(root.join("PlantP.md"), format!("---\ntype: PartDef\nname: PlantP\n{plant_part}---\nThe plant.\n")).unwrap();
    let step = step.map(|s| format!("plantStep: \"{s}\"\n")).unwrap_or_default();
    std::fs::write(
        root.join("Sys.md"),
        format!("---\ntype: PartDef\nname: Sys\n{step}features:\n  - {{name: ctl, typedBy: CtlP}}\n  - {{name: p, typedBy: PlantP}}\nbindingConnections:\n  - {{from: ctl.u, to: p.u}}\n  - {{from: p.y, to: ctl.y}}\n---\nThe system.\n"),
    )
    .unwrap();
    for (name, text) in files {
        std::fs::write(root.join(format!("{name}.md")), text).unwrap();
    }
    root
}

fn build(root: &PathBuf) -> Result<SystemModel, Vec<String>> {
    SystemModel::build_in(&walk_model(root).unwrap(), "Sys", root)
}

fn num(v: Value) -> f64 {
    match v {
        Value::Num(q) => q.v,
        other => panic!("not a number: {other:?}"),
    }
}

/// `ctl.y` after `secs` of the composite at `root`, with `u` set first.
fn y_after(root: &PathBuf, u: f64, secs: f64) -> f64 {
    let m = build(root).unwrap_or_else(|e| panic!("{e:?}"));
    let mut s = System::new(&m, Options::default());
    // `u` keeps the dimension the machine declares
    let Value::Num(mut q) = s.var("ctl", "u").unwrap() else { panic!() };
    q.v = u;
    s.set("ctl", "u", Value::Num(q));
    s.advance(secs);
    assert!(s.error().is_none(), "{:?}", s.error());
    num(s.var("ctl", "y").unwrap())
}

fn codes(root: &PathBuf) -> Vec<String> {
    let elems = walk_model(root).unwrap();
    let r = validate_with_config(&elems, &ValidateConfig::with_model_root(root));
    r.errors().chain(r.warnings()).map(|f| format!("{} {}", f.code, f.message)).collect()
}

#[test]
fn a_first_order_lag_reaches_its_analytic_value() {
    let exact = 1.0 - (-5.0f64).exp();
    let rk4 = y_after(&world("plant: Lag\n", Some("10 ms"), &[("Lag", lag(""))]), 1.0, 1.0);
    assert!((rk4 - exact).abs() < 1e-7, "rk4 {rk4} vs {exact}");
    let euler = y_after(&world("plant: Lag\n", Some("10 ms"), &[("Lag", lag("method: euler\n"))]), 1.0, 1.0);
    assert!((euler - exact).abs() < 1e-2 && (euler - exact).abs() > 1e-4, "euler {euler} vs {exact}");
    // a shorter integration sub-step brings forward Euler closer, at the same communication step
    let fine = y_after(&world("plant: Lag\n", Some("10 ms"), &[("Lag", lag("method: euler\nmaxStep: \"1 ms\"\n"))]), 1.0, 1.0);
    assert!((fine - exact).abs() < (euler - exact).abs() / 5.0, "maxStep: {fine} vs {euler}");
}

#[test]
fn the_same_model_gives_the_same_bits_every_run() {
    let root = world("plant: Lag\n", Some("10 ms"), &[("Lag", lag(""))]);
    let a = y_after(&root, 0.7, 0.83);
    let b = y_after(&root, 0.7, 0.83);
    assert_eq!(a.to_bits(), b.to_bits());
    let (m1, m2) = (build(&root).unwrap(), build(&root).unwrap());
    assert_eq!(m1.hash, m2.hash);
    // a changed equation is a changed plant: the evidence hash moves
    let other = world("plant: Lag\n", Some("10 ms"), &[("Lag", lag("").replace("(k * u - x)", "(k * u - x * 1)"))]);
    assert_ne!(build(&other).unwrap().hash, m1.hash);
}

#[test]
fn parameters_are_overridden_per_part() {
    let exact = 2.0 * (1.0 - (-5.0f64).exp());
    let y = y_after(&world("plant:\n  def: Lag\n  parameters: {k: 2}\n", Some("10 ms"), &[("Lag", lag(""))]), 1.0, 1.0);
    assert!((y - exact).abs() < 1e-6, "{y}");
}

#[test]
fn the_library_plants_behave_as_documented() {
    let lib = |decl: &str, u: f64, secs: f64| y_after(&world(decl, Some("10 ms"), &[]), u, secs);
    assert!((lib("plant: {library: Gain, parameters: {k: 3}}\n", 2.0, 0.1) - 6.0).abs() < 1e-12);
    let exact = 1.0 - (-5.0f64).exp();
    assert!((lib("plant: {library: FirstOrderLag, parameters: {tau: \"200 ms\"}}\n", 1.0, 1.0) - exact).abs() < 1e-7);
    // the integrator's unit is its output's; the input is its rate (1/s here)
    let i = y_after(&world("plant: {library: Integrator}\n", Some("10 ms"), &[("CtlM", CTL.replace("{name: u, value: \"1\"}", "{name: u, value: \"1 / 1 s\"}"))]), 1.0, 0.5);
    assert!((i - 0.5).abs() < 1e-9, "{i}");
    assert_eq!(lib("plant: {library: Saturation, parameters: {lo: -1, hi: 0.5}}\n", 2.0, 0.1), 0.5);
    // a 30 ms delay: the initial value until the first sample is old enough, then the input
    let d = world("plant: {library: DelayLine, parameters: {delay: \"30 ms\", initial: 9}}\n", Some("10 ms"), &[]);
    assert_eq!(y_after(&d, 1.0, 0.02), 9.0);
    assert_eq!(y_after(&d, 1.0, 0.05), 1.0);
}

#[test]
fn an_impedance_divider_is_a_grid_seen_through_its_impedance() {
    let ctl = "---\ntype: StateDef\nname: CtlM\nvariables:\n  - {name: u, value: \"2 SI::A\"}\n  - {name: y, value: \"0 SI::V\"}\nsubStates:\n  - {name: Run, isInitial: true}\n---\n";
    let root = world("plant: {library: ImpedanceDivider, parameters: {vSrc: \"230 SI::V\", z: \"0.5 SI::ohm\"}}\n", Some("10 ms"), &[("CtlM", ctl.to_string())]);
    let root_txt = std::fs::read_to_string(root.join("Sys.md")).unwrap().replace("ctl.u, to: p.u", "ctl.u, to: p.i").replace("p.y, to: ctl.y", "p.v, to: ctl.y");
    std::fs::write(root.join("Sys.md"), root_txt).unwrap();
    let m = build(&root).unwrap_or_else(|e| panic!("{e:?}"));
    let mut s = System::new(&m, Options::default());
    s.advance(0.05);
    assert!((num(s.var("ctl", "y").unwrap()) - 229.0).abs() < 1e-9);
}

#[test]
fn a_plant_part_needs_the_communication_step_and_dimensioned_bindings() {
    let e = build(&world("plant: Lag\n", None, &[("Lag", lag(""))])).err().unwrap();
    assert!(e.iter().any(|x| x.contains("E934") && x.contains("plantStep")), "{e:?}");
    let ctl = CTL.replace("{name: y, value: \"0\"}", "{name: y, value: \"0 SI::V\"}");
    let e = build(&world("plant: Lag\n", Some("10 ms"), &[("Lag", lag("")), ("CtlM", ctl)])).err().unwrap();
    assert!(e.iter().any(|x| x.contains("dimension")), "{e:?}");
    let root = world("plant: Lag\n", Some("10 ms"), &[("Lag", lag(""))]);
    let t = std::fs::read_to_string(root.join("Sys.md")).unwrap().replace("p.y, to: ctl.y", "p.ghost, to: ctl.y");
    std::fs::write(root.join("Sys.md"), t).unwrap();
    assert!(build(&root).err().unwrap().iter().any(|x| x.contains("ghost")));
    // no FMI backend is involved: the model validates clean
    let clean = codes(&world("plant: Lag\n", Some("10 ms"), &[("Lag", lag(""))]));
    assert!(clean.iter().all(|c| !c.starts_with('E')), "{clean:?}");
}

#[test]
fn a_malformed_plant_def_is_reported_where_it_is_written() {
    let cases: &[(&str, &str, &str)] = &[
        ("\"der(x) = (k * u - x) / tau\"", "\"x = u\"", "E938"),
        ("\"y := x\"", "\"y := x + ghost\"", "E938"),
        ("\"y := x\"", "\"z := x\"", "E938"),
        ("\"der(x) = (k * u - x) / tau\"", "\"der(x) = k * u - x\"", "E935"),
        ("\"y := x\"", "\"y := x * tau\"", "E935"),
    ];
    for (from, to, code) in cases {
        let root = world("plant: Lag\n", Some("10 ms"), &[("Lag", lag("").replace(from, to))]);
        let c = codes(&root);
        assert!(c.iter().any(|x| x.starts_with(code)), "{to}: {c:?}");
        // the part that uses it gets one E939 pointing at the definition, not a copy of every finding
        assert!(c.iter().any(|x| x.starts_with("E939") && x.contains("has errors")), "{to}: {c:?}");
    }
    let looped = lag("").replace("outputs:\n  - {name: y}", "outputs:\n  - {name: y}\n  - {name: w}").replace("\"y := x\"", "\"y := w\"\n  - \"w := y\"");
    let c = codes(&world("plant: Lag\n", Some("10 ms"), &[("Lag", looped)]));
    assert!(c.iter().any(|x| x.starts_with("E938") && x.contains("algebraic loop")), "{c:?}");
    let c = codes(&world("plant: Lag\n", Some("10 ms"), &[("Lag", lag("method: heun\n"))]));
    assert!(c.iter().any(|x| x.starts_with("E938") && x.contains("heun")), "{c:?}");
}

#[test]
fn a_plant_declaration_that_does_not_fit_is_e939() {
    let cases: &[(&str, &str)] = &[
        ("plant: Ghost\n", "does not resolve"),
        ("plant: CtlP\n", "not a PlantDef"),
        ("plant: {library: Nope}\n", "not a library plant"),
        ("plant: {def: Lag, parameters: {zeta: 1}}\n", "zeta"),
        ("plant: {def: Lag, parameters: {tau: 3}}\n", "dimension"),
        ("plant: {def: Lag, library: Gain}\n", "exactly one"),
    ];
    for (decl, what) in cases {
        let c = codes(&world(decl, Some("10 ms"), &[("Lag", lag(""))]));
        assert!(c.iter().any(|x| x.starts_with("E939") && x.contains(what)), "{decl}: {c:?}");
    }
    let c = codes(&world("plant: Lag\nfmu:\n  file: x.fmu\n  fmiVersion: \"2.0\"\n  kind: cosimulation\n", Some("10 ms"), &[("Lag", lag(""))]));
    assert!(c.iter().any(|x| x.starts_with("E930")), "{c:?}");
    // the plant keys belong on a PlantDef
    let c = codes(&world("plant: Lag\nequations: [\"y := 1\"]\n", Some("10 ms"), &[("Lag", lag(""))]));
    assert!(c.iter().any(|x| x.starts_with("E938") && x.contains("equations")), "{c:?}");
}

#[test]
fn the_new_functions_take_a_dimensionless_argument_and_are_libm_exact() {
    use syscribe_model::expr::parse;
    let ev = |s: &str| parse(s).unwrap().eval(&|_: &str| None);
    let v = |s: &str| match ev(s) {
        Ok(Value::Num(q)) => q.v,
        other => panic!("{s}: {other:?}"),
    };
    assert_eq!((v("exp(0)"), v("ln(1)"), v("sin(0)"), v("cos(0)"), v("tanh(0)")), (1.0, 0.0, 0.0, 1.0, 0.0));
    assert_eq!(v("exp(1)").to_bits(), libm::exp(1.0).to_bits());
    assert_eq!(v("sin(0.5)").to_bits(), libm::sin(0.5).to_bits());
    assert!(ev("exp(1 s)").is_err() && ev("ln(0)").is_err() && ev("ln(-1)").is_err());
    // a state machine that uses one with a dimensioned argument is refused before it runs
    let ctl = "---\ntype: StateDef\nname: CtlM\nvariables:\n  - {name: u, value: \"1\"}\n  - {name: y, value: \"0\"}\n  - {name: t, value: \"1 s\"}\nsubStates:\n  - name: Run\n    isInitial: true\n    transitions:\n      - {target: Done, accept: {when: \"exp(t) > 1\"}}\n  - {name: Done, isFinal: true}\n---\n";
    let c = codes(&world("plant: Lag\n", Some("10 ms"), &[("Lag", lag("")), ("CtlM", ctl.to_string())]));
    assert!(c.iter().any(|x| x.starts_with("E935") && x.contains("exp")), "{c:?}");
}

#[test]
fn the_lag_response_has_the_same_bits_on_every_platform() {
    // only IEEE add/multiply/divide and repeated squaring for integer powers: no platform-dependent libm call
    let y = y_after(&world("plant: Lag\n", Some("10 ms"), &[("Lag", lag(""))]), 0.7, 0.83);
    assert_eq!(y.to_bits(), GOLDEN_LAG_BITS, "{y} = {:#x}", y.to_bits());
}

const GOLDEN_LAG_BITS: u64 = 0x3fe6_0c00_2167_fbc0;

#[test]
fn an_output_whose_name_starts_with_der_is_an_output() {
    let derate = lag("").replace("outputs:\n  - {name: y}", "outputs:\n  - {name: y}\n  - {name: derate}").replace("\"y := x\"", "\"y := x\"\n  - \"derate := x\"");
    let c = codes(&world("plant: Lag\n", Some("10 ms"), &[("Lag", derate)]));
    assert!(c.iter().all(|x| !x.starts_with('E')), "{c:?}");
}

#[test]
fn a_scaled_unit_is_refused_rather_than_read_as_its_base_unit() {
    let mv = lag("").replace("outputs:\n  - {name: y}", "outputs:\n  - {name: y, unit: mV}");
    let c = codes(&world("plant: Lag\n", Some("10 ms"), &[("Lag", mv)]));
    assert!(c.iter().any(|x| x.starts_with("E938") && x.contains("mV")), "{c:?}");
}

#[test]
fn a_binding_between_two_plants_is_refused_by_validate() {
    let root = world("plant: {library: FirstOrderLag}\n", Some("10 ms"), &[("GainP", "---\ntype: PartDef\nname: GainP\nplant: {library: Gain}\n---\nA gain.\n".to_string())]);
    let sys = "---\ntype: PartDef\nname: Sys\nplantStep: \"10 ms\"\nfeatures:\n  - {name: ctl, typedBy: CtlP}\n  - {name: p, typedBy: PlantP}\n  - {name: g, typedBy: GainP}\nbindingConnections:\n  - {from: ctl.u, to: p.u}\n  - {from: p.y, to: g.u}\n  - {from: g.y, to: ctl.y}\n---\nThe system.\n";
    std::fs::write(root.join("Sys.md"), sys).unwrap();
    let c = codes(&root);
    assert!(c.iter().any(|x| x.starts_with("E933") && x.contains("both ends are plants")), "{c:?}");
}

#[test]
fn a_part_that_runs_a_plant_def_references_it() {
    let root = world("plant: Lag\n", Some("10 ms"), &[("Lag", lag(""))]);
    let elems = walk_model(&root).unwrap();
    let part = elems.iter().find(|e| e.qualified_name == "PlantP").unwrap();
    assert!(syscribe_model::mutate::guard::element_ref_strings(part).contains(&("plant", "Lag".to_string())));
    let by: Vec<String> = syscribe_model::mutate::guard::referrers(&elems, "Lag").into_iter().map(|(q, _)| q).collect();
    assert_eq!(by, vec!["PlantP".to_string()]);
    let map = std::fs::read_to_string(root.join("PlantP.md")).unwrap().replace("plant: Lag", "plant: {def: Lag, parameters: {k: 2}}");
    std::fs::write(root.join("PlantP.md"), map).unwrap();
    let elems = walk_model(&root).unwrap();
    assert_eq!(syscribe_model::mutate::guard::referrers(&elems, "Lag").len(), 1);
}

#[test]
fn a_negative_delay_and_too_many_sub_steps_are_refused() {
    let fmea = "---\ntype: FMEASheet\nid: FMEA-D-001\nname: delay\nstatus: draft\nentries:\n  - {id: FM-D-001, ref: PlantP, failureMode: \"negative delay\", injection: [{kind: parameter, parameter: delay, value: \"-1 s\"}]}\n---\n";
    let root = world("plant: {library: DelayLine, parameters: {delay: \"30 ms\"}}\n", Some("10 ms"), &[("FMEA-D-001", fmea.to_string())]);
    let m = build(&root).unwrap_or_else(|e| panic!("{e:?}"));
    let mut s = System::new(&m, Options::default());
    let e = s.inject("FM-D-001").unwrap_err();
    assert!(e.contains("non-negative"), "{e}");
    // the refused fault left nothing behind: the line still delays by 30 ms
    s.advance(0.05);
    assert!(s.error().is_none(), "{:?}", s.error());
    let tiny = build(&world("plant: Lag\n", Some("10 ms"), &[("Lag", lag("maxStep: \"1 ns\"\n"))])).unwrap();
    let mut s = System::new(&tiny, Options::default());
    s.advance(0.02);
    assert!(s.error().is_some_and(|e| e.contains("sub-steps")), "{:?}", s.error());
}

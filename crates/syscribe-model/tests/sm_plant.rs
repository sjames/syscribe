//! TC-TRS-FMI-001 / REQ-TRS-FMI-001 (GH #291): the co-simulation loop of a state-machine run with plants,
//! tested with an analytic mock plant so no FMI library is needed.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::expr::{format_value, Value};
use syscribe_model::ir::{lower, Machine};
use syscribe_model::plant::{Causality, CoSim, FmuInfo, Plant, VarInfo, VarType};
use syscribe_model::sm_run::{Options, Run};
use syscribe_model::units::Dim;
use syscribe_model::walker::walk_model;

type Calls = Rc<RefCell<Vec<(f64, f64)>>>;
type Inputs = Rc<RefCell<Vec<f64>>>;

/// A ball in free fall with an extra dimensionless input `thrust` (upward acceleration in m/s²), in feet to exercise unit
/// conversion: `h` is reported in feet.
struct Ball {
    info: FmuInfo,
    h: f64,
    v: f64,
    thrust: f64,
    fail_after: Option<f64>,
    calls: Calls,
    thrust_seen: Inputs,
}

const FT: f64 = 0.3048;

fn var(name: &str, causality: Causality, ty: VarType, dim: Option<Dim>, factor: f64) -> VarInfo {
    VarInfo { name: name.into(), causality, ty, dim, factor, offset: 0.0 }
}

fn length() -> Dim {
    Dim([1, 0, 0, 0, 0, 0, 0])
}

impl Ball {
    fn new(calls: Calls, thrust_seen: Inputs) -> Ball {
        Ball {
            info: FmuInfo {
                fmi_version: "2.0".into(),
                guid: "{mock}".into(),
                sha256: "00".into(),
                supports_cosimulation: true,
                variables: vec![
                    var("h", Causality::Output, VarType::Real, Some(length()), FT),
                    var("v", Causality::Output, VarType::Real, Some(Dim([1, 0, -1, 0, 0, 0, 0])), 1.0),
                    var("thrust", Causality::Input, VarType::Real, None, 1.0),
                    var("hit", Causality::Output, VarType::Boolean, None, 1.0),
                ],
            },
            h: 1.0 / FT, // 1 m, in feet
            v: 0.0,
            thrust: 0.0,
            fail_after: None,
            calls,
            thrust_seen,
        }
    }
}

impl Plant for Ball {
    fn info(&self) -> &FmuInfo {
        &self.info
    }
    fn set(&mut self, name: &str, v: f64) -> Result<(), String> {
        match name {
            "thrust" => {
                self.thrust = v;
                Ok(())
            }
            other => Err(format!("no input {other}")),
        }
    }
    fn get(&mut self, name: &str) -> Result<f64, String> {
        match name {
            "h" => Ok(self.h),
            "v" => Ok(self.v),
            "hit" => Ok(if self.h * FT < 0.0 { 1.0 } else { 0.0 }),
            other => Err(format!("no output {other}")),
        }
    }
    fn step(&mut self, t: f64, h: f64) -> Result<(), String> {
        self.calls.borrow_mut().push((t, h));
        self.thrust_seen.borrow_mut().push(self.thrust);
        if self.fail_after.is_some_and(|f| t >= f) {
            return Err("solver diverged".into());
        }
        let a = -9.81 + self.thrust;
        let h_m = self.h * FT + self.v * h + 0.5 * a * h * h;
        self.v += a * h;
        self.h = h_m / FT;
        Ok(())
    }
    fn provenance(&self) -> serde_json::Value {
        serde_json::json!({"kind": "mock", "guid": self.info.guid})
    }
}

fn machine(yaml: &str) -> Machine {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir: PathBuf = std::env::temp_dir().join(format!("syscribe-plant-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(dir.join("M.md"), format!("---\ntype: StateDef\nname: M\n{yaml}---\n")).unwrap();
    lower(&walk_model(&dir).unwrap(), "M").unwrap_or_else(|d| panic!("{d:?}"))
}

const FALL: &str = "variables:
  - {name: height, value: \"0\", unit: \"SI::m\"}
  - {name: thrust, value: \"0\"}
  - {name: touched, value: \"false\"}
subStates:
  - name: Falling
    isInitial: true
    transitions:
      - {target: Bounced, accept: {when: \"height < 0.1 m\"}}
  - name: Bounced
";

fn cosim(m: &Machine, ball: Ball, step: f64, conns: &[(&str, &str)]) -> Result<CoSim, Vec<String>> {
    CoSim::new(vec![("ball".to_string(), Box::new(ball) as Box<dyn Plant>)], conns.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect(), step, m)
}

fn fall_cosim(m: &Machine, calls: Calls, inputs: Inputs, step: f64) -> CoSim {
    cosim(m, Ball::new(calls, inputs), step, &[("ball.h", "height"), ("thrust", "ball.thrust")]).unwrap()
}

#[test]
fn initial_outputs_are_read_before_the_first_settle_and_converted_to_si() {
    let m = machine(FALL);
    let (c, i) = (Calls::default(), Inputs::default());
    let r = Run::with_plants(&m, Options::default(), fall_cosim(&m, c, i, 0.1));
    assert!(r.error().is_none(), "{:?}", r.error());
    let h = r.var("height").unwrap();
    let Value::Num(q) = h else { panic!() };
    assert!((q.v - 1.0).abs() < 1e-9, "1 m, read from a plant that reports feet: {}", format_value(&r.var("height").unwrap()));
    assert_eq!(r.active(), ["Falling"]);
}

#[test]
fn a_when_over_a_plant_output_fires_at_the_first_step_boundary_after_the_crossing() {
    let m = machine(FALL);
    let (c, i) = (Calls::default(), Inputs::default());
    let mut r = Run::with_plants(&m, Options::default(), fall_cosim(&m, c.clone(), i, 0.1));
    r.advance(1.0);
    assert_eq!(r.active(), ["Bounced"]);
    // free fall from 1 m crosses 0.1 m at t = sqrt(2 * 0.9 / 9.81) = 0.4283 s: the first boundary after is 0.5 s
    let step = r.trace().iter().find(|s| s.cause.starts_with("change")).expect("a change step");
    assert!((step.time - 0.5).abs() < 1e-9, "fired at {}", step.time);
}

#[test]
fn a_discrete_timer_cuts_the_communication_step_at_its_exact_due_time() {
    let m = machine(
        "variables:
  - {name: height, value: \"0\", unit: \"SI::m\"}
  - {name: thrust, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    transitions: [{target: B, accept: {after: \"0.25 s\"}}]
  - name: B
",
    );
    let (c, i) = (Calls::default(), Inputs::default());
    let mut r = Run::with_plants(&m, Options::default(), fall_cosim(&m, c.clone(), i, 0.1));
    r.advance(0.3);
    assert_eq!(r.active(), ["B"]);
    let steps: Vec<(f64, f64)> = c.borrow().clone();
    let expect = [(0.0, 0.1), (0.1, 0.1), (0.2, 0.05), (0.25, 0.05)];
    assert_eq!(steps.len(), expect.len(), "{steps:?}");
    for ((t, h), (et, eh)) in steps.iter().zip(expect) {
        assert!((t - et).abs() < 1e-9 && (h - eh).abs() < 1e-9, "{steps:?}");
    }
    let timer = r.trace().iter().find(|s| s.cause.starts_with("timer")).unwrap();
    assert!((timer.time - 0.25).abs() < 1e-9);
}

#[test]
fn machine_variables_reach_plant_inputs_held_constant_over_each_step() {
    let m = machine(FALL);
    let (c, i) = (Calls::default(), Inputs::default());
    let mut r = Run::with_plants(&m, Options::default(), fall_cosim(&m, c, i.clone(), 0.1));
    r.advance(0.1);
    r.set("thrust", Value::Num(syscribe_model::expr::Quantity::scalar(9.81)));
    r.advance(0.2);
    let seen = i.borrow().clone();
    assert_eq!(seen.len(), 3, "{seen:?}");
    assert!(seen[0].abs() < 1e-12, "the first step used the initial input: {seen:?}");
    assert!((seen[1] - 9.81).abs() < 1e-12 && (seen[2] - 9.81).abs() < 1e-12, "the change took effect at the next boundary: {seen:?}");
    // thrust cancels gravity after the first step: v stays at -0.981 m/s
    let Value::Num(h) = r.var("height").unwrap() else { panic!() };
    assert!((h.v - 0.7548).abs() < 1e-3, "0.951 m after the first step, then constant speed: {}", h.v);
}

#[test]
fn a_boolean_output_maps_to_a_boolean_variable() {
    let m = machine(&FALL.replace("value: \"false\"", "value: \"false\""));
    let (c, i) = (Calls::default(), Inputs::default());
    let co = cosim(&m, Ball::new(c, i), 0.1, &[("ball.hit", "touched")]).unwrap();
    let mut r = Run::with_plants(&m, Options::default(), co);
    assert_eq!(r.var("touched"), Some(Value::Bool(false)));
    r.advance(0.6);
    assert_eq!(r.var("touched"), Some(Value::Bool(true)));
}

#[test]
fn connections_are_checked_before_anything_runs() {
    let m = machine(FALL);
    let mk = || Ball::new(Calls::default(), Inputs::default());
    let e = cosim(&m, mk(), 0.1, &[("ball.h", "thrust")]).err().expect("length into a dimensionless variable");
    assert!(e.iter().any(|x| x.contains("dimension")), "{e:?}");
    let e = cosim(&m, mk(), 0.1, &[("ball.nope", "height")]).err().unwrap();
    assert!(e.iter().any(|x| x.contains("nope")), "{e:?}");
    let e = cosim(&m, mk(), 0.1, &[("height", "ball.h")]).err().unwrap();
    assert!(e.iter().any(|x| x.contains("output")), "an output cannot be written: {e:?}");
    let e = cosim(&m, mk(), 0.1, &[("ball.thrust", "thrust")]).err().unwrap();
    assert!(e.iter().any(|x| x.contains("input")), "an input cannot be read: {e:?}");
    let e = cosim(&m, mk(), 0.1, &[("ball.h", "ghost")]).err().unwrap();
    assert!(e.iter().any(|x| x.contains("ghost")), "{e:?}");
    let e = cosim(&m, mk(), 0.1, &[("ball.hit", "height")]).err().unwrap();
    assert!(e.iter().any(|x| x.contains("type") || x.contains("boolean")), "{e:?}");
    let e = cosim(&m, mk(), 0.0, &[]).err().unwrap();
    assert!(e.iter().any(|x| x.contains("step")), "{e:?}");
}

#[test]
fn a_plant_failure_stops_the_run_and_names_the_plant() {
    let m = machine(FALL);
    let mut ball = Ball::new(Calls::default(), Inputs::default());
    ball.fail_after = Some(0.15);
    let co = cosim(&m, ball, 0.1, &[("ball.h", "height")]).unwrap();
    let mut r = Run::with_plants(&m, Options::default(), co);
    r.advance(1.0);
    assert!(r.error().is_some_and(|e| e.contains("ball") && e.contains("diverged")), "{:?}", r.error());
    assert_eq!(r.trace().last().unwrap().cause, "error");
}

#[test]
fn the_run_exposes_the_plants_provenance_and_the_values_exchanged() {
    let m = machine(FALL);
    let (c, i) = (Calls::default(), Inputs::default());
    let mut r = Run::with_plants(&m, Options::default(), fall_cosim(&m, c, i, 0.1));
    r.advance(0.2);
    let p = r.plant_provenance();
    assert_eq!(p.len(), 1);
    assert_eq!(p[0]["name"], "ball");
    assert_eq!(p[0]["step"], 0.1);
    assert_eq!(p[0]["plant"]["kind"], "mock");
    assert!(r.trace().iter().any(|s| s.cause == "plant" && s.changed.iter().any(|c| c.var == "height")), "exchanged values are traced");
}

#[test]
fn without_plants_advance_behaves_exactly_as_before() {
    let m = machine("variables:\n  - {name: v, value: \"0\"}\nsubStates:\n  - name: A\n    isInitial: true\n    transitions: [{target: B, accept: {after: \"2 s\"}}]\n  - name: B\n");
    let mut r = Run::new(&m, Options::default());
    r.advance(1.0);
    assert_eq!(r.active(), ["A"]);
    r.advance(1.5);
    assert_eq!(r.active(), ["B"]);
    assert!(r.plant_provenance().is_empty());
}

#[test]
fn an_absurd_number_of_communication_steps_is_refused_instead_of_hanging() {
    let m = machine(FALL);
    let (c, i) = (Calls::default(), Inputs::default());
    let mut r = Run::with_plants(&m, Options::default(), fall_cosim(&m, c.clone(), i, 1e-9));
    r.advance(1e6);
    assert!(r.error().is_some_and(|e| e.contains("communication steps")), "{:?}", r.error());
    assert!(c.borrow().len() < 10, "no steps were taken");
}

#[test]
fn two_plants_with_the_same_name_are_refused() {
    let m = machine(FALL);
    let mk = || Box::new(Ball::new(Calls::default(), Inputs::default())) as Box<dyn Plant>;
    let e = CoSim::new(vec![("ball".to_string(), mk()), ("ball".to_string(), mk())], vec![("ball.h".into(), "height".into())], 0.1, &m).err().expect("refused");
    assert!(e.iter().any(|x| x.contains("distinct")), "{e:?}");
}

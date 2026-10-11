//! TC-TRS-SYSCOMP-001 / REQ-TRS-SYSCOMP-001 (GH #299): FMU plants as parts of a composite, tested with an analytic mock
//! plant so no FMI library is needed. (The backend is process-global; the "no backend" case is in `system_plant_gating.rs`.)

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, Once};

use syscribe_model::expr::{format_value, parse_value, Value};
use syscribe_model::plant::{register_backend, Causality, FmuInfo, Plant, PlantBackend, VarInfo, VarType};
use syscribe_model::sm_run::Options;
use syscribe_model::system_run::{System, SystemModel};
use syscribe_model::units::Dim;
use syscribe_model::walker::walk_model;

static CALLS: Mutex<Vec<(String, f64, f64)>> = Mutex::new(Vec::new());
static WRITES: Mutex<Vec<(String, f64)>> = Mutex::new(Vec::new());

fn var(name: &str, c: Causality, ty: VarType, dim: Option<Dim>) -> VarInfo {
    VarInfo { name: name.into(), causality: c, ty, dim, factor: 1.0, offset: 0.0 }
}

fn info(file: &str) -> FmuInfo {
    FmuInfo {
        fmi_version: "2.0".into(),
        guid: "{mock}".into(),
        sha256: format!("sha-{file}"),
        supports_cosimulation: true,
        variables: vec![
            var("h", Causality::Output, VarType::Real, Some(Dim([1, 0, 0, 0, 0, 0, 0]))),
            var("v", Causality::Output, VarType::Real, Some(Dim([1, 0, -1, 0, 0, 0, 0]))),
            var("thrust", Causality::Input, VarType::Real, None),
            var("e", Causality::Parameter, VarType::Real, None),
        ],
    }
}

/// A ball in free fall from 1 m; `thrust` is an upward acceleration added to gravity. A file named `*-broken.fmu` fails its first step.
struct Ball {
    file: String,
    info: FmuInfo,
    h: f64,
    v: f64,
    thrust: f64,
}

impl Plant for Ball {
    fn info(&self) -> &FmuInfo {
        &self.info
    }
    fn set(&mut self, name: &str, v: f64) -> Result<(), String> {
        WRITES.lock().unwrap().push((self.file.clone(), v));
        match name {
            "thrust" => {
                self.thrust = v;
                Ok(())
            }
            o => Err(format!("cannot set {o}")),
        }
    }
    fn get(&mut self, name: &str) -> Result<f64, String> {
        match name {
            "h" => Ok(self.h),
            "v" => Ok(self.v),
            o => Err(format!("cannot get {o}")),
        }
    }
    fn step(&mut self, t: f64, h: f64) -> Result<(), String> {
        CALLS.lock().unwrap().push((self.file.clone(), t, h));
        if self.file.ends_with("-broken.fmu") {
            return Err("solver diverged".into());
        }
        let a = -9.81 + self.thrust;
        self.h += self.v * h + 0.5 * a * h * h;
        self.v += a * h;
        Ok(())
    }
    fn provenance(&self) -> serde_json::Value {
        serde_json::json!({"kind": "mock", "file": self.file})
    }
}

struct Mock;

impl PlantBackend for Mock {
    fn name(&self) -> String {
        "mock".into()
    }
    fn inspect(&self, file: &Path) -> Result<FmuInfo, String> {
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        if !file.exists() {
            return Err(format!("{name}: no such file"));
        }
        Ok(info(&name))
    }
    fn instantiate(&self, file: &Path, _parameters: &[(String, f64)]) -> Result<Box<dyn Plant>, String> {
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        Ok(Box::new(Ball { info: info(&name), file: name, h: 1.0, v: 0.0, thrust: 0.0 }))
    }
}

fn setup() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        register_backend(Box::new(Mock));
    });
}

const CTL: &str = "variables:
  - {name: height, value: \"0\", unit: \"SI::m\"}
  - {name: thrust, value: \"0\"}
  - {name: count, value: \"0\"}
subStates:
  - name: Falling
    isInitial: true
    transitions:
      - {target: Bounced, accept: {when: \"height < 0.1 m\"}}
  - name: Bounced
";

struct World {
    root: PathBuf,
    fmu: String,
}

fn world(extra_ctl: Option<&str>, bindings: &str, plant_step: Option<&str>, fmu_file: &str) -> World {
    setup();
    static N: AtomicU64 = AtomicU64::new(0);
    let id = N.fetch_add(1, Ordering::SeqCst);
    let root: PathBuf = std::env::temp_dir().join(format!("syscribe-sysplant-{}-{id}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let fmu = fmu_file.replace("{id}", &format!("{}-{id}", std::process::id()));
    std::fs::write(root.join(&fmu), "x").unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("CtlM.md"), format!("---\ntype: StateDef\nname: CtlM\n{}---\n", extra_ctl.unwrap_or(CTL))).unwrap();
    std::fs::write(root.join("CtlP.md"), "---\ntype: PartDef\nname: CtlP\nexhibitsStates: [CtlM]\n---\nController.\n").unwrap();
    std::fs::write(root.join("BallP.md"), format!("---\ntype: PartDef\nname: BallP\nfmu:\n  file: {fmu}\n  fmiVersion: \"2.0\"\n  kind: cosimulation\n  parameters: {{e: 0.8}}\n---\nThe ball.\n")).unwrap();
    let step = plant_step.map(|s| format!("plantStep: \"{s}\"\n")).unwrap_or_default();
    std::fs::write(root.join("Sys.md"), format!("---\ntype: PartDef\nname: Sys\n{step}features:\n  - {{name: ctl, typedBy: CtlP}}\n  - {{name: ball, typedBy: BallP}}\nbindingConnections:\n{bindings}---\nThe system.\n")).unwrap();
    World { root, fmu }
}

fn build(w: &World) -> Result<SystemModel, Vec<String>> {
    SystemModel::build_in(&walk_model(&w.root).unwrap(), "Sys", &w.root)
}

const READ: &str = "  - {from: ball.h, to: ctl.height}\n";
const WRITE: &str = "  - {from: ctl.thrust, to: ball.thrust}\n";

fn calls_of(fmu: &str) -> Vec<(f64, f64)> {
    CALLS.lock().unwrap().iter().filter(|c| c.0 == fmu).map(|c| (c.1, c.2)).collect()
}

#[test]
fn a_plant_part_drives_a_machine_variable_and_a_when_trigger_fires_at_the_step_boundary() {
    let w = world(None, &format!("{READ}{WRITE}"), Some("100 ms"), "ball-{id}.fmu");
    let m = build(&w).unwrap();
    let mut s = System::new(&m, Options::default());
    assert!(s.error().is_none(), "{:?}", s.error());
    assert_eq!(format_value(&s.var("ctl", "height").unwrap()), format_value(&parse_value("1 m").unwrap()), "initial outputs are read first");
    s.advance(1.0);
    assert_eq!(s.active("ctl").unwrap(), ["Bounced"]);
    // free fall from 1 m crosses 0.1 m at 0.4283 s: the first 100 ms boundary after is 0.5 s
    let at = s.trace().into_iter().find(|(p, st)| p == "ctl" && st.cause.starts_with("change")).map(|(_, st)| st.time).unwrap();
    assert!((at - 0.5).abs() < 1e-9, "fired at {at}");
}

#[test]
fn a_timer_cuts_the_plant_step_and_inputs_are_held_over_it() {
    let ctl = "variables:
  - {name: height, value: \"0\", unit: \"SI::m\"}
  - {name: thrust, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: B, accept: {after: \"250 ms\"}, assign: \"thrust := 9.81\"}
  - name: B
";
    let w = world(Some(ctl), &format!("{READ}{WRITE}"), Some("100 ms"), "ball-{id}.fmu");
    let m = build(&w).unwrap();
    let mut s = System::new(&m, Options::default());
    s.advance(0.4);
    let c = calls_of(&w.fmu);
    let starts: Vec<f64> = c.iter().map(|x| x.0).collect();
    assert!(starts.iter().any(|t| (t - 0.2).abs() < 1e-9) && starts.iter().any(|t| (t - 0.25).abs() < 1e-9), "the 250 ms timer cuts a step: {c:?}");
    let (t, h) = c.iter().find(|x| (x.0 - 0.2).abs() < 1e-9).unwrap();
    assert!((h - 0.05).abs() < 1e-9 && (*t + *h - 0.25).abs() < 1e-9, "{c:?}");
    // from 0.25 s the thrust cancels gravity: the acceleration is zero, so the ball keeps its velocity
    let h = |s: &System| match s.var("ctl", "height").unwrap() {
        Value::Num(q) => q.v,
        _ => panic!(),
    };
    let (h0, _) = (h(&s), s.advance(0.1));
    let h1 = h(&s);
    s.advance(0.1);
    let h2 = h(&s);
    assert!(((h1 - h0) - (h2 - h1)).abs() < 1e-9, "constant velocity after the timer: {h0} {h1} {h2}");
    assert!(h1 < h0, "still falling");
}

#[test]
fn every_system_run_starts_with_fresh_plants() {
    let w = world(None, READ, Some("100 ms"), "ball-{id}.fmu");
    let m = build(&w).unwrap();
    let mut a = System::new(&m, Options::default());
    a.advance(0.3);
    let b = System::new(&m, Options::default());
    assert_eq!(format_value(&b.var("ctl", "height").unwrap()), format_value(&parse_value("1 m").unwrap()));
    assert_eq!(b.plant_provenance().len(), 1);
    assert_eq!(b.plant_provenance()[0]["name"], "ball");
}

#[test]
fn plant_bindings_are_checked_before_anything_runs() {
    let cases = [
        ("  - {from: ball.h, to: ctl.thrust}\n", "dimension"),
        ("  - {from: ball.zzz, to: ctl.height}\n", "zzz"),
        ("  - {from: ctl.thrust, to: ball.h}\n", "output"),
        ("  - {from: ball.h, to: ctl.ghost}\n", "ghost"),
    ];
    for (b, what) in cases {
        let w = world(None, b, Some("100 ms"), "ball-{id}.fmu");
        let e = build(&w).err().unwrap_or_else(|| panic!("{b} should be refused"));
        assert!(e.iter().any(|x| x.contains(what)), "{b}: {e:?}");
    }
    let w = world(None, READ, None, "ball-{id}.fmu");
    assert!(build(&w).err().unwrap().iter().any(|x| x.contains("plantStep")), "a plant needs the step");
}

#[test]
fn a_plant_failure_stops_the_run_and_names_the_plant() {
    let w = world(None, READ, Some("100 ms"), "ball-{id}-broken.fmu");
    let m = build(&w).unwrap();
    let mut s = System::new(&m, Options::default());
    s.advance(0.5);
    assert!(s.error().is_some_and(|e| e.contains("ball") && e.contains("diverged")), "{:?}", s.error());
}

#[test]
fn a_changed_fmu_is_detected_when_the_declared_hash_no_longer_matches() {
    let w = world(None, READ, Some("100 ms"), "ball-{id}.fmu");
    let text = std::fs::read_to_string(w.root.join("BallP.md")).unwrap().replace("kind: cosimulation", "kind: cosimulation\n  sha256: not-the-hash");
    std::fs::write(w.root.join("BallP.md"), text).unwrap();
    assert!(build(&w).err().unwrap().iter().any(|x| x.contains("sha256")));
}

#[test]
fn a_step_budget_refuses_absurd_runs() {
    let w = world(None, READ, Some("1 ns"), "ball-{id}.fmu");
    let m = build(&w).unwrap();
    let mut s = System::new(&m, Options::default());
    s.advance(1e6);
    assert!(s.error().is_some_and(|e| e.contains("communication steps")), "{:?}", s.error());
}


#[test]
fn a_timer_due_at_a_communication_point_sees_the_plant_outputs_of_that_point() {
    let ctl = "variables:
  - {name: height, value: \"0\", unit: \"SI::m\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: Landed, accept: {after: \"300 ms\"}, guard: \"height < 0.7 m\"}
  - name: Landed
";
    let w = world(Some(ctl), READ, Some("100 ms"), "ball-{id}.fmu");
    let m = build(&w).unwrap();
    let mut s = System::new(&m, Options::default());
    s.advance(0.3);
    // at 0.2 s the ball is at 0.804 m (not below 0.7); at 0.3 s it is at 0.559 m: the guard must see the 0.3 s value
    assert_eq!(s.active("ctl").unwrap(), ["Landed"]);
}

#[test]
fn all_outputs_of_a_communication_point_reach_the_machine_together() {
    let ctl = "variables:
  - {name: height, value: \"0\", unit: \"SI::m\"}
  - {name: velocity, value: \"0 m / 1 s\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: Wrong, accept: {when: \"height < 0.7 m and velocity > -2.5 m / 1 s\"}}
  - name: Wrong
";
    let w = world(Some(ctl), &format!("{READ}  - {{from: ball.v, to: ctl.velocity}}\n"), Some("100 ms"), "ball-{id}.fmu");
    let m = build(&w).unwrap();
    let mut s = System::new(&m, Options::default());
    s.advance(0.5);
    // at 0.3 s the true snapshot is h = 0.559 m, v = -2.94 m/s (guard false); a new height with the previous velocity
    // (-1.96 m/s) would make it true
    assert_eq!(s.active("ctl").unwrap(), ["A"], "the guard never saw a half-updated snapshot");
}

#[test]
fn two_sources_for_one_variable_and_a_changed_plant_configuration() {
    let w = world(None, &format!("{READ}{READ}"), Some("100 ms"), "ball-{id}.fmu");
    assert!(build(&w).err().unwrap().iter().any(|x| x.contains("already has a source")));
    let a = build(&world(None, READ, Some("100 ms"), "ball-{id}.fmu")).unwrap();
    let b = build(&world(None, READ, Some("20 ms"), "ball-{id}.fmu")).unwrap();
    assert_ne!(a.hash, b.hash, "the communication step is part of the evidence hash");
}

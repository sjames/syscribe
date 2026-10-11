//! GH #311: a state machine can use a feature parameter in a guard, an assignment or a timer; the value comes from the
//! Configuration the run is projected onto (`simulate --config`).

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-mp-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&r).unwrap();
    let w = |n: &str, c: &str| std::fs::write(r.join(n), c).unwrap();
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features.md", "---\ntype: FeatureModel\nname: Features\nfeatureTree:\n  - name: Profile\n    id: FEAT-MP-PROFILE\n    mandatory: true\n    parameters:\n      - {name: delayS, type: ScalarValues::Integer, range: \"1..=600\", default: 3, isRequired: true, bindingTime: load}\n      - {name: limit, type: ScalarValues::Integer, range: \"1..=100\", default: 10, isRequired: true, bindingTime: load}\n---\n");
    for (cid, d, l) in [("CONF-MP-FAST-001", 3, 10), ("CONF-MP-SLOW-001", 5, 20)] {
        w(&format!("{cid}.md"), &format!("---\ntype: Configuration\nid: {cid}\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Profile: true\nparameterBindings:\n  Features::Profile.delayS: {d}\n  Features::Profile.limit: {l}\n---\n"));
    }
    w("M.md", "---\ntype: StateDef\nname: M\nvariables:\n  - {name: level, value: \"0\"}\nsubStates:\n  - name: Wait\n    isInitial: true\n    transitions:\n      - {target: Armed, accept: {after: \"{{Features::Profile.delayS}} s\"}, assign: \"level := {{Features::Profile.limit}}\"}\n  - name: Armed\n    transitions:\n      - {target: Over, accept: Check, guard: \"level > 15\"}\n  - {name: Over, isFinal: true}\n---\n");
    w("REQ-MP-001.md", "---\ntype: Requirement\nid: REQ-MP-001\nname: r\nstatus: draft\nreqDomain: software\n---\n\nThe machine shall.\n");
    w("TC-MP-001.md", "---\ntype: TestCase\nid: TC-MP-001\nname: t\nstatus: active\ntestLevel: L3\nsubject: M\nverifies: [REQ-MP-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: not yet\n    When 3 s elapse\n    Then M is in state Wait\n\n  Scenario: armed after the delay with the limit\n    When 5 s elapse\n    Then M is in state Armed\n    And level >= 10\n\n  Scenario: the guard uses the limit\n    When 6 s elapse\n    And Check is injected\n    Then M is in state Over\n```\n");
    r
}

fn sim(root: &PathBuf, conf: Option<&str>) -> (i32, String) {
    let mut a = vec!["simulate".to_string(), "TC-MP-001".into()];
    if let Some(c) = conf {
        a.push("--config".into());
        a.push(c.into());
    }
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(&a).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

#[test]
fn the_parameter_values_come_from_the_configuration() {
    let root = model();
    // fast: delay 3 s, limit 10 → the guard (level > 15) never holds; scenario 1 fails because the machine already left Wait
    let (c, t) = sim(&root, Some("CONF-MP-FAST-001"));
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("FAIL") && t.contains("not yet"), "3 s elapse is enough with delay 3: {t}");
    // slow: delay 5 s, limit 20 → all three scenarios hold
    let (c, t) = sim(&root, Some("CONF-MP-SLOW-001"));
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("3/3"), "{t}");
}

#[test]
fn without_a_configuration_the_placeholders_are_reported_not_misread() {
    let (c, t) = sim(&model(), None);
    assert_ne!(c, 0, "{t}");
    assert!(t.contains("{{") || t.contains("--config") || t.contains("placeholder"), "{t}");
}

#[test]
fn the_base_model_validates_with_placeholders_in_a_machine() {
    let root = model();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).arg("validate").output().unwrap();
    let t = String::from_utf8_lossy(&o.stdout).to_string();
    assert!(!t.contains("E882") && !t.contains("E884") && !t.contains("W933"), "the symbolic base model is not an expression error: {t}");
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).args(["validate", "--all-configs"]).output().unwrap();
    let detail = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).args(["validate", "--config", "CONF-MP-FAST-001"]).output().unwrap();
    assert!(String::from_utf8_lossy(&o.stdout).contains("2 pass"), "{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&detail.stdout));
}

#[test]
fn compile_and_mutate_take_a_configuration_too() {
    let root = model();
    let run = |args: &[&str]| {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).args(args).output().unwrap();
        (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
    };
    let (c, t) = run(&["compile", "M", "--config", "CONF-MP-SLOW-001", "--json"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("\"delaySeconds\": 5") || t.contains("5.0"), "the timer is 5 s under the slow configuration: {t}");
    let (c, _) = run(&["mutate", "M", "--config", "CONF-MP-SLOW-001", "--json"]);
    assert_eq!(c, 0);
    let (c, t) = run(&["compile", "M", "--config", "CONF-MP-FAST-001", "--json"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("\"delaySeconds\": 3") || t.contains("3.0"), "{t}");
}

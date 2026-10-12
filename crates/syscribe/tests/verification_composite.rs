//! GH #332: a verification composite holds the system under test (architecture elements that carry their own machine,
//! and executable parts that `models:` one) plus environment parts marked `role: environment`. An environment part is
//! not the product: it has no architecture counterpart (no W991), is no evidence for a requirement (W992), ignores a
//! `models:`/`portMap:` of its own (W993), and is listed apart in the simulate coverage and inject reports. An unknown
//! `role:` or one off a part usage is E992.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// A software `Manager` and `Relay` joined in `Board`; `Relay` carries its own machine. `Monitor` models `Manager`.
/// `Bench` holds a stimulus `stim` (environment), `mon` and the architecture `Relay` itself. Clean.
fn base() -> BTreeMap<&'static str, String> {
    let mut m = BTreeMap::new();
    let mut put = |k: &'static str, v: &str| {
        m.insert(k, v.to_string());
    };
    put("_index.md", "---\ntype: Package\nname: Root\n---\n");
    put("Reqs/REQ-VC-001.md", "---\ntype: Requirement\nid: REQ-VC-001\nname: trip\nstatus: draft\nreqDomain: software\n---\n\nThe manager shall trip.\n");
    put("Reqs/REQ-VC-002.md", "---\ntype: Requirement\nid: REQ-VC-002\nname: open\nstatus: draft\nreqDomain: software\n---\n\nThe relay driver shall open.\n");
    put("Reqs/REQ-VC-003.md", "---\ntype: Requirement\nid: REQ-VC-003\nname: sense\nstatus: draft\nreqDomain: software\n---\n\nThe sensor shall report.\n");
    put("Arch/Manager.md", "---\ntype: PartDef\nname: Manager\ndomain: software\nsatisfies: [REQ-VC-001]\nfeatures:\n  - {name: samples, type: Port, direction: in}\n  - {name: trip, type: Port, direction: out}\n---\n");
    put("Arch/Relay.md", "---\ntype: PartDef\nname: Relay\ndomain: software\nsatisfies: [REQ-VC-002]\nexhibitsStates: [Exec::RelayM]\nfeatures:\n  - {name: cmd, type: Port, direction: in}\n---\n");
    put("Arch/Sensor.md", "---\ntype: PartDef\nname: Sensor\ndomain: software\nsatisfies: [REQ-VC-003]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\n");
    put("Arch/Board.md", "---\ntype: PartDef\nname: Board\ndomain: software\nfeatures:\n  - {name: sensor, typedBy: Arch::Sensor}\n  - {name: mgr, typedBy: Arch::Manager}\n  - {name: relay, typedBy: Arch::Relay}\nconnections:\n  - {from: mgr.trip, to: relay.cmd}\n---\n");
    put("Exec/MonM.md", "---\ntype: StateDef\nname: MonM\nsubStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {target: A, accept: {payload: Sample, via: samples}, effect: {send: Trip, via: out}}\n---\n");
    put("Exec/Monitor.md", "---\ntype: PartDef\nname: Monitor\nmodels: [Arch::Manager]\nportMap: {out: trip}\nexhibitsStates: [Exec::MonM]\nfeatures:\n  - {name: samples, type: Port, direction: in}\n  - {name: out, type: Port, direction: out}\n---\n");
    put("Exec/RelayM.md", "---\ntype: StateDef\nname: RelayM\nsubStates:\n  - name: Closed\n    isInitial: true\n    transitions:\n      - {target: Open, accept: {payload: Trip, via: cmd}}\n  - {name: Open}\n---\n");
    put("Exec/StimM.md", "---\ntype: StateDef\nname: StimM\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {target: Idle, accept: Go, effect: {send: Sample, via: out}}\n---\n");
    put("Exec/Stim.md", "---\ntype: PartDef\nname: Stim\nexhibitsStates: [Exec::StimM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\n");
    put(
        "Exec/Bench.md",
        "---\ntype: PartDef\nname: Bench\nfeatures:\n  - {name: stim, typedBy: Exec::Stim, role: environment}\n  - {name: mon, typedBy: Exec::Monitor}\n  - {name: rel, typedBy: Arch::Relay}\nconnections:\n  - {from: stim.out, to: mon.samples}\n  - {from: mon.out, to: rel.cmd}\n---\n",
    );
    put(
        "FMEA-VC-001.md",
        "---\ntype: FMEASheet\nid: FMEA-VC-001\nname: \"FMEA of the bench\"\nstatus: draft\nentries:\n  - {id: FM-VC-001, ref: Exec::Stim, failureMode: \"stimulus dead\", injection: [{kind: silent}]}\n  - {id: FM-VC-002, ref: Exec::Monitor, failureMode: \"monitor dead\", injection: [{kind: silent}]}\n---\n",
    );
    put(
        "TC-VC-001.md",
        "---\ntype: TestCase\nid: TC-VC-001\nname: bench\nstatus: active\ntestLevel: L3\nsubject: Exec::Bench\nverifies: [REQ-VC-001, REQ-VC-002]\n---\n\n```gherkin\nFeature: bench\n\n  @invariant\n  Scenario: a sample opens the relay\n    When stim receives Go\n    Then rel is in state Open\n```\n",
    );
    m
}

fn write(files: &BTreeMap<&'static str, String>) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-vcomp-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&r);
    for (k, v) in files {
        let p = r.join(k);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, v).unwrap();
    }
    r
}

fn run(files: &BTreeMap<&'static str, String>, args: &[&str]) -> String {
    let root = write(files);
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).args(args).output().unwrap();
    let _ = std::fs::remove_dir_all(&root);
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn validate(files: &BTreeMap<&'static str, String>) -> String {
    run(files, &[])
}

fn lines<'a>(out: &'a str, code: &str) -> Vec<&'a str> {
    out.lines().filter(|l| l.starts_with(&format!("| {code} |"))).collect()
}

fn edit(f: &mut BTreeMap<&'static str, String>, k: &'static str, from: &str, to: &str) {
    let v = f.get_mut(k).unwrap();
    assert!(v.contains(from), "{k} has no {from:?}");
    *v = v.replace(from, to);
}

#[test]
fn a_verification_composite_validates_clean() {
    let out = validate(&base());
    for c in ["E990", "E991", "E992", "W990", "W991", "W992", "W993", "E930", "E931", "E932", "W936"] {
        assert!(lines(&out, c).is_empty(), "{c}:\n{out}");
    }
    assert!(!out.lines().any(|l| l.starts_with("| E") && l.as_bytes().get(3).is_some_and(u8::is_ascii_digit)), "{out}");
}

#[test]
fn an_architecture_element_used_directly_is_its_own_counterpart() {
    // `rel` is Arch::Relay itself: REQ-VC-002 is evidenced (no W992), and `mon.out → rel.cmd` is checked against Board
    let mut f = base();
    edit(&mut f, "Arch/Board.md", "connections:\n  - {from: mgr.trip, to: relay.cmd}\n", "");
    let out = validate(&f);
    assert!(lines(&out, "W991").iter().any(|l| l.contains("connection mon.out → rel.cmd of 'Exec::Bench' has no counterpart in the architecture: no connection joins 'Arch::Manager.trip' and 'Arch::Relay.cmd'")), "{out}");
    assert!(lines(&out, "W992").is_empty(), "{out}");
}

#[test]
fn an_environment_part_has_no_architecture_counterpart_and_is_no_evidence() {
    // Stim models Arch::Sensor; as an environment part its connection is not compared with the architecture
    let mut f = base();
    edit(&mut f, "Exec/Stim.md", "exhibitsStates:", "models: [Arch::Sensor]\nexhibitsStates:");
    edit(&mut f, "TC-VC-001.md", "verifies: [REQ-VC-001, REQ-VC-002]", "verifies: [REQ-VC-001, REQ-VC-002, REQ-VC-003]");
    let out = validate(&f);
    assert!(lines(&out, "W991").is_empty(), "{out}");
    assert!(lines(&out, "W992").iter().any(|l| l.contains("REQ-VC-003")), "an environment part is no evidence for the sensor's requirement:\n{out}");
    // the same part as a product part: the connection needs a counterpart, and it is evidence
    edit(&mut f, "Exec/Bench.md", ", role: environment}", "}");
    let out = validate(&f);
    assert!(lines(&out, "W991").iter().any(|l| l.contains("connection stim.out → mon.samples")), "{out}");
    assert!(lines(&out, "W992").is_empty(), "{out}");
}

#[test]
fn models_on_an_environment_usage_is_ignored_with_w993() {
    let mut f = base();
    edit(&mut f, "Exec/Bench.md", "{name: stim, typedBy: Exec::Stim, role: environment}", "{name: stim, typedBy: Exec::Stim, role: environment, models: [Arch::Sensor]}");
    let out = validate(&f);
    assert!(lines(&out, "W993").iter().any(|l| l.contains("part 'stim' of 'Exec::Bench': is `role: environment`")), "{out}");
    assert!(lines(&out, "E991").is_empty() && lines(&out, "W991").is_empty(), "{out}");
}

#[test]
fn an_unknown_role_or_a_role_off_a_part_usage_is_e992() {
    let mut f = base();
    edit(&mut f, "Exec/Bench.md", "role: environment}", "role: enviroment}");
    let out = validate(&f);
    assert!(lines(&out, "E992").iter().any(|l| l.contains("feature 'stim' has `role: enviroment`, which is not a role (the roles are: environment)")), "{out}");
    let mut f = base();
    edit(&mut f, "Exec/Stim.md", "{name: out, type: Port, direction: out}", "{name: out, type: Port, direction: out, role: environment}");
    let out = validate(&f);
    assert!(lines(&out, "E992").iter().any(|l| l.contains("feature 'out' has `role:`, which is recognised on a part usage only")), "{out}");
}

#[test]
fn simulate_coverage_lists_environment_parts_apart() {
    let out = run(&base(), &["simulate", "Exec::Bench", "--coverage"]);
    assert!(out.contains("1/1 scenario(s) pass"), "{out}");
    assert!(out.contains("coverage of Exec::Bench.stim (Exec::StimM) [environment]"), "{out}");
    assert!(out.contains("coverage of Exec::Bench.mon (Exec::MonM) (hash"), "{out}");
    // stim's one state is not in the aggregate: mon's one and rel's two
    assert!(out.lines().any(|l| l.starts_with("aggregate coverage of Exec::Bench") && l.contains("states 3/3") && l.ends_with("(product parts only; environment parts not counted: stim)")), "{out}");
    let json = run(&base(), &["simulate", "Exec::Bench", "--coverage", "--json"]);
    let start = json.find('{').unwrap();
    let v: serde_json::Value = serde_json::from_str(&json[start..]).unwrap_or_else(|e| panic!("{e}: {json}"));
    let cov = &v["coverage"];
    assert_eq!(cov["environment"], serde_json::json!(["stim"]), "{cov}");
    let roles: Vec<(String, String)> = cov["parts"].as_array().unwrap().iter().map(|p| (p["part"].as_str().unwrap().to_string(), p["role"].as_str().unwrap().to_string())).collect();
    assert!(roles.contains(&("stim".into(), "environment".into())) && roles.contains(&("mon".into(), "product".into())), "{roles:?}");
}

#[test]
fn inject_marks_faults_of_environment_parts() {
    let out = run(&base(), &["inject", "Exec::Bench", "--report-only"]);
    assert!(out.contains("environment parts (not the product): stim"), "{out}");
    assert!(out.lines().any(|l| l.contains("FM-VC-001 [environment]")), "{out}");
    assert!(out.lines().any(|l| l.contains("FM-VC-002") && !l.contains("[environment]")), "{out}");
    let json = run(&base(), &["inject", "Exec::Bench", "--report-only", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json[json.find('{').unwrap()..]).unwrap_or_else(|e| panic!("{e}: {json}"));
    assert_eq!(v["environment"], serde_json::json!(["stim"]));
    let env: Vec<bool> = v["sets"].as_array().unwrap().iter().map(|s| s["environment"].as_bool().unwrap()).collect();
    assert_eq!(env.iter().filter(|e| **e).count(), 1, "{v}");
}

#[test]
fn an_architecture_composite_with_an_undomained_part_is_still_architecture() {
    // a cable with no `domain:` in a domain-less Board: Board's connections still count as architecture
    let mut f = base();
    edit(&mut f, "Arch/Board.md", "domain: software\n", "");
    edit(&mut f, "Arch/Board.md", "  - {name: relay, typedBy: Arch::Relay}\n", "  - {name: relay, typedBy: Arch::Relay}\n  - {name: cable, typedBy: Arch::Cable}\n");
    f.insert("Arch/Cable.md", "---\ntype: PartDef\nname: Cable\n---\n".into());
    let out = validate(&f);
    assert!(lines(&out, "W991").is_empty(), "{out}");
}

#[test]
fn a_composite_with_an_environment_part_is_never_architecture() {
    // every product part of Bench2 is an architecture element; its environment part makes it a verification composite
    let mut f = base();
    f.insert(
        "Exec/Bench2.md",
        "---\ntype: PartDef\nname: Bench2\nfeatures:\n  - {name: env, typedBy: Arch::Sensor, role: environment}\n  - {name: mgr, typedBy: Arch::Manager}\n  - {name: rel, typedBy: Arch::Relay}\nconnections:\n  - {from: mgr.trip, to: rel.cmd}\n  - {from: rel.cmd, to: mgr.samples}\n---\n".into(),
    );
    let out = validate(&f);
    assert!(lines(&out, "W991").iter().any(|l| l.contains("connection rel.cmd → mgr.samples of 'Exec::Bench2' has no counterpart")), "{out}");
}

#[test]
fn a_missing_connection_between_elements_used_directly_is_reported() {
    let mut f = base();
    edit(&mut f, "Exec/Bench.md", "  - {from: mon.out, to: rel.cmd}\n", "");
    let out = validate(&f);
    assert!(lines(&out, "W991").iter().any(|l| l.contains("architecture connection 'Arch::Manager.trip' → 'Arch::Relay.cmd' (in 'Arch::Board') has no counterpart in 'Exec::Bench'")), "{out}");
}

#[test]
fn coverage_gates_ignore_environment_parts() {
    let mut f = base();
    edit(&mut f, "Exec/StimM.md", "effect: {send: Sample, via: out}}\n", "effect: {send: Sample, via: out}}\n      - {target: Never, accept: Stop}\n  - {name: Never}\n");
    let root = write(&f);
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).args(["simulate", "Exec::Bench", "--coverage", "--min-states", "100"]).output().unwrap();
    let t = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    assert!(t.contains("coverage of Exec::Bench.stim (Exec::StimM) [environment]") && t.contains("states        1/2"), "{t}");
    assert!(!t.contains("coverage gate"), "{t}");
    assert_eq!(o.status.code(), Some(0), "{t}");
    let json = run(&f, &["simulate", "Exec::Bench", "--coverage", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json[json.find('{').unwrap()..]).unwrap();
    assert_eq!(v["coverage"]["parts"][0]["basis"]["machine"], "Exec::Bench.stim (Exec::StimM)", "the label keeps its shape; `role` carries the role");
}

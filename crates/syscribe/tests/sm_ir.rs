//! TC-TRS-SMIR-001 / REQ-TRS-SMIR-001 (GH #278): lowering a StateDef to the IR, printed by `compile`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const MACHINE: &str = "variables:
  - {name: count, value: \"0\"}
  - {name: limit, value: \"3\", unit: \"SI::s\"}
subStates:
  - name: A
    isInitial: true
    entryAction: {assign: \"count := count + 1\"}
    transitions:
      - {target: B, accept: {after: \"2 s\"}, guard: \"count < 3\", effect: Pkg::Beep}
  - name: B
    transitions:
      - {target: A, accept: {when: \"count > 2\"}}
      - {internal: true, accept: {payload: Pkg::Ping, via: p, as: ev}, guard: \"ev.n > 1\", assign: \"count := 0\"}
      - {target: Done, accept: {done: B1}}
    subStates:
      - name: B1
        isInitial: true
      - name: B2
        isFinal: true
  - name: Done
    isFinal: true
";

fn model(machine: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-smir-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{machine}---\n")).unwrap();
    root
}

fn compile(machine: &str, name: &str) -> (i32, String, serde_json::Value) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model(machine)).args(["compile", name, "--json"]).output().unwrap();
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    let json = serde_json::from_str(&out).unwrap_or(serde_json::Value::Null);
    (o.status.code().unwrap_or(-1), format!("{out}{}", String::from_utf8_lossy(&o.stderr)), json)
}

fn state<'a>(m: &'a serde_json::Value, path: &str) -> &'a serde_json::Value {
    m["states"].as_array().unwrap().iter().find(|s| s["path"] == path).unwrap_or_else(|| panic!("no state {path}: {m}"))
}

#[test]
fn flat_tables() {
    let (code, text, m) = compile(MACHINE, "M");
    assert_eq!(code, 0, "{text}");
    let paths: Vec<_> = m["states"].as_array().unwrap().iter().map(|s| s["path"].as_str().unwrap()).collect();
    assert_eq!(paths, ["A", "B", "B.B1", "B.B2", "Done"], "document order, parents before children");
    for (i, s) in m["states"].as_array().unwrap().iter().enumerate() {
        assert_eq!(s["id"], i);
    }
    assert_eq!(state(&m, "A")["kind"], "atomic");
    assert_eq!(state(&m, "B")["kind"], "compound");
    assert_eq!(state(&m, "B.B2")["kind"], "final");
    assert_eq!(state(&m, "B.B1")["parent"], state(&m, "B")["id"]);
    assert_eq!(state(&m, "B")["initial"], state(&m, "B.B1")["id"]);
    assert_eq!(m["initial"], state(&m, "A")["id"]);
    assert_eq!(m["variables"][0]["name"], "count");
    assert_eq!(m["variables"][0]["slot"], 0);
    assert_eq!(m["variables"][1]["slot"], 1);
    assert_eq!(m["variables"][1]["type"], "number");
}

#[test]
fn triggers_and_actions() {
    let (_, text, m) = compile(MACHINE, "M");
    let t = m["transitions"].as_array().unwrap();
    assert_eq!(t.len(), 4, "{text}");
    for (i, x) in t.iter().enumerate() {
        assert_eq!(x["id"], i, "transitions keep document order");
    }
    assert_eq!(t[0]["trigger"]["kind"], "timer");
    assert_eq!(t[0]["trigger"]["delaySeconds"], 2.0);
    assert_eq!(t[0]["guard"]["src"], "count < 3");
    assert_eq!(t[0]["actions"][0]["kind"], "behavior");
    assert_eq!(t[0]["actions"][0]["ref"], "Pkg::Beep");
    assert_eq!(t[1]["trigger"]["kind"], "change");
    assert_eq!(t[2]["internal"], true);
    assert!(t[2]["target"].is_null());
    assert_eq!(t[2]["trigger"]["kind"], "event");
    assert_eq!(t[2]["trigger"]["name"], "p/Pkg::Ping");
    assert_eq!(t[2]["trigger"]["bind"], "ev");
    assert_eq!(t[2]["actions"][0]["kind"], "assign");
    assert_eq!(t[2]["actions"][0]["var"], 0);
    assert_eq!(t[3]["trigger"]["kind"], "done");
    assert_eq!(t[3]["trigger"]["state"], state(&m, "B.B1")["id"]);
    assert_eq!(t[3]["target"], state(&m, "Done")["id"]);
    assert_eq!(state(&m, "A")["entry"][0]["kind"], "assign");
    assert!(m["events"].as_array().unwrap().iter().any(|e| e == "p/Pkg::Ping"));
    // names resolve to variable slots and event fields
    let refs = t[2]["guard"]["refs"].as_array().unwrap();
    assert!(refs.iter().any(|r| r["name"] == "ev.n" && r["kind"] == "event"), "{refs:?}");
    let refs0 = t[0]["guard"]["refs"].as_array().unwrap();
    assert!(refs0.iter().any(|r| r["name"] == "count" && r["kind"] == "var" && r["slot"] == 0), "{refs0:?}");
}

#[test]
fn source_and_hash() {
    let (_, _, m) = compile(MACHINE, "M");
    assert_eq!(m["transitions"][0]["origin"]["qname"], "M");
    assert_eq!(m["transitions"][0]["origin"]["path"], "subStates[0].transitions[0]");
    assert_eq!(m["transitions"][3]["origin"]["path"], "subStates[1].transitions[2]");
    assert_eq!(state(&m, "B.B2")["origin"]["path"], "subStates[1].subStates[1]");
    let h = m["hash"].as_str().unwrap();
    assert_eq!(h.len(), 64);
    let (_, _, again) = compile(MACHINE, "M");
    assert_eq!(again["hash"], m["hash"], "deterministic");
    let (_, _, other) = compile(&MACHINE.replace("count < 3", "count < 4"), "M");
    assert_ne!(other["hash"], m["hash"], "a changed machine changes the hash");
}

#[test]
fn diagnostics() {
    let (c, t, _) = compile(&MACHINE.replace("target: Done", "target: Nowhere"), "M");
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("Nowhere"), "{t}");
    let (c, t, _) = compile(&MACHINE.replace("count < 3", "the door is jammed"), "M");
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("W933"), "{t}");
    let (c, t, _) = compile(&MACHINE.replace("{after: \"2 s\"}", "{after: \"2 kg\"}"), "M");
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("E882"), "{t}");
    let amb = "subStates:\n  - {name: A, isInitial: true, transitions: [{target: X, accept: Pkg::Go}]}\n  - {name: P, subStates: [{name: X, isInitial: true}]}\n  - {name: Q, subStates: [{name: X, isInitial: true}]}\n";
    let (c, t, _) = compile(amb, "M");
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("ambiguous"), "{t}");
}

#[test]
fn usage() {
    let (c, _, _) = compile(MACHINE, "Nope");
    assert_eq!(c, 1);
    let (c, _, _) = compile(MACHINE, "Root");
    assert_eq!(c, 1, "a package is not a state machine");
}

fn simulate(machine: &str, script: &str, extra: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model(machine)).args(["simulate", "M", "--do", script]).args(extra).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

#[test]
fn simulate_runs_a_script_and_prints_the_trace() {
    let (c, t) = simulate(MACHINE, "advance 2 s; send p/Pkg::Ping n=5", &["--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert!(j["active"].as_array().unwrap().iter().any(|s| s == "B"), "{t}");
    assert_eq!(j["machineHash"].as_str().unwrap().len(), 64);
    assert!(j["trace"].as_array().unwrap().iter().any(|s| s["cause"].as_str().unwrap().starts_with("timer")), "{t}");
    assert_eq!(j["variables"]["count"], "0");
    let (c, t) = simulate(MACHINE, "advance 2 s; send p/Pkg::Ping n=5", &[]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("timer"), "{t}");
}

#[test]
fn simulate_exits_2_on_a_run_error_and_1_on_a_bad_script() {
    let (c, t) = simulate(MACHINE, "advance 2 s; send p/Pkg::Ping", &[]);
    assert_eq!(c, 2, "missing payload field n: {t}");
    let (c, _) = simulate(MACHINE, "frobnicate", &[]);
    assert_eq!(c, 1);
}

#[test]
fn a_target_name_prefers_the_sources_own_children() {
    let y = "subStates:\n  - name: A\n    isInitial: true\n    transitions: [{target: X, accept: Pkg::Go}]\n    subStates: [{name: X, isInitial: true}]\n  - {name: B, subStates: [{name: X, isInitial: true}]}\n";
    let (c, t, m) = compile(y, "M");
    assert_eq!(c, 0, "{t}");
    assert_eq!(m["transitions"][0]["target"], state(&m, "A.X")["id"]);
}

#[test]
fn a_transition_without_a_target_or_a_trigger_is_a_diagnostic_not_a_silent_drop() {
    let (c, t, _) = compile("subStates:\n  - name: A\n    isInitial: true\n    transitions: [{accept: Pkg::Go}]\n", "M");
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("W929"), "{t}");
}

fn run_cli(machine: &str, args: &[&str]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model(machine)).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).to_string(), String::from_utf8_lossy(&o.stderr).to_string())
}

#[test]
fn export_scxml_writes_a_document_and_notes() {
    let (c, out, err) = run_cli(MACHINE, &["export-scxml", "M"]);
    assert_eq!(c, 0, "{err}");
    assert!(out.contains("<scxml ") && out.contains("<final id=\"B.B2\""), "{out}");
    assert!(err.contains("Pkg::Beep"), "behaviour references are reported: {err}");
    let (c, _, _) = run_cli(MACHINE, &["export-scxml", "Nope"]);
    assert_eq!(c, 1);
}

const TINY: &str = "subStates:\n  - name: A\n    isInitial: true\n    transitions: [{target: B, accept: Cmd::Go}]\n  - {name: B}\n";

#[test]
fn crosscheck_accepts_an_agreeing_engine_and_reports_a_difference() {
    let (c, out, err) = run_cli(TINY, &["simulate", "M", "--do", "send Cmd::Go", "--crosscheck", "cat >/dev/null; echo '{\"configurations\":[[\"A\"],[\"B\"]]}'"]);
    assert_eq!(c, 0, "{out}{err}");
    assert!(out.contains("agrees"), "{out}");
    let (c, _, err) = run_cli(TINY, &["simulate", "M", "--do", "send Cmd::Go", "--crosscheck", "cat >/dev/null; echo '{\"configurations\":[[\"A\"],[\"A\"]]}'"]);
    assert_eq!(c, 2);
    assert!(err.contains("after command 1") && err.contains("\"B\"") , "{err}");
    let (c, _, err) = run_cli(TINY, &["simulate", "M", "--do", "send Cmd::Go", "--crosscheck", "cat >/dev/null; exit 3"]);
    assert_eq!(c, 2);
    assert!(err.contains("engine failed"), "{err}");
}

#[test]
fn the_engine_receives_the_document_and_the_commands() {
    let (c, out, err) = run_cli(TINY, &["simulate", "M", "--do", "send Cmd::Go; advance 2 s", "--crosscheck", "in=$(cat); echo \"$in\" | grep -q '\"event\":\"Cmd.Go\"' && echo \"$in\" | grep -q '<scxml' && echo '{\"configurations\":[[\"A\"],[\"B\"],[\"B\"]]}'"]);
    assert_eq!(c, 0, "{out}{err}");
}

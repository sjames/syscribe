//! GH #333 / #344: `requirement-coverage` and `exec-coverage`, the requirement view and the architecture-element view
//! of one executable-evidence engine; `@verifies` scenario tags (W994, W995); `mutate --record`; the MCP tools; and the
//! grid-tie inverter as the acceptance case.

mod common;
use common::*;
use serde_json::{json, Value};

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// `Arch::Ctl` carries the machine `Exec::CtlM` and satisfies REQ-EC-001/002/006 is satisfied by `Arch::Other` (no
/// machine). TC-EC-001 (subject CtlM) tags its scenarios; TC-EC-002 is descriptive.
fn base() -> BTreeMap<&'static str, String> {
    let mut m = BTreeMap::new();
    let mut put = |k: &'static str, v: &str| {
        m.insert(k, v.to_string());
    };
    put("_index.md", "---\ntype: Package\nname: Root\n---\n");
    let req = |id: &str, extra: &str| format!("---\ntype: Requirement\nid: {id}\nname: \"{id}\"\nstatus: draft\nreqDomain: software\n{extra}---\n\nThe controller shall do {id}.\n");
    put("Reqs/REQ-EC-001.md", &req("REQ-EC-001", ""));
    put("Reqs/REQ-EC-002.md", &req("REQ-EC-002", ""));
    put("Reqs/REQ-EC-003.md", &req("REQ-EC-003", ""));
    put("Reqs/REQ-EC-004.md", &req("REQ-EC-004", ""));
    put("Reqs/REQ-EC-005.md", &req("REQ-EC-005", "verificationMethod: inspection\n"));
    put("Reqs/REQ-EC-006.md", &req("REQ-EC-006", ""));
    put("Arch/Ctl.md", "---\ntype: PartDef\nname: Ctl\ndomain: software\nsatisfies: [REQ-EC-001, REQ-EC-002]\nexhibitsStates: [Exec::CtlM]\n---\n");
    put("Arch/Other.md", "---\ntype: PartDef\nname: Other\ndomain: software\nsatisfies: [REQ-EC-003, REQ-EC-006]\n---\n");
    put(
        "Exec/CtlM.md",
        "---\ntype: StateDef\nname: CtlM\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {target: Run, accept: Go}\n  - name: Run\n    transitions:\n      - {target: Idle, accept: Stop}\n      - {target: Fault, accept: Bad}\n  - name: Fault\n---\n",
    );
    put(
        "TC-EC-001.md",
        "---\ntype: TestCase\nid: TC-EC-001\nname: controller\nstatus: active\ntestLevel: L2\nsubject: Exec::CtlM\nverifies: [REQ-EC-001, REQ-EC-002, REQ-EC-006]\n---\n\n```gherkin\nFeature: controller\n\n  @verifies REQ-EC-001\n  Scenario: go runs\n    When Go is injected\n    Then CtlM is in state Run\n\n  @verifies REQ-EC-002, REQ-EC-006\n  Scenario: stop idles\n    When Go is injected\n    And Stop is injected\n    Then CtlM is in state Idle\n```\n",
    );
    put("TC-EC-002.md", "---\ntype: TestCase\nid: TC-EC-002\nname: bench check\nstatus: active\ntestLevel: L4\nverifies: [REQ-EC-003]\n---\n\nMeasure it on the bench.\n");
    m
}

fn write(files: &BTreeMap<&'static str, String>) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-execcov-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&r);
    for (k, v) in files {
        let p = r.join(k);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, v).unwrap();
    }
    r
}

fn cli(root: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

fn run(files: &BTreeMap<&'static str, String>, args: &[&str]) -> (i32, String) {
    let root = write(files);
    let r = cli(&root, args);
    let _ = std::fs::remove_dir_all(&root);
    r
}

fn json_of(files: &BTreeMap<&'static str, String>, args: &[&str]) -> Value {
    let (code, out) = run(files, args);
    assert!(code == 0 || code == 2, "{out}");
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"))
}

fn req<'a>(v: &'a Value, id: &str) -> &'a Value {
    v["requirements"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap_or_else(|| panic!("no {id} in {v}"))
}

fn elem<'a>(v: &'a Value, q: &str) -> &'a Value {
    v["elements"].as_array().unwrap().iter().find(|r| r["qualifiedName"] == q).unwrap_or_else(|| panic!("no {q} in {v}"))
}

fn edit(f: &mut BTreeMap<&'static str, String>, k: &'static str, from: &str, to: &str) {
    let v = f.get_mut(k).unwrap();
    assert!(v.contains(from), "{k} has no {from:?}");
    *v = v.replace(from, to);
}

fn lines<'a>(out: &'a str, code: &str) -> Vec<&'a str> {
    out.lines().filter(|l| l.starts_with(&format!("| {code} |"))).collect()
}

#[test]
fn each_requirement_gets_a_verdict() {
    let v = json_of(&base(), &["requirement-coverage", "--json"]);
    let verdict = |id: &str| req(&v, id)["verdict"].as_str().unwrap().to_string();
    assert_eq!(verdict("REQ-EC-001"), "executable");
    assert_eq!(verdict("REQ-EC-002"), "executable");
    assert_eq!(verdict("REQ-EC-003"), "descriptive-only");
    assert_eq!(verdict("REQ-EC-004"), "unverified");
    assert_eq!(verdict("REQ-EC-005"), "not-applicable");
    // CtlM is carried by Arch::Ctl only: the scenario that claims REQ-EC-006 exercises nothing that satisfies it
    assert_eq!(verdict("REQ-EC-006"), "unverified");
    assert_eq!(req(&v, "REQ-EC-006")["claimedNotExercised"][0]["scenario"], "stop idles", "{v}");
    assert_eq!(v["summary"]["executable"], 2);
    assert_eq!(v["summary"]["notApplicable"], 1);
    assert_eq!(req(&v, "REQ-EC-003")["descriptive"][0]["testCase"], "TC-EC-002");
}

#[test]
fn verifies_tags_narrow_a_scenario_to_its_requirements() {
    let v = json_of(&base(), &["requirement-coverage", "--json"]);
    let scen = |id: &str| req(&v, id)["scenarios"].as_array().unwrap().iter().map(|s| s["scenario"].as_str().unwrap().to_string()).collect::<Vec<_>>();
    assert_eq!(scen("REQ-EC-001"), vec!["go runs"]);
    assert_eq!(scen("REQ-EC-002"), vec!["stop idles"]);
    // coverage is merged over the requirement's own scenarios only
    let c1 = &req(&v, "REQ-EC-001")["coverage"][0];
    assert_eq!((c1["transitions"]["covered"].as_u64(), c1["transitions"]["total"].as_u64()), (Some(1), Some(3)), "{v}");
    let c2 = &req(&v, "REQ-EC-002")["coverage"][0];
    assert_eq!(c2["transitions"]["covered"], 2, "{v}");
    // untagged, every scenario verifies every requirement of the TestCase
    let mut f = base();
    edit(&mut f, "TC-EC-001.md", "  @verifies REQ-EC-001\n", "");
    edit(&mut f, "TC-EC-001.md", "  @verifies REQ-EC-002, REQ-EC-006\n", "");
    let v = json_of(&f, &["requirement-coverage", "--json"]);
    assert_eq!(req(&v, "REQ-EC-001")["scenarios"].as_array().unwrap().len(), 2, "{v}");
    // a `# verifies:` comment before a step works like a tag
    let mut f = base();
    edit(&mut f, "TC-EC-001.md", "  @verifies REQ-EC-001\n  Scenario: go runs\n    When Go", "  Scenario: go runs\n    # verifies: REQ-EC-002\n    When Go");
    let v = json_of(&f, &["requirement-coverage", "--json"]);
    assert_eq!(req(&v, "REQ-EC-002")["scenarios"].as_array().unwrap().len(), 2, "{v}");
    assert_eq!(req(&v, "REQ-EC-001")["verdict"], "unverified", "{v}");
}

#[test]
fn thresholds_and_failing_scenarios_make_evidence_weak() {
    let v = json_of(&base(), &["requirement-coverage", "--json", "--min-transitions", "50"]);
    assert_eq!(req(&v, "REQ-EC-001")["verdict"], "executable-weak", "{v}");
    assert!(req(&v, "REQ-EC-001")["reasons"][0].as_str().unwrap().contains("Exec::CtlM: transitions 33% < 50%"), "{v}");
    assert_eq!(req(&v, "REQ-EC-002")["verdict"], "executable", "{v}");
    let mut f = base();
    edit(&mut f, "TC-EC-001.md", "Then CtlM is in state Idle", "Then CtlM is in state Fault");
    let v = json_of(&f, &["requirement-coverage", "--json"]);
    assert_eq!(req(&v, "REQ-EC-002")["verdict"], "executable-weak", "{v}");
    assert!(req(&v, "REQ-EC-002")["reasons"][0].as_str().unwrap().contains("scenario 'stop idles' of TC-EC-001 is fail"), "{v}");
    // a mutation threshold with no recorded score is weak and says how to get one
    let v = json_of(&base(), &["requirement-coverage", "--json", "--min-score", "50"]);
    assert!(req(&v, "REQ-EC-001")["reasons"][0].as_str().unwrap().contains("no mutation score"), "{v}");
}

#[test]
fn requirement_coverage_gates_exit_2() {
    let f = base();
    assert_eq!(run(&f, &["requirement-coverage", "--require", "REQ-EC-001"]).0, 0);
    let (code, out) = run(&f, &["requirement-coverage", "--require", "REQ-EC-003"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("REQ-EC-003 is descriptive-only, not executable"), "{out}");
    // 2 executable of 5 applicable = 40%
    let (code, out) = run(&f, &["requirement-coverage", "--min-executable", "50"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("40.0% of the applicable requirements"), "{out}");
    assert_eq!(run(&f, &["requirement-coverage", "--min-executable", "40"]).0, 0);
    let (code, out) = run(&f, &["requirement-coverage", "--require", "REQ-NOPE-001"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(run(&f, &["requirement-coverage", "--min-executable", "x"]).0, 1);
    assert_eq!(run(&f, &["requirement-coverage", "--config", "CONF-NOPE"]).0, 1);
}

#[test]
fn requirement_coverage_text_shows_detail_for_named_requirements() {
    let (code, out) = run(&base(), &["requirement-coverage", "REQ-EC-001"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("REQ-EC-001  executable"), "{out}");
    assert!(out.contains("scenario  TC-EC-001 / go runs  [pass]"), "{out}");
    assert!(out.contains("coverage  Exec::CtlM: states 2/3, transitions 1/3"), "{out}");
    assert!(out.contains("satisfied by Arch::Ctl"), "{out}");
}

#[test]
fn unknown_verifies_tags_and_unclaimed_requirements_warn() {
    let mut f = base();
    edit(&mut f, "TC-EC-001.md", "@verifies REQ-EC-001\n", "@verifies REQ-EC-004\n");
    let (_, out) = run(&f, &[]);
    assert!(lines(&out, "W994").iter().any(|l| l.contains("TC-EC-001: scenario 'go runs' claims REQ-EC-004")), "{out}");
    // every scenario is tagged and none claims REQ-EC-001
    assert!(lines(&out, "W995").iter().any(|l| l.contains("TC-EC-001 verifies REQ-EC-001, but every scenario is tagged and none claims it")), "{out}");
    // the base model is clean
    let (_, out) = run(&base(), &[]);
    assert!(lines(&out, "W994").is_empty() && lines(&out, "W995").is_empty(), "{out}");
    // one untagged scenario verifies everything: no W995
    let mut f = base();
    edit(&mut f, "TC-EC-001.md", "  @verifies REQ-EC-002, REQ-EC-006\n", "");
    let (_, out) = run(&f, &[]);
    assert!(lines(&out, "W995").is_empty(), "{out}");
}

#[test]
fn exec_coverage_shows_each_architecture_element() {
    let v = json_of(&base(), &["exec-coverage", "--json"]);
    let ctl = elem(&v, "Arch::Ctl");
    assert_eq!(ctl["executableItself"], true);
    assert_eq!(ctl["scenarios"], 2);
    assert_eq!(ctl["requirements"]["executable"], json!(["REQ-EC-001", "REQ-EC-002"]));
    let other = elem(&v, "Arch::Other");
    assert_eq!(other["executableItself"], false);
    assert_eq!(other["modelledBy"], json!([]));
    assert_eq!(other["scenarios"], 0);
    assert_eq!(other["requirements"]["descriptiveOnly"], json!(["REQ-EC-003"]));
    assert_eq!(other["requirements"]["unverified"], json!(["REQ-EC-006"]));
    assert_eq!(v["summary"]["modelled"], 1);
    let f = base();
    let (code, out) = run(&f, &["exec-coverage", "--min-modelled", "60"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("1 of 2 software/hardware element(s) modelled (50.0%)"), "{out}");
    let (code, out) = run(&f, &["exec-coverage", "--require", "Arch::Other"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("Arch::Other is not modelled"), "{out}");
    assert_eq!(run(&f, &["exec-coverage", "--require", "Arch::Ctl"]).0, 0);
}

#[test]
fn an_executable_part_that_models_an_element_covers_it() {
    let mut f = base();
    f.insert("Exec/OtherSim.md", "---\ntype: PartDef\nname: OtherSim\nmodels: [Arch::Other]\nexhibitsStates: [Exec::CtlM]\n---\n".to_string());
    let v = json_of(&f, &["exec-coverage", "--json"]);
    let other = elem(&v, "Arch::Other");
    assert_eq!(other["modelledBy"], json!(["Exec::OtherSim"]), "{v}");
    assert_eq!(other["scenarios"], 2, "{v}");
    let v = json_of(&f, &["requirement-coverage", "--json"]);
    assert_eq!(req(&v, "REQ-EC-006")["verdict"], "executable", "{v}");
}

#[test]
fn recorded_mutation_scores_are_read_and_go_stale() {
    let root = write(&base());
    let (code, out) = cli(&root, &["mutate", "Exec::CtlM", "--record"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("recorded 1 mutation score(s)"), "{out}");
    assert!(root.join(".syscribe/mutation.json").is_file());
    let (_, out) = cli(&root, &["requirement-coverage", "--json"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    let m = &req(&v, "REQ-EC-001")["mutation"][0];
    assert_eq!((m["machine"].as_str(), m["source"].as_str(), m["stale"].as_bool()), (Some("Exec::CtlM"), Some("recorded"), Some(false)), "{v}");
    let (_, out) = cli(&root, &["exec-coverage", "--json"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(elem(&v, "Arch::Ctl")["mutation"][0]["machine"], "Exec::CtlM", "{v}");
    // the machine changes: the score is stale
    let p = root.join("Exec/CtlM.md");
    let t = std::fs::read_to_string(&p).unwrap().replace("  - name: Fault\n", "  - name: Fault\n    transitions:\n      - {target: Idle, accept: Reset}\n");
    std::fs::write(&p, t).unwrap();
    let (_, out) = cli(&root, &["requirement-coverage", "--json"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(req(&v, "REQ-EC-001")["mutation"][0]["stale"], true, "{v}");
    // --mutate computes a fresh score
    let (_, out) = cli(&root, &["requirement-coverage", "--json", "--mutate"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(req(&v, "REQ-EC-001")["mutation"][0]["source"], "computed", "{v}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn mcp_tools_return_the_cli_evidence() {
    let root = write(&base());
    let before = dir_hash(&root);
    let mut mcp = Mcp::start(&root);
    mcp.initialize();
    let r = mcp.call_tool("requirement_coverage", json!({"only": ["REQ-EC-001", "REQ-EC-003"]}));
    assert_eq!(r["requirements"].as_array().unwrap().len(), 2, "{r}");
    assert_eq!(req(&r, "REQ-EC-001")["verdict"], "executable", "{r}");
    assert_eq!(req(&r, "REQ-EC-003")["verdict"], "descriptive-only", "{r}");
    let e = mcp.call_tool("exec_coverage", json!({}));
    assert_eq!(elem(&e, "Arch::Ctl")["scenarios"], 2, "{e}");
    assert_eq!(e["summary"]["modelled"], 1, "{e}");
    assert_eq!(before, dir_hash(&root), "the evidence tools write nothing");
    let _ = std::fs::remove_dir_all(&root);
}

/// The acceptance case: the grid-tie inverter's protection chain, US configuration.
#[test]
fn grid_tie_inverter_requirement_coverage() {
    let gti = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_gti");
    let (code, out) = cli(&gti, &["requirement-coverage", "--config", "CONF-GTI-US-M3-001", "--json"]);
    assert_eq!(code, 0, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap();
    assert!(v["errors"].as_array().unwrap().is_empty(), "{v}");
    assert!(v["scenarios"].as_array().unwrap().iter().all(|s| s["status"] == "pass"), "{v}");
    for id in ["REQ-GTI-SYS-007", "REQ-GTI-SYS-010", "REQ-GTI-SYS-030", "REQ-GTI-SYS-038", "REQ-GTI-SYS-039", "REQ-GTI-SW-001", "REQ-GTI-HW-008", "REQ-GTI-HW-009"] {
        assert_eq!(req(&v, id)["verdict"], "executable", "{id}");
    }
    // the insulation requirement is evidenced only by the insulation scenarios its tags name
    let ins = req(&v, "REQ-GTI-SYS-030")["scenarios"].as_array().unwrap();
    assert!(!ins.is_empty() && ins.iter().all(|s| s["scenario"].as_str().unwrap().contains("insulation")), "{ins:?}");
    // the hardware overcurrent trip is verified only by descriptive safety TestCases
    assert_eq!(req(&v, "REQ-GTI-SYS-037")["verdict"], "descriptive-only");
    assert_eq!(req(&v, "REQ-GTI-SYS-061")["verdict"], "not-applicable");
    let (code, out) = cli(&gti, &["exec-coverage", "--config", "CONF-GTI-US-M3-001", "--json"]);
    assert_eq!(code, 0, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap();
    assert!(elem(&v, "System::Software::Supervisor")["scenarios"].as_u64().unwrap() > 0, "{v}");
}

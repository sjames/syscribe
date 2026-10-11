//! TC-TRS-FAULT-002 / REQ-TRS-FAULT-002: the fault-injection campaign.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const FMEA: &str = "---
type: FMEASheet
id: FMEA-LINK-001
name: \"FMEA of the link\"
status: draft
entries:
  - {id: FM-TX-001, ref: TxP, failureMode: \"command lost\", injection: [{kind: drop, port: out}]}
  - {id: FM-TX-002, ref: TxP, failureMode: \"command late\", injection: [{kind: delay, port: out, by: \"10 ms\"}]}
  - {id: FM-TX-003, ref: TxP, failureMode: \"level stuck\", injection: [{kind: stuck, variable: level, value: \"3\"}]}
  - {id: FM-TX-004, ref: TxP, failureMode: \"level biased\", injection: [{kind: offset, variable: level, by: \"2\"}]}
  - {id: FM-TX-005, ref: TxP, failureMode: \"payload corrupted\", injection: [{kind: corrupt, port: out, field: n, value: \"99\"}]}
  - {id: FM-RX-001, ref: RxP, failureMode: \"receiver dead\", injection: [{kind: silent}]}
  - {id: FM-RX-002, ref: RxP, failureMode: \"an analysis row without a fault model\"}
---
";

const FILES: &[(&str, &str)] = &[
    ("TxM", "---\ntype: StateDef\nname: TxM\nvariables:\n  - {name: level, value: \"7\"}\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {target: Idle, accept: Tick, effect: {send: Cmd, via: out, payload: {n: \"level\"}}}\n---\n"),
    ("RxM", "---\ntype: StateDef\nname: RxM\nvariables:\n  - {name: got, value: \"0\"}\nsubStates:\n  - name: Waiting\n    isInitial: true\n    transitions:\n      - {target: Done, accept: {payload: Cmd, via: inp, as: m}, assign: \"got := m.n\"}\n  - {name: Done, isFinal: true}\n---\n"),
    ("TxP", "---\ntype: PartDef\nname: TxP\nexhibitsStates: [TxM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\nTransmitter.\n"),
    ("RxP", "---\ntype: PartDef\nname: RxP\nexhibitsStates: [RxM]\nfeatures:\n  - {name: inp, type: Port, direction: in}\n---\nReceiver.\n"),
    ("Sys", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\nconnections:\n  - {from: tx.out, to: rx.inp}\n---\nThe system.\n"),
    ("REQ-SYS-001", "---\ntype: Requirement\nid: REQ-SYS-001\nname: delivery\nstatus: approved\nreqDomain: software\n---\n\nThe receiver shall get the command.\n"),
    ("FMEA-LINK-001", FMEA),
];

const TC: &str = "---
type: TestCase
id: TC-SYS-001
name: link invariants
status: active
testLevel: L3
subject: Sys
verifies: [REQ-SYS-001]
---

```gherkin
Feature: link

  Scenario: nominal delivery
    When tx receives Tick
    Then rx is in state Done

  @invariant
  Scenario: the receiver never accepts a wrong value
    When tx receives Tick
    Then rx.got == 0 or rx.got == 7

  @Invariant
  Scenario: the receiver never sees more than the nominal value
    When tx receives Tick
    Then rx.got <= 7
```
";

fn model(tc: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-campaign-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, c) in FILES {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    std::fs::write(root.join("TC-SYS-001.md"), tc).unwrap();
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).to_string(), String::from_utf8_lossy(&o.stderr).to_string())
}

fn set<'a>(j: &'a serde_json::Value, label: &str) -> &'a serde_json::Value {
    j["sets"].as_array().unwrap().iter().find(|s| s["label"] == label).unwrap_or_else(|| panic!("no set {label}: {j}"))
}

#[test]
fn every_single_fault_is_run_against_the_invariants() {
    let (c, t, e) = run(&model(TC), &["inject", "Sys", "--json"]);
    assert_eq!(c, 2, "a defeated invariant fails the gate: {t}{e}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|x| panic!("{x}: {t}"));
    assert_eq!(j["invariants"], 2, "the two tagged scenarios, whatever the case of the tag");
    for ok in ["FM-TX-001", "FM-TX-002", "FM-RX-001"] {
        assert_eq!(set(&j, ok)["status"], "survived", "{ok}");
    }
    for bad in ["FM-TX-003", "FM-TX-004", "FM-TX-005"] {
        let s = set(&j, bad);
        assert_eq!(s["status"], "defeated", "{bad}: {s}");
        assert!(s["defeats"].as_array().unwrap().iter().any(|d| d["scenario"] == "the receiver never accepts a wrong value"), "{s}");
    }
    assert_eq!(j["sets"].as_array().unwrap().len(), 6, "the prose row has no fault model: {j}");
}

#[test]
fn the_text_report_and_the_report_only_flag() {
    let (c, t, _) = run(&model(TC), &["inject", "Sys", "--report-only"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("DEFEATED") && t.contains("FM-TX-005") && t.contains("SURVIVED") && t.contains("FM-TX-001"), "{t}");
    assert!(t.contains("never accepts a wrong value"), "the defeated invariant is named: {t}");
    assert!(t.contains("FM-RX-002") && t.contains("no usable fault model"), "analysis rows that cannot be exercised are listed: {t}");
}

#[test]
fn minimal_cut_sets_of_a_fault_tree_are_fault_sets() {
    let root = model(TC);
    std::fs::create_dir_all(root.join("FT-LINK-001")).unwrap();
    std::fs::write(root.join("FT-LINK-001.md"), "---\ntype: FaultTree\nid: FT-LINK-001\nname: tree\nstatus: draft\n---\nTree.\n").unwrap();
    let ev = |id: &str, extra: &str| format!("---\ntype: FaultTreeEvent\nid: {id}\nname: \"{id}\"\neventKind: basic\nref: TxP\nfailureRate: 1.0e-7\n{extra}---\nEvent.\n");
    std::fs::write(root.join("FT-LINK-001/FTE-TX-001.md"), ev("FTE-TX-001", "injection:\n  - {kind: silent}\n")).unwrap();
    std::fs::write(root.join("FT-LINK-001/FTE-LK-001.md"), ev("FTE-LK-001", "injection:\n  - {kind: drop, port: out}\n")).unwrap();
    std::fs::write(root.join("FT-LINK-001/FTE-XX-001.md"), ev("FTE-XX-001", "")).unwrap();
    std::fs::write(root.join("FT-LINK-001/FTG-AND-001.md"), "---\ntype: FaultTreeGate\nid: FTG-AND-001\nname: both\ngateType: AND\ninputs: [FTE-TX-001, FTE-LK-001]\n---\nBoth.\n").unwrap();
    std::fs::write(root.join("FT-LINK-001/FTG-TOP-001.md"), "---\ntype: FaultTreeGate\nid: FTG-TOP-001\nname: top\ngateType: OR\ninputs: [FTG-AND-001, FTE-XX-001]\n---\nTop.\n").unwrap();
    let (c, t, e) = run(&root, &["inject", "Sys", "--ft", "FT-LINK-001", "--max-order", "2", "--json", "--report-only"]);
    assert_eq!(c, 0, "{t}{e}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|x| panic!("{x}: {t}"));
    let both = set(&j, "FTE-LK-001 + FTE-TX-001");
    assert_eq!(both["status"], "survived", "{both}");
    assert!(both["origin"].as_str().unwrap().contains("cut set") && both["origin"].as_str().unwrap().contains("order 2"), "{both}");
    let ni = j["notInjectable"].as_array().unwrap();
    assert!(ni.iter().any(|n| n["events"].as_array().unwrap().iter().any(|x| x == "FTE-XX-001")), "an event without a fault model cannot be injected: {j}");
}

#[test]
fn the_cut_set_note_says_higher_orders_were_left_out_not_that_the_analysis_overflowed() {
    // GH #316: --max-order below the order of a cut set prunes it; that is the requested limit, not a cap being hit
    let root = model(TC);
    std::fs::create_dir_all(root.join("FT-LINK-001")).unwrap();
    std::fs::write(root.join("FT-LINK-001.md"), "---\ntype: FaultTree\nid: FT-LINK-001\nname: tree\nstatus: draft\n---\nTree.\n").unwrap();
    let ev = |id: &str, extra: &str| format!("---\ntype: FaultTreeEvent\nid: {id}\nname: \"{id}\"\neventKind: basic\nref: TxP\nfailureRate: 1.0e-7\n{extra}---\nEvent.\n");
    std::fs::write(root.join("FT-LINK-001/FTE-TX-001.md"), ev("FTE-TX-001", "injection:\n  - {kind: silent}\n")).unwrap();
    std::fs::write(root.join("FT-LINK-001/FTE-LK-001.md"), ev("FTE-LK-001", "injection:\n  - {kind: drop, port: out}\n")).unwrap();
    std::fs::write(root.join("FT-LINK-001/FTG-TOP-001.md"), "---\ntype: FaultTreeGate\nid: FTG-TOP-001\nname: top\ngateType: AND\ninputs: [FTE-TX-001, FTE-LK-001]\n---\nTop.\n").unwrap();
    let (_, t, e) = run(&root, &["inject", "Sys", "--ft", "FT-LINK-001", "--max-order", "1", "--report-only"]);
    assert!(t.contains("higher order") && !t.contains("too many"), "{t}{e}");
    let (_, t, e) = run(&root, &["inject", "Sys", "--ft", "FT-LINK-001", "--max-order", "2", "--report-only"]);
    assert!(!t.contains("higher order") && !t.contains("too many"), "all cut sets were run, so no note: {t}{e}");
}

#[test]
fn the_campaign_is_honest_about_what_it_can_say() {
    // no @invariant scenario at all
    let none = TC.replace("  @invariant\n", "").replace("  @Invariant\n", "");
    let (c, _, e) = run(&model(&none), &["inject", "Sys"]);
    assert_eq!(c, 1, "{e}");
    assert!(e.contains("@invariant"), "{e}");
    // an invariant that is false without faults
    let wrong = TC.replace("Then rx.got == 0 or rx.got == 7", "Then rx.got == 5");
    let (c, _, e) = run(&model(&wrong), &["inject", "Sys"]);
    assert_eq!(c, 2, "{e}");
    assert!(e.contains("without faults") && e.contains("never accepts a wrong value"), "{e}");
    // unknown sheet and fault tree are usage errors
    let (c, _, e) = run(&model(TC), &["inject", "Sys", "--fmea", "FMEA-NOPE-001"]);
    assert_eq!(c, 1, "{e}");
    let (c, _, e) = run(&model(TC), &["inject", "Sys", "--ft", "FT-NOPE-001"]);
    assert_eq!(c, 1, "{e}");
}

#[test]
fn a_testcase_can_be_named_and_a_sheet_restricts_the_faults() {
    let (c, t, _) = run(&model(TC), &["inject", "TC-SYS-001", "--fmea", "FMEA-LINK-001", "--json", "--report-only"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["sets"].as_array().unwrap().len(), 6);
}

#[test]
fn gherkin_tags_are_recorded_and_inherited_from_the_feature() {
    use syscribe_model::sm_scen::extract_scenarios;
    let doc = "```gherkin\n@safety\nFeature: f\n\n  @invariant @slow\n  Scenario: a\n    Then 1 == 1\n\n  Scenario: b\n    Then 1 == 1\n```\n";
    let s = extract_scenarios(doc);
    assert_eq!(s[0].tags, ["@safety", "@invariant", "@slow"]);
    assert_eq!(s[1].tags, ["@safety"]);
}

#[test]
fn an_empty_campaign_is_not_a_pass() {
    // no row of the sheet has a fault model that applies
    let root = model(TC);
    std::fs::write(root.join("FMEA-LINK-001.md"), "---\ntype: FMEASheet\nid: FMEA-LINK-001\nname: s\nstatus: draft\nentries:\n  - {id: FM-RX-002, ref: RxP, failureMode: \"prose only\"}\n---\n").unwrap();
    let (c, _, e) = run(&root, &["inject", "Sys"]);
    assert_eq!(c, 1, "{e}");
    assert!(e.contains("no fault set to run"), "{e}");
}

#[test]
fn tags_do_not_leak_to_the_wrong_scenario() {
    use syscribe_model::sm_scen::extract_scenarios;
    let doc = "```gherkin\nFeature: f\n\n  Scenario Outline: o\n    Then <a> == 1\n\n  @invariant\n  Examples:\n    | a |\n    | 1 |\n\n  Scenario: later\n    Then 1 == 1\n```\n\n```gherkin\n@safety\nFeature: g\n\n  Scenario: s\n    Then 1 == 1\n```\n\n```gherkin\nScenario: bare\n  Then 1 == 1\n```\n";
    let s = extract_scenarios(doc);
    let later = s.iter().find(|x| x.title == "later").unwrap();
    assert!(!later.has_tag("invariant"), "a tag above Examples does not tag the next scenario: {:?}", later.tags);
    let bare = s.iter().find(|x| x.title == "bare").unwrap();
    assert!(bare.tags.is_empty(), "feature tags do not carry into a later block: {:?}", bare.tags);
    assert!(s.iter().find(|x| x.title == "s").unwrap().has_tag("@safety"));
}

#[test]
fn invariants_must_not_inject_faults_themselves() {
    let tc = TC.replace("  @invariant\n  Scenario: the receiver never accepts a wrong value\n    When tx receives Tick", "  @invariant\n  Scenario: the receiver never accepts a wrong value\n    Given fault FM-TX-001 is injected\n    When tx receives Tick");
    let (c, _, e) = run(&model(&tc), &["inject", "Sys"]);
    assert_eq!(c, 1, "{e}");
    assert!(e.contains("injects the faults itself"), "{e}");
}

fn tree_with_events(root: &PathBuf, ccf: bool) {
    std::fs::create_dir_all(root.join("FT-LINK-001")).unwrap();
    std::fs::write(root.join("FT-LINK-001.md"), "---\ntype: FaultTree\nid: FT-LINK-001\nname: tree\nstatus: draft\n---\nTree.\n").unwrap();
    let ev = |id: &str, extra: &str| format!("---\ntype: FaultTreeEvent\nid: {id}\nname: \"{id}\"\neventKind: basic\nref: TxP\nfailureRate: 1.0e-7\n{extra}---\nEvent.\n");
    let group = if ccf { "ccfGroup: LINK\nccfBeta: 0.1\n" } else { "" };
    std::fs::write(root.join("FT-LINK-001/FTE-TX-001.md"), ev("FTE-TX-001", &format!("injection:\n  - {{kind: silent}}\n{group}"))).unwrap();
    std::fs::write(root.join("FT-LINK-001/FTE-LK-001.md"), ev("FTE-LK-001", &format!("injection:\n  - {{kind: drop, port: out}}\n{group}"))).unwrap();
    std::fs::write(root.join("FT-LINK-001/FTG-AND-001.md"), "---\ntype: FaultTreeGate\nid: FTG-AND-001\nname: both\ngateType: AND\ninputs: [FTE-TX-001, FTE-LK-001]\n---\nBoth.\n").unwrap();
}

#[test]
fn the_sheet_restriction_really_restricts_and_common_cause_runs_all_members() {
    let root = model(TC);
    tree_with_events(&root, true);
    let ids = |args: &[&str]| -> Vec<String> {
        let (_, t, e) = run(&root, args);
        let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|x| panic!("{x}: {t}{e}"));
        j["sets"].as_array().unwrap().iter().map(|s| s["label"].as_str().unwrap().to_string()).collect()
    };
    let all = ids(&["inject", "Sys", "--json", "--report-only"]);
    assert!(all.contains(&"FTE-TX-001".to_string()) && all.contains(&"FM-TX-001".to_string()), "{all:?}");
    let sheet = ids(&["inject", "Sys", "--json", "--report-only", "--fmea", "FMEA-LINK-001"]);
    assert!(sheet.contains(&"FM-TX-001".to_string()) && !sheet.contains(&"FTE-TX-001".to_string()), "{sheet:?}");
    // a common-cause group becomes a cut set that fails every member together
    let with_ft = ids(&["inject", "Sys", "--json", "--report-only", "--ft", "FT-LINK-001", "--max-order", "2"]);
    assert!(with_ft.contains(&"FTE-LK-001 + FTE-TX-001".to_string()), "{with_ft:?}");
    let (_, t, _) = run(&root, &["inject", "Sys", "--json", "--report-only", "--ft", "FT-LINK-001"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["cutSetOrder"], 2, "the order that was run is stated: {j}");
}

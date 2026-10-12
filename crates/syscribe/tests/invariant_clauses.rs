//! TC-TRS-INV-001 / REQ-TRS-INV-001 (GH #342): named properties (`Where`), degraded clauses (`Except under`) and the
//! temporal operators `within`, `always`/`from now on` and `never`, in scenarios and in the fault campaign.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// `a` and `b` are the two isolating elements (think relays, open at 0), `c` an alarm. `Slow` opens both 2.2 s later,
/// `Blip` opens them at 1 s, closes `a` again at 1.5 s and opens it for good at 2.2 s, `Trip` opens both at 100 ms.
const FILES: &[(&str, &str)] = &[
    (
        "M",
        "---
type: StateDef
name: M
variables:
  - {name: a, value: \"1\"}
  - {name: b, value: \"1\"}
  - {name: c, value: \"0\"}
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - {target: Slow, accept: Slow}
      - {target: B1, accept: Blip}
      - {target: Tripping, accept: Trip}
  - name: Slow
    transitions:
      - {target: Off, accept: {after: \"2200 ms\"}, assign: \"a := 0; b := 0; c := 1\"}
  - name: B1
    transitions:
      - {target: B2, accept: {after: \"1 s\"}, assign: \"a := 0; b := 0\"}
  - name: B2
    transitions:
      - {target: B3, accept: {after: \"500 ms\"}, assign: \"a := 1\"}
  - name: B3
    transitions:
      - {target: Off, accept: {after: \"700 ms\"}, assign: \"a := 0\"}
  - name: Tripping
    transitions:
      - {target: Off, accept: {after: \"100 ms\"}, assign: \"a := 0; b := 0; c := 1\"}
  - name: Off
---
",
    ),
    ("MP", "---\ntype: PartDef\nname: MP\nexhibitsStates: [M]\n---\nThe protection.\n"),
    ("Sys", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: m, typedBy: MP}\n---\nThe system.\n"),
    ("REQ-SYS-001", "---\ntype: Requirement\nid: REQ-SYS-001\nname: isolation\nstatus: approved\nreqDomain: software\n---\n\nThe system shall isolate.\n"),
    (
        "FMEA-SYS-001",
        "---
type: FMEASheet
id: FMEA-SYS-001
name: \"FMEA of the protection\"
status: draft
entries:
  - {id: FM-SYS-001, ref: MP, failureMode: \"a welded\", injection: [{kind: stuck, variable: a, value: \"1\"}]}
  - {id: FM-SYS-002, ref: MP, failureMode: \"b welded\", injection: [{kind: stuck, variable: b, value: \"1\"}]}
  - {id: FM-SYS-003, ref: MP, failureMode: \"alarm dead\", injection: [{kind: stuck, variable: c, value: \"0\"}]}
---
",
    ),
];

const HEAD: &str = "---\ntype: TestCase\nid: TC-SYS-001\nname: isolation\nstatus: active\ntestLevel: L3\nsubject: Sys\nverifies: [REQ-SYS-001]\n---\n\n";

fn model(gherkin: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-inv-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, c) in FILES {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    std::fs::write(root.join("TC-SYS-001.md"), format!("{HEAD}```gherkin\n{gherkin}```\n")).unwrap();
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

/// Each scenario's (status, message, degraded clauses) from `simulate --json`.
fn verdicts(root: &PathBuf) -> Vec<(String, String, String, String)> {
    let (_, t) = run(root, &["simulate", "TC-SYS-001", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}"));
    j["verdicts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| (v["scenario"].as_str().unwrap().to_string(), v["status"].as_str().unwrap().to_string(), v["message"].as_str().unwrap_or("").to_string(), v["degraded"].to_string()))
        .collect()
}

fn verdict<'a>(v: &'a [(String, String, String, String)], title: &str) -> &'a (String, String, String, String) {
    v.iter().find(|x| x.0 == title).unwrap_or_else(|| panic!("no scenario '{title}': {v:?}"))
}

#[test]
fn within_means_reached_by_the_deadline_and_kept() {
    let root = model(
        "Feature: within

  Scenario: reached at 2.2 s fails within 2 s
    When m receives Slow
    Then within 2 s: m.a == 0 and m.b == 0

  Scenario: reached at 2.2 s passes within 2.5 s
    When m receives Slow
    Then within 2.5 s: m.a == 0 and m.b == 0
    When 5 s elapses

  Scenario: reached at 1 s and lost at 1.5 s fails within 2 s
    When m receives Blip
    Then within 2 s: m.a == 0

  Scenario: lost after the deadline also fails
    When m receives Blip
    Then within 1200 ms: m.a == 0
    When 3 s elapses

  Scenario: a property already true is reached at once
    Then within 0 s: m.c == 0
",
    );
    let v = verdicts(&root);
    let x = verdict(&v, "reached at 2.2 s fails within 2 s");
    assert!(x.1 == "fail" && x.2.contains("is not reached within 2 s (by t = 2 s)"), "{x:?}");
    assert_eq!(verdict(&v, "reached at 2.2 s passes within 2.5 s").1, "pass", "{v:?}");
    let x = verdict(&v, "reached at 1 s and lost at 1.5 s fails within 2 s");
    assert!(x.1 == "fail" && x.2.contains("was reached at t = 1 s but no longer holds at t = 1.5 s"), "{x:?}");
    let x = verdict(&v, "lost after the deadline also fails");
    assert!(x.1 == "fail" && x.2.contains("no longer holds at t = 1.5 s"), "{x:?}");
    assert_eq!(verdict(&v, "a property already true is reached at once").1, "pass", "{v:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn always_and_never_hold_at_every_settled_instant() {
    let root = model(
        "Feature: always

  Scenario: always and from now on hold when nothing violates them
    When m receives Blip
    And 1100 ms elapses
    Then always m.b == 0
    And from now on m.c == 0
    When 3 s elapses
    Then m.a == 0

  Scenario: a transient violation fails never
    When m receives Blip
    And 1100 ms elapses
    Then never m.a == 1
    When 3 s elapses

  Scenario: always holds over the run
    Then always m.c == 0
    When m receives Blip
    And 3 s elapses
",
    );
    let v = verdicts(&root);
    assert_eq!(verdict(&v, "always and from now on hold when nothing violates them").1, "pass", "{v:?}");
    let x = verdict(&v, "a transient violation fails never");
    assert!(x.1 == "fail" && x.2.contains("`m.a == 1` holds at t = 1.5 s, and must never"), "{x:?}");
    assert_eq!(verdict(&v, "always holds over the run").1, "pass", "{v:?}");
    // a violation that recovers before the end still fails `always`
    let root2 = model("Feature: f\n\n  Scenario: transient\n    When m receives Blip\n    And 1100 ms elapses\n    Then always m.a == 0\n    When 3 s elapses\n    Then m.a == 0\n");
    let v = verdicts(&root2);
    let x = verdict(&v, "transient");
    assert!(x.1 == "fail" && x.2.contains("`m.a == 0` does not hold at t = 1.5 s"), "{x:?}");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&root2);
}

const SAFE: &str = "Feature: safe state
  Where the safe state is m.a == 0 and m.b == 0
  Except under FM-SYS-001: m.b == 0
  Except under FM-SYS-002: m.a == 0

";

#[test]
fn degraded_clauses_replace_the_property_only_for_their_faults() {
    let root = model(&format!(
        "{SAFE}  Scenario: no fault, the full property
    When m receives Trip
    Then within 1 s the safe state holds

  Scenario: a welded a uses its clause
    Given fault FM-SYS-001 is injected
    When m receives Trip
    Then within 1 s the safe state holds

  Scenario: a fault no clause names uses the full property
    Given fault FM-SYS-003 is injected
    When m receives Trip
    Then within 1 s the safe state holds

  Scenario: both welded and no clause of their own
    Given fault FM-SYS-001 is injected
    And fault FM-SYS-002 is injected
    When m receives Trip
    Then within 1 s the safe state holds

  Scenario: both welded with a clause of their own
    Given fault FM-SYS-001 is injected
    And fault FM-SYS-002 is injected
    When m receives Trip
    Then within 1 s the safe state holds
    Except under FM-SYS-001 and FM-SYS-002: m.c == 1
"
    ));
    let v = verdicts(&root);
    let x = verdict(&v, "no fault, the full property");
    assert!(x.1 == "pass" && x.3 == "null", "{x:?}");
    let x = verdict(&v, "a welded a uses its clause");
    assert!(x.1 == "pass" && x.3.contains("'the safe state': Except under FM-SYS-001"), "{x:?}");
    let x = verdict(&v, "a fault no clause names uses the full property");
    assert!(x.1 == "pass" && x.3 == "null", "{x:?}");
    let x = verdict(&v, "both welded and no clause of their own");
    assert!(x.1 == "error" && x.2.contains("no applicable degraded clause for 'the safe state'") && x.2.contains("Except under FM-SYS-001 and FM-SYS-002"), "{x:?}");
    let x = verdict(&v, "both welded with a clause of their own");
    assert!(x.1 == "pass" && x.3.contains("Except under FM-SYS-001 and FM-SYS-002"), "{x:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_campaign_reports_a_set_that_survives_only_degraded() {
    let root = model(&format!(
        "{SAFE}  @invariant
  Scenario: a trip reaches the safe state and keeps it
    When m receives Trip
    Then within 1 s the safe state holds
    When 2 s elapses
"
    ));
    let (_, t) = run(&root, &["inject", "TC-SYS-001", "--json", "--report-only"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}"));
    let set = |l: &str| j["sets"].as_array().unwrap().iter().find(|s| s["label"] == l).cloned().unwrap_or_else(|| panic!("{l}: {j}"));
    assert_eq!(set("FM-SYS-001")["status"], "survived");
    assert!(set("FM-SYS-001")["degraded"][0]["message"].as_str().unwrap().contains("Except under FM-SYS-001"), "{j}");
    assert!(set("FM-SYS-003")["degraded"].is_null(), "{j}");
    let (_, text) = run(&root, &["inject", "TC-SYS-001", "--report-only"]);
    assert!(text.contains("SURVIVED (DEGRADED) FM-SYS-001"), "{text}");
    assert!(text.contains("2 only in a degraded safe state"), "{text}");
    // a recorded campaign keeps the clause, and `results campaigns diff` sees a change from full to degraded
    let (c, t) = run(&root, &["inject", "TC-SYS-001", "--record", "--report-only"]);
    assert_eq!(c, 0, "{t}");
    let rec = std::fs::read_to_string(root.join(".syscribe/campaigns.json")).unwrap();
    assert!(rec.contains("Except under FM-SYS-002"), "{rec}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn malformed_clauses_and_unknown_names_are_e886() {
    let root = model(
        "Feature: bad
  Except under FM-SYS-001: m.b == 0
  Where the safe state is m.a ==

  Scenario: names a property nobody declared
    When m receives Trip
    Then within 1 s the isolated state holds

  Scenario: an operator outside Then
    When within 1 s: m.a == 0
",
    );
    let (_, t) = run(&root, &["validate"]);
    assert!(t.contains("E886") && t.contains("belongs to the `Where` before it"), "{t}");
    assert!(t.contains("is not an expression"), "{t}");
    assert!(t.contains("there is no `Where the isolated state is <expression>`"), "{t}");
    assert!(t.contains("`within` is a Then step"), "{t}");
    // an Except naming a fault the model does not have is E937
    let root2 = model(&format!("{}\n  Except under FM-SYS-099: m.a == 0\n\n  Scenario: s\n    When m receives Trip\n    Then within 1 s the safe state holds\n", SAFE.trim_end()));
    let (_, t) = run(&root2, &["validate"]);
    assert!(t.contains("E937") && t.contains("'FM-SYS-099'"), "{t}");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&root2);
}

/// The worked example: the recorded grid-tie campaigns judge the precise safe state of SG-GTI-001 with its degraded
/// forms; a welded relay survives only degraded, and the sensing common cause alone energises a dead grid.
#[test]
fn the_grid_tie_campaigns_judge_the_precise_safe_state() {
    let gti = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_gti");
    let rec: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(gti.join(".syscribe/campaigns.json")).unwrap()).unwrap();
    for key in ["TC-GTI-PROT-US-001@CONF-GTI-US-M3-001", "TC-GTI-PROT-IN-001@CONF-GTI-IN-M3-001"] {
        let sets = rec[key][0]["sets"].as_array().unwrap_or_else(|| panic!("no record {key}"));
        let set = |l: &str| sets.iter().find(|s| s["label"] == l).unwrap_or_else(|| panic!("{key}: no set {l}"));
        assert_eq!(set("FTE-GTI-002")["status"], "survived", "{key}");
        assert!(set("FTE-GTI-002")["degraded"][0]["message"].as_str().unwrap().contains("Except under FTE-GTI-002"), "{key}");
        assert!(set("FTE-GTI-006")["degraded"].is_null(), "{key}: the hardware trip's failure leaves the full safe state");
        assert_eq!(set("FTE-GTI-009")["status"], "defeated", "{key}");
        assert_eq!(set("FTE-GTI-009")["defeats"].as_array().unwrap().len(), 1, "{key}: only the dead-grid invariant");
        assert_eq!(sets.iter().filter(|s| s["status"] == "defeated").count(), 11, "{key}");
    }
}

/// A degraded clause is reported only when it was needed: the full property failed and the clause held. `never`
/// always judges the full property, since a weaker one would make it stricter.
#[test]
fn a_clause_is_degraded_only_when_the_full_property_fails() {
    let root = model(
        "Feature: f
  Where the safe state is m.a == 0 and m.b == 0
  Except under FM-SYS-003: m.a == 0
  Except under FM-SYS-001: m.b == 0

  Scenario: the alarm fault leaves the full safe state
    Given fault FM-SYS-003 is injected
    When m receives Trip
    Then within 1 s the safe state holds

  Scenario: never uses the full property
    Given fault FM-SYS-001 is injected
    Then never the safe state holds
    When m receives Trip
    And 2 s elapses
",
    );
    let v = verdicts(&root);
    let x = verdict(&v, "the alarm fault leaves the full safe state");
    assert!(x.1 == "pass" && x.3 == "null", "{x:?}");
    let x = verdict(&v, "never uses the full property");
    assert!(x.1 == "pass" && x.3 == "null", "{x:?}");
    let (_, text) = run(&root, &["simulate", "TC-SYS-001"]);
    assert!(!text.contains("degraded clause"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A Scenario Outline property is checked once each row fills in its placeholders.
#[test]
fn outline_properties_are_checked_after_expansion() {
    let root = model(
        "Feature: outline

  Scenario Outline: trip to <v>
    Where the alarm state is m.c == <v>
    When m receives Trip
    Then within 1 s the alarm state holds

    Examples:
      | v |
      | 1 |
      | = |
",
    );
    let v = verdicts(&root);
    assert_eq!(verdict(&v, "trip to <v> [row 1]").1, "pass", "{v:?}");
    let (_, t) = run(&root, &["validate"]);
    assert!(t.contains("E886") && t.contains("row 2: `m.c == =` is not an expression"), "{t}");
    assert!(!t.contains("`m.c == <v>` is not an expression"), "{t}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A property after `Rule:` belongs to that rule, never to the scenario before it or to a later rule; a scenario's
/// own `Except under` stays in that scenario.
#[test]
fn properties_are_scoped_to_their_feature_rule_or_scenario() {
    let root = model(
        "Feature: rules
  Where the safe state is m.a == 0 and m.b == 0

  Scenario: before any rule
    When m receives Trip
    Then within 1 s the safe state holds

  Scenario: a welded a with its own clause
    Given fault FM-SYS-001 is injected
    When m receives Trip
    Then within 1 s the safe state holds
    Except under FM-SYS-001: m.b == 0

  Scenario: a welded a without one
    Given fault FM-SYS-001 is injected
    When m receives Trip
    Then within 1 s the safe state holds

  Rule: first
    Where the isolated state is m.a == 0

    Scenario: in the first rule
      When m receives Trip
      Then within 1 s the isolated state holds

  Rule: second

    Scenario: in the second rule
      When m receives Trip
      Then within 1 s the isolated state holds
",
    );
    let v = verdicts(&root);
    assert_eq!(verdict(&v, "before any rule").1, "pass", "{v:?}");
    assert_eq!(verdict(&v, "in the first rule").1, "pass", "{v:?}");
    let x = verdict(&v, "a welded a with its own clause");
    assert!(x.1 == "pass" && x.3.contains("Except under FM-SYS-001"), "{x:?}");
    assert_eq!(verdict(&v, "a welded a without one").1, "fail", "{v:?}");
    let (_, t) = run(&root, &["validate"]);
    assert!(t.contains("scenario 'in the second rule'") && t.contains("there is no `Where the isolated state is <expression>`"), "{t}");
    assert!(!t.contains("scenario 'in the first rule'"), "{t}");
    let _ = std::fs::remove_dir_all(&root);
}

/// Declaring a property twice in one scope is E886; a scenario may still replace a feature's property.
#[test]
fn a_property_named_twice_in_one_scope_is_e886() {
    let root = model(
        "Feature: dup
  Where the safe state is m.a == 0
  Where the safe state is m.b == 0

  Scenario: replaces the feature's property
    Where the safe state is m.c == 1
    When m receives Trip
    Then within 1 s the safe state holds
",
    );
    let (_, t) = run(&root, &["validate"]);
    assert!(t.contains("E886") && t.contains("property 'the safe state' is already declared at line"), "{t}");
    assert_eq!(t.matches("is already declared").count(), 1, "{t}");
    let _ = std::fs::remove_dir_all(&root);
}

/// Properties are judged only in composite scenarios: on a state machine they are E886, not silently ignored.
#[test]
fn properties_on_a_machine_test_case_are_e886() {
    let root = model("Feature: m\n  Where p is a == 0\n\n  Scenario: s\n    Given the machine is in Idle\n    When Trip is injected\n");
    let tc = std::fs::read_to_string(root.join("TC-SYS-001.md")).unwrap().replace("subject: Sys", "subject: M");
    std::fs::write(root.join("TC-SYS-001.md"), tc).unwrap();
    let (_, t) = run(&root, &["validate"]);
    assert!(t.contains("E886") && t.contains("properties are for composite scenarios"), "{t}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A scenario-level `Except under` is part of what a recorded campaign judged: changing it changes the invariants.
#[test]
fn a_scenario_clause_changes_the_recorded_invariants() {
    let scenario = "{SAFE}  @invariant\n  Scenario: trip\n    When m receives Trip\n    Then within 1 s the safe state holds\n";
    let root = model(&scenario.replace("{SAFE}", SAFE));
    let (c, t) = run(&root, &["inject", "TC-SYS-001", "--record", "--run", "a", "--report-only"]);
    assert_eq!(c, 0, "{t}");
    let tc = root.join("TC-SYS-001.md");
    let body = std::fs::read_to_string(&tc).unwrap().replace("holds\n```", "holds\n    Except under FM-SYS-001 and FM-SYS-002: m.c == 1\n```");
    std::fs::write(&tc, body).unwrap();
    let (c, t) = run(&root, &["inject", "TC-SYS-001", "--record", "--run", "b", "--report-only"]);
    assert_eq!(c, 0, "{t}");
    let (_, t) = run(&root, &["results", "campaigns", "diff", "TC-SYS-001"]);
    assert!(t.contains("the invariants changed"), "{t}");
    let _ = std::fs::remove_dir_all(&root);
}

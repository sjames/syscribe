//! TC-TRS-RISK-001 / REQ-TRS-RISK-001 (GH #341): fault-injection campaigns recorded as evidence, and every defeat tied to
//! a recorded `ResidualRisk` decision.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const FMEA: &str = "---
type: FMEASheet
id: FMEA-LINK-001
name: \"FMEA of the link\"
status: draft
entries:
  - {id: FM-TX-001, ref: TxP, failureMode: \"command lost\", injection: [{kind: drop, port: out}]}
  - {id: FM-TX-003, ref: TxP, failureMode: \"level stuck\", injection: [{kind: stuck, variable: level, value: \"3\"}]}
  - {id: FM-TX-004, ref: TxP, failureMode: \"level biased\", injection: [{kind: offset, variable: level, by: \"2\"}]}
---
";

const FILES: &[(&str, &str)] = &[
    ("TxM", "---\ntype: StateDef\nname: TxM\nvariables:\n  - {name: level, value: \"7\"}\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {target: Idle, accept: Tick, effect: {send: Cmd, via: out, payload: {n: \"level\"}}}\n---\n"),
    ("RxM", "---\ntype: StateDef\nname: RxM\nvariables:\n  - {name: got, value: \"0\"}\nsubStates:\n  - name: Waiting\n    isInitial: true\n    transitions:\n      - {target: Done, accept: {payload: Cmd, via: inp, as: m}, assign: \"got := m.n\"}\n  - {name: Done, isFinal: true}\n---\n"),
    ("TxP", "---\ntype: PartDef\nname: TxP\nexhibitsStates: [TxM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\nTransmitter.\n"),
    ("RxP", "---\ntype: PartDef\nname: RxP\nexhibitsStates: [RxM]\nfeatures:\n  - {name: inp, type: Port, direction: in}\n---\nReceiver.\n"),
    ("Sys", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\nconnections:\n  - {from: tx.out, to: rx.inp}\n---\nThe system.\n"),
    ("REQ-SYS-001", "---\ntype: Requirement\nid: REQ-SYS-001\nname: delivery\nstatus: approved\nreqDomain: software\n---\n\nThe receiver shall get the command.\n"),
    ("SG-SYS-001", "---\ntype: SafetyGoal\nid: SG-SYS-001\nname: No wrong command is acted on\nstatus: draft\nasilLevel: B\n---\nGoal.\n"),
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

  @invariant
  Scenario: the receiver never accepts a wrong value
    When tx receives Tick
    Then rx.got == 0 or rx.got == 7
```
";

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-risk-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, c) in FILES {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    std::fs::write(root.join("TC-SYS-001.md"), TC).unwrap();
    root
}

fn run(root: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

fn risk(root: &Path, id: &str, body: &str) {
    std::fs::write(root.join(format!("{id}.md")), format!("---\ntype: ResidualRisk\nid: {id}\nname: \"{id}\"\n{body}---\nThe decision.\n")).unwrap();
}

/// The validate findings of one code, as their message lines.
fn findings(root: &Path, code: &str) -> Vec<String> {
    let (_, t) = run(root, &["validate"]);
    t.lines().filter(|l| l.contains(code)).map(str::to_string).collect()
}

fn records(root: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root.join(".syscribe/campaigns.json")).unwrap()).unwrap()
}

fn without_time(mut v: serde_json::Value) -> serde_json::Value {
    for list in v.as_object_mut().unwrap().values_mut() {
        for r in list.as_array_mut().unwrap() {
            r.as_object_mut().unwrap().remove("recordedAtUnix");
        }
    }
    v
}

fn composite_hash(root: &Path) -> String {
    records(root)["TC-SYS-001"][0]["compositeHash"].as_str().unwrap().to_string()
}

#[test]
fn a_recorded_campaign_is_listed_and_a_second_run_records_the_same() {
    let root = model();
    let (c, t) = run(&root, &["inject", "TC-SYS-001", "--record", "--report-only"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("recorded the campaign of TC-SYS-001"), "{t}");
    let first = records(&root);
    let rec = &first["TC-SYS-001"][0];
    assert_eq!(rec["composite"], "Sys");
    assert_eq!(rec["invariants"][0]["scenario"], "the receiver never accepts a wrong value");
    let sets = rec["sets"].as_array().unwrap();
    assert_eq!(sets.iter().filter(|s| s["status"] == "defeated").count(), 2, "{rec}");
    assert!(sets.iter().any(|s| s["label"] == "FM-TX-003" && s["defeats"][0]["scenario"] == "the receiver never accepts a wrong value"), "{rec}");
    assert_eq!(rec["engine"], env!("CARGO_PKG_VERSION"));
    let (c, t) = run(&root, &["results", "campaigns"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("TC-SYS-001") && t.contains("3 set(s): 1 survived, 2 defeated"), "{t}");
    // same model, same record: the unnamed run replaces the earlier unnamed one
    run(&root, &["inject", "TC-SYS-001", "--record", "--report-only"]);
    let second = records(&root);
    assert_eq!(second["TC-SYS-001"].as_array().unwrap().len(), 1);
    assert_eq!(without_time(first), without_time(second));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_unexplained_defeat_is_a_warning_until_a_decision_names_it() {
    let root = model();
    // nothing recorded: dormant
    assert!(findings(&root, "W940").is_empty());
    run(&root, &["inject", "TC-SYS-001", "--record", "--report-only"]);
    let w = findings(&root, "W940");
    assert_eq!(w.len(), 2, "{w:?}");
    assert!(w.iter().any(|l| l.contains("FM-TX-003") && l.contains("never accepts a wrong value")), "{w:?}");
    risk(&root, "RISK-SYS-001", "status: accepted\nfaultSet: [FM-TX-003]\ngoal: SG-SYS-001\napprovedBy: jane\n");
    let w = findings(&root, "W940");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("FM-TX-004"), "{w:?}");
    // a rejected decision does not explain the defeat, and says so
    risk(&root, "RISK-SYS-002", "status: rejected\nfaultSet: [FMEA-LINK-001::FM-TX-004]\n");
    let w = findings(&root, "W940");
    assert!(w.len() == 1 && w[0].contains("RISK-SYS-002 rejected it"), "{w:?}");
    // gate-able
    let (c, _) = run(&root, &["validate", "--deny", "W940"]);
    assert_ne!(c, 0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_decision_the_campaign_contradicts_is_flagged() {
    let root = model();
    run(&root, &["inject", "TC-SYS-001", "--record", "--report-only"]);
    risk(&root, "RISK-SYS-001", "status: accepted\nfaultSet: [FM-TX-001]\napprovedBy: jane\n");
    risk(&root, "RISK-SYS-002", "status: mitigated\nfaultSet: [FM-TX-004]\nmitigatedBy: [REQ-SYS-001]\n");
    let w = findings(&root, "W941");
    assert!(w.iter().any(|l| l.contains("RISK-SYS-001") && l.contains("now survives")), "{w:?}");
    assert!(w.iter().any(|l| l.contains("RISK-SYS-002") && l.contains("still defeats")), "{w:?}");
    // a mitigated decision is not also an unexplained defeat
    assert!(findings(&root, "W940").iter().all(|l| !l.contains("FM-TX-004")));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn changing_the_composite_makes_the_decision_and_the_record_stale() {
    let root = model();
    run(&root, &["inject", "TC-SYS-001", "--record", "--report-only"]);
    let h = composite_hash(&root);
    risk(&root, "RISK-SYS-001", &format!("status: accepted\nfaultSet: [FM-TX-003]\napprovedBy: jane\ncampaignHash: {}\n", &h[..12]));
    assert!(findings(&root, "W942").is_empty() && findings(&root, "W944").is_empty());
    let tx = FILES[0].1.replace("value: \"7\"", "value: \"0\"");
    std::fs::write(root.join("TxM.md"), tx).unwrap();
    let w = findings(&root, "W942");
    assert!(w.len() == 1 && w[0].contains(&h[..12]), "{w:?}");
    let w = findings(&root, "W944");
    assert!(w.len() == 1 && w[0].contains("inject TC-SYS-001 --record"), "{w:?}");
    // recording again settles the campaign; the decision stays stale until it is reviewed
    run(&root, &["inject", "TC-SYS-001", "--record", "--report-only"]);
    assert!(findings(&root, "W944").is_empty());
    assert_eq!(findings(&root, "W942").len(), 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_malformed_or_expired_decision() {
    let root = model();
    risk(&root, "RISK-SYS-001", "status: accepted\nfaultSet: [REQ-SYS-001]\ngoal: REQ-SYS-001\nexpires: 2020-01-01\ncampaignHash: xyz\n");
    risk(&root, "RISK-SYS-002", "status: open\n");
    let e = findings(&root, "E942");
    for want in ["approvedBy", "campaignHash", "faultSet` is required", "must be one of"] {
        assert!(e.iter().any(|l| l.contains(want)), "{want}: {e:?}");
    }
    let e = findings(&root, "E944");
    assert!(e.iter().any(|l| l.contains("faultSet entry") && l.contains("Requirement")), "{e:?}");
    assert!(e.iter().any(|l| l.contains("goal") && l.contains("SafetyGoal")), "{e:?}");
    assert_eq!(findings(&root, "W943").len(), 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn named_runs_are_kept_and_compared() {
    let root = model();
    run(&root, &["inject", "TC-SYS-001", "--record", "--run", "a", "--report-only"]);
    // the invariant now tolerates the bias: FM-TX-004 survives
    std::fs::write(root.join("TC-SYS-001.md"), TC.replace("rx.got == 0 or rx.got == 7", "rx.got == 0 or rx.got == 7 or rx.got == 9")).unwrap();
    run(&root, &["inject", "TC-SYS-001", "--record", "--run", "b", "--report-only"]);
    assert_eq!(records(&root)["TC-SYS-001"].as_array().unwrap().len(), 2);
    let (c, t) = run(&root, &["results", "campaigns", "diff", "TC-SYS-001"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("a → b") && t.contains("FM-TX-004: defeated → survived") && t.contains("the invariants changed"), "{t}");
    let (c, t) = run(&root, &["results", "campaigns", "diff", "TC-SYS-001", "b", "a", "--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["diff"]["changed"][0], serde_json::json!(["FM-TX-004", "survived", "defeated"]));
    let (c, _) = run(&root, &["results", "campaigns", "diff", "TC-SYS-001", "a", "zz"]);
    assert_eq!(c, 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_report_shows_each_defeat_with_its_decision() {
    let root = model();
    risk(&root, "RISK-SYS-001", "status: accepted\nfaultSet: [FM-TX-003]\napprovedBy: jane\n");
    let md = root.join("campaign.md");
    let (c, t) = run(&root, &["inject", "TC-SYS-001", "--report-only", "--report", md.to_str().unwrap()]);
    assert_eq!(c, 0, "{t}");
    let text = std::fs::read_to_string(&md).unwrap();
    assert!(text.contains("# Fault-injection campaign: TC-SYS-001"), "{text}");
    assert!(text.lines().any(|l| l.starts_with("| FM-TX-003 ") && l.contains("RISK-SYS-001 (accepted)")), "{text}");
    assert!(text.lines().any(|l| l.starts_with("| FM-TX-004 ") && l.contains("**none**")), "{text}");
    let js = root.join("campaign.json");
    run(&root, &["inject", "TC-SYS-001", "--report-only", "--report", js.to_str().unwrap()]);
    let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&js).unwrap()).unwrap();
    let s = j["sets"].as_array().unwrap().iter().find(|s| s["label"] == "FM-TX-003").unwrap();
    assert_eq!(s["decisions"][0]["id"], "RISK-SYS-001");
    assert!(!root.join(".syscribe/campaigns.json").exists(), "a report alone records nothing");
    let (c, t) = run(&root, &["inject", "TC-SYS-001", "--record", "--config", "REQ-SYS-001"]);
    assert_ne!(c, 0, "{t}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_safety_case_and_audit_show_the_campaign() {
    let root = model();
    run(&root, &["inject", "TC-SYS-001", "--record", "--report-only"]);
    risk(&root, "RISK-SYS-001", "status: accepted\nfaultSet: [FM-TX-003]\ngoal: SG-SYS-001\napprovedBy: jane\n");
    let (_, t) = run(&root, &["safety-case"]);
    assert!(t.contains("[solution] campaign:TC-SYS-001 — fault-injection campaign TC-SYS-001: 3 sets, 1 survived, 2 defeated (1 explained)"), "{t}");
    assert!(t.contains("RISK-SYS-001") && t.contains("UNEXPLAINED: FM-TX-004"), "{t}");
    let (_, t) = run(&root, &["safety-case", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["campaigns"][0]["unexplained"], serde_json::json!(["FM-TX-004"]));
    let (_, t) = run(&root, &["audit"]);
    assert!(t.contains("1 unexplained defeat(s) (W940)") && t.contains("FM-TX-004"), "{t}");
    let _ = std::fs::remove_dir_all(&root);
}

/// The worked example: the grid-tie inverter's recorded campaigns are current, and every defeated set has a decision.
#[test]
fn the_grid_tie_campaigns_are_current_and_every_defeat_is_decided() {
    let gti = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_gti");
    let j = records(&gti);
    for key in ["TC-GTI-PROT-US-001@CONF-GTI-US-M3-001", "TC-GTI-PROT-IN-001@CONF-GTI-IN-M3-001"] {
        assert!(j.to_string().contains(key), "no recorded campaign {key}");
    }
    for conf in ["CONF-GTI-US-M3-001", "CONF-GTI-IN-M3-001"] {
        let (c, t) = run(&gti, &["validate", "--config", conf, "--deny", "W940", "--deny", "W941", "--deny", "W944", "--deny", "W946"]);
        assert!(c == 0 && !t.contains("W94"), "{conf}: {t}");
    }
    // not vacuous: the campaigns do defeat sets, each one recorded with the invocation that ran it
    for list in j.as_object().unwrap().values() {
        let rec = &list[0];
        assert_eq!(rec["invocation"]["ft"], "FT-GTI-001", "{rec}");
        let defeated: Vec<&serde_json::Value> = rec["sets"].as_array().unwrap().iter().filter(|s| s["status"] == "defeated").collect();
        assert_eq!(defeated.len(), 10, "{rec}");
    }
    assert_eq!(std::fs::read_dir(gti.join("Safety/ResidualRisk")).unwrap().filter(|e| e.as_ref().unwrap().path().to_string_lossy().contains("RISK-")).count(), 10);
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &p);
        } else {
            std::fs::copy(e.path(), p).unwrap();
        }
    }
}

/// A private copy of the worked example, so a test may edit it.
fn gti_copy() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-risk-gti-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&root);
    copy_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_gti"), &root);
    root
}

const US: &str = "CONF-GTI-US-M3-001";

fn validate_codes(root: &Path, conf: &str, code: &str) -> Vec<String> {
    let (_, t) = run(root, &["validate", "--config", conf]);
    t.lines().filter(|l| l.contains(code)).map(str::to_string).collect()
}

#[test]
fn a_corrupt_record_file_is_reported_and_never_overwritten() {
    let root = model();
    std::fs::create_dir_all(root.join(".syscribe")).unwrap();
    std::fs::write(root.join(".syscribe/campaigns.json"), "{ not json").unwrap();
    let w = findings(&root, "W945");
    assert_eq!(w.len(), 1, "{w:?}");
    let (c, t) = run(&root, &["inject", "TC-SYS-001", "--record", "--report-only"]);
    assert_ne!(c, 0, "{t}");
    assert_eq!(std::fs::read_to_string(root.join(".syscribe/campaigns.json")).unwrap(), "{ not json");
    let (c, t) = run(&root, &["results", "campaigns"]);
    assert_eq!(c, 1, "{t}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_safety_case_agrees_with_the_validator_on_what_is_explained() {
    let root = model();
    run(&root, &["inject", "TC-SYS-001", "--record", "--report-only"]);
    // a mitigated decision on a set that still defeats is contradicted (W941), not explained
    risk(&root, "RISK-SYS-001", "status: mitigated\nfaultSet: [FM-TX-003]\ngoal: SG-SYS-001\nmitigatedBy: [REQ-SYS-001]\n");
    // a misspelt status explains nothing (E942) and the defeat stays unexplained (W940)
    risk(&root, "RISK-SYS-002", "status: acceptd\nfaultSet: [FM-TX-004]\ngoal: SG-SYS-001\n");
    assert!(findings(&root, "W941").iter().any(|l| l.contains("RISK-SYS-001")));
    assert!(findings(&root, "W940").iter().any(|l| l.contains("FM-TX-004")));
    let (_, t) = run(&root, &["safety-case"]);
    assert!(t.contains("2 defeated (0 explained)") && t.contains("[UNDEVELOPED]"), "{t}");
    assert!(t.contains("CONTRADICTED: FM-TX-003") && t.contains("UNEXPLAINED: FM-TX-004"), "{t}");
    // once every defeat has an accepted decision the node is developed
    risk(&root, "RISK-SYS-001", "status: accepted\nfaultSet: [FM-TX-003]\ngoal: SG-SYS-001\napprovedBy: jane\n");
    risk(&root, "RISK-SYS-002", "status: accepted\nfaultSet: [FM-TX-004]\ngoal: SG-SYS-001\napprovedBy: jane\n");
    let (_, t) = run(&root, &["safety-case"]);
    let line = t.lines().find(|l| l.contains("campaign:TC-SYS-001")).unwrap_or_default().to_string();
    assert!(line.contains("(2 explained)") && !line.contains("UNDEVELOPED"), "{t}");
    let _ = std::fs::remove_dir_all(&root);
}

/// Findings 1, 3, 5 and 6 of the review, on the worked example: a record judged under its configuration goes stale
/// when a machine changes, a narrower re-record warns and leaves decisions unchecked, the campaign shows in the safety
/// case before any decision exists, and the un-configured audit points at the configured records.
#[test]
fn the_grid_tie_records_are_judged_under_their_configuration() {
    let root = gti_copy();
    assert!(validate_codes(&root, US, "W944").is_empty());
    let (_, t) = run(&root, &["audit"]);
    assert!(t.contains(&format!("campaigns recorded on {US}: 0 defeat(s) no decision explains")), "{t}");

    // no decisions at all: the campaign still hangs under the fault tree's goal, undeveloped, and the audit counts it
    let risks = root.join("Safety/ResidualRisk");
    let aside = root.with_extension("risks");
    std::fs::rename(&risks, &aside).unwrap();
    let (_, t) = run(&root, &["safety-case", "--config", US]);
    let line = t.lines().find(|l| l.contains("campaign:TC-GTI-PROT-US-001")).unwrap_or_default().to_string();
    assert!(line.contains("10 defeated (0 explained)") && line.contains("[UNDEVELOPED]"), "{t}");
    let (_, t) = run(&root, &["audit"]);
    assert!(t.contains(&format!("campaigns recorded on {US}: 10 defeat(s) no decision explains")), "{t}");
    std::fs::rename(&aside, &risks).unwrap();

    // a narrower re-record: the order-2 and order-3 sets the decisions name are no longer run
    let (_, t) = run(&root, &["inject", "TC-GTI-PROT-US-001", "--ft", "FT-GTI-001", "--max-order", "1", "--config", US, "--record", "--report-only"]);
    assert!(t.contains("--max-order 3"), "the warning names the earlier command: {t}");
    let w = validate_codes(&root, US, "W946");
    assert!(w.len() >= 10 && w.iter().any(|l| l.contains("RISK-GTI-010")), "{w:?}");

    // a machine edit makes the configured record stale, and the message prints the command to re-run
    let p = root.join("Behavior/Machines/PerturbationCheck.md");
    std::fs::write(&p, std::fs::read_to_string(&p).unwrap().replace("iInj := 0.2 SI::A", "iInj := 0.3 SI::A")).unwrap();
    let w = validate_codes(&root, US, "W944");
    assert!(w.len() == 1 && w[0].contains("--config CONF-GTI-US-M3-001 --record"), "{w:?}");

    let _ = std::fs::remove_dir_all(&root);
}

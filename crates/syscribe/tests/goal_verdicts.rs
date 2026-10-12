//! TC-TRS-GOALV-001 / REQ-TRS-GOALV-001 (GH #343): a verdict per safety goal from its fault tree (rate-only PFH,
//! PFDavg in demand mode, configurable and per-goal targets, rate provenance, plausibility), and the fault tree compared
//! with the executable campaign.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn dir(tag: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-goalv-{tag}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("FT-SYS-001")).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    root
}

fn write(root: &Path, rel: &str, text: &str) {
    std::fs::write(root.join(rel), text).unwrap();
}

/// A SIL 2 goal over `gate` of two events (λ 1e-7 and 2e-7 /h) and a 10 000 h mission.
fn model(gate: &str, goal_extra: &str, event_extra: &str) -> PathBuf {
    let root = dir("v");
    write(&root, "SG-SYS-001.md", &format!("---\ntype: SafetyGoal\nid: SG-SYS-001\nname: No unintended output\nstatus: draft\nsilLevel: 2\n{goal_extra}---\nGoal.\n"));
    write(&root, "FT-SYS-001.md", "---\ntype: FaultTree\nid: FT-SYS-001\nname: Unintended output\ntopEvent: SG-SYS-001\nstatus: draft\nmissionTime: \"10000 h\"\n---\nTree.\n");
    write(&root, "FT-SYS-001/FTG-SYS-001.md", &format!("---\ntype: FaultTreeGate\nid: FTG-SYS-001\nname: Output\ngateType: {gate}\ninputs: [FTE-SYS-001, FTE-SYS-002]\n---\nGate.\n"));
    write(&root, "FT-SYS-001/FTE-SYS-001.md", &format!("---\ntype: FaultTreeEvent\nid: FTE-SYS-001\nname: Channel A fails\neventKind: basic\nfailureRate: 1.0e-7\n{event_extra}---\nA.\n"));
    write(&root, "FT-SYS-001/FTE-SYS-002.md", &format!("---\ntype: FaultTreeEvent\nid: FTE-SYS-002\nname: Channel B fails\neventKind: basic\nfailureRate: 2.0e-7\n{event_extra}---\nB.\n"));
    root
}

fn run(root: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

/// The JSON of `metrics` (stdout only: the exit summary goes to stderr).
fn metrics_json(root: &Path, extra: &[&str]) -> serde_json::Value {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).arg("metrics").arg("--json").args(extra).output().unwrap();
    let t = String::from_utf8_lossy(&o.stdout);
    serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}"))
}

fn goal(root: &Path) -> serde_json::Value {
    let j = metrics_json(root, &[]);
    j.as_array().unwrap().iter().find(|g| g["id"] == "SG-SYS-001").cloned().unwrap()
}

fn codes(root: &Path, code: &str) -> Vec<String> {
    let (_, t) = run(root, &["validate"]);
    t.lines().filter(|l| l.contains(code)).map(str::to_string).collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-4 * b.abs()
}

#[test]
fn a_goal_without_coverage_data_gets_a_rate_only_verdict_from_the_exact_top_event() {
    let root = model("OR", "", "");
    let g = goal(&root);
    // P(top) = 1 − e^(−3e-7 · 1e4) = 2.99550e-3 over 10 000 h: PFH 2.99550e-7 /h, below SIL 2's 1e-6
    let pfh = (1.0 - (-3.0e-3f64).exp()) / 1.0e4;
    assert!(close(g["figure"]["value"].as_f64().unwrap(), pfh), "{g}");
    assert_eq!(g["figure"]["metric"], "PFH");
    assert_eq!(g["figure"]["basis"], "rate only (top event / mission time)");
    assert_eq!(g["verdict"], "pass");
    assert_eq!(g["target"]["source"], "built-in default");
    assert_eq!(g["rateOnly"], true);
    // no rateStatus: the verdict rests on unverified rates and says so
    assert_eq!(g["unverifiedRates"], serde_json::json!(["FTE-SYS-001", "FTE-SYS-002"]));
    let (c, t) = run(&root, &["metrics"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("pass (unverified rates)"), "{t}");
    // a rate-only figure does not raise W033 unless the goal declares its own target
    assert!(codes(&root, "W033").is_empty());
    let root = model("OR", "", "rateStatus: verified\nrateSource: \"vendor data sheet rev C\"\n");
    assert_eq!(goal(&root)["unverifiedRates"], serde_json::json!([]));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_goal_target_and_the_config_table_override_the_built_in_defaults() {
    // the goal's own target, below the figure: fail, and W033 since the goal opted in
    let root = model("OR", "target: {mode: continuous, value: 1.0e-7, unit: /h}\n", "");
    let g = goal(&root);
    assert_eq!(g["verdict"], "fail");
    assert_eq!(g["target"]["source"], "goal target");
    assert_eq!(g["target"]["value"], 1.0e-7);
    let (c, _) = run(&root, &["metrics"]);
    assert_eq!(c, 2);
    let w = codes(&root, "W033");
    assert!(w.len() == 1 && w[0].contains("PFH") && w[0].contains("1e-7"), "{w:?}");
    // a `.syscribe.toml` table over the built-ins
    let root = model("OR", "", "");
    write(&root, ".syscribe.toml", "[reliability.targets.sil.2]\npfh = 1.0e-7\n");
    let g = goal(&root);
    assert_eq!(g["verdict"], "fail");
    assert_eq!(g["target"]["source"], ".syscribe.toml");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn demand_mode_judges_the_average_probability_of_failure_on_demand() {
    let demand = "target: {mode: demand, proofTestInterval: \"8760 h\"}\n";
    // OR: each event alone, PFDavg = Σ λ·TI/2 = 3e-7 · 8760 / 2 = 1.314e-3, below SIL 2's 1e-2
    let g = goal(&model("OR", demand, ""));
    assert_eq!(g["figure"]["metric"], "PFDavg");
    assert!(close(g["figure"]["value"].as_f64().unwrap(), 1.314e-3), "{g}");
    assert_eq!(g["verdict"], "pass");
    assert_eq!(g["target"]["value"], 1.0e-2);
    // AND: the two together, PFDavg = λ1·λ2·TI²/3 = 2e-14 · 8760² / 3 = 5.11584e-7
    let g = goal(&model("AND", demand, ""));
    assert!(close(g["figure"]["value"].as_f64().unwrap(), 2.0e-14 * 8760.0f64.powi(2) / 3.0), "{g}");
    // diagnostic coverage leaves only the undetected part: λ_DU = λ · (1 − DC)
    let g = goal(&model("OR", demand, "diagnosticCoverage: 0.9\n"));
    assert!(close(g["figure"]["value"].as_f64().unwrap(), 1.314e-4), "{g}");
    // the same OR tree in continuous mode is a frequency, not a probability
    let g = goal(&model("OR", "", ""));
    assert!(close(g["figure"]["value"].as_f64().unwrap(), (1.0 - (-3.0e-3f64).exp()) / 1.0e4), "{g}");
}

#[test]
fn goals_with_coverage_data_keep_their_first_order_figures() {
    let m = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../qual/fixtures/TC-TRS-SAFE-005/metrics");
    let j = metrics_json(&m, &[]);
    let g = j.as_array().unwrap().iter().find(|g| g["id"] == "SG-MET-001").unwrap();
    assert!(close(g["pmhf"].as_f64().unwrap(), 1.1e-8) && close(g["spfm"].as_f64().unwrap(), 0.945), "{g}");
    assert_eq!(g["verdict"], "fail");
    assert_eq!(g["figure"]["basis"], "first-order (diagnostic coverage)");
}

#[test]
fn an_implausible_rate_makes_the_figure_not_credible() {
    // 1e-4 /h over 10 000 h: the event fails with probability 0.63
    let root = model("OR", "", "");
    let p = root.join("FT-SYS-001/FTE-SYS-001.md");
    std::fs::write(&p, std::fs::read_to_string(&p).unwrap().replace("1.0e-7", "1.0e-4")).unwrap();
    let g = goal(&root);
    assert_eq!(g["verdict"], "not credible", "{g}");
    assert!(g["notCredible"].as_array().unwrap().iter().any(|n| n["event"] == "FTE-SYS-001"), "{g}");
    let (c, t) = run(&root, &["metrics"]);
    assert_eq!(c, 0, "a figure that is not credible is not a failure: {t}");
    assert!(t.contains("not credible") && t.contains("FTE-SYS-001 0.63"), "{t}");
    // an unverified rate is labelled in the verdict; a claimed one is a warning
    assert!(codes(&root, "W988").is_empty());
    std::fs::write(&p, std::fs::read_to_string(&p).unwrap().replace("failureRate: 1.0e-4\n", "failureRate: 1.0e-4\nrateStatus: verified\n")).unwrap();
    let w = codes(&root, "W988");
    assert!(w.len() == 1 && w[0].contains("FTE-SYS-001") && w[0].contains("0.63"), "{w:?}");
    // the bound is the user's: raised, the figure is credible again (and fails SIL 2)
    write(&root, ".syscribe.toml", "[reliability]\nplausibleProbability = 0.9\n");
    assert!(codes(&root, "W988").is_empty());
    assert_eq!(goal(&root)["verdict"], "fail");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn malformed_targets_statuses_and_tables_are_reported() {
    let root = model("OR", "target: {mode: sometimes, value: -1, unit: furlongs}\n", "rateStatus: maybe\n");
    write(&root, ".syscribe.toml", "[reliability.targets.sil.9]\npfh = 1.0e-7\n");
    let e = codes(&root, "E985");
    for want in ["`target.mode`", "`target.value`", "`target.unit`"] {
        assert!(e.iter().any(|l| l.contains(want)), "{want}: {e:?}");
    }
    assert_eq!(codes(&root, "E986").len(), 2);
    assert!(codes(&root, "W989").iter().any(|l| l.contains("sil.9")));
    let root = model("OR", "target: {mode: demand}\n", "");
    assert!(codes(&root, "E985").iter().any(|l| l.contains("proofTestInterval")));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn sensitivity_names_the_events_whose_rate_moves_the_verdict() {
    let root = model("OR", "", "");
    let (_, t) = run(&root, &["metrics", "--sensitivity"]);
    // ×10 on λ = 2e-7 gives a PFH of about 2.1e-6, above SIL 2's 1e-6
    assert!(t.lines().any(|l| l.starts_with("| FTE-SYS-002 (moves the verdict)") && l.ends_with("| pass | fail |")), "{t}");
    let g = metrics_json(&root, &["--sensitivity"]);
    assert_eq!(g[0]["sensitivity"][0]["event"], "FTE-SYS-002");
    let _ = std::fs::remove_dir_all(&root);
}

/// The link from the residual-risk tests, with a fault tree: FTE-SYS-001 (level stuck at 3) is a cut set the model
/// confirms, FTE-SYS-002 (a dropped command) one it survives, and FM-TX-004 (a level bias) defeats the invariant
/// though the tree does not have it.
fn campaign_model() -> PathBuf {
    let root = dir("c");
    let files: &[(&str, &str)] = &[
        ("TxM.md", "---\ntype: StateDef\nname: TxM\nvariables:\n  - {name: level, value: \"7\"}\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {target: Idle, accept: Tick, effect: {send: Cmd, via: out, payload: {n: \"level\"}}}\n---\n"),
        ("RxM.md", "---\ntype: StateDef\nname: RxM\nvariables:\n  - {name: got, value: \"0\"}\nsubStates:\n  - name: Waiting\n    isInitial: true\n    transitions:\n      - {target: Done, accept: {payload: Cmd, via: inp, as: m}, assign: \"got := m.n\"}\n  - {name: Done, isFinal: true}\n---\n"),
        ("TxP.md", "---\ntype: PartDef\nname: TxP\nexhibitsStates: [TxM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\nTransmitter.\n"),
        ("RxP.md", "---\ntype: PartDef\nname: RxP\nexhibitsStates: [RxM]\nfeatures:\n  - {name: inp, type: Port, direction: in}\n---\nReceiver.\n"),
        ("Sys.md", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\nconnections:\n  - {from: tx.out, to: rx.inp}\n---\nThe system.\n"),
        ("REQ-SYS-001.md", "---\ntype: Requirement\nid: REQ-SYS-001\nname: delivery\nstatus: approved\nreqDomain: software\n---\n\nThe receiver shall get the command.\n"),
        ("SG-SYS-001.md", "---\ntype: SafetyGoal\nid: SG-SYS-001\nname: No wrong command is acted on\nstatus: draft\nasilLevel: B\n---\nGoal.\n"),
        ("FMEA-LINK-001.md", "---\ntype: FMEASheet\nid: FMEA-LINK-001\nname: \"FMEA of the link\"\nstatus: draft\nentries:\n  - {id: FM-TX-003, ref: TxP, failureMode: \"level stuck\", ftaRef: FTE-SYS-001, injection: [{kind: stuck, variable: level, value: \"3\"}]}\n  - {id: FM-TX-004, ref: TxP, failureMode: \"level biased\", injection: [{kind: offset, variable: level, by: \"2\"}]}\n---\n"),
        ("FT-SYS-001.md", "---\ntype: FaultTree\nid: FT-SYS-001\nname: Wrong command\ntopEvent: SG-SYS-001\nstatus: draft\nmissionTime: \"10000 h\"\n---\nTree.\n"),
        ("FT-SYS-001/FTG-SYS-001.md", "---\ntype: FaultTreeGate\nid: FTG-SYS-001\nname: Wrong command\ngateType: OR\ninputs: [FTE-SYS-001, FTE-SYS-002]\n---\nGate.\n"),
        ("FT-SYS-001/FTE-SYS-001.md", "---\ntype: FaultTreeEvent\nid: FTE-SYS-001\nname: Level stuck\neventKind: basic\nref: TxP\nfailureRate: 1.0e-7\ninjection: [{kind: stuck, variable: level, value: \"3\"}]\n---\nStuck.\n"),
        ("FT-SYS-001/FTE-SYS-002.md", "---\ntype: FaultTreeEvent\nid: FTE-SYS-002\nname: Command lost\neventKind: basic\nref: TxP\nfailureRate: 1.0e-7\ninjection: [{kind: drop, port: out}]\n---\nLost.\n"),
        ("TC-SYS-001.md", "---\ntype: TestCase\nid: TC-SYS-001\nname: link invariants\nstatus: active\ntestLevel: L3\nsubject: Sys\nverifies: [REQ-SYS-001]\n---\n\n```gherkin\nFeature: link\n\n  @invariant\n  Scenario: the receiver never accepts a wrong value\n    When tx receives Tick\n    Then rx.got == 0 or rx.got == 7\n```\n"),
    ];
    for (n, c) in files {
        write(&root, n, c);
    }
    root
}

#[test]
fn the_campaign_finds_a_cut_set_missing_from_the_tree_and_a_conservative_one() {
    let root = campaign_model();
    let (c, t) = run(&root, &["inject", "TC-SYS-001", "--ft", "FT-SYS-001", "--consistency", "--json"]);
    assert_eq!(c, 2, "a defeat exits 2: {t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}"));
    let rows = j["consistency"]["rows"].as_array().unwrap();
    let row = |f: &str| rows.iter().find(|r| r["faults"] == serde_json::json!([f])).unwrap_or_else(|| panic!("{f}: {rows:?}"));
    assert_eq!(row("FTE-SYS-001")["finding"], "confirmed");
    assert!(row("FTE-SYS-001")["treeProbability"].as_f64().is_some());
    assert_eq!(row("FTE-SYS-002")["finding"], "conservative or a model gap");
    assert_eq!(row("FTE-SYS-002")["verdict"], "survived");
    assert_eq!(row("FM-TX-004")["finding"], "missing from the tree");
    assert!(row("FM-TX-004")["treeProbability"].is_null());
    // FM-TX-003 is the event FTE-SYS-001 (its ftaRef): the tree has it
    assert!(!rows.iter().any(|r| r["faults"] == serde_json::json!(["FM-TX-003"])), "{rows:?}");
    // pairs containing a defeating fault, or a cut set, are not run
    assert!(j["sets"].as_array().unwrap().iter().all(|s| !(s["origin"].as_str().unwrap().starts_with("consistency") && s["faults"].as_array().unwrap().iter().any(|f| f == "FM-TX-004"))));
    let (_, t) = run(&root, &["inject", "TC-SYS-001", "--ft", "FT-SYS-001", "--consistency", "--report-only"]);
    assert!(t.contains("1 set(s) missing from the tree, 1 cut set(s) the model survives"), "{t}");
    // the search is bounded, and needs a tree
    let (_, t) = run(&root, &["inject", "TC-SYS-001", "--ft", "FT-SYS-001", "--consistency", "--max-runs", "1", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert!(j["consistency"]["runs"].as_u64().unwrap() <= 1);
    let (c, _) = run(&root, &["inject", "TC-SYS-001", "--consistency"]);
    assert_eq!(c, 1);
    let _ = std::fs::remove_dir_all(&root);
}

/// The worked example: the goal's 1.4e-1 figure over 15 years is flagged as not credible, naming its largest
/// contributor.
#[test]
fn the_grid_tie_figure_is_not_credible_and_names_its_largest_contributor() {
    let gti = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_gti");
    let (c, t) = run(&gti, &["metrics"]);
    assert_eq!(c, 0, "{t}");
    let j = metrics_json(&gti, &[]);
    let g = j.as_array().unwrap().iter().find(|g| g["id"] == "SG-GTI-001").unwrap();
    assert_eq!(g["verdict"], "not credible", "{g}");
    assert!(g["topEventProbability"].as_f64().unwrap() > 0.1, "{g}");
    // FTE-GTI-005 carries most of the top event; FTE-GTI-009 moves it most but contributes little
    assert_eq!(g["largestContributor"]["event"], "FTE-GTI-005", "{g}");
    assert_eq!(g["highestBirnbaum"]["event"], "FTE-GTI-009", "{g}");
    assert!(g["notCredible"].as_array().unwrap().iter().any(|n| n["event"] == "FTE-GTI-005"), "{g}");
    assert_eq!(g["unverifiedRates"].as_array().unwrap().len(), 10, "every GTI rate is an unverified proposal: {g}");
    let (_, t) = run(&gti, &["validate"]);
    assert!(!t.contains("W988"), "the GTI rates are unverified, so they are labelled rather than warned about: {t}");
}

/// Three faults on the transmitter, each harmless alone: `a` and `b` together corrupt the command, `c` never does.
/// `tree` is the gate over FTE-SYS-001/-003 (or the gate and its inputs given).
fn pair_model(gate: &str, inputs: &str) -> PathBuf {
    let root = dir("p");
    let files: Vec<(&str, String)> = vec![
        ("TxM.md", "---\ntype: StateDef\nname: TxM\nvariables:\n  - {name: level, value: \"7\"}\n  - {name: a, value: \"0\"}\n  - {name: b, value: \"0\"}\n  - {name: c, value: \"0\"}\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {target: Idle, accept: Tick, effect: {send: Cmd, via: out, payload: {n: \"level + a * b\"}}}\n---\n".into()),
        ("RxM.md", "---\ntype: StateDef\nname: RxM\nvariables:\n  - {name: got, value: \"0\"}\nsubStates:\n  - name: Waiting\n    isInitial: true\n    transitions:\n      - {target: Done, accept: {payload: Cmd, via: inp, as: m}, assign: \"got := m.n\"}\n  - {name: Done, isFinal: true}\n---\n".into()),
        ("TxP.md", "---\ntype: PartDef\nname: TxP\nexhibitsStates: [TxM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\nTransmitter.\n".into()),
        ("RxP.md", "---\ntype: PartDef\nname: RxP\nexhibitsStates: [RxM]\nfeatures:\n  - {name: inp, type: Port, direction: in}\n---\nReceiver.\n".into()),
        ("Sys.md", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\nconnections:\n  - {from: tx.out, to: rx.inp}\n---\nThe system.\n".into()),
        ("REQ-SYS-001.md", "---\ntype: Requirement\nid: REQ-SYS-001\nname: delivery\nstatus: approved\nreqDomain: software\n---\n\nThe receiver shall get the command.\n".into()),
        ("SG-SYS-001.md", "---\ntype: SafetyGoal\nid: SG-SYS-001\nname: No wrong command is acted on\nstatus: draft\nasilLevel: B\n---\nGoal.\n".into()),
        ("FT-SYS-001.md", "---\ntype: FaultTree\nid: FT-SYS-001\nname: Wrong command\ntopEvent: SG-SYS-001\nstatus: draft\nmissionTime: \"10000 h\"\n---\nTree.\n".into()),
        ("FT-SYS-001/FTG-SYS-001.md", format!("---\ntype: FaultTreeGate\nid: FTG-SYS-001\nname: Wrong command\ngateType: {gate}\ninputs: [{inputs}]\n---\nGate.\n")),
        ("TC-SYS-001.md", "---\ntype: TestCase\nid: TC-SYS-001\nname: link invariants\nstatus: active\ntestLevel: L3\nsubject: Sys\nverifies: [REQ-SYS-001]\n---\n\n```gherkin\nFeature: link\n\n  @invariant\n  Scenario: the receiver never accepts a wrong value\n    When tx receives Tick\n    Then rx.got == 0 or rx.got == 7\n```\n".into()),
    ];
    for (n, c) in &files {
        write(&root, n, c);
    }
    for (i, v) in ["a", "b", "c"].iter().enumerate() {
        write(&root, &format!("FT-SYS-001/FTE-SYS-00{}.md", i + 1), &format!("---\ntype: FaultTreeEvent\nid: FTE-SYS-00{}\nname: {v} stuck\neventKind: basic\nref: TxP\nfailureRate: 1.0e-7\ninjection: [{{kind: stuck, variable: {v}, value: \"1\"}}]\n---\n{v}.\n", i + 1));
    }
    root
}

fn consistency(root: &Path, extra: &[&str]) -> (i32, serde_json::Value) {
    let mut args = vec!["inject", "TC-SYS-001", "--ft", "FT-SYS-001", "--json"];
    args.extend_from_slice(extra);
    let (c, t) = run(root, &args);
    (c, serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}")))
}

/// The search itself: it runs the pairs no cut set or defeat implies, finds the pair the tree lacks, and stops at the
/// run cap.
#[test]
fn the_search_runs_the_pairs_the_tree_does_not_imply() {
    // the tree says a and c together; the model is defeated by a and b together
    let root = pair_model("AND", "FTE-SYS-001, FTE-SYS-003");
    let (_, j) = consistency(&root, &["--consistency"]);
    let c = &j["consistency"];
    assert_eq!(c["runs"], 2, "a+b and b+c run, a+c is the cut set: {c}");
    assert_eq!(c["truncated"], false);
    let rows = c["rows"].as_array().unwrap();
    let row = |f: &[&str]| rows.iter().find(|r| r["faults"] == serde_json::json!(f)).unwrap_or_else(|| panic!("{f:?}: {rows:?}"));
    assert_eq!(row(&["FTE-SYS-001", "FTE-SYS-002"])["finding"], "missing from the tree");
    assert_eq!(row(&["FTE-SYS-001", "FTE-SYS-003"])["finding"], "conservative or a model gap");
    // the run cap stops the search and says so
    let (_, j) = consistency(&root, &["--consistency", "--max-runs", "1"]);
    assert_eq!(j["consistency"]["runs"], 1);
    assert_eq!(j["consistency"]["truncated"], true);
    let _ = std::fs::remove_dir_all(&root);
}

/// A cut set above the campaign's `--max-order` is still a cut set of the tree: never "missing from the tree".
#[test]
fn a_cut_set_above_the_campaign_order_is_not_missing_from_the_tree() {
    let root = pair_model("AND", "FTE-SYS-001, FTE-SYS-002");
    let (_, j) = consistency(&root, &["--consistency"]);
    let rows = j["consistency"]["rows"].as_array().unwrap();
    assert!(rows.iter().any(|r| r["faults"] == serde_json::json!(["FTE-SYS-001", "FTE-SYS-002"]) && r["finding"] == "confirmed"), "{rows:?}");
    let (_, j) = consistency(&root, &["--consistency", "--max-order", "1"]);
    let rows = j["consistency"]["rows"].as_array().unwrap();
    assert!(!rows.iter().any(|r| r["finding"] == "missing from the tree"), "{rows:?}");
    assert!(rows.iter().any(|r| r["faults"] == serde_json::json!(["FTE-SYS-001", "FTE-SYS-002"]) && r["finding"] == "not run (above the campaign's order)"), "{rows:?}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A search order above the number of faults ends cleanly instead of overflowing.
#[test]
fn a_search_order_above_the_number_of_faults_ends_cleanly() {
    let root = pair_model("OR", "FTE-SYS-001, FTE-SYS-002");
    for n in ["FTE-SYS-002", "FTE-SYS-003"] {
        let p = root.join(format!("FT-SYS-001/{n}.md"));
        let t = std::fs::read_to_string(&p).unwrap();
        let t: String = t.lines().filter(|l| !l.starts_with("injection:")).map(|l| format!("{l}\n")).collect();
        std::fs::write(&p, t).unwrap();
    }
    let (c, j) = consistency(&root, &["--consistency"]);
    assert!(c == 0 || c == 2, "{j}");
    assert_eq!(j["consistency"]["runs"], 0, "{j}");
    let root2 = pair_model("OR", "FTE-SYS-001, FTE-SYS-002");
    let (c, j) = consistency(&root2, &["--consistency", "--consistency-order", "5"]);
    assert!(c == 0 || c == 2, "{j}");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&root2);
}

/// A cut set whose event is injected only through an FMEA row (its `ftaRef`) gets that row's verdict.
#[test]
fn a_cut_set_run_through_its_fmea_row_is_confirmed() {
    let root = campaign_model();
    let p = root.join("FT-SYS-001/FTE-SYS-001.md");
    let t: String = std::fs::read_to_string(&p).unwrap().lines().filter(|l| !l.starts_with("injection:")).map(|l| format!("{l}\n")).collect();
    std::fs::write(&p, t).unwrap();
    let (_, j) = consistency(&root, &["--consistency"]);
    let rows = j["consistency"]["rows"].as_array().unwrap();
    let r = rows.iter().find(|r| r["faults"] == serde_json::json!(["FTE-SYS-001"])).unwrap();
    assert_eq!(r["finding"], "confirmed", "{r}");
    let _ = std::fs::remove_dir_all(&root);
}

/// Demand mode: an ASIL goal has no PFDavg limit, so it is `no target` (never a pass on coverage alone), and the
/// credibility test is λ·TI, not λ·T.
#[test]
fn demand_mode_needs_a_pfd_limit_and_judges_credibility_over_the_proof_test_interval() {
    let demand = "target: {mode: demand, proofTestInterval: \"8760 h\"}\n";
    let root = model("OR", demand, "");
    let p = root.join("SG-SYS-001.md");
    std::fs::write(&p, std::fs::read_to_string(&p).unwrap().replace("silLevel: 2", "asilLevel: D")).unwrap();
    let g = goal(&root);
    assert_eq!(g["verdict"], "no target", "{g}");
    let (_, t) = run(&root, &["metrics"]);
    assert!(t.contains("no PFDavg limit"), "{t}");
    let root = model("OR", demand, "diagnosticCoverage: 0.99\n");
    let p = root.join("SG-SYS-001.md");
    std::fs::write(&p, std::fs::read_to_string(&p).unwrap().replace("silLevel: 2", "asilLevel: D")).unwrap();
    assert_eq!(goal(&root)["verdict"], "no target");
    // λ 1e-6 over a 131 400 h mission is 0.88, but over the 8760 h proof-test interval 0.0088: credible
    let root = model("OR", demand, "");
    for (n, from) in [("FT-SYS-001.md", "\"10000 h\""), ("FT-SYS-001/FTE-SYS-001.md", "1.0e-7")] {
        let p = root.join(n);
        let to = if n == "FT-SYS-001.md" { "\"131400 h\"" } else { "1.0e-6" };
        std::fs::write(&p, std::fs::read_to_string(&p).unwrap().replace(from, to)).unwrap();
    }
    let g = goal(&root);
    assert_eq!(g["verdict"], "pass", "{g}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A rate-only `fail` is advisory until the project opts in; a `.syscribe.toml` limit is an opt-in like `target:`.
#[test]
fn a_rate_only_failure_gates_only_once_the_project_opts_in() {
    let root = model("OR", "", "");
    let p = root.join("FT-SYS-001/FTE-SYS-001.md");
    std::fs::write(&p, std::fs::read_to_string(&p).unwrap().replace("1.0e-7", "2.0e-6")).unwrap();
    let g = goal(&root);
    assert!(g["verdict"] == "fail" && g["gates"] == false, "{g}");
    let (c, t) = run(&root, &["metrics"]);
    assert!(c == 0 && t.contains("advisory"), "{t}");
    assert!(codes(&root, "W033").is_empty());
    write(&root, ".syscribe.toml", "[reliability.targets.sil.2]\npfh = 1.0e-6\n");
    let g = goal(&root);
    assert!(g["gates"] == true && g["target"]["source"] == ".syscribe.toml", "{g}");
    assert_eq!(run(&root, &["metrics"]).0, 2);
    assert_eq!(codes(&root, "W033").len(), 1);
    // a `pfd` limit alone leaves the continuous limit with the built-in table
    write(&root, ".syscribe.toml", "[reliability.targets.sil.2]\npfd = 1.0e-3\n");
    assert_eq!(goal(&root)["target"]["source"], "built-in default");
    let _ = std::fs::remove_dir_all(&root);
}

/// The new fields stay W047 on types that do not use them, and a malformed target is shown as ignored.
#[test]
fn misplaced_fields_are_w047_and_a_malformed_target_is_shown_as_ignored() {
    let root = model("OR", "target: {mode: demand, proofTestInterval: soon}\n", "");
    write(&root, "P.md", "---\ntype: PartDef\nname: P\ntarget: {value: 1}\nrateStatus: verified\n---\nP.\n");
    let w = codes(&root, "W047");
    assert!(w.iter().any(|l| l.contains("'target'")) && w.iter().any(|l| l.contains("'rateStatus'")), "{w:?}");
    assert_eq!(goal(&root)["targetIgnored"], true);
    let (_, t) = run(&root, &["metrics"]);
    assert!(t.contains("malformed (E985) and was ignored"), "{t}");
    // `--rate-status none` lists a rate with no status, not an element with no rate
    let (_, t) = run(&root, &["list", "FaultTreeEvent", "--rate-status", "none"]);
    assert!(t.contains("FTE-SYS-001"), "{t}");
    let (_, t) = run(&root, &["list", "FaultTreeGate", "--rate-status", "none"]);
    assert!(t.contains("FTG-SYS-001"), "a gate is not a rate holder, so the filter leaves it: {t}");
    let _ = std::fs::remove_dir_all(&root);
}

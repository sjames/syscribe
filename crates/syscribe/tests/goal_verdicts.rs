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

/// The worked example: the goal's 1.4e-1 figure over 15 years is flagged as not credible, with FTE-GTI-009 as the
/// dominant event.
#[test]
fn the_grid_tie_figure_is_not_credible_and_names_the_dominant_event() {
    let gti = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_gti");
    let (c, t) = run(&gti, &["metrics"]);
    assert_eq!(c, 0, "{t}");
    let j = metrics_json(&gti, &[]);
    let g = j.as_array().unwrap().iter().find(|g| g["id"] == "SG-GTI-001").unwrap();
    assert_eq!(g["verdict"], "not credible", "{g}");
    assert!(g["topEventProbability"].as_f64().unwrap() > 0.1, "{g}");
    assert_eq!(g["dominantEvent"]["event"], "FTE-GTI-009");
    assert!(g["notCredible"].as_array().unwrap().iter().any(|n| n["event"] == "FTE-GTI-005"), "{g}");
    assert_eq!(g["unverifiedRates"].as_array().unwrap().len(), 10, "every GTI rate is an unverified proposal: {g}");
    let (_, t) = run(&gti, &["validate"]);
    assert!(!t.contains("W988"), "the GTI rates are unverified, so they are labelled rather than warned about: {t}");
}

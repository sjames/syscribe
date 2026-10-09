//! GH #216/#217/#218: safety-case traversal (`syscribe_model::safety_case`),
//! GSN validation (E878, W861), FMEA row quality (W931, W932, W904 without
//! E115), FMEDA rows feeding `metrics`, and the `[audit]` config table.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::config::{AuditConfig, ValidateConfig};
use syscribe_model::resolver::Resolver;
use syscribe_model::safety_case::{build, BuildOptions, GoalVerdict, NodeKind, NodeStatus, Verdict};
use syscribe_model::validator::{validate_with_config, ValidationResult};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-safety-case-test-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn validate(root: &Path) -> ValidationResult {
    let elements = walk_model(root).unwrap();
    validate_with_config(&elements, &ValidateConfig::with_model_root(root.to_path_buf()))
}

fn count(r: &ValidationResult, code: &str) -> usize {
    r.findings.iter().filter(|f| f.code == code).count()
}

const GOAL: &str = "---\ntype: SafetyGoal\nid: SG-T-001\nname: G\nstatus: approved\nasilLevel: B\nsafeState: s\n---\n";

fn req(id: &str, extra: &str) -> String {
    format!("---\ntype: Requirement\nid: {id}\nname: R {id}\nstatus: approved\n{extra}---\n")
}

fn tc(id: &str, verifies: &str) -> String {
    format!(
        "---\ntype: TestCase\nid: {id}\nname: T {id}\ntestLevel: L3\nstatus: approved\nverifies: [{verifies}]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n"
    )
}

#[test]
fn implicit_chain_kept_with_argument_and_walks_derived_children() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: M\n---\n");
    write(&root, "SG-T-001.md", GOAL);
    write(
        &root,
        "ARG-T-001.md",
        "---\ntype: Argument\nid: ARG-T-001\nname: A\nstatus: approved\nsupports: SG-T-001\nevidence: [REQ-TST-001]\n---\n",
    );
    write(&root, "REQ-TST-001.md", &req("REQ-TST-001", "derivedFromSafetyGoal: SG-T-001\n"));
    // Uncited by the Argument: must still be folded in, with a derived child and a test.
    write(&root, "REQ-TST-002.md", &req("REQ-TST-002", "derivedFromSafetyGoal: SG-T-001\n"));
    write(&root, "REQ-TST-003.md", &req("REQ-TST-003", "derivedFrom: REQ-TST-002\nbreakdownAdr: ADR-T-001\n"));
    write(&root, "TC-TST-001.md", &tc("TC-TST-001", "REQ-TST-003"));

    let elements = walk_model(&root).unwrap();
    let resolver = Resolver::new(&elements);
    let verdict = |_: &syscribe_model::element::RawElement| Verdict::Pass;
    let case = build(&elements, &resolver, "", BuildOptions::default(), &verdict);
    assert_eq!(case.goals.len(), 1);
    let g = &case.goals[0].root;
    let arg = g.children.iter().find(|c| c.kind == NodeKind::Claim).expect("argument child");
    assert!(arg.children.iter().any(|c| c.id == "REQ-TST-001"));
    let implicit: Vec<&str> =
        g.children.iter().filter(|c| c.implicit).map(|c| c.id.as_str()).collect();
    assert_eq!(implicit, vec!["REQ-TST-002"], "uncited requirement folded in, cited one not repeated");
    let r2 = g.children.iter().find(|c| c.id == "REQ-TST-002").unwrap();
    let r3 = r2.children.iter().find(|c| c.id == "REQ-TST-003").expect("derived child walked");
    assert!(r3.children.iter().any(|c| c.id == "TC-TST-001" && c.status == NodeStatus::Supported));
    // REQ-TST-001 has no test: the goal is incomplete and the node is undeveloped.
    assert_eq!(case.goals[0].verdict, GoalVerdict::Incomplete);
    assert!(case.completeness.undeveloped_ids.contains(&"REQ-TST-001".to_string()));

    let off = build(&elements, &resolver, "", BuildOptions { include_implicit: false }, &verdict);
    assert!(off.goals[0].root.children.iter().all(|c| !c.implicit));
    let none = build(&elements, &resolver, "SG-NOPE", BuildOptions::default(), &verdict);
    assert!(none.goals.is_empty());
}

#[test]
fn failing_test_makes_goal_failing() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: M\n---\n");
    write(&root, "SG-T-001.md", GOAL);
    write(&root, "REQ-TST-001.md", &req("REQ-TST-001", "derivedFromSafetyGoal: SG-T-001\n"));
    write(&root, "TC-TST-001.md", &tc("TC-TST-001", "REQ-TST-001"));
    let elements = walk_model(&root).unwrap();
    let resolver = Resolver::new(&elements);
    let case = build(&elements, &resolver, "", BuildOptions::default(), &|_| Verdict::Fail);
    assert_eq!(case.goals[0].verdict, GoalVerdict::Failing);
    assert_eq!(case.completeness.tests_fail, 1);
}

#[test]
fn argument_cycle_is_e878_and_solution_without_evidence_is_w861() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: M\n---\n");
    write(&root, "SG-T-001.md", GOAL);
    write(
        &root,
        "ARG-T-001.md",
        "---\ntype: Argument\nid: ARG-T-001\nname: A\nstatus: draft\nsupports: SG-T-001\nevidence: ARG-T-002\n---\n",
    );
    write(
        &root,
        "ARG-T-002.md",
        "---\ntype: Argument\nid: ARG-T-002\nname: B\nstatus: draft\nevidence: ARG-T-001\n---\n",
    );
    write(
        &root,
        "ARG-T-003.md",
        "---\ntype: Argument\nid: ARG-T-003\nname: S\nstatus: approved\nargumentType: solution\nsupports: SG-T-001\n---\n",
    );
    let r = validate(&root);
    assert_eq!(count(&r, "E878"), 2);
    assert_eq!(count(&r, "W861"), 1);
    assert_eq!(count(&r, "E854"), 0);
}

#[test]
fn fmea_row_quality_and_single_report_of_dangling_ref() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: M\n---\n");
    write(
        &root,
        "FMEA-T-001.md",
        "---\ntype: FMEASheet\nid: FMEA-T-001\nname: S\nstatus: approved\nentries:\n  - {id: FM-T-001, failureMode: a, fmeaSeverity: 10, occurrence: 2, detection: 2}\n  - {id: FM-T-002, failureMode: b}\n  - {id: FM-T-003, failureMode: c, ref: No::Where, fmeaSeverity: 2, occurrence: 2, detection: 2}\n---\n",
    );
    let r = validate(&root);
    assert_eq!(count(&r, "W932"), 1);
    assert_eq!(count(&r, "W931"), 1);
    assert_eq!(count(&r, "W904"), 1);
    assert_eq!(count(&r, "E115"), 0, "unresolved FMEA ref is reported once");
}

#[test]
fn fmea_row_failure_rate_feeds_goal_metrics() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: M\n---\n");
    write(&root, "SG-T-001.md", &GOAL.replace("asilLevel: B", "asilLevel: D"));
    write(
        &root,
        "FT-TST-001/_index.md",
        "---\ntype: FaultTree\nid: FT-TST-001\nname: FT\nstatus: approved\ntopEvent: SG-T-001\n---\n",
    );
    write(
        &root,
        "FT-TST-001/FTE-TST-001.md",
        "---\ntype: FaultTreeEvent\nid: FTE-TST-001\nname: E\nstatus: approved\neventKind: basic\n---\n",
    );
    write(
        &root,
        "FMEA-T-001.md",
        "---\ntype: FMEASheet\nid: FMEA-T-001\nname: S\nstatus: approved\nentries:\n  - {id: FM-T-001, failureMode: a, fmeaSeverity: 5, occurrence: 2, detection: 2, ftaRef: FTE-TST-001, failureRate: 1.0e-7, diagnosticCoverage: 0.5, recommendedAction: x}\n---\n",
    );
    let elements = walk_model(&root).unwrap();
    let resolver = Resolver::new(&elements);
    let reports = syscribe_model::metrics::report_all(&elements, &resolver);
    let m = reports[0].metrics.as_ref().expect("metrics computed from the FMEA row");
    assert!((m.spfm.unwrap() - 0.5).abs() < 1e-9);
    assert!(!reports[0].gate.as_ref().unwrap().passed(), "ASIL D target missed");
    let r = validate(&root);
    assert_eq!(count(&r, "W033"), 1);
    assert_eq!(count(&r, "E922"), 0, "failureRate/diagnosticCoverage are recognised row keys");
}

#[test]
fn audit_config_defaults_overrides_and_problems() {
    let root = tempdir();
    let d = AuditConfig::load(&root);
    assert_eq!(d, AuditConfig::default());
    assert_eq!(d.fail_on, vec!["W306".to_string()]);
    assert_eq!(d.fail_on_asil["W805"], vec!["C".to_string(), "D".to_string()]);

    write(
        &root,
        ".syscribe.toml",
        "[audit]\nfail_on = [\"W306\", \"W800\", \"bogus\"]\nunknown = 1\n[audit.fail_on_asil]\nW033 = [\"b\", \"D\"]\nW805 = [\"Z\"]\n",
    );
    let c = AuditConfig::load(&root);
    assert_eq!(c.fail_on, vec!["W306".to_string(), "W800".to_string()]);
    assert_eq!(c.fail_on_asil.len(), 1);
    assert_eq!(c.fail_on_asil["W033"], vec!["B".to_string(), "D".to_string()]);
    assert_eq!(c.problems.len(), 3, "{:?}", c.problems);

    write(&root, "_index.md", "---\ntype: Package\nname: M\n---\n");
    let r = validate(&root);
    assert_eq!(count(&r, "W934"), 3);

    write(&root, ".syscribe.toml", "[audit]\n[audit.fail_on_asil]\n");
    let empty = AuditConfig::load(&root);
    assert!(empty.fail_on_asil.is_empty(), "an empty table opts out of the ASIL-gated defaults");
}

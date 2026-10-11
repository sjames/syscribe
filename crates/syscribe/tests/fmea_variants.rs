//! GH #313: an FMEA row can be gated by `appliesWhen:`, and an injectable fault's value can follow a feature parameter.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-fv-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&r).unwrap();
    let w = |n: &str, c: &str| std::fs::write(r.join(n), c).unwrap();
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features.md", "---\ntype: FeatureModel\nname: Features\nfeatureTree:\n  - {name: Opt, id: FEAT-FV-OPT}\n  - name: Nominal\n    id: FEAT-FV-NOM\n    mandatory: true\n    parameters:\n      - {name: hz, type: ScalarValues::Integer, range: \"40..=70\", default: 50, isRequired: true, bindingTime: load}\n---\n");
    for (cid, opt, hz) in [("CONF-FV-ON-001", true, 60), ("CONF-FV-OFF-001", false, 50)] {
        w(&format!("{cid}.md"), &format!("---\ntype: Configuration\nid: {cid}\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: {opt}\n  Features::Nominal: true\nparameterBindings:\n  Features::Nominal.hz: {hz}\n---\n"));
    }
    w("OptPart.md", "---\ntype: PartDef\nname: OptPart\nappliesWhen: Features::Opt\n---\nOptional part.\n");
    w("Always.md", "---\ntype: PartDef\nname: Always\n---\nAlways there.\n");
    w("Use.md", "---\ntype: Part\nname: Use\ntypedBy: Always\n---\nUse.\n");
    w("M.md", "---\ntype: StateDef\nname: M\nvariables:\n  - {name: f, value: \"0\"}\nsubStates:\n  - {name: A, isInitial: true}\n---\n");
    w("P.md", "---\ntype: PartDef\nname: P\nexhibitsStates: [M]\n---\nP.\n");
    w("Sys.md", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: p, typedBy: P}\n---\nSys.\n");
    w("REQ-FV-001.md", "---\ntype: Requirement\nid: REQ-FV-001\nname: r\nstatus: draft\nreqDomain: software\n---\n\nShall.\n");
    w("TC-FV-001.md", "---\ntype: TestCase\nid: TC-FV-001\nname: t\nstatus: active\ntestLevel: L3\nsubject: Sys\nverifies: [REQ-FV-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: the stuck value is the nominal of the variant\n    Given fault FM-FV-002 is injected\n    Then p.f >= 55\n```\n");
    w("FMEA-FV-001.md", "---\ntype: FMEASheet\nid: FMEA-FV-001\nname: s\nstatus: draft\nentries:\n  - {id: FM-FV-001, ref: OptPart, failureMode: \"the optional part fails\", appliesWhen: Features::Opt}\n  - {id: FM-FV-002, ref: P, failureMode: \"sensor reads nominal\", injection: [{kind: stuck, variable: f, value: \"{{Features::Nominal.hz}}\"}]}\n---\n");
    r
}

fn run(root: &PathBuf, args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

#[test]
fn a_gated_row_is_accepted_and_drops_out_of_a_variant_without_its_feature() {
    let root = model();
    let t = run(&root, &["validate"]);
    assert!(!t.contains("E922"), "appliesWhen is a recognised key of a row: {t}");
    let t = run(&root, &["validate", "--config", "CONF-FV-OFF-001"]);
    assert!(!t.contains("W904") && !t.contains("W019") && !t.contains("E226"), "the gated row is not part of the variant: {t}");
    assert!(run(&root, &["validate", "--all-configs"]).contains("2 pass"));
}

#[test]
fn a_fault_value_can_follow_a_feature_parameter() {
    let root = model();
    let sim = |c: &str| {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).args(["simulate", "TC-FV-001", "--config", c]).output().unwrap();
        (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
    };
    assert_eq!(sim("CONF-FV-ON-001").0, 0, "60 Hz stuck value: {}", sim("CONF-FV-ON-001").1);
    assert_eq!(sim("CONF-FV-OFF-001").0, 2, "50 Hz stuck value fails the >= 55 check: {}", sim("CONF-FV-OFF-001").1);
    let t = run(&root, &["validate"]);
    assert!(!t.contains("E936"), "the symbolic fault model is not malformed: {t}");
}

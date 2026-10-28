//! REQ-TRS-TCELEM-001: `TestCase.verifiesElement:` and the untested-element warning `W104`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const TC_BODY: &str = "\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n";

/// A model of one `PartDef` named `Unit` plus the extra files given as (name, content).
fn model(part_extra: &str, files: &[(&str, String)]) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-tcelem-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&r).unwrap();
    let w = |n: &str, c: &str| std::fs::write(r.join(n), c).unwrap();
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Unit.md", &format!("---\ntype: PartDef\nname: Unit\nstatus: approved\n{part_extra}---\nUnit.\n"));
    for (n, c) in files {
        w(n, c);
    }
    r
}

fn tc(status: &str, link: &str) -> (&'static str, String) {
    ("TC-TE-001.md", format!("---\ntype: TestCase\nid: TC-TE-001\nname: t\nstatus: {status}\ntestLevel: L1\n{link}---\n{TC_BODY}"))
}

fn validate(root: &PathBuf) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).arg("validate").output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn has(out: &str, code: &str) -> bool {
    out.contains(&format!("| {code} |"))
}

#[test]
fn untested_active_element_warns_and_a_direct_link_clears_it() {
    let out = validate(&model("", &[]));
    assert!(has(&out, "W104"), "an untested active PartDef raises W104: {out}");
    let out = validate(&model("", &[tc("active", "verifiesElement: [Unit]\n")]));
    assert!(!has(&out, "W104") && !has(&out, "E013") && !has(&out, "E129") && !has(&out, "E102"), "a resolving verifiesElement clears W104 and satisfies E013: {out}");
}

#[test]
fn draft_exempt_and_inactive_test_do_not_count() {
    assert!(!has(&validate(&model("testExempt: pure data holder\n", &[])), "W104"), "testExempt suppresses W104");
    assert!(!has(&validate(&model("", &[]).tap_draft()), "W104"), "a draft element never raises W104");
    let out = validate(&model("", &[tc("draft", "verifiesElement: [Unit]\n")]));
    assert!(has(&out, "W104"), "a draft TestCase does not clear W104: {out}");
}

#[test]
fn a_satisfied_verified_requirement_clears_w104() {
    let req = ("REQ-TE-001.md", "---\ntype: Requirement\nid: REQ-TE-001\nname: r\nstatus: approved\nreqDomain: software\n---\n\nThe unit shall work.\n".to_string());
    let t = tc("active", "verifies: [REQ-TE-001]\n");
    let out = validate(&model("satisfies: [REQ-TE-001]\ndomain: software\n", &[req, t]));
    assert!(!has(&out, "W104"), "W104 is cleared through a satisfied, verified requirement: {out}");
}

#[test]
fn bad_targets_are_reported() {
    let out = validate(&model("", &[tc("active", "verifiesElement: [Nope]\n")]));
    assert!(has(&out, "E102"), "an unresolved target is E102: {out}");
    let req = ("REQ-TE-001.md", "---\ntype: Requirement\nid: REQ-TE-001\nname: r\nstatus: draft\nreqDomain: software\n---\n\nThe unit shall work.\n".to_string());
    let out = validate(&model("", &[req, tc("active", "verifiesElement: [REQ-TE-001]\n")]));
    assert!(has(&out, "E129"), "a target of the wrong kind is E129: {out}");
}

#[test]
fn a_testcase_with_no_link_at_all_is_still_e013() {
    assert!(has(&validate(&model("", &[tc("active", "")])), "E013"));
}

trait TapDraft {
    fn tap_draft(self) -> Self;
}
impl TapDraft for PathBuf {
    fn tap_draft(self) -> Self {
        let p = self.join("Unit.md");
        let s = std::fs::read_to_string(&p).unwrap().replace("status: approved", "status: draft");
        std::fs::write(&p, s).unwrap();
        self
    }
}

#[test]
fn verifying_an_element_the_variant_removes_escapes_as_w019() {
    let feat = ("Features.md", "---\ntype: FeatureModel\nname: Features\nfeatureTree:\n  - {name: Opt, id: FEAT-TE-OPT}\n---\n".to_string());
    let conf = |id: &str, on: bool| (Box::leak(format!("{id}.md").into_boxed_str()) as &str, format!("---\ntype: Configuration\nid: {id}\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: {on}\n---\n"));
    let root = model("appliesWhen: Features::Opt\n", &[feat, conf("CONF-TE-ON-001", true), conf("CONF-TE-OFF-001", false), tc("active", "verifiesElement: [Unit]\n")]);
    let lens = |c: &str| {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).args(["validate", "--config", c]).output().unwrap();
        format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
    };
    let off = lens("CONF-TE-OFF-001");
    assert!(has(&off, "W019") && off.contains("TC-TE-001.md"), "an ungated test naming a removed element escapes: {off}");
    assert!(!has(&lens("CONF-TE-ON-001"), "W019"), "nothing escapes when the feature is on");
}

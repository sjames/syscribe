//! TC-TRS-PHOLD-001 / GH #264, #265.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::projection::{project, resolve_selection, SelectionOutcome};
use syscribe_model::validator::{validate, Finding};
use syscribe_model::walker::walk_model;

fn model(files: &[(&str, &str)]) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-phold-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (rel, c) in files {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    r
}

const DISPLAY: (&str, &str) = (
    "Features/Display.md",
    "---\ntype: FeatureDef\nid: FEAT-PH-100\nname: Display\ngroupKind: optional\nparameters:\n  - {name: sizeInch, type: ScalarValues::Real, unit: in, isRequired: true}\n  - {name: refreshHz, type: ScalarValues::Real, unit: Hz, default: 60}\n  - {name: fixedMs, type: ScalarValues::Real, unit: ms, isFixed: true, value: 33}\n  - {name: live, type: ScalarValues::Real, bindingTime: runtime}\n---\n\nDisplay.\n",
);
const OTHER: (&str, &str) = ("Features/Other.md", "---\ntype: FeatureDef\nid: FEAT-PH-101\nname: Other\ngroupKind: optional\nparameters:\n  - {name: p, type: ScalarValues::Real, default: 1}\n---\n\nOther.\n");
const CONF_A: (&str, &str) = (
    "Configs/CONF-PH-ALPHA-001.md",
    "---\nid: CONF-PH-ALPHA-001\ntype: Configuration\nname: a\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Display: true\n  Features::Other: false\nparameterBindings:\n  Features::Display.sizeInch: 8\n---\n\nA.\n",
);
const CONF_B: (&str, &str) = (
    "Configs/CONF-PH-BRAVO-001.md",
    "---\nid: CONF-PH-BRAVO-001\ntype: Configuration\nname: b\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Display: true\n  Features::Other: false\nparameterBindings:\n  Features::Display.sizeInch: 12.3\n---\n\nB.\n",
);

fn req(id: &str, status: &str, aw: &str, body: &str) -> (String, String) {
    (
        format!("Reqs/{id}.md"),
        format!("---\nid: {id}\ntype: Requirement\nname: \"Panel is {{{{Features::Display.sizeInch}}}} in\"\nstatus: {status}\nreqDomain: software\n{aw}---\n\n{body}\n"),
    )
}

fn proj(root: &PathBuf, conf: &str) -> Vec<syscribe_model::element::RawElement> {
    let els = walk_model(root).unwrap();
    let SelectionOutcome::Resolved(sel) = resolve_selection(&els, conf) else { panic!("unresolved") };
    project(&els, &sel)
}

fn doc_of(els: &[syscribe_model::element::RawElement], id: &str) -> String {
    els.iter().find(|e| e.frontmatter.id.as_deref() == Some(id)).unwrap().doc.clone()
}

fn name_of(els: &[syscribe_model::element::RawElement], id: &str) -> String {
    els.iter().find(|e| e.frontmatter.id.as_deref() == Some(id)).unwrap().frontmatter.name.clone().unwrap()
}

fn base_model(extra: Vec<(String, String)>) -> PathBuf {
    let mut files: Vec<(&str, String)> = vec![
        (DISPLAY.0, DISPLAY.1.to_string()),
        (OTHER.0, OTHER.1.to_string()),
        (CONF_A.0, CONF_A.1.to_string()),
        (CONF_B.0, CONF_B.1.to_string()),
    ];
    for (a, b) in &extra {
        files.push((a.as_str(), b.clone()));
    }
    let refs: Vec<(&str, &str)> = files.iter().map(|(a, b)| (*a, b.as_str())).collect();
    model(&refs)
}

fn codes(root: &PathBuf) -> Vec<Finding> {
    validate(&walk_model(root).unwrap()).findings
}

fn n(f: &[Finding], c: &str) -> usize {
    f.iter().filter(|x| x.code == c).count()
}

#[test]
fn each_configuration_sees_its_own_value_and_the_base_stays_symbolic() {
    let r = base_model(vec![req(
        "REQ-PH-001",
        "approved",
        "",
        "The panel shall be {{Features::Display.sizeInch|unit}} at {{FEAT-PH-100.refreshHz|unit}}, frame {{Features::Display.fixedMs}} ms.",
    )]);
    let a = proj(&r, "CONF-PH-ALPHA-001");
    assert_eq!(doc_of(&a, "REQ-PH-001").trim(), "The panel shall be 8 in at 60 Hz, frame 33 ms.");
    assert_eq!(name_of(&a, "REQ-PH-001"), "Panel is 8 in");
    let b = proj(&r, "CONF-PH-BRAVO-001");
    assert!(doc_of(&b, "REQ-PH-001").contains("12.3 in"), "{}", doc_of(&b, "REQ-PH-001"));
    let base = walk_model(&r).unwrap();
    assert!(doc_of(&base, "REQ-PH-001").contains("{{Features::Display.sizeInch|unit}}"));
}

#[test]
fn an_unresolvable_placeholder_is_left_as_written() {
    let r = base_model(vec![req("REQ-PH-001", "draft", "", "Value {{Features::Display.live}}.")]);
    let a = proj(&r, "CONF-PH-ALPHA-001");
    assert!(doc_of(&a, "REQ-PH-001").contains("{{Features::Display.live}}"));
}

#[test]
fn a_clean_model_raises_none_of_the_new_findings() {
    let r = base_model(vec![req("REQ-PH-001", "approved", "appliesWhen: Features::Display\n", "Size {{Features::Display.sizeInch}}.")]);
    let f = codes(&r);
    for c in ["E240", "E241", "E242", "E243", "W245", "W246"] {
        assert_eq!(n(&f, c), 0, "{c}: {f:?}");
    }
}

#[test]
fn a_placeholder_without_a_feature_model_or_configuration_is_e240() {
    let r = model(&[("REQ-PH-001.md", "---\nid: REQ-PH-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\n---\n\nSize {{Features::Display.sizeInch}}.\n")]);
    assert_eq!(n(&codes(&r), "E240"), 1);
    // A feature model but no configuration is also a gate failure.
    let r = model(&[
        DISPLAY,
        ("REQ-PH-001.md", "---\nid: REQ-PH-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\n---\n\nSize {{Features::Display.sizeInch}}.\n"),
    ]);
    assert_eq!(n(&codes(&r), "E240"), 1);
}

#[test]
fn unknown_feature_or_parameter_is_e241() {
    let r = base_model(vec![
        req("REQ-PH-001", "draft", "appliesWhen: Features::Display\n", "{{Features::Nope.x}}"),
        req("REQ-PH-002", "draft", "appliesWhen: Features::Display\n", "{{Features::Display.nope}}"),
    ]);
    assert_eq!(n(&codes(&r), "E241"), 2);
}

#[test]
fn an_ungated_element_referencing_an_unselected_feature_is_e242() {
    // Other is off in both configurations; the element is active (no appliesWhen).
    let r = base_model(vec![(
        "Reqs/REQ-PH-001.md".to_string(),
        "---\nid: REQ-PH-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\n---\n\nX {{Features::Other.p}}.\n".to_string(),
    )]);
    assert_eq!(n(&codes(&r), "E242"), 1);
    // Properly gated: no E242.
    let r = base_model(vec![(
        "Reqs/REQ-PH-001.md".to_string(),
        "---\nid: REQ-PH-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\nappliesWhen: Features::Other\n---\n\nX {{Features::Other.p}}.\n".to_string(),
    )]);
    assert_eq!(n(&codes(&r), "E242"), 0);
}

#[test]
fn unbound_required_parameter_is_w245_for_draft_and_e243_for_approved() {
    // A third configuration that does not bind sizeInch (required, no default).
    let c = (
        "Configs/CONF-PH-CHARLIE-001.md".to_string(),
        "---\nid: CONF-PH-CHARLIE-001\ntype: Configuration\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Display: true\n  Features::Other: false\n---\n\nC.\n".to_string(),
    );
    let r = base_model(vec![c.clone(), req("REQ-PH-001", "draft", "appliesWhen: Features::Display\n", "{{Features::Display.sizeInch}}")]);
    let f = codes(&r);
    assert_eq!((n(&f, "W245"), n(&f, "E243")), (1, 0), "{f:?}");
    let r = base_model(vec![c, req("REQ-PH-001", "approved", "appliesWhen: Features::Display\n", "{{Features::Display.sizeInch}}")]);
    let f = codes(&r);
    assert_eq!((n(&f, "W245"), n(&f, "E243")), (0, 1), "{f:?}");
}

#[test]
fn a_runtime_parameter_reference_is_w246() {
    let r = base_model(vec![req("REQ-PH-001", "draft", "appliesWhen: Features::Display\n", "{{Features::Display.live}}")]);
    assert_eq!(n(&codes(&r), "W246"), 1);
}

#[test]
fn placeholders_in_code_spans_and_fences_are_literal() {
    // Documentation about placeholders must be able to show them (this repo's own model does).
    use syscribe_model::placeholders::find;
    assert!(find("see `{{Features::Display.sizeInch}}` here").is_empty());
    assert!(find("```\n{{Features::Display.sizeInch}}\n```").is_empty());
    assert_eq!(find("real {{Features::Display.sizeInch}} and `{{Features::Display.refreshHz}}`").len(), 1);
    let r = base_model(vec![req("REQ-PH-001", "draft", "appliesWhen: Features::Display\n", "Write `{{Features::Display.sizeInch}}` to reference it.")]);
    let a = proj(&r, "CONF-PH-ALPHA-001");
    assert!(doc_of(&a, "REQ-PH-001").contains("`{{Features::Display.sizeInch}}`"));
    // And a model that only documents the syntax raises no gate finding.
    let r = model(&[("REQ-PH-001.md", "---\nid: REQ-PH-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\n---\n\nUse `{{Features::X.y}}`.\n")]);
    assert_eq!(n(&codes(&r), "E240"), 0);
}

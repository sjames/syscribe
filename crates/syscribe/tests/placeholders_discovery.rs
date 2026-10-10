//! TC-TRS-PHOLD-003 / GH #267.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-phdisc-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/Display.md", "---\ntype: FeatureDef\nid: FEAT-PD-100\nname: Display\ngroupKind: optional\nparameters:\n  - {name: sizeInch, type: ScalarValues::Real, unit: in}\n  - {name: refreshHz, type: ScalarValues::Real, unit: Hz, default: 60}\n  - {name: pixels, type: ScalarValues::Real, derivedFrom: \"sizeInch * 100\"}\n---\n\nD.\n");
    for (id, v) in [("CONF-PD-ALPHA-001", Some("8")), ("CONF-PD-BRAVO-001", None)] {
        let b = v.map(|v| format!("parameterBindings:\n  Features::Display.sizeInch: {v}\n")).unwrap_or_default();
        w(
            &format!("Configs/{id}.md"),
            &format!("---\nid: {id}\ntype: Configuration\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Display: true\n{b}---\n\nC.\n"),
        );
    }
    w(
        "Reqs/REQ-PD-001.md",
        "---\nid: REQ-PD-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\nappliesWhen: Features::Display\n---\n\nThe panel shall be {{Features::Display.sizeInch|unit}} at {{FEAT-PD-100.refreshHz|unit}}.\n",
    );
    r
}

fn run(root: &Path, args: &[&str]) -> String {
    String::from_utf8_lossy(&Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap().stdout).into_owned()
}

#[test]
fn the_feature_card_lists_consumers_and_per_configuration_values_as_json() {
    let out = run(&model(), &["feature", "Features::Display", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    let cons = &v["parameterConsumers"];
    assert!(cons["sizeInch"].as_array().unwrap().iter().any(|q| q.as_str().unwrap().ends_with("REQ-PD-001")), "{v}");
    assert!(cons["refreshHz"].as_array().unwrap().iter().any(|q| q.as_str().unwrap().ends_with("REQ-PD-001")), "{v}");
    let vals = &v["parameterValues"];
    assert_eq!(vals["sizeInch"]["CONF-PD-ALPHA-001"], "8", "{v}");
    assert!(vals["sizeInch"]["CONF-PD-BRAVO-001"].is_null(), "unbound: {v}");
    assert_eq!(vals["refreshHz"]["CONF-PD-BRAVO-001"], "60", "default: {v}");
}

#[test]
fn the_feature_card_text_names_the_consumers() {
    let out = run(&model(), &["feature", "Features::Display"]);
    assert!(out.contains("Consumers") && out.contains("REQ-PD-001"), "{out}");
    assert!(out.contains("CONF-PD-ALPHA-001") && out.contains("8"), "{out}");
}

#[test]
fn links_and_refs_show_placeholder_references() {
    let r = model();
    let links = run(&r, &["links", "REQ-PD-001"]);
    assert!(links.contains("placeholder") && links.contains("Features::Display"), "{links}");
    let refs = run(&r, &["refs", "Features::Display"]);
    assert!(refs.contains("REQ-PD-001"), "{refs}");
}

#[test]
fn a_derived_parameter_shows_as_derived_not_unbound() {
    let out = run(&model(), &["feature", "Features::Display", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["parameterValues"]["pixels"]["CONF-PD-ALPHA-001"], "(derived)", "{v}");
    let text = run(&model(), &["feature", "Features::Display"]);
    assert!(text.contains("(derived)"), "{text}");
}

#[test]
fn a_mistyped_parameter_is_not_listed_as_a_consumer() {
    let r = model();
    std::fs::write(
        r.join("Reqs/REQ-PD-002.md"),
        "---\nid: REQ-PD-002\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\nappliesWhen: Features::Display\n---\n\nShall {{Features::Display.sizeInchh}}.\n",
    )
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(&run(&r, &["feature", "Features::Display", "--json"])).unwrap();
    assert!(v["parameterConsumers"].get("sizeInchh").is_none(), "{v}");
}

#[test]
fn a_feature_without_parameters_keeps_its_card_unchanged() {
    let r = model();
    std::fs::write(r.join("Features/Plain.md"), "---\ntype: FeatureDef\nid: FEAT-PD-101\nname: Plain\ngroupKind: optional\n---\n\nP.\n").unwrap();
    let text = run(&r, &["feature", "Features::Plain"]);
    assert!(!text.contains("Consumers") && !text.contains("Parameter values"), "{text}");
}

#[test]
fn refs_accepts_the_feature_by_id() {
    let refs = run(&model(), &["refs", "FEAT-PD-100"]);
    assert!(refs.contains("REQ-PD-001"), "{refs}");
}

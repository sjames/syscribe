//! `REQ-TRS-FMED-004`: an abstract feature behaves as in a proper feature model. It is a grouping
//! feature with no realisation of its own, so it does not distinguish products (variants are counted
//! over concrete features), a configuration need not name it, nothing can usefully be conditioned on
//! it, and one that groups nothing is not abstract. Each test builds the same model twice, with and
//! without `isAbstract`, so the difference is the semantics and nothing else.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::feature_model::{analysis_json, check_feature_model, check_feature_model_deep, configure_selection, enumerate_variants, EnumOutcome};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// Root (mandatory) > Opt (optional; abstract when `abstract_opt`) > X, Y (optional); Z (optional, requires Opt).
/// Configuration CONF-X selects Root and X only: it does not name Opt.
fn model(abstract_opt: bool) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-abs-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    let abs = if abstract_opt { "isAbstract: true\n" } else { "" };
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "F/_index.md", "---\ntype: Package\nname: F\n---\n");
    write(&r, "F/Root.md", "---\ntype: FeatureDef\nid: FEAT-ROOT\nname: Root\nmandatory: true\n---\n");
    write(&r, "F/Root/Opt.md", &format!("---\ntype: FeatureDef\nid: FEAT-OPT\nname: Opt\n{abs}---\n"));
    write(&r, "F/Root/Opt/X.md", "---\ntype: FeatureDef\nid: FEAT-XX\nname: X\n---\n");
    write(&r, "F/Root/Opt/Y.md", "---\ntype: FeatureDef\nid: FEAT-YY\nname: Y\n---\n");
    write(&r, "F/Root/Z.md", "---\ntype: FeatureDef\nid: FEAT-ZZ\nname: Z\nrequires: [FEAT-OPT]\n---\n");
    write(
        &r,
        "C/CONF-X-001.md",
        "---\ntype: Configuration\nid: CONF-X-001\nname: X\nstatus: draft\nfeatureModel: F\nfeatures:\n  F::Root: true\n  F::Root::Opt::X: true\n---\n",
    );
    r
}

#[test]
fn products_are_counted_over_concrete_features_only() {
    // Without abstract: Opt off allows only the empty choice below it (X, Y, Z all off); Opt on allows any of X, Y and Z: 1 + 4 * 2 = 9 models.
    let concrete = configure_selection(&walk_model(&model(false)).unwrap(), &BTreeMap::new());
    let abs = configure_selection(&walk_model(&model(true)).unwrap(), &BTreeMap::new());
    let (n_concrete, n_abs) = (concrete["products"]["count"].as_u64().unwrap(), abs["products"]["count"].as_u64().unwrap());
    assert!(n_abs < n_concrete, "Opt on and off with nothing chosen below it is one product: {n_abs} vs {n_concrete}");
    // With Opt abstract the products are the choices over X, Y, Z: eight, the empty one counted once.
    assert_eq!(n_abs, 8, "{abs}");
    assert_eq!(n_concrete, 9, "{concrete}");
}

#[test]
fn enumerated_variants_list_concrete_features_and_do_not_repeat() {
    let EnumOutcome::Variants { configs, truncated } = enumerate_variants(&walk_model(&model(true)).unwrap(), 1000) else { panic!("variants") };
    assert!(!truncated);
    assert_eq!(configs.len(), 8);
    assert!(configs.iter().all(|c| !c.iter().any(|f| f == "F::Root::Opt")), "an abstract feature is not part of a product: {configs:?}");
    let EnumOutcome::Variants { configs, .. } = enumerate_variants(&walk_model(&model(false)).unwrap(), 1000) else { panic!("variants") };
    assert_eq!(configs.len(), 9);
}

#[test]
fn a_configuration_need_not_name_an_abstract_feature() {
    // CONF-X selects X but says nothing of Opt. X needs Opt: invalid if Opt is a concrete feature
    // the configuration leaves out, valid if Opt is abstract and so completed.
    let concrete = analysis_json(&walk_model(&model(false)).unwrap());
    assert_eq!(concrete["invalidConfigurations"], serde_json::json!(["CONF-X-001"]), "{concrete}");
    let abs = analysis_json(&walk_model(&model(true)).unwrap());
    assert_eq!(abs["invalidConfigurations"], serde_json::json!([]), "{abs}");
    let deep = check_feature_model_deep(&walk_model(&model(true)).unwrap());
    assert!(deep.invalid_configs.is_empty() && deep.findings.iter().all(|f| f.code != "E225"), "{:?}", deep.findings.iter().map(|f| &f.message).collect::<Vec<_>>());
    let deep = check_feature_model_deep(&walk_model(&model(false)).unwrap());
    assert_eq!(deep.invalid_configs, vec!["CONF-X-001".to_string()]);
}

#[test]
fn a_configuration_that_names_an_abstract_feature_is_an_error_whichever_way() {
    let r = model(true);
    write(&r, "C/CONF-ABS-001.md", "---\ntype: Configuration\nid: CONF-ABS-001\nname: A\nstatus: draft\nfeatureModel: F\nfeatures:\n  F::Root: true\n  F::Root::Opt: true\n  F::Root::Opt::X: true\n---\n");
    write(&r, "C/CONF-OFF-001.md", "---\ntype: Configuration\nid: CONF-OFF-001\nname: O\nstatus: draft\nfeatureModel: F\nfeatures:\n  F::Root: true\n  FEAT-OPT: false\n---\n");
    let errs: Vec<(String, String)> = validate(&walk_model(&r).unwrap()).findings.into_iter().filter(|f| f.code == "E238").map(|f| (f.file.clone(), f.message.clone())).collect();
    assert_eq!(errs.len(), 2, "{errs:?}");
    assert!(errs.iter().any(|(f, m)| f.ends_with("CONF-ABS-001.md") && m.contains("selects the abstract feature 'F::Root::Opt'")), "{errs:?}");
    assert!(errs.iter().any(|(f, m)| f.ends_with("CONF-OFF-001.md") && m.contains("deselects")), "an id key counts too: {errs:?}");
    // The configuration that leaves the abstract feature out is fine, and so is the same name on a concrete feature.
    assert!(!errs.iter().any(|(f, _)| f.ends_with("CONF-X-001.md")));
    let concrete = model(false);
    write(&concrete, "C/CONF-ABS-001.md", "---\ntype: Configuration\nid: CONF-ABS-001\nname: A\nstatus: draft\nfeatureModel: F\nfeatures:\n  F::Root: true\n  F::Root::Opt: true\n---\n");
    assert!(!validate(&walk_model(&concrete).unwrap()).findings.iter().any(|f| f.code == "E238"));
}

#[test]
fn a_configuration_can_still_be_wrong_about_its_concrete_features() {
    // Selecting Z (which requires Opt) is fine, but a configuration that deselects Root breaks the model whatever Opt is.
    let r = model(true);
    write(&r, "C/CONF-BAD-001.md", "---\ntype: Configuration\nid: CONF-BAD-001\nname: B\nstatus: draft\nfeatureModel: F\nfeatures:\n  F::Root: false\n  F::Root::Opt::X: true\n---\n");
    let a = analysis_json(&walk_model(&r).unwrap());
    assert_eq!(a["invalidConfigurations"], serde_json::json!(["CONF-BAD-001"]), "{a}");
}

#[test]
fn requiring_an_abstract_feature_is_met_by_what_entails_it_and_not_otherwise() {
    let findings = |abs: bool| {
        let r = model(abs);
        write(&r, "C/CONF-Z-001.md", "---\ntype: Configuration\nid: CONF-Z-001\nname: Z\nstatus: draft\nfeatureModel: F\nfeatures:\n  F::Root: true\n  F::Root::Z: true\n---\n");
        check_feature_model(&walk_model(&r).unwrap())
    };
    assert!(findings(false).iter().any(|f| f.code == "E219" && f.message.contains("F::Root::Opt")), "a concrete Opt must be named");
    assert!(!findings(true).iter().any(|f| f.code == "E219"), "an abstract Opt is completed: {:?}", findings(true).iter().map(|f| &f.message).collect::<Vec<_>>());
}

/// The effective value of the abstract feature `F::Root::Opt` in the configuration with this id.
fn opt_in(elements: &[syscribe_model::element::RawElement], conf: &str) -> bool {
    let c = elements.iter().find(|e| e.frontmatter.id.as_deref() == Some(conf)).unwrap();
    syscribe_model::projection::canonical_selection(elements, c).get("F::Root::Opt").copied().unwrap_or(false)
}

#[test]
fn an_abstract_feature_is_on_exactly_when_the_concrete_selection_entails_it() {
    let r = model(true);
    write(&r, "C/CONF-NONE-001.md", "---\ntype: Configuration\nid: CONF-NONE-001\nname: N\nstatus: draft\nfeatureModel: F\nfeatures:\n  F::Root: true\n---\n");
    write(&r, "C/CONF-Z-001.md", "---\ntype: Configuration\nid: CONF-Z-001\nname: Z\nstatus: draft\nfeatureModel: F\nfeatures:\n  F::Root: true\n  F::Root::Z: true\n---\n");
    write(&r, "C/CONF-LIE-001.md", "---\ntype: Configuration\nid: CONF-LIE-001\nname: L\nstatus: draft\nfeatureModel: F\nfeatures:\n  F::Root: true\n  F::Root::Opt: true\n---\n");
    let els = walk_model(&r).unwrap();
    assert!(opt_in(&els, "CONF-X-001"), "a selected child forces its abstract parent on");
    assert!(opt_in(&els, "CONF-Z-001"), "so does a `requires:` that targets it");
    assert!(!opt_in(&els, "CONF-NONE-001"), "nothing forces it: it is off");
    assert!(!opt_in(&els, "CONF-LIE-001"), "an explicit entry is an error (E238), not input: the derived value stands");
    // Concrete features are untouched.
    let x = els.iter().find(|e| e.frontmatter.id.as_deref() == Some("CONF-X-001")).unwrap();
    let sel = syscribe_model::projection::canonical_selection(&els, x);
    assert_eq!(sel.get("F::Root::Opt::X"), Some(&true));
    assert_eq!(sel.get("F::Root::Opt::Y").copied().unwrap_or(false), false);
}

#[test]
fn a_mandatory_abstract_group_is_on_even_with_nothing_chosen_below_it() {
    let r = tempdir_model();
    let els = walk_model(&r).unwrap();
    let c = els.iter().find(|e| e.frontmatter.id.as_deref() == Some("CONF-M-001")).unwrap();
    let sel = syscribe_model::projection::canonical_selection(&els, c);
    assert_eq!(sel.get("F::Root::Platform"), Some(&true), "the model forces it on: {sel:?}");
}

fn tempdir_model() -> PathBuf {
    let r = model(true);
    write(&r, "F/Root/Platform.md", "---\ntype: FeatureDef\nid: FEAT-PLATFORM\nname: Platform\nmandatory: true\nisAbstract: true\n---\n");
    write(&r, "F/Root/Platform/Soc.md", "---\ntype: FeatureDef\nid: FEAT-SOC\nname: Soc\n---\n");
    write(&r, "C/CONF-M-001.md", "---\ntype: Configuration\nid: CONF-M-001\nname: M\nstatus: draft\nfeatureModel: F\nfeatures:\n  F::Root: true\n---\n");
    r
}

#[test]
fn an_element_conditioned_on_an_abstract_feature_is_active_where_it_is_entailed() {
    use syscribe_model::projection::{is_active_canon, canonical_selection};
    let r = model(true);
    write(&r, "Parts/Gated.md", "---\ntype: PartDef\nname: Gated\nappliesWhen: F::Root::Opt\n---\n");
    write(&r, "C/CONF-NONE-001.md", "---\ntype: Configuration\nid: CONF-NONE-001\nname: N\nstatus: draft\nfeatureModel: F\nfeatures:\n  F::Root: true\n---\n");
    let els = walk_model(&r).unwrap();
    let gated = els.iter().find(|e| e.qualified_name == "Parts::Gated").unwrap();
    let active = |conf: &str| {
        let c = els.iter().find(|e| e.frontmatter.id.as_deref() == Some(conf)).unwrap();
        let sel = canonical_selection(&els, c);
        is_active_canon(gated, &sel, &syscribe_model::variability::package_conditions(&els), &syscribe_model::variability::feature_id_to_qname(&els))
    };
    assert!(active("CONF-X-001"), "Opt follows from X, so the element is in the product");
    assert!(!active("CONF-NONE-001"), "nothing under Opt is chosen, so it is not");
    // No warning for gating on one: it is meaningful now.
    assert!(!check_feature_model(&els).iter().any(|f| f.code == "W238"));
}

#[test]
fn an_abstract_feature_that_groups_nothing_is_flagged() {
    let r = model(true);
    write(&r, "F/Root/Lonely.md", "---\ntype: FeatureDef\nid: FEAT-LONELY\nname: Lonely\nisAbstract: true\n---\n");
    let w: Vec<String> = check_feature_model(&walk_model(&r).unwrap()).into_iter().filter(|f| f.code == "W239").map(|f| f.message).collect();
    assert_eq!(w.len(), 1, "Opt has children, Lonely has none: {w:?}");
    assert!(w[0].contains("Lonely"));
}

#[test]
fn abstract_features_stay_in_the_analysis_and_remain_selectable() {
    let a = analysis_json(&walk_model(&model(true)).unwrap());
    assert_eq!(a["features"]["F::Root::Opt"]["state"], "normal", "{a}");
    let c = configure_selection(&walk_model(&model(true)).unwrap(), &BTreeMap::from([("F::Root::Opt".to_string(), true)]));
    assert_eq!(c["satisfiable"], true);
    assert_eq!(c["features"]["F::Root::Opt"]["state"], "selected", "an abstract feature can still be chosen: {c}");
}

#[test]
fn the_load_list_does_not_offer_derived_abstract_values_as_choices() {
    let els = walk_model(&model(true)).unwrap();
    let list = syscribe_model::feature_model::configurations_json(&els);
    let sel = &list[0]["selection"];
    assert_eq!(sel["F::Root::Opt::X"], true);
    assert!(sel.get("F::Root::Opt").is_none(), "{sel}");
}

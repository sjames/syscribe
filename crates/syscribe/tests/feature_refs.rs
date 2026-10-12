//! GH #325: a feature is named by a dotted tree path inside its `featureTree:`
//! sheet, by qname or `FEAT-` id elsewhere; `requires:`/`excludes:` on a tree
//! entry accept the dotted path, and every unresolved reference says which form
//! the field takes and what it most likely meant.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const TREE: &str = "---\ntype: FeatureModel\nname: Features\nfeatureTree:\n  - {name: Protection, id: FEAT-FR-PROT, mandatory: true, groupKind: optional}\n  - {name: Protection.Afci, id: FEAT-FR-AFCI, groupKind: optional}\n  - name: Protection.Rsd\n    id: FEAT-FR-RSD\n    groupKind: optional\n    REFS\n  - {name: Mppt, id: FEAT-FR-MPPT, mandatory: true, groupKind: optional}\n  - name: Mppt.GlobalScan\n    id: FEAT-FR-GSCAN\n    groupKind: optional\n    parameters:\n      - {name: scanPeriodS, type: ScalarValues::Integer, default: 600, isRequired: true, bindingTime: load}\n---\n";

fn model(refs: &str, features: &[&str], bindings: &[&str], applies_when: &str, body: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-fr-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&r);
    std::fs::create_dir_all(r.join("Features")).unwrap();
    let w = |n: &str, c: &str| std::fs::write(r.join(n), c).unwrap();
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/_index.md", &TREE.replace("REFS", refs));
    let feats: String = features.iter().map(|f| format!("  {f}: true\n")).collect();
    let binds: String = bindings.iter().map(|b| format!("  {b}: 300\n")).collect();
    w("CONF-FR-001.md", &format!("---\ntype: Configuration\nid: CONF-FR-001\nname: c\nstatus: draft\nfeatureModel: Features\nfeatures:\n{feats}parameterBindings:\n{binds}---\n"));
    w("REQ-FR-001.md", &format!("---\ntype: Requirement\nid: REQ-FR-001\nname: r\nstatus: draft\nreqDomain: software\nappliesWhen: {applies_when}\n---\n\nThe inverter shall {body}.\n"));
    r
}

const ALL: &[&str] = &["Features::Protection", "Features::Protection::Afci", "Features::Protection::Rsd", "Features::Mppt", "Features::Mppt::GlobalScan"];
const GOOD_BIND: &[&str] = &["Features::Mppt::GlobalScan.scanPeriodS"];

fn run(root: &PathBuf, args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn line<'a>(out: &'a str, code: &str) -> &'a str {
    out.lines().find(|l| l.starts_with(&format!("| {code} |"))).unwrap_or_else(|| panic!("no {code} in:\n{out}"))
}

#[test]
fn a_dotted_tree_path_resolves_in_requires_and_excludes() {
    let root = model("requires: [Protection.Afci]\n    excludes: [Mppt.GlobalScan]", ALL, GOOD_BIND, "Features::Protection::Afci", "trip");
    let out = run(&root, &["feature-check"]);
    assert!(!out.contains("E212"), "{out}");
    // The edge is real, not dropped: Rsd excludes GlobalScan, and the config selects both.
    assert!(out.contains("E220"), "{out}");
}

#[test]
fn qname_and_feat_id_forms_still_resolve() {
    let root = model("requires: [Features::Protection::Afci, FEAT-FR-AFCI]", ALL, GOOD_BIND, "FEAT-FR-AFCI", "trip");
    let out = run(&root, &["feature-check"]);
    assert!(!out.contains("E212") && !out.contains("E219"), "{out}");
    assert!(!run(&root, &["validate"]).contains("| E"), "a clean model has no errors");
}

#[test]
fn unresolved_requires_names_the_forms_and_closest_match() {
    let root = model("requires: [Protection.Afcy]", ALL, GOOD_BIND, "Features::Protection::Afci", "trip");
    let l = line(&run(&root, &["feature-check"]), "E212").to_string();
    assert!(l.contains("dotted tree path") && l.contains("FEAT-"), "{l}");
    assert!(l.contains("did you mean 'Features::Protection::Afci'"), "{l}");
}

#[test]
fn unresolved_applies_when_names_the_forms_and_closest_match() {
    let root = model("", ALL, GOOD_BIND, "Features::Protection.Afci", "trip");
    let l = line(&run(&root, &["validate"]), "E209").to_string();
    assert!(l.contains("qualified name") && l.contains("FEAT-"), "{l}");
    assert!(l.contains("did you mean 'Features::Protection::Afci'"), "{l}");
}

#[test]
fn unresolved_configuration_feature_is_reported_with_closest_match() {
    let feats = ["Features::Protection", "Features::Protection::Afci", "Protection.Rsd", "Features::Mppt", "Features::Mppt::GlobalScan"];
    let root = model("", &feats, GOOD_BIND, "Features::Protection::Afci", "trip");
    let l = line(&run(&root, &["validate"]), "E239").to_string();
    assert!(l.contains("'Protection.Rsd'") && l.contains("FEAT-"), "{l}");
    assert!(l.contains("did you mean 'Features::Protection::Rsd'"), "{l}");
}

#[test]
fn feat_id_key_in_configuration_is_not_e239() {
    let feats = ["Features::Protection", "FEAT-FR-AFCI", "Features::Mppt", "Features::Mppt::GlobalScan"];
    let root = model("", &feats, GOOD_BIND, "Features::Protection::Afci", "trip");
    assert!(!run(&root, &["validate"]).contains("E239"));
}

#[test]
fn unresolved_parameter_binding_names_the_form_and_the_path_it_spells() {
    let root = model("", ALL, &["Features::Mppt.GlobalScan.scanPeriodS"], "Features::Protection::Afci", "trip");
    let l = line(&run(&root, &["validate"]), "E222").to_string();
    assert!(l.contains("`.` only before the parameter"), "{l}");
    assert!(l.contains("did you mean 'Features::Mppt::GlobalScan.scanPeriodS'"), "{l}");
}

#[test]
fn misspelt_parameter_binding_suggests_the_declared_parameter() {
    let root = model("", ALL, &["Features::Mppt::GlobalScan.scanPeriod"], "Features::Protection::Afci", "trip");
    let l = line(&run(&root, &["validate"]), "E222").to_string();
    assert!(l.contains("did you mean 'Features::Mppt::GlobalScan.scanPeriodS'"), "{l}");
}

#[test]
fn malformed_placeholder_is_reported_with_the_path_it_spells() {
    let root = model("", ALL, GOOD_BIND, "Features::Protection::Afci", "scan every {{Features::Mppt.GlobalScan.scanPeriodS}} s");
    let l = line(&run(&root, &["validate"]), "E241").to_string();
    assert!(l.contains("`.` only before the parameter"), "{l}");
    assert!(l.contains("did you mean '{{Features::Mppt::GlobalScan.scanPeriodS}}'"), "{l}");
}

#[test]
fn unresolved_placeholder_feature_names_the_form() {
    let root = model("", ALL, GOOD_BIND, "Features::Protection::Afci", "scan every {{Features::Mppt::GlobalScn.scanPeriodS}} s");
    let l = line(&run(&root, &["validate"]), "E241").to_string();
    assert!(l.contains("{{<FeatureDef qualified name or FEAT- id>.<parameter>}}"), "{l}");
    assert!(l.contains("did you mean '{{Features::Mppt::GlobalScan.scanPeriodS}}'"), "{l}");
}

#[test]
fn a_valid_placeholder_and_template_text_are_not_reported() {
    let root = model("", ALL, GOOD_BIND, "Features::Protection::Afci", "scan every {{Features::Mppt::GlobalScan.scanPeriodS}} s, see {{ user.name }} and `{{Features::Mppt.GlobalScan.scanPeriodS}}`");
    assert!(!run(&root, &["validate"]).contains("E241"));
}

#[test]
fn a_bare_name_outside_the_sheet_keeps_its_meaning() {
    // A model-root FeatureDef `Wdt` named from inside the `Features` sheet still
    // resolves to it; the sheet-relative rewrite only applies to the sheet's own entries.
    let root = model("requires: [Wdt]", ALL, GOOD_BIND, "Features::Protection::Afci", "trip");
    std::fs::write(root.join("Wdt.md"), "---\ntype: FeatureDef\nid: FEAT-FR-WDT\nname: Wdt\n---\n").unwrap();
    let out = run(&root, &["feature-check"]);
    assert!(!out.contains("E212"), "{out}");
    assert!(out.contains("E219"), "Rsd is selected without the root Wdt it requires: {out}");
}

#[test]
fn a_leaf_name_key_in_configuration_is_e239_and_suggests_the_qname() {
    let feats = ["Features::Protection", "Afci", "Features::Mppt", "Features::Mppt::GlobalScan"];
    let root = model("", &feats, GOOD_BIND, "Features::Protection::Afci", "trip");
    let l = line(&run(&root, &["validate"]), "E239").to_string();
    assert!(l.contains("did you mean 'Features::Protection::Afci'"), "{l}");
}

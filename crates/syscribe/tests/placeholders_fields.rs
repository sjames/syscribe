//! TC-TRS-PHOLDFIELD-001 / GH #268: whole-value placeholders in typed frontmatter fields.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// `asil_default` / `rate` are the parameter values of CONF-PF-ALPHA-001 (B, 1e-7); BRAVO binds D / 0.5.
fn model(alpha: (&str, &str), field_lines: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-phf-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w(
        "Features/Safety.md",
        "---\ntype: FeatureDef\nid: FEAT-PF-100\nname: Safety\ngroupKind: optional\nparameters:\n  - {name: level, type: ScalarValues::String}\n  - {name: rate, type: ScalarValues::Real}\n---\n\nS.\n",
    );
    for (id, level, rate) in [("CONF-PF-ALPHA-001", alpha.0, alpha.1), ("CONF-PF-BRAVO-001", "D", "0.5")] {
        w(
            &format!("Configs/{id}.md"),
            &format!("---\nid: {id}\ntype: Configuration\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Safety: true\nparameterBindings:\n  Features::Safety.level: {level}\n  Features::Safety.rate: {rate}\n---\n\nC.\n"),
        );
    }
    w(
        "Reqs/REQ-PF-001.md",
        &format!("---\nid: REQ-PF-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\nappliesWhen: Features::Safety\n{field_lines}---\n\nShall.\n"),
    );
    r
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

/// The element as `list Requirement --json` reports it, under a `--config` lens when given.
fn show(root: &Path, conf: Option<&str>) -> serde_json::Value {
    let mut a = vec!["list", "Requirement", "--json"];
    if let Some(c) = conf {
        a.extend(["--config", c]);
    }
    let (o, c) = run(root, &a);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"));
    let rows = v.as_array().cloned().or_else(|| v["elements"].as_array().cloned()).unwrap_or_else(|| panic!("{o}"));
    rows.into_iter().find(|r| r["id"] == "REQ-PF-001").unwrap_or_else(|| panic!("REQ-PF-001 in {o}"))
}

const FIELDS: &str = "asilLevel: \"{{Features::Safety.level}}\"\nsilLevel: \"{{Features::Safety.rate}}\"\n";

#[test]
fn the_base_model_parses_and_validates_with_the_fields_unset() {
    let r = model(("B", "1"), "asilLevel: \"{{Features::Safety.level}}\"\n");
    let (o, c) = run(&r, &["validate"]);
    assert_eq!(c, 0, "no type or enum error for the placeholder: {o}");
    assert!(show(&r, None)["asilLevel"].is_null(), "unset without a configuration");
}

#[test]
fn each_configuration_sees_its_own_typed_value() {
    let r = model(("B", "1"), "asilLevel: \"{{Features::Safety.level}}\"\n");
    assert_eq!(show(&r, Some("CONF-PF-ALPHA-001"))["asilLevel"], "B");
    assert_eq!(show(&r, Some("CONF-PF-BRAVO-001"))["asilLevel"], "D");
    let r2 = model(("B", "1"), "silLevel: \"{{Features::Safety.rate}}\"\n");
    assert_eq!(show(&r2, Some("CONF-PF-ALPHA-001"))["silLevel"], 1, "an integer field");
}

#[test]
fn enumerated_values_are_normalised_to_upper_case() {
    let r = model(("b", "1"), "asilLevel: \"{{Features::Safety.level}}\"\n");
    assert_eq!(show(&r, Some("CONF-PF-ALPHA-001"))["asilLevel"], "B");
}

#[test]
fn an_out_of_domain_value_is_e247_and_leaves_the_field_unset() {
    let r = model(("Z", "9"), FIELDS);
    let (o, c) = run(&r, &["validate", "--config", "CONF-PF-ALPHA-001"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("E247") && o.contains("asilLevel") && o.contains("CONF-PF-ALPHA-001"), "{o}");
    assert!(o.contains("silLevel"), "9 is not a SIL: {o}");
    assert!(show(&r, Some("CONF-PF-ALPHA-001"))["asilLevel"].is_null());
    // BRAVO binds the valid level D (its 0.5 is still not a SIL): only the SIL field is reported there
    let (b, _) = run(&r, &["validate", "--config", "CONF-PF-BRAVO-001"]);
    assert!(b.lines().any(|l| l.contains("E247") && l.contains("silLevel") && l.contains("CONF-PF-BRAVO-001")), "{b}");
    assert!(!b.lines().any(|l| l.contains("field 'asilLevel'") && l.contains("CONF-PF-BRAVO-001")), "BRAVO's level D is valid: {b}");
}

#[test]
fn the_text_placeholder_rules_apply_to_field_placeholders() {
    let r = model(("B", "1"), "asilLevel: \"{{Features::Nope.level}}\"\n");
    let (o, _) = run(&r, &["validate"]);
    assert!(o.contains("E241"), "unknown feature reference: {o}");
}

#[test]
fn a_field_outside_the_list_keeps_its_ordinary_behaviour() {
    let r = model(("B", "1"), "displayOrder: \"{{Features::Safety.rate}}\"\n");
    let (o, c) = run(&r, &["validate"]);
    // displayOrder is not placeholder-capable: the string is not a number, exactly as before
    assert!(c != 0 || o.contains("displayOrder") || o.contains("W047") || o.contains("E00"), "unchanged diagnostics: {o}");
}

#[test]
fn a_mixed_or_unit_suffixed_value_is_not_a_whole_value_placeholder() {
    let r = model(("B", "1"), "asilLevel: \"ASIL {{Features::Safety.level}}\"\n");
    let (o, c) = run(&r, &["validate"]);
    assert!(c != 0, "the mixed string is an invalid asilLevel as before: {o}");
}

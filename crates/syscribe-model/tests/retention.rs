//! `REQ-TRS-BT-031..034`: variant retention at a binding stage, the dependents that rely on a
//! retained element's absence, and the late-bound safety rule — over the qualification
//! fixtures. Includes the compatibility property: an unstaged Configuration is projected
//! exactly as the original two-valued gate does.

use std::collections::BTreeSet;
use std::path::PathBuf;

use syscribe_model::element::RawElement;
use syscribe_model::projection::{canonical_selection, is_active_canon, project_raw, validate_projected};
use syscribe_model::retention::report;
use syscribe_model::validator::validate;
use syscribe_model::variability;
use syscribe_model::walker::walk_model;

fn load(tc: &str, name: &str) -> Vec<RawElement> {
    walk_model(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../qual/fixtures").join(format!("TC-TRS-BT-{tc}")).join(name)).unwrap()
}

fn cfg(els: &[RawElement], id: &str) -> usize {
    els.iter().position(|e| e.frontmatter.id.as_deref() == Some(id)).unwrap()
}

fn ids(v: &[RawElement]) -> BTreeSet<String> {
    v.iter().filter(|e| e.frontmatter.element_type.as_ref().is_some_and(|t| format!("{t:?}") == "Requirement")).filter_map(|e| e.frontmatter.id.clone()).collect()
}

#[test]
fn a_staged_projection_keeps_and_marks_what_the_open_feature_decides() {
    let els = load("031", "stage");
    let sel = canonical_selection(&els, &els[cfg(&els, "CONF-BT-A1")]);
    let view = project_raw(&els, &sel);
    assert_eq!(ids(&view), ["REQ-BASE-001", "REQ-NOSENSOR-001", "REQ-SENSOR-001"].map(String::from).into());
    let marked: BTreeSet<_> = view.iter().filter(|e| e.frontmatter.variant_retained).filter_map(|e| e.frontmatter.id.clone()).collect();
    assert!(marked.contains("REQ-SENSOR-001") && marked.contains("REQ-NOSENSOR-001") && !marked.contains("REQ-BASE-001"), "{marked:?}");

    // Decided at runtime: nothing is retained, one side is dropped.
    for (conf, kept) in [("CONF-BT-R0", "REQ-NOSENSOR-001"), ("CONF-BT-R1", "REQ-SENSOR-001")] {
        let v = project_raw(&els, &canonical_selection(&els, &els[cfg(&els, conf)]));
        assert_eq!(ids(&v), ["REQ-BASE-001", kept].map(String::from).into(), "{conf}");
        assert!(v.iter().all(|e| !e.frontmatter.variant_retained), "{conf}");
    }
}

#[test]
fn the_report_lists_retained_elements_and_what_relies_on_their_absence() {
    let els = load("031", "stage");
    let r = report(&els, cfg(&els, "CONF-BT-A1"));
    let got: Vec<_> = r.retained.iter().map(|x| x.id.clone().unwrap()).collect();
    assert_eq!(got, ["REQ-NOSENSOR-001", "REQ-SENSOR-001", "TC-NOSENSOR-001"]);
    assert!(r.retained.iter().all(|x| x.because == ["Features::Sensor"]));
    assert_eq!(r.reliances.len(), 1, "{:?}", r.reliances);
    assert_eq!((r.reliances[0].dependent.as_str(), r.reliances[0].retained.as_str()), ("Tests::TC-NOSENSOR-001", "Req::REQ-SENSOR-001"));
    assert!(report(&els, cfg(&els, "CONF-BT-R0")).retained.is_empty());

    // validate_projected lists exactly the same thing.
    let sel = canonical_selection(&els, &els[cfg(&els, "CONF-BT-A1")]);
    let f = validate_projected(&els, &Default::default(), &sel);
    assert_eq!(f.iter().filter(|x| x.code == "I750").count(), r.retained.len());
    assert_eq!(f.iter().filter(|x| x.code == "W752").count(), r.reliances.len());
}

#[test]
fn unstaged_projection_is_exactly_the_two_valued_gate() {
    let els = load("033", "unstaged");
    for conf in ["CONF-BT-A1", "CONF-BT-R0", "CONF-BT-R1"] {
        let i = cfg(&els, conf);
        let sel = canonical_selection(&els, &els[i]);
        let view = project_raw(&els, &sel);
        let (pkg, alias) = (variability::package_conditions(&els), variability::feature_id_to_qname(&els));
        let plain: BTreeSet<_> = els.iter().filter(|e| is_active_canon(e, &sel, &pkg, &alias)).map(|e| e.qualified_name.clone()).collect();
        let got: BTreeSet<_> = view.iter().map(|e| e.qualified_name.clone()).collect();
        // The projection also drops allocation-gated elements and unbound TestPlans; none exist here.
        assert_eq!(got, plain, "{conf}");
        assert!(view.iter().all(|e| !e.frontmatter.variant_retained) && report(&els, i).retained.is_empty(), "{conf}");
    }
}

#[test]
fn late_bound_safety_findings_follow_the_covers_declarations() {
    let codes = |name: &str| -> Vec<(String, String)> {
        validate(&load("034", name)).findings.into_iter().filter(|f| f.code == "W753" || f.code == "W754").map(|f| (f.code.to_string(), f.file)).collect()
    };
    let unc = codes("uncovered");
    assert_eq!(unc.iter().filter(|(c, _)| c == "W753").count(), 2, "{unc:?}");
    assert!(unc.iter().any(|(_, f)| f.ends_with("REQ-AKV-001.md")) && unc.iter().any(|(_, f)| f.ends_with("REQ-ASENS-001.md")));
    let cov = codes("covered");
    assert_eq!(cov.iter().filter(|(c, _)| c == "W753").count(), 0, "{cov:?}");
    assert_eq!(cov.iter().filter(|(c, _)| c == "W754").count(), 1, "the typo'd entry");
    assert!(codes("nowindows").is_empty());
}

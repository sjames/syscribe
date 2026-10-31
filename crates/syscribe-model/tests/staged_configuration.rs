//! `REQ-TRS-BT-011..015`, `021`: staged Configurations and value provenance. Drives the
//! qualification fixtures (`qual/fixtures/TC-TRS-BT-0xx`) through the library, so the
//! rules are pinned by `cargo test` as well as by `qual/tests/run_qual.sh`.

use std::path::PathBuf;

use syscribe_model::provenance::{provenance, Source};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn fixture(tc: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../qual/fixtures").join(format!("TC-TRS-BT-{tc}")).join(name)
}

/// (code, message) of every finding with a staged-configuration code, or `W017`/`W027`.
fn staged(tc: &str, name: &str) -> Vec<(String, String)> {
    let els = walk_model(&fixture(tc, name)).unwrap();
    validate(&els)
        .findings
        .into_iter()
        .filter(|f| (f.code.starts_with("E75") || f.code == "W017" || f.code == "W027" || f.code == "E205" || f.code == "E206") && f.code != "E750")
        .map(|f| (f.code.to_string(), f.message))
        .collect()
}

fn count(f: &[(String, String)], code: &str) -> usize {
    f.iter().filter(|(c, _)| c == code).count()
}

#[test]
fn stage_validity_window_membership_and_monotonic_chain() {
    assert_eq!(count(&staged("011", "unknown"), "E751"), 1);
    assert!(staged("011", "valid").is_empty());
    assert_eq!(count(&staged("011", "unstaged"), "W027"), 1, "unstaged keeps W027");

    let early = staged("012", "early");
    assert_eq!(count(&early, "E752"), 2, "{early:?}");
    assert!(staged("012", "inwindow").is_empty() && staged("012", "runtime_ok").is_empty());
    assert_eq!(count(&staged("012", "runtime_early"), "E752"), 1);

    assert_eq!(count(&staged("013", "earlier"), "E753"), 1);
    assert!(staged("013", "later").is_empty());
}

#[test]
fn closed_values_and_narrowing() {
    assert_eq!(count(&staged("014", "rebind"), "E754"), 1);
    assert_eq!(count(&staged("014", "featclosed"), "E754"), 1);
    assert!(staged("014", "same_value").is_empty() && staged("014", "narrow_ok").is_empty() && staged("014", "enum_ok").is_empty());

    assert_eq!(count(&staged("014", "widen"), "E755"), 1);
    assert_eq!(count(&staged("014", "beyond_declared"), "E755"), 1);
    assert_eq!(count(&staged("014", "unnamed"), "E755"), 1);

    let outside = staged("014", "outside");
    assert_eq!((count(&outside, "E756"), count(&outside, "E205")), (1, 0), "{outside:?}: a narrowed-range miss is E756, not also E205");
    assert_eq!(count(&staged("014", "enum_outside"), "E756"), 1);
}

#[test]
fn closed_window_unbound_is_e757_instead_of_w017() {
    let closed = staged("015", "closed");
    assert_eq!(count(&closed, "E757"), 1, "{closed:?}");
    assert!(closed.iter().all(|(c, m)| !(c == "W017" && m.contains("reqearly"))), "{closed:?}");
    assert_eq!(count(&staged("015", "open"), "E757"), 0);
    let unstaged = staged("015", "unstaged");
    assert_eq!((count(&unstaged, "E757"), count(&unstaged, "W017")), (0, 2), "{unstaged:?}");
}

#[test]
fn provenance_reports_source_and_stage_of_each_value() {
    let els = walk_model(&fixture("021", "chain")).unwrap();
    let b = els.iter().position(|e| e.frontmatter.id.as_deref() == Some("CONF-BT-B1")).unwrap();
    let p = provenance(&els, b);
    assert_eq!(p.stage.as_deref(), Some("load"));
    let param = |k: &str| p.parameters.iter().find(|q| q.key == k).unwrap_or_else(|| panic!("no {k} in {:?}", p.parameters));

    let kv = param("Features::Motor.kv");
    assert_eq!((kv.source.clone(), kv.stage.as_deref()), (Source::Inherited { from: "CONF-BT-A1".into() }, Some("compile")));
    let spin = param("Features::Motor.spin");
    assert_eq!((spin.source.clone(), spin.stage.as_deref()), (Source::Own, Some("load")));
    assert_eq!(param("Features::Motor.lim").source, Source::Fixed);
    assert_eq!(param("Features::Motor.dflt").source, Source::Default);
    assert!(matches!(param("Features::Motor.ratio").source, Source::Derived { .. }));
    assert_eq!(param("Features::Comp.kv").source, Source::BindTo { from: "Features::Motor.kv".into() });
    assert_eq!(param("Features::Comp.kv").value, kv.value, "a propagated value is the system parameter's");

    let sel = |f: &str| p.selections.iter().find(|s| s.feature == f).unwrap();
    assert_eq!(sel("Features::Motor").source, Source::Inherited { from: "CONF-BT-A1".into() });
    assert_eq!(sel("Features::Sensor").source, Source::Own);
}

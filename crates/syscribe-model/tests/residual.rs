//! `REQ-TRS-BT-016..020`: residual variability, three-valued constraints, the staged-chain
//! completion check and the later-window `W017` rule, over the qualification fixtures.

use std::path::PathBuf;

use syscribe_model::feature_model::{check_feature_model, check_feature_model_deep, Tri};
use syscribe_model::residual::residual;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn load(tc: &str, name: &str) -> Vec<syscribe_model::element::RawElement> {
    walk_model(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../qual/fixtures").join(format!("TC-TRS-BT-{tc}")).join(name)).unwrap()
}

fn cfg(els: &[syscribe_model::element::RawElement], id: &str) -> usize {
    els.iter().position(|e| e.frontmatter.id.as_deref() == Some(id)).unwrap()
}

#[test]
fn residual_groups_open_items_by_next_stage_and_lists_closed_apart() {
    let els = load("016", "chain");
    let r = residual(&els, cfg(&els, "CONF-BT-A1"));
    let stages: Vec<_> = r.groups.iter().map(|g| (g.closed, g.stage.map(|s| s.name()), g.items.iter().map(|i| i.key.as_str()).collect::<Vec<_>>())).collect();
    assert_eq!(stages[0], (false, Some("compile".into()), vec!["Features::Motor.cc", "Features::Radio"]));
    assert_eq!(stages[1], (false, Some("load".into()), vec!["Features::Motor.spin", "Features::Sensor"]));
    assert_eq!(stages[2], (false, Some("runtime".into()), vec!["Features::Motor.gain"]));
    assert!(!r.open_keys().contains("Features::Motor.kv"), "a bound parameter is not open");
    assert_eq!(r.groups[2].binder.as_deref(), Some("system"), "the runtime stage's binder");

    let rb = residual(&els, cfg(&els, "CONF-BT-B1"));
    let closed: Vec<_> = rb.groups.iter().filter(|g| g.closed).flat_map(|g| g.items.iter().map(|i| i.key.as_str())).collect();
    assert_eq!(closed, vec!["Features::Motor.cc", "Features::Radio"]);
}

#[test]
fn constraint_state_follows_the_partial_binding() {
    let state = |name: &str| {
        let els = load("017", name);
        residual(&els, cfg(&els, "CONF-BT-A1")).constraints.iter().find(|c| c.id == "PC-BT-1").map(|c| c.state).unwrap()
    };
    assert_eq!(state("viol"), Tri::Violated);
    assert_eq!(state("undet"), Tri::Undetermined);
    assert_eq!(state("sat"), Tri::Satisfied);

    let e221 = |name: &str| check_feature_model(&load("017", name)).into_iter().filter(|f| f.code == "E221").count();
    assert_eq!((e221("viol"), e221("undet"), e221("unstaged_viol"), e221("full_viol")), (1, 0, 0, 1));
}

#[test]
fn deep_check_blames_the_earliest_configuration_and_spares_partial_ones() {
    let deep = |name: &str| check_feature_model_deep(&load("019", name)).findings;
    let broken = deep("broken");
    let e758: Vec<_> = broken.iter().filter(|f| f.code == "E758").collect();
    assert_eq!(e758.len(), 1, "{broken:?}");
    assert!(e758[0].message.contains("CONF-BT-A1") && !e758[0].message.contains("CONF-BT-B1"));
    assert!(deep("fine").iter().all(|f| !matches!(f.code, "E758" | "E225")));
    assert!(deep("partial_ok").iter().all(|f| !matches!(f.code, "E758" | "E225")));

    let e219 = |name: &str| check_feature_model(&load("019", name)).into_iter().filter(|f| f.code == "E219").count();
    assert_eq!((e219("partial_ok"), e219("not_partial"), e219("unstaged")), (0, 1, 1));
}

#[test]
fn later_window_required_parameters_do_not_warn() {
    let w017: Vec<String> = validate(&load("020", "open")).findings.into_iter().filter(|f| f.code == "W017").map(|f| f.message).collect();
    assert_eq!(w017.len(), 1, "{w017:?}");
    assert!(w017[0].contains("reqcomp"));
}

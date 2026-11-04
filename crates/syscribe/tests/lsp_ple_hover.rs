//! TC-TRS-LSPHOVER-001 (GH #420): variability hover and code lenses.

mod common;
use common::*;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn staged(tag: &str) -> PathBuf {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model");
    let dst = std::env::temp_dir().join(format!("syscribe-lsphover-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dst);
    copy(&src, &dst);
    dst
}
fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy(&e.path(), &p);
        } else {
            std::fs::copy(e.path(), p).unwrap();
        }
    }
}

/// (line, column) of the first occurrence of `needle`, `plus` columns in.
fn pos(file: &Path, needle: &str, plus: usize) -> (u32, u32) {
    let text = std::fs::read_to_string(file).unwrap();
    for (i, l) in text.lines().enumerate() {
        if let Some(c) = l.find(needle) {
            return (i as u32, (c + plus) as u32);
        }
    }
    panic!("{needle} not in {}", file.display());
}

fn hover_text(m: &Path, file: &Path, needle: &str, plus: usize) -> Option<String> {
    let mut lsp = Lsp::start(m);
    lsp.initialize();
    let (l, c) = pos(file, needle, plus);
    let h = lsp.hover(file, l, c);
    lsp.shutdown();
    h["contents"]["value"].as_str().map(String::from)
}

fn lenses(m: &Path, file: &Path) -> Vec<(u32, String)> {
    let mut lsp = Lsp::start(m);
    lsp.initialize();
    let v: Value = lsp.code_lens(file);
    lsp.shutdown();
    v.as_array().cloned().unwrap_or_default().iter().map(|l| (l["range"]["start"]["line"].as_u64().unwrap() as u32, l["command"]["title"].as_str().unwrap().to_string())).collect()
}

#[test]
fn hover_on_applies_when_lists_the_configurations_it_is_active_in() {
    let m = staged("aw");
    let part = m.join("Gadget.md");
    std::fs::write(&part, "---\ntype: PartDef\nname: Gadget\nappliesWhen: Features::Telemetry\n---\n\ng\n").unwrap();
    let t = hover_text(&m, &part, "appliesWhen", 3).expect("hover on the key");
    // Telemetry is selected by DEPLOY, inherited by FLIGHT; BUILD does not select it.
    assert!(t.contains("Active in 2 of 3 configurations") && t.contains("`CONF-UAV-DEPLOY-001`") && t.contains("`CONF-UAV-FLIGHT-001`"), "{t}");
    assert!(t.contains("Inactive in: `CONF-UAV-BUILD-001`"), "{t}");
    // Hovering a feature name inside the expression still shows that feature, as before.
    let f = hover_text(&m, &part, "Features::Telemetry", 3).expect("feature hover");
    assert!(f.contains("Telemetry") && !f.contains("Active in"), "{f}");
    // An element that is not gated has no activity to report, and its other keys have no variability hover.
    let plain = m.join("Plain.md");
    std::fs::write(&plain, "---\ntype: PartDef\nname: Plain\n---\n\np\n").unwrap();
    assert!(hover_text(&m, &plain, "name", 1).is_none());
    let (l, _) = pos(&part, "name: Gadget", 0);
    let mut lsp = Lsp::start(&m);
    lsp.initialize();
    assert!(lsp.hover(&part, l, 1)["contents"]["value"].is_null(), "another key of a gated element");
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn hover_on_a_binding_shows_range_unit_window_and_its_constraints() {
    let m = staged("bind");
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    let t = hover_text(&m, &conf, "Features::Propulsion.motorKV", 14).expect("hover on the key");
    assert!(t.contains("rpm/V") && t.contains("range 900..1200") && t.contains("bindable compile..load") && t.contains("Bound here at stage `load`"), "{t}");
    assert!(t.contains("`PC-UAV-CURRENT`") && t.contains("satisfied"), "the constraint it takes part in, as it stands here: {t}");
    // The build configuration only narrows the parameter; there the same constraint is still undetermined.
    let build = m.join("Configurations/CONF-UAV-BUILD-001.md");
    let t = hover_text(&m, &build, "Features::Propulsion.motorKV", 14).expect("hover on a narrowing key");
    assert!(t.contains("Narrowed here at stage `compile`") && t.contains("undetermined") && t.contains("range 900..1200"), "{t}");
    // Past the colon, on the value, there is no binding hover.
    assert!(hover_text(&m, &conf, "motorKV: 1000", 11).is_none());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn lenses_say_how_many_configurations_and_who_bound_each_parameter() {
    let m = staged("lens");
    let part = m.join("Gadget.md");
    std::fs::write(&part, "---\ntype: PartDef\nname: Gadget\nappliesWhen: Features::Telemetry\n---\n\ng\n").unwrap();
    let l = lenses(&m, &part);
    assert!(l.iter().any(|(line, t)| *line == 3 && t == "active in 2 of 3 configurations"), "on the appliesWhen line: {l:?}");
    let prop = m.join("Features/Propulsion.md");
    let l = lenses(&m, &prop);
    assert!(l.iter().any(|(_, t)| t == "bound at load by CONF-UAV-DEPLOY-001"), "{l:?}");
    let (motor_line, _) = pos(&prop, "name: motorKV", 0);
    assert!(l.iter().any(|(line, t)| *line == motor_line && t.starts_with("bound at load")), "on the parameter's own line: {l:?}");
    let tel = m.join("Features/Telemetry.md");
    let l = lenses(&m, &tel);
    assert!(l.iter().any(|(_, t)| t == "bound at runtime by CONF-UAV-FLIGHT-001"), "{l:?}");
    assert!(l.iter().any(|(_, t)| t.starts_with("bound at load by CONF-UAV-DEPLOY-001")), "the mode: {l:?}");
    // A parameter nobody binds says so, with its default.
    std::fs::write(m.join("Configurations/CONF-UAV-FLIGHT-001.md"), std::fs::read_to_string(m.join("Configurations/CONF-UAV-FLIGHT-001.md")).unwrap().replace("  Features::Telemetry.rateHz: 20\n", "")).unwrap();
    let l = lenses(&m, &tel);
    assert!(l.iter().any(|(_, t)| t == "unbound — default 10"), "{l:?}");
    // Existing lenses are kept.
    let req = common_fixture_lens();
    assert!(req.iter().any(|t| t.contains("verifiedBy")), "{req:?}");
    let _ = std::fs::remove_dir_all(&m);
}

fn common_fixture_lens() -> Vec<String> {
    let model = fixture_copy();
    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    let v = lsp.code_lens(&model.join("Requirements/REQ-FX-001.md"));
    lsp.shutdown();
    v.as_array().cloned().unwrap_or_default().iter().map(|l| l["command"]["title"].as_str().unwrap().to_string()).collect()
}

#[test]
fn it_stays_interactive_on_the_largest_example_model() {
    let gti = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../model_gti");
    let mut lsp = Lsp::start(&gti);
    lsp.initialize();
    // A gated element and a feature with parameters, both in the GTI model.
    let features = std::fs::read_dir(gti.join("Features")).ok();
    let file = features.and_then(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).find(|p| p.extension().is_some_and(|x| x == "md") && std::fs::read_to_string(p).map(|t| t.contains("parameters:")).unwrap_or(false)));
    if let Some(f) = file {
        let started = std::time::Instant::now();
        let _ = lsp.code_lens(&f);
        let took = started.elapsed();
        assert!(took < std::time::Duration::from_secs(5), "code lens on a feature took {took:?}");
    }
    lsp.shutdown();
}

//! TC-TRS-DELTA-001 (GH #415): `syscribe/modelChanged` — what a reload changed, debounced.

mod common;
use common::*;
use serde_json::Value;

const DEBOUNCE: &str = "120";
const METHOD: &str = "syscribe/modelChanged";

fn req(id: &str, status: &str) -> String {
    format!("---\ntype: Requirement\nid: {id}\nname: \"R {id}\"\nstatus: {status}\nreqDomain: software\nreqClass: system\n---\n\nBody of {id}.\n")
}

fn started(model: &std::path::Path) -> Lsp {
    let mut lsp = Lsp::start_with_env(model, &[("SYSCRIBE_LSP_NOTIFY_DEBOUNCE_MS", DEBOUNCE)]);
    lsp.initialize_with_model_changed();
    lsp
}

fn count(lsp: &Lsp) -> usize {
    lsp.notifications.iter().filter(|n| n["method"] == METHOD).count()
}

/// Let the debounce window pass, then force the server's output through with a plain request.
fn settle(lsp: &mut Lsp) {
    std::thread::sleep(std::time::Duration::from_millis(600));
    lsp.workspace_symbol("zzz-nothing");
}

fn assert_schema(params: &Value) {
    let (text, c) = {
        let o = std::process::Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["schema", "model-changed"]).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).to_string(), o.status.code())
    };
    assert_eq!(c, Some(0));
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let url = "https://syscribe.dev/schemas/model-changed.schema.json";
    compiler.add_resource(url, serde_json::from_str(&text).unwrap()).unwrap();
    let idx = compiler.compile(url, &mut schemas).unwrap();
    schemas.validate(params, idx).unwrap_or_else(|e| panic!("{e:#}\n{params}"));
}

#[test]
fn a_new_file_is_reported_as_added_with_a_validation_change() {
    let model = fixture_copy();
    let mut lsp = started(&model);
    let p = model.join("Requirements/REQ-FXLSP-N1.md");
    std::fs::write(&p, req("REQ-FXLSP-N1", "draft")).unwrap();
    lsp.did_save(&p, None);
    let n = lsp.wait_for_notification(METHOD);
    assert_schema(&n);
    assert_eq!(n["sequence"], 1);
    assert!(n["added"].as_array().unwrap().iter().any(|q| q == "Requirements::REQ-FXLSP-N1"), "{n}");
    assert!(n["changed"].as_array().unwrap().is_empty() && n["removed"].as_array().unwrap().is_empty(), "{n}");
    assert_eq!(n["featureModelChanged"], false);
    lsp.shutdown();
}

#[test]
fn changed_and_removed_are_reported_and_sequence_increases() {
    let model = fixture_copy();
    let mut lsp = started(&model);
    let victim = model.join("Requirements/REQ-FX-001.md");
    let text = std::fs::read_to_string(&victim).unwrap();
    std::fs::write(&victim, text.replace("Requirements text", "Requirements text, reworded")).unwrap();
    std::fs::write(&victim, format!("{text}\nAn added line.\n")).unwrap();
    lsp.did_save(&victim, None);
    let n = lsp.wait_for_notification(METHOD);
    assert!(n["changed"].as_array().unwrap().iter().any(|q| q == "Requirements::REQ-FX-001"), "{n}");

    std::fs::remove_file(&victim).unwrap();
    lsp.did_change_watched_files(&victim, 3);
    let n2 = lsp.wait_for_notification(METHOD);
    assert_schema(&n2);
    assert_eq!(n2["sequence"], 2);
    assert!(n2["removed"].as_array().unwrap().iter().any(|q| q == "Requirements::REQ-FX-001"), "{n2}");
    assert_eq!(n2["validationChanged"], true, "the requirement's tests now dangle: {n2}");
    lsp.shutdown();
}

#[test]
fn a_burst_of_reloads_is_one_notification() {
    let model = fixture_copy();
    let mut lsp = started(&model);
    for i in 0..5 {
        let p = model.join(format!("Requirements/REQ-FXLSP-B{i}.md"));
        std::fs::write(&p, req(&format!("REQ-FXLSP-B{i}"), "draft")).unwrap();
        lsp.did_save(&p, None);
    }
    let n = lsp.wait_for_notification(METHOD);
    assert_eq!(n["added"].as_array().unwrap().len(), 5, "{n}");
    settle(&mut lsp);
    assert_eq!(count(&lsp), 0, "no second notification for the same burst");
    lsp.shutdown();
}

#[test]
fn an_edit_and_its_undo_report_nothing() {
    let model = fixture_copy();
    let mut lsp = started(&model);
    let p = model.join("Requirements/REQ-FX-001.md");
    let original = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, format!("{original}\nTemporary.\n")).unwrap();
    lsp.did_save(&p, None);
    std::fs::write(&p, &original).unwrap();
    lsp.did_save(&p, None);
    settle(&mut lsp);
    assert_eq!(count(&lsp), 0, "{:?}", lsp.notifications);
    lsp.shutdown();
}

#[test]
fn feature_model_and_configuration_changes_are_flagged() {
    let model = fixture_copy();
    // Add a feature and a configuration to the fixture.
    std::fs::create_dir_all(model.join("Features")).unwrap();
    std::fs::write(model.join("Features/_index.md"), "---\ntype: Package\nname: Features\n---\n").unwrap();
    std::fs::write(model.join("Features/Alpha.md"), "---\nid: FEAT-NA-ALPHA\ntype: FeatureDef\nname: Alpha\n---\n\nA.\n").unwrap();
    std::fs::create_dir_all(model.join("Configurations")).unwrap();
    std::fs::write(model.join("Configurations/_index.md"), "---\ntype: Package\nname: Configurations\n---\n").unwrap();
    std::fs::write(model.join("Configurations/CONF-NA-ONE.md"), "---\nid: CONF-NA-ONE\ntype: Configuration\nname: One\nfeatures:\n  FEAT-NA-ALPHA: true\n---\n\nc\n").unwrap();
    let mut lsp = started(&model);
    // Change the feature: the feature model changed, the configuration did not.
    std::fs::write(model.join("Features/Alpha.md"), "---\nid: FEAT-NA-ALPHA\ntype: FeatureDef\nname: Alpha\n---\n\nA, changed.\n").unwrap();
    lsp.did_save(&model.join("Features/Alpha.md"), None);
    let n = lsp.wait_for_notification(METHOD);
    assert_eq!(n["featureModelChanged"], true, "{n}");
    assert!(n["configurationsChanged"].as_array().unwrap().is_empty(), "{n}");
    // Change the configuration.
    std::fs::write(model.join("Configurations/CONF-NA-ONE.md"), "---\nid: CONF-NA-ONE\ntype: Configuration\nname: One\nfeatures:\n  FEAT-NA-ALPHA: false\n---\n\nc\n").unwrap();
    lsp.did_save(&model.join("Configurations/CONF-NA-ONE.md"), None);
    let n = lsp.wait_for_notification(METHOD);
    assert_eq!(n["featureModelChanged"], false, "{n}");
    assert!(n["configurationsChanged"].as_array().unwrap().iter().any(|c| c.as_str().unwrap().contains("CONF-NA-ONE")), "{n}");
    lsp.shutdown();
}

#[test]
fn a_client_that_did_not_opt_in_receives_nothing_and_capabilities_advertise_it() {
    let model = fixture_copy();
    let mut lsp = Lsp::start_with_env(&model, &[("SYSCRIBE_LSP_NOTIFY_DEBOUNCE_MS", DEBOUNCE)]);
    let init = lsp.initialize();
    assert!(init["capabilities"]["experimental"]["syscribe"]["lspNotifications"].as_array().unwrap().iter().any(|m| m == METHOD));
    assert_eq!(init["capabilities"]["experimental"]["syscribe"]["jsonSchemas"]["model-changed"]["transport"], "lsp");
    let p = model.join("Requirements/REQ-FXLSP-N2.md");
    std::fs::write(&p, req("REQ-FXLSP-N2", "draft")).unwrap();
    lsp.did_save(&p, None);
    settle(&mut lsp);
    assert_eq!(count(&lsp), 0);
    lsp.shutdown();
}

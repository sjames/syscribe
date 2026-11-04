//! TC-TRS-LSPPLE-001 (GH #419): product-line-aware completion over LSP.

mod common;
use common::*;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn staged(tag: &str) -> PathBuf {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model");
    let dst = std::env::temp_dir().join(format!("syscribe-lspple-{}-{tag}", std::process::id()));
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

/// (line, column) of the end of the first occurrence of `needle` in `file`, plus `plus` columns.
fn pos(file: &Path, needle: &str, plus: usize) -> (u32, u32) {
    let text = std::fs::read_to_string(file).unwrap();
    for (i, l) in text.lines().enumerate() {
        if let Some(c) = l.find(needle) {
            return (i as u32, (c + needle.len() + plus) as u32);
        }
    }
    panic!("{needle} not in {}", file.display());
}

fn items(res: &Value) -> Vec<Value> {
    res.as_array().cloned().or_else(|| res.get("items").and_then(|i| i.as_array()).cloned()).unwrap_or_default()
}
fn labels(res: &Value) -> Vec<String> {
    items(res).iter().filter_map(|i| i["label"].as_str().map(String::from)).collect()
}
fn doc(res: &Value, label: &str) -> String {
    let i = items(res).into_iter().find(|i| i["label"] == label).unwrap_or_else(|| panic!("no {label}"));
    i["documentation"]["value"].as_str().unwrap_or("").to_string()
}

fn complete(model: &Path, file: &Path, needle: &str, plus: usize) -> Value {
    let mut lsp = Lsp::start(model);
    lsp.initialize();
    let (l, c) = pos(file, needle, plus);
    let res = lsp.completion(file, l, c);
    lsp.shutdown();
    res
}

#[test]
fn feature_keys_of_a_configuration_offer_qualified_names_and_ids_with_documentation() {
    let m = staged("feat");
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    let res = complete(&m, &conf, "  Features::Telemetry", 0);
    let l = labels(&res);
    for want in ["Features::Telemetry", "FEAT-UAV-TELEM", "Features::Propulsion", "FEAT-UAV-PROP"] {
        assert!(l.contains(&want.to_string()), "{want} in {l:?}");
    }
    assert!(!l.iter().any(|x| x.contains('.')), "no parameter paths where a feature is expected: {l:?}");
    let d = doc(&res, "Features::Propulsion");
    assert!(d.contains("FEAT-UAV-PROP") && d.contains("group mandatory") && d.contains("motorKV") && d.contains("maxCurrentA"), "{d}");
    assert_eq!(d.matches("mandatory").count(), 1, "said once: {d}");
    assert!(doc(&res, "Features::Telemetry").contains("bound load..runtime"), "a feature's binding window");
    // The edit replaces the whole qualified name under the cursor, `::` included.
    let item = items(&res).into_iter().find(|i| i["label"] == "Features::Propulsion").unwrap();
    let r = &item["textEdit"]["range"];
    assert_eq!(r["start"]["character"], 2);
    assert_eq!(r["end"]["character"], 2 + "Features::Telemetry".len() as u64);
    // The value after the colon is a boolean.
    let v = complete(&m, &conf, "  Features::Telemetry: ", 0);
    assert_eq!(labels(&v), ["true", "false"]);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn parameter_bindings_offer_parameter_paths_then_the_enumeration() {
    let m = staged("param");
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    let res = complete(&m, &conf, "  Features::Propulsion.motorKV", 0);
    let l = labels(&res);
    assert!(l.contains(&"Features::Propulsion.motorKV".to_string()) && l.contains(&"Features::Telemetry.mode".to_string()), "{l:?}");
    assert!(l.iter().all(|x| x.contains('.')), "only parameter paths: {l:?}");
    let d = doc(&res, "Features::Propulsion.motorKV");
    assert!(d.contains("rpm/V") && d.contains("range 900..1200") && d.contains("required") && d.contains("compile..load"), "{d}");
    let d = doc(&res, "Features::Propulsion.maxCurrentA");
    assert!(d.contains("default 40") && d.contains("load..runtime"), "{d}");
    // The value of an enumerated parameter completes to its enumValues, each documented.
    let v = complete(&m, &conf, "  Features::Telemetry.mode: ", 0);
    assert_eq!(labels(&v), ["eco", "normal", "sport"]);
    assert!(doc(&v, "eco").contains("one of eco, normal, sport"));
    // A parameter without an enumeration offers nothing rather than noise.
    assert!(labels(&complete(&m, &conf, "  Features::Propulsion.motorKV: ", 0)).is_empty());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn stage_names_complete_from_the_vocabulary() {
    let m = staged("stage");
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    let l = labels(&complete(&m, &conf, "bindingStage: ", 0));
    assert_eq!(&l[..3], ["compile", "load", "runtime"], "earliest first: {l:?}");
    // And on a feature's bindingTime / latestBindingTime, which are not top-level keys of a Configuration.
    let tel = m.join("Features/Telemetry.md");
    assert!(labels(&complete(&m, &tel, "bindingTime: ", 0)).contains(&"load".to_string()));
    assert!(labels(&complete(&m, &tel, "latestBindingTime: ", 0)).contains(&"runtime".to_string()));
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn applies_when_requires_excludes_and_constraints() {
    let m = staged("expr");
    std::fs::write(m.join("Part.md"), "---\ntype: PartDef\nname: Part\nappliesWhen: Features::Telemetry and \n---\n\np\n").unwrap();
    let l = labels(&complete(&m, &m.join("Part.md"), "appliesWhen: Features::Telemetry and ", 0));
    assert!(l.contains(&"Features::Propulsion".to_string()) && l.contains(&"FEAT-UAV-TELEM".to_string()), "{l:?}");
    // Not on the field's own name.
    let (line, col) = pos(&m.join("Part.md"), "appliesWh", 0);
    let mut lsp = Lsp::start(&m);
    lsp.initialize();
    assert!(labels(&lsp.completion(&m.join("Part.md"), line, col)).is_empty());
    lsp.shutdown();
    std::fs::write(m.join("Features/Extra.md"), "---\nid: FEAT-UAV-EXTRA\ntype: FeatureDef\nname: Extra\nrequires:\n  - Features::Telemetry\nexcludes: [Features::Propulsion]\n---\n\ne\n").unwrap();
    let extra = m.join("Features/Extra.md");
    assert!(labels(&complete(&m, &extra, "  - Features::Telemetry", 0)).contains(&"Features::Propulsion".to_string()));
    assert!(labels(&complete(&m, &extra, "excludes: [Features::Propulsion", 0)).contains(&"Features::Telemetry".to_string()));
    // A parameter constraint's expression completes parameter paths.
    let idx = m.join("Features/_index.md");
    let l = labels(&complete(&m, &idx, "expression: \"Features::Propulsion.motorKV", 0));
    assert!(l.contains(&"Features::Telemetry.rateHz".to_string()) && l.iter().all(|x| x.contains('.')), "{l:?}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn other_fields_keep_their_completion() {
    // The earlier contexts are untouched: a Requirement's status and a TestCase's verifies list.
    let model = fixture_copy();
    let req = model.join("Requirements/REQ-FX-001.md");
    let l = labels(&complete(&model, &req, "status: ", 0));
    assert!(l.contains(&"draft".to_string()) && l.contains(&"approved".to_string()), "{l:?}");
    let tc = model.join("Verification/TC-FX-001.md");
    let l = labels(&complete(&model, &tc, "  - REQ-FX-001", 0));
    assert!(!l.is_empty() && l.iter().all(|x| x.starts_with("REQ-")), "{l:?}");
}

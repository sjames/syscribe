//! `REQ-TRS-BT-023..030`: the late-bound parameter contract — which entries a stage carries,
//! how values resolve, the deterministic YAML and its hash, the JSON Schema, and `check_file`
//! — over the worked example `examples/staged-configuration`.

use std::path::PathBuf;

use syscribe_model::binding_time::BindingTime;
use syscribe_model::element::RawElement;
use syscribe_model::params::{check_file, contract, Contract};
use syscribe_model::walker::walk_model;

fn model() -> Vec<RawElement> {
    walk_model(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model")).unwrap()
}

fn contract_of(els: &[RawElement], conf: &str, stage: Option<&str>) -> Result<Contract, String> {
    let i = els.iter().position(|e| e.frontmatter.id.as_deref() == Some(conf)).unwrap();
    contract(els, i, stage.map(|s| BindingTime::parse(s).unwrap()))
}

fn names(c: &Contract) -> Vec<String> {
    c.params.iter().map(|p| format!("{}.{}", p.feature, p.name)).collect()
}

fn codes(c: &Contract, text: &str) -> Vec<&'static str> {
    check_file(c, text, "values.yaml").into_iter().map(|f| f.code).collect()
}

#[test]
fn entries_are_those_whose_window_contains_the_stage() {
    let els = model();
    let load = contract_of(&els, "CONF-UAV-DEPLOY-001", None).unwrap();
    assert_eq!(names(&load), ["FEAT-UAV-PROP.maxCurrentA", "FEAT-UAV-PROP.motorKV", "FEAT-UAV-TELEM.mode"]);
    assert_eq!(load.toggles.iter().map(|t| (t.feature.as_str(), t.selected)).collect::<Vec<_>>(), [("FEAT-UAV-TELEM", true)]);

    let run = contract_of(&els, "CONF-UAV-FLIGHT-001", None).unwrap();
    assert_eq!(names(&run), ["FEAT-UAV-PROP.maxCurrentA", "FEAT-UAV-TELEM.mode", "FEAT-UAV-TELEM.rateHz"]);

    // An explicit stage wins over the Configuration's own.
    let at_runtime = contract_of(&els, "CONF-UAV-DEPLOY-001", Some("runtime")).unwrap();
    assert!(names(&at_runtime).contains(&"FEAT-UAV-TELEM.rateHz".to_string()));

    let unstaged_err = {
        let mut e = model();
        let i = e.iter().position(|x| x.frontmatter.id.as_deref() == Some("CONF-UAV-DEPLOY-001")).unwrap();
        e[i].frontmatter.binding_stage = None;
        contract(&e, i, None).unwrap_err()
    };
    assert!(unstaged_err.contains("--stage"), "{unstaged_err}");
}

#[test]
fn values_resolve_binding_then_default_then_fallback_and_record_their_source() {
    let els = model();
    let c = contract_of(&els, "CONF-UAV-FLIGHT-001", None).unwrap();
    let val = |n: &str| c.params.iter().find(|p| p.name == n).and_then(|p| p.value.clone()).unwrap();
    assert_eq!(val("rateHz").1, "own (runtime)");
    assert_eq!(val("mode").1, "inherited from CONF-UAV-DEPLOY-001 (load)");
    assert!(c.unresolved().is_empty());

    // Build stage: motorKV is required, window compile..load, nothing binds it yet.
    let build = contract_of(&els, "CONF-UAV-BUILD-001", None).unwrap();
    let open: Vec<_> = build.unresolved().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(open, ["motorKV"]);
    assert!(build.to_yaml("test").contains("motorKV: null  # OPEN"));
}

#[test]
fn yaml_is_deterministic_and_the_hash_tracks_the_contract() {
    let els = model();
    let a = contract_of(&els, "CONF-UAV-DEPLOY-001", None).unwrap();
    let b = contract_of(&els, "CONF-UAV-DEPLOY-001", None).unwrap();
    assert_eq!(a.to_yaml("v"), b.to_yaml("v"));
    assert!(a.hash().starts_with("blake3:") && a.hash().len() == "blake3:".len() + 64);
    assert_eq!(a.hash(), b.hash());
    // The generator version is a comment, not part of the contract.
    assert_eq!(a.to_yaml("1").lines().filter(|l| !l.starts_with("# Generated")).collect::<Vec<_>>(), a.to_yaml("2").lines().filter(|l| !l.starts_with("# Generated")).collect::<Vec<_>>());
    // A different Configuration or stage is a different contract.
    assert_ne!(a.hash(), contract_of(&els, "CONF-UAV-FLIGHT-001", None).unwrap().hash());
    assert_ne!(a.hash(), contract_of(&els, "CONF-UAV-DEPLOY-001", Some("runtime")).unwrap().hash());
    // A change to a parameter's range changes the hash.
    let mut changed = model();
    for e in changed.iter_mut().filter(|e| e.frontmatter.id.as_deref() == Some("FEAT-UAV-PROP")) {
        for p in e.frontmatter.parameters.iter_mut().flatten() {
            if let serde_yaml::Value::Mapping(m) = p {
                if m.get("name").and_then(|n| n.as_str()) == Some("maxCurrentA") {
                    m.insert("range".into(), "0..55".into());
                }
            }
        }
    }
    assert_ne!(a.hash(), contract_of(&changed, "CONF-UAV-DEPLOY-001", None).unwrap().hash());
}

#[test]
fn schema_describes_every_entry() {
    let c = contract_of(&model(), "CONF-UAV-DEPLOY-001", None).unwrap();
    let s = c.schema();
    assert_eq!(s["$schema"], "https://json-schema.org/draft/2020-12/schema");
    let kv = &s["properties"]["parameters"]["properties"]["FEAT-UAV-PROP"]["properties"]["motorKV"];
    assert_eq!((kv["type"].as_str(), kv["minimum"].as_f64(), kv["maximum"].as_f64()), (Some("number"), Some(900.0), Some(1200.0)));
    let mode = &s["properties"]["parameters"]["properties"]["FEAT-UAV-TELEM"]["properties"]["mode"];
    assert_eq!(mode["enum"], serde_json::json!(["eco", "normal", "sport"]));
    assert_eq!(s["properties"]["parameters"]["properties"]["FEAT-UAV-PROP"]["required"], serde_json::json!(["motorKV"]));
    assert_eq!(s["properties"]["features"]["properties"]["FEAT-UAV-TELEM"]["type"], "boolean");
}

#[test]
fn check_file_finds_each_kind_of_defect_and_staleness() {
    let c = contract_of(&model(), "CONF-UAV-DEPLOY-001", None).unwrap();
    let good = c.to_yaml("test");
    assert!(codes(&c, &good).is_empty(), "{:?}", codes(&c, &good));

    let edit = |from: &str, to: &str| good.replacen(from, to, 1);
    assert_eq!(codes(&c, &edit("motorKV: 1000", "motorKV: 5000")), ["E205"]);
    assert_eq!(codes(&c, &edit("mode: sport", "mode: turbo")), ["E206"]);
    assert_eq!(codes(&c, &edit("maxCurrentA: 45", "maxCurrentA: lots")), ["E761"]);
    assert_eq!(codes(&c, &edit("mode: sport", "modez: sport")), ["E760", "E762"], "an unknown parameter, and the required one now missing");
    assert_eq!(codes(&c, &format!("{good}surprise: 1\n")), ["E760"]);
    assert_eq!(codes(&c, &edit("    motorKV: 1000", "    motorKV: null")), ["E762"], "a null placeholder is still open");
    assert_eq!(codes(&c, "- not a mapping\n"), ["E761"]);

    // Stale: edit the header hash, drop it, or check against a changed model.
    assert_eq!(codes(&c, &edit("blake3:", "blake3:00")), ["W751"]);
    assert_eq!(codes(&c, &good.lines().filter(|l| !l.starts_with("# projection-hash")).collect::<Vec<_>>().join("\n")), ["W751"]);
}

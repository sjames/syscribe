//! TC-TRS-LSPTOK-001 (GH #422): semantic tokens for the variability fields.

mod common;
use common::*;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn staged(tag: &str) -> PathBuf {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model");
    let dst = std::env::temp_dir().join(format!("syscribe-lsptok-{}-{tag}", std::process::id()));
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

/// (text, type name, unresolved) of every token in `file`, decoded from the delta encoding.
fn tokens(model: &Path, file: &Path) -> Vec<(String, String, bool)> {
    let mut lsp = Lsp::start(model);
    let init = lsp.initialize();
    let legend = &init["capabilities"]["semanticTokensProvider"]["legend"];
    let types: Vec<String> = legend["tokenTypes"].as_array().unwrap().iter().map(|t| t.as_str().unwrap().to_string()).collect();
    let res: Value = lsp.semantic_tokens(file);
    lsp.shutdown();
    let data: Vec<u64> = res["data"].as_array().cloned().unwrap_or_default().iter().map(|n| n.as_u64().unwrap()).collect();
    assert_eq!(data.len() % 5, 0);
    let text = std::fs::read_to_string(file).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let (mut line, mut col) = (0u64, 0u64);
    let mut out = Vec::new();
    for t in data.chunks(5) {
        line += t[0];
        col = if t[0] == 0 { col + t[1] } else { t[1] };
        let s: String = lines[line as usize].chars().skip(col as usize).take(t[2] as usize).collect();
        out.push((s, types[t[3] as usize].clone(), t[4] & 1 == 1));
    }
    out
}

fn has(toks: &[(String, String, bool)], text: &str, ty: &str, unresolved: bool) -> bool {
    toks.iter().any(|(t, y, u)| t == text && y == ty && *u == unresolved)
}

#[test]
fn the_legend_is_advertised() {
    let m = staged("legend");
    let mut lsp = Lsp::start(&m);
    let init = lsp.initialize();
    let p = &init["capabilities"]["semanticTokensProvider"];
    assert_eq!(p["legend"]["tokenTypes"], serde_json::json!(["class", "enumMember", "property", "keyword"]));
    assert_eq!(p["legend"]["tokenModifiers"], serde_json::json!(["unresolved"]));
    assert_eq!(p["full"], true);
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn applies_when_marks_features_and_flags_unknown_ones() {
    let m = staged("aw");
    let part = m.join("Gadget.md");
    std::fs::write(&part, "---\ntype: PartDef\nname: Gadget\nappliesWhen: Features::Telemetry and not Features::Nope\n---\n\nBody mentions Features::Telemetry too.\n").unwrap();
    let t = tokens(&m, &part);
    assert!(has(&t, "Features::Telemetry", "class", false), "{t:?}");
    assert!(has(&t, "Features::Nope", "class", true), "an unknown feature is flagged: {t:?}");
    assert_eq!(t.len(), 2, "operators, other fields and the body are not tokens: {t:?}");
    std::fs::write(&part, "---\ntype: PartDef\nname: Gadget\nappliesWhen: FEAT-UAV-TELEM or FEAT-NOPE-XX\n---\n\nx\n").unwrap();
    let t = tokens(&m, &part);
    assert!(has(&t, "FEAT-UAV-TELEM", "enumMember", false) && has(&t, "FEAT-NOPE-XX", "enumMember", true), "{t:?}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn configuration_keys_and_stages() {
    let m = staged("conf");
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    let t = tokens(&m, &conf);
    assert!(has(&t, "Features::Telemetry", "class", false), "{t:?}");
    for p in ["Features::Propulsion.motorKV", "Features::Propulsion.maxCurrentA", "Features::Telemetry.mode"] {
        assert!(has(&t, p, "property", false), "{p} in {t:?}");
    }
    assert!(has(&t, "load", "keyword", false), "{t:?}");
    assert!(!t.iter().any(|(s, ..)| s == "1000" || s == "sport"), "values are not references: {t:?}");
    // Typos are flagged, not hidden.
    let text = std::fs::read_to_string(&conf).unwrap().replace("motorKV: 1000", "motrKV: 1000").replace("bindingStage: load", "bindingStage: loda");
    std::fs::write(&conf, text).unwrap();
    let t = tokens(&m, &conf);
    assert!(has(&t, "Features::Propulsion.motrKV", "property", true), "{t:?}");
    assert!(has(&t, "loda", "keyword", true), "{t:?}");
    assert!(has(&t, "Features::Propulsion.maxCurrentA", "property", false), "the others stay resolved");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn constraint_expressions_and_requires_lists() {
    let m = staged("expr");
    let idx = m.join("Features/_index.md");
    let t = tokens(&m, &idx);
    assert!(has(&t, "Features::Propulsion.motorKV", "property", false) && has(&t, "Features::Propulsion.maxCurrentA", "property", false), "{t:?}");
    assert!(has(&t, "Features::Propulsion", "class", false), "its appliesWhen: {t:?}");
    assert!(!t.iter().any(|(s, ..)| s == "0.04" || s == "100" || s == "<="), "numbers and operators are not tokens: {t:?}");
    std::fs::write(m.join("Features/Extra.md"), "---\nid: FEAT-UAV-EXTRA\ntype: FeatureDef\nname: Extra\nrequires:\n  - Features::Telemetry\nexcludes: [Features::Gone]\n---\n\ne\n").unwrap();
    let t = tokens(&m, &m.join("Features/Extra.md"));
    assert!(has(&t, "Features::Telemetry", "class", false) && has(&t, "Features::Gone", "class", true), "{t:?}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_file_without_these_fields_has_no_tokens() {
    let model = fixture_copy();
    assert!(tokens(&model, &model.join("Requirements/REQ-FX-001.md")).is_empty());
}

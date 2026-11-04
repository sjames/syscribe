//! TC-TRS-LSPFIX-001 (GH #421): quick fixes for the product-line and binding-time diagnostics, one test per code.

mod common;
use common::*;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn staged(tag: &str) -> PathBuf {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model");
    let dst = std::env::temp_dir().join(format!("syscribe-lspfix-{}-{tag}", std::process::id()));
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

fn edit_file(file: &Path, from: &str, to: &str) {
    let t = std::fs::read_to_string(file).unwrap();
    assert!(t.contains(from), "{from} in {t}");
    std::fs::write(file, t.replacen(from, to, 1)).unwrap();
}

fn actions(model: &Path, file: &Path) -> Vec<Value> {
    let mut lsp = Lsp::start(model);
    lsp.initialize();
    let n = std::fs::read_to_string(file).unwrap().lines().count() as u32;
    let v = lsp.code_action(file, 0, n);
    lsp.shutdown();
    v.as_array().cloned().unwrap_or_default()
}

fn by_code<'a>(acts: &'a [Value], code: &str) -> Vec<&'a Value> {
    acts.iter().filter(|a| a["data"]["code"] == code).collect()
}

/// Apply a code action's edits to the file's text (single-line edits, as the server produces them).
fn apply(file: &Path, action: &Value) -> String {
    let uri = common::file_uri(file);
    let edits = action["edit"]["changes"][&uri].as_array().unwrap_or_else(|| panic!("no edits for {uri}: {action}")).clone();
    let mut lines: Vec<String> = std::fs::read_to_string(file).unwrap().split('\n').map(str::to_string).collect();
    let mut es = edits;
    es.sort_by_key(|e| std::cmp::Reverse((e["range"]["start"]["line"].as_u64().unwrap(), e["range"]["start"]["character"].as_u64().unwrap())));
    for e in es {
        let (sl, sc) = (e["range"]["start"]["line"].as_u64().unwrap() as usize, e["range"]["start"]["character"].as_u64().unwrap() as usize);
        let (el, ec) = (e["range"]["end"]["line"].as_u64().unwrap() as usize, e["range"]["end"]["character"].as_u64().unwrap() as usize);
        let head: String = lines[sl].chars().take(sc).collect();
        let tail: String = lines.get(el).map(|l| l.chars().skip(ec).collect()).unwrap_or_default();
        let merged = format!("{head}{}{tail}", e["newText"].as_str().unwrap());
        let end = el.min(lines.len() - 1);
        lines.splice(sl..=end, merged.split('\n').map(str::to_string));
    }
    lines.join("\n")
}

/// Does `validate` report `code` on `file`?
fn reports(model: &Path, file: &Path, code: &str) -> bool {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(["validate", "--json"]).output().unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&o.stdout)));
    v.as_array().unwrap().iter().any(|f| f["code"] == code && f["file"].as_str().is_some_and(|p| p.ends_with(file.file_name().unwrap().to_str().unwrap())))
}

/// Apply the preferred fix for `code` and check the diagnostic is gone and the file still validates cleanly.
fn fixed_by(model: &Path, file: &Path, code: &str, title_has: &str) -> String {
    assert!(reports(model, file, code), "{code} is raised before the fix");
    let acts = actions(model, file);
    let fixes = by_code(&acts, code);
    assert!(!fixes.is_empty(), "no {code} fix among {:?}", acts.iter().map(|a| a["title"].clone()).collect::<Vec<_>>());
    let fix = fixes.iter().find(|a| a["isPreferred"] == true).or(fixes.first()).unwrap();
    assert!(fix["title"].as_str().unwrap().contains(title_has), "{}", fix["title"]);
    assert_eq!(fix["kind"], "quickfix");
    let new_text = apply(file, fix);
    std::fs::write(file, &new_text).unwrap();
    assert!(!reports(model, file, code), "{code} is gone after the fix:\n{new_text}");
    new_text
}

#[test]
fn e222_suggests_the_closest_declared_parameter() {
    let m = staged("e222");
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    edit_file(&conf, "Features::Propulsion.motorKV: 1000", "Features::Propulsion.motrKV: 1000");
    let t = fixed_by(&m, &conf, "E222", "Features::Propulsion.motorKV");
    assert!(t.contains("Features::Propulsion.motorKV: 1000") && !t.contains("motrKV"));
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn e754_removes_a_rebinding_of_a_closed_value() {
    let m = staged("e754");
    edit_file(&m.join("Configurations/CONF-UAV-BUILD-001.md"), "parameterNarrowing:", "parameterBindings:\n  Features::Propulsion.motorKV: 1000\nparameterNarrowing:");
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    edit_file(&conf, "Features::Propulsion.motorKV: 1000", "Features::Propulsion.motorKV: 1050");
    let t = fixed_by(&m, &conf, "E754", "Remove the rebinding of 'Features::Propulsion.motorKV'");
    assert!(!t.contains("motorKV: 1050") && t.contains("maxCurrentA: 45"), "only that line goes:\n{t}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn e754_removes_a_reversed_selection() {
    let m = staged("e754s");
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    edit_file(&conf, "features:\n", "features:\n  Features::Propulsion: false\n");
    let acts = actions(&m, &conf);
    let f = by_code(&acts, "E754");
    assert!(f.iter().any(|a| a["title"].as_str().unwrap().contains("Remove the selection of 'Features::Propulsion'")), "{:?}", acts.iter().map(|a| a["title"].clone()).collect::<Vec<_>>());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn e751_offers_only_the_stage_that_validates() {
    let m = staged("e751");
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    edit_file(&conf, "bindingStage: load", "bindingStage: loda");
    let acts = actions(&m, &conf);
    let f = by_code(&acts, "E751");
    let titles: Vec<&str> = f.iter().map(|a| a["title"].as_str().unwrap()).collect();
    // `compile` and `runtime` are valid stages, but this Configuration binds things at `load` and derives from a
    // `compile` base: those two would each raise new findings, so they are validated out and only `load` is offered.
    assert_eq!(titles, ["Use bindingStage 'load'"]);
    assert_eq!(f[0]["isPreferred"], true);
    let t = fixed_by(&m, &conf, "E751", "load");
    assert!(t.contains("bindingStage: load"));
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn e752_removes_a_binding_the_stage_cannot_make() {
    let m = staged("e752");
    let build = m.join("Configurations/CONF-UAV-BUILD-001.md");
    edit_file(&build, "parameterNarrowing:", "parameterBindings:\n  Features::Propulsion.maxCurrentA: 20\nparameterNarrowing:");
    let t = fixed_by(&m, &build, "E752", "Remove the binding of 'Features::Propulsion.maxCurrentA'");
    assert!(!t.contains("maxCurrentA") && t.contains("parameterNarrowing"), "{t}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn e756_moves_a_value_back_inside_what_was_narrowed() {
    let m = staged("e756");
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    edit_file(&conf, "Features::Propulsion.motorKV: 1000", "Features::Propulsion.motorKV: 1150");
    let t = fixed_by(&m, &conf, "E756", "Change the value to 1100");
    assert!(t.contains("motorKV: 1100"), "{t}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn w016_renames_selections_to_features() {
    let m = staged("w016");
    let solo = m.join("Configurations/CONF-UAV-SOLO-001.md");
    std::fs::write(&solo, "---\ntype: Configuration\nid: CONF-UAV-SOLO-001\nname: Solo\nstatus: draft\nfeatureModel: Features\nselections:\n  Features::Propulsion: true\n---\n\ns\n").unwrap();
    let t = fixed_by(&m, &solo, "W016", "Rename `selections:` to `features:`");
    assert!(t.contains("features:\n  Features::Propulsion: true") && !t.contains("selections:"), "{t}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn e201_adds_a_derived_feature_id() {
    let m = staged("e201");
    let f = m.join("Features/Extra.md");
    std::fs::write(&f, "---\ntype: FeatureDef\nname: Extra\n---\n\ne\n").unwrap();
    let t = fixed_by(&m, &f, "E201", "Add id: FEAT-EXTRA");
    assert!(t.starts_with("---\nid: FEAT-EXTRA\ntype: FeatureDef"), "{t}");
    // An id that is taken is not offered again: the next candidate includes the package.
    let g = m.join("Features/Extra2.md");
    std::fs::write(&g, "---\ntype: FeatureDef\nname: Extra\n---\n\ne\n").unwrap();
    let acts = actions(&m, &g);
    let titles: Vec<String> = by_code(&acts, "E201").iter().map(|a| a["title"].as_str().unwrap().to_string()).collect();
    assert!(titles.iter().all(|t| !t.contains("FEAT-EXTRA\"") && t != "Add id: FEAT-EXTRA"), "{titles:?}");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn a_file_with_none_of_these_findings_gets_none_of_these_fixes() {
    let m = staged("none");
    let conf = m.join("Configurations/CONF-UAV-DEPLOY-001.md");
    let acts = actions(&m, &conf);
    assert!(acts.iter().all(|a| a["data"]["code"].is_null()), "{acts:?}");
    // And the server never wrote the file.
    let before = std::fs::read_to_string(&conf).unwrap();
    let _ = actions(&m, &conf);
    assert_eq!(std::fs::read_to_string(&conf).unwrap(), before);
    let _ = std::fs::remove_dir_all(&m);
}

//! LSP navigation precision on a `FeatureModel` sheet (REQ-TRS-FM-005
//! corner-case review): go-to-definition/references/workspace-symbol on a
//! `FeatureDef` synthesized from a multi-entry `featureTree:` must land on
//! that entry's own line, not line 0 of a possibly large shared sheet file
//! (every entry would otherwise be indistinguishable from every other).
//! A genuine per-file `FeatureDef` is the regression guard: unchanged, line 0.

mod common;
use common::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "syscribe-lsp-fm-nav-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let features = dir.join("Features");
    std::fs::create_dir_all(&features).unwrap();
    // Wdt is deliberately the *last* entry, several lines in, so "line 0"
    // and "Wdt's real line" are unambiguously different positions.
    std::fs::write(
        features.join("_index.md"),
        r#"---
type: FeatureModel
name: Features
featureTree:
  - name: Platform
    id: FEAT-PLATFORM-001
    mandatory: true
    groupKind: alternative
  - name: Platform.CortexM
    id: FEAT-CORTEXM-001
    groupKind: optional
  - name: Wdt
    id: FEAT-WDT-001
    groupKind: optional
---
"#,
    )
    .unwrap();
    std::fs::write(
        features.join("SafeMode.md"),
        "---\ntype: FeatureDef\nid: FEAT-SAFEMODE-001\nname: SafeMode\ngroupKind: optional\n---\n",
    )
    .unwrap();
    let arch = dir.join("Architecture");
    std::fs::create_dir_all(&arch).unwrap();
    std::fs::write(
        arch.join("WdtDriver.md"),
        "---\ntype: PartDef\nname: WdtDriver\nappliesWhen: Features::Wdt\n---\n",
    )
    .unwrap();
    dir
}

fn line_of(loc: &serde_json::Value) -> u64 {
    loc.get("range")
        .and_then(|r| r.get("start"))
        .and_then(|s| s.get("line"))
        .and_then(|l| l.as_u64())
        .unwrap_or_else(|| panic!("no range.start.line in {loc:?}"))
}

#[test]
fn definition_on_a_synthesized_feature_lands_on_its_own_line() {
    let model = temp_model();
    let driver_path = model.join("Architecture/WdtDriver.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 3 is `appliesWhen: Features::Wdt` (0: ---, 1: type, 2: name, 3: appliesWhen).
    let res = lsp.definition(&driver_path, 3, 20);
    assert_eq!(line_of(&res), 11, "lands on Wdt's own '- name: Wdt' line, not line 0: {res:?}");

    lsp.shutdown();
}

#[test]
fn definition_on_a_genuine_per_file_feature_is_unchanged() {
    let model = temp_model();
    let sheet_path = model.join("Features/_index.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 8 is `- name: Platform.CortexM`; reuse it as a "from" position by
    // pointing at a reference to SafeMode instead — simplest is workspace/symbol.
    let res = lsp.workspace_symbol("SafeMode");
    let results = res.as_array().expect("symbol array");
    let safemode = results
        .iter()
        .find(|s| s.get("name").and_then(|n| n.as_str()) == Some("SafeMode"))
        .unwrap_or_else(|| panic!("SafeMode not in workspace/symbol results: {results:?}"));
    assert_eq!(
        line_of(&safemode["location"]),
        0,
        "a genuine per-file FeatureDef is unaffected — still line 0: {safemode:?}"
    );
    let _ = sheet_path;

    lsp.shutdown();
}

#[test]
fn workspace_symbol_distinguishes_every_entry_in_the_same_sheet() {
    let model = temp_model();

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    let res = lsp.workspace_symbol("");
    let results = res.as_array().expect("symbol array").clone();

    let line_for = |name: &str| -> u64 {
        let entry = results
            .iter()
            .find(|s| s.get("name").and_then(|n| n.as_str()) == Some(name))
            .unwrap_or_else(|| panic!("{name} not in workspace/symbol results: {results:?}"));
        line_of(&entry["location"])
    };
    let platform = line_for("Platform");
    let cortexm = line_for("CortexM");
    let wdt = line_for("Wdt");
    assert!(
        platform != cortexm && cortexm != wdt && platform != wdt,
        "each of the sheet's 3 features has a distinct line: Platform={platform}, CortexM={cortexm}, Wdt={wdt}"
    );

    lsp.shutdown();
}

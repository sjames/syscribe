//! Integration tests for WASM foreign-format plugin ingestion
//! (ADR-SYS-PLUGIN-001, REQ-TRS-PLUGIN-*).
//!
//! Requires the `wasm-plugins` feature: `cargo test -p syscribe-model --features wasm-plugins`.
#![cfg(feature = "wasm-plugins")]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::{validate_with_config, Severity};
use syscribe_model::walker::walk_model;

/// Every test in this file JIT-compiles at least one ~2.4MB wasmtime module —
/// several seconds of CPU each, uncached between processes. Left to `cargo
/// test`'s default parallelism, enough of these racing at once starves each
/// other for CPU and makes wall-clock assertions (like the hang-plugin
/// interruption test) flaky under load, not because anything is actually
/// broken. Serializing them against each other removes that contention; they
/// still run in parallel with every other crate's test binary.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial_guard() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

/// The toy SysMLv2-subset example plugin, built from
/// `examples/wasm-plugins/sysmlv2-toy/` and checked in as a test fixture.
fn toy_wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/plugins/sysmlv2-toy.wasm")
}

/// A unique throwaway directory under the OS temp dir (mirrors `config.rs`'s
/// test helper — this crate hand-rolls its own rather than depending on `tempfile`).
fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-plugins-test-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// Base fixture: a root package plus a `SysML2Legacy` package whose `_index.md`
/// declares `foreignFormat: sysmlv2`, and a native `Part` referencing the
/// plugin-emitted `PartDef` by qname via `supertype:`. `toml_plugins_table` lets
/// each test control what `[plugins.sysmlv2]` resolves to.
fn base_fixture(root: &Path, toml_plugins_table: &str) {
    write(root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(root, ".syscribe.toml", toml_plugins_table);
    write(
        root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nforeignFormat: sysmlv2\n---\n",
    );
    write(
        root,
        "SysML2Legacy/Sensor.sysml",
        r#"part def PressureSensor {
  doc "Measures cabin pressure."
}

requirement def SamplingRate {
  id "REQ-TOY-001"
  doc "The system shall sample the pressure sensor at 10 Hz."
}
"#,
    );
    write(
        root,
        "Vehicle/Sensor.md",
        "---\ntype: Part\nname: Sensor\nsupertype: SysML2Legacy::PressureSensor\n---\n",
    );
}

fn plugins_toml(wasm: &str) -> String {
    format!("[plugins.sysmlv2]\nwasm = \"{wasm}\"\n")
}

#[test]
fn plugin_elements_merge_and_cross_references_resolve() {
    let _serial = serial_guard();
    let root = tempdir();
    let wasm = toy_wasm_path();
    base_fixture(&root, &plugins_toml(&wasm.display().to_string()));

    let elements = walk_model(&root).unwrap();

    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::PressureSensor")
        .expect("plugin-emitted PartDef should be in the graph");
    assert_eq!(part.frontmatter.element_type, Some(syscribe_model::element::ElementType::PartDef));

    let req = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::SamplingRate")
        .expect("plugin-emitted RequirementDef should be in the graph");
    assert_eq!(req.frontmatter.id.as_deref(), Some("REQ-TOY-001"));

    // The .sysml source file itself must never appear as a native RawElement.
    assert!(elements.iter().all(|e| !e.file_path.ends_with(".sysml")));

    let result = validate_with_config(&elements, &ValidateConfig::with_model_root(&root));
    let errors: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "expected no errors, got: {errors:#?}");

    // No "unresolved supertype" style finding should mention Sensor/PressureSensor —
    // proving `supertype:` resolved straight through into the plugin-origin element.
    let unresolved: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.message.to_lowercase().contains("supertype") && f.message.contains("Sensor"))
        .collect();
    assert!(unresolved.is_empty(), "supertype should have resolved, got: {unresolved:#?}");

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn missing_wasm_path_produces_e530() {
    let _serial = serial_guard();
    let root = tempdir();
    base_fixture(&root, &plugins_toml("does-not-exist.wasm"));

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &ValidateConfig::with_model_root(&root));

    assert!(
        result.findings.iter().any(|f| f.code == "E530"),
        "expected E530, got: {:#?}",
        result.findings
    );
    assert!(elements
        .iter()
        .all(|e| e.qualified_name != "SysML2Legacy::PressureSensor"));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn unresolved_alias_produces_e532() {
    let _serial = serial_guard();
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, ".syscribe.toml", ""); // no [plugins] table at all
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nforeignFormat: sysmlv2\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &ValidateConfig::with_model_root(&root));

    assert!(
        result.findings.iter().any(|f| f.code == "E532"),
        "expected E532, got: {:#?}",
        result.findings
    );

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn two_foreign_packages_with_different_aliases_coexist() {
    let _serial = serial_guard();
    // Two independent [plugins.*] entries (both pointing at the same toy wasm —
    // it's a generic content-based parser, so reusing the binary under two
    // aliases is a legitimate way to prove the two invocations don't interfere
    // with each other), each owning its own package subtree.
    let root = tempdir();
    let wasm = toy_wasm_path().display().to_string();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        ".syscribe.toml",
        &format!("[plugins.aliasA]\nwasm = \"{wasm}\"\n\n[plugins.aliasB]\nwasm = \"{wasm}\"\n"),
    );
    write(
        &root,
        "PkgA/_index.md",
        "---\ntype: Package\nname: PkgA\nforeignFormat: aliasA\n---\n",
    );
    write(&root, "PkgA/Src.sysml", "part def WidgetA {\n  doc \"A's widget.\"\n}\n");
    write(
        &root,
        "PkgB/_index.md",
        "---\ntype: Package\nname: PkgB\nforeignFormat: aliasB\n---\n",
    );
    write(&root, "PkgB/Src.sysml", "part def WidgetB {\n  doc \"B's widget.\"\n}\n");
    // Native content in an unrelated third directory — proves neither foreign
    // package's stripping pass reaches outside its own subtree.
    write(&root, "Vehicle/NativeElsewhere.md", "---\ntype: Part\nname: NativeElsewhere\n---\n");

    let elements = walk_model(&root).unwrap();

    assert!(elements.iter().any(|e| e.qualified_name == "PkgA::WidgetA"), "PkgA's plugin element missing");
    assert!(elements.iter().any(|e| e.qualified_name == "PkgB::WidgetB"), "PkgB's plugin element missing");
    // Cross-contamination check: neither package's plugin should have produced
    // the other's element under the wrong qname prefix.
    assert!(elements.iter().all(|e| e.qualified_name != "PkgA::WidgetB"));
    assert!(elements.iter().all(|e| e.qualified_name != "PkgB::WidgetA"));
    // Unrelated native content survives — stripping stayed scoped to each
    // foreign package's own directory.
    assert!(elements.iter().any(|e| e.qualified_name == "Vehicle::NativeElsewhere"));

    let result = validate_with_config(&elements, &ValidateConfig::with_model_root(&root));
    let errors: Vec<_> = result.findings.iter().filter(|f| f.severity == Severity::Error).collect();
    assert!(errors.is_empty(), "expected no errors, got: {errors:#?}");

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn plugin_self_reported_diagnostics_fold_into_w532() {
    let _serial = serial_guard();
    let root = tempdir();
    let wasm = toy_wasm_path();
    base_fixture(&root, &plugins_toml(&wasm.display().to_string()));
    // A second .sysml file with no recognisable `part def`/`requirement def`
    // block — the toy plugin appends a diagnostic for it (still succeeds).
    write(&root, "SysML2Legacy/Unparseable.sysml", "not a recognised construct at all\n");

    let elements = walk_model(&root).unwrap();
    // The well-formed file's elements still make it through.
    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy::PressureSensor"));

    let result = validate_with_config(&elements, &ValidateConfig::with_model_root(&root));
    assert!(
        result.findings.iter().any(|f| f.code == "W532"),
        "expected W532 for the plugin's self-reported diagnostic, got: {:#?}",
        result.findings
    );

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn foreign_format_on_a_non_index_file_is_ignored_not_destructive() {
    let _serial = serial_guard();
    let root = tempdir();
    let wasm = toy_wasm_path();
    // `foreignFormat:` on a plain element (not `_index.md`) must not be treated
    // as a package anchor — it has no subtree to own, and must not cause any
    // sibling native element to be stripped from the graph.
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, ".syscribe.toml", &plugins_toml(&wasm.display().to_string()));
    write(
        &root,
        "Misplaced/NotAPackage.md",
        "---\ntype: Part\nname: NotAPackage\nforeignFormat: sysmlv2\n---\n",
    );
    write(&root, "Misplaced/Sibling.md", "---\ntype: Part\nname: Sibling\n---\n");

    let elements = walk_model(&root).unwrap();
    assert!(elements.iter().any(|e| e.qualified_name == "Misplaced::NotAPackage"));
    assert!(elements.iter().any(|e| e.qualified_name == "Misplaced::Sibling"));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn hung_plugin_is_interrupted_not_left_hanging() {
    let _serial = serial_guard();
    // tests/fixtures/plugins/hang-toy.wasm's parse() never returns (built from
    // tests/fixtures/plugins-src/hang-toy/) — proves timeout_ms/fuel actually
    // interrupt a stuck plugin rather than hanging walk_model/validate forever.
    let root = tempdir();
    let hang_wasm = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/plugins/hang-toy.wasm");
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        ".syscribe.toml",
        &format!("[plugins.hang]\nwasm = \"{}\"\ntimeout_ms = 300\n", hang_wasm.display()),
    );
    write(
        &root,
        "Foreign/_index.md",
        "---\ntype: Package\nname: Foreign\nforeignFormat: hang\n---\n",
    );

    // This proves "doesn't hang forever", not "interrupted within N ms" — the
    // 300ms timeout_ms above is what's actually under test; wasmtime's uncached
    // JIT compilation of the ~2.4MB module already costs several seconds on its
    // own, and running alongside other WASM-compiling tests in the same `cargo
    // test` process can add real CPU contention on top of that. The bound here
    // just needs to be well short of "actually hung" (minutes/forever).
    let start = std::time::Instant::now();
    let elements = walk_model(&root).unwrap();
    let elapsed = start.elapsed();
    assert!(
        elapsed < std::time::Duration::from_secs(60),
        "walk_model must not hang indefinitely on a stuck plugin — took {elapsed:?}"
    );

    let result = validate_with_config(&elements, &ValidateConfig::with_model_root(&root));
    assert!(
        result.findings.iter().any(|f| f.code == "W530"),
        "expected W530 for the interrupted plugin, got: {:#?}",
        result.findings
    );
    let unrelated_errors: Vec<_> = result.findings.iter().filter(|f| f.severity == Severity::Error).collect();
    assert!(unrelated_errors.is_empty(), "no unrelated errors expected: {unrelated_errors:#?}");

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn invalid_wasm_module_produces_w530_not_a_crash() {
    let _serial = serial_guard();
    let root = tempdir();
    // A file that exists but isn't a valid wasm module.
    write(&root, "not-a-plugin.wasm", "this is not webassembly");
    base_fixture(&root, &plugins_toml("not-a-plugin.wasm"));

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &ValidateConfig::with_model_root(&root));

    assert!(
        result.findings.iter().any(|f| f.code == "W530"),
        "expected W530, got: {:#?}",
        result.findings
    );
    // The rest of the model still validates — no unrelated errors leaked in.
    let unrelated_errors: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .collect();
    assert!(
        unrelated_errors.is_empty(),
        "a plugin execution failure must not cascade into unrelated errors: {unrelated_errors:#?}"
    );

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn second_walk_model_call_on_unchanged_content_hits_the_cache() {
    let _serial = serial_guard();
    let root = tempdir();
    let wasm = toy_wasm_path();
    base_fixture(&root, &plugins_toml(&wasm.display().to_string()));

    let first_start = std::time::Instant::now();
    let first = walk_model(&root).unwrap();
    let first_elapsed = first_start.elapsed();
    assert!(first.iter().any(|e| e.qualified_name == "SysML2Legacy::PressureSensor"));
    assert!(
        root.join(".syscribe/cache/plugins.json").exists(),
        "a successful invocation must populate the on-disk cache"
    );

    let second_start = std::time::Instant::now();
    let second = walk_model(&root).unwrap();
    let second_elapsed = second_start.elapsed();
    assert!(second.iter().any(|e| e.qualified_name == "SysML2Legacy::PressureSensor"));

    // A cache hit skips wasmtime entirely (just a content hash + JSON file
    // read) — dramatically faster than the first, uncached, JIT-compiling
    // call. Not asserting a tight bound on the first call (JIT compile time
    // varies a lot under CI/contention, as observed in `hung_plugin_is_
    // interrupted_not_left_hanging`) — just that the second is fast in
    // absolute terms, which JIT compilation never is.
    assert!(
        second_elapsed < std::time::Duration::from_secs(1),
        "expected a cache hit to be fast; first call took {first_elapsed:?}, second took {second_elapsed:?}"
    );

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn changing_foreign_content_busts_the_cache() {
    let _serial = serial_guard();
    let root = tempdir();
    let wasm = toy_wasm_path();
    base_fixture(&root, &plugins_toml(&wasm.display().to_string()));

    let first = walk_model(&root).unwrap();
    assert!(first.iter().any(|e| e.qualified_name == "SysML2Legacy::PressureSensor"));
    assert!(!first.iter().any(|e| e.qualified_name == "SysML2Legacy::TemperatureSensor"));

    // Change the .sysml content — this must bust the cache, not keep serving
    // the first call's stale elements.
    write(
        &root,
        "SysML2Legacy/Sensor.sysml",
        "part def TemperatureSensor {\n  doc \"Measures cabin temperature.\"\n}\n",
    );

    let second = walk_model(&root).unwrap();
    assert!(
        second.iter().any(|e| e.qualified_name == "SysML2Legacy::TemperatureSensor"),
        "new content must be reflected, not served from a stale cache entry"
    );
    assert!(
        !second.iter().any(|e| e.qualified_name == "SysML2Legacy::PressureSensor"),
        "old content must not linger once the source changed"
    );

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn dry_run_never_reads_or_writes_the_cache() {
    let _serial = serial_guard();
    use syscribe_model::element::{ElementType, RawElement, RawFrontmatter};

    let root = tempdir();
    let wasm = toy_wasm_path();
    base_fixture(&root, &plugins_toml(&wasm.display().to_string()));

    // Hand-built, not from walk_model — dry_run only needs the one element
    // declaring `foreignFormat:` to locate the package directory. Going
    // through walk_model here would call apply_foreign_plugins on the normal
    // (cached) merge path first, making the "dry-run never writes the cache"
    // assertion below meaningless (the file would already exist for an
    // unrelated reason).
    let pkg_index = RawElement {
        qualified_name: "SysML2Legacy".to_string(),
        file_path: root.join("SysML2Legacy/_index.md").display().to_string(),
        frontmatter: RawFrontmatter {
            element_type: Some(ElementType::Package),
            foreign_format: Some("sysmlv2".to_string()),
            ..Default::default()
        },
        doc: String::new(),
        parse_issue: None,
        derived: Default::default(),
        derive_findings: Vec::new(),
    };

    let out = syscribe_model::plugins::dry_run("sysmlv2", &root, std::slice::from_ref(&pkg_index)).unwrap();
    assert!(out.contains("PressureSensor"));
    assert!(
        !root.join(".syscribe/cache/plugins.json").exists(),
        "dry-run must never populate the cache"
    );

    // A subsequent real walk_model call still populates it normally — proving
    // the absence above was specifically because of dry-run, not some other
    // reason (e.g. a test-fixture mistake).
    walk_model(&root).unwrap();
    assert!(root.join(".syscribe/cache/plugins.json").exists());

    std::fs::remove_dir_all(&root).ok();
}

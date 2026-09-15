//! Tests for `REQ-TRS-ID-007`: additional stable-ID prefixes configured via
//! `[ids.prefixes]` in `<model_root>/.syscribe.toml`.
//!
//! No end-to-end coverage existed for this mechanism before this file — see
//! `docs/validation/rules.md`'s "Additional stable-ID prefixes (W046)"
//! section for the documented contract this exercises: extras are additive
//! (the built-in prefix always stays valid), pure identity (only id
//! validation/resolution is affected), a malformed prefix or unknown-type
//! key is ignored with `W046` while well-formed siblings still take effect,
//! and a configured extra prefix participates in `E023` (`[ids] max_digits`)
//! identically to a built-in one.
//!
//! `set_extra_id_prefixes_by_type` (`resolver.rs`) installs the compiled
//! extras into a process-global registry as a side effect of
//! `ValidateConfig::with_model_root`. That registry is shared by every test
//! in this binary, so — unlike the other `tests/*.rs` fixtures, which don't
//! touch it — the tests here serialize on `REGISTRY_LOCK` to avoid one
//! test's `.syscribe.toml` clobbering another's mid-assertion.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::{validate_with_config, ValidationResult};
use syscribe_model::walker::walk_model;

static REGISTRY_LOCK: Mutex<()> = Mutex::new(());

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-id-prefixes-test-{}-{}",
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

fn validate(root: &Path) -> ValidationResult {
    let elements = walk_model(root).unwrap();
    validate_with_config(&elements, &ValidateConfig::with_model_root(root.to_path_buf()))
}

fn codes(result: &ValidationResult) -> Vec<&str> {
    result.findings.iter().map(|f| f.code).collect()
}

fn messages_for<'a>(result: &'a ValidationResult, code: &str) -> Vec<&'a str> {
    result
        .findings
        .iter()
        .filter(|f| f.code == code)
        .map(|f| f.message.as_str())
        .collect()
}

/// Baseline/control: with no `.syscribe.toml` at all, a non-built-in-prefixed
/// id on a `Requirement` raises `E006` — establishes what "configuring the
/// extra prefix changes this" is being contrasted against.
#[test]
fn without_any_config_a_foreign_prefixed_id_raises_e006() {
    let _guard = REGISTRY_LOCK.lock().unwrap();
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Req.md",
        "---\ntype: Requirement\nid: STK-SCHED-001\nname: Sched req\nstatus: draft\n---\nBody.\n",
    );
    let result = validate(&root);
    assert!(
        codes(&result).contains(&"E006"),
        "STK-* should not validate as a Requirement id with no [ids.prefixes] configured; findings: {:?}",
        codes(&result)
    );
}

/// A well-formed `[ids.prefixes]` entry makes the extra prefix a valid
/// `Requirement` id (no `E006`) while the built-in `REQ-*` prefix stays valid
/// too — additive, not a replacement — and raises no `W046`.
#[test]
fn configured_extra_prefix_avoids_e006_and_builtin_prefix_still_works() {
    let _guard = REGISTRY_LOCK.lock().unwrap();
    let root = tempdir();
    write(
        &root,
        ".syscribe.toml",
        "[ids.prefixes]\nRequirement = [\"STK\"]\n",
    );
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "StkReq.md",
        "---\ntype: Requirement\nid: STK-SCHED-001\nname: Stakeholder sched req\nstatus: draft\n---\nBody.\n",
    );
    write(
        &root,
        "BuiltinReq.md",
        "---\ntype: Requirement\nid: REQ-SCHED-001\nname: Builtin sched req\nstatus: draft\n---\nBody.\n",
    );
    let result = validate(&root);
    assert!(
        !codes(&result).contains(&"E006"),
        "both STK-* (configured extra) and REQ-* (built-in) should validate; findings: {:?}",
        codes(&result)
    );
    assert!(
        !codes(&result).contains(&"W046"),
        "a well-formed [ids.prefixes] entry should not raise W046; findings: {:?}",
        codes(&result)
    );
}

/// A malformed prefix string (lowercase — fails `^[A-Z][A-Z0-9]{1,11}$`) is
/// reported as `W046` and never installed: an id shaped to match it still
/// raises `E006`.
#[test]
fn malformed_prefix_raises_w046_and_is_never_installed() {
    let _guard = REGISTRY_LOCK.lock().unwrap();
    let root = tempdir();
    write(&root, ".syscribe.toml", "[ids.prefixes]\nRequirement = [\"stk\"]\n");
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Req.md",
        "---\ntype: Requirement\nid: stk-SCHED-001\nname: Sched req\nstatus: draft\n---\nBody.\n",
    );
    let result = validate(&root);
    assert!(
        codes(&result).contains(&"W046"),
        "malformed prefix 'stk' should raise W046; findings: {:?}",
        codes(&result)
    );
    assert!(
        messages_for(&result, "W046").iter().any(|m| m.contains("stk")),
        "W046 message should name the offending prefix; messages: {:?}",
        messages_for(&result, "W046")
    );
    assert!(
        codes(&result).contains(&"E006"),
        "id shaped for the malformed prefix should still be rejected (prefix never installed); findings: {:?}",
        codes(&result)
    );
}

/// A malformed prefix in the same list as a well-formed one only blocks
/// itself — the well-formed sibling still takes effect.
#[test]
fn malformed_prefix_does_not_block_a_wellformed_sibling() {
    let _guard = REGISTRY_LOCK.lock().unwrap();
    let root = tempdir();
    write(
        &root,
        ".syscribe.toml",
        "[ids.prefixes]\nRequirement = [\"stk\", \"SYS\"]\n",
    );
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Req.md",
        "---\ntype: Requirement\nid: SYS-SCHED-001\nname: Sys sched req\nstatus: draft\n---\nBody.\n",
    );
    let result = validate(&root);
    assert!(
        codes(&result).contains(&"W046"),
        "the malformed 'stk' entry should still raise W046; findings: {:?}",
        codes(&result)
    );
    assert!(
        !codes(&result).contains(&"E006"),
        "SYS-* should validate fine despite its malformed sibling 'stk'; findings: {:?}",
        codes(&result)
    );
}

/// An `[ids.prefixes]` key that isn't a recognised id-identified element type
/// is ignored and reported as `W046`, without affecting unrelated validation.
#[test]
fn unknown_type_key_raises_w046() {
    let _guard = REGISTRY_LOCK.lock().unwrap();
    let root = tempdir();
    write(
        &root,
        ".syscribe.toml",
        "[ids.prefixes]\nNotARealType = [\"FOO\"]\n",
    );
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Req.md",
        "---\ntype: Requirement\nid: REQ-SCHED-001\nname: Sched req\nstatus: draft\n---\nBody.\n",
    );
    let result = validate(&root);
    assert!(
        codes(&result).contains(&"W046"),
        "an unknown type key should raise W046; findings: {:?}",
        codes(&result)
    );
    assert!(
        messages_for(&result, "W046")
            .iter()
            .any(|m| m.contains("NotARealType") && m.contains("not an id-identified element type")),
        "W046 message should name the offending key; messages: {:?}",
        messages_for(&result, "W046")
    );
    assert!(
        !codes(&result).contains(&"E006"),
        "the unrelated, correctly-formed REQ-* id should still validate; findings: {:?}",
        codes(&result)
    );
}

/// An id using a configured extra prefix is still subject to `[ids]
/// max_digits` exactly like a built-in-prefixed id.
#[test]
fn extra_prefix_id_respects_configured_max_digits() {
    let _guard = REGISTRY_LOCK.lock().unwrap();
    let root = tempdir();
    write(
        &root,
        ".syscribe.toml",
        "[ids]\nmax_digits = 4\n[ids.prefixes]\nRequirement = [\"STK\"]\n",
    );
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Req.md",
        "---\ntype: Requirement\nid: STK-SCHED-12345\nname: Sched req\nstatus: draft\n---\nBody.\n",
    );
    let result = validate(&root);
    assert!(
        codes(&result).contains(&"E023"),
        "a 5-digit suffix should exceed the configured max_digits of 4 even under an extra prefix; findings: {:?}",
        codes(&result)
    );
}

/// `FeatureDef` is a no-suffix-required stable-id kind (`REQ-TRS-ID-006`): a
/// configured extra prefix for it needs no trailing digits either.
#[test]
fn feature_def_extra_prefix_needs_no_numeric_suffix() {
    let _guard = REGISTRY_LOCK.lock().unwrap();
    let root = tempdir();
    write(
        &root,
        ".syscribe.toml",
        "[ids.prefixes]\nFeatureDef = [\"FX\"]\n",
    );
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Feature.md",
        "---\ntype: FeatureDef\nid: FX-ABS\nname: Abs\ngroupKind: mandatory\n---\nBody.\n",
    );
    let result = validate(&root);
    assert!(
        !codes(&result).contains(&"E006"),
        "FX-ABS (no trailing digits) should validate as a FeatureDef id under the configured extra prefix; findings: {:?}",
        codes(&result)
    );
}

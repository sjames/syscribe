//! Tests for `REQ-TRS-HPLE-004`: the transitive closure of every
//! `isRequired: true`, no-`default:` parameter — of every `FeatureDef`
//! actually selected anywhere in a `Configuration`'s `subConfigurations:`
//! subtree, at any depth — that remains unbound after applying every
//! `parameterBindings:` entry from that `Configuration` down through every
//! tier already resolved beneath it, reported as the opt-in, `--deny`-
//! gateable `W513`. Never a hard error at an intermediate tier's own
//! isolated validation.
//!
//! Fixture pattern mirrors `hple_bindguard.rs` (tempdir + write helpers,
//! `walk_model` + `validate_with_config`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-hple-openparam-test-{}-{}",
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

fn codes(result: &syscribe_model::validator::ValidationResult) -> Vec<&str> {
    result.findings.iter().map(|f| f.code).collect()
}

fn cfg_with_root(root: &Path) -> ValidateConfig {
    ValidateConfig::with_model_root(root.to_path_buf())
}

fn repos_toml(alias_path: &Path) -> String {
    format!(
        "[repos.peer]\npath = \"{}\"\n",
        alias_path.display().to_string().replace('\\', "\\\\")
    )
}

/// A peer repo with an optional `Cargo` feature carrying: an open (isRequired,
/// no default) `capacityKg`; a fixed `fixedRate` (`value:`, never "open" at
/// all); and a runtime-bound `liveTemp` (opts out of the completeness check
/// per REQ-TRS-PARAM-004, mirroring `W017`'s own carve-out). `cargo_selected`
/// controls whether the peer's own `Configuration` selects `Cargo` at all.
fn write_peer_repo(conf_id: &str, cargo_selected: bool) -> PathBuf {
    let peer_dir = tempdir();
    let mroot = peer_dir.join("model");
    write(&mroot, "_index.md", "---\ntype: Package\nname: PeerRoot\n---\n");
    write(
        &mroot,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-PEER-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &mroot,
        "Features/Cargo.md",
        "---\ntype: FeatureDef\nid: FEAT-PEER-CARGO\nname: Cargo\ngroupKind: optional\nparameters:\n  - name: capacityKg\n    type: ScalarValues::Real\n    range: \"0.5..5.0\"\n    isRequired: true\n  - name: fixedRate\n    type: ScalarValues::Real\n    value: 1.0\n  - name: liveTemp\n    type: ScalarValues::Real\n    isRequired: true\n    bindingTime: runtime\n---\n",
    );
    write(
        &mroot,
        "Configurations/PeerConf.md",
        &format!(
            "---\ntype: Configuration\nid: {conf_id}\nname: Peer configuration\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n  Features::Cargo: {cargo_selected}\n---\n"
        ),
    );
    peer_dir
}

/// A consolidating repo whose `Main` Configuration names `peer_conf_id` via
/// `subConfigurations:` and supplies `extra_bindings` in its own
/// `parameterBindings:`.
fn write_consolidator(peer_dir: &Path, peer_conf_id: &str, extra_bindings: &str) -> PathBuf {
    let root = tempdir();
    write(&root, ".syscribe.toml", &repos_toml(peer_dir));
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    let bindings = if extra_bindings.is_empty() {
        String::new()
    } else {
        format!("parameterBindings:\n{extra_bindings}")
    };
    write(
        &root,
        "Configurations/Main.md",
        &format!(
            "---\ntype: Configuration\nid: CONF-MAIN-OP-001\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: {peer_conf_id}\n{bindings}---\n"
        ),
    );
    root
}

#[test]
fn unbound_required_param_reachable_through_a_peer_is_w513() {
    let peer_dir = write_peer_repo("CONF-OP-PEER-001", true);
    let root = write_consolidator(&peer_dir, "CONF-OP-PEER-001", "");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(codes(&result).contains(&"W513"), "expected W513: {:#?}", result.findings);
    let f = result.findings.iter().find(|f| f.code == "W513").unwrap();
    assert!(f.file.ends_with("Main.md"), "{:#?}", f);
    assert!(f.message.contains("capacityKg"), "{}", f.message);
    assert_eq!(result.errors().count(), 0, "an open parameter is never a hard error: {:#?}", result.findings);
}

#[test]
fn bound_by_this_configuration_itself_is_not_w513() {
    let peer_dir = write_peer_repo("CONF-OP-PEER-002", true);
    let root = write_consolidator(&peer_dir, "CONF-OP-PEER-002", "  Features::Cargo.capacityKg: 2.0\n");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        !codes(&result).contains(&"W513"),
        "Main's own binding closes the parameter — no W513: {:#?}",
        result.findings
    );
}

#[test]
fn bound_by_a_nearer_local_intermediate_tier_is_not_w513() {
    // Main -> Sub (local) -> peer Owner. Sub closes the peer's open
    // parameter itself; from Main's perspective the whole subtree is closed.
    let peer_dir = write_peer_repo("CONF-OP-PEER-003", true);

    let root = tempdir();
    write(&root, ".syscribe.toml", &repos_toml(&peer_dir));
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &root,
        "Configurations/Sub.md",
        "---\ntype: Configuration\nid: CONF-OP-SUB-001\nname: Sub\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-OP-PEER-003\nparameterBindings:\n  Features::Cargo.capacityKg: 1.0\n---\n",
    );
    write(
        &root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-OP-MAIN-003\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-OP-SUB-001\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    let w513_on_main: Vec<_> =
        result.findings.iter().filter(|f| f.code == "W513" && f.file.ends_with("Main.md")).collect();
    assert!(
        w513_on_main.is_empty(),
        "Sub already closed it — Main must see the subtree as complete: {:#?}",
        result.findings
    );
    // Sub itself, evaluated independently, has nothing further to defer either.
    let w513_on_sub: Vec<_> =
        result.findings.iter().filter(|f| f.code == "W513" && f.file.ends_with("Sub.md")).collect();
    assert!(w513_on_sub.is_empty(), "{:#?}", result.findings);
}

#[test]
fn unselected_feature_never_contributes_w513() {
    // Cargo isn't selected by the peer at all — its parameters are out of
    // scope for this completeness check entirely (not "open", just N/A).
    let peer_dir = write_peer_repo("CONF-OP-PEER-004", false);
    let root = write_consolidator(&peer_dir, "CONF-OP-PEER-004", "");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        !codes(&result).contains(&"W513"),
        "an unselected feature's parameters must not appear in the closure: {:#?}",
        result.findings
    );
}

#[test]
fn fixed_and_runtime_params_never_contribute_w513() {
    // capacityKg is bound (closed); fixedRate (value:) and liveTemp
    // (bindingTime: runtime) are the only other parameters and neither is
    // ever "open" in this check's sense.
    let peer_dir = write_peer_repo("CONF-OP-PEER-005", true);
    let root = write_consolidator(&peer_dir, "CONF-OP-PEER-005", "  Features::Cargo.capacityKg: 2.0\n");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        !codes(&result).contains(&"W513"),
        "fixed and runtime-bound parameters must never appear in the closure: {:#?}",
        result.findings
    );
}

#[test]
fn open_param_two_tiers_down_with_nothing_closing_it_is_still_w513() {
    let tier2_dir = write_peer_repo("CONF-OP-TIER2-001", true);

    let tier1_dir = tempdir();
    let tier1_mroot = tier1_dir.join("model");
    write(&tier1_mroot, ".syscribe.toml", &repos_toml(&tier2_dir));
    write(&tier1_mroot, "_index.md", "---\ntype: Package\nname: Tier1Root\n---\n");
    write(
        &tier1_mroot,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-TIER1-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &tier1_mroot,
        "Configurations/Tier1Conf.md",
        "---\ntype: Configuration\nid: CONF-OP-TIER1-001\nname: Tier1\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-OP-TIER2-001\n---\n",
    );

    let root = write_consolidator(&tier1_dir, "CONF-OP-TIER1-001", "");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        codes(&result).contains(&"W513"),
        "a parameter left open two tiers down must still surface: {:#?}",
        result.findings
    );
}

#[test]
fn purely_local_subconfigurations_chain_never_raises_w513() {
    // No peer repo at all — one shared feature model, where an unbound
    // required parameter is already W017, unconditionally.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &root,
        "Features/Cargo.md",
        "---\ntype: FeatureDef\nid: FEAT-CARGO\nname: Cargo\ngroupKind: optional\nparameters:\n  - name: capacityKg\n    type: ScalarValues::Real\n    isRequired: true\n---\n",
    );
    write(
        &root,
        "Configurations/Sub.md",
        "---\ntype: Configuration\nid: CONF-OP-SUB-002\nname: Sub\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n  Features::Cargo: true\nparameterBindings:\n  Features::Cargo.capacityKg: 2.0\n---\n",
    );
    write(
        &root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-OP-MAIN-002\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n  Features::Cargo: true\nsubConfigurations: CONF-OP-SUB-002\nparameterBindings:\n  Features::Cargo.capacityKg: 3.0\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        !codes(&result).contains(&"W513"),
        "a purely local subConfigurations chain must never raise W513: {:#?}",
        result.findings
    );
}

#[test]
fn w513_is_gateable_via_deny_but_never_a_hard_error_on_its_own() {
    let peer_dir = write_peer_repo("CONF-OP-PEER-006", true);
    let root = write_consolidator(&peer_dir, "CONF-OP-PEER-006", "");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    // It is a Warning-severity finding, not an Error — --deny is what a CI
    // gate uses to escalate it, not this function itself.
    let f = result.findings.iter().find(|f| f.code == "W513").unwrap();
    assert_eq!(f.severity, syscribe_model::validator::Severity::Warning, "{:#?}", f);
    assert_eq!(result.errors().count(), 0, "{:#?}", result.findings);
}

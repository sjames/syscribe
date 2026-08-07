//! Tests for `REQ-TRS-HPLE-003`: a `parameterBindings:` entry reaching into a
//! `subConfigurations:`-consolidated subtree must target a parameter that is
//! genuinely **open**: the owning tier's `Configuration` actually selects
//! the feature (`E519` — the cross-tier extension of `E203`), and no nearer
//! tier on the path down to it has already supplied a value (`E523`).
//!
//! Fixture pattern mirrors `hple_parambind.rs`/`hple_subconfigurations.rs`
//! (tempdir + write helpers, `walk_model` + `validate_with_config`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-hple-bindguard-test-{}-{}",
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

/// A peer repo with one optional `Cargo` feature carrying an open (isRequired,
/// no default) `capacityKg` parameter. `cargo_selected` controls whether the
/// peer's own `Configuration` selects it; `own_binding` — if non-empty — is
/// spliced into the peer Configuration's own `parameterBindings:` (used to
/// construct the "already bound by a nearer/owning tier" scenarios).
fn write_peer_repo(conf_id: &str, cargo_selected: bool, own_binding: &str) -> PathBuf {
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
        "---\ntype: FeatureDef\nid: FEAT-PEER-CARGO\nname: Cargo\ngroupKind: optional\nparameters:\n  - name: capacityKg\n    type: ScalarValues::Real\n    range: \"0.5..5.0\"\n    isRequired: true\n---\n",
    );
    let bindings = if own_binding.is_empty() {
        String::new()
    } else {
        format!("parameterBindings:\n{own_binding}")
    };
    write(
        &mroot,
        "Configurations/PeerConf.md",
        &format!(
            "---\ntype: Configuration\nid: {conf_id}\nname: Peer configuration\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n  Features::Cargo: {cargo_selected}\n{bindings}---\n"
        ),
    );
    peer_dir
}

/// A consolidating repo whose `Main` Configuration names `peer_conf_id` via
/// `subConfigurations:` and supplies `extra_bindings` in its own
/// `parameterBindings:` (dotted keys under test).
fn write_consolidator(peer_dir: &Path, peer_conf_id: &str, extra_bindings: &str) -> PathBuf {
    let root = tempdir();
    write(&root, ".syscribe.toml", &repos_toml(peer_dir));
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &root,
        "Configurations/Main.md",
        &format!(
            "---\ntype: Configuration\nid: CONF-MAIN-BG-001\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: {peer_conf_id}\nparameterBindings:\n{extra_bindings}---\n"
        ),
    );
    root
}

// ── E519: cross-tier binding targets a feature the owning tier didn't select ──

#[test]
fn feature_not_selected_by_owning_peer_is_e519() {
    let peer_dir = write_peer_repo("CONF-BG-PEER-001", false, "");
    let root = write_consolidator(&peer_dir, "CONF-BG-PEER-001", "  Features::Cargo.capacityKg: 2.0\n");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(codes(&result).contains(&"E519"), "expected E519: {:#?}", result.findings);
    let f = result.findings.iter().find(|f| f.code == "E519").unwrap();
    assert!(f.message.contains("Features::Cargo"), "{}", f.message);
}

#[test]
fn feature_selected_and_open_never_raises_e519_or_e523() {
    let peer_dir = write_peer_repo("CONF-BG-PEER-002", true, "");
    let root = write_consolidator(&peer_dir, "CONF-BG-PEER-002", "  Features::Cargo.capacityKg: 2.0\n");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        !codes(&result).iter().any(|c| *c == "E519" || *c == "E523"),
        "a genuinely open, selected parameter must not be flagged: {:#?}",
        result.findings
    );
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

// ── E523: double-bind of something a nearer tier already closed ────────────

#[test]
fn already_bound_by_the_owning_peer_itself_is_e523() {
    let peer_dir = write_peer_repo(
        "CONF-BG-PEER-003",
        true,
        "  Features::Cargo.capacityKg: 1.5\n",
    );
    let root = write_consolidator(&peer_dir, "CONF-BG-PEER-003", "  Features::Cargo.capacityKg: 2.0\n");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(codes(&result).contains(&"E523"), "expected E523: {:#?}", result.findings);
    let f = result.findings.iter().find(|f| f.code == "E523").unwrap();
    assert!(
        f.message.contains("CONF-BG-PEER-003"),
        "E523 message should name the nearer (owning) Configuration: {}",
        f.message
    );
    // The owning tier's own binding is legitimate on its own terms — this is
    // purely about the *second*, redundant supply from Main.
    assert!(
        !codes(&result).contains(&"E519"),
        "the feature IS selected by its owner; only E523 should fire: {:#?}",
        result.findings
    );
}

#[test]
fn already_bound_by_a_local_intermediate_tier_is_e523() {
    // Main -> Sub (local) -> peer Owner (cross-repo). Sub itself closes the
    // peer's open parameter via its own parameterBindings; Main then tries
    // to supply it too.
    let peer_dir = write_peer_repo("CONF-BG-PEER-004", true, "");

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
        "---\ntype: Configuration\nid: CONF-BG-SUB-001\nname: Sub\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-BG-PEER-004\nparameterBindings:\n  Features::Cargo.capacityKg: 1.0\n---\n",
    );
    write(
        &root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-BG-MAIN-004\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-BG-SUB-001\nparameterBindings:\n  Features::Cargo.capacityKg: 2.0\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    let e523: Vec<_> = result.findings.iter().filter(|f| f.code == "E523" && f.file.ends_with("Main.md")).collect();
    assert_eq!(e523.len(), 1, "expected exactly one E523 on Main: {:#?}", result.findings);
    assert!(
        e523[0].message.contains("CONF-BG-SUB-001"),
        "E523 should name the nearer local tier (Sub), not the peer owner: {}",
        e523[0].message
    );
}

#[test]
fn nearer_tier_wins_the_e523_label_over_the_owner() {
    // Three tiers: Main -> Tier1 (peer) -> Tier2 (Tier1's own peer, owns
    // Cargo). BOTH Tier1 and Tier2 bind capacityKg in their own
    // parameterBindings; Main's own attempt must be reported against Tier1
    // (nearer), not Tier2 (the owner) -- REQ-TRS-HPLE-003's "any one tier
    // along the path" phrasing, verified for the specific tier named.
    let tier2_dir = write_peer_repo(
        "CONF-BG-TIER2-001",
        true,
        "  Features::Cargo.capacityKg: 1.0\n",
    );

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
        "---\ntype: Configuration\nid: CONF-BG-TIER1-001\nname: Tier1\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-BG-TIER2-001\nparameterBindings:\n  Features::Cargo.capacityKg: 1.5\n---\n",
    );

    let root = write_consolidator(&tier1_dir, "CONF-BG-TIER1-001", "  Features::Cargo.capacityKg: 2.0\n");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    let e523: Vec<_> = result.findings.iter().filter(|f| f.code == "E523").collect();
    assert_eq!(e523.len(), 1, "expected exactly one E523: {:#?}", result.findings);
    assert!(
        e523[0].message.contains("CONF-BG-TIER1-001") && !e523[0].message.contains("CONF-BG-TIER2-001"),
        "the nearer tier (Tier1) should win the label over the owner (Tier2): {}",
        e523[0].message
    );
}

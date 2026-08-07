//! Tests for `REQ-TRS-HPLE-001`: a `Configuration` may declare
//! `subConfigurations:` naming one or more other `Configuration` elements —
//! reachable locally or via a loaded peer repo (§14) — each of which must
//! resolve to a real `Configuration` that is itself internally valid
//! (SAT-satisfiable and error-free) before it can be consolidated.
//!
//! Fixture pattern mirrors `sysmlv2_graceful_degradation.rs` (tempdir + write
//! helpers, `walk_model` + `validate`/`validate_with_config`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-hple-subconfig-test-{}-{}",
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

/// A minimal, single-mandatory-root feature model + one Configuration
/// selecting it — deliberately as small as possible so "validates cleanly"
/// assertions are not accidentally tripped by unrelated feature-model noise.
fn write_minimal_feature_model_and_config(root: &Path, conf_id: &str, conf_qname: &str) {
    write(root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        root,
        &format!("{conf_qname}.md"),
        &format!(
            "---\ntype: Configuration\nid: {conf_id}\nname: A configuration\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n"
        ),
    );
}

fn codes(result: &syscribe_model::validator::ValidationResult) -> Vec<&str> {
    result.findings.iter().map(|f| f.code).collect()
}

fn cfg_with_root(root: &Path) -> ValidateConfig {
    ValidateConfig::with_model_root(root.to_path_buf())
}

// ── No subConfigurations anywhere ⇒ inert (ADR-SYS-HPLE-001 Consequences) ──

#[test]
fn model_with_no_subconfigurations_is_unaffected() {
    let root = tempdir();
    write_minimal_feature_model_and_config(&root, "CONF-PLAIN-001", "Configurations/Plain");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        !codes(&result).iter().any(|c| c.starts_with("E51")),
        "unexpected HPLE finding on a model with no subConfigurations: {:#?}",
        result.findings
    );
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

// ── Local resolution ────────────────────────────────────────────────────────

#[test]
fn local_valid_target_validates_cleanly() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &root,
        "Configurations/Sub.md",
        "---\ntype: Configuration\nid: CONF-SUB-001\nname: Sub\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n",
    );
    write(
        &root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-MAIN-001\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-SUB-001\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn local_dangling_target_is_e516() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-MAIN-002\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-NOPE-999\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(codes(&result).contains(&"E516"), "expected E516: {:#?}", result.findings);
}

#[test]
fn local_non_configuration_target_is_e517() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-MAIN-003\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: FEAT-ROOT\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(codes(&result).contains(&"E517"), "expected E517: {:#?}", result.findings);
}

// ── Peer resolution (§14 [repos]) ───────────────────────────────────────────

/// Writes a peer repo whose model root is `<peer_dir>/model/`, and returns
/// `peer_dir` (the repo root passed as `path` in `[repos]`).
fn write_peer_repo(conf_id: &str, sub_selected: bool) -> PathBuf {
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
        "Features/Sub.md",
        "---\ntype: FeatureDef\nid: FEAT-PEER-SUB\nname: Sub\ngroupKind: mandatory\n---\n",
    );
    write(
        &mroot,
        "Configurations/PeerConf.md",
        &format!(
            "---\ntype: Configuration\nid: {conf_id}\nname: Peer configuration\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n  Features::Sub: {sub_selected}\n---\n"
        ),
    );
    peer_dir
}

fn write_local_consolidator(peer_dir: &Path, peer_conf_id: &str) -> PathBuf {
    let root = tempdir();
    write(
        &root,
        ".syscribe.toml",
        &format!(
            "[repos.peer]\npath = \"{}\"\n",
            peer_dir.display().to_string().replace('\\', "\\\\")
        ),
    );
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
            "---\ntype: Configuration\nid: CONF-MAIN-010\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: {peer_conf_id}\n---\n"
        ),
    );
    root
}

#[test]
fn peer_valid_configuration_validates_cleanly() {
    // sub_selected: true ⇒ mandatory child selected whenever its mandatory
    // parent is ⇒ a consistent, valid Configuration.
    let peer_dir = write_peer_repo("CONF-PEER-001", true);
    let root = write_local_consolidator(&peer_dir, "CONF-PEER-001");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert_eq!(
        result.errors().count(),
        0,
        "expected the peer Configuration to validate cleanly: {:#?}",
        result.findings
    );
}

#[test]
fn peer_invalid_configuration_is_caught_via_real_walk_and_validate() {
    // sub_selected: false while Sub is a mandatory child of the (selected)
    // Root ⇒ this Configuration's own selection violates the peer's feature
    // model (E225 in that repo's `check_feature_model_deep`). This is the
    // control proving the peer was genuinely walked and validated, not just
    // existence-checked: the *only* difference from the clean test above is
    // this boolean, on a file inside the peer repo.
    let peer_dir = write_peer_repo("CONF-PEER-002", false);
    let root = write_local_consolidator(&peer_dir, "CONF-PEER-002");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        codes(&result).contains(&"E518"),
        "expected E518 naming the invalid peer Configuration: {:#?}",
        result.findings
    );
    // The finding must be attached to the *consolidating* Configuration's file.
    let e518 = result.findings.iter().find(|f| f.code == "E518").unwrap();
    assert!(e518.file.ends_with("Configurations/Main.md"), "{:#?}", e518);
    assert!(
        e518.message.contains("CONF-PEER-002"),
        "E518 message should name the failing peer Configuration: {}",
        e518.message
    );
}

#[test]
fn peer_dangling_target_is_e516() {
    let peer_dir = write_peer_repo("CONF-PEER-003", true);
    let root = write_local_consolidator(&peer_dir, "CONF-DOES-NOT-EXIST-999");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(codes(&result).contains(&"E516"), "expected E516: {:#?}", result.findings);
}

#[test]
fn peer_non_configuration_target_is_e517() {
    let peer_dir = write_peer_repo("CONF-PEER-004", true);
    // Point subConfigurations at the peer's FeatureDef stable id instead of its Configuration.
    let root = write_local_consolidator(&peer_dir, "FEAT-PEER-ROOT");

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(codes(&result).contains(&"E517"), "expected E517: {:#?}", result.findings);
}

//! Tests for `REQ-TRS-HPLE-002`: `Configuration.parameterBindings:` reaches
//! transitively through a `subConfigurations:` subtree via ordinary qname
//! resolution — a dotted key may target a parameter belonging to a
//! `FeatureDef` reachable at any depth through `subConfigurations:`, local or
//! cross-repo, using that parameter's ordinary, already-mounted qname.
//!
//! Fixture pattern mirrors `hple_subconfigurations.rs` (tempdir + write
//! helpers, `walk_model` + `validate_with_config`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-hple-parambind-test-{}-{}",
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

/// A peer repo with a mandatory root feature and one optional child carrying
/// a bindable parameter (`capacityKg`, range 0.5..5.0, required, no default —
/// exactly the "descendant declares an open parameter with no upward-pointing
/// field" shape `ADR-SYS-HPLE-001` Decision 3 describes).
fn write_peer_repo(conf_id: &str) -> PathBuf {
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
        "---\ntype: FeatureDef\nid: FEAT-PEER-CARGO\nname: Cargo\ngroupKind: optional\nparameters:\n  - name: capacityKg\n    type: ScalarValues::Real\n    range: \"0.5..5.0\"\n    isRequired: true\n  - name: mode\n    type: ScalarValues::String\n    enumValues: [eco, fast]\n    isRequired: true\n  - name: fixedRate\n    type: ScalarValues::Real\n    value: 1.0\n---\n",
    );
    // The peer's own Configuration deliberately leaves `capacityKg`/`mode`
    // unbound (isRequired, no default: — ADR-SYS-HPLE-001 Decision 3's
    // "descendant declares an open parameter with no upward-pointing field")
    // so a consolidator above it can legally supply them. A required-and-
    // unbound param is only ever W017 (warning), never an error, so this
    // still validates cleanly on its own (REQ-TRS-HPLE-001's peer-validity
    // gate only checks `.errors()`).
    write(
        &mroot,
        "Configurations/PeerConf.md",
        &format!(
            "---\ntype: Configuration\nid: {conf_id}\nname: Peer configuration\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n  Features::Cargo: true\n---\n"
        ),
    );
    peer_dir
}

/// A consolidating repo whose `Main` Configuration names `peer_conf_id` via
/// `subConfigurations:` and supplies `extra_bindings` in its own
/// `parameterBindings:` (dotted keys under test).
fn write_local_consolidator(peer_dir: &Path, peer_conf_id: &str, extra_bindings: &str) -> PathBuf {
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
            "---\ntype: Configuration\nid: CONF-MAIN-020\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: {peer_conf_id}\nparameterBindings:\n{extra_bindings}\n---\n"
        ),
    );
    root
}

// ── Cross-repo transitive resolution ────────────────────────────────────────

#[test]
fn dotted_key_resolves_a_peer_feature_param_one_tier_down() {
    let peer_dir = write_peer_repo("CONF-PB-PEER-001");
    let root = write_local_consolidator(
        &peer_dir,
        "CONF-PB-PEER-001",
        "  Features::Cargo.capacityKg: 3.5\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        !codes(&result).contains(&"E222"),
        "a peer FeatureDef param reachable via subConfigurations must resolve, not E222: {:#?}",
        result.findings
    );
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn dotted_key_to_a_feature_not_reachable_via_subconfigurations_still_e222() {
    // Same peer repo exists and is even walkable, but Main declares no
    // subConfigurations at all — the peer's FeatureDef must stay unreachable.
    let peer_dir = write_peer_repo("CONF-PB-PEER-002");
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
        "---\ntype: Configuration\nid: CONF-MAIN-021\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nparameterBindings:\n  Features::Cargo.capacityKg: 3.5\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        codes(&result).contains(&"E222"),
        "a FeatureDef in a repo that is loaded but never named by subConfigurations must stay unresolved: {:#?}",
        result.findings
    );
}

#[test]
fn range_violation_on_a_transitively_resolved_param_is_e205() {
    let peer_dir = write_peer_repo("CONF-PB-PEER-003");
    let root = write_local_consolidator(
        &peer_dir,
        "CONF-PB-PEER-003",
        "  Features::Cargo.capacityKg: 99.0\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        codes(&result).contains(&"E205"),
        "an out-of-range value bound through subConfigurations must still be caught: {:#?}",
        result.findings
    );
}

#[test]
fn enum_violation_on_a_transitively_resolved_param_is_e206() {
    let peer_dir = write_peer_repo("CONF-PB-PEER-004");
    let root = write_local_consolidator(
        &peer_dir,
        "CONF-PB-PEER-004",
        "  Features::Cargo.mode: turbo\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        codes(&result).contains(&"E206"),
        "an enumValues violation bound through subConfigurations must still be caught: {:#?}",
        result.findings
    );
}

#[test]
fn fixed_param_bound_through_subconfigurations_is_e204() {
    let peer_dir = write_peer_repo("CONF-PB-PEER-005");
    let root = write_local_consolidator(
        &peer_dir,
        "CONF-PB-PEER-005",
        "  Features::Cargo.fixedRate: 2.0\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        codes(&result).contains(&"E204"),
        "overriding a fixed (value:) param through subConfigurations must still be caught: {:#?}",
        result.findings
    );
}

#[test]
fn transitively_resolved_binding_never_raises_e203() {
    // E203 ("feature not selected") is scoped to this Configuration's own
    // local `features:` selection map — a peer's own feature selection is the
    // peer's business (already gated by REQ-TRS-HPLE-001's peer-validity
    // check), not something `Main` can be said to select or not. Whether this
    // specific cross-tier binding is *permitted* is REQ-TRS-HPLE-003's job.
    let peer_dir = write_peer_repo("CONF-PB-PEER-006");
    let root = write_local_consolidator(
        &peer_dir,
        "CONF-PB-PEER-006",
        "  Features::Cargo.capacityKg: 3.5\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        !codes(&result).contains(&"E203"),
        "a transitively-resolved binding must not be flagged 'feature not selected': {:#?}",
        result.findings
    );
}

// ── Multi-tier depth ─────────────────────────────────────────────────────────

#[test]
fn dotted_key_resolves_two_tiers_down_through_a_peers_own_subconfigurations() {
    // Tier 2 (bottom): the peer repo with the bindable FeatureDef.
    let tier2_dir = write_peer_repo("CONF-PB-TIER2-001");

    // Tier 1 (middle): its own repo, consolidating tier 2 via its own
    // subConfigurations + its own [repos] table — nothing shared with tier 0.
    let tier1_dir = tempdir();
    let tier1_mroot = tier1_dir.join("model");
    // `.syscribe.toml` must sit alongside tier1's own model root (the path
    // `ValidateConfig::with_model_root` actually reads it from once this repo
    // is loaded as a peer below) — not at the outer repo-root/`path` level.
    write(
        &tier1_mroot,
        ".syscribe.toml",
        &format!(
            "[repos.peer]\npath = \"{}\"\n",
            tier2_dir.display().to_string().replace('\\', "\\\\")
        ),
    );
    write(&tier1_mroot, "_index.md", "---\ntype: Package\nname: Tier1Root\n---\n");
    write(
        &tier1_mroot,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-TIER1-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &tier1_mroot,
        "Configurations/Tier1Conf.md",
        "---\ntype: Configuration\nid: CONF-PB-TIER1-001\nname: Tier1\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-PB-TIER2-001\n---\n",
    );

    // Tier 0 (top): consolidates tier 1, and binds a parameter that lives two
    // hops down in tier 2 — using its ordinary, already-mounted qname, no new
    // addressing syntax (REQ-TRS-HPLE-002's core claim).
    let root = write_local_consolidator(
        &tier1_dir,
        "CONF-PB-TIER1-001",
        "  Features::Cargo.capacityKg: 4.0\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        !codes(&result).contains(&"E222"),
        "a param two subConfigurations hops down must still resolve: {:#?}",
        result.findings
    );
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

// ── Local (same-model) subConfigurations — regression guard ────────────────
//
// A purely local `subConfigurations:` chain shares one `elements` slice —
// unlike the cross-repo case, there is only one feature model/SAT instance
// for the whole repo (`check_feature_model[_deep]` is not scoped per
// `Configuration`), so a locally-reachable `FeatureDef`'s parameters were
// already resolvable through the flat, whole-model `feature_params` table
// before this change; this is a regression guard, not new-capability
// coverage. `Main` still selects the feature itself (E203 stays meaningful
// within one shared feature model, unlike across a repo boundary where the
// consolidating tier structurally cannot see the peer's own feature tree).

#[test]
fn dotted_key_resolves_a_local_subconfigurations_targets_feature_param() {
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
        "---\ntype: FeatureDef\nid: FEAT-CARGO\nname: Cargo\ngroupKind: optional\nparameters:\n  - name: capacityKg\n    type: ScalarValues::Real\n    range: \"0.5..5.0\"\n    isRequired: true\n---\n",
    );
    write(
        &root,
        "Configurations/Sub.md",
        "---\ntype: Configuration\nid: CONF-PB-SUB-001\nname: Sub\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n  Features::Cargo: true\nparameterBindings:\n  Features::Cargo.capacityKg: 2.0\n---\n",
    );
    write(
        &root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-PB-MAIN-001\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n  Features::Cargo: true\nsubConfigurations: CONF-PB-SUB-001\nparameterBindings:\n  Features::Cargo.capacityKg: 3.5\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

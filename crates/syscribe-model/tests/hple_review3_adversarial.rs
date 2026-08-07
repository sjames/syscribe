//! Independent third-round adversarial review of REQ-TRS-HPLE-001's
//! topological-sort transitivity fix (commit 1049ca0) and the mixed
//! local+peer subConfigurations surface area neither prior round tested.
//!
//! Fixture helpers deliberately re-derived rather than imported from
//! `hple_subconfigurations.rs`, so nothing here can pass by accidentally
//! sharing a bug with the implementer's own harness.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-hple-review3-{}-{}",
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

fn write_features(root: &Path) {
    write(root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
}

fn write_chain_link(root: &Path, qname: &str, id: &str, sub_of: Option<&str>) {
    let sub_line = sub_of
        .map(|t| format!("subConfigurations: {t}\n"))
        .unwrap_or_default();
    write(
        root,
        &format!("{qname}.md"),
        &format!(
            "---\ntype: Configuration\nid: {id}\nname: {id}\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n{sub_line}---\n"
        ),
    );
}

/// Writes a peer repo with a genuinely valid Configuration.
fn write_valid_peer_repo(conf_id: &str) -> PathBuf {
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
        "Configurations/PeerConf.md",
        &format!(
            "---\ntype: Configuration\nid: {conf_id}\nname: Peer configuration\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n"
        ),
    );
    peer_dir
}

/// Writes a peer repo with a Configuration that is genuinely INvalid
/// (violates its own feature model's mandatory-parent-implies-child rule).
fn write_broken_peer_repo(conf_id: &str) -> PathBuf {
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
            "---\ntype: Configuration\nid: {conf_id}\nname: Peer configuration\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n  Features::Sub: false\n---\n"
        ),
    );
    peer_dir
}

fn write_repos_toml(root: &Path, alias: &str, peer_dir: &Path) {
    write(
        root,
        ".syscribe.toml",
        &format!(
            "[repos.{alias}]\npath = \"{}\"\n",
            peer_dir.display().to_string().replace('\\', "\\\\")
        ),
    );
}

// ═══════════════════════════════════════════════════════════════════════
// PRIORITY 1: mixed local + peer entries on the SAME Configuration
// ═══════════════════════════════════════════════════════════════════════

/// M has subConfigurations: [LocalBroken, PeerGood]. Local entry must be
/// flagged E518 (or E201 propagated); peer entry (genuinely valid) must NOT
/// spawn a spurious finding.
#[test]
fn mixed_list_local_broken_peer_valid_only_local_flagged() {
    let root = tempdir();
    write_features(&root);

    let peer_dir = write_valid_peer_repo("CONF-PEER-GOOD-001");
    write_repos_toml(&root, "peer", &peer_dir);

    // Local broken target: missing `status:` -> E201.
    write(
        &root,
        "Configurations/LocalBroken.md",
        "---\ntype: Configuration\nid: CONF-LOCALBROKEN-001\nname: LocalBroken\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n",
    );

    write(
        &root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-MAIN-MIX-001\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: [CONF-LOCALBROKEN-001, CONF-PEER-GOOD-001]\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    // 1. Local problem flagged.
    let main_e518s: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.code == "E518" && f.file.ends_with("Main.md"))
        .collect();
    assert!(
        main_e518s.iter().any(|f| f.message.contains("CONF-LOCALBROKEN-001")),
        "expected E518 on Main naming the broken LOCAL target: {:#?}",
        result.findings
    );

    // 2. No spurious finding for the genuinely-valid peer target.
    assert!(
        !main_e518s.iter().any(|f| f.message.contains("CONF-PEER-GOOD-001")),
        "peer target is genuinely valid -- must NOT be named in any E518: {:#?}",
        main_e518s
    );
    assert!(
        !codes(&result).contains(&"E516"),
        "peer target resolves fine -- no E516 expected: {:#?}",
        result.findings
    );

    // 3. Exactly one E518 on Main (not two, not zero) -- neither path
    // interferes with or masks the other.
    assert_eq!(
        main_e518s.len(),
        1,
        "expected exactly one E518 on Main (local problem only): {:#?}",
        main_e518s
    );
}

/// Inverse: M has subConfigurations: [LocalGood, PeerBroken]. Peer entry
/// must be flagged; local entry (genuinely valid) must NOT spawn a
/// spurious finding.
#[test]
fn mixed_list_local_valid_peer_broken_only_peer_flagged() {
    let root = tempdir();
    write_features(&root);

    let peer_dir = write_broken_peer_repo("CONF-PEER-BAD-001");
    write_repos_toml(&root, "peer", &peer_dir);

    write(
        &root,
        "Configurations/LocalGood.md",
        "---\ntype: Configuration\nid: CONF-LOCALGOOD-001\nname: LocalGood\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n",
    );

    write(
        &root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-MAIN-MIX-002\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: [CONF-LOCALGOOD-001, CONF-PEER-BAD-001]\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    let main_e518s: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.code == "E518" && f.file.ends_with("Main.md"))
        .collect();

    assert!(
        main_e518s.iter().any(|f| f.message.contains("CONF-PEER-BAD-001")),
        "expected E518 on Main naming the broken PEER target: {:#?}",
        result.findings
    );
    assert!(
        !main_e518s.iter().any(|f| f.message.contains("CONF-LOCALGOOD-001")),
        "local target is genuinely valid -- must NOT be named in any E518: {:#?}",
        main_e518s
    );
    assert_eq!(
        main_e518s.len(),
        1,
        "expected exactly one E518 on Main (peer problem only): {:#?}",
        main_e518s
    );

    // LocalGood itself must validate clean (no E201 or similar).
    let local_good_errors: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.severity == syscribe_model::validator::Severity::Error && f.file.ends_with("LocalGood.md"))
        .collect();
    assert!(
        local_good_errors.is_empty(),
        "LocalGood is genuinely valid -- expected no errors on its own file: {:#?}",
        local_good_errors
    );
}

/// A broken local target that is itself several tiers into a local chain
/// (reusing the transitivity machinery), alongside a genuinely valid peer
/// target in the SAME subConfigurations list. Confirms the transitive local
/// finding still surfaces correctly even with a peer entry present too.
#[test]
fn mixed_list_transitive_local_chain_plus_valid_peer() {
    let root = tempdir();
    write_features(&root);

    let peer_dir = write_valid_peer_repo("CONF-PEER-GOOD-002");
    write_repos_toml(&root, "peer", &peer_dir);

    // 3-level local chain: Z (broken) <- Y (sub: Z) <- X (sub: [Y, peer]).
    // Naming deliberately reversed vs dependency order.
    write(
        &root,
        "Configurations/AAA_Z.md",
        "---\ntype: Configuration\nid: CONF-MIXCHAINZ-001\nname: Z\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n",
    );
    write_chain_link(&root, "Configurations/BBB_Y", "CONF-MIXCHAINY-001", Some("CONF-MIXCHAINZ-001"));

    write(
        &root,
        "Configurations/CCC_X.md",
        "---\ntype: Configuration\nid: CONF-MIXCHAINX-001\nname: X\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: [CONF-MIXCHAINY-001, CONF-PEER-GOOD-002]\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    // Z's own E201.
    assert!(
        result.findings.iter().any(|f| f.code == "E201"),
        "expected Z's own E201: {:#?}",
        result.findings
    );

    // Y gets E518 naming Z.
    let y_e518 = result
        .findings
        .iter()
        .find(|f| f.code == "E518" && f.file.ends_with("BBB_Y.md"));
    assert!(y_e518.is_some(), "expected Y to get E518 naming broken Z: {:#?}", result.findings);

    // X (which ALSO has the valid peer entry in its list) must still get
    // its own transitive E518 naming Y.
    let x_e518s: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.code == "E518" && f.file.ends_with("CCC_X.md"))
        .collect();
    assert!(
        x_e518s.iter().any(|f| f.message.contains("CONF-MIXCHAINY-001")),
        "expected X to get a transitive E518 naming Y even with a peer entry present too: {:#?}",
        result.findings
    );
    // And no spurious finding naming the valid peer.
    assert!(
        !x_e518s.iter().any(|f| f.message.contains("CONF-PEER-GOOD-002")),
        "valid peer entry must not be spuriously flagged on X: {:#?}",
        x_e518s
    );
    assert_eq!(
        x_e518s.len(),
        1,
        "expected exactly one E518 on X (the transitive local one only): {:#?}",
        x_e518s
    );
}

// ═══════════════════════════════════════════════════════════════════════
// PRIORITY 2: independent verification of the topological fix (fresh
// fixtures, both forward and reverse file-naming orders, plus a 5-level
// chain beyond what was tested).
// ═══════════════════════════════════════════════════════════════════════

fn assert_chain_all_levels_flagged(root: &Path, links: &[(&str, &str)]) {
    // links: [(file_stem_without_ext, id)], ordered leaf-first (deepest
    // dependency first) as in write order; only the first link is broken.
    let elements = walk_model(root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(root));

    assert!(
        result.findings.iter().any(|f| f.code == "E201"),
        "expected the leaf's own E201: {:#?}",
        result.findings
    );

    for (i, (stem, _id)) in links.iter().enumerate().skip(1) {
        let file_match = format!("{stem}.md");
        let e518 = result
            .findings
            .iter()
            .find(|f| f.code == "E518" && f.file.ends_with(&file_match));
        assert!(
            e518.is_some(),
            "expected level {} ({}) to get E518: {:#?}",
            i,
            stem,
            result.findings
        );
    }
}

/// Fresh 3-level chain, forward file-naming order (alphabetical = dependency
/// order this time, the OPPOSITE bias from the implementer's own test).
#[test]
fn independent_3level_chain_forward_naming_order() {
    let root = tempdir();
    write_features(&root);

    write(
        &root,
        "Configurations/L1_Leaf.md",
        "---\ntype: Configuration\nid: CONF-IND3-101\nname: Leaf\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n",
    );
    write_chain_link(&root, "Configurations/L2_Mid", "CONF-IND3-102", Some("CONF-IND3-101"));
    write_chain_link(&root, "Configurations/L3_Root", "CONF-IND3-103", Some("CONF-IND3-102"));

    assert_chain_all_levels_flagged(
        &root,
        &[
            ("L1_Leaf", "CONF-IND3-101"),
            ("L2_Mid", "CONF-IND3-102"),
            ("L3_Root", "CONF-IND3-103"),
        ],
    );
}

/// Same 3-level chain, reverse file-naming order (leaf sorts LAST
/// alphabetically -- opposite of both the forward test above and the
/// implementer's own test, which had the leaf sort first).
#[test]
fn independent_3level_chain_reverse_naming_order() {
    let root = tempdir();
    write_features(&root);

    write(
        &root,
        "Configurations/Z_Leaf.md",
        "---\ntype: Configuration\nid: CONF-IND3R-101\nname: Leaf\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n",
    );
    write_chain_link(&root, "Configurations/M_Mid", "CONF-IND3R-102", Some("CONF-IND3R-101"));
    write_chain_link(&root, "Configurations/A_Root", "CONF-IND3R-103", Some("CONF-IND3R-102"));

    assert_chain_all_levels_flagged(
        &root,
        &[
            ("Z_Leaf", "CONF-IND3R-101"),
            ("M_Mid", "CONF-IND3R-102"),
            ("A_Root", "CONF-IND3R-103"),
        ],
    );
}

/// Fresh 4-level chain, forward naming order.
#[test]
fn independent_4level_chain_forward_naming_order() {
    let root = tempdir();
    write_features(&root);

    write(
        &root,
        "Configurations/L1_Leaf.md",
        "---\ntype: Configuration\nid: CONF-IND4-101\nname: L1\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n",
    );
    write_chain_link(&root, "Configurations/L2", "CONF-IND4-102", Some("CONF-IND4-101"));
    write_chain_link(&root, "Configurations/L3", "CONF-IND4-103", Some("CONF-IND4-102"));
    write_chain_link(&root, "Configurations/L4_Root", "CONF-IND4-104", Some("CONF-IND4-103"));

    assert_chain_all_levels_flagged(
        &root,
        &[
            ("L1_Leaf", "CONF-IND4-101"),
            ("L2", "CONF-IND4-102"),
            ("L3", "CONF-IND4-103"),
            ("L4_Root", "CONF-IND4-104"),
        ],
    );
}

/// Fresh 4-level chain, reverse naming order.
#[test]
fn independent_4level_chain_reverse_naming_order() {
    let root = tempdir();
    write_features(&root);

    write(
        &root,
        "Configurations/Z1_Leaf.md",
        "---\ntype: Configuration\nid: CONF-IND4R-101\nname: L1\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n",
    );
    write_chain_link(&root, "Configurations/Y2", "CONF-IND4R-102", Some("CONF-IND4R-101"));
    write_chain_link(&root, "Configurations/X3", "CONF-IND4R-103", Some("CONF-IND4R-102"));
    write_chain_link(&root, "Configurations/A4_Root", "CONF-IND4R-104", Some("CONF-IND4R-103"));

    assert_chain_all_levels_flagged(
        &root,
        &[
            ("Z1_Leaf", "CONF-IND4R-101"),
            ("Y2", "CONF-IND4R-102"),
            ("X3", "CONF-IND4R-103"),
            ("A4_Root", "CONF-IND4R-104"),
        ],
    );
}

/// 5-level chain -- one tier beyond anything tested by the implementer, to
/// gain confidence the fix is genuinely general (a real topological sort)
/// rather than coincidentally sufficient for exactly 3 and 4 levels.
#[test]
fn independent_5level_chain_propagates_transitively() {
    let root = tempdir();
    write_features(&root);

    write(
        &root,
        "Configurations/Q1_Leaf.md",
        "---\ntype: Configuration\nid: CONF-IND5-101\nname: L1\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n",
    );
    write_chain_link(&root, "Configurations/Q2", "CONF-IND5-102", Some("CONF-IND5-101"));
    write_chain_link(&root, "Configurations/Q3", "CONF-IND5-103", Some("CONF-IND5-102"));
    write_chain_link(&root, "Configurations/Q4", "CONF-IND5-104", Some("CONF-IND5-103"));
    write_chain_link(&root, "Configurations/Q5_Root", "CONF-IND5-105", Some("CONF-IND5-104"));

    assert_chain_all_levels_flagged(
        &root,
        &[
            ("Q1_Leaf", "CONF-IND5-101"),
            ("Q2", "CONF-IND5-102"),
            ("Q3", "CONF-IND5-103"),
            ("Q4", "CONF-IND5-104"),
            ("Q5_Root", "CONF-IND5-105"),
        ],
    );
}

// ═══════════════════════════════════════════════════════════════════════
// PRIORITY 3: cycle detection under the topological approach
// ═══════════════════════════════════════════════════════════════════════

/// Simple 2-node local cycle, fresh fixture: A <-> B, both otherwise fine.
/// Must be reported gracefully (E518 per implementer's report), no panic,
/// no hang.
#[test]
fn simple_2node_cycle_reports_gracefully() {
    let root = tempdir();
    write_features(&root);
    write_chain_link(&root, "Configurations/Alpha", "CONF-CYC2-101", Some("CONF-CYC2-102"));
    write_chain_link(&root, "Configurations/Beta", "CONF-CYC2-102", Some("CONF-CYC2-101"));

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    assert!(
        result.findings.iter().any(|f| f.code == "E518"),
        "expected the 2-node cycle to be reported as E518: {:#?}",
        result.findings
    );
}

/// Mixed case: a local cycle among X<->Y coexists with a separate, valid
/// local chain P->Q->R with no relation to X/Y. The cycle must be reported
/// for X/Y while P/Q/R still validate cleanly -- NOT swept into the cycle
/// report or otherwise disturbed by Kahn's leftover-set handling.
#[test]
fn mixed_cycle_and_unrelated_valid_chain_are_independent() {
    let root = tempdir();
    write_features(&root);

    // Cycle: X <-> Y.
    write_chain_link(&root, "Configurations/CycX", "CONF-MIXCYC-101", Some("CONF-MIXCYC-102"));
    write_chain_link(&root, "Configurations/CycY", "CONF-MIXCYC-102", Some("CONF-MIXCYC-101"));

    // Unrelated, entirely valid chain: R (leaf, valid) <- Q (sub: R) <- P (sub: Q).
    write_chain_link(&root, "Configurations/ChainR", "CONF-MIXCYC-103", None);
    write_chain_link(&root, "Configurations/ChainQ", "CONF-MIXCYC-104", Some("CONF-MIXCYC-103"));
    write_chain_link(&root, "Configurations/ChainP", "CONF-MIXCYC-105", Some("CONF-MIXCYC-104"));

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    // Cycle reported, attached to X and/or Y.
    let cyc_e518: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.code == "E518" && (f.file.ends_with("CycX.md") || f.file.ends_with("CycY.md")))
        .collect();
    assert!(!cyc_e518.is_empty(), "expected the X<->Y cycle to be reported: {:#?}", result.findings);

    // P, Q, R must NOT get any E518/E51x findings -- the unrelated valid
    // chain must not be swept up into the cycle's leftover handling.
    for stem in ["ChainP", "ChainQ", "ChainR"] {
        let bad: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.file.ends_with(&format!("{stem}.md")) && f.code.starts_with("E51"))
            .collect();
        assert!(
            bad.is_empty(),
            "unrelated valid chain member {} must not be affected by the X<->Y cycle: {:#?}",
            stem,
            bad
        );
    }
    assert_eq!(result.errors().count(), cyc_e518.len(), "expected ONLY the cycle-related errors: {:#?}", result.findings);
}

/// A local cycle with an outside consolidator: Z has subConfigurations:
/// [X], where X is part of the X<->Y cycle. Z must get its own finding
/// reflecting that its dependency is itself broken/circular -- the cycle
/// must not "swallow" the propagation.
#[test]
fn cycle_with_outside_consolidator_still_propagates() {
    let root = tempdir();
    write_features(&root);

    write_chain_link(&root, "Configurations/CycX", "CONF-OUTCYC-101", Some("CONF-OUTCYC-102"));
    write_chain_link(&root, "Configurations/CycY", "CONF-OUTCYC-102", Some("CONF-OUTCYC-101"));
    write_chain_link(&root, "Configurations/Outside_Z", "CONF-OUTCYC-103", Some("CONF-OUTCYC-101"));

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    // The cycle itself reported.
    assert!(
        result
            .findings
            .iter()
            .any(|f| f.code == "E518" && (f.file.ends_with("CycX.md") || f.file.ends_with("CycY.md"))),
        "expected the X<->Y cycle to be reported: {:#?}",
        result.findings
    );

    // Z, which depends on X (part of the cycle), must ALSO get its own
    // E518 -- the outside consolidator's dependency is broken/circular.
    let z_e518 = result
        .findings
        .iter()
        .find(|f| f.code == "E518" && f.file.ends_with("Outside_Z.md"));
    assert!(
        z_e518.is_some(),
        "expected Z (consolidates X, which is part of a cycle) to get its own E518 -- \
         propagation must not be swallowed by the cycle: {:#?}",
        result.findings
    );
    assert!(
        z_e518.unwrap().message.contains("CONF-OUTCYC-101"),
        "Z's E518 should name X, its immediate (circular) target: {}",
        z_e518.unwrap().message
    );
}

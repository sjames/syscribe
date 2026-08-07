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

/// Local-vs-peer parity (fix for the confirmed asymmetry gap): a local target
/// with an ordinary *structural* error (`E201` missing `status:` — something
/// `check_feature_model_deep` has no way to see, since it only reasons about
/// the Boolean feature layer) must surface as an `E518` on the consolidator,
/// exactly as a peer target with a validation error already does. Before the
/// fix this produced nothing at all on the consolidator.
#[test]
fn local_target_with_ordinary_structural_error_is_e518() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    // Missing `status:` -> E201, a plain structural error with nothing to do
    // with SAT/feature-model semantics.
    write(
        &root,
        "Configurations/BadSub.md",
        "---\ntype: Configuration\nid: CONF-BADSUB-001\nname: Bad Sub\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n",
    );
    write(
        &root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-MAIN-004\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-BADSUB-001\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate_with_config(&elements, &cfg_with_root(&root));

    // The target itself still gets its own ordinary E201.
    let e201 = result.findings.iter().find(|f| f.code == "E201");
    assert!(e201.is_some(), "expected the target's own E201: {:#?}", result.findings);
    assert!(e201.unwrap().file.ends_with("Configurations/BadSub.md"));

    // ...and the consolidator now also gets an E518 naming it.
    let e518 = result
        .findings
        .iter()
        .find(|f| f.code == "E518" && f.file.ends_with("Configurations/Main.md"));
    assert!(
        e518.is_some(),
        "expected E518 on the consolidator for a locally-resolved, structurally-invalid target: {:#?}",
        result.findings
    );
    assert!(
        e518.unwrap().message.contains("CONF-BADSUB-001"),
        "E518 message should name the failing local target: {}",
        e518.unwrap().message
    );
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

// ── Stack safety under a genuine cross-repo cycle ───────────────────────────

/// A real cross-repo mutual cycle (repo A's Configuration consolidates repo
/// B's, and repo B's consolidates repo A's back) validated from a thread with
/// a **constrained, 2 MiB stack** — Tokio's documented worker-thread default,
/// and exactly the stack every `validate_with_config` call from
/// `crates/syscribe/src/mcp/mod.rs` / `crates/syscribe/src/lsp/mod.rs` runs
/// on in production. Before the stack-safety fix (dedicated large-stack
/// thread per peer-recursion step), this reliably crashed the whole process
/// with SIGABRT (stack overflow) before any finding could ever be produced —
/// this test is the regression guard for that failure mode: it must return
/// normally (not abort) and report an error, not silently succeed.
///
/// Note on *which* error: because a genuine cross-repo `subConfigurations`
/// cycle can only exist when both repos' `[repos]` tables also point at each
/// other (that mutual `[repos]` entry is what makes each repo resolvable as
/// the other's peer in the first place), the *pre-existing*, independent
/// `E510` circular-`[repos]`-import check (`chain_reaches`, unrelated to
/// `subConfigurations`) legitimately also fires here, at the very first
/// recursion step — so this fixture's failure is reported as `E518`
/// "not internally valid" (citing that `E510`), not the `HPLE_MAX_DEPTH`
/// message. The depth guard itself — for a chain with no `[repos]`-level
/// cycle for `E510` to catch — is exercised in isolation by
/// `deep_acyclic_subconfigurations_chain_hits_the_bounded_depth_guard_without_crashing`
/// below. What both tests share, and what actually matters here, is: no
/// crash, and a reported error rather than a silent "valid".
#[test]
fn cross_repo_cyclic_subconfigurations_reports_e518_even_on_a_constrained_stack() {
    let repo_a = tempdir();
    let a_root = repo_a.join("model");
    let repo_b = tempdir();
    let b_root = repo_b.join("model");

    write(&a_root, "_index.md", "---\ntype: Package\nname: RepoA\n---\n");
    write(
        &a_root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-AREPO-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &a_root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-AREPO-001\nname: A Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-BREPO-001\n---\n",
    );
    write(
        &a_root,
        ".syscribe.toml",
        &format!("[repos.b]\npath = \"{}\"\n", repo_b.display()),
    );

    write(&b_root, "_index.md", "---\ntype: Package\nname: RepoB\n---\n");
    write(
        &b_root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-BREPO-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &b_root,
        "Configurations/Main.md",
        "---\ntype: Configuration\nid: CONF-BREPO-001\nname: B Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-AREPO-001\n---\n",
    );
    write(
        &b_root,
        ".syscribe.toml",
        &format!("[repos.a]\npath = \"{}\"\n", repo_a.display()),
    );

    // Tokio's documented default worker-thread stack size — the exact
    // adversarial condition the reviewer reproduced a SIGABRT under.
    const CONSTRAINED_STACK: usize = 2 * 1024 * 1024;

    let handle = std::thread::Builder::new()
        .stack_size(CONSTRAINED_STACK)
        .spawn(move || {
            let elements = walk_model(&a_root).unwrap();
            let cfg = ValidateConfig::with_model_root(a_root.clone());
            validate_with_config(&elements, &cfg)
        })
        .expect("failed to spawn the constrained-stack test thread");

    let result = handle.join().expect(
        "validating a genuine cross-repo circular subConfigurations chain must return \
         normally, not abort the process, even from a 2 MiB stack",
    );

    assert!(
        result.findings.iter().any(|f| f.code == "E518"),
        "expected an E518 on the consolidator: {:#?}",
        result.findings
    );
}

/// Exercises the `HPLE_MAX_DEPTH` bound itself, in isolation from the
/// pre-existing `E510` circular-`[repos]`-import check: a **strictly
/// forward** chain of repos (repo *i*'s `[repos]` table points only to repo
/// *i+1*, never backward) has no cycle anywhere in its `[repos]` graph for
/// `chain_reaches` to catch, yet is deep enough (more hops than
/// `HPLE_MAX_DEPTH`) that the recursive peer-validity walk must still
/// terminate on its own bound rather than recursing indefinitely. Run from
/// the same constrained, 2 MiB stack as the mutual-cycle test above, since a
/// long genuine recursion is exactly what the dedicated-thread fix has to
/// keep stack-safe for its whole depth, not just a two-hop bounce.
#[test]
fn deep_acyclic_subconfigurations_chain_hits_the_bounded_depth_guard_without_crashing() {
    const HOPS: usize = 20; // > HPLE_MAX_DEPTH (16), with margin.

    let repos: Vec<PathBuf> = (0..HOPS).map(|_| tempdir()).collect();
    let model_roots: Vec<PathBuf> = repos.iter().map(|r| r.join("model")).collect();

    for i in 0..HOPS {
        let mroot = &model_roots[i];
        write(mroot, "_index.md", &format!("---\ntype: Package\nname: Repo{i}\n---\n"));
        write(
            mroot,
            "Features/_index.md",
            &format!(
                "---\ntype: FeatureDef\nid: FEAT-CHAIN{i}-ROOT\nname: Root\ngroupKind: mandatory\n---\n"
            ),
        );
        if i + 1 < HOPS {
            write(
                mroot,
                "Configurations/Main.md",
                &format!(
                    "---\ntype: Configuration\nid: CONF-CHAIN{i}-001\nname: Chain {i}\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: CONF-CHAIN{}-001\n---\n",
                    i + 1
                ),
            );
            write(
                mroot,
                ".syscribe.toml",
                &format!("[repos.next]\npath = \"{}\"\n", repos[i + 1].display()),
            );
        } else {
            // The last link: a plain, valid leaf Configuration, no further hop.
            write(
                mroot,
                "Configurations/Main.md",
                &format!(
                    "---\ntype: Configuration\nid: CONF-CHAIN{i}-001\nname: Chain {i}\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n---\n"
                ),
            );
        }
    }

    const CONSTRAINED_STACK: usize = 2 * 1024 * 1024;
    let root0 = model_roots[0].clone();
    let handle = std::thread::Builder::new()
        .stack_size(CONSTRAINED_STACK)
        .spawn(move || {
            let elements = walk_model(&root0).unwrap();
            let cfg = ValidateConfig::with_model_root(root0.clone());
            validate_with_config(&elements, &cfg)
        })
        .expect("failed to spawn the constrained-stack test thread");

    let result = handle.join().expect(
        "a deep (> HPLE_MAX_DEPTH), but genuinely acyclic, subConfigurations chain must return \
         normally, not abort the process, even from a 2 MiB stack",
    );

    // No [repos]-level cycle exists anywhere in this strictly-forward chain,
    // so E510 must not fire -- confirming the failure below is genuinely the
    // depth guard, not a rediscovery of the unrelated circular-import check.
    assert!(
        !result.findings.iter().any(|f| f.code == "E510"),
        "this fixture is a strictly forward chain -- no E510 should fire: {:#?}",
        result.findings
    );
    assert!(
        result
            .findings
            .iter()
            .any(|f| f.code == "E518" && f.message.contains("consolidation depth")),
        "expected an E518 naming the exceeded consolidation depth: {:#?}",
        result.findings
    );
}

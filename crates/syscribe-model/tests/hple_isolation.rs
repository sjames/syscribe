//! Tests for `REQ-TRS-HPLE-005`: a lower-tier product-line model carries zero
//! awareness of, or reference to, anything above it. This requirement is
//! architectural/structural rather than independently unit-testable in
//! isolation (its own Scope section) — `PI-HPLE-SUBCONFIG-001`,
//! `PI-HPLE-PARAMBIND-001`, and `PI-HPLE-BINDGUARD-001`'s own test suites
//! already cover that the *downward* resolution mechanisms they add never
//! require, accept, or resolve an upward-pointing reference from a
//! descendant. What those suites don't specifically probe is the one
//! pre-existing mechanism a descendant *could* try to misuse for this —
//! `bindTo:` (component→system parameter propagation, ADR-SYS-HPLE-001
//! Decision 3's explicitly rejected alternative) — so this file positively
//! confirms `bindTo:` cannot be repurposed to cross a `subConfigurations:`
//! boundary, rather than inventing new mechanism-building work of its own.
//!
//! `bindTo:`'s only checks (`E202` propagation-range, `E229` binding-time
//! ordering) live in `feature_model::check_feature_model`, which — like every
//! other feature-model function in this codebase — takes exactly one
//! `elements` slice at a time. No call site anywhere concatenates two repos'
//! elements into one slice for this purpose (verified by inspection: every
//! call passes either a single model's own `elements` or one specific peer's
//! own `walk_model` result, never both together). A descendant's `bindTo:`
//! therefore cannot even *see* a consolidating tier's parameters to begin
//! with, structurally, independent of whether the string it names happens to
//! coincide with something real one tier up — this is what the tests below
//! demonstrate concretely rather than merely assert by inspection.
//!
//! Fixture pattern mirrors `hple_bindguard.rs`/`hple_openparam.rs` (tempdir +
//! write helpers, `walk_model` + `feature_model::check_feature_model`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::feature_model::check_feature_model;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-hple-isolation-test-{}-{}",
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

/// A lower-tier ("battery pack") model. Its `Cell` FeatureDef's `voltage`
/// parameter declares `bindTo: "Features::TopSecret.forbidden"` — a dotted
/// path that names nothing in this model at all, but *does* coincidentally
/// name a real `FeatureDef` parameter in the separate, higher-tier model
/// built by `write_consolidator` below. `own_binding` — if `Some` — is
/// spliced into this tier's *own* `Configuration.parameterBindings:` (used
/// for the positive control: the mechanism resolving correctly when the
/// match is genuinely local).
fn write_lower_tier(own_binding: Option<f64>) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: BatteryRoot\n---\n");
    write(
        &root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-BATTERY-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &root,
        "Features/Cell.md",
        "---\ntype: FeatureDef\nid: FEAT-BATTERY-CELL\nname: Cell\ngroupKind: mandatory\nparameters:\n  - name: voltage\n    type: ScalarValues::Real\n    bindTo: \"Features::TopSecret.forbidden\"\n    range: \"0..10\"\n---\n",
    );
    let bindings = match own_binding {
        Some(v) => format!("parameterBindings:\n  Features::TopSecret.forbidden: {v}\n"),
        None => String::new(),
    };
    write(
        &root,
        "Configurations/BatteryConf.md",
        &format!(
            "---\ntype: Configuration\nid: CONF-ISO-BATTERY-001\nname: Battery configuration\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\n{bindings}---\n"
        ),
    );
    root
}

/// The higher tier ("vehicle") that genuinely owns
/// `Features::TopSecret.forbidden` and consolidates the lower tier above via
/// `subConfigurations:`. `own_binding` mirrors the lower tier's — spliced
/// into *this* tier's own `parameterBindings:`, to probe whether a value
/// supplied up here could ever be mistaken, by the lower tier's `bindTo:`
/// range check, for a same-model match.
fn write_higher_tier(lower_dir: &Path, lower_conf_id: &str, own_binding: Option<f64>) -> PathBuf {
    let root = tempdir();
    write(
        &root,
        ".syscribe.toml",
        &format!(
            "[repos.battery]\npath = \"{}\"\n",
            lower_dir.display().to_string().replace('\\', "\\\\")
        ),
    );
    write(&root, "_index.md", "---\ntype: Package\nname: VehicleRoot\n---\n");
    write(
        &root,
        "Features/_index.md",
        "---\ntype: FeatureDef\nid: FEAT-VEHICLE-ROOT\nname: Root\ngroupKind: mandatory\n---\n",
    );
    write(
        &root,
        "Features/TopSecret.md",
        "---\ntype: FeatureDef\nid: FEAT-TOPSECRET\nname: TopSecret\ngroupKind: mandatory\nparameters:\n  - name: forbidden\n    type: ScalarValues::Real\n---\n",
    );
    let bindings = match own_binding {
        Some(v) => format!("parameterBindings:\n  Features::TopSecret.forbidden: {v}\n"),
        None => String::new(),
    };
    write(
        &root,
        "Configurations/Main.md",
        &format!(
            "---\ntype: Configuration\nid: CONF-ISO-MAIN-001\nname: Main\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features: true\nsubConfigurations: {lower_conf_id}\n{bindings}---\n"
        ),
    );
    root
}

/// Positive control: when the lower tier's own `Configuration` binds the
/// *same* key its own `bindTo:` names, `check_feature_model` — run on that
/// tier's own elements alone, exactly as `feature-check` would — correctly
/// catches the out-of-range value. This is not itself an isolation test; it
/// establishes that `bindTo:`'s propagation-range mechanism actually does
/// something, so the isolation tests below are a meaningful absence, not
/// just an inert feature never firing regardless of input.
#[test]
fn bind_to_range_check_fires_normally_within_its_own_model() {
    let lower_dir = write_lower_tier(Some(99.0)); // 99 is outside the declared 0..10 range
    let elements = walk_model(&lower_dir).unwrap();

    let findings = check_feature_model(&elements);
    assert!(
        findings.iter().any(|f| f.code == "E202"),
        "expected E202 when the match is genuinely local: {:#?}",
        findings
    );
}

/// The core isolation claim: the lower tier's `bindTo:` + `range:` metadata
/// for `voltage` never enters the higher tier's own `elements` slice at all
/// (it lives in a different repo, never walked as part of the higher tier's
/// own model) — so `check_feature_model` on the higher tier's own elements
/// raises `E202` **zero** times, full stop, regardless of what value the
/// higher tier happens to bind to the coincidentally-matching dotted key.
#[test]
fn a_lower_tiers_bind_to_target_never_becomes_visible_to_the_higher_tier() {
    let lower_dir = write_lower_tier(None); // lower tier binds nothing itself
    let higher_dir = write_higher_tier(&lower_dir, "CONF-ISO-BATTERY-001", Some(999.0));

    let higher_elements = walk_model(&higher_dir).unwrap();
    let findings = check_feature_model(&higher_elements);

    assert!(
        !findings.iter().any(|f| f.code == "E202"),
        "the higher tier's own elements never contain the lower tier's bindTo/range \
         declaration — E202 must never fire here, coincidental key match or not: {:#?}",
        findings
    );
}

/// The symmetric half: validating the lower tier *on its own* (exactly as
/// its own independent CI would, and exactly as `PI-HPLE-SUBCONFIG-001`'s
/// peer-validity gate does before consolidation is permitted) is completely
/// unaffected by the fact that a sibling, unrelated model happens to declare
/// a matching key with an out-of-range value — that model's
/// `parameterBindings:` never enters the lower tier's own `elements` either.
#[test]
fn the_higher_tiers_binding_never_leaks_down_into_the_lower_tiers_own_validation() {
    let lower_dir = write_lower_tier(None);
    // The higher tier exists on disk and *would* trip its own irrelevant
    // checks if walked, but the lower tier's own validation never reads it —
    // there is no upward file-system or elements-slice reference at all.
    let _higher_dir = write_higher_tier(&lower_dir, "CONF-ISO-BATTERY-001", Some(999.0));

    let lower_elements = walk_model(&lower_dir).unwrap();
    let findings = check_feature_model(&lower_elements);

    assert!(
        !findings.iter().any(|f| f.code == "E202"),
        "the lower tier's own validation must be unaffected by a higher tier's bindings: {:#?}",
        findings
    );
}

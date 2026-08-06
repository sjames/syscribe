//! Regression coverage for W024 ("orphan FeatureDef") canonicalization
//! (task #11): `check_feature_model` builds `referenced_by_applies_when` from
//! raw `appliesWhen:` operand strings. A bare `FEAT-*` stable-id operand must
//! be canonicalized to the FeatureDef's qualified name before being checked
//! against `fd.qualified_name`, exactly like the sibling `appliesWhen`
//! consumers in the same file (W014's `aw_expr`, the deep-analysis `elem_aw`)
//! already do via `variability::canon_feature_ref`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::feature_model;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-feature-model-orphan-test-{}-{}",
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

/// Bare `FEAT-*` id form: `appliesWhen: FEAT-ROTOR` where `FEAT-ROTOR` is the
/// FeatureDef's stable id and its qname is `Features::Rotor`. Before the fix,
/// W024 false-fired an orphan warning here because the raw id never matched
/// the qname-keyed `fnames`/`fd.qualified_name` comparison.
#[test]
fn bare_id_applies_when_does_not_false_fire_orphan() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/Rotor.md",
        "---\ntype: FeatureDef\nid: FEAT-ROTOR\nname: Rotor\n---\n",
    );
    write(
        &root,
        "Arch/Motor.md",
        "---\ntype: PartDef\nname: Motor\nappliesWhen: FEAT-ROTOR\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let findings = feature_model::check_feature_model(&elements);
    assert!(
        !findings.iter().any(|f| f.code == "W024"),
        "FEAT-ROTOR should not be flagged orphan — it's referenced (bare id form) by Arch::Motor's appliesWhen: {:#?}",
        findings
    );
}

/// Qualified-name form of the same reference must also keep working (this
/// was already correct before the fix — regression guard against breaking it).
#[test]
fn qname_applies_when_still_does_not_false_fire_orphan() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/Rotor.md",
        "---\ntype: FeatureDef\nid: FEAT-ROTOR\nname: Rotor\n---\n",
    );
    write(
        &root,
        "Arch/Motor.md",
        "---\ntype: PartDef\nname: Motor\nappliesWhen: Features::Rotor\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let findings = feature_model::check_feature_model(&elements);
    assert!(
        !findings.iter().any(|f| f.code == "W024"),
        "FEAT-ROTOR should not be flagged orphan — it's referenced (qname form) by Arch::Motor's appliesWhen: {:#?}",
        findings
    );
}

/// A genuinely unreferenced FeatureDef must still be flagged — the fix must
/// not blanket-suppress W024.
#[test]
fn truly_unreferenced_feature_is_still_flagged_orphan() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Features/Lonely.md",
        "---\ntype: FeatureDef\nid: FEAT-LONELY\nname: Lonely\n---\n",
    );

    let elements = walk_model(&root).unwrap();
    let findings = feature_model::check_feature_model(&elements);
    assert!(
        findings.iter().any(|f| f.code == "W024"),
        "FEAT-LONELY has no appliesWhen reference and no Configuration selection — should still be flagged: {:#?}",
        findings
    );
}

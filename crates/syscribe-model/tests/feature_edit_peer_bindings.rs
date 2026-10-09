//! GH #193: `removeParameter` must refuse while a `Configuration` in a `[repos]` peer (a
//! consolidating tier, section 14.7) still binds the parameter, whether the peer's
//! `parameterBindings:` key uses the feature's native qualified name or a `repoImports:` mount.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;
use syscribe_model::feature_edit::{apply, EditOp};

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-fedit-peer-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn op(json: Value) -> EditOp {
    serde_json::from_value(json).unwrap()
}

fn remove_param(root: &Path, name: &str) -> Result<(), String> {
    apply(root, &op(serde_json::json!({"op": "removeParameter", "feature": "Features::Cargo", "name": name}))).map(|_| ())
}

fn cargo_file(low: &Path) -> String {
    std::fs::read_to_string(low.join("Features/Cargo.md")).unwrap()
}

/// A lower-tier model (`low/model`) declaring `Features::Cargo` with parameters `capacityKg`
/// and `spare`, plus an upper-tier repo (`up/model`) that lists it under `[repos]` and whose
/// Configuration binds a parameter with `bind_key`. Returns `(low_root, up_root)`.
fn two_tier(bind_key: &str, mount: bool) -> (PathBuf, PathBuf) {
    let base = tempdir();
    let (low, up) = (base.join("low/model"), base.join("up/model"));
    write(&low, "_index.md", "---\ntype: Package\nname: LowRoot\n---\n");
    write(&low, "Features/_index.md", "---\ntype: FeatureDef\nid: FEAT-LOW-ROOT\nname: Features\ngroupKind: mandatory\n---\n");
    write(&low, "Features/Cargo.md", "---\ntype: FeatureDef\nid: FEAT-LOW-CARGO\nname: Cargo\ngroupKind: optional\nparameters:\n  - name: capacityKg\n    type: ScalarValues::Real\n    range: \"0.5..5.0\"\n  - name: spare\n    type: ScalarValues::Real\n---\n");
    let up_index = if mount {
        "---\ntype: Package\nname: UpRoot\nrepoImports:\n  - repo: low\n    qname: Features\n    as: LowFeatures\n---\n"
    } else {
        "---\ntype: Package\nname: UpRoot\n---\n"
    };
    write(&up, "_index.md", up_index);
    write(&up, "Configurations/CONF-UP-001.md", &format!("---\ntype: Configuration\nid: CONF-UP-001\nname: Up\nstatus: draft\nfeatureModel: Features\nfeatures: {{}}\nparameterBindings:\n  {bind_key}: 4.0\n---\n"));
    write(&up, ".syscribe.toml", &format!("[repos.low]\npath = \"{}\"\n", low.parent().unwrap().display()));
    write(&low, ".syscribe.toml", &format!("[repos.up]\npath = \"{}\"\n", up.parent().unwrap().display()));
    (low, up)
}

#[test]
fn removal_is_refused_while_a_peer_configuration_binds_the_parameter_by_native_qname() {
    let (low, up) = two_tier("Features::Cargo.capacityKg", false);
    let before = cargo_file(&low);
    let err = remove_param(&low, "capacityKg").unwrap_err();
    assert!(err.contains("CONF-UP-001") && err.contains("repo 'up'") && err.contains("removeBinding"), "{err}");
    assert_eq!(cargo_file(&low), before, "a refusal changes nothing");
    // A parameter nobody binds is still removable.
    remove_param(&low, "spare").unwrap();
    // Once the peer drops its binding the removal goes through.
    write(&up, "Configurations/CONF-UP-001.md", "---\ntype: Configuration\nid: CONF-UP-001\nname: Up\nstatus: draft\nfeatureModel: Features\nfeatures: {}\n---\n");
    remove_param(&low, "capacityKg").unwrap();
    assert!(!cargo_file(&low).contains("capacityKg"));
}

#[test]
fn removal_sees_a_peer_binding_written_through_a_repo_import_mount() {
    let (low, _up) = two_tier("LowFeatures::Cargo.capacityKg", true);
    let err = remove_param(&low, "capacityKg").unwrap_err();
    assert!(err.contains("CONF-UP-001") && err.contains("repo 'up'"), "{err}");
}

#[test]
fn a_peer_binding_of_another_parameter_or_feature_does_not_block_removal() {
    let (low, _up) = two_tier("Features::Cargo.spare", false);
    remove_param(&low, "capacityKg").unwrap();
    let (low, _up) = two_tier("Other::Cargo.capacityKg", false);
    remove_param(&low, "capacityKg").unwrap();
}

#[test]
fn a_model_without_repos_is_unaffected() {
    let (low, _up) = two_tier("Features::Cargo.capacityKg", false);
    std::fs::remove_file(low.join(".syscribe.toml")).unwrap();
    remove_param(&low, "capacityKg").unwrap();
}

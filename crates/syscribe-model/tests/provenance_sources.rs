//! Every `Source` kind of `provenance()` and its JSON form (REQ-TRS-BT-021..022).

use std::path::PathBuf;

use syscribe_model::provenance::{provenance, to_json};
use syscribe_model::walker::walk_model;

fn write(root: &std::path::Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

fn model() -> PathBuf {
    let d = std::env::temp_dir().join(format!("syscribe_prov_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    write(&d, "_index.md", "---\ntype: Package\nname: M\n---\n");
    write(
        &d,
        "F/Sys.md",
        "---\ntype: FeatureDef\nid: FEAT-PV-SYS\nname: Sys\ngroupKind: optional\nparameters:\n  - name: rate\n    type: ScalarValues::Real\n    default: 5\n---\n",
    );
    write(
        &d,
        "F/Dev.md",
        "---\ntype: FeatureDef\nid: FEAT-PV-DEV\nname: Dev\ngroupKind: optional\nparameters:\n  - name: fixedP\n    type: ScalarValues::Real\n    value: 7\n  - name: calc\n    type: ScalarValues::Real\n    derivedFrom: \"2 * 3\"\n  - name: tied\n    type: ScalarValues::Real\n    bindTo: F::Sys.rate\n  - name: dflt\n    type: ScalarValues::Real\n    default: 1\n---\n",
    );
    write(
        &d,
        "C/CONF-PV-BASE-001.md",
        "---\ntype: Configuration\nid: CONF-PV-BASE-001\nname: Base\nfeatures:\n  FEAT-PV-SYS: true\nparameterBindings:\n  F::Sys.rate: 9\n---\n",
    );
    write(
        &d,
        "C/CONF-PV-TOP-001.md",
        "---\ntype: Configuration\nid: CONF-PV-TOP-001\nname: Top\nderivedFrom: CONF-PV-BASE-001\nsubConfigurations: [CONF-PV-BASE-001]\nfeatures:\n  FEAT-PV-DEV: true\nparameterBindings:\n  Elsewhere::Tier.x: 3\n---\n",
    );
    d
}

#[test]
fn every_source_kind_is_reported_and_serialised() {
    let els = walk_model(&model()).unwrap();
    let idx = els.iter().position(|e| e.frontmatter.id.as_deref() == Some("CONF-PV-TOP-001")).unwrap();
    let p = provenance(&els, idx);
    let kinds: std::collections::BTreeSet<&str> = p.parameters.iter().map(|q| q.source.kind()).collect();
    for k in ["inherited", "fixed", "derived", "bindTo", "default", "subConfiguration"] {
        assert!(kinds.contains(k), "missing {k} in {kinds:?}");
    }
    assert!(p.selections.iter().any(|s| s.source.kind() == "inherited"));
    assert!(p.selections.iter().all(|s| !s.source.describe().is_empty()));
    let j = to_json(&p);
    assert!(j["parameters"].as_array().unwrap().iter().any(|q| q["source"]["kind"] == "subConfiguration" && q["source"]["tiers"].is_array()));
    assert!(j["parameters"].as_array().unwrap().iter().any(|q| q["source"]["kind"] == "derived" && q["source"]["expr"] == "2 * 3"));
    assert!(j["parameters"].as_array().unwrap().iter().any(|q| q["source"]["kind"] == "bindTo" && q["source"]["from"].is_string()));
}

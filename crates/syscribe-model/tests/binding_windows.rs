//! `REQ-TRS-BT-006..009`: binding windows on features and parameters — validity
//! (`E230`, `E750`), dependency ordering (`E229`) and its single, shared emission by
//! `validate` and `feature-check`. Uses the default `compile < load < runtime` vocabulary.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::feature_model::{binding_window_findings, check_feature_model};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn feature(id: &str, name: &str, top: &str, params: &str) -> String {
    let params = if params.is_empty() { String::new() } else { format!("parameters:\n{params}") };
    format!("---\ntype: FeatureDef\nid: {id}\nname: {name}\ngroupKind: optional\n{top}{params}---\n")
}

fn param(name: &str, extra: &str) -> String {
    let extra: String = extra.lines().map(|l| format!("    {l}\n")).collect();
    format!("  - name: {name}\n    type: ScalarValues::Real\n    range: \"0..100\"\n{extra}")
}

fn model(files: &[(&str, String)]) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-bw-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    for (rel, c) in files {
        write(&r, rel, c);
    }
    r
}

fn codes(root: &Path) -> Vec<(String, String)> {
    binding_window_findings(&walk_model(root).unwrap()).into_iter().map(|f| (f.code.to_string(), f.message)).collect()
}

fn count(f: &[(String, String)], code: &str) -> usize {
    f.iter().filter(|(c, _)| c == code).count()
}

#[test]
fn unknown_stage_and_inverted_window_on_features_and_parameters() {
    let r = model(&[(
        "Features/Motor.md",
        feature(
            "FEAT-BW-01", "Motor", "bindingTime: bogus\n",
            &(param("kv", "latestBindingTime: nope") + &param("inv", "bindingTime: load\nlatestBindingTime: compile")),
        ),
    )]);
    let f = codes(&r);
    assert_eq!(count(&f, "E230"), 2, "{f:?}");
    assert_eq!(count(&f, "E750"), 1, "{f:?}");
    assert!(f.iter().any(|(c, m)| c == "E230" && m.contains("not one of compile, load, runtime")), "{f:?}");
}

#[test]
fn spans_single_stages_and_latest_only_are_clean_and_unspecified_is_exempt() {
    let r = model(&[(
        "Features/Motor.md",
        feature(
            "FEAT-BW-02", "Motor", "bindingTime: compile\nlatestBindingTime: load\n",
            &(param("a", "bindingTime: runtime") + &param("b", "latestBindingTime: load") + &param("c", "")),
        ),
    )]);
    assert!(codes(&r).is_empty(), "{:?}", codes(&r));
}

#[test]
fn dependency_ordering_uses_window_overlap() {
    // a (compile) derived from b (load..runtime): closes before b opens -> E229.
    // c (compile..load) derived from d (load): overlap -> feasible.
    let r = model(&[(
        "Features/Motor.md",
        feature(
            "FEAT-BW-03", "Motor", "",
            &(param("a", "bindingTime: compile\nderivedFrom: \"b > 1\"")
                + &param("b", "bindingTime: load\nlatestBindingTime: runtime")
                + &param("c", "bindingTime: compile\nlatestBindingTime: load\nderivedFrom: \"d > 1\"")
                + &param("d", "bindingTime: load")),
        ),
    )]);
    let f = codes(&r);
    assert_eq!(count(&f, "E229"), 1, "{f:?}");
    assert!(f[0].1.contains("'Features::Motor.a' (bindingTime compile)") && f[0].1.contains("(bindingTime load..runtime)"), "{f:?}");
}

#[test]
fn feature_requires_and_bind_to_edges_are_checked() {
    let r = model(&[
        ("Features/A.md", feature("FEAT-BW-04", "A", "bindingTime: compile\nrequires: [FEAT-BW-05]\n", "")),
        ("Features/B.md", feature("FEAT-BW-05", "B", "bindingTime: load\nlatestBindingTime: runtime\n", "")),
        ("Features/Sys.md", feature("FEAT-BW-06", "Sys", "", &param("kv", "bindingTime: load"))),
        ("Features/Comp.md", feature("FEAT-BW-07", "Comp", "", &param("kv", "bindingTime: compile\nbindTo: \"Features::Sys.kv\""))),
    ]);
    let f = codes(&r);
    assert_eq!(count(&f, "E229"), 2, "{f:?}");
    assert!(f.iter().any(|(_, m)| m.contains("feature 'Features::A'") && m.contains("requires 'Features::B'")), "{f:?}");
    assert!(f.iter().any(|(_, m)| m.contains("'Features::Comp.kv'") && m.contains("bound to 'Features::Sys.kv'")), "{f:?}");
}

#[test]
fn validate_and_feature_check_each_report_a_finding_once() {
    let r = model(&[(
        "Features/Motor.md",
        feature("FEAT-BW-08", "Motor", "", &(param("a", "bindingTime: compile\nderivedFrom: \"b > 1\"") + &param("b", "bindingTime: load"))),
    )]);
    let els = walk_model(&r).unwrap();
    let v = validate(&els).findings.into_iter().filter(|f| f.code == "E229").count();
    let c = check_feature_model(&els).into_iter().filter(|f| f.code == "E229").count();
    assert_eq!((v, c), (1, 1));
}

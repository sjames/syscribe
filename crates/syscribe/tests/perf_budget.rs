//! TC-TRS-TIMING-001 (GH #429): the measurement behind the incremental-revalidation decision, and a guard on it.
//!
//! * `timing` reports each phase on the repository's own models and on synthetic models of growing size;
//! * a regression guard asserts generous ceilings (a debug build is several times slower than a release one), so a
//!   change that makes load or validation super-linear fails here;
//! * `cargo test --test perf_budget -- --ignored --nocapture` prints the scaling table used in
//!   `docs/design/incremental-validation.md`.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn timing(model: &Path) -> Value {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(["timing", "--json"]).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    serde_json::from_slice(&o.stdout).unwrap()
}

/// `n` requirements, each verified by a test case and satisfied by a part, in packages of 50, plus a small feature model with
/// four Configurations.
fn synthetic(n: usize, tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-perf-{}-{tag}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let w = |rel: String, c: String| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md".into(), "---\ntype: Package\nname: Root\n---\n".into());
    w("Features/_index.md".into(), "---\ntype: Package\nname: Features\n---\n".into());
    for (i, f) in ["Alpha", "Beta", "Gamma", "Delta"].iter().enumerate() {
        w(format!("Features/{f}.md"), format!("---\nid: FEAT-PF-{}\ntype: FeatureDef\nname: {f}\n---\n\n{f}\n", f.to_uppercase()));
        let _ = i;
    }
    w("Configurations/_index.md".into(), "---\ntype: Package\nname: Configurations\n---\n".into());
    for c in 0..4 {
        w(format!("Configurations/CONF-PF-C{c}.md"), format!("---\nid: CONF-PF-C{c}\ntype: Configuration\nname: C{c}\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Alpha: {}\n  Features::Beta: {}\n---\n\nc\n", c % 2 == 0, c / 2 == 1));
    }
    for i in 0..n {
        let pkg = i / 50;
        w(format!("P{pkg}/_index.md"), format!("---\ntype: Package\nname: P{pkg}\n---\n"));
        w(format!("P{pkg}/REQ-PF-{i:05}.md"), format!("---\nid: REQ-PF-{i:05}\ntype: Requirement\nname: R{i}\nstatus: approved\nreqDomain: system\nappliesWhen: Features::Alpha\n---\n\nThe system shall do thing {i}.\n"));
        w(format!("P{pkg}/TC-PF-{i:05}.md"), format!("---\nid: TC-PF-{i:05}\ntype: TestCase\nname: T{i}\nstatus: active\ntestLevel: L2\nverifies: [REQ-PF-{i:05}]\n---\n\n```gherkin\nFeature: f{i}\n  Scenario: s\n    Then ok\n```\n"));
    }
    root
}

#[test]
fn the_repositorys_own_models_are_well_inside_the_budget() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    // Budget (debug build): load + validate of the largest real model (about 1,500 files) in a few seconds.
    // A release build is several times faster; see docs/design/incremental-validation.md for the measurements.
    for (m, files, ceiling_ms) in [("model", 1000, 8000u64), ("model_gti", 200, 4000)] {
        let t = timing(&repo.join(m));
        assert!(t["files"].as_u64().unwrap() >= files, "{m}: {t}");
        assert!(t["totalMs"].as_u64().unwrap() < ceiling_ms, "{m} took {} ms of load+validate: {t}", t["totalMs"]);
        for k in ["loadMs", "validateMs", "variantsMs", "elements", "findings", "configurations"] {
            assert!(t[k].is_u64(), "{k} in {t}");
        }
        assert_eq!(t["schemaVersion"], "1.0");
    }
}

#[test]
fn doubling_the_model_does_not_more_than_triple_the_time() {
    // Super-linear growth is what would make an editor stall; a regression to quadratic behaviour fails this.
    let (small, large) = (synthetic(300, "scale"), synthetic(600, "scale"));
    let (a, b) = (timing(&small), timing(&large));
    let total = |t: &Value| (t["loadMs"].as_u64().unwrap() + t["validateMs"].as_u64().unwrap()).max(1) as f64;
    assert!(b["elements"].as_u64().unwrap() > a["elements"].as_u64().unwrap() * 3 / 2);
    // 300 ms floor so timer noise on the small run cannot make a healthy model look bad.
    assert!(total(&b) < (total(&a).max(300.0)) * 3.0, "300 elements: {} ms, 600: {} ms", total(&a), total(&b));
    assert!(total(&b) < 15_000.0, "600 requirements + 600 tests took {} ms", total(&b));
    let _ = (std::fs::remove_dir_all(&small), std::fs::remove_dir_all(&large));
}

#[test]
#[ignore = "prints the scaling table; run with --ignored --nocapture"]
fn scaling_table() {
    println!("\n| requirements+tests | files | elements | load ms | validate ms | variants ms | peak MiB |\n|---:|---:|---:|---:|---:|---:|---:|");
    for n in [250, 500, 1000, 2000, 4000] {
        let m = synthetic(n, "table");
        let t = timing(&m);
        println!("| {} | {} | {} | {} | {} | {} | {} |", n * 2, t["files"], t["elements"], t["loadMs"], t["validateMs"], t["variantsMs"], t["peakRssKiB"].as_u64().unwrap_or(0) / 1024);
        let _ = std::fs::remove_dir_all(&m);
    }
}

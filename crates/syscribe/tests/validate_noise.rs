//! TC-TRS-NOISE-001 / GH #245.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(extra_tc: bool) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir()
        .join(format!("syscribe-noise-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)))
        .join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    for (i, f) in ["Alpha", "Bravo", "Charlie"].iter().enumerate() {
        let fu = f.to_uppercase();
        w(
            &format!("Features/{f}.md"),
            &format!("---\ntype: FeatureDef\nid: FEAT-NZ-{}\nname: {f}\ngroupKind: optional\n---\n\nF.\n", 100 + i),
        );
        w(
            &format!("Configs/Conf{f}.md"),
            &format!(
                "---\nid: CONF-NZ-{fu}-001\ntype: Configuration\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Alpha: {}\n  Features::Bravo: {}\n  Features::Charlie: {}\n---\n\nC.\n",
                *f == "Alpha", *f == "Bravo", *f == "Charlie"
            ),
        );
    }
    for id in ["REQ-NZ-001", "REQ-NZ-002"] {
        w(
            &format!("Reqs/{id}.md"),
            &format!("---\nid: {id}\ntype: Requirement\nname: r\nstatus: approved\nreqDomain: software\nverificationMethod: test\n---\n\nThe system **shall** hold.\n"),
        );
    }
    if extra_tc {
        // Covers REQ-NZ-002 only in the Alpha configuration.
        w(
            "Tests/TC-NZ-001.md",
            "---\nid: TC-NZ-001\ntype: TestCase\ntestLevel: L3\nstatus: approved\nname: t\nappliesWhen: Features::Alpha\nverifies: [REQ-NZ-002]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n",
        );
    }
    r
}

fn validate(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).arg("validate").args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into_owned(), o.status.code().unwrap_or(-1))
}

fn w015_lines(out: &str) -> Vec<&str> {
    out.lines().filter(|l| l.contains("| W015 |")).collect()
}

#[test]
fn w015_is_one_finding_per_requirement_naming_every_configuration() {
    let (out, _) = validate(&model(false), &[]);
    let l = w015_lines(&out);
    assert_eq!(l.len(), 2, "{out}");
    for line in &l {
        for c in ["CONF-NZ-ALPHA-001", "CONF-NZ-BRAVO-001", "CONF-NZ-CHARLIE-001"] {
            assert!(line.contains(c), "{line}");
        }
    }
    assert!(l.iter().any(|x| x.contains("REQ-NZ-001")) && l.iter().any(|x| x.contains("REQ-NZ-002")), "{out}");
}

#[test]
fn partially_covered_requirement_names_only_the_uncovered_configurations() {
    let (out, _) = validate(&model(true), &[]);
    let line = w015_lines(&out).into_iter().find(|x| x.contains("REQ-NZ-002")).expect("REQ-NZ-002 W015");
    assert!(!line.contains("CONF-NZ-ALPHA-001"), "{line}");
    assert!(line.contains("CONF-NZ-BRAVO-001") && line.contains("CONF-NZ-CHARLIE-001"), "{line}");
}

#[test]
fn deny_w015_still_gates() {
    let (_, code) = validate(&model(false), &["--deny", "W015"]);
    assert_eq!(code, 2);
}

#[test]
fn summary_prints_counts_per_code_not_findings() {
    let (out, _) = validate(&model(false), &["--summary"]);
    assert!(out.contains("| W015 |") && out.contains("| 2 |"), "{out}");
    assert!(!out.contains("REQ-NZ-001"), "per-finding rows must be absent: {out}");
}

#[test]
fn summary_json_is_a_count_array_and_exit_code_is_unchanged() {
    let root = model(false);
    let (plain, c1) = validate(&root, &["--deny", "W015"]);
    let (json, c2) = validate(&root, &["--summary", "--json", "--deny", "W015"]);
    assert_eq!(c1, c2, "{plain}");
    let v: serde_json::Value = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}: {json}"));
    let row = v.as_array().unwrap().iter().find(|r| r["code"] == "W015").expect("W015 row");
    assert_eq!(row["count"], 2);
    assert_eq!(row["severity"], "warning");
}

#[test]
fn config_lens_w015_names_only_the_selected_configuration() {
    // GH #245 review: under --config X, other configurations must not appear.
    let root = model(false);
    std::fs::write(
        root.join("Tests/TC-NZ-009.md"),
        "---\nid: TC-NZ-009\ntype: TestCase\ntestLevel: L3\nstatus: approved\nname: t\nappliesWhen: Features::Bravo\nverifies: [REQ-NZ-002]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n",
    )
    .unwrap_or_else(|_| {
        std::fs::create_dir_all(root.join("Tests")).unwrap();
        std::fs::write(
            root.join("Tests/TC-NZ-009.md"),
            "---\nid: TC-NZ-009\ntype: TestCase\ntestLevel: L3\nstatus: approved\nname: t\nappliesWhen: Features::Bravo\nverifies: [REQ-NZ-002]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Given a\n    Then b\n```\n",
        )
        .unwrap();
    });
    let (out, _) = validate(&root, &["--config", "CONF-NZ-ALPHA-001"]);
    let line = w015_lines(&out).into_iter().find(|x| x.contains("REQ-NZ-002")).unwrap_or_else(|| panic!("{out}"));
    assert!(line.contains("CONF-NZ-ALPHA-001"), "{line}");
    assert!(!line.contains("CONF-NZ-BRAVO-001") && !line.contains("CONF-NZ-CHARLIE-001"), "{line}");
}

#[test]
fn summary_text_gate_output_matches_the_normal_report() {
    let root = model(false);
    let (out, code) = validate(&root, &["--summary", "--deny", "W015"]);
    assert_eq!(code, 2);
    assert!(out.contains("gated by --deny W015"), "{out}");
    assert!(out.contains("FAIL"), "{out}");
}

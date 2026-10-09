//! GH #223 heat tables: `fmea report --format`, `cyber-risk --format`,
//! `hara matrix`, and the `export-html` heat report. Black-box against a tiny
//! temp model.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn model() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("syscribe-heat-{}-{}", std::process::id(), n)).join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: HeatTest\n---\n\nRoot.\n");
    write(
        &root,
        "Safety/HE-HT-001.md",
        "---\nid: HE-HT-001\ntype: HazardousEvent\nname: Unintended accel\nstatus: draft\nseverity: S3\nexposure: E4\ncontrollability: C3\n---\n\nb\n",
    );
    write(
        &root,
        "Safety/HE-HT-002.md",
        "---\nid: HE-HT-002\ntype: HazardousEvent\nname: Minor\nstatus: draft\nseverity: S1\nexposure: E2\ncontrollability: C1\n---\n\nb\n",
    );
    write(
        &root,
        "Safety/SG-HT-001.md",
        "---\nid: SG-HT-001\ntype: SafetyGoal\nname: Avoid accel\nstatus: draft\nasilLevel: D\nhazardousEvents: [HE-HT-001]\n---\n\nb\n",
    );
    write(&root, "Fmea/_index.md", "---\ntype: FMEASheet\nid: FMEA-HT-001\nname: Sheet\nstatus: draft\n---\n\nb\n");
    write(
        &root,
        "Fmea/FM-HT-001.md",
        "---\nid: FM-HT-001\ntype: FMEAEntry\nname: Stuck\nstatus: draft\nfailureMode: Stuck\nfmeaSeverity: 9\noccurrence: 4\ndetection: 6\nrpn: 216\n---\n\nb\n",
    );
    write(
        &root,
        "Sec/DS-HT-001.md",
        "---\nid: DS-HT-001\ntype: DamageScenario\nname: D\nstatus: draft\ndamageSeverity: severe\n---\n\nb\n",
    );
    write(
        &root,
        "Sec/TS-HT-001.md",
        "---\nid: TS-HT-001\ntype: ThreatScenario\nname: T\nstatus: draft\ndamageScenarios: [DS-HT-001]\nattackFeasibility: high\n---\n\nb\n",
    );
    root
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.code().unwrap_or(-1))
}

#[test]
fn hara_matrix_places_events_and_goals_golden() {
    let root = model();
    let (md, code) = run(&root, &["hara", "matrix"]);
    assert_eq!(code, 0);
    assert!(md.starts_with("## HARA ASIL matrix\n"), "{md}");
    assert!(md.contains("| **S3 / E4** | B | C | D: HE-HT-001, SG-HT-001 (goal) |"), "{md}");
    assert!(md.contains("| **S1 / E2** | QM: HE-HT-002 | QM | QM |"), "{md}");
    let (js, _) = run(&root, &["hara", "matrix", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let cell = v["cells"].as_array().unwrap().iter().find(|c| c["row"] == "S3 / E4" && c["col"] == "C3").unwrap();
    assert_eq!(cell["label"], "D");
    assert_eq!(cell["tone"], "critical");
    assert_eq!(cell["elements"].as_array().unwrap().len(), 2);
    let (html, _) = run(&root, &["hara", "matrix", "--format", "html"]);
    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("class=\"t-critical\"") && html.contains("HE-HT-001"));
    assert!(!html.contains("<script") && !html.contains("http://") && !html.contains("https://"));
}

#[test]
fn hara_usage_and_bad_format() {
    let root = model();
    let (_, code) = run(&root, &["hara"]);
    assert_eq!(code, 1);
    let (_, code) = run(&root, &["hara", "matrix", "--format", "xml"]);
    assert_eq!(code, 1);
}

#[test]
fn fmea_heat_view_and_default_unchanged() {
    let root = model();
    let (md, code) = run(&root, &["fmea", "report", "--format", "md"]);
    assert_eq!(code, 0);
    // severity 9 row, occurrence 4 column: S*O = 36 (high), entry RPN 216 (critical band).
    assert!(md.contains("| **9** | 9 medium | 18 medium | 27 high | 36 high: FM-HT-001 (RPN 216) |"), "{md}");
    let (js, _) = run(&root, &["fmea", "report", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let cell = v["cells"].as_array().unwrap().iter().find(|c| c["row"] == "9" && c["col"] == "4").unwrap();
    assert_eq!(cell["elements"][0]["tone"], "critical");
    // Default and --json outputs keep their original shape.
    let (def, _) = run(&root, &["fmea", "report"]);
    assert!(def.starts_with("| ID | Failure Mode | Effect | Severity | Occurrence | Detection | RPN |"), "{def}");
    let (j, _) = run(&root, &["fmea", "report", "--json"]);
    assert!(j.trim_start().starts_with('['), "{j}");
    // Unknown sheet filter still exits 1 in heat mode.
    let (_, code) = run(&root, &["fmea", "report", "--fmea-sheet", "NOPE", "--format", "md"]);
    assert_eq!(code, 1);
}

#[test]
fn cyber_risk_heat_view_and_default_unchanged() {
    let root = model();
    let (md, code) = run(&root, &["cyber-risk", "--format", "md"]);
    assert_eq!(code, 0);
    assert!(md.contains("| **severe** | medium | high | critical | critical: TS-HT-001 |"), "{md}");
    let (js, _) = run(&root, &["cyber-risk", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["rows"][0], "severe");
    let (html, _) = run(&root, &["cyber-risk", "--format", "html"]);
    assert!(html.contains("TS-HT-001") && html.contains("t-critical"));
    let (def, _) = run(&root, &["cyber-risk"]);
    assert!(def.starts_with("# Cybersecurity Risk Determination (ISO/SAE 21434 §15.8)\n"), "{def}");
    assert!(def.contains("| Threat | Severity | Feasibility | Risk | Treatment | Addressed | Flag |"));
}

#[test]
fn cyber_heat_follows_the_configured_method() {
    let root = model();
    write(&root, ".syscribe.toml", "[cyber]\nmethod = \"annex\"\n");
    let (md, _) = run(&root, &["cyber-risk", "--format", "md"]);
    assert!(md.contains("annex"), "{md}");
    assert!(md.contains("TS-HT-001"), "{md}");
}

#[test]
fn export_html_gets_a_heat_report() {
    let root = model();
    let out = std::env::temp_dir().join(format!("syscribe-heat-out-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let (_, code) = run(&root, &["export-html", "--out", out.to_str().unwrap()]);
    assert_eq!(code, 0);
    let page = std::fs::read_to_string(out.join("reports/heat.html")).unwrap();
    assert!(page.contains("HARA ASIL matrix") && page.contains("FMEA severity x occurrence") && page.contains("TARA risk matrix"));
    assert!(page.contains("HE-HT-001") && page.contains("FM-HT-001") && page.contains("TS-HT-001"));
    let idx = std::fs::read_to_string(out.join("index.html")).unwrap();
    assert!(idx.contains("reports/heat.html"));
}

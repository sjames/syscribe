//! Security-analysis validation gaps (GH #219, #220, #221): TARASheet row
//! integrity, risk-input completeness, attack-tree shape, zone/conduit
//! consistency, list-valued security fields, VulnerabilityReport hygiene,
//! HazardousEvent expansion in `co-analysis`, and CycloneDX VEX in `sbom`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn write(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, format!("---\n{}---\n\nbody\n", body)).unwrap();
}

fn model(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-secval-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "type: Package\nname: Root\n");
    root
}

fn run(root: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn syscribe");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn has(report: &str, code: &str) -> bool {
    report.lines().any(|l| l.starts_with(&format!("| {} |", code)))
}

#[test]
fn tara_rows_zones_trees_and_lists() {
    let r = model("a");
    write(
        &r,
        "TARA-AA-001.md",
        "type: TARASheet\nid: TARA-AA-001\nname: T\nstatus: approved\n\
assetTable:\n  - id: ASSET-AA-001\n    name: A\n    status: approved\n    assetOwner: Nope\n  - name: noid\n\
damageTable:\n  - id: DS-AA-001\n    name: D\n    status: approved\n    bogusKey: 1\n\
threatTable:\n  - id: TS-AA-001\n    name: T\n    status: approved\n    riskTreatment: reduce\n    residualRisk: x\n",
    );
    write(&r, "REQ-AA-001.md", "type: Requirement\nid: REQ-AA-001\nname: R\nstatus: draft\nderivedFromCybersecurityGoal: [CSG-AA-001, CSG-AA-002]\n");
    write(&r, "REQ-AA-002.md", "type: Requirement\nid: REQ-AA-002\nname: R\nstatus: draft\nderivedFromCybersecurityGoal: {a: b}\n");
    write(&r, "AT-AA-001/_index.md", "type: AttackTree\nid: AT-AA-001\nname: T\nstatus: approved\nthreatRef: TS-AA-001\n");
    write(&r, "AT-AA-001/ATG-AA-001.md", "type: AttackTreeGate\nid: ATG-AA-001\nname: G\ngateType: AND\ninputs: [ATG-AA-002, ATS-AA-001, ATS-AA-001]\n");
    write(&r, "AT-AA-001/ATG-AA-002.md", "type: AttackTreeGate\nid: ATG-AA-002\nname: G\ngateType: OR\ninputs: [ATG-AA-001]\n");
    write(&r, "AT-AA-001/ATS-AA-001.md", "type: AttackStep\nid: ATS-AA-001\nname: S\n");
    write(&r, "Z/_index.md", "type: Package\nname: Z\n");
    write(&r, "Z/P1.md", "type: PartDef\nname: P1\n");
    write(&r, "Z/ZN-AA-001.md", "type: Zone\nid: ZN-AA-001\nname: Z1\nstatus: approved\ntargetSL: 2\nmembers: [Z::P1]\n");
    write(&r, "Z/ZN-AA-002.md", "type: Zone\nid: ZN-AA-002\nname: Z2\nstatus: approved\ntargetSL: 2\nmembers: [Z::P1]\n");
    write(&r, "Z/CD-AA-001.md", "type: Conduit\nid: CD-AA-001\nname: C\nstatus: approved\nfromZone: ZN-AA-001\ntoZone: ZN-AA-001\n");

    let rep = run(&r, &["validate"]);
    for code in [
        "E960", "E961", "E962", "E963", "E965", "W961", "W962", "W963", "W967", "W968", "W971", "W972",
        "W973", "W960",
    ] {
        assert!(has(&rep, code), "expected {code} in:\n{rep}");
    }
    assert!(rep.contains("`derivedFromCybersecurityGoal` must be a string or a list of strings"), "{rep}");
    // List form resolves (E831 names the missing second goal, not a YAML error).
    assert!(!rep.contains("not valid YAML"), "{rep}");

    let conduits = run(&r, &["conduits", "--json"]);
    assert!(conduits.contains("\"pass\": null"), "unknown must not pass: {conduits}");
    let _ = std::fs::remove_dir_all(&r);
}

#[test]
fn vulnerability_report_and_sbom() {
    let r = model("b");
    write(&r, "Sw/_index.md", "type: Package\nname: Sw\n");
    write(
        &r,
        "Sw/Dep.md",
        "type: PartDef\nname: Dep\nimplementedBy: [\"crates.io:foo@1.2.3\"]\n",
    );
    write(
        &r,
        "VR-BB-001.md",
        "type: VulnerabilityReport\nid: VR-BB-001\nname: V\nstatus: resolved\ncveId: CVE-24-1\ncvssScore: 9.5\n\
cvssSeverity: low\ncvssVector: junk\naffectedElements: [\"pkg:cargo/foo@1.2.3\", Sw::Dep]\nthreatScenarios: [Nope]\n",
    );
    write(&r, "VR-BB-002.md", "type: VulnerabilityReport\nid: VR-BB-002\nname: V\nstatus: bogus\n");
    let rep = run(&r, &["validate"]);
    for code in ["W974", "W975", "W976", "W978", "E967"] {
        assert!(has(&rep, code), "expected {code} in:\n{rep}");
    }
    assert!(!has(&rep, "E830"), "a pkg: purl is not a dangling element ref:\n{rep}");

    let sbom = run(&r, &["sbom"]);
    let doc: serde_json::Value = serde_json::from_str(&sbom).expect("sbom json");
    let v = &doc["vulnerabilities"][0];
    assert_eq!(v["id"], "CVE-24-1");
    assert_eq!(v["affects"].as_array().map(|a| a.len()), Some(1), "purl + owner dedupe to one component: {sbom}");
    let _ = std::fs::remove_dir_all(&r);
}

#[test]
fn coanalysis_expands_hazardous_event_to_safety_goal() {
    let r = model("c");
    write(&r, "HE-CC-001.md", "type: HazardousEvent\nid: HE-CC-001\nname: H\nstatus: approved\n");
    write(&r, "SG-CC-001.md", "type: SafetyGoal\nid: SG-CC-001\nname: G\nstatus: approved\nhazardousEvents: [HE-CC-001]\nasilLevel: B\n");
    write(&r, "DS-CC-001.md", "type: DamageScenario\nid: DS-CC-001\nname: D\nstatus: approved\nimpactCategories: [safety]\nhazardRef: HE-CC-001\n");
    let out = run(&r, &["co-analysis", "--json"]);
    assert!(out.contains("SG-CC-001"), "safety goal must appear: {out}");
    assert!(out.contains("HE-CC-001"), "{out}");
    let _ = std::fs::remove_dir_all(&r);
}

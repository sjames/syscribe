//! `show` prints every field an element declares (query.rs `cmd_show`). One element
//! carrying all of them; each label must appear in the output.

use std::process::Command;

// (frontmatter key, YAML value, label printed by `show`)
const FIELDS: &[(&str, &str, &str)] = &[
    ("extRef", "[DNG:1]", "extRef"),
    ("locale", "en", "locale"),
    ("isAbstract", "true", "abstract"),
    ("domain", "software", "domain"),
    ("requirementKind", "functional", "requirementKind"),
    ("reqDomain", "software", "reqDomain"),
    ("silLevel", "2", "SIL"),
    ("asilLevel", "B", "ASIL"),
    ("decomposedFrom", "ASIL_D", "decomposedFrom"),
    ("dalLevel", "B", "DAL"),
    ("verificationMethod", "test", "verificationMethod"),
    ("testLevel", "L2", "testLevel"),
    ("coverageTarget", "branch", "coverageTarget"),
    ("multiplicity", "\"1..3\"", "multiplicity"),
    ("direction", "in", "direction"),
    ("breakdownAdr", "ADR-X-001", "breakdownAdr"),
    ("itemType", "task", "itemType"),
    ("parent", "PI-X-001", "parent"),
    ("achieves", "[REQ-X-001]", "achieves"),
    ("blockedBy", "[PI-X-002]", "blockedBy"),
    ("claimedBy", "agent1", "claimedBy"),
    ("claimedAt", "2026-01-01T00:00:00Z", "claimedAt"),
    ("derivedFromCybersecurityGoal", "[CG-X-001]", "derivedFromCybersecurityGoal"),
    ("derivedFromSafetyGoal", "SG-X-001", "derivedFromSafetyGoal"),
    ("argumentType", "claim", "argumentType"),
    ("supports", "[A::B]", "supports"),
    ("appliesTo", "[A::B]", "appliesTo"),
    ("featureModel", "A::FM", "featureModel"),
    ("diagramKind", "bdd", "diagramKind"),
    ("subject", "A::B", "subject"),
    ("topEvent", "SG-X-001", "topEvent"),
    ("threatRef", "TS-X-001", "threatRef"),
    ("missionTime", "\"1e9 h\"", "missionTime"),
    ("gateType", "and", "gateType"),
    ("eventKind", "basic", "eventKind"),
    ("failureRate", "0.001", "failureRate"),
    ("probability", "0.5", "probability"),
    ("failureMode", "stuck", "failureMode"),
    ("effect", "loss", "effect"),
    ("cause", "wear", "cause"),
    ("fmeaSeverity", "7", "fmeaSeverity"),
    ("occurrence", "3", "occurrence"),
    ("detection", "4", "detection"),
    ("rpn", "84", "RPN"),
    ("recommendedAction", "inspect", "recommendedAction"),
    ("severity", "S2", "severity"),
    ("exposure", "E3", "exposure"),
    ("controllability", "C2", "controllability"),
    ("operationalSituation", "highway", "operationalSituation"),
    ("consequence", "Ca", "consequence"),
    ("freqExposure", "Fa", "freqExposure"),
    ("avoidance", "Pa", "avoidance"),
    ("demandRate", "W1", "demandRate"),
    ("safeState", "stopped", "safeState"),
    ("ftti", "100ms", "ftti"),
    ("plLevel", "d", "plLevel"),
    ("damageSeverity", "severe", "damageSeverity"),
    ("attackFeasibility", "high", "attackFeasibility"),
    ("attackVector", "network", "attackVector"),
    ("calLevel", "CAL2", "calLevel"),
    ("securityProperty", "[integrity]", "securityProperty"),
    ("controlType", "prevention", "controlType"),
    ("cvssScore", "7.5", "cvssScore"),
    ("cveId", "CVE-2026-0001", "cveId"),
];

#[test]
fn show_prints_every_declared_field() {
    let d = std::env::temp_dir().join(format!("syscribe_showf_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("A")).unwrap();
    std::fs::write(d.join("_index.md"), "---\ntype: Package\nname: M\n---\n").unwrap();
    let mut fm = String::from("---\ntype: PartDef\nname: Kitchen\n");
    for (k, v, _) in FIELDS {
        fm.push_str(&format!("{k}: {v}\n"));
    }
    fm.push_str("---\nbody\n");
    std::fs::write(d.join("A/Kitchen.md"), fm).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&d).args(["show", "A::Kitchen"]).output().unwrap();
    let out = String::from_utf8_lossy(&o.stdout);
    let missing: Vec<&str> = FIELDS.iter().map(|f| f.2).filter(|l| !out.contains(&format!("**{l}**"))).collect();
    assert!(missing.is_empty(), "show did not print: {missing:?}\n{out}\n{}", String::from_utf8_lossy(&o.stderr));
}

#[test]
fn show_prints_type_gated_fields() {
    let d = std::env::temp_dir().join(format!("syscribe_showg_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("A")).unwrap();
    std::fs::write(d.join("_index.md"), "---\ntype: Package\nname: M\n---\n").unwrap();
    std::fs::write(d.join("A/D.md"), "---\nid: ADR-SF-001\ntype: ADR\nname: D\nstatus: accepted\ndate: 2026-01-01\ndeciders: [alice, bob]\n---\nbody\n").unwrap();
    std::fs::write(d.join("A/E.md"), "---\ntype: FaultTreeEvent\nname: E\neventKind: basic\nref: A::D\n---\n").unwrap();
    let show = |q: &str| {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&d).args(["show", q]).output().unwrap();
        String::from_utf8_lossy(&o.stdout).into_owned()
    };
    let adr = show("ADR-SF-001");
    assert!(adr.contains("**date**") && adr.contains("alice, bob"), "{adr}");
    assert!(show("A::E").contains("**ref**"));
}

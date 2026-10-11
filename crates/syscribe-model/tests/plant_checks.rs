//! REQ-TRS-FMI-001 (GH #291): the static checks E888 / E889 / E966, run through a mock plant backend.
//! (The registered backend is process-global, so the "no backend" case is in `plant_gating.rs`.)

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Once;

use syscribe_model::config::ValidateConfig;
use syscribe_model::plant::{register_backend, Causality, FmuInfo, Plant, PlantBackend, VarInfo, VarType};
use syscribe_model::units::Dim;
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

struct Mock;

fn v(name: &str, c: Causality, ty: VarType, dim: Option<Dim>) -> VarInfo {
    VarInfo { name: name.into(), causality: c, ty, dim, factor: 1.0, offset: 0.0 }
}

impl PlantBackend for Mock {
    fn name(&self) -> String {
        "mock".into()
    }
    fn inspect(&self, file: &Path) -> Result<FmuInfo, String> {
        let text = std::fs::read_to_string(file).map_err(|e| format!("cannot read {}: {e}", file.display()))?;
        let mut lines = text.lines();
        let (sha, version, cs) = (lines.next().unwrap_or("").to_string(), lines.next().unwrap_or("2.0").to_string(), lines.next() != Some("no-cs"));
        if sha == "not-an-fmu" {
            return Err("not a zip archive".into());
        }
        Ok(FmuInfo {
            fmi_version: version,
            guid: "{mock}".into(),
            sha256: sha,
            supports_cosimulation: cs,
            variables: vec![
                v("h", Causality::Output, VarType::Real, Some(Dim([1, 0, 0, 0, 0, 0, 0]))),
                v("e", Causality::Parameter, VarType::Real, None),
                v("u", Causality::Input, VarType::Real, None),
                v("hit", Causality::Output, VarType::Boolean, None),
            ],
        })
    }
    fn instantiate(&self, _file: &Path, _parameters: &[(String, f64)]) -> Result<Box<dyn Plant>, String> {
        Err("not needed".into())
    }
}

fn setup() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        register_backend(Box::new(Mock));
    });
}

const PART: &str = "type: Part\nname: Ball\nfmu:\n  file: ball.fmu\n  fmiVersion: \"2.0\"\n  kind: cosimulation\n  sha256: abc123\n  parameters: {e: 0.8}\n";
const CTL: &str = "type: StateDef\nname: Ctl\nvariables:\n  - {name: height, value: \"0\", unit: \"SI::m\"}\n  - {name: thrust, value: \"0\"}\n  - {name: touched, value: \"false\"}\nsubStates:\n  - {name: A, isInitial: true}\n";
const TC: &str = "type: TestCase\nid: TC-FMI-001\nname: ball\nstatus: active\ntestLevel: L2\nsubject: Ctl\nplants: [Ball]\nplantStep: \"10 ms\"\nplantConnections:\n  - {from: Ball.h, to: height}\n  - {from: thrust, to: Ball.u}\n";

fn findings(part: &str, tc: &str, fmu_text: &str) -> Vec<String> {
    setup();
    static N: AtomicU64 = AtomicU64::new(0);
    let dir: PathBuf = std::env::temp_dir().join(format!("syscribe-plantchk-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(dir.join("ball.fmu"), fmu_text).unwrap();
    std::fs::write(dir.join("Ball.md"), format!("---\n{part}---\n")).unwrap();
    std::fs::write(dir.join("Ctl.md"), format!("---\n{CTL}---\n")).unwrap();
    std::fs::write(dir.join("TC-FMI-001.md"), format!("---\n{tc}---\n\n```gherkin\nFeature: f\n\n  Scenario: s\n    Given thrust is 0\n```\n")).unwrap();
    let elems = walk_model(&dir).unwrap();
    {
        let r = validate_with_config(&elems, &ValidateConfig::with_model_root(&dir));
        r.errors().chain(r.warnings()).chain(r.infos()).map(|f| format!("{} {}", f.code, f.message)).collect()
    }
}

fn good() -> Vec<String> {
    findings(PART, TC, "abc123\n2.0\n")
}

fn has(f: &[String], code: &str, what: &str) -> bool {
    f.iter().any(|x| x.starts_with(code) && x.contains(what))
}

#[test]
fn a_well_formed_plant_and_wiring_validate_clean() {
    let f = good();
    for c in ["E888", "E889", "E966", "I011"] {
        assert!(!f.iter().any(|x| x.starts_with(c)), "{c} unexpected: {f:?}");
    }
}

#[test]
fn a_malformed_declaration_is_e888() {
    for (from, to, what) in [
        ("kind: cosimulation", "kind: modelExchange", "kind"),
        ("  file: ball.fmu\n", "", "file"),
        ("parameters: {e: 0.8}", "parameters: {nope: 1}", "nope"),
        ("parameters: {e: 0.8}", "parameters: {e: banana}", "e"),
        ("kind: cosimulation", "kind: cosimulation\n  colour: red", "colour"),
    ] {
        let f = findings(&PART.replace(from, to), TC, "abc123\n2.0\n");
        assert!(has(&f, "E888", what), "{from} → {to}: {f:?}");
    }
    let f = findings(PART, &TC.replace("10 ms", "10 kg"), "abc123\n2.0\n");
    assert!(has(&f, "E888", "plantStep"), "{f:?}");
    let f = findings(PART, &TC.replace("[Ball]", "[Nowhere]"), "abc123\n2.0\n");
    assert!(has(&f, "E888", "Nowhere"), "{f:?}");
}

#[test]
fn an_unreadable_mismatched_or_tampered_fmu_is_e966() {
    assert!(has(&findings(PART, TC, "wrong-hash\n2.0\n"), "E966", "sha256"));
    assert!(has(&findings(PART, TC, "abc123\n3.0\n"), "E966", "fmiVersion"));
    assert!(has(&findings(PART, TC, "abc123\n2.0\nno-cs\n"), "E966", "Co-Simulation"));
    assert!(has(&findings(PART, TC, "not-an-fmu\n"), "E966", "not a zip"));
    assert!(has(&findings(&PART.replace("ball.fmu", "missing.fmu"), TC, "abc123\n2.0\n"), "E966", "missing.fmu"));
}

#[test]
fn bad_connections_are_e889() {
    let f = findings(PART, &TC.replace("to: height", "to: thrust"), "abc123\n2.0\n");
    assert!(has(&f, "E889", "dimension"), "{f:?}");
    let f = findings(PART, &TC.replace("Ball.h", "Ball.zzz"), "abc123\n2.0\n");
    assert!(has(&f, "E889", "zzz"), "{f:?}");
    let f = findings(PART, &TC.replace("{from: thrust, to: Ball.u}", "{from: thrust, to: Ball.h}"), "abc123\n2.0\n");
    assert!(has(&f, "E889", "output"), "{f:?}");
    let f = findings(PART, &TC.replace("to: height", "to: ghost"), "abc123\n2.0\n");
    assert!(has(&f, "E889", "ghost"), "{f:?}");
    let f = findings(PART, &TC.replace("to: height", "to: touched"), "abc123\n2.0\n");
    assert!(has(&f, "E889", "type"), "{f:?}");
}

#[test]
fn parameters_cannot_be_wired_and_odd_yaml_types_are_e888_not_a_parse_failure() {
    let f = findings(PART, &TC.replace("{from: thrust, to: Ball.u}", "{from: thrust, to: Ball.e}"), "abc123\n2.0\n");
    assert!(has(&f, "E889", "only inputs"), "{f:?}");
    let f = findings(PART, &TC.replace("\"10 ms\"", "0.01"), "abc123\n2.0\n");
    assert!(has(&f, "E888", "plantStep"), "{f:?}");
    let f = findings(PART, &TC.replace("[Ball]", "Ball"), "abc123\n2.0\n");
    assert!(has(&f, "E888", "plants"), "{f:?}");
}

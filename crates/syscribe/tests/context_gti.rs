//! `REQ-TRS-VIS-030` (GH #337) on the grid-tie inverter model: `Context::GtiContext` holds the inverter and its
//! actors; the derived `GtiContextIbd` draws them «actor», every boundary port of the inverter reaches an actor (no
//! `W996`), and `--config CONF-GTI-IN-M2-001` (no rapid shutdown) drops the rapid-shutdown initiator.

use std::path::PathBuf;
use std::process::Command;

fn gti() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../model_gti")
}

fn run(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(gti()).args(args).output().unwrap();
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn the_context_ibd_draws_the_actors_around_the_inverter() {
    let mmd = run(&["diagram", "export", "Diagrams::GtiContextIbd", "--format", "mermaid"]);
    for actor in ["pvArray", "grid", "owner", "installer", "rsdInitiator"] {
        assert!(mmd.contains(actor), "{actor} in:\n{mmd}");
    }
    let svg = run(&["diagram", "export", "Diagrams::GtiContextIbd", "--format", "svg"]);
    assert_eq!(svg.matches("«actor»").count(), 5, "five actors drawn «actor»");
    let india = run(&["diagram", "export", "Diagrams::GtiContextIbd", "--format", "mermaid", "--config", "CONF-GTI-IN-M2-001"]);
    assert!(india.contains("pvArray") && !india.contains("rsdInitiator"), "{india}");
}

#[test]
fn every_boundary_port_of_the_inverter_reaches_an_actor() {
    let report = run(&["validate"]);
    assert!(!report.contains("W996"), "{report}");
}

//! CLI paths of `params`, `residual` and `configure` provenance (ADR-SYS-BT-001), run
//! against the worked example `examples/staged-configuration`.

use std::path::PathBuf;
use std::process::{Command, Output};

fn example() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/staged-configuration/model")
}

fn run(model: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(args).output().unwrap()
}

fn text(o: &[u8]) -> String {
    String::from_utf8_lossy(o).into_owned()
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("syscribe_btcli_{}_{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn export_writes_file_and_schema_and_check_passes() {
    let d = scratch("export");
    let (y, s) = (d.join("f.yaml"), d.join("s.json"));
    let o = run(&example(), &["params", "export", "--config", "CONF-UAV-FLIGHT-001", "--out", y.to_str().unwrap(), "--schema", s.to_str().unwrap()]);
    assert!(o.status.success(), "{}", text(&o.stderr));
    assert!(std::fs::read_to_string(&y).unwrap().contains("projection-hash: blake3:"));
    serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(&s).unwrap()).unwrap();
    let c = run(&example(), &["params", "check", y.to_str().unwrap(), "--config", "CONF-UAV-FLIGHT-001"]);
    assert!(c.status.success());
    assert!(text(&c.stdout).contains("matches the contract"));
}

#[test]
fn export_to_stdout_with_explicit_stage() {
    let o = run(&example(), &["params", "export", "--config", "CONF-UAV-FLIGHT-001", "--stage", "runtime"]);
    assert!(o.status.success());
    assert!(text(&o.stdout).contains("parameters:"));
}

#[test]
fn export_refuses_open_required_unless_allowed() {
    let o = run(&example(), &["params", "export", "--config", "CONF-UAV-BUILD-001"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(text(&o.stderr).contains("have no value"));
    let a = run(&example(), &["params", "export", "--config", "CONF-UAV-BUILD-001", "--allow-open"]);
    assert!(a.status.success());
    assert!(text(&a.stdout).contains("OPEN"));
}

#[test]
fn usage_and_lookup_errors() {
    assert_eq!(run(&example(), &["params"]).status.code(), Some(2));
    assert_eq!(run(&example(), &["params", "export"]).status.code(), Some(2));
    assert_eq!(run(&example(), &["params", "check"]).status.code(), Some(2));
    assert_eq!(run(&example(), &["params", "export", "--config", "CONF-NOPE-001"]).status.code(), Some(1));
    let o = run(&example(), &["params", "export", "--config", "CONF-UAV-FLIGHT-001", "--stage", "bogus"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(text(&o.stderr).contains("Unknown stage"));
    let o = run(&example(), &["params", "check", "/nonexistent/file.yaml", "--config", "CONF-UAV-FLIGHT-001"]);
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn check_reports_errors_and_deny_gates_warnings() {
    let d = scratch("check");
    let y = d.join("f.yaml");
    let o = run(&example(), &["params", "export", "--config", "CONF-UAV-FLIGHT-001", "--out", y.to_str().unwrap()]);
    assert!(o.status.success());
    // Wrong type and unknown key => errors, exit 1.
    let good = std::fs::read_to_string(&y).unwrap();
    let bad = good.replace("rateHz: 10", "rateHz: fast") + "bogus_key: 1\n";
    let b = d.join("bad.yaml");
    std::fs::write(&b, &bad).unwrap();
    let c = run(&example(), &["params", "check", b.to_str().unwrap(), "--config", "CONF-UAV-FLIGHT-001"]);
    assert_eq!(c.status.code(), Some(1), "{}", text(&c.stdout));
    assert!(text(&c.stdout).contains("error(s)"));
    // A stale header is only a warning; --deny turns it into exit 2.
    let stale = good.replacen("projection-hash: blake3:", "projection-hash: blake3:00", 1);
    let s = d.join("stale.yaml");
    std::fs::write(&s, &stale).unwrap();
    let w = run(&example(), &["params", "check", s.to_str().unwrap(), "--config", "CONF-UAV-FLIGHT-001"]);
    assert!(w.status.success(), "{}", text(&w.stdout));
    assert!(text(&w.stdout).contains("W751"));
    let dn = run(&example(), &["params", "check", s.to_str().unwrap(), "--config", "CONF-UAV-FLIGHT-001", "--deny", "W751"]);
    assert_eq!(dn.status.code(), Some(2));
}

#[test]
fn residual_text_json_and_missing() {
    let t = run(&example(), &["residual", "--config", "CONF-UAV-BUILD-001"]);
    assert!(t.status.success());
    let out = text(&t.stdout);
    assert!(out.contains("Next bindable at"), "{out}");
    let j = run(&example(), &["residual", "--config", "CONF-UAV-BUILD-001", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&j.stdout).unwrap();
    assert_eq!(v["configuration"].as_str().is_some(), true);
    let f = run(&example(), &["residual", "--config", "CONF-UAV-FLIGHT-001"]);
    assert!(text(&f.stdout).contains("Nothing is left open"));
    assert_eq!(run(&example(), &["residual", "--config", "CONF-NOPE-001"]).status.code(), Some(1));
}

#[test]
fn configure_shows_provenance_and_json() {
    let t = run(&example(), &["configure", "CONF-UAV-DEPLOY-001"]);
    assert!(t.status.success());
    let out = text(&t.stdout);
    assert!(out.contains("| own | load |") && out.contains("| default |"), "{out}");
    let f = run(&example(), &["configure", "CONF-UAV-FLIGHT-001"]);
    assert!(text(&f.stdout).contains("inherited from"), "{}", text(&f.stdout));
    let j = run(&example(), &["configure", "CONF-UAV-DEPLOY-001", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&j.stdout).unwrap();
    assert!(v.to_string().contains("source"));
}

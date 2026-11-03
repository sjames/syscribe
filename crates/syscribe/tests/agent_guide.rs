//! TC-TRS-AGENTGUIDE-001 (GH #441): `agent-guide` generates a project-specific block from the
//! model and `.syscribe.toml`, writes only its marked region, and `--check` detects staleness.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const BEGIN: &str = "<!-- syscribe:agent-guide -->";
const END: &str = "<!-- /syscribe:agent-guide -->";

fn run(model: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).arg("agent-guide").args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

fn model(toml: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let base = std::env::temp_dir().join(format!("syscribe-agentguide-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let d = base.join("model");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(d.join(".syscribe.toml"), toml).unwrap();
    d
}

fn file(m: &Path, name: &str) -> String {
    m.parent().unwrap().join(name).to_string_lossy().into_owned()
}

#[test]
fn the_guide_is_built_from_the_model_and_its_config() {
    let m = model("[ids.prefixes]\nRequirement = [\"STK\", \"SYS\"]\n\n[profiles.safety]\npromote = [\"W004\", \"W002\"]\nsil = \"3\"\n\n[gates]\ndeny_meta = [\"Regulatory.status=unverified\"]\n");
    let (out, err, code) = run(&m, &[]);
    assert_eq!(code, 0, "{err}");
    assert!(out.starts_with(BEGIN) && out.trim_end().ends_with(END), "{out}");
    // ids: the built-in prefix, the configured extras, and the next-id command
    let req = out.lines().find(|l| l.starts_with("| Requirement ")).unwrap_or_else(|| panic!("{out}"));
    assert!(req.contains("`REQ`") && req.contains("STK, SYS") && req.contains("next-id REQ-<AREA>"), "{req}");
    // types: vocabulary, required fields, commands
    let pi = out.lines().find(|l| l.starts_with("- **PlanningItem**")).unwrap();
    assert!(pi.contains("todo · in_progress · blocked · done") && pi.contains("`claim`") && pi.contains("`release`"), "{pi}");
    assert!(out.contains("- **ADR** — status: proposed · accepted · deprecated · superseded"), "{out}");
    // gates: one validate per profile, the standing gate
    assert!(out.contains("validate` —") && out.contains("validate --profile safety` — promotes W002, W004 for integrity 3"), "{out}");
    assert!(out.contains("deny-meta `Regulatory.status=unverified`"), "{out}");
    // rules
    assert!(out.contains("never hand-edit `claimedBy`/`claimedAt`") && out.contains("`--dry-run` on `set`"), "{out}");
}

#[test]
fn output_is_deterministic() {
    let m = model("");
    assert_eq!(run(&m, &[]).0, run(&m, &[]).0);
}

#[test]
fn write_replaces_only_the_marked_region_and_never_touches_other_text() {
    let m = model("");
    let f = file(&m, "CLAUDE.md");
    let hand = "# Project rules\n\nHand-written text above.\n\n";
    let tail = "\nHand-written text below, with `code` and <!-- other comment -->.\n";
    std::fs::write(&f, format!("{hand}{BEGIN}\nstale content\n{END}\n{tail}")).unwrap();
    let (out, err, code) = run(&m, &["--write", &f]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("Updated"), "{out}");
    let after = std::fs::read_to_string(&f).unwrap();
    assert!(after.starts_with(hand) && after.ends_with(tail), "outside text is byte-identical:\n{after}");
    assert!(!after.contains("stale content") && after.contains("### Safe mutation"));
    // Idempotent.
    let (out, _e, code) = run(&m, &["--write", &f]);
    assert_eq!(code, 0);
    assert!(out.contains("already up to date"), "{out}");
    assert_eq!(std::fs::read_to_string(&f).unwrap(), after);
}

#[test]
fn write_creates_a_missing_file_and_appends_to_one_without_a_region() {
    let m = model("");
    let new = file(&m, "NEW.md");
    assert_eq!(run(&m, &["--write", &new]).2, 0);
    let created = std::fs::read_to_string(&new).unwrap();
    assert!(created.starts_with(BEGIN) && created.trim_end().ends_with(END));

    let plain = file(&m, "PLAIN.md");
    std::fs::write(&plain, "# Existing\n\nKeep me.\n").unwrap();
    assert_eq!(run(&m, &["--write", &plain]).2, 0);
    let appended = std::fs::read_to_string(&plain).unwrap();
    assert!(appended.starts_with("# Existing\n\nKeep me.\n\n") && appended.contains(BEGIN), "{appended}");
}

#[test]
fn check_fails_when_an_id_prefix_changes_until_regenerated() {
    let m = model("[ids.prefixes]\nRequirement = [\"STK\"]\n");
    let f = file(&m, "AGENTS.md");
    std::fs::write(&f, "# Rules\n").unwrap();
    assert_eq!(run(&m, &["--write", &f]).2, 0);
    let (out, _e, code) = run(&m, &["--check", &f]);
    assert_eq!(code, 0);
    assert!(out.contains("up to date"), "{out}");

    // The model's configuration changes.
    std::fs::write(m.join(".syscribe.toml"), "[ids.prefixes]\nRequirement = [\"STK\", \"SYS\"]\n").unwrap();
    let (_o, err, code) = run(&m, &["--check", &f]);
    assert_eq!(code, 1);
    assert!(err.contains("out of date") && err.contains("--write"), "{err}");
    assert_eq!(run(&m, &["--write", &f]).2, 0);
    assert_eq!(run(&m, &["--check", &f]).2, 0);
}

#[test]
fn check_and_write_refuse_what_they_cannot_trust() {
    let m = model("");
    let no_region = file(&m, "A.md");
    std::fs::write(&no_region, "# nothing here\n").unwrap();
    let (_o, err, code) = run(&m, &["--check", &no_region]);
    assert_eq!(code, 1);
    assert!(err.contains("no `<!-- syscribe:agent-guide -->` region"), "{err}");
    assert_eq!(run(&m, &["--check", &file(&m, "missing.md")]).2, 1);

    let damaged = file(&m, "B.md");
    let text = format!("before\n{BEGIN}\nonly a begin marker\nafter\n");
    std::fs::write(&damaged, &text).unwrap();
    let (_o, err, code) = run(&m, &["--write", &damaged]);
    assert_eq!(code, 1);
    assert!(err.contains("exactly one"), "{err}");
    assert_eq!(std::fs::read_to_string(&damaged).unwrap(), text, "a damaged file is left untouched");

    assert_eq!(run(&m, &["--write", &no_region, "--check", &no_region]).2, 1);
}

#[test]
fn sections_and_json() {
    let m = model("");
    let (out, _e, code) = run(&m, &["--sections", "types"]);
    assert_eq!(code, 0);
    assert!(out.contains("### Types") && !out.contains("### Ids") && !out.contains("### Safe mutation"), "{out}");
    let (j, _e, code) = run(&m, &["--format", "json", "--sections", "ids,types"]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["schemaVersion"], "1.0");
    assert!(v["ids"].is_array() && v["types"].is_array() && v.get("rules").is_none());
    let (_o, err, code) = run(&m, &["--sections", "tpyes"]);
    assert_eq!(code, 1);
    assert!(err.contains("Did you mean 'types'?"), "{err}");
    assert_eq!(run(&m, &["--format", "xml"]).2, 1);
    assert_eq!(run(&m, &["--format", "json", "--write", &file(&m, "J.md")]).2, 1);
}

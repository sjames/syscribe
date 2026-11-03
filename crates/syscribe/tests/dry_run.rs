//! TC-TRS-DRYRUN-001 (GH #444): every command that writes files honours a global `--dry-run`.
//!
//! For each writer the same invocation is run twice on fresh copies of a bundled example model: for real
//! (which must change the tree — proving the command really is a writer) and with `--dry-run` (which must
//! leave the whole tree, outputs included, byte-identical).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let (from, to) = (e.path(), dst.join(e.file_name()));
        if e.file_type().unwrap().is_dir() {
            copy_dir(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

fn snapshot(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let e = e.unwrap();
            let p = e.path();
            if e.file_type().unwrap().is_dir() {
                walk(base, &p, out);
            } else {
                out.insert(p.strip_prefix(base).unwrap().to_string_lossy().into_owned(), std::fs::read(&p).unwrap());
            }
        }
    }
    let mut m = BTreeMap::new();
    walk(dir, dir, &mut m);
    m
}

/// A scratch directory holding `model/` (a copy of a bundled model) and an empty `out/`.
fn scratch(model_src: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let base = std::env::temp_dir().join(format!("syscribe-dry-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    copy_dir(&repo_root().join(model_src), &base.join("model"));
    std::fs::create_dir_all(base.join("out")).unwrap();
    base
}

fn run(base: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).current_dir(base).arg("-m").arg("model").args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
}

struct Case {
    name: &'static str,
    model: &'static str,
    args: &'static [&'static str],
    /// Files to create in the scratch directory before the command (relative to it).
    files: &'static [(&'static str, &'static str)],
    /// Commands run for real first (e.g. claim, so there is something to release).
    setup: &'static [&'static [&'static str]],
}

const PLAN: &str = "examples/planning-item/model";
const UAV: &str = "model";

const CASES: &[Case] = &[
    Case { name: "set status", model: PLAN, args: &["set", "PI-RTH-DOCS-001", "status=in_progress"], files: &[], setup: &[] },
    Case { name: "set question.add", model: PLAN, args: &["set", "PI-RTH-DOCS-001", "question.add", "text=Which units?"], files: &[], setup: &[] },
    Case { name: "set question.answer", model: PLAN, args: &["set", "PI-RTH-DOCS-001", "question.answer", "Q1", "text=percent"], files: &[], setup: &[] },
    Case { name: "set evidence.add", model: PLAN, args: &["set", "PI-RTH-DOCS-001", "evidence.add", "ref=REQ-RTH-001"], files: &[], setup: &[] },
    Case { name: "claim", model: PLAN, args: &["claim", "PI-RTH-DOCS-001", "--by", "agent-9"], files: &[], setup: &[] },
    Case { name: "release", model: PLAN, args: &["release", "PI-RTH-DOCS-001"], files: &[], setup: &[&["claim", "PI-RTH-DOCS-001", "--by", "agent-9"]] },
    Case { name: "move", model: UAV, args: &["move", "GroundStation::GroundControlStation", "GroundStation::GCS2"], files: &[], setup: &[] },
    Case { name: "applies-when --set", model: UAV, args: &["applies-when", "REQ-UAV-NAV-001", "--set", "Features::DataLink"], files: &[], setup: &[] },
    Case { name: "baseline create", model: UAV, args: &["baseline", "create", "--tag", "t1"], files: &[], setup: &[] },
    Case { name: "suspect accept --all-unbaselined", model: UAV, args: &["suspect", "accept", "--all-unbaselined"], files: &[], setup: &[] },
    Case {
        name: "ingest-results",
        model: UAV,
        args: &["ingest-results", "out/r.xml"],
        files: &[("out/r.xml", "<testsuite name=\"s\"><testcase classname=\"c\" name=\"n\"/></testsuite>")],
        setup: &[],
    },
    Case { name: "plantuml", model: UAV, args: &["plantuml"], files: &[], setup: &[] },
    Case { name: "summarize (cache)", model: PLAN, args: &["summarize"], files: &[], setup: &[] },
    Case { name: "export-html", model: PLAN, args: &["export-html", "--out", "out/html"], files: &[], setup: &[] },
    Case { name: "diagram export --all", model: UAV, args: &["diagram", "export", "--all", "--format", "svg", "--out-dir", "out/d"], files: &[], setup: &[] },
    Case { name: "export-reqif", model: UAV, args: &["export-reqif", "--output", "out/x.reqif"], files: &[], setup: &[] },
    Case { name: "trace-export", model: UAV, args: &["trace-export", "--out", "out/t.json"], files: &[], setup: &[] },
    Case { name: "sbom", model: UAV, args: &["sbom", "--output", "out/s.json"], files: &[], setup: &[] },
    Case { name: "magicgrid --svg", model: "model_mg", args: &["magicgrid", "--svg", "-o", "out/mg.svg"], files: &[], setup: &[] },
    Case { name: "params export", model: "examples/staged-configuration/model", args: &["params", "export", "--config", "CONF-UAV-FLIGHT-001", "--out", "out/p.yaml"], files: &[], setup: &[] },
    Case { name: "agent-guide --write", model: PLAN, args: &["agent-guide", "--write", "out/G.md"], files: &[], setup: &[] },
];

fn prepare(c: &Case) -> PathBuf {
    let base = scratch(c.model);
    for (rel, content) in c.files {
        std::fs::write(base.join(rel), content).unwrap();
    }
    for s in c.setup {
        let (_o, e, code) = run(&base, s);
        assert_eq!(code, 0, "{}: setup {s:?} failed: {e}", c.name);
    }
    base
}

#[test]
fn every_writer_changes_the_tree_for_real_and_leaves_it_untouched_under_dry_run() {
    for c in CASES {
        // Real run: this must be a writer.
        let real = prepare(c);
        let before = snapshot(&real);
        let (_o, e, code) = run(&real, c.args);
        assert_eq!(code, 0, "{}: the real run failed: {e}", c.name);
        assert_ne!(snapshot(&real), before, "{}: the real run changed nothing, so it proves nothing about --dry-run", c.name);

        // Dry run, flag after the command.
        let dry = prepare(c);
        let before = snapshot(&dry);
        let mut a: Vec<&str> = c.args.to_vec();
        a.push("--dry-run");
        let (_o, e, code) = run(&dry, &a);
        assert_eq!(code, 0, "{}: --dry-run failed: {e}", c.name);
        assert_eq!(snapshot(&dry), before, "{}: --dry-run changed the tree", c.name);

        // Dry run, flag before the command: the same.
        let dry2 = prepare(c);
        let before = snapshot(&dry2);
        let mut b: Vec<&str> = vec!["--dry-run"];
        b.extend_from_slice(c.args);
        let (_o, e, code) = run(&dry2, &b);
        assert_eq!(code, 0, "{}: leading --dry-run failed: {e}", c.name);
        assert_eq!(snapshot(&dry2), before, "{}: a leading --dry-run changed the tree", c.name);
        for d in [&real, &dry, &dry2] {
            let _ = std::fs::remove_dir_all(d);
        }
    }
}

#[test]
fn a_dry_run_says_what_it_declined_to_do() {
    let c = &CASES[0];
    let base = prepare(c);
    let (out, err, code) = run(&base, &["--dry-run", "set", "PI-RTH-DOCS-001", "status=in_progress"]);
    assert_eq!(code, 0);
    assert!(out.contains("+status: in_progress"), "a diff of the change is shown: {out}");
    assert!(!err.contains("Set status"), "it does not claim to have written: {err}");
    // A generated output says which file it would have written, and does not claim it did.
    let (out, err, _c) = run(&base, &["export-reqif", "--output", "out/x.reqif", "--dry-run"]);
    assert!(err.contains("dry run: would write") && err.contains("x.reqif"), "{err}");
    assert!(err.contains("Would write") && !err.contains("Wrote "), "the status line is truthful: {err}{out}");
    assert!(err.contains("dry run: 1 file(s) not written"), "{err}");
    assert!(!base.join("out/x.reqif").exists());
    let (out, _e, _c) = run(&base, &["export-html", "--out", "out/h", "--dry-run"]);
    assert!(out.contains("Would write") && !out.contains("Wrote "), "{out}");
}

/// A dry run must refuse exactly what a real run refuses: candidate validation is planning, not a write, so
/// it has to happen under `--dry-run` too (a dry run that approves what a real run rejects is worse than none).
#[test]
fn a_dry_run_refuses_what_a_real_run_refuses() {
    let base = scratch(PLAN);
    let before = snapshot(&base);
    let bad = ["set", "REQ-RTH-001", "breakdownAdr=Nope::Missing"];
    let (_o, e, real) = run(&base, &bad);
    assert_eq!(real, 1, "the real run is refused: {e}");
    let mut dry: Vec<&str> = bad.to_vec();
    dry.push("--dry-run");
    let (_o, e, code) = run(&base, &dry);
    assert_eq!(code, 1, "the dry run must be refused too: {e}");
    assert!(e.contains("Refusing"), "{e}");
    assert_eq!(snapshot(&base), before);
}

#[test]
fn scaffold_gherkin_fix_honours_dry_run() {
    let base = scratch(PLAN);
    std::fs::write(
        base.join("model/TC-SCAF-001.md"),
        "---\nid: TC-SCAF-001\ntype: TestCase\nname: t\nstatus: draft\ntestLevel: L2\nverifies: [REQ-RTH-001]\ntestFunctions:\n  - function: f\n    scenario: Does the thing\n---\n\nBody.\n",
    )
    .unwrap();
    let before = snapshot(&base);
    let (_o, e, code) = run(&base, &["scaffold-gherkin", "TC-SCAF-001", "--fix", "--dry-run"]);
    assert_eq!(code, 0, "{e}");
    assert_eq!(snapshot(&base), before);
    let (_o, e, code) = run(&base, &["scaffold-gherkin", "TC-SCAF-001", "--fix"]);
    assert_eq!(code, 0, "{e}");
    assert_ne!(snapshot(&base), before, "without the flag it writes");
}

#[test]
fn install_hook_honours_dry_run() {
    let base = std::env::temp_dir().join(format!("syscribe-dry-hook-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    copy_dir(&repo_root().join(PLAN), &base.join("model"));
    let git = |a: &[&str]| assert!(Command::new("git").arg("-C").arg(&base).args(a).output().unwrap().status.success());
    git(&["init", "-q"]);
    let (_o, e, code) = run(&base, &["check-staged", "--install-hook", "--dry-run"]);
    assert_eq!(code, 0, "{e}");
    assert!(!base.join(".git/hooks/pre-commit").exists(), "no hook was written");
    let (_o, _e, code) = run(&base, &["check-staged", "--install-hook"]);
    assert_eq!(code, 0);
    assert!(base.join(".git/hooks/pre-commit").exists());
    let _ = std::fs::remove_dir_all(&base);
}

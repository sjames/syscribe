//! TC-TRS-VIS-032 / GH #339: `diagram export --all` renders every diagram of a model to a directory, byte for
//! byte the same on every run and machine, and `--check` fails when a committed render is out of date.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn gti() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_gti")
}

/// A scratch directory, removed when the test ends.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for Scratch {
    type Target = PathBuf;
    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

fn scratch(tag: &str) -> Scratch {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-export-all-{tag}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&d);
    Scratch(d)
}

/// A copy of model_gti in a scratch directory.
fn gti_copy(tag: &str) -> Scratch {
    let s = scratch(tag);
    copy_dir(&gti(), &s);
    s
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &p);
        } else {
            std::fs::copy(e.path(), p).unwrap();
        }
    }
}

fn run(model: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn files(dir: &Path) -> Vec<(PathBuf, String)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(PathBuf, String)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                out.push((p.strip_prefix(base).unwrap().to_path_buf(), std::fs::read_to_string(&p).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out
}

#[test]
fn the_committed_grid_tie_renders_are_current() {
    // the CI gate: model_gti/generated is what the model renders to today
    let (o, c) = run(&gti(), &["diagram", "export", "--all", "--check"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("up to date"), "{o}");
}

#[test]
fn every_format_renders_the_same_bytes_twice() {
    for format in ["svg", "plantuml", "mermaid", "dot"] {
        let (a, b) = (scratch("a"), scratch("b"));
        for d in [&a, &b] {
            let (o, c) = run(&gti(), &["diagram", "export", "--all", "--format", format, "--out-dir", d.to_str().unwrap()]);
            assert_eq!(c, 0, "{format}: {o}");
        }
        let (fa, fb) = (files(&a), files(&b));
        assert!(fa.len() >= 8, "{format}: {:?}", fa.iter().map(|f| &f.0).collect::<Vec<_>>());
        assert_eq!(fa, fb, "{format}: two runs differ");
    }
}

#[test]
fn every_svg_shape_and_edge_names_its_model_element() {
    let out = scratch("ref");
    let (o, c) = run(&gti(), &["diagram", "export", "--all", "--out-dir", out.to_str().unwrap()]);
    assert_eq!(c, 0, "{o}");
    let mut drawn = 0;
    for (path, svg) in files(&out) {
        for line in svg.lines() {
            let l = line.trim_start();
            if l.contains(" id=\"s-") || l.contains(" id=\"e-") {
                drawn += 1;
                assert!(l.contains("sysml:ref=\""), "{}: no sysml:ref on {l}", path.display());
            }
        }
    }
    assert!(drawn > 100, "only {drawn} shapes and edges");
}

#[test]
fn editing_a_drawn_element_makes_the_check_fail_until_regenerated() {
    let root = gti_copy("edit");
    let (o, c) = run(&root, &["diagram", "export", "--all", "--check"]);
    assert_eq!(c, 0, "{o}");
    let f = root.join("System/GridTieInverter.md");
    let src = std::fs::read_to_string(&f).unwrap();
    assert!(src.contains("name: Grid-Tie Inverter"), "fixture moved");
    std::fs::write(&f, src.replace("name: Grid-Tie Inverter", "name: Grid-Tie Inverter Mk2")).unwrap();
    let (o, c) = run(&root, &["diagram", "export", "--all", "--check"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("differs:") && o.contains("GtiIbd.svg"), "{o}");
    let (o, c) = run(&root, &["diagram", "export", "--all"]);
    assert_eq!(c, 0, "{o}");
    assert!(std::fs::read_to_string(root.join("generated/Diagrams/GtiIbd.svg")).unwrap().contains("Mk2"));
    let (o, c) = run(&root, &["diagram", "export", "--all", "--check"]);
    assert_eq!(c, 0, "{o}");
}

#[test]
fn a_render_no_diagram_produces_is_reported_then_removed() {
    let root = gti_copy("stale");
    std::fs::remove_file(root.join("Diagrams/GtiPowerView.md")).unwrap();
    std::fs::write(root.join("generated/Diagrams/Notes.txt"), "kept: not a render\n").unwrap();
    let (o, c) = run(&root, &["diagram", "export", "--all", "--check"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("left over:") && o.contains("GtiPowerView.svg"), "{o}");
    let (o, c) = run(&root, &["diagram", "export", "--all"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("removed") && !root.join("generated/Diagrams/GtiPowerView.svg").exists(), "{o}");
    // only files of the exported format are the export's own
    assert!(root.join("generated/Diagrams/Notes.txt").exists());
    let (o, c) = run(&root, &["diagram", "export", "--all", "--check"]);
    assert_eq!(c, 0, "{o}");
}

#[test]
fn a_missing_render_is_reported() {
    let root = gti_copy("missing");
    std::fs::remove_file(root.join("generated/Diagrams/GtiBdd.svg")).unwrap();
    let (o, c) = run(&root, &["diagram", "export", "--all", "--check"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("missing:") && o.contains("GtiBdd.svg"), "{o}");
}

#[test]
fn all_takes_at_most_one_configuration() {
    let (o, c) = run(&gti(), &["diagram", "export", "--all", "--check", "--config", "CONF-GTI-US-M3-001", "--config", "CONF-GTI-US-M3-001"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("one --config"), "{o}");
}

#[test]
fn renders_do_not_depend_on_where_the_checkout_is() {
    // two checkouts at different paths, and the default output directory inside each: PlantUML names its style
    // `!include` relative to the file, so even its text is the same
    let (a, b) = (gti_copy("where-a"), gti_copy("where-b"));
    let style = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.plantuml");
    for root in [&a, &b] {
        // model_gti's style file is `../.plantuml/sysml.iuml`, beside the model in the checkout
        copy_dir(&style, &root.join("../.plantuml"));
        std::fs::remove_dir_all(root.join("generated")).unwrap();
    }
    for format in ["svg", "plantuml"] {
        for root in [&a, &b] {
            let (o, c) = run(root, &["diagram", "export", "--all", "--format", format]);
            assert_eq!(c, 0, "{format}: {o}");
        }
        assert_eq!(files(&a.join("generated")), files(&b.join("generated")), "{format}");
    }
    let puml = std::fs::read_to_string(a.join("generated/Diagrams/GtiBdd.puml")).unwrap();
    assert!(puml.contains("!include ../../../.plantuml/sysml.iuml\n"), "{puml}");
    let _ = std::fs::remove_dir_all(a.join("../.plantuml"));
}

#[test]
fn only_files_the_export_wrote_are_ever_removed() {
    let root = gti_copy("safe");
    // a hand-drawn picture of the export's format in the output directory, and the hand-drawn companions beside the
    // diagrams, exported into their own directory
    std::fs::write(root.join("generated/Diagrams/Hand.svg"), "<svg/>").unwrap();
    let (o, c) = run(&root, &["diagram", "export", "--all", "--check"]);
    assert_eq!(c, 0, "an unlisted file is not left over: {o}");
    let (o, c) = run(&root, &["diagram", "export", "--all"]);
    assert_eq!(c, 0, "{o}");
    assert!(root.join("generated/Diagrams/Hand.svg").exists(), "{o}");
    let diagrams = root.join("Diagrams");
    for _ in 0..2 {
        let (o, c) = run(&root, &["diagram", "export", "--all", "--out-dir", diagrams.to_str().unwrap()]);
        assert_eq!(c, 0, "{o}");
        assert!(!o.contains("removed"), "{o}");
    }
    assert!(diagrams.join("GtiFirmwareView.svg").exists() && diagrams.join("GtiBlockDefinitionView.svg").exists());
    // a directory that holds the model is refused
    for dir in [root.to_str().unwrap().to_string(), format!("{}/..", root.display())] {
        let (o, c) = run(&root, &["diagram", "export", "--all", "--out-dir", &dir]);
        assert_eq!(c, 1, "{o}");
        assert!(o.contains("contains the model"), "{o}");
    }
}

#[cfg(unix)]
#[test]
fn a_manifest_entry_through_a_link_never_removes_a_file_outside() {
    let root = gti_copy("link");
    let outside = scratch("outside");
    std::fs::create_dir_all(&*outside).unwrap();
    std::fs::write(outside.join("keep.svg"), "<svg/>").unwrap();
    std::os::unix::fs::symlink(&*outside, root.join("generated/ext")).unwrap();
    let m = root.join("generated/.syscribe-generated");
    let mut text = std::fs::read_to_string(&m).unwrap();
    text.push_str("ext/keep.svg\n../escape.svg\n");
    std::fs::write(&m, text).unwrap();
    std::fs::write(root.join("escape.svg"), "<svg/>").unwrap();
    let (o, c) = run(&root, &["diagram", "export", "--all"]);
    assert_eq!(c, 0, "{o}");
    assert!(outside.join("keep.svg").exists() && root.join("escape.svg").exists(), "{o}");
}

#[test]
fn a_diagram_that_fails_to_render_fails_the_run_and_keeps_its_file() {
    let root = gti_copy("fail");
    let d = root.join("Diagrams/GtiBdd.md");
    let src = std::fs::read_to_string(&d).unwrap();
    std::fs::write(&d, src.replace("subject: System::GridTieInverter", "subject: System::NoSuchPart")).unwrap();
    assert_ne!(std::fs::read_to_string(&d).unwrap(), src, "fixture moved");
    let (o, c) = run(&root, &["diagram", "export", "--all", "--check"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("error:") && o.contains("GtiBdd") && !o.contains("left over"), "{o}");
    let (o, c) = run(&root, &["diagram", "export", "--all"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("removed nothing") && root.join("generated/Diagrams/GtiBdd.svg").exists(), "{o}");
}

#[test]
fn one_configuration_renders_beside_the_whole_model() {
    let root = gti_copy("config");
    let before = files(&root.join("generated/Diagrams"));
    let (o, c) = run(&root, &["diagram", "export", "--all", "--config", "CONF-GTI-US-M3-001"]);
    assert_eq!(c, 0, "{o}");
    assert!(root.join("generated/CONF-GTI-US-M3-001/Diagrams/GtiIbd.svg").exists(), "{o}");
    assert_eq!(files(&root.join("generated/Diagrams")), before, "the whole-model renders are untouched");
    let (o, c) = run(&root, &["diagram", "export", "--all", "--check"]);
    assert_eq!(c, 0, "{o}");
}

#[test]
fn flags_that_do_not_fit_are_refused() {
    for args in [
        &["diagram", "export", "Diagrams::GtiBdd", "--check"][..],
        &["diagram", "export", "Diagrams::GtiBdd", "--out-dir", "x"][..],
        &["diagram", "export", "--all", "Diagrams::GtiBdd"][..],
        &["diagram", "export", "--all", "--out", "x.svg"][..],
    ] {
        let (o, c) = run(&gti(), args);
        assert_eq!(c, 1, "{args:?}: {o}");
    }
}

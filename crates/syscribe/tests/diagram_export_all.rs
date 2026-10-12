//! TC-TRS-VIS-032 / GH #339: `diagram export --all` renders every diagram of a model to a directory, byte for
//! byte the same on every run and machine, and `--check` fails when a committed render is out of date.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn gti() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_gti")
}

fn scratch(tag: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-export-all-{tag}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&d);
    d
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
        // PlantUML names its style include by absolute path, which is the same within one checkout
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
    let root = scratch("edit");
    copy_dir(&gti(), &root);
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
    let root = scratch("stale");
    copy_dir(&gti(), &root);
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
    let root = scratch("missing");
    copy_dir(&gti(), &root);
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

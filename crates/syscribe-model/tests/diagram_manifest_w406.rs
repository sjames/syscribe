//! TC-TRS-W406-001 / GH #263.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn codes(diagram: &str) -> Vec<String> {
    static N: AtomicU64 = AtomicU64::new(0);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-w406-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(r.join("Arch")).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(r.join("Arch/Unit.md"), "---\ntype: PartDef\nname: Unit\n---\n").unwrap();
    std::fs::write(r.join("Arch/D.md"), diagram).unwrap();
    let els = walk_model(&r).unwrap();
    validate(&els).findings.into_iter().map(|f| f.code.to_string()).collect()
}

const MANIFEST_FM: &str = "---\ntype: Diagram\nname: D\ndiagramKind: BDD\nsubject: Arch::Unit\nshapes:\n  - {id: s-unit, ref: Arch::Unit, kind: part}\n  - {id: s-other, ref: Arch::Unit, kind: part}\nedges:\n  - {id: e-1, source: s-unit, target: s-other, kind: connection}\n---\n";

#[test]
fn manifest_diagram_without_inline_svg_raises_neither() {
    let c = codes(&format!("{MANIFEST_FM}\nBody text only.\n"));
    assert!(!c.iter().any(|x| x == "W406" || x == "W407"), "{c:?}");
}

#[test]
fn manifest_diagram_with_an_inline_svg_is_still_checked() {
    let c = codes(&format!("{MANIFEST_FM}\n```svg\n<svg xmlns=\"http://www.w3.org/2000/svg\"><g id=\"s-unit\"/></svg>\n```\n"));
    assert!(c.iter().any(|x| x == "W406"), "stale SVG must still be flagged: {c:?}");
}

#[test]
fn mapping_form_manifest_without_inline_svg_raises_neither() {
    let fm = "---\ntype: Diagram\nname: D\ndiagramKind: BDD\nsubject: Arch::Unit\nshapes:\n  s-a: {ref: Arch::Unit}\n  s-b: Arch::Unit\nedges:\n  e-1: {source: s-a, target: s-b, kind: connection}\n---\n\nNo svg.\n";
    let c = codes(fm);
    assert!(!c.iter().any(|x| x == "W406" || x == "W407"), "{c:?}");
}

#[test]
fn mapping_form_manifest_with_a_stale_inline_svg_is_still_checked() {
    let fm = "---\ntype: Diagram\nname: D\ndiagramKind: BDD\nsubject: Arch::Unit\nshapes:\n  s-a: {ref: Arch::Unit}\n  s-b: Arch::Unit\n---\n\n```svg\n<svg xmlns=\"http://www.w3.org/2000/svg\"><g id=\"s-a\"/></svg>\n```\n";
    let c = codes(fm);
    assert!(c.iter().any(|x| x == "W406"), "{c:?}");
}

//! `REQ-TRS-SYSMLV2-031`: the shared submodel inspection data (`sysmlv2::report`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::sysmlv2::report::{submodels, submodels_json};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-report-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn model(files: &[(&str, &str)]) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "S/_index.md", "---\ntype: Package\nname: S\nsysmlSubmodel: true\n---\n");
    for (n, c) in files {
        write(&root, &format!("S/{n}"), c);
    }
    root
}

#[test]
fn report_lists_files_kinds_unmapped_and_findings() {
    let root = model(&[
        ("A.sysml", "package P {\n  part def X;\n  part def Y;\n  individual def C;\n}\n"),
        ("B.sysml", "package Q { port def Pt; }\n"),
        ("Bad.sysml", "package { this is not valid ((("),
    ]);
    let elements = walk_model(&root).unwrap();
    let subs = submodels(&elements);
    assert_eq!(subs.len(), 1);
    let s = &subs[0];
    assert_eq!(s.package, "S");
    assert_eq!(s.files.len(), 3);
    assert_eq!(s.files_parsed(), 2);
    assert_eq!(s.elements_by_kind.get("PartDef"), Some(&2));
    assert_eq!(s.elements_by_kind.get("PortDef"), Some(&1));
    assert_eq!(s.unmapped.get("individual def"), Some(&1));
    let codes: Vec<&str> = s.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(codes.contains(&"W543"), "{codes:?}");
    assert!(codes.contains(&"W541"), "{codes:?}");
}

#[test]
fn report_json_shape_and_no_submodel_is_empty() {
    let root = model(&[("A.sysml", "package P { part def X; }\n")]);
    let elements = walk_model(&root).unwrap();
    let j = submodels_json(&elements);
    let s = &j["submodels"][0];
    assert_eq!(s["package"], "S");
    assert_eq!(s["fileCount"], 1);
    assert_eq!(s["filesParsed"], 1);
    assert_eq!(s["elementsByKind"]["PartDef"], 1);
    assert_eq!(s["unmappedTotal"], 0);
    assert!(s["findings"].as_array().unwrap().is_empty());

    let plain = tempdir();
    write(&plain, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    let elements = walk_model(&plain).unwrap();
    assert!(submodels(&elements).is_empty());
    assert!(submodels_json(&elements)["submodels"].as_array().unwrap().is_empty());
}

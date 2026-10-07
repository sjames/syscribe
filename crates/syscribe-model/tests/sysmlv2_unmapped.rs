//! `REQ-TRS-SYSMLV2-030`: unmapped SysMLv2 constructs raise one advisory `W543` per file.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-unmapped-{}-{}",
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
fn one_w543_per_file_with_per_kind_counts() {
    let root = model(&[
        (
            "A.sysml",
            "package P {\n  part def Keep;\n  calc def C1 { return x : Real; }\n  calc def C2 { return y : Real; }\n  constraint def K { }\n  use case def U;\n}\n",
        ),
        ("B.sysml", "package Q { part def Keep2; }\n"),
    ]);
    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let w: Vec<_> = result.findings.iter().filter(|f| f.code == "W543").collect();
    assert_eq!(w.len(), 1, "{:#?}", result.findings);
    assert!(w[0].file.ends_with("A.sysml"));
    for k in ["calc def x2", "constraint def x1", "use case def x1"] {
        assert!(w[0].message.contains(k), "missing {k} in {}", w[0].message);
    }
    // Advisory: the mapped part defs are still synthesized.
    assert!(elements.iter().any(|e| e.qualified_name == "S::P::Keep"));
    assert!(elements.iter().any(|e| e.qualified_name == "S::Q::Keep2"));
}

#[test]
fn fully_mapped_file_is_silent() {
    let root = model(&[("A.sysml", "package P { import ScalarValues::*; part def Keep; }\n")]);
    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    assert!(result.findings.iter().all(|f| f.code != "W543"), "{:#?}", result.findings);
}

#[test]
fn nested_package_members_are_counted() {
    let root = model(&[("A.sysml", "package P { package In { metadata def M; } }\n")]);
    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let w: Vec<_> = result.findings.iter().filter(|f| f.code == "W543").collect();
    assert_eq!(w.len(), 1);
    assert!(w[0].message.contains("metadata def x1"));
}

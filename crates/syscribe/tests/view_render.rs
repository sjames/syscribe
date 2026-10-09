//! `view render` and View `expose:` patterns/filters (GH #205).

use std::process::Command;

fn bin() -> String {
    env!("CARGO_BIN_EXE_syscribe").to_string()
}

fn model(tag: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-view-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let w = |rel: &str, body: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    };
    w("_index.md", "---\ntype: Package\n---\n");
    w("A/_index.md", "---\ntype: Package\n---\nA.\n");
    w("A/Eng.md", "---\ntype: PartDef\nmetadata: [Safety]\n---\nx\n");
    w("A/Plain.md", "---\ntype: PartDef\n---\nx\n");
    w("A/Sub/_index.md", "---\ntype: Package\n---\nS.\n");
    w("A/Sub/Deep.md", "---\ntype: PartDef\nmetadata: [Safety]\n---\nx\n");
    w("Meta/Safety.md", "---\ntype: MetadataDef\n---\nx\n");
    w("Views/Direct.md", "---\ntype: View\nexpose: ['A::*']\n---\nx\n");
    w("Views/Deep.md", "---\ntype: View\nexpose:\n  - {target: 'A::**', filter: '@Safety'}\n---\nx\n");
    w("Views/Rel.md", "---\ntype: ViewDef\nexpose: ['Nowhere::*']\n---\nx\n");
    root
}

fn run(root: &std::path::Path, args: &[&str]) -> (i32, String, String) {
    let o = Command::new(bin()).arg("-m").arg(root).args(args).output().unwrap();
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

#[test]
fn wildcard_and_filter_select_elements() {
    let root = model("select");
    let (c, out, _) = run(&root, &["view", "render", "Views::Direct", "--format", "table"]);
    assert_eq!(c, 0);
    assert!(out.contains("A::Eng") && out.contains("A::Plain") && out.contains("A::Sub"), "{out}");
    assert!(!out.contains("A::Sub::Deep"), "`A::*` is direct members only: {out}");

    let (c, out, _) = run(&root, &["view", "render", "Views::Deep", "--format", "table"]);
    assert_eq!(c, 0);
    assert!(out.contains("A::Eng") && out.contains("A::Sub::Deep"), "{out}");
    assert!(!out.contains("A::Plain"), "filter @Safety excludes Plain: {out}");

    let (_, out, _) = run(&root, &["view", "render", "Views::Deep", "--format", "mermaid"]);
    assert!(out.starts_with("flowchart TD"));
    let (_, out, _) = run(&root, &["view", "render", "Views::Deep", "--format", "json"]);
    assert!(out.contains("\"qualifiedName\": \"A::Eng\""));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn unresolved_expose_warns_w502_for_a_viewdef_too_and_wildcards_resolve() {
    let root = model("w502");
    let (_, out, _) = run(&root, &["validate"]);
    assert!(out.contains("W502") && out.contains("Nowhere::*"), "{out}");
    assert_eq!(out.matches("W502").count(), 1, "A::* and A::** must resolve: {out}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn errors_are_reported() {
    let root = model("errors");
    assert_eq!(run(&root, &["view", "render", "A::Eng"]).0, 1);
    assert_eq!(run(&root, &["view", "render", "Views::Nope"]).0, 1);
    assert_eq!(run(&root, &["view"]).0, 2);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn ingested_sysml_view_filter_becomes_filter_condition() {
    let root = std::env::temp_dir().join(format!("syscribe-view-{}-ingest", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("Sys")).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\n---\n").unwrap();
    std::fs::write(root.join("Sys/_index.md"), "---\ntype: Package\nsysmlSubmodel: true\n---\n").unwrap();
    std::fs::write(
        root.join("Sys/v.sysml"),
        "package V {\n  metadata def SafetyTag;\n  part def Eng { @SafetyTag; }\n  part def Plain;\n  view def ArchView { filter @SafetyTag; }\n  view arch : ArchView { expose V::*; render asTreeDiagram; }\n}\n",
    )
    .unwrap();
    let (c, out, _) = run(&root, &["view", "render", "Sys::V::arch", "--format", "table"]);
    assert_eq!(c, 0);
    assert!(out.contains("Sys::V::Eng") && !out.contains("Sys::V::Plain"), "{out}");
    let _ = std::fs::remove_dir_all(&root);
}

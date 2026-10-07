//! `REQ-TRS-VIS-002`: the one manifest parser (`vis::manifest`) feeds the
//! validator, which reports a malformed `shapes:`/`edges:`/`layout:` entry as
//! `E405` and a stale `layout:` pin as `W416` — through the real walker and
//! validator on a temp model, not the parser's unit tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::resolver::Resolver;
use syscribe_model::validator::{validate, Finding};
use syscribe_model::vis::{build_graph, NodeKind};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-vis-manifest-{}-{}",
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

/// A small model with two `PartDef`s plus one `Diagram` whose frontmatter
/// body (everything after `type: Diagram`) is `diagram_fm`.
fn model_with_diagram(diagram_fm: &str, body: &str) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
    write(&root, "Sys/Engine.md", "---\ntype: PartDef\nname: Engine\n---\n\nThe engine.\n");
    write(&root, "Sys/Motor.md", "---\ntype: PartDef\nname: Motor\n---\n\nThe motor.\n");
    write(&root, "Diagrams/_index.md", "---\ntype: Package\nname: Diagrams\n---\n");
    write(
        &root,
        "Diagrams/D.md",
        &format!("---\ntype: Diagram\nname: D\n{diagram_fm}---\n\n{body}\n"),
    );
    root
}

fn diagram_findings(root: &Path) -> Vec<Finding> {
    let elements = walk_model(root).unwrap();
    validate(&elements)
        .findings
        .into_iter()
        .filter(|f| f.file.ends_with("Diagrams/D.md"))
        .collect()
}

fn codes<'a>(findings: &'a [Finding], code: &str) -> Vec<&'a Finding> {
    findings.iter().filter(|f| f.code == code).collect()
}

#[test]
fn string_shorthand_shapes_are_well_formed_and_reach_the_ir() {
    let root = model_with_diagram(
        "diagramKind: BDD\nsubject: Sys\nshapes:\n  engine: Sys::Engine\n  motor: Sys::Motor\nedges:\n  inh:\n    source: motor\n    target: engine\n    kind: inheritance\n",
        "A BDD written with the string shorthand.",
    );
    let findings = diagram_findings(&root);
    assert!(codes(&findings, "E405").is_empty(), "{findings:?}");
    assert!(codes(&findings, "W416").is_empty(), "{findings:?}");

    let elements = walk_model(&root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::D").unwrap();
    let (graph, issues) = build_graph(d, &elements, &resolver).expect("a BDD has an IR");
    assert!(issues.is_empty(), "{issues:?}");
    let engine = graph.node("engine").expect("shorthand shape is in the IR");
    assert_eq!(engine.kind, NodeKind::Block);
    assert!(engine.resolved);
    assert_eq!(engine.element_type.as_deref(), Some("PartDef"));
    assert_eq!(engine.label, "Engine");
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn unknown_shape_kind_is_one_e405_naming_the_shape_and_the_rest_still_validates() {
    let root = model_with_diagram(
        "diagramKind: BDD\nsubject: Sys\nshapes:\n  engine: Sys::Engine\n  weird:\n    ref: Sys::Motor\n    kind: gizmo\n  motor:\n    ref: Sys::Motor\n    kind: PartDef\n",
        "One shape with a kind outside the vocabulary.",
    );
    let findings = diagram_findings(&root);
    let e405 = codes(&findings, "E405");
    assert_eq!(e405.len(), 1, "{findings:?}");
    assert!(e405[0].message.contains("`weird`"), "{}", e405[0].message);
    assert!(e405[0].message.contains("gizmo"), "{}", e405[0].message);

    let elements = walk_model(&root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::D").unwrap();
    let (graph, _) = build_graph(d, &elements, &resolver).unwrap();
    assert!(graph.node("engine").is_some(), "the well-formed shapes survive");
    assert!(graph.node("motor").is_some());
    assert!(graph.node("weird").is_none(), "the malformed entry is skipped, not the diagram");
}

#[test]
fn stale_layout_key_is_w416() {
    let root = model_with_diagram(
        "diagramKind: BDD\nsubject: Sys\nshapes:\n  engine: Sys::Engine\nlayout:\n  engine:\n    x: 10\n    y: 20\n  ghost:\n    x: 0\n    y: 0\n",
        "A pin for a shape that no longer exists.",
    );
    let findings = diagram_findings(&root);
    assert!(codes(&findings, "E405").is_empty(), "{findings:?}");
    let w416 = codes(&findings, "W416");
    assert_eq!(w416.len(), 1, "{findings:?}");
    assert!(w416[0].message.contains("`ghost`"), "{}", w416[0].message);
}

#[test]
fn shapes_sequence_of_scalars_is_e405() {
    let root = model_with_diagram(
        "diagramKind: IBD\nsubject: Sys::Engine\nshapes:\n  - Sys::Engine\n  - Sys::Motor\n",
        "shapes: as a list of strings, which the schema does not define.",
    );
    let findings = diagram_findings(&root);
    let e405 = codes(&findings, "E405");
    assert!(!e405.is_empty(), "{findings:?}");
    assert!(e405.iter().any(|f| f.message.contains("`shapes`")), "{findings:?}");
}

#[test]
fn mermaid_kind_has_no_ir_and_no_e405() {
    let root = model_with_diagram(
        "diagramKind: Mermaid\nsubject: Sys\nshapes:\n  - 1\n  - 2\n",
        "```mermaid\ngraph TD\n  %% ref: Sys::Engine\n  A[Engine] --> B[Motor]\n```",
    );
    let findings = diagram_findings(&root);
    assert!(codes(&findings, "E405").is_empty(), "{findings:?}");
    assert!(codes(&findings, "W416").is_empty(), "{findings:?}");
    assert!(codes(&findings, "E400").is_empty(), "the mermaid block is present: {findings:?}");

    let elements = walk_model(&root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::D").unwrap();
    assert!(build_graph(d, &elements, &resolver).is_none(), "a Mermaid-kind diagram has no IR");
}

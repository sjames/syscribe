//! `REQ-TRS-VIS-009` (Phase 0): the PlantUML writer is a pure function of the
//! Diagram IR, and its output for every `pumlMode: companion` diagram in the
//! demo model (`model/`) is pinned by the snapshots under
//! `tests/snapshots/plantuml/<qname>.puml`.
//!
//! The snapshots were produced from the pre-IR writer (the one that parsed
//! `shapes:`/`edges:` itself) on the same diagrams, rendered without a
//! `PlantumlConfig` so they carry the built-in skinparam preamble and no
//! machine-specific `!include` path or `[[link]]`s. One line differs from
//! that oracle by design: `Views::SafetyReqDiagram` draws `s-flightc` (a
//! `PartDef` the manifest tags `kind: Part`) as `<<part def>>`, because the
//! IR's stereotype is the resolved element's real type, not the manifest's
//! word for it.
//!
//! To refresh after an intentional change: `SYSCRIBE_UPDATE_SNAPSHOTS=1 cargo
//! test -p syscribe-model --test vis_plantuml_snapshot`, then review the diff.

use std::path::{Path, PathBuf};

use syscribe_model::element::ElementType;
use syscribe_model::plantuml::render_plantuml;
use syscribe_model::walker::walk_model;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/plantuml")
}

#[test]
fn companion_diagrams_of_the_demo_model_match_their_snapshots() {
    let model = repo_root().join("model");
    let elements = walk_model(&model).expect("walk the demo model");
    let update = std::env::var_os("SYSCRIBE_UPDATE_SNAPSHOTS").is_some();

    let companions: Vec<_> = elements
        .iter()
        .filter(|e| {
            e.frontmatter.element_type == Some(ElementType::Diagram)
                && e.frontmatter.puml_mode.as_deref() == Some("companion")
        })
        .collect();
    assert!(
        companions.len() >= 10,
        "the demo model carries the companion diagrams this test pins; found {}",
        companions.len()
    );

    let mut mismatches = Vec::new();
    for elem in &companions {
        let qname = &elem.qualified_name;
        let rendered = render_plantuml(elem, &elements, None)
            .unwrap_or_else(|| panic!("{qname}: a companion diagram's kind has a PlantUML mapping"));
        // `::` is not a legal path character on Windows (the release matrix checks
        // the repository out there), so a qualified name maps to `__` in the file name.
        let path = snapshot_dir().join(format!("{}.puml", qname.replace("::", "__")));
        if update {
            std::fs::create_dir_all(snapshot_dir()).unwrap();
            std::fs::write(&path, &rendered).unwrap();
            continue;
        }
        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{qname}: missing snapshot {}: {e}", path.display()));
        if expected != rendered {
            mismatches.push(format!(
                "--- {qname} (snapshot {})\nexpected:\n{expected}\nrendered:\n{rendered}",
                path.display()
            ));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn snapshot_directory_has_no_orphans() {
    // Every snapshot file corresponds to a companion diagram that still exists,
    // so a renamed or removed diagram cannot leave a stale, never-checked file.
    if std::env::var_os("SYSCRIBE_UPDATE_SNAPSHOTS").is_some() {
        // The update run of the sibling test may still be writing the files.
        return;
    }
    let model = repo_root().join("model");
    let elements = walk_model(&model).expect("walk the demo model");
    let qnames: std::collections::BTreeSet<String> = elements
        .iter()
        .filter(|e| e.frontmatter.puml_mode.as_deref() == Some("companion"))
        .map(|e| e.qualified_name.replace("::", "__"))
        .collect();
    for entry in std::fs::read_dir(snapshot_dir()).expect("snapshot directory exists") {
        let name = entry.unwrap().file_name().to_string_lossy().to_string();
        let Some(stem) = name.strip_suffix(".puml") else {
            panic!("unexpected file in snapshot directory: {name}");
        };
        assert!(qnames.contains(stem), "snapshot {name} names no companion diagram of the demo model");
    }
}

#[test]
fn kinds_without_a_plantuml_mapping_render_nothing() {
    let model = repo_root().join("model");
    let elements = walk_model(&model).expect("walk the demo model");
    for e in elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Diagram)) {
        let kind = e.frontmatter.diagram_kind.as_deref();
        let mapped = matches!(kind, Some("BDD" | "IBD" | "StateMachine" | "Sequence" | "Requirement" | "Action" | "Allocation" | "UseCase"));
        assert_eq!(
            render_plantuml(e, &elements, None).is_some(),
            mapped,
            "{}: diagramKind {kind:?}",
            e.qualified_name
        );
    }
}

//! Integration coverage for issue #107: widening `resolve_scoped_ref`
//! (`REQ-TRS-SYSMLV2-016`) to `graph.rs`'s `TypedBy` edge and `W007`'s
//! "defined but never used as a supertype or type" usage tracking, so a
//! SysMLv2-authored, package-relative `typedBy:`/`supertype:` reference
//! (the literal text a `.sysml` author wrote, e.g. `part x :
//! Services::Documented;` inside a *different* package than `Documented`'s
//! own) is recognised as real usage and as a real graph edge, not only an
//! already-fully-qualified reference.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::graph::{build_graph, EdgeKind};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-typed-by-scoped-test-{}-{}",
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

fn base_model(root: &Path) {
    write(root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
}

/// Issue #107's own minimal repro, ingested through the real SysMLv2 pipeline
/// (not a hand-built `RawElement`, unlike the `validator.rs` unit tests) —
/// `Services::Documented`, referenced only from `System::Top` via the
/// package-relative `typedBy: "Services::Documented"` `ingest.rs` produces
/// verbatim for `part x : Services::Documented;`.
fn write_repro(root: &Path) {
    write(
        root,
        "SysML2Legacy/Model.sysml",
        "package Services {\n\
         part def Documented {\n\
         doc /* Real documentation text here. */\n\
         }\n\
         }\n\
         package System {\n\
         part def Top {\n\
         doc /* Top's own doc. */\n\
         part x : Services::Documented;\n\
         }\n\
         }\n",
    );
}

#[test]
fn a_cross_package_typed_by_reference_is_a_real_graph_edge() {
    let root = tempdir();
    base_model(&root);
    write_repro(&root);

    let elements = walk_model(&root).unwrap();
    let (graph, idx) = build_graph(&elements);
    let x_idx = *idx
        .get("SysML2Legacy::System::Top::x")
        .expect("x node should exist");
    let documented_idx = *idx
        .get("SysML2Legacy::Services::Documented")
        .expect("Documented node should exist");

    let has_edge = graph
        .edges_connecting(x_idx, documented_idx)
        .any(|e| *e.weight() == EdgeKind::TypedBy);
    assert!(
        has_edge,
        "expected a real TypedBy edge from Top::x to Services::Documented \
         (graph.rs's exact-index lookup used to silently drop package-relative typedBy:)"
    );
}

#[test]
fn a_cross_package_typed_by_reference_suppresses_w007_on_the_referenced_def() {
    let root = tempdir();
    base_model(&root);
    write_repro(&root);

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let fires_for_documented = result
        .findings
        .iter()
        .any(|f| f.code == "W007" && f.message.contains("Services::Documented"));
    assert!(
        !fires_for_documented,
        "Services::Documented is used (cross-package) so W007 should not fire for it: {:?}",
        result.findings
    );
}

/// A genuinely unused `*Def` (nothing anywhere references it as
/// `supertype:`/`typedBy:`, cross-package or otherwise) still fires `W007` —
/// confirming the widening is additive, matching issue #107's own
/// acceptance criteria that the CarOS submodel's one genuine top-of-hierarchy
/// unused type must keep firing.
#[test]
fn a_genuinely_unused_cross_package_def_still_raises_w007() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Model.sysml",
        "package Services {\n\
         part def Orphan {\n\
         doc /* Never referenced anywhere. */\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let fires_for_orphan = result
        .findings
        .iter()
        .any(|f| f.code == "W007" && f.message.contains("Services::Orphan"));
    assert!(fires_for_orphan, "expected W007 for the genuinely unused Orphan: {:?}", result.findings);
}

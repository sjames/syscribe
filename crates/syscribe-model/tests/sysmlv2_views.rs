//! Integration tests for mapping `view def`/`view`, `viewpoint def`/
//! `viewpoint`, and `rendering def`/`rendering` onto the native
//! `ViewDef`/`View`/`ViewpointDef`/`RenderingDef`/`Rendering` schema
//! (`REQ-TRS-SYSMLV2-020`/`-021`/`-022`).
//!
//! Two-layer, mirroring `sysmlv2_states.rs`: (a) raw YAML shape assertions
//! on `frontmatter.expose`/`.viewpoint`/`.stakeholders`/`.concerns`/
//! `.rendering`, and (b) driving `validator::validate` to confirm the
//! *existing* `W500`/`W502` checks fire on synthesized input exactly as
//! they would on hand-authored input, with no `validator.rs` changes.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::ElementType;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-views-test-{}-{}",
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

fn codes(findings: &[syscribe_model::validator::Finding]) -> Vec<&str> {
    findings.iter().map(|f| f.code).collect()
}

#[test]
fn a_view_def_and_view_usage_become_real_elements_with_expose_viewpoint_rendering() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Views.sysml",
        "package Behavior {\n\
         viewpoint def SomeViewpoint { }\n\
         rendering def SomeRendering;\n\
         part def Target;\n\
         view def SomeViewDef { render r : SomeRendering; }\n\
         view myView : SomeViewDef { expose Target::*; satisfy SomeViewpoint; render r2 : SomeRendering; }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let qnames: Vec<&str> = elements.iter().map(|e| e.qualified_name.as_str()).collect();

    let view_def = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Behavior::SomeViewDef")
        .expect("SomeViewDef should be a real element");
    assert_eq!(view_def.frontmatter.element_type, Some(ElementType::ViewDef));
    assert_eq!(view_def.frontmatter.rendering.as_deref(), Some("SomeRendering"));
    // The grammar structurally cannot carry expose/viewpoint on a `view
    // def` -- confirm neither is ever populated here.
    assert!(view_def.frontmatter.expose.is_none(), "{:#?}", view_def.frontmatter.expose);
    assert!(view_def.frontmatter.viewpoint.is_none(), "{:#?}", view_def.frontmatter.viewpoint);

    let view_usage = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Behavior::myView")
        .expect("myView should be a real element");
    assert_eq!(view_usage.frontmatter.element_type, Some(ElementType::View), "{qnames:#?}");
    assert_eq!(view_usage.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("SomeViewDef"));
    assert_eq!(view_usage.frontmatter.viewpoint.as_deref(), Some("SomeViewpoint"));
    assert_eq!(view_usage.frontmatter.rendering.as_deref(), Some("SomeRendering"));
    let expose = view_usage.frontmatter.expose.as_deref().expect("expose present");
    assert_eq!(expose.len(), 1);
    // Flat plain-string entry, never the richer `{ref, isRecursive, filter}`
    // map form.
    assert_eq!(expose[0].as_str(), Some("Target::*"));
}

#[test]
fn viewpoint_def_lifts_stakeholders_and_concerns() {
    let root = tempdir();
    base_model(&root);
    // Real fixture syntax confirmed against the vendored parser's own
    // `tests/fixtures/viewpoint-stakeholder-purpose.sysml`.
    write(
        &root,
        "SysML2Legacy/Viewpoints.sysml",
        "package ViewpointDemo {\n\
         viewpoint def SafetyView {\n\
         stakeholder SafetyConcern;\n\
         purpose ReliabilityPurpose;\n\
         frame SafetyFrame;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::ViewpointDemo::SafetyView")
        .expect("SafetyView should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::ViewpointDef));
    assert_eq!(el.frontmatter.stakeholders.as_deref(), Some(&["SafetyConcern".to_string()][..]));
    assert_eq!(el.frontmatter.concerns.as_deref(), Some(&["ReliabilityPurpose".to_string()][..]));
    // `methods:`/`satisfiedBy:` are deliberately never populated -- no AST
    // source, and the OSLC upstream-link-direction rule points the other way.
    assert!(el.frontmatter.methods.is_none());
    assert!(el.frontmatter.satisfied_by.is_none());
}

#[test]
fn viewpoint_usage_maps_onto_element_type_view() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Viewpoints.sysml",
        "package ViewpointDemo {\n\
         viewpoint def SafetyView { }\n\
         viewpoint safety defined by SafetyView { }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::ViewpointDemo::safety")
        .expect("safety should be a real element");
    // No dedicated `Viewpoint` usage `ElementType` exists -- a `viewpoint`
    // usage maps onto `View`, matching the doc's own framing.
    assert_eq!(el.frontmatter.element_type, Some(ElementType::View));
    assert_eq!(el.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("SafetyView"));
}

#[test]
fn rendering_def_and_usage_become_real_elements() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Renderings.sysml",
        "package RenderingDemo {\n\
         rendering def MyRenderer;\n\
         rendering skin typed by MyRenderer;\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let def = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::RenderingDemo::MyRenderer")
        .expect("MyRenderer should be a real element");
    assert_eq!(def.frontmatter.element_type, Some(ElementType::RenderingDef));

    let usage = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::RenderingDemo::skin")
        .expect("skin should be a real element");
    assert_eq!(usage.frontmatter.element_type, Some(ElementType::Rendering));
    assert_eq!(usage.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()), Some("MyRenderer"));
}

#[test]
fn a_view_nested_inside_a_part_def_becomes_a_real_element() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Nested.sysml",
        "package Behavior {\n\
         view def SomeViewDef;\n\
         part def Housing {\n\
         view innerView : SomeViewDef;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Behavior::Housing::innerView")
        .expect("a view nested inside a part def body should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::View));
}

#[test]
fn a_view_nested_inside_a_part_usage_stays_invisible() {
    let root = tempdir();
    base_model(&root);
    // `PartUsageBodyElement` carries no variant for the whole
    // view/viewpoint/rendering family at all per this grammar
    // (`REQ-TRS-SYSMLV2-020`/`-021`/`-022`'s documented gap) -- stronger
    // than a silent per-element skip, this is an outright parser rejection:
    // a `view` declared directly inside a `part` usage body fails to parse
    // at all, gracefully degrading to a `W541` finding
    // (`REQ-TRS-SYSMLV2-006`) rather than a crash or a synthesized element.
    write(
        &root,
        "SysML2Legacy/Nested.sysml",
        "package Behavior {\n\
         view def SomeViewDef;\n\
         part def Housing;\n\
         part housing : Housing {\n\
         view innerView : SomeViewDef;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    assert!(
        !elements.iter().any(|e| e.qualified_name.ends_with("::innerView")),
        "a view nested inside a part usage body should stay invisible: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    let result = validate(&elements);
    assert!(codes(&result.findings).contains(&"W541"), "{:#?}", result.findings);
}

#[test]
fn w500_and_w502_fire_on_synthesized_output_exactly_as_on_hand_authored() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Broken.sysml",
        "package Behavior {\n\
         view def SomeViewDef;\n\
         part def Target;\n\
         view myView : SomeViewDef { expose Target::DoesNotExist; satisfy NoSuchViewpoint; }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let cs = codes(&result.findings);
    assert!(cs.contains(&"W500"), "{:#?}", result.findings);
    assert!(cs.contains(&"W502"), "{:#?}", result.findings);
}

#[test]
fn a_clean_view_raises_no_w500_or_w502() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Clean.sysml",
        "package Behavior {\n\
         viewpoint def SomeViewpoint { }\n\
         part def Target;\n\
         view def SomeViewDef;\n\
         view myView : SomeViewDef { expose Target; satisfy SomeViewpoint; }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let cs = codes(&result.findings);
    assert!(!cs.contains(&"W500"), "{:#?}", result.findings);
    assert!(!cs.contains(&"W502"), "{:#?}", result.findings);
}

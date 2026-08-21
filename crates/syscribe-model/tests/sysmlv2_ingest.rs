//! Integration tests for native SysML v2/KerML parsing + graph merge
//! (`REQ-TRS-SYSMLV2-002`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::ElementType;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-ingest-test-{}-{}",
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

#[test]
fn a_sysml_package_becomes_a_qname_mapped_package_element() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(&root, "SysML2Legacy/Sensors.sysml", "package Sensors { }\n");

    let elements = walk_model(&root).unwrap();

    let pkg = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Sensors")
        .unwrap_or_else(|| {
            panic!(
                "expected SysML2Legacy::Sensors, got: {:#?}",
                elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
            )
        });
    assert_eq!(pkg.frontmatter.element_type, Some(ElementType::Package));
    assert!(pkg.file_path.ends_with("Sensors.sysml"));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn nested_sysml_packages_derive_a_double_colon_qname() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Nested.sysml",
        "package Outer { package Inner { } }\n",
    );

    let elements = walk_model(&root).unwrap();

    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy::Outer"));
    assert!(elements
        .iter()
        .any(|e| e.qualified_name == "SysML2Legacy::Outer::Inner"));
}

#[test]
fn part_def_and_nested_part_usage_map_with_supertype_and_typed_by() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Vehicle.sysml",
        "package Vehicle {\n\
         part def Engine :> PowerSource {\n\
         part cylinder1 : Cylinder;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();

    let engine = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Vehicle::Engine")
        .unwrap_or_else(|| {
            panic!(
                "expected SysML2Legacy::Vehicle::Engine, got: {:#?}",
                elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
            )
        });
    assert_eq!(engine.frontmatter.element_type, Some(ElementType::PartDef));
    assert!(engine.file_path.ends_with("Vehicle.sysml"));
    assert_eq!(
        engine.frontmatter.supertype.as_ref().and_then(|v| v.as_str()),
        Some("PowerSource")
    );

    let cylinder = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Vehicle::Engine::cylinder1")
        .unwrap_or_else(|| {
            panic!(
                "expected nested part usage cylinder1, got: {:#?}",
                elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
            )
        });
    assert_eq!(cylinder.frontmatter.element_type, Some(ElementType::Part));
    assert_eq!(
        cylinder.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()),
        Some("Cylinder")
    );

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn variation_part_def_carries_is_variation() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Config.sysml",
        "package Config {\n variation part def RotorConfig;\n }\n",
    );

    let elements = walk_model(&root).unwrap();

    let rotor = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Config::RotorConfig")
        .unwrap_or_else(|| {
            panic!(
                "expected RotorConfig, got: {:#?}",
                elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
            )
        });
    assert_eq!(rotor.frontmatter.is_variation, Some(true));
}

#[test]
fn remaining_fixed_set_kinds_map_attribute_port_connection_interface_item_requirement_allocation() {
    // What syntax without an explicit `def` keyword resolves to (`*Def` vs.
    // `*Usage`) is the parser's own disambiguation call, confirmed against its
    // actual AST output rather than assumed: bare `attribute`/`port`/
    // `interface`/`item` land as `*Def` (their "def" keyword is optional to
    // this parser), while bare `connection`/`requirement`/`allocation` land as
    // `*Usage`. Either way, a Def's `:`/`:>` clause maps to `supertype` and a
    // Usage's maps to `typed_by` — this test locks in that observed mapping.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Mixed.sysml",
        "package Mixed {\n\
         attribute mass : Real;\n\
         port fuelPort : FuelPort;\n\
         connection wiring : Wire;\n\
         interface iface : SomeInterface;\n\
         item fuel : Fuel;\n\
         requirement enduranceReq : EnduranceReqType;\n\
         allocation allocA : AllocKind;\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let qnames: Vec<&String> = elements.iter().map(|e| &e.qualified_name).collect();

    let find = |qname: &str| {
        elements
            .iter()
            .find(|e| e.qualified_name == qname)
            .unwrap_or_else(|| panic!("expected {qname}, got: {qnames:#?}"))
    };
    let expect_supertype = |qname: &str, ty: ElementType, supertype: &str| {
        let e = find(qname);
        assert_eq!(e.frontmatter.element_type, Some(ty), "wrong type for {qname}");
        assert_eq!(
            e.frontmatter.supertype.as_ref().and_then(|v| v.as_str()),
            Some(supertype),
            "wrong supertype for {qname}"
        );
    };
    let expect_typed_by = |qname: &str, ty: ElementType, typed_by: &str| {
        let e = find(qname);
        assert_eq!(e.frontmatter.element_type, Some(ty), "wrong type for {qname}");
        assert_eq!(
            e.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()),
            Some(typed_by),
            "wrong typed_by for {qname}"
        );
    };

    expect_supertype("SysML2Legacy::Mixed::mass", ElementType::AttributeDef, "Real");
    expect_supertype("SysML2Legacy::Mixed::fuelPort", ElementType::PortDef, "FuelPort");
    expect_typed_by("SysML2Legacy::Mixed::wiring", ElementType::Connection, "Wire");
    expect_supertype("SysML2Legacy::Mixed::iface", ElementType::InterfaceDef, "SomeInterface");
    expect_supertype("SysML2Legacy::Mixed::fuel", ElementType::ItemDef, "Fuel");
    expect_typed_by(
        "SysML2Legacy::Mixed::enduranceReq",
        ElementType::Requirement,
        "EnduranceReqType",
    );
    expect_typed_by("SysML2Legacy::Mixed::allocA", ElementType::Allocation, "AllocKind");

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn variant_membership_inside_a_variation_part_def_maps_with_variant_of() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Variants.sysml",
        "package Variants {\n\
         variation part def RotorConfig {\n\
         variant part quad : QuadRotor;\n\
         variant part hex : HexRotor;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();

    let quad = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Variants::RotorConfig::quad")
        .unwrap_or_else(|| {
            panic!(
                "expected variant 'quad', got: {:#?}",
                elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
            )
        });
    assert_eq!(quad.frontmatter.element_type, Some(ElementType::Part));
    assert_eq!(quad.frontmatter.is_variant, Some(true));
    assert_eq!(
        quad.frontmatter.variant_of.as_deref(),
        Some("SysML2Legacy::Variants::RotorConfig")
    );
    assert_eq!(
        quad.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()),
        Some("QuadRotor")
    );

    assert!(elements
        .iter()
        .any(|e| e.qualified_name == "SysML2Legacy::Variants::RotorConfig::hex"));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn bare_variant_reference_to_a_sibling_usage_does_not_shadow_it() {
    // Regression: `variant quad;` (the untyped bare-reference form) used to
    // synthesize a second, hollow `RawElement` at the exact same qname as the
    // real `part quad : QuadRotor;` usage declared alongside it, silently
    // shadowing the real one in any qname-keyed lookup.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Variants.sysml",
        "package Variants {\n\
         variation part def Config {\n\
         part quad : QuadRotor;\n\
         variant quad;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let matches: Vec<_> = elements
        .iter()
        .filter(|e| e.qualified_name == "SysML2Legacy::Variants::Config::quad")
        .collect();

    // Exactly one element at that qname — the real usage, untouched.
    assert_eq!(
        matches.len(),
        1,
        "bare `variant quad;` must not create a duplicate/shadow element: {matches:#?}"
    );
    assert_eq!(matches[0].frontmatter.element_type, Some(ElementType::Part));
    assert_eq!(
        matches[0].frontmatter.typed_by.as_ref().and_then(|v| v.as_str()),
        Some("QuadRotor")
    );
    // The real usage is not itself marked as a variant by this bare reference
    // (that linkage is a later task's job) — it stays exactly what it was.
    assert_eq!(matches[0].frontmatter.is_variant, None);

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn dangling_bare_variant_reference_synthesizes_nothing() {
    // A bare `variant` reference to a name that doesn't exist anywhere in the
    // body: no diagnostic code is invented for this yet, it's simply invisible.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Variants.sysml",
        "package Variants {\n\
         variation part def Config {\n\
         variant ghost;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();

    assert!(!elements.iter().any(|e| e.qualified_name.contains("ghost")));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn a_file_mixing_mapped_and_unmapped_constructs_parses_fully_and_keeps_the_mapped_ones() {
    // REQ-TRS-SYSMLV2-007: full-grammar parsing, fixed-set mapping. `calc
    // def`/`case def` (still outside the fixed set even after
    // REQ-TRS-SYSMLV2-018/-019 moved State/Action into it — see
    // REQ-TRS-SYSMLV2-000's Scope, which explicitly keeps these deferred)
    // must not fail the parse or drop the file — they are simply invisible,
    // while a mapped `part def` in the very same file/package still comes
    // through.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Mixed.sysml",
        "package Boundary {\n\
         part def Vehicle;\n\
         calc def ComputeMargin;\n\
         case def InspectVehicle;\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();

    // The mapped PartDef came through despite the unmapped siblings.
    assert!(
        elements
            .iter()
            .any(|e| e.qualified_name == "SysML2Legacy::Boundary::Vehicle"),
        "mapped PartDef should survive alongside unmapped constructs: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    // None of the unmapped constructs synthesized an element.
    assert!(!elements.iter().any(|e| e.qualified_name.contains("ComputeMargin")));
    assert!(!elements.iter().any(|e| e.qualified_name.contains("InspectVehicle")));

    // No parse failure was recorded for the file — it's a fully successful
    // parse, just with a narrower mapped-element yield than its full content.
    let result = validate(&elements);
    let w541: Vec<_> = result.findings.iter().filter(|f| f.code == "W541").collect();
    assert!(w541.is_empty(), "unmapped constructs must not be parse failures: {w541:#?}");
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn a_named_port_nested_inside_an_interface_def_is_mapped() {
    // Regression: InterfaceDef/ConnectionDef/ItemDef/PortDef bodies were not
    // recursed into at all, so a named port declared directly on an interface
    // def (real, common SysML v2 usage) was silently invisible — never a
    // cross-reference target, no diagnostic.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Power.sysml",
        "package Power {\n\
         interface def PowerInterface {\n\
         port supplyPort : PowerOutPort;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();

    let port = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Power::PowerInterface::supplyPort")
        .unwrap_or_else(|| {
            panic!(
                "expected PowerInterface::supplyPort, got: {:#?}",
                elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
            )
        });
    assert_eq!(port.frontmatter.element_type, Some(ElementType::Port));
    assert_eq!(
        port.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()),
        Some("PowerOutPort")
    );

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn an_attribute_nested_inside_an_item_def_is_mapped() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Fuel.sysml",
        "package Fuel {\n\
         item def FuelItem {\n\
         attribute quantity : Real;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();

    let attr = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Fuel::FuelItem::quantity")
        .unwrap_or_else(|| {
            panic!(
                "expected FuelItem::quantity, got: {:#?}",
                elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
            )
        });
    // Bare `attribute quantity : Real;` resolves to AttributeUsage inside an
    // item def body (the parser's own def/usage disambiguation differs by
    // body context — see `remaining_fixed_set_kinds_map_...`'s note for the
    // package-level case, which resolves the same bare syntax to AttributeDef).
    assert_eq!(attr.frontmatter.element_type, Some(ElementType::Attribute));
    assert_eq!(
        attr.frontmatter.typed_by.as_ref().and_then(|v| v.as_str()),
        Some("Real")
    );

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn nested_members_of_connection_def_and_port_def_are_mapped() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Wiring.sysml",
        "package Wiring {\n\
         connection def Harness {\n\
         item pin : Pin;\n\
         }\n\
         port def SignalPort {\n\
         attribute level : Real;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();

    assert!(
        elements
            .iter()
            .any(|e| e.qualified_name == "SysML2Legacy::Wiring::Harness::pin"),
        "expected Harness::pin, got: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    assert!(
        elements
            .iter()
            .any(|e| e.qualified_name == "SysML2Legacy::Wiring::SignalPort::level"),
        "expected SignalPort::level, got: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn two_files_declaring_the_same_package_merge_into_one_namespace() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    // Two separate files both contribute to the same `Shared` SysML v2 package.
    write(&root, "SysML2Legacy/PartA.sysml", "package Shared { package Left { } }\n");
    write(&root, "SysML2Legacy/PartB.sysml", "package Shared { package Right { } }\n");

    let elements = walk_model(&root).unwrap();

    // Exactly one Shared package element — not two colliding on qname.
    let shared: Vec<_> = elements
        .iter()
        .filter(|e| e.qualified_name == "SysML2Legacy::Shared")
        .collect();
    assert_eq!(
        shared.len(),
        1,
        "same-named package across two files should merge into one element, got: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    // Both files' nested content landed inside the merged namespace.
    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy::Shared::Left"));
    assert!(elements.iter().any(|e| e.qualified_name == "SysML2Legacy::Shared::Right"));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors (e.g. a spurious E108): {:#?}", result.findings);
}

#[test]
fn a_parse_failure_in_one_file_does_not_abort_the_rest_of_the_subtree() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    // Malformed: unbalanced braces, should fail to parse.
    write(&root, "SysML2Legacy/Broken.sysml", "package Broken { part def X {\n");
    // A second, well-formed file in the same subtree.
    write(&root, "SysML2Legacy/Good.sysml", "package Good { }\n");

    let elements = walk_model(&root).unwrap();

    // The good file's package still made it into the graph.
    assert!(
        elements.iter().any(|e| e.qualified_name == "SysML2Legacy::Good"),
        "good file's package should still be ingested: {:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
    // Nothing at all came from the broken file.
    assert!(!elements.iter().any(|e| e.qualified_name.contains("Broken")));

    let result = validate(&elements);
    let w541: Vec<_> = result.findings.iter().filter(|f| f.code == "W541").collect();
    assert_eq!(w541.len(), 1, "expected exactly one W541 for the broken file, got: {w541:#?}");
    assert!(w541[0].file.contains("Broken.sysml"));
    // A parse failure is a warning, never an error — never aborts the rest of validate.
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

//! Cross-reference linking between a marker-synthesized element and the
//! native Syscribe model (`ADR-SYS-ANNOTATE-001`).
//!
//! Generic cross-references (`satisfies:`, `derivedFrom:`, `allocatedTo:`,
//! `typedBy:`, `supertype:`, …) are resolved purely by qname/id lookup with
//! no origin gating, so they already worked bidirectionally before any of
//! this — these tests document that, rather than testing new code.
//! `verifies:` is the one exception: `E104` hard-gates its target's
//! legality, and this file's real subject is the widening that lets it
//! target an annotation-synthesized element too, via
//! `crate::annotations::synthesized_qnames` / `Resolver::is_verify_target`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-annotated-source-links-test-{}-{}",
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

fn firmware_package(root: &Path) {
    write(root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        root,
        "Firmware/_index.md",
        "---\ntype: Package\nname: Firmware\nannotationFormat: c-linecomment\nmarker: '//\\s*@syscribe\\b'\ninclude: [\"**/*.c\"]\n---\n",
    );
}

#[test]
fn marker_element_satisfies_a_native_requirement() {
    let root = tempdir();
    firmware_package(&root);
    write(
        &root,
        "Requirements/REQ-ENGCTL-100.md",
        "---\ntype: Requirement\nid: REQ-ENGCTL-100\nname: \"Engine controller shall hold target RPM\"\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\nThe engine controller shall hold a commanded target RPM.\n",
    );
    write(
        &root,
        "Firmware/engine.c",
        "// @syscribe\n// type: Part\n// name: EngineController\n// domain: software\n// satisfies: [REQ-ENGCTL-100]\n// doc: x\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
    // The marker-emitted `satisfies:` should count as the requirement's
    // satisfying element — no W300 "no satisfying element" leaf warning.
    assert!(
        !result.findings.iter().any(|f| f.code == "W300"),
        "requirement should be considered satisfied: {:#?}",
        result.findings
    );
}

#[test]
fn native_testcase_verifies_a_marker_synthesized_part() {
    let root = tempdir();
    firmware_package(&root);
    write(
        &root,
        "Firmware/engine.c",
        "// @syscribe\n// type: Part\n// name: EngineController\n// doc: x\n",
    );
    write(
        &root,
        "Tests/TC-ENGCTL-001.md",
        "---\ntype: TestCase\nid: TC-ENGCTL-001\nname: \"Engine controller test\"\ntestLevel: L1\nstatus: draft\nverifies:\n  - Firmware::EngineController\n---\n\n```gherkin\nFeature: Engine control\n  Scenario: it works\n    Given an engine controller\n    When it runs\n    Then it holds target RPM\n```\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let e104: Vec<_> = result.findings.iter().filter(|f| f.code == "E104").collect();
    assert!(e104.is_empty(), "E104 should not fire for an annotation-synthesized target: {:#?}", result.findings);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn native_testcase_verifying_an_ordinary_hand_authored_part_still_raises_e104() {
    // Regression guard: the widening must stay scoped to *actually*
    // annotation-synthesized elements, never to a hand-authored element of
    // the same kind living outside any `annotationFormat:` package.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Arch/EngineController.md",
        "---\ntype: PartDef\nname: EngineController\n---\n\nHand-authored, not annotation-synthesized.\n",
    );
    write(
        &root,
        "Tests/TC-ENGCTL-002.md",
        "---\ntype: TestCase\nid: TC-ENGCTL-002\nname: \"Engine controller test\"\ntestLevel: L1\nstatus: draft\nverifies:\n  - Arch::EngineController\n---\n\n```gherkin\nFeature: Engine control\n  Scenario: it works\n    Given an engine controller\n    When it runs\n    Then it holds target RPM\n```\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let e104: Vec<_> = result.findings.iter().filter(|f| f.code == "E104").collect();
    assert_eq!(e104.len(), 1, "hand-authored PartDef must not become a legal verify target: {:#?}", result.findings);
}

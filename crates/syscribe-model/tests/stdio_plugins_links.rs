//! Cross-reference linking between a stdio plugin's foreign-format subtree
//! and the native Syscribe model (`ADR-SYS-PLUGIN-002`).
//!
//! Generic cross-references (`satisfies:`, `derivedFrom:`, `allocatedTo:`,
//! `typedBy:`, `supertype:`, …) are resolved purely by qname/id lookup with
//! no origin gating, so they already worked bidirectionally before any of
//! this — these tests document that, rather than testing new code.
//! `verifies:` is the one exception: `E104` hard-gates its target's
//! legality, and until now that gate only widened for SysMLv2-synthesized
//! elements (`REQ-TRS-SYSMLV2-004`). This file's real subject is extending
//! that widening to plugin-synthesized elements too, via
//! `crate::plugins::synthesized_qnames` / `Resolver::is_verify_target`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-stdio-plugins-links-test-{}-{}",
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

fn toy_plugin_config(envelope_json: &str) -> String {
    format!(
        "[plugins.toydsl]\ncommand = \"/bin/sh\"\nargs = [\"-c\", \"cat >/dev/null; echo '{}'\"]\n",
        envelope_json.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

#[test]
fn plugin_element_satisfies_a_native_requirement() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Requirements/REQ-TOY-100.md",
        "---\ntype: Requirement\nid: REQ-TOY-100\nname: \"Coolant flow shall be regulated\"\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\nThe system shall regulate coolant flow.\n",
    );
    write(
        &root,
        "Legacy/_index.md",
        "---\ntype: Package\nname: Legacy\nforeignFormat: toydsl\n---\n",
    );
    write(
        &root,
        ".syscribe.toml",
        &toy_plugin_config(r#"{"elements":[{"qname":"FlowController","type":"Part","domain":"software","doc":"x","satisfies":["REQ-TOY-100"]}]}"#),
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
    // The plugin-emitted `satisfies:` should count as the requirement's
    // satisfying element — no W300 "no satisfying element" leaf warning.
    assert!(
        !result.findings.iter().any(|f| f.code == "W300"),
        "requirement should be considered satisfied: {:#?}",
        result.findings
    );
}

#[test]
fn native_testcase_verifies_a_plugin_emitted_partdef() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Legacy/_index.md",
        "---\ntype: Package\nname: Legacy\nforeignFormat: toydsl\n---\n",
    );
    write(
        &root,
        ".syscribe.toml",
        &toy_plugin_config(r#"{"elements":[{"qname":"FlowController","type":"PartDef","doc":"x"}]}"#),
    );
    write(
        &root,
        "Tests/TC-TOY-001.md",
        "---\ntype: TestCase\nid: TC-TOY-001\nname: \"Flow controller test\"\ntestLevel: L1\nstatus: draft\nverifies:\n  - Legacy::FlowController\n---\n\n```gherkin\nFeature: Flow control\n  Scenario: it works\n    Given a flow controller\n    When it runs\n    Then it regulates flow\n```\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let e104: Vec<_> = result.findings.iter().filter(|f| f.code == "E104").collect();
    assert!(e104.is_empty(), "E104 should not fire for a plugin-synthesized target: {:#?}", result.findings);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
}

#[test]
fn native_testcase_verifying_an_ordinary_hand_authored_partdef_still_raises_e104() {
    // Regression guard: the widening must stay scoped to *actually*
    // plugin-synthesized elements, never to a hand-authored element of the
    // same kind living outside any `foreignFormat:` package.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Arch/FlowController.md",
        "---\ntype: PartDef\nname: FlowController\n---\n\nHand-authored, not plugin-synthesized.\n",
    );
    write(
        &root,
        "Tests/TC-TOY-002.md",
        "---\ntype: TestCase\nid: TC-TOY-002\nname: \"Flow controller test\"\ntestLevel: L1\nstatus: draft\nverifies:\n  - Arch::FlowController\n---\n\n```gherkin\nFeature: Flow control\n  Scenario: it works\n    Given a flow controller\n    When it runs\n    Then it regulates flow\n```\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let e104: Vec<_> = result.findings.iter().filter(|f| f.code == "E104").collect();
    assert_eq!(e104.len(), 1, "hand-authored PartDef must not become a legal verify target: {:#?}", result.findings);
}

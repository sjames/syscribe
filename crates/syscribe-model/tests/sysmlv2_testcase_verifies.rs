//! Integration tests for a native `TestCase`'s `verifies:` field targeting a
//! SysMLv2-originated element (`REQ-TRS-SYSMLV2-004`).
//!
//! Investigation finding (see this task's report): `TestCase.verifies:`
//! resolution itself (`Resolver::resolve_ref`) already did a generic
//! id-or-qname lookup across *all* elements regardless of origin — a SysMLv2
//! element's qname was already reachable with zero changes. The gap was
//! `E104`, which unconditionally required the *resolved target* to be a
//! native Requirement (`Resolver::is_native_requirement`), rejecting any
//! SysMLv2-mapped target outright, and the `verifiedBy` reverse index, which
//! only ever recorded an entry when the target had a stable `id` (a SysMLv2
//! element never does). Both are fixed in `resolver.rs`/`validator.rs`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-tc-verifies-test-{}-{}",
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

fn gherkin_body() -> &'static str {
    "```gherkin\nGiven a precondition\nWhen an action occurs\nThen an outcome is observed\n```\n"
}

#[test]
fn testcase_verifies_a_top_level_sysmlv2_part_usage() {
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
         part droneInstance : Drone;\n\
         }\n",
    );
    write(
        &root,
        "Tests/TC-SCHED-001.md",
        &format!(
            "---\ntype: TestCase\nid: TC-SCHED-001\nname: Drone instance check\nstatus: active\ntestLevel: L2\nverifies: [SysML2Legacy::Vehicle::droneInstance]\n---\n{}",
            gherkin_body()
        ),
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    // No E102 (unresolved) / E104 (wrong target type) — the qname resolved,
    // and the widened E104 check accepts a mapped SysMLv2 Part usage.
    assert!(
        !result.findings.iter().any(|f| f.code == "E102" || f.code == "E104"),
        "unexpected dangling/wrong-type finding: {:#?}",
        result.findings
    );

    // The verifiedBy reverse index includes the SysMLv2 element's qname as a
    // key (it has no stable id), with the TestCase's id as the entry.
    let verifiers = result
        .verified_by
        .get("SysML2Legacy::Vehicle::droneInstance")
        .unwrap_or_else(|| panic!("expected a verifiedBy entry, got: {:#?}", result.verified_by));
    assert_eq!(verifiers, &vec!["TC-SCHED-001".to_string()]);
}

#[test]
fn testcase_verifies_a_deeply_nested_sysmlv2_element() {
    // Same proof, but against an element several qname segments deep — a
    // nested package containing a part def containing a nested part usage —
    // matching REQ-TRS-SYSMLV2-002's qname derivation, to rule out this only
    // working for a trivial top-level case.
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
        "package Outer {\n\
         package Inner {\n\
         part def Drone {\n\
         part fuelPort : FuelPort;\n\
         }\n\
         }\n\
         }\n",
    );
    write(
        &root,
        "Tests/TC-SCHED-002.md",
        &format!(
            "---\ntype: TestCase\nid: TC-SCHED-002\nname: Fuel port check\nstatus: active\ntestLevel: L2\nverifies: [SysML2Legacy::Outer::Inner::Drone::fuelPort]\n---\n{}",
            gherkin_body()
        ),
    );

    let elements = walk_model(&root).unwrap();

    // Sanity: the deeply-nested element really exists at that qname.
    assert!(elements
        .iter()
        .any(|e| e.qualified_name == "SysML2Legacy::Outer::Inner::Drone::fuelPort"));

    let result = validate(&elements);
    assert!(
        !result.findings.iter().any(|f| f.code == "E102" || f.code == "E104"),
        "unexpected dangling/wrong-type finding: {:#?}",
        result.findings
    );

    let verifiers = result
        .verified_by
        .get("SysML2Legacy::Outer::Inner::Drone::fuelPort")
        .unwrap_or_else(|| panic!("expected a verifiedBy entry, got: {:#?}", result.verified_by));
    assert_eq!(verifiers, &vec!["TC-SCHED-002".to_string()]);
}

#[test]
fn e104_still_fires_for_a_target_that_resolves_but_is_not_a_recognized_verify_target() {
    // The widening is by element *kind*, not blanket-permissive: a target
    // that resolves to an ordinary Package (not in the widened set, and not
    // a native Requirement) still fails E104 exactly as before.
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
         }\n",
    );
    write(
        &root,
        "Tests/TC-SCHED-003.md",
        &format!(
            "---\ntype: TestCase\nid: TC-SCHED-003\nname: Bad target\nstatus: active\ntestLevel: L2\nverifies: [SysML2Legacy::Vehicle]\n---\n{}",
            gherkin_body()
        ),
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    let e104: Vec<_> = result.findings.iter().filter(|f| f.code == "E104").collect();
    assert_eq!(e104.len(), 1, "expected exactly one E104, got: {:#?}", result.findings);
}

#[test]
fn testcase_still_verifies_a_native_requirement_unchanged() {
    // Regression guard: the original, unchanged rule (native Requirement
    // target) still works exactly as before.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Requirements/REQ-SCHED-001.md",
        "---\ntype: Requirement\nid: REQ-SCHED-001\nname: Sched req\nstatus: approved\nreqDomain: software\n---\nThe system shall schedule things.\n",
    );
    write(
        &root,
        "Tests/TC-SCHED-004.md",
        &format!(
            "---\ntype: TestCase\nid: TC-SCHED-004\nname: Native req check\nstatus: active\ntestLevel: L2\nverifies: [REQ-SCHED-001]\n---\n{}",
            gherkin_body()
        ),
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    assert!(
        !result.findings.iter().any(|f| f.code == "E102" || f.code == "E104"),
        "unexpected dangling/wrong-type finding: {:#?}",
        result.findings
    );
    assert_eq!(
        result.verified_by.get("REQ-SCHED-001"),
        Some(&vec!["TC-SCHED-004".to_string()])
    );
}

// ── Review-finding regressions ──────────────────────────────────────────────
// A first version of this widening was reviewed and found to have real blast
// radius beyond SysMLv2 (see the task report for full detail). These three
// tests each lock in the specific fix for one finding.

#[test]
fn e104_still_rejects_a_hand_authored_native_part_target() {
    // Finding #1 (SEVERE): the widened E104 branch must be gated on actual
    // SysMLv2 origin (`sysmlv2_qnames`), not element kind alone. A plain
    // hand-authored model with zero SysMLv2 involvement, `verifies:`
    // targeting an ordinary native `PartDef`, must still fail E104 exactly as
    // it did before this feature ever existed.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Arch/Engine.md",
        "---\ntype: PartDef\nname: Engine\n---\nAn engine.\n",
    );
    write(
        &root,
        "Tests/TC-ARCH-001.md",
        &format!(
            "---\ntype: TestCase\nid: TC-ARCH-001\nname: Bad verifies target\nstatus: active\ntestLevel: L2\nverifies: [Arch::Engine]\n---\n{}",
            gherkin_body()
        ),
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    let e104: Vec<_> = result.findings.iter().filter(|f| f.code == "E104").collect();
    assert_eq!(
        e104.len(),
        1,
        "a hand-authored native PartDef target must still fail E104: {:#?}",
        result.findings
    );
    assert!(
        !result.verified_by.contains_key("Arch::Engine"),
        "a rejected target must not appear in verified_by: {:#?}",
        result.verified_by
    );
}

#[test]
fn testcase_verifies_a_sysmlv2_requirement_def_target() {
    // Finding #2 (MEDIUM): REQ-TRS-SYSMLV2-007's fixed set includes
    // Requirement(Def/Usage), but a SysMLv2-synthesized one never has an id,
    // so it can never pass `is_native_requirement`. It must still be a legal
    // verifies: target via the provenance-gated widened set.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Safety.sysml",
        "package Safety {\n\
         requirement def SafetyReq;\n\
         }\n",
    );
    write(
        &root,
        "Tests/TC-SAFE-001.md",
        &format!(
            "---\ntype: TestCase\nid: TC-SAFE-001\nname: Safety req check\nstatus: active\ntestLevel: L2\nverifies: [SysML2Legacy::Safety::SafetyReq]\n---\n{}",
            gherkin_body()
        ),
    );

    let elements = walk_model(&root).unwrap();
    assert!(elements
        .iter()
        .any(|e| e.qualified_name == "SysML2Legacy::Safety::SafetyReq"));

    let result = validate(&elements);
    assert!(
        !result.findings.iter().any(|f| f.code == "E102" || f.code == "E104"),
        "unexpected dangling/wrong-type finding: {:#?}",
        result.findings
    );
    assert_eq!(
        result.verified_by.get("SysML2Legacy::Safety::SafetyReq"),
        Some(&vec!["TC-SAFE-001".to_string()])
    );
}

#[test]
fn testcase_verifies_a_sysmlv2_requirement_usage_target() {
    // Same as above, the bare (no `def`) requirement-usage form.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Safety.sysml",
        "package Safety {\n\
         requirement checkSafety : SafetyReqType;\n\
         }\n",
    );
    write(
        &root,
        "Tests/TC-SAFE-002.md",
        &format!(
            "---\ntype: TestCase\nid: TC-SAFE-002\nname: Safety usage check\nstatus: active\ntestLevel: L2\nverifies: [SysML2Legacy::Safety::checkSafety]\n---\n{}",
            gherkin_body()
        ),
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    assert!(
        !result.findings.iter().any(|f| f.code == "E102" || f.code == "E104"),
        "unexpected dangling/wrong-type finding: {:#?}",
        result.findings
    );
    assert_eq!(
        result.verified_by.get("SysML2Legacy::Safety::checkSafety"),
        Some(&vec!["TC-SAFE-002".to_string()])
    );
}

#[test]
fn sysmlv2_elements_own_verify_does_not_pollute_verified_by() {
    // Finding #3 (HIGH): REQ-TRS-SYSMLV2-003's SysMLv2 element with its own
    // `verify:` (e.g. `requirement checkReq { verify 'REQ-X'; }`) must NOT
    // appear in `verified_by` — only an element that itself carries a stable
    // id (in practice, always a native TestCase) is recorded there. Before
    // this fix, the id-else-qname fallback on the *source* side let this
    // SysMLv2 element's own qname in, polluting every `verified_by` consumer
    // (the web server's /api/validation, `export`/`export-html`, `query`,
    // the LSP CodeLens count, the Rhai property, and the MCP `evidence` tool).
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Requirements/REQ-POLLUTE-001.md",
        "---\ntype: Requirement\nid: REQ-POLLUTE-001\nname: Pollute req\nstatus: approved\nreqDomain: software\n---\nThe system shall not be polluted.\n",
    );
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Checks.sysml",
        "package Checks {\n\
         requirement checkReq {\n\
         verify 'REQ-POLLUTE-001';\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    // Sanity: the SysMLv2 element really does carry the verify: reference.
    let checker = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Checks::checkReq")
        .unwrap();
    assert_eq!(checker.frontmatter.verifies, Some(vec!["REQ-POLLUTE-001".to_string()]));

    let result = validate(&elements);

    let verifiers = result.verified_by.get("REQ-POLLUTE-001");
    assert!(
        verifiers.is_none_or(|v| v.is_empty()),
        "the SysMLv2 element's own verify: must not pollute verified_by: {:#?}",
        result.verified_by
    );

    // Broader net: no value anywhere in verified_by is qname-shaped (contains
    // "::") — every recorded entry must be a real stable id, never a qname
    // fallback for a source lacking one.
    for (key, sources) in &result.verified_by {
        for s in sources {
            assert!(
                !s.contains("::"),
                "verified_by[{key}] contains a qname-shaped (non-id) source '{s}': {:#?}",
                result.verified_by
            );
        }
    }
}

//! Integration tests for a SysMLv2 element's native `satisfy`/`verify`
//! relationship targeting a native Syscribe `Requirement` (`REQ-TRS-SYSMLV2-003`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-trace-test-{}-{}",
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
fn satisfy_by_quoted_req_id_resolves_and_suppresses_w300() {
    // The ADR's own example form: `satisfy 'REQ-SCHED-001';` — SysML v2's
    // quoted-name syntax for a hyphenated identifier.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Requirements/REQ-SCHED-001.md",
        "---\ntype: Requirement\nid: REQ-SCHED-001\nname: Sched req\nstatus: approved\nreqDomain: software\n---\nThe system shall schedule things.\n",
    );
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Vehicle.sysml",
        "package Vehicle {\n\
         part droneInstance : Drone {\n\
         satisfy 'REQ-SCHED-001';\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();

    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Vehicle::droneInstance")
        .unwrap_or_else(|| {
            panic!(
                "expected droneInstance, got: {:#?}",
                elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
            )
        });
    // The mapper carries the target string verbatim — quotes already stripped
    // by the parser's own lexer, exactly as a hand-authored `satisfies:` would
    // be written.
    assert_eq!(part.frontmatter.satisfies, Some(vec!["REQ-SCHED-001".to_string()]));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
    // No dangling-reference finding for the satisfy target: resolution used
    // the existing id-or-qname resolver unchanged, and it found REQ-SCHED-001.
    assert!(
        !result.findings.iter().any(|f| f.message.contains("REQ-SCHED-001") && f.code.starts_with('E')),
        "unexpected error referencing the target requirement: {:#?}",
        result.findings
    );
    // W300 ("leaf Requirement has no satisfying architecture element") is
    // suppressed — proof the qname-keyed `satisfied_reqs` reverse index used
    // internally by the validator picked up the SysMLv2 element's qname as a
    // satisfier of REQ-SCHED-001.
    assert!(
        !result.findings.iter().any(|f| f.code == "W300"),
        "W300 should be suppressed once satisfied by the SysMLv2 element: {:#?}",
        result.findings
    );
}

#[test]
fn satisfy_by_syscribe_qualified_name_resolves() {
    // REQ-TRS-SYSMLV2-003: "...or by its Syscribe qualified name." A
    // Requirement is id-identified and file-named after its id
    // (`Requirements/REQ-QNAME-001.md`), so its *qualified* name is
    // `Requirements::REQ-QNAME-001` — distinct from the bare id form
    // exercised above (that string resolves via the qname index, not the
    // id index: `Requirements::REQ-QNAME-001` doesn't itself match the
    // `REQ-*` id pattern). The hyphenated last segment still needs SysML v2's
    // quoted-name syntax even inside a qualified name — each `::`-segment is
    // independently either a bare identifier or a quoted string.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Requirements/REQ-QNAME-001.md",
        "---\ntype: Requirement\nid: REQ-QNAME-001\nname: Qname req\nstatus: approved\nreqDomain: software\n---\nThe system shall do the thing.\n",
    );
    write(
        &root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
    write(
        &root,
        "SysML2Legacy/Vehicle.sysml",
        "package Vehicle {\n\
         part droneInstance : Drone {\n\
         satisfy Requirements::'REQ-QNAME-001';\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Vehicle::droneInstance")
        .unwrap();
    assert_eq!(
        part.frontmatter.satisfies,
        Some(vec!["Requirements::REQ-QNAME-001".to_string()])
    );

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "unexpected errors: {:#?}", result.findings);
    assert!(
        !result.findings.iter().any(|f| f.code == "W300"),
        "W300 should be suppressed — the qname-form satisfy target resolved: {:#?}",
        result.findings
    );
}

#[test]
fn verify_by_quoted_req_id_resolves_with_no_dangling_finding() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &root,
        "Requirements/REQ-VERIFY-001.md",
        "---\ntype: Requirement\nid: REQ-VERIFY-001\nname: Verify req\nstatus: approved\nreqDomain: software\n---\nThe system shall be verifiable.\n",
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
         verify 'REQ-VERIFY-001';\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();

    let req = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Checks::checkReq")
        .unwrap_or_else(|| {
            panic!(
                "expected checkReq, got: {:#?}",
                elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
            )
        });
    assert_eq!(req.frontmatter.verifies, Some(vec!["REQ-VERIFY-001".to_string()]));

    let result = validate(&elements);
    // E102 (unresolved verifies reference) / E104 (target not a native
    // Requirement) must both be absent — the reference resolved, and to the
    // right kind of element.
    assert!(
        !result.findings.iter().any(|f| f.code == "E102" || f.code == "E104"),
        "unexpected dangling/wrong-type finding: {:#?}",
        result.findings
    );
}

#[test]
fn dangling_verify_target_raises_the_normal_e102_finding() {
    // Scope item 3: a verify target that resolves to neither a real id nor a
    // real qname raises the same E102 this codebase already raises for any
    // other unresolved `verifies:` — confirmed to be the very same code path
    // (`validator.rs`'s generic `fm.verifies` cross-reference check), not a
    // new SysMLv2-specific diagnostic.
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
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
         verify 'REQ-DOES-NOT-EXIST-001';\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);

    let e102: Vec<_> = result.findings.iter().filter(|f| f.code == "E102").collect();
    assert_eq!(e102.len(), 1, "expected exactly one E102, got: {:#?}", result.findings);
    assert!(e102[0].message.contains("REQ-DOES-NOT-EXIST-001"));
}

#[test]
fn dangling_satisfy_target_is_carried_but_currently_raises_no_finding() {
    // Scope item 3, satisfy side: unlike `verifies:` (E102 above),
    // `validator.rs` does not currently raise ANY finding for a `satisfies:`
    // target that resolves to nothing, outside multi-repo mode (`E512`, which
    // requires `[repos]` to be configured at all) — confirmed by reading every
    // site that reads `fm.satisfies` in `validator.rs`: each only acts in the
    // `Some(target)` (resolved) branch, with no `else` for the unresolved
    // case. This is a pre-existing characteristic of the general Requirement-
    // traceability validation, not something this task's SysMLv2 mapping
    // introduces or is in scope to change (REQ-TRS-SYSMLV2-003 only requires
    // carrying the target verbatim through the existing, unmodified
    // resolver). This test locks in that observed behavior so a future change
    // to either side is a deliberate, visible diff here.
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
         part droneInstance : Drone {\n\
         satisfy 'REQ-DOES-NOT-EXIST-001';\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let part = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Vehicle::droneInstance")
        .unwrap();
    assert_eq!(
        part.frontmatter.satisfies,
        Some(vec!["REQ-DOES-NOT-EXIST-001".to_string()])
    );

    let result = validate(&elements);
    assert_eq!(
        result.errors().count(),
        0,
        "no error is currently raised for a dangling satisfies target outside multi-repo mode: {:#?}",
        result.findings
    );
}

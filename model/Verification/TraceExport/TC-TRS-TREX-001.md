---
id: TC-TRS-TREX-001
type: TestCase
testLevel: L3
status: active
name: "Verify the trace document carries the normative fields in order, full qualified names with ids on every reference, the W300/W002/W305 coverage block and summary, the unresolved form for a dangling reference, and sidecar verdicts."
verifies:
  - REQ-TRS-TREX-001
sourceFile: repo:crates/syscribe/tests/trace_export.rs
testFunctions:
  - document_has_the_normative_shape_and_field_order
  - every_reference_is_a_full_qualified_name_with_id
  - coverage_mirrors_w300_w002_w305_and_summary_counts
  - a_dangling_reference_is_kept_as_unresolved
  - verdicts_come_from_the_results_sidecar_and_are_null_without_one
tags:
  - traceability
  - export
  - cli
---

Black-box CLI tests in `crates/syscribe/tests/trace_export.rs`, run with
`cargo test -p syscribe --test trace_export`, spawning the built binary against a model written
to a temp directory: `Requirements::Core::REQ-TX-000` (parent, verified by the active L4
`TC-TX-000`, refined by `UseCases::Login`), `REQ-TX-001` (derived, satisfied by the PartDef
`Arch::Alpha`), `REQ-TX-002` (derived, verified by the active L2 `TC-TX-002` with a passing
`testFunctions` entry and the retired `TC-TX-003`), `REQ-TX-003` (`derivedFrom: [REQ-TX-999]`,
dangling) and a `.syscribe/results.json` sidecar.

```gherkin
Feature: the trace document (TC-TRS-TREX-001)

  Scenario: normative shape and field order
    When the tool runs trace-export
    Then the document has version 1, modelRoot, config null, sort directory and a summary
    And the top-level, entry, coverage and summary keys appear in the specified order

  Scenario: full qualified names on every reference
    When the tool runs trace-export
    Then derivedFrom, derivedChildren, breakdownAdr, satisfiedBy, verifiedBy and refinedBy
         name their targets by full qualified name with id, status, type/domain or testLevel

  Scenario: coverage and summary
    When the tool runs trace-export
    Then the parent is not a leaf, verified and integration-verified
    And the satisfied child is a satisfied leaf, the verified child a verified leaf
    And the summary reads 4 requirements, 3 leaves, 1 satisfied, 2 verified, 1 integration-verified

  Scenario: a dangling reference
    When the tool runs trace-export
    Then REQ-TX-003's derivedFrom entry is {qname: "REQ-TX-999", unresolved: true}

  Scenario: verdicts
    Given a results sidecar recording tx_002 as passed
    When the tool runs trace-export
    Then TC-TX-002's verdict is pass and the retired TC-TX-003's is unknown
    And without the sidecar every verdict is null
```

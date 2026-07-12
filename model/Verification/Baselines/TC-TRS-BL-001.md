---
type: TestCase
id: TC-TRS-BL-001
name: "Baseline is a recognized element type with the BL-* id scheme and schema"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/baseline.rs
tags:
  - baseline
  - schema
verifies:
  - REQ-TRS-BL-001
---

Verifies that `type: Baseline` is a first-class, recognized element type keyed by a stable
`BL-*` id, with its documented schema fields bound (not swallowed as unknown).

```gherkin
Feature: Baseline element type

  Scenario: A Baseline element validates and is not unknown
    Given a model containing a `type: Baseline` element with id BL-2026-07 and gitTag REL-2026-07
    When the model is validated
    Then the element raises no unknown-type or unrecognized-field diagnostic

  Scenario: The BL-* id scheme matches the FEAT-style relaxed pattern
    Given ids BL-2026-07 and BL-QUARTERLY-001
    Then both satisfy ^BL(-[A-Z0-9]{2,12})+$ (no forced numeric suffix)

  Scenario: A malformed BL id is rejected
    Given a Baseline whose id does not match the BL-* pattern
    When the model is validated
    Then an id-pattern error is reported

  Scenario: id is distinct from the git tag
    Given a Baseline with id BL-2026-07 and gitTag REL-2026-07
    Then the element identity is BL-2026-07 and the free-form tag REL-2026-07 is not the id

  Scenario: Schema fields are bound, and frozenScope does not collide with scope
    Given a Baseline declaring gitTag, gitCommit, approver, frozenScope, seal, and supersedes
    When the element is read back
    Then each field is parsed into its typed slot, not the unknown catch-all
    And the structured frozenScope object does not conflict with the free-form `scope` field
```

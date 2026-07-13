---
type: TestCase
id: TC-TRS-BL-007
name: "baseline list and show are read-only and print provenance"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/baseline.rs
tags:
  - baseline
  - cli
verifies:
  - REQ-TRS-BL-007
---

Verifies the read-only inventory subcommands: `list` enumerates baselines deterministically,
`show` prints full provenance, and neither mutates the model.

```gherkin
Feature: baseline list / show

  Scenario: list enumerates baselines deterministically
    Given several Baseline elements
    When `baseline list` is run twice
    Then the same ordered set of baselines (id, name, status, date, scope) is printed

  Scenario: show prints provenance
    When `baseline show BL-2026-07` is run
    Then name, date, approver, gitTag, gitCommit, frozenScope, elementCount, aggregateHash,
      supersedes, and the manifest path are printed

  Scenario: The read commands are read-only
    When list and show are run
    Then no model file is modified
```

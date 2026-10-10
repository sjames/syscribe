---
type: TestCase
id: TC-TRS-PLEREF-001
name: "references to gated elements project to W019 and allocations inherit endpoint gates"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/ple_reference_projection.rs
verifies:
  - REQ-TRS-PLEREF-001
tags:
  - variability
---

```gherkin
Feature: projection of references to gated elements

  Scenario: list references
    Then validate --config on a variant that drops the target reports W019 and no E716, E704, E851, E603

  Scenario: allocation gate
    Then an ungated Allocation to a gated part is dropped from that variant without an escape finding

  Scenario: plain validate is unchanged
    Then a reference that resolves nowhere is still E716 in plain validate
```

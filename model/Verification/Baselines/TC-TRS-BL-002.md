---
type: TestCase
id: TC-TRS-BL-002
name: "Full-content seal is byte-exact and deterministically aggregated"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/baseline.rs
tags:
  - baseline
  - hashing
verifies:
  - REQ-TRS-BL-002
---

Verifies that the seal hashes full canonical element content (editorial included), that any
in-scope edit changes it, and that the aggregate is deterministic and order-independent.

```gherkin
Feature: Baseline content seal

  Scenario: The aggregate is reproducible
    Given a sealed baseline over a fixed scope
    When the aggregate hash is recomputed from the unchanged model
    Then it equals the stored seal.aggregateHash

  Scenario: An editorial edit changes the seal
    Given a sealed baseline
    When an in-scope element's editorial field (e.g. name or displayOrder) is changed
    Then the recomputed aggregate hash differs from the seal
    # (contrast with suspect-links, where editorial edits are excluded)

  Scenario: A normative edit changes the seal
    Given a sealed baseline
    When an in-scope element's body or status is changed
    Then the recomputed aggregate hash differs from the seal

  Scenario: Aggregation is order-independent
    Given two models with the same elements in different file/enumeration order
    Then their aggregate hashes over the same scope are identical
```

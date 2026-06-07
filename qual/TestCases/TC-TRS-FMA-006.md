---
id: TC-TRS-FMA-006
type: TestCase
testLevel: L3
status: draft
title: "Verify determinism, the size guard, and the Boolean-only scope statement."
verifies:
  - REQ-TRS-FMA-006
---

```gherkin
Feature: Decision procedure obligations
  Scenario: determinism
    Given a feature model
    When feature-check --deep --json runs twice
    Then the two outputs are identical
  Scenario: size guard
    Given a feature model exceeding the size limit
    Then feature-check --deep prints a skip diagnostic and exits 0
  Scenario: scope statement
    Given a feature model
    When feature-check --deep runs
    Then the output states the analysis covers the Boolean layer only
```

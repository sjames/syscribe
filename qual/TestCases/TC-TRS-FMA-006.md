---
id: TC-TRS-FMA-006
type: TestCase
testLevel: L3
status: draft
title: "Verify determinism, ~500-feature scale, the size guard, and the Boolean-only scope statement."
verifies:
  - REQ-TRS-FMA-006
---

```gherkin
Feature: Decision procedure obligations
  Scenario: determinism
    Given a feature model
    When feature-check --deep --json runs twice
    Then the two outputs are identical
  Scenario: ~500-feature scale is analyzed, not skipped
    Given a feature model of about 500 features
    When feature-check --deep runs
    Then it completes within interactive time, is not skipped, and is correct
  Scenario: size guard above the limit
    Given a feature model exceeding the documented feature-count limit (1000)
    When feature-check --deep runs
    Then it prints a skip diagnostic and exits 0
  Scenario: scope statement
    Given a feature model
    When feature-check --deep runs
    Then the output states the analysis covers the Boolean layer only
```

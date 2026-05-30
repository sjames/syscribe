---
id: TC-TRS-VAL-009
type: TestCase
testLevel: L3
status: draft
title: "Verify that E500-E503, W500-W502, and W600-W601 are emitted for Allocation, View, and documentation violations."
verifies:
  - REQ-TRS-VAL-009
---

Verify that Allocation cross-reference errors, View cross-reference warnings, and documentation completeness warnings are emitted correctly.

```gherkin
Feature: Allocation, View, and documentation validation

  Scenario Outline: Each code is emitted for its triggering condition
    Given a model fixture for <code>
    When the tool validates the model
    Then a finding with code <code> is present in the output

    Examples:
      | code |
      | E500 |
      | E501 |
      | E502 |
      | E503 |
      | W500 |
      | W501 |
      | W502 |
      | W600 |
      | W601 |
```

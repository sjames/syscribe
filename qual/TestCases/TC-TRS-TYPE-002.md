---
id: TC-TRS-TYPE-002
type: TestCase
testLevel: L3
status: draft
title: "Verify CalculationDef and Calculation are recognised and validated without E005."
verifies:
  - REQ-TRS-TYPE-002
---

Verify CalculationDef and Calculation are recognised and validated without E005.

```gherkin
Feature: Element type recognition — CalculationDef, Calculation

  Scenario: Declared types are recognised and validate clean
    Given a minimal valid model containing one element of each of: CalculationDef, Calculation
    When the tool validates the model
    Then no E005 finding is emitted
    And the validation exits with code 0

  Scenario: Each element is parsed at its declared type
    Given the same minimal valid model
    When the model is exported
    Then each element appears at its declared type

  Scenario: A sibling file with an unrecognised type still bites
    Given a sibling file whose type: value is not in the inventory
    When the tool validates the model
    Then an E005 finding is emitted
    And the validation exits non-zero
```

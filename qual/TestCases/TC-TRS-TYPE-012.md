---
id: TC-TRS-TYPE-012
type: TestCase
testLevel: L3
status: draft
title: "Verify State and ExhibitState are recognised and validated without E005."
verifies:
  - REQ-TRS-TYPE-012
---

Verify State and ExhibitState are recognised and validated without E005.

```gherkin
Feature: Element type recognition — State, ExhibitState

  Scenario: Declared types are recognised and validate clean
    Given a minimal valid model containing one element of each of: State, ExhibitState
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

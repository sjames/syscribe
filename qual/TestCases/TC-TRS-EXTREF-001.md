---
id: TC-TRS-EXTREF-001
type: TestCase
testLevel: L3
status: draft
name: "Verify the extRef common field parses (string or list) and duplicate detection W028."
verifies:
  - REQ-TRS-EXTREF-001
---

Verify that `extRef` is accepted on any element as a single string or a list, that a reference shared by two elements produces `W028`, that unique or absent references produce none, and that `W028` is gateable.

```gherkin
Feature: extRef field and duplicate detection

  Scenario: single-string and list-valued extRef parse without error
    Given elements declaring extRef as a string and as a list, all unique
    When the tool validates the model
    Then no W028 finding is emitted
    And validation does not error on the extRef field

  Scenario: the same extRef on two elements produces W028
    Given two elements that declare the same extRef value
    When the tool validates the model
    Then a W028 finding is emitted

  Scenario: a model with no extRef produces no W028
    Given elements that declare no extRef
    When the tool validates the model
    Then no W028 finding is emitted

  Scenario: W028 is gateable with --deny
    Given a model with a duplicate extRef
    When the tool validates with --deny W028
    Then the tool exits non-zero
```

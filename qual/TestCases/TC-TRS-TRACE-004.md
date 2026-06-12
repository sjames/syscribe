---
id: TC-TRS-TRACE-004
type: TestCase
testLevel: L3
status: draft
name: "Verify that W300 is emitted for an approved leaf Requirement with no satisfying element."
verifies:
  - REQ-TRS-TRACE-004
---

Verify that W300 is emitted for an approved leaf Requirement with no satisfying element.

```gherkin
Feature: Unassigned leaf requirement warning

  Scenario: Approved leaf Requirement with no satisfies: produces W300
    Given a leaf Requirement (no derivedChildren) with status: approved
    And no element in the model has satisfies: pointing to that Requirement
    When the tool is invoked
    Then a W300 finding is emitted for that Requirement

  Scenario: Approved leaf Requirement with one satisfies: does not produce W300
    Given a leaf Requirement with status: approved
    And an architecture element with satisfies: [REQ-LEAF-001]
    When the tool is invoked
    Then no W300 finding is emitted for REQ-LEAF-001

  Scenario: Draft leaf Requirement with no satisfies: does not produce W300
    Given a leaf Requirement with status: draft
    And no satisfying element
    When the tool is invoked
    Then no W300 finding is emitted
```

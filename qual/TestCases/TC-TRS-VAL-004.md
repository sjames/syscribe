---
id: TC-TRS-VAL-004
type: TestCase
testLevel: L3
status: draft
name: "Verify that integrity-level propagation errors E841-E843 and W808 are enforced."
verifies:
  - REQ-TRS-VAL-004
---

Verify that integrity-level propagation errors E841-E843 and W808 are enforced.

```gherkin
Feature: Integrity-level propagation enforcement

  Scenario: E841 — derivedFromSafetyGoal element missing integrity level
    Given a SafetyGoal with asilLevel: D
    And a Requirement with derivedFromSafetyGoal: pointing to it but no asilLevel:
    When the tool is invoked
    Then an E841 finding is emitted for the Requirement

  Scenario: E842 — derivedFrom element missing integrity level
    Given a parent Requirement with asilLevel: C
    And a child Requirement with derivedFrom: pointing to the parent but no asilLevel:
    When the tool is invoked
    Then an E842 finding is emitted for the child Requirement

  Scenario: E843 — satisfies element missing integrity level
    Given a Requirement with silLevel: 2
    And an architecture element with satisfies: pointing to it but no silLevel:
    When the tool is invoked
    Then an E843 finding is emitted for the architecture element

  Scenario: W808 — element integrity level lower than source without breakdownAdr:
    Given a parent Requirement with asilLevel: D
    And a child Requirement with asilLevel: B, derivedFrom: the parent, and no breakdownAdr:
    When the tool is invoked
    Then a W808 finding is emitted for the child Requirement
```

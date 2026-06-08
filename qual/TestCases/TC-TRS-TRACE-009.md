---
id: TC-TRS-TRACE-009
type: TestCase
testLevel: L3
status: draft
title: "Verify that E016/E017/E018 are emitted for cycles in supertype, derivedFrom, and subsets graphs."
verifies:
  - REQ-TRS-TRACE-009
---

Verify that the tool detects cycles in all three hierarchical relationship types and emits the corresponding error code.

```gherkin
Feature: Cycle detection in hierarchical relationship graphs

  Scenario: supertype cycle produces E016
    Given two PartDef elements A and B
    And A has supertype: B
    And B has supertype: A
    When the tool validates the model
    Then exactly one E016 finding is emitted
    And no E017 or E018 findings are emitted

  Scenario: derivedFrom cycle produces E017
    Given two Requirement elements REQ-A-001 and REQ-B-001
    And REQ-A-001 has derivedFrom: [REQ-B-001]
    And REQ-B-001 has derivedFrom: [REQ-A-001]
    When the tool validates the model
    Then exactly one E017 finding is emitted
    And no E016 or E018 findings are emitted

  Scenario: subsets cycle produces E018
    Given two feature elements F1 and F2
    And F1 has subsets: F2
    And F2 has subsets: F1
    When the tool validates the model
    Then exactly one E018 finding is emitted
    And no E016 or E017 findings are emitted

  Scenario: typedBy self-reference produces E107
    Given a usage element typed by itself
    When the tool validates the model
    Then exactly one E107 finding is emitted
    And no E016, E017, or E018 findings are emitted

  Scenario: typedBy cycle produces E107
    Given two usage elements A and B
    And A has typedBy: B
    And B has typedBy: A
    When the tool validates the model
    Then exactly one E107 finding is emitted

  Scenario: removing the cycle removes the finding
    Given a model that previously contained a derivedFrom cycle between REQ-A-001 and REQ-B-001
    And the derivedFrom link on REQ-B-001 has been removed
    When the tool validates the model
    Then no E017 finding is emitted
```

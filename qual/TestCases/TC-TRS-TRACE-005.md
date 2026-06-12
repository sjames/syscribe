---
id: TC-TRS-TRACE-005
type: TestCase
testLevel: L3
status: draft
name: "Verify that E312 is emitted when a parent Requirement appears in a satisfies: list."
verifies:
  - REQ-TRS-TRACE-005
---

Verify that E312 is emitted when a parent Requirement appears in a satisfies: list.

```gherkin
Feature: Parent requirement cannot be assigned

  Scenario: Architecture element satisfying a parent Requirement produces E312
    Given a parent Requirement REQ-PARENT-001 that has derived children
    And an architecture element with satisfies: [REQ-PARENT-001]
    When the tool is invoked
    Then an E312 finding is emitted

  Scenario: Architecture element satisfying a leaf Requirement does not produce E312
    Given a leaf Requirement REQ-LEAF-001 with no derived children
    And an architecture element with satisfies: [REQ-LEAF-001]
    When the tool is invoked
    Then no E312 finding is emitted
```

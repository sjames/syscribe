---
id: TC-TRS-TRACE-002
type: TestCase
testLevel: L3
status: draft
name: "Verify that E310 is emitted when derivedFrom: is present but breakdownAdr: is absent."
verifies:
  - REQ-TRS-TRACE-002
---

Verify that E310 is emitted when derivedFrom: is present but breakdownAdr: is absent.

```gherkin
Feature: Breakdown ADR required for derived requirements

  Scenario: Requirement with derivedFrom: but no breakdownAdr: produces E310
    Given a Requirement with derivedFrom: [REQ-PARENT-001]
    And REQ-PARENT-001 exists in the model
    And the Requirement has no breakdownAdr: field
    When the tool is invoked
    Then exactly one E310 finding is emitted for that Requirement

  Scenario: Requirement with derivedFrom: and valid breakdownAdr: produces no E310
    Given a Requirement with derivedFrom: [REQ-PARENT-001]
    And breakdownAdr: ADR-SYS-001 where ADR-SYS-001 has status: accepted
    When the tool is invoked
    Then no E310 finding is emitted
```

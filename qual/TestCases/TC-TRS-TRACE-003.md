---
id: TC-TRS-TRACE-003
type: TestCase
testLevel: L3
status: draft
name: "Verify that W303 is emitted when a breakdownAdr: references a proposed ADR on an approved requirement."
verifies:
  - REQ-TRS-TRACE-003
---

Verify that W303 is emitted when a breakdownAdr: references a proposed ADR on an approved requirement.

```gherkin
Feature: Proposed breakdown ADR warning

  Scenario: Approved requirement with proposed breakdownAdr: produces W303
    Given a Requirement with status: approved
    And breakdownAdr: ADR-DRAFT-001 where ADR-DRAFT-001 has status: proposed
    When the tool is invoked
    Then a W303 finding is emitted for that Requirement

  Scenario: Draft requirement with proposed breakdownAdr: does not produce W303
    Given a Requirement with status: draft
    And breakdownAdr: ADR-DRAFT-001 where ADR-DRAFT-001 has status: proposed
    When the tool is invoked
    Then no W303 finding is emitted

  Scenario: Approved requirement with accepted breakdownAdr: does not produce W303
    Given a Requirement with status: approved
    And breakdownAdr: ADR-ACCEPTED-001 where ADR-ACCEPTED-001 has status: accepted
    When the tool is invoked
    Then no W303 finding is emitted
```

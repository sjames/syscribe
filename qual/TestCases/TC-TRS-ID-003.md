---
id: TC-TRS-ID-003
type: TestCase
testLevel: L3
status: draft
name: "Verify that ADR elements are validated against the ADR-* id pattern."
verifies:
  - REQ-TRS-ID-003
---

Verify that ADR elements are validated against the ADR-* id pattern.

```gherkin
Feature: ADR id pattern validation

  Scenario: Valid ADR id is accepted
    Given an ADR element with id: ADR-SYS-001 and status: accepted
    When the tool is invoked
    Then no E300 or E301 finding is emitted for that element

  Scenario: ADR id not matching pattern produces E300
    Given an ADR element with id: ADR-sys-001 (lowercase)
    When the tool is invoked
    Then an E300 finding is emitted for that element

  Scenario: ADR missing id produces E301
    Given an ADR element with no id: field
    When the tool is invoked
    Then an E301 finding is emitted for that element
```

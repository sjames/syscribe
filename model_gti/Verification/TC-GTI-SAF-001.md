---
type: TestCase
id: TC-GTI-SAF-001
name: "Insulation resistance check"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-030
  - REQ-GTI-SW-003
  - REQ-GTI-HW-003
---

```gherkin
Feature: Insulation measurement

  Scenario: low insulation blocks connection
    Given the array-to-ground resistance is set below the limit
    When the inverter starts
    Then it does not close the grid relays
    And it reports the measured value in kΩ

  Scenario: good insulation allows connection
    Given the array-to-ground resistance is 5 MΩ
    When the inverter starts
    Then the check passes
```

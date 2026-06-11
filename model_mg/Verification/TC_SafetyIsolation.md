---
type: TestCase
id: TC-EVCS-002
title: "Verify the connector de-energises within 100 ms of a fault"
status: approved
testLevel: L5
verifies:
  - REQ-EVCS-SYS-003
---

## Test Procedure

```gherkin
Feature: Fault isolation timing

  Scenario: Insulation fault during charging
    Given an active charging session
    When an insulation fault is injected on the DC bus
    Then the DC contactors open and the connector voltage drops below the touch-safe threshold within 100 ms
```

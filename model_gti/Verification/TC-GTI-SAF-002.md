---
type: TestCase
id: TC-GTI-SAF-002
name: "Residual-current monitoring"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-031
---

```gherkin
Feature: Residual current

  Scenario: continuous residual current
    Given a leakage resistor injects a residual current
    When the current rises above 300 mA
    Then the inverter disconnects within the specified time

  Scenario: sudden residual current steps
    Given the inverter runs
    When a step of 30, 60 and 150 mA is applied in turn
    Then it disconnects within the time required for each step
```

---
type: TestCase
id: TC-ENG-PERF-003
name: Integration test — closed-loop fuel control lambda accuracy
status: active
testLevel: L2
verifies:
  - REQ-ENG-PERF-002
---

```gherkin
Feature: Fuel control lambda accuracy

  Scenario: Steady-state lambda at 2000 rpm 50 % load
    Given the FuelControl SWC is running with a lambda sensor plant model
    And engine speed is 2000 rpm, load 50 %, coolant 80 °C
    When the closed loop has run for 30 s
    Then the lambda reading is within 0.02 of 1.00

  Scenario: Load step transient recovery
    Given steady-state at lambda 1.00
    When load steps from 30 % to 70 % in 100 ms
    Then lambda recovers to within 0.04 of 1.00 within 500 ms
```

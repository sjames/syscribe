---
type: TestCase
id: TC-ENG-PERF-001
name: Performance simulation — throttle and fuel control response over drive cycle
status: active
testLevel: L4
verifies:
  - REQ-ENG-PERF-000
---

```gherkin
Feature: Engine ECU performance simulation

  Scenario: Throttle and fuel response across WLTP drive cycle
    Given the ECU software is loaded into a QEMU simulation with plant model
    And the drive cycle is set to WLTP Class 3
    When the full 1800-second cycle is executed
    Then throttle response latency is ≤ 50 ms at all transient events
    And mean lambda error is ≤ 0.02 above 1000 rpm with coolant > 60 °C
    And no error DTCs are set during the cycle

  Scenario: Cold-start enrichment transient
    Given the coolant temperature is set to −20 °C
    When the engine cranks and reaches 400 rpm
    Then fuel enrichment is applied within 2 firing events
    And the engine reaches idle speed without stall
```

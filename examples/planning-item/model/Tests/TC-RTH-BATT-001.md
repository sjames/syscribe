---
type: TestCase
id: TC-RTH-BATT-001
name: "Battery-threshold monitor triggers RTH at the configured cutoff"
status: active
testLevel: L3
verifies:
  - REQ-RTH-001
tags:
  - rth
  - battery
---

Bench test: simulate a declining battery-level feed into the flight
controller and confirm a return-to-home event is initiated exactly when the
level crosses the configured critical threshold.

```gherkin
Feature: Battery-threshold RTH trigger

  Scenario: RTH initiates at the critical battery threshold
    Given the flight controller is airborne in normal flight mode
    When the simulated battery level crosses the critical threshold
    Then a return-to-home event shall be initiated within one control cycle
```

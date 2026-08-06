---
type: TestCase
id: TC-DRONE-ENDUR-001
name: "Bench endurance run confirms minimum flight time"
status: active
testLevel: L3
verifies:
  - REQ-DRONE-ENDUR-001
tags:
  - propulsion
  - endurance
---

Bench test: run the rotor assembly at nominal payload-equivalent load until battery cutoff, and confirm elapsed thrust-producing time meets the endurance floor.

```gherkin
Feature: Rotor assembly endurance

  Scenario: Nominal-load endurance run meets the floor
    Given the rotor assembly is loaded to nominal-payload-equivalent thrust
    When the bench run is started from a full charge
    Then continuous thrust output shall be sustained for at least 20 minutes
```

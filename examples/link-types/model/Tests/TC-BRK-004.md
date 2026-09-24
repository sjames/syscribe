---
type: TestCase
id: TC-BRK-004
name: "Regenerative share reaches 70 percent during service braking"
status: active
testLevel: L2
verifies:
  - REQ-BRK-004
tags:
  - brakes
---

Regenerative share reaches 70 percent during service braking.

```gherkin
Feature: Regenerative share reaches 70 percent during service braking

  Scenario: Regenerative share reaches 70 percent during service braking
    Given the brake-by-wire system is powered and calibrated
    When a 0.2 g deceleration is requested with the battery able to accept charge
    Then at least 70 percent of the torque shall come from regenerative braking
```

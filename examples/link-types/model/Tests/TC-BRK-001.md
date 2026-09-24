---
type: TestCase
id: TC-BRK-001
name: "Vehicle stops without wheel lock on a low-friction surface"
status: active
testLevel: L4
verifies:
  - REQ-BRK-001
tags:
  - brakes
---

Vehicle stops without wheel lock on a low-friction surface.

```gherkin
Feature: Vehicle stops without wheel lock on a low-friction surface

  Scenario: Vehicle stops without wheel lock on a low-friction surface
    Given the brake-by-wire system is powered and calibrated
    When the vehicle brakes at full demand from 100 km/h on a wet surface
    Then the vehicle shall stop with no wheel locked for longer than 50 ms
```

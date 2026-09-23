---
type: TestCase
id: TC-BRK-003
name: "Modulator releases caliper pressure within 10 ms"
status: active
testLevel: L3
verifies:
  - REQ-BRK-003
tags:
  - brakes
---

Modulator releases caliper pressure within 10 ms.

```gherkin
Feature: Modulator releases caliper pressure within 10 ms

  Scenario: Modulator releases caliper pressure within 10 ms
    Given the brake-by-wire system is powered and calibrated
    When a pressure-release command is issued to the modulator
    Then caliper pressure shall fall by at least 80 percent within 10 ms
```

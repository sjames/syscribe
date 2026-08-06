---
type: TestCase
id: TC-RTH-NOISE-001
name: "A single transient battery-sensor glitch does not trigger RTH"
status: active
testLevel: L3
verifies:
  - REQ-RTH-003
tags:
  - rth
  - robustness
---

Regression test for the false-trigger bug fixed by
`Planning::PI-RTH-BUGFIX-001`: inject a single-sample battery-level dropout
below the critical threshold, immediately followed by a recovery to a normal
reading, and confirm no return-to-home event is initiated.

```gherkin
Feature: RTH debounce against transient sensor noise

  Scenario: A single glitched low-battery sample does not trigger RTH
    Given the flight controller is airborne in normal flight mode
    When a single simulated battery-level sample glitches below the critical threshold and immediately recovers
    Then no return-to-home event shall be initiated
```

---
type: TestCase
id: TC-BRK-002
name: "Wheel-lock onset detected within 20 ms"
status: active
testLevel: L2
verifies:
  - REQ-BRK-002
tags:
  - brakes
---

Wheel-lock onset detected within 20 ms.

```gherkin
Feature: Wheel-lock onset detected within 20 ms

  Scenario: Wheel-lock onset detected within 20 ms
    Given the brake-by-wire system is powered and calibrated
    When a simulated wheel-slip ratio exceeds the lock threshold
    Then the brake controller shall flag lock onset within 20 ms
```

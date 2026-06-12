---
id: TC-FIX-BRK-001
type: TestCase
testLevel: L3
status: active
title: Verify service brake deceleration
verifies:
  - REQ-FIX-BRK-001
---
Verify the service brake deceleration requirement.

```gherkin
Feature: braking
  Scenario: hard stop
    Given a moving vehicle
    When the service brake is applied
    Then deceleration is at least 6 m/s^2
```

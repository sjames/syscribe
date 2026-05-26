---
type: TestCase
id: TC-UAV-ENDUR-001
title: "UAV sustains 25-minute flight at nominal payload under standard conditions"
status: active
testLevel: L5
verifies:
  - REQ-UAV-ENDUR-001
tags:
  - endurance
  - flight-test
---

Hardware-in-the-loop flight test executed per ATP-001. UAV is configured with nominal 0.5 kg survey payload and full battery pack. Test is conducted in calm wind conditions (< 5 m/s).

Run: `cargo xtask hil -- endurance-test --min-duration 25`

```gherkin
Feature: Flight endurance

  Background:
    Given the UAV is fully assembled with nominal 0.5 kg payload
    And the battery pack is fully charged
    And ambient wind speed is less than 5 m/s

  Scenario: UAV maintains flight for at least 25 minutes
    When the UAV takes off and transitions to autonomous waypoint mission
    Then the UAV remains airborne continuously for at least 25 minutes
    And the battery charge at landing is greater than 10 %

  Scenario: Flight aborts gracefully if battery falls below threshold mid-mission
    Given the UAV is flying at the 20-minute mark
    When the battery charge drops to 10 %
    Then the UAV shall initiate autonomous return-to-home
    And shall land within 3 minutes of initiating return
```

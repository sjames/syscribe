---
type: TestCase
id: TC-UAV-SAFE-001
title: "Autonomous landing descent rate ≤ 3 m/s on battery-critical trigger"
status: active
testLevel: L5
verifies:
  - REQ-UAV-SAFE-001
tags:
  - safety
  - contingency
  - landing
---

Hardware-in-the-loop test. A simulated battery-critical alert is injected at 30 m AGL while the UAV is in autonomous hover. Descent rate is measured via barometric altimeter log at 10 Hz.

Run: `cargo xtask hil -- safe-landing-test`

```gherkin
Feature: Autonomous safe landing on battery-critical event

  Background:
    Given the UAV is hovering autonomously at 30 m AGL
    And the barometric altimeter is logging at 10 Hz

  Scenario: Descent rate does not exceed 3 m/s during autonomous landing
    When a battery-critical alert is injected via the fault injection interface
    Then the UAV shall transition to autonomous landing mode within 1 second
    And the descent rate measured by the altimeter shall not exceed 3.0 m/s at any sample
    And the UAV shall reach the ground within 30 seconds

  Scenario: Landing is initiated on link-loss exceeding 3 seconds
    When the command link is interrupted for 4 seconds
    Then the UAV shall initiate autonomous landing
    And the descent rate shall not exceed 3.0 m/s
```

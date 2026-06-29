---
type: TestCase
id: TC-UAV-MISSION-001
name: "Full survey mission demonstrates combined endurance, navigation, and data-link performance"
status: active
testLevel: L5
verifies:
  - REQ-UAV-PERF-000
tags:
  - performance
  - integration
  - flight-test
---

System-level hardware-in-the-loop flight test that exercises a complete survey mission
end-to-end, verifying the emergent mission-performance envelope (endurance, navigation accuracy,
and command/telemetry link range together) rather than each leaf requirement in isolation.

Run: `cargo xtask hil -- mission-performance --profile survey`

```gherkin
Feature: Mission performance integration

  Background:
    Given the UAV is assembled in the survey configuration with a full battery pack
    And ambient wind speed is less than 5 m/s
    And a survey mission plan covering the full operating range is loaded

  Scenario: The UAV completes the planned survey mission within its performance envelope
    When the UAV flies the full autonomous survey mission to completion
    Then the mission duration is at least 25 minutes without a battery-critical abort
    And the navigation position error stays within 1.5 m CEP throughout the mission
    And the command/telemetry link remains connected out to at least 5 km line of sight
    And all collected survey data is downlinked successfully on return
```

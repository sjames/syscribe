---
type: TestCase
id: TC-GTI-SYS-003
name: "Trip and ride-through sweep"
status: active
testLevel: L5
verifies:
  - REQ-GTI-SYS-007
  - REQ-GTI-SYS-008
  - REQ-GTI-HW-001
tags: [reg-unverified]
---

```gherkin
Feature: Grid disturbances on the grid simulator

  Scenario: trip above and below the limits
    Given the inverter runs at rated power
    When voltage or frequency steps to each limit of the active profile
    Then it disconnects within the profile clearing time
    And it does not disconnect for a step just inside the limit

  Scenario: ride-through
    Given the inverter runs at rated power
    When voltage sags to the ride-through region of the profile for its allowed duration
    Then it stays connected and resumes full power within the profile ramp time
```

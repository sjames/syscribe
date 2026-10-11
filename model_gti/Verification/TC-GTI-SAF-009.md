---
type: TestCase
id: TC-GTI-SAF-009
name: "Sensing common cause: perturbation-response check"
status: active
testLevel: L5
verifies:
  - REQ-GTI-SYS-045
  - REQ-GTI-SW-008
  - REQ-GTI-HW-010
tags: [reg-unverified]
---

```gherkin
Feature: Both voltage chains stuck at nominal

  Scenario: grid lost with both sensing chains reading nominal
    Given the inverter runs at rated power on a grid simulator
    And both voltage-sensing chains are forced to report nominal
    When the grid breaker opens
    Then the perturbation response is missing
    And the inverter stops energising and opens the relays within the island-clearing time of the profile

  Scenario: healthy grid is not tripped
    Given the inverter runs at rated power on a grid with the profile's maximum impedance
    When the perturbation check runs for one hour
    Then no trip occurs

  Scenario: weak grid
    Given the grid impedance is raised above the profile limit
    When the next perturbation is applied
    Then the inverter treats the grid as lost
```

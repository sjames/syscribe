---
type: TestCase
id: TC-EVCS-001
name: "Verify the station delivers at least 150 kW DC to one vehicle"
status: approved
testLevel: L5
verifies:
  - REQ-EVCS-SYS-001
---

## Test Procedure

```gherkin
Feature: Peak delivered power

  Scenario: Single vehicle at full power
    Given a compatible EV connected to one stall
    And nominal grid and thermal conditions
    When the charging session reaches steady state
    Then the measured DC output power is at least 150 kW
```

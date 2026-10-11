---
type: TestCase
id: TC-GTI-SAF-006
name: "Thermal derating and shutdown"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-036
---

```gherkin
Feature: Thermal chamber

  Scenario: derating above 45 °C
    Given the inverter at rated power in a thermal chamber
    When ambient rises from 45 °C to 60 °C
    Then output power derates linearly

  Scenario: over-temperature shutdown
    Given the heatsink reaches the maximum temperature
    When the limit is exceeded
    Then the inverter shuts down and logs the event
```

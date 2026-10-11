---
type: TestCase
id: TC-GTI-SYS-008
name: "1000 V input limit and start-up gate"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-025
  - REQ-GTI-HW-007
  - REQ-GTI-SW-007
---

```gherkin
Feature: High-voltage inputs

  Scenario: over-voltage open circuit
    Given a PV simulator with open-circuit voltage of 1010 V
    When the inverter starts
    Then it does not connect the tracker

  Scenario: survival at 1100 V
    Given a PV simulator at 1100 V open circuit for 1 hour
    When the voltage returns to 800 V
    Then the inverter starts normally

  Scenario: below start voltage
    Given a PV simulator at 240 V
    When the inverter starts
    Then the tracker stays off
```

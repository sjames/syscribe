---
type: TestCase
id: TC-GTI-SYS-005
name: "MPPT efficiency with mismatched strings"
status: active
testLevel: L5
verifies:
  - REQ-GTI-STK-002
  - REQ-GTI-SYS-020
  - REQ-GTI-SYS-021
  - REQ-GTI-SYS-022
  - REQ-GTI-HW-006
  - REQ-GTI-SW-006
---

```gherkin
Feature: Multi-string tracking

  Scenario: independent trackers
    Given tracker 1 has strings of different orientation from tracker 2 on PV simulators
    When irradiance differs by 40 % between the trackers
    Then each tracker settles at its own maximum power point
    And total power equals the sum of the individually available power within 0.5 %

  Scenario: static tracking efficiency
    Given a PV simulator with a standard I-V curve
    When the tracker runs the static test profile
    Then static MPPT efficiency is at least 99.5 %

  Scenario: input range
    Given a PV simulator with open-circuit voltage of 1000 V
    When the inverter starts
    Then every tracker accepts the input and tracks between 200 V and 850 V
```

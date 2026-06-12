---
id: TC-HIL-001
type: TestCase
name: "HIL timing measurement of the control loop deadline"
status: active
testLevel: L5
tags: [timing]
verifies:
  - REQ-WCET-001
---
```gherkin
Feature: control loop timing
  Scenario: worst case
    Given worst-case load on the HIL rig
    Then the measured loop time is below 10 ms
```

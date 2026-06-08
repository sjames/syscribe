---
type: TestCase
id: TC-DEMO-001
testLevel: L3
status: active
title: "Verify the independent torque monitor trips on disagreement"
verifies:
  - REQ-DEMO-001
---

```gherkin
Feature: Independent torque monitor
  Scenario: monitor trips on channel disagreement
    Given two torque channels disagree beyond tolerance
    When the monitor evaluates the channels
    Then the torque request is limited to zero
```

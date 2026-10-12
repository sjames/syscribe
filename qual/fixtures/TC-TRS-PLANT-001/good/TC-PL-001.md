---
type: TestCase
id: TC-PL-001
name: "lag response"
status: active
testLevel: L3
subject: Sys
verifies: [REQ-PL-001]
---

```gherkin
Feature: lag

  Scenario: the step response reaches its analytic value
    When 1 s elapses
    Then ctl.y > 0.99326 and ctl.y < 0.99327
    And ctl is in state Settled

  Scenario: a doubled gain
    Given fault FM-PL-001 is injected
    When 1 s elapses
    Then ctl.y > 1.986 and ctl.y < 1.987

  Scenario: a stuck output
    Given fault FM-PL-002 is injected
    When 1 s elapses
    Then ctl.y == 0.5
    And ctl is in state Run
```

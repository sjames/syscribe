---
type: TestCase
id: TC-SIL-SAFE-001
name: System integration — 2oo2 channel disagreement forces immediate safe state
status: active
testLevel: L5
verifies:
  - REQ-SIL-SAFE-000
  - REQ-SIL-SW-001
---

```gherkin
Feature: 2oo2D safe state enforcement

  Scenario: Channel A and Channel B disagree on route conflict assessment
    Given both vital channels are running normally and a route request is pending
    When fault injection causes Channel A to output "route clear" while Channel B outputs "route blocked"
    Then both channels assert the safe state within 50ms (all signals to most-restrictive aspect)
    And a "channel comparison failure" DTC is logged with timestamp
    And the interlocking remains in safe state until manual reset by an authorised maintainer

  Scenario: Cross-comparison bus failure forces safe state
    Given both channels are running and the cross-comparison bus is active
    When the cross-comparison bus physical link is disconnected
    Then both channels assert the safe state within one scan cycle
    And neither channel resumes normal operation without the comparison bus restored
```

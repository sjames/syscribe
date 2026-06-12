---
type: TestCase
id: TC-SIL-SAFE-002
name: System integration — signal reverts to red within one scan cycle when track section becomes occupied
status: active
testLevel: L5
verifies:
  - REQ-SIL-SAFE-002
---

```gherkin
Feature: Continuous signal supervision after clearance

  Scenario: Track circuit becomes occupied while signal is clear
    Given a route is set, all conditions are satisfied, and a home signal shows a proceed aspect
    When a train occupies the track circuit on the cleared route
    Then the signal returns to the most-restrictive aspect within 50ms
    And the route remains locked until the train has cleared the overlap

  Scenario: Points detection lost while signal is clear
    Given a route is set with signal cleared and points confirmed in position
    When fault injection removes the points detection confirmation
    Then the signal returns to most-restrictive aspect within 50ms
    And the PointsController reports a "detection lost" fault
    And the route is locked until detection is restored and confirmed
```

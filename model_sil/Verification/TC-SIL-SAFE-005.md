---
type: TestCase
id: TC-SIL-SAFE-005
title: Integration — PointsController position confirmation blocks signal clearance
status: active
testLevel: L4
verifies:
  - REQ-SIL-SAFE-003
---

```gherkin
Feature: Points position confirmation is required before signal clearance

  Scenario: Signal remains at danger when points detection is absent
    Given the interlocking has a route set from signal S1 across points P1 to P2
    And the SignalController is evaluating clearance conditions for S1
    And the PointsController has not yet received a detection confirmation from the points machine for P1
    When one complete scan cycle elapses after the route-set command
    Then the SignalController shall not clear signal S1
    And the diagnostic log shall record "points unconfirmed — signal clearance suppressed" for P1

  Scenario: Signal clears only after points detection current confirmed
    Given the route from S1 through P1 is set and locked
    And all other clearance conditions are met (track sections clear, no conflict, LX confirmed)
    When the PointsController receives a valid detection current from P1 in the commanded position
    Then the SignalController shall clear S1 to the appropriate proceed aspect within one scan cycle (≤ 20 ms)

  Scenario: Loss of detection during route occupation causes immediate signal reversion
    Given signal S1 is clear and a train is on approach to the route
    And PointsController has P1 confirmed in the correct position
    When the detection current from P1 drops (simulated by disconnecting the detection relay)
    Then the PointsController shall report "position unconfirmed" within one scan cycle
    And the SignalController shall revert S1 to the most-restrictive aspect within one scan cycle (≤ 20 ms)
    And the route shall enter approach-locked state preventing manual release until the train clears

  Scenario: Points timeout — move command with no confirmation causes unconfirmed report
    Given the RouteProcessor has commanded P1 to move to the reverse position
    When the configured moveTimeoutMs (5000 ms) elapses with no detection current from the reverse detection relay
    Then the PointsController shall report "position unconfirmed" for P1
    And the SignalController shall not permit signal clearance for any route through P1
    And a diagnostic event shall be raised for "points move timeout" on P1
```

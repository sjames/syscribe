---
type: TestCase
id: TC-SIL-SYS-001
name: HIL — End-to-end system integration test for SIL 4 interlocking safe state
status: active
testLevel: L5
verifies:
  - REQ-SIL-SYS-000
---

```gherkin
Feature: SIL 4 Computer-Based Interlocking meets system-level safety requirements

  Scenario: Safe route setting and train movement from signaller request to track clearance
    Given the Station 1 interlocking is fully initialised on the HIL bench with simulated track, points, and signals
    And both VitalProcessor channels (Channel A and Channel B) are operating within normal parameters
    And all track circuit sections are reporting clear
    And all points are in their default positions and detection-confirmed
    When the signaller issues a route-set request for route R1 via the operator workstation
    Then the ConflictChecker shall confirm no conflict with any currently-set route within one scan cycle (≤ 20 ms)
    And the RouteProcessor shall lock route R1 and all points on the route within one scan cycle
    And the SignalController shall clear signal S1 at the approach to route R1 within one scan cycle of lock confirmation
    And the simulated train shall enter the first track section (TC1) within the expected transit time
    And on TC1 occupation the approach-locking timer shall start preventing manual route release
    And on clearing TC3 (the overlap) the route shall be automatically released and S1 shall return to danger

  Scenario: 2oo2D channel disagreement triggers immediate system safe state
    Given the HIL bench is running with both VitalProcessor channels operational
    And route R1 is set with signal S1 at proceed
    When the fault injector forces Channel B to compute a different output state for points P1 than Channel A
    Then both channels shall detect the cross-comparison mismatch within one scan cycle (≤ 20 ms)
    And both channels shall independently assert the safe state: all signals red, points locked, level crossing barriers down
    And the safe state shall be maintained until a manual reset by authorised maintenance personnel
    And the event log shall record "2oo2D comparison failure" with timestamp and the differing output values

  Scenario: System recovers from single track circuit transient without loss of route integrity
    Given route R2 is set and locked with no train currently on the route
    When a simulated electrical transient causes TC2 (a section within route R2) to briefly report occupied for one scan cycle
    And the track circuit returns to reporting clear in the following scan cycle
    Then the SignalController shall immediately revert S2 to danger on the "occupied" report
    And S2 shall remain at danger for the configured debounce period (3 consecutive clear reports = 60 ms)
    And after the debounce period S2 shall automatically return to the proceed aspect
    And route R2 shall remain locked throughout — no spurious route release shall occur

  Scenario: Level crossing barriers-not-confirmed prevents signal clearance for crossing route
    Given route R3 passes over level crossing LC-001
    And all other conditions for route R3 clearance are satisfied (sections clear, points confirmed, no conflict)
    And the LevelCrossingModule reports "barriers not confirmed down" for LC-001
    When the SignalController evaluates clearance conditions for signal S3 on route R3
    Then S3 shall not be cleared (shall remain at the most-restrictive aspect)
    And when the LevelCrossingModule subsequently reports "barriers confirmed down" for LC-001
    Then S3 shall be cleared within one scan cycle (≤ 20 ms)
```

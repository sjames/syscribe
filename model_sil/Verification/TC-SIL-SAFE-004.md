---
type: TestCase
id: TC-SIL-SAFE-004
name: Integration test — level crossing barriers must be confirmed down before signal clearance
status: active
testLevel: L4
verifies:
  - REQ-SIL-SAFE-004
---

```gherkin
Feature: Level crossing pre-condition for signal clearance

  Scenario: Signal blocked when barriers not confirmed down
    Given a route through level crossing LC-001 is requested
    And the LevelCrossingModule has activated barrier descent
    But the barrier confirmation contacts have not yet closed
    When the RouteProcessor evaluates the clearance preconditions
    Then the signal remains at most-restrictive aspect
    And the system logs "awaiting barrier confirmation on LC-001"

  Scenario: Signal clears only after both detection circuits confirm barriers down
    Given a route through LC-001 is requested and barriers are commanded down
    When both primary and secondary detection circuits confirm barrier-lowered position
    Then the RouteProcessor approves signal clearance and the SignalController clears the signal
    And the elapsed time from barrier command to signal clearance is recorded for audit
```

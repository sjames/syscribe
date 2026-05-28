---
type: TestCase
id: TC-ENG-PERF-002
title: Unit test — throttle control step response latency
status: active
testLevel: L1
verifies:
  - REQ-ENG-PERF-001
---

```gherkin
Feature: Throttle control unit response

  Scenario: Step demand — 10 % to 90 % command
    Given the ThrottleControl SWC is running in a unit test harness
    And the TPS input is at 10 % position
    When a step demand of 90 % is applied at time T=0
    Then the output command reaches 90 % within 50 ms
    And the overshoot is less than 5 % of final value

  Scenario: Fail-safe output on fault flag
    Given the ThrottleControl SWC is in normal operation
    When the SafetyMonitor sets the fault flag
    Then the throttle output is set to the fail-safe value (7 %) within 10 ms
```

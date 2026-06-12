---
type: TestCase
id: TC-UAV-FC-001
name: "FC detects injected sensor failure within 50 ms on HIL bench"
status: active
testLevel: L5
verifies:
  - REQ-UAV-FC-001
tags:
  - safety
  - fault-detection
  - flight-controller
---

Hardware-in-the-loop test using the fault injection bench. Sensor failure signals are injected via the HIL interface and response time is measured from fault signal assertion to FC mode-transition command on the CAN bus.

Run: `cargo xtask hil -- fc-fault-injection`

```gherkin
Feature: Flight controller sensor fault detection

  Background:
    Given the HIL bench is configured with the flight controller under test
    And the fault injection interface is connected
    And CAN bus monitoring is active at 1 kHz sample rate

  Scenario: IMU failure is detected within 50 ms
    When a simulated IMU sensor failure is injected
    Then the flight controller shall assert a FAULT_DETECTED flag within 50 ms
    And the FC shall transition to degraded mode
    And a fault telemetry event shall be emitted on the CAN bus

  Scenario: GPS failure is detected within 50 ms
    When a simulated GPS loss-of-fix condition is injected
    Then the flight controller shall assert a FAULT_DETECTED flag within 50 ms
    And the FC shall switch to dead-reckoning mode

  Scenario: Single sensor failure does not cause immediate loss of control
    When any single sensor failure is injected
    Then the UAV attitude estimate error shall remain below 5 degrees for at least 2 seconds
```

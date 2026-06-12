---
type: TestCase
id: TC-ENG-SAFE-001
name: HIL — safety system end-to-end fault response under all single-point faults
status: active
testLevel: L5
verifies:
  - REQ-ENG-SAFE-000
---

```gherkin
Feature: Safety system hardware-in-the-loop fault injection

  Scenario: TPS divergence fault end-to-end
    Given the ECU is running on hardware with a real throttle actuator
    When TPS track divergence of 6 % is injected via fault injection harness
    Then the safety monitor detects the fault within 100 ms
    And the throttle actuator returns to the fail-safe position within 200 ms
    And the engine speed drops to idle within 500 ms

  Scenario: Watchdog timeout causes MCU reset
    Given the ECU is running on hardware
    When the software watchdog service is suppressed for 15 ms
    Then the hardware watchdog resets the MCU within 30 ms
    And the ECU reboots into the safe state
    And a power-on-reset DTC is recorded in non-volatile memory
```

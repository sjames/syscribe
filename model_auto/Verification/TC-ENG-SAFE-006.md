---
type: TestCase
id: TC-ENG-SAFE-006
title: HIL — stuck-open throttle detected by position feedback verification
status: active
testLevel: L5
verifies:
  - REQ-ENG-SAFE-005
---

```gherkin
Feature: Throttle close command verified by TPS position feedback

  Scenario: Stuck-open actuator triggers fuel cut within 200 ms
    Given the ECU is running on the HIL bench with a simulated stuck-open throttle actuator
    And the throttle plate is mechanically held at 30 % opening by the fault injector
    When the ECU issues a throttle-close command with target position less than 5 %
    And the TPS reading remains above 15 % for 200 ms after the command
    Then the safety monitor asserts a fuel cut on all cylinders within one engine cycle
    And DTC P2111 (Throttle Actuator Control System Stuck Open) is set in DTC memory
    And the malfunction indicator lamp state is set to illuminate
    And the throttle target is forced to the limp-home position of 7 %

  Scenario: Normal throttle close does not trigger false fault
    Given the ECU is running normally with a correctly functioning throttle actuator
    When the ECU issues a throttle-close command at any engine speed above 1000 rpm
    Then the TPS reading falls below 15 % within 150 ms
    And no fuel cut is asserted
    And DTC P2111 is not set

  Scenario: TPS track divergence during close command triggers immediate fuel cut
    Given the ECU has issued a throttle-close command
    And TPS track A reads 25 % while TPS track B reads 5 % (divergence of 20 %)
    When the safety monitor evaluates the TPS divergence condition
    Then the fuel cut is asserted immediately without waiting for the 200 ms window
    And DTC P0122/P0123 (TPS track divergence) is also set alongside P2111
```

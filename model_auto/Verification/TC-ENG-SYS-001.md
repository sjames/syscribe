---
type: TestCase
id: TC-ENG-SYS-001
name: System integration — end-to-end engine management under nominal and fault conditions
status: active
testLevel: L3
verifies:
  - REQ-ENG-SYS-000
---

```gherkin
Feature: Engine ECU system integration

  Scenario: Nominal engine management cycle
    Given the engine is running at 2000 rpm under 50 % load
    And all sensor inputs are within normal range
    When the driver applies 30 % accelerator demand
    Then the throttle position reaches 30 % within 50 ms
    And the lambda sensor reads 1.00 ± 0.02 within 2 s
    And no DTC is set

  Scenario: CAN authentication failure isolation
    Given the engine is running normally
    When a CAN frame with invalid MAC is received on the powertrain bus
    Then the frame is rejected by the CAN security module
    And a security DTC is logged within 50 ms
    And engine operation continues uninterrupted

  Scenario: Safety monitor fault isolation
    Given the engine is running at 3000 rpm
    When a TPS track divergence of 8 % is injected
    Then the safety monitor detects the fault within 100 ms
    And throttle is commanded to the fail-safe position
    And a safety DTC is logged
```

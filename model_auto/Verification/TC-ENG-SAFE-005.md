---
type: TestCase
id: TC-ENG-SAFE-005
name: HIL — rev limiter enforces fuel cut and ignition retard independently of TPS
status: active
testLevel: L5
verifies:
  - REQ-ENG-SAFE-004
---

```gherkin
Feature: Rev limiter operates independently of throttle position feedback

  Scenario: Soft ignition retard activates at 6200 rpm with TPS disconnected
    Given the ECU is running on the HIL bench with the throttle position sensor harness disconnected
    And engine speed is being driven by the dyno at a ramp rate of 200 rpm/s
    When engine speed crosses 6200 rpm as measured from the crankshaft position sensor event counter
    Then the safety monitor applies ignition retard of at least 20° BTDC within one engine cycle
    And the retard is applied per-cylinder without using TPS, MAP, or pedal demand signals
    And the engine torque output decreases measurably on the dyno load cell

  Scenario: Hard fuel cut activates at 6500 rpm within 20 ms with TPS disconnected
    Given the ECU is running on the HIL bench with the throttle position sensor harness disconnected
    And engine speed is ramping through 6500 rpm
    When the crankshaft event counter indicates the speed threshold has been exceeded
    Then fuel injection is suppressed for every cylinder within 20 ms of threshold crossing
    And engine speed begins to decrease within 100 ms
    And a rev limiter active flag is set in the ECU diagnostic data

  Scenario: Fuel injection re-enables with 100 rpm hysteresis
    Given the hard fuel cut is active with engine speed above 6500 rpm
    When engine speed drops below 6400 rpm as measured by the crankshaft position sensor
    Then fuel injection is re-enabled cylinder-by-cylinder as each injection window opens
    And no torque step exceeding 50 Nm occurs during the re-enable transition
```

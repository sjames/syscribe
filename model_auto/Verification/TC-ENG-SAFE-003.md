---
type: TestCase
id: TC-ENG-SAFE-003
name: HIL — hardware watchdog resets ECU within 30 ms of missed service window
status: active
testLevel: L5
verifies:
  - REQ-ENG-SAFE-002
---

```gherkin
Feature: Hardware watchdog timer HIL

  Scenario: Missed service window triggers MCU reset
    Given the ECU hardware is powered and the watchdog is armed
    When the watchdog service call is suppressed at T=0
    Then the hardware watchdog asserts reset within 30 ms
    And the MCU enters reset state as measured on the RESET pin

  Scenario: Early service detected by windowed watchdog
    Given the watchdog is in windowed mode with a 10 ms window
    When the service call is made at 4 ms (before the window opens at 5 ms)
    Then the watchdog asserts reset within 5 ms of the premature service
```

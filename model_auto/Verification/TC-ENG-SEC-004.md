---
type: TestCase
id: TC-ENG-SEC-004
title: UDS readMemoryByAddress is blocked without authenticated session
status: active
testLevel: L4
verifies:
  - REQ-ENG-SEC-004
---

```gherkin
Feature: UDS readMemoryByAddress access control and audit logging

  Scenario: readMemoryByAddress is rejected when no authenticated session is active
    Given the Engine ECU is powered on and in the default diagnostic session
    And no security access level 0x11 session has been established
    When a UDS service 0x23 readMemoryByAddress request is sent targeting the calibration memory region
    Then the ECU returns UDS Negative Response Code 0x33 (securityAccessDenied) within 100 ms
    And a security DTC is set within 200 ms of the attempt

  Scenario: readMemoryByAddress succeeds in authenticated session and creates audit log entry
    Given the Engine ECU is powered on
    And a valid security access challenge-response has been completed at level 0x11
    When a UDS service 0x23 readMemoryByAddress request is sent targeting the calibration memory region
    Then the ECU responds with the requested calibration data
    And one new audit log entry is written to the tamper-evident ring buffer
    And the log entry contains the request timestamp, start address, length, and session level 0x11
    And the log entry is protected by a CMAC integrity tag
```

---
type: TestCase
id: TC-ENG-SEC-002
title: Integration test — UDS programming session requires cryptographic authentication
status: active
testLevel: L2
verifies:
  - REQ-ENG-SEC-002
---

```gherkin
Feature: UDS programming session authentication

  Scenario: Valid authentication grants programming access
    Given the DiagnosticSecurityLayer is running in a test harness
    When a valid seed-and-key challenge-response is performed
    Then programming security level is granted
    And the ECU accepts calibration data writes

  Scenario: Invalid key is rejected and DTC set
    Given the DiagnosticSecurityLayer is active
    When an incorrect key is provided in the challenge-response
    Then access is denied within 200 ms
    And security DTC U3101 is set in DTC memory

  Scenario: Three failures trigger lockout
    Given the DiagnosticSecurityLayer is active
    When three consecutive invalid keys are submitted
    Then the programming security level is locked out for 10 minutes
    And further authentication attempts are rejected during the lockout period
```

---
type: TestCase
id: TC-ENG-SEC-003
title: Firmware signature verification rejects unsigned and rollback images
status: active
testLevel: L4
verifies:
  - REQ-ENG-SEC-003
---

```gherkin
Feature: Firmware update security — signature verification and rollback prevention

  Scenario: Firmware image with invalid ECDSA signature is rejected
    Given the Engine ECU bootloader is in programming mode via OBD-II
    And the OTP memory holds the OEM root CA public key
    When a firmware image with an invalid ECDSA P-256 signature is transmitted via UDS 0x36
    Then the bootloader rejects the image before programming flash
    And the bootloader returns UDS Negative Response Code 0x22 (conditionsNotCorrect)
    And a security DTC is set within 200 ms of the rejection

  Scenario: Signed firmware image with version lower than OTP counter is rejected
    Given the Engine ECU bootloader is in programming mode via OBD-II
    And the OTP monotonic version counter holds value N
    And a valid ECDSA P-256 signed firmware image exists with version number N-1
    When the rollback image is transmitted via UDS 0x36
    Then the bootloader verifies the signature as cryptographically valid
    But the bootloader rejects the image due to version counter mismatch
    And the bootloader returns UDS Negative Response Code 0x22 (conditionsNotCorrect)
    And a security DTC is set within 200 ms of the rejection
    And the OTP counter value remains unchanged at N
```

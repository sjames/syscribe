---
type: TestCase
id: TC-ENG-SEC-001
name: Integration test — CAN security module rejects frames with invalid MAC
status: active
testLevel: L2
verifies:
  - REQ-ENG-SEC-001
---

```gherkin
Feature: CAN message authentication

  Scenario: Valid MAC frame is accepted
    Given the CAN security module is initialised with the shared key
    When a safety-critical CAN frame with a valid CMAC-AES-128 MAC is received
    Then the frame is accepted and passed to the application layer

  Scenario: Invalid MAC frame is rejected and DTC set
    Given the CAN security module is running
    When a CAN frame with a corrupted MAC is received
    Then the frame is rejected within 50 ms
    And security DTC U3100 is set in the DTC memory

  Scenario: Replayed frame with stale freshness counter is rejected
    Given a valid frame with freshness counter N has been accepted
    When the same frame is replayed with freshness counter N
    Then the frame is rejected as a replay attack
    And a security DTC is set
```

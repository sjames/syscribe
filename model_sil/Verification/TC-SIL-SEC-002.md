---
type: TestCase
id: TC-SIL-SEC-002
name: Integration — EN 50159 Category 2 safety codes reject all defined fault classes on field bus
status: active
testLevel: L4
verifies:
  - REQ-SIL-SEC-002
---

```gherkin
Feature: EN 50159 Category 2 safety codes protect field bus commands against replay and insertion

  Scenario: Field bus message with inverted CRC bit is rejected
    Given the SafetyCommLayer is transmitting point-movement commands to ObjectController 1
    When a single bit is flipped in the CRC field of a captured frame using the fault injector
    And the corrupted frame is forwarded to ObjectController 1
    Then ObjectController 1 shall detect the CRC mismatch
    And ObjectController 1 shall not execute the point-movement command
    And a "CRC error" diagnostic event shall be raised within one field bus cycle (≤ 20 ms)
    And the points machine shall remain in its previous position

  Scenario: Replayed valid field bus command is rejected by sequence counter check
    Given a valid point-movement command frame has been captured by the fault injector on the field bus
    And the SafetyCommLayer has since transmitted at least one subsequent command (incrementing the sequence counter)
    When the captured (stale) frame is re-injected onto the field bus
    Then ObjectController 1 shall detect the out-of-sequence message (sequence counter is lower than expected)
    And ObjectController 1 shall not execute the replayed command
    And a "sequence counter violation" diagnostic event shall be raised

  Scenario: Field bus command with future timestamp outside the acceptance window is rejected
    Given the clocks on the vital processor and ObjectController 1 are synchronised to within ± 1 ms via IEEE 1588 PTP
    When a command frame is received with a timestamp more than 100 ms in the future relative to the receiver's current time
    Then ObjectController 1 shall reject the frame as "timestamp out of window"
    And ObjectController 1 shall not execute the command
    And a "timestamp violation" diagnostic event shall be raised
    And the field bus shall continue to operate normally for subsequent in-window commands

  Scenario: Command from unexpected source address is rejected
    Given ObjectController 1 is configured to accept commands only from VitalProcessor Channel A and Channel B addresses
    When a command frame is received claiming the source address of a non-vital diagnostic workstation
    Then ObjectController 1 shall reject the frame due to source address mismatch
    And a "source address violation — possible insertion attack" security event shall be raised and forwarded to the maintainer workstation log
```

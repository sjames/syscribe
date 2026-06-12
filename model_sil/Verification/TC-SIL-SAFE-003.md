---
type: TestCase
id: TC-SIL-SAFE-003
name: Integration test — conflict checker blocks simultaneous conflicting route requests
status: active
testLevel: L4
verifies:
  - REQ-SIL-SAFE-001
---

```gherkin
Feature: Route conflict prevention

  Scenario: Two conflicting routes cannot both be set
    Given route R1 (platform 1 entry from down direction) is set and locked
    When the signaller requests route R2 (platform 1 entry from up direction, conflicting with R1)
    Then the RouteProcessor rejects R2 with reason "conflicting route active"
    And the rejection is logged with both route identifiers
    And R1 remains set without any modification

  Scenario: Conflict matrix is enforced after channel restart
    Given one vital channel has been restarted (e.g., after a power cycle)
    When the restarted channel completes its initialisation self-test
    Then the channel loads the conflict matrix from flash and verifies its CRC before accepting any route requests
    And no route request is processed until the CRC check passes
```

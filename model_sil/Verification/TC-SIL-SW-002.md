---
type: TestCase
id: TC-SIL-SW-002
title: Integration test — EN 50159 Category 2 safety codes reject all defined fault classes
status: active
testLevel: L4
verifies:
  - REQ-SIL-SW-004
---

```gherkin
Feature: Safety communication fault detection

  Scenario: CRC-corrupted message is rejected
    Given a valid safety message is in transit on the cross-comparison bus
    When a single-bit flip is injected into the message payload by a fault injector
    Then the receiving channel's SafetyCommLayer rejects the message
    And no action is taken on the corrupted payload
    And a "communication integrity fault" is logged

  Scenario: Replayed old message is detected and rejected
    Given a valid safety message with sequence number N was sent 5 seconds ago
    When the same message with sequence number N is injected again
    Then the SafetyCommLayer detects the stale sequence number and rejects the message
    And the rejection is logged as a "replay attack / sequence error"

  Scenario: Message outside safety time window is rejected
    Given the configured safety time window is 150ms
    When a valid message arrives with a timestamp 300ms older than the receiver's clock
    Then the SafetyCommLayer rejects the message as "timestamp out of window"
```

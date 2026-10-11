---
type: TestCase
id: TC-GTI-MCU-002
name: "Software self-test"
status: active
testLevel: L3
verifies:
  - REQ-GTI-STK-007
  - REQ-GTI-SYS-071
---

```gherkin
Feature: Self-test

  Scenario: failure forces safe state
    Given a fault is injected into RAM, flash CRC or the clock
    When the self-test runs
    Then the safe state is entered within 100 ms
```
